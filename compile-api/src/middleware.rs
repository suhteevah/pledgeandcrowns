// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP middleware: request metrics, client-IP derivation, per-IP rate
//! limiting, bearer auth, `/metrics` access, and the request-body cap.
//!
//! Layer order (outermost first), assembled in [`crate::build_router`]:
//!
//! ```text
//! TraceLayer -> record_metrics -> CORS
//!   -> /health                         (never rate limited)
//!   -> request_deadline -> global_rate_limit
//!        -> /compile, /compile-real:   limit_body -> require_bearer
//!             -> [/compile-real only] real_rate_limit -> handler
//!
//! metrics listener (PNC_METRICS_BIND, loopback only, separate router):
//!   TraceLayer -> /metrics: require_metrics_access -> handler
//! ```
//!
//! CORS sits outside the limiter so a browser can read a 429. The global
//! limiter sits outside auth so token guessing is throttled too. `/health`
//! is mounted outside the limiter so it can never be rate limited into
//! unavailability (it does no work and touches no shared state).
//! `/metrics` is not on the public router at all (re-review P2).

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use axum::body::{Body, Bytes};
use axum::extract::{ConnectInfo, MatchedPath, Request, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http_body_util::BodyExt;
use subtle::ConstantTimeEq;

use crate::config::Secret;
use crate::metrics::RejectReason;
use crate::rate_limit::{Decision, RateLimiter};
use crate::{AppState, MAX_BODY_BYTES, Rejection};

/// Header consulted when `trust_proxy` is on.
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";

/// After a body exceeds [`MAX_BODY_BYTES`], keep reading (and discarding)
/// up to this many more bytes before answering 413, so a client that is
/// still uploading has finished and actually receives the response.
/// Closing a socket with unread input makes the kernel send a TCP reset,
/// and the client then usually loses the 413.
pub const BODY_DRAIN_MAX_BYTES: usize = 1024 * 1024;

/// Upper bound on time spent draining an oversized body.
pub const BODY_DRAIN_MAX_TIME: Duration = Duration::from_secs(2);

/// Count every response by route template + status. Unmatched paths are
/// folded into one `"unmatched"` series so scanners cannot blow up label
/// cardinality.
pub async fn record_metrics(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "unmatched".to_string());
    let resp = next.run(req).await;
    state
        .metrics()
        .record_request(&route, resp.status().as_u16());
    resp
}

/// Derive the client IP used for rate limiting.
///
/// - Default: the TCP peer address (`ConnectInfo<SocketAddr>`).
/// - With `trust_proxy`: the **right-most** `X-Forwarded-For` entry,
///   i.e. the address the nearest (trusted) proxy saw. Left-hand entries
///   are client-supplied and ignored. Falls back to the peer if the
///   header is absent or unparsable.
pub fn client_ip(
    headers: &HeaderMap,
    peer: Option<SocketAddr>,
    trust_proxy: bool,
) -> Option<IpAddr> {
    if trust_proxy {
        let forwarded = headers
            .get_all(X_FORWARDED_FOR)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(','))
            .map(str::trim)
            .rfind(|s| !s.is_empty());
        if let Some(raw) = forwarded {
            match raw.parse::<IpAddr>() {
                Ok(ip) => return Some(ip),
                Err(_) => tracing::debug!(
                    "client_ip: unparsable X-Forwarded-For entry {raw:?}, using peer"
                ),
            }
        }
    }
    peer.map(|p| p.ip())
}

fn peer_addr(req: &Request) -> Option<SocketAddr> {
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| *addr)
}

fn request_ip(state: &AppState, req: &Request) -> Result<IpAddr, Rejection> {
    client_ip(req.headers(), peer_addr(req), state.config().trust_proxy).ok_or_else(|| {
        tracing::error!(
            "client address unavailable: serve with into_make_service_with_connect_info \
             (use pledge_compile_api::serve)"
        );
        Rejection::ClientAddrUnavailable
    })
}

fn apply_limit(state: &AppState, limiter: &RateLimiter, req: &Request) -> Result<(), Rejection> {
    let ip = request_ip(state, req)?;
    match limiter.check(ip) {
        Decision::Allow => Ok(()),
        Decision::FailOpen => {
            state.metrics().record_rate_fail_open();
            Ok(())
        }
        Decision::Deny { retry_after_secs } => {
            tracing::warn!(
                limiter = limiter.name(),
                %ip,
                path = req.uri().path(),
                retry_after_secs,
                "rejected: rate limited"
            );
            state.metrics().record_rejection(RejectReason::Rate);
            Err(Rejection::RateLimited { retry_after_secs })
        }
    }
}

/// Per-IP budget across every route except `/health` (design/05 §2:
/// 600/min). Fails open under table pressure (see [`crate::rate_limit`]).
pub async fn global_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    match apply_limit(&state, state.global_limiter(), &req) {
        Ok(()) => next.run(req).await,
        Err(r) => r.into_response(),
    }
}

/// Tighter per-IP budget on `/compile-real` (default 20/min).
pub async fn real_rate_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    match apply_limit(&state, state.real_limiter(), &req) {
        Ok(()) => next.run(req).await,
        Err(r) => r.into_response(),
    }
}

fn presented_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .map(str::trim)
}

/// Constant-time match against every token; never short-circuits, so
/// timing reveals neither which token matched nor how many bytes of a
/// guess were right. (Token *length* can leak through `ct_eq`'s length
/// check; tokens are >= 32 random chars so that reveals nothing useful.)
fn token_matches(tokens: &[Secret], presented: &str) -> bool {
    let mut matched = subtle::Choice::from(0u8);
    for t in tokens {
        matched |= t.expose().ct_eq(presented.as_bytes());
    }
    bool::from(matched)
}

/// Bearer auth for the player routes. A no-op when no token is configured
/// (the config layer guarantees that only happens on a loopback bind).
pub async fn require_bearer(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let tokens = &state.config().auth_tokens;
    if tokens.is_empty() {
        return next.run(req).await;
    }
    let Some(presented) = presented_bearer(req.headers()) else {
        tracing::warn!(path = req.uri().path(), "rejected: missing bearer token");
        state.metrics().record_rejection(RejectReason::Auth);
        return Rejection::Unauthorized.into_response();
    };
    if token_matches(tokens, presented) {
        next.run(req).await
    } else {
        tracing::warn!(path = req.uri().path(), "rejected: wrong bearer token");
        state.metrics().record_rejection(RejectReason::Auth);
        Rejection::Unauthorized.into_response()
    }
}

/// True when the request came straight from a loopback peer, not through
/// the reverse proxy. With `trust_proxy` on, every proxied request also
/// has a loopback peer, so the presence of `X-Forwarded-For` marks it as
/// external.
pub fn is_direct_loopback(
    headers: &HeaderMap,
    peer: Option<SocketAddr>,
    trust_proxy: bool,
) -> bool {
    let loopback_peer = peer.is_some_and(|p| p.ip().is_loopback());
    let proxied = trust_proxy && headers.contains_key(X_FORWARDED_FOR);
    loopback_peer && !proxied
}

/// `/metrics` access. Player tokens are deliberately NOT accepted.
///
/// - `PNC_METRICS_TOKENS` set: a matching metrics bearer token is required.
/// - unset: only direct loopback clients (see [`is_direct_loopback`]).
pub async fn require_metrics_access(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let cfg = state.config();
    if !cfg.metrics_tokens.is_empty() {
        let ok = presented_bearer(req.headers())
            .is_some_and(|presented| token_matches(&cfg.metrics_tokens, presented));
        if ok {
            return next.run(req).await;
        }
        tracing::warn!("rejected: /metrics without a valid metrics token");
        state.metrics().record_rejection(RejectReason::Auth);
        return Rejection::Unauthorized.into_response();
    }
    if is_direct_loopback(req.headers(), peer_addr(&req), cfg.trust_proxy) {
        return next.run(req).await;
    }
    tracing::warn!("rejected: /metrics from a non-loopback client (no metrics token configured)");
    state.metrics().record_rejection(RejectReason::Auth);
    Rejection::Forbidden("metrics are only served to loopback clients".to_string()).into_response()
}

/// Whole-request deadline (`PNC_REQUEST_DEADLINE_SECS`) for the player
/// routes: queue wait, build and run included. On expiry the handler
/// future is dropped and the client gets 503 `deadline_exceeded`; a build
/// already running on the blocking pool keeps its build slot until it
/// really ends (see `compile_real`), so this frees the connection, never
/// the capacity.
pub async fn request_deadline(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let limit = state.config().request_deadline;
    let path = req.uri().path().to_string();
    match tokio::time::timeout(limit, next.run(req)).await {
        Ok(resp) => resp,
        Err(_) => {
            tracing::warn!(path, ?limit, "rejected: request deadline exceeded");
            state.metrics().record_rejection(RejectReason::Timeout);
            Rejection::DeadlineExceeded {
                secs: limit.as_secs(),
            }
            .into_response()
        }
    }
}

/// Read the whole request body with a hard cap of [`MAX_BODY_BYTES`] and a
/// time limit of `PNC_BODY_READ_TIMEOUT_SECS` for the whole body.
///
/// - A declared `Content-Length` over the cap is refused without
///   buffering anything.
/// - A body that turns out to be over the cap while streaming (chunked,
///   or a lying `Content-Length`) is refused as soon as it crosses it.
/// - A body that does not arrive in time (slow upload) gets 408 with
///   `Connection: close`.
///
/// In the size cases we then *drain* (read and discard) up to
/// [`BODY_DRAIN_MAX_BYTES`] / [`BODY_DRAIN_MAX_TIME`] more before sending
/// the 413 with `Connection: close`, so a client that is still uploading
/// reliably receives it. Draining is bounded, so it cannot be used to
/// hold the connection open indefinitely.
///
/// An accepted body is passed on fully buffered (at most 68 KiB).
pub async fn limit_body(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let limit = state.config().body_read_timeout;
    let path = req.uri().path().to_string();
    let read = tokio::time::timeout(limit, read_limited_body(&state, req)).await;
    match read {
        Ok(Ok(req)) => next.run(req).await,
        Ok(Err(resp)) => *resp,
        Err(_) => {
            tracing::warn!(path, ?limit, "rejected: request body not received in time");
            state.metrics().record_rejection(RejectReason::Timeout);
            Rejection::RequestTimeout {
                secs: limit.as_secs(),
            }
            .into_response()
        }
    }
}

/// Buffer the body under the size cap; `Err` is the refusal response.
async fn read_limited_body(state: &AppState, req: Request) -> Result<Request, Box<Response>> {
    let declared = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    let path = req.uri().path().to_string();
    let (parts, mut body) = req.into_parts();

    if let Some(len) = declared
        && len > MAX_BODY_BYTES as u64
    {
        tracing::warn!(
            path,
            declared = len,
            limit = MAX_BODY_BYTES,
            "rejected: declared body too large"
        );
        state.metrics().record_rejection(RejectReason::Body);
        if len <= (MAX_BODY_BYTES + BODY_DRAIN_MAX_BYTES) as u64 {
            drain(&mut body, BODY_DRAIN_MAX_BYTES + MAX_BODY_BYTES).await;
        }
        return Err(Box::new(body_too_large()));
    }

    let mut buf: Vec<u8> = Vec::with_capacity(declared.map_or(8 * 1024, |l| l as usize));
    loop {
        match body.frame().await {
            None => break,
            Some(Ok(frame)) => {
                let Ok(data) = frame.into_data() else {
                    continue; // trailers
                };
                if buf.len() + data.len() > MAX_BODY_BYTES {
                    tracing::warn!(
                        path,
                        limit = MAX_BODY_BYTES,
                        "rejected: streamed body exceeded limit"
                    );
                    state.metrics().record_rejection(RejectReason::Body);
                    drain(&mut body, BODY_DRAIN_MAX_BYTES).await;
                    return Err(Box::new(body_too_large()));
                }
                buf.extend_from_slice(&data);
            }
            Some(Err(e)) => {
                tracing::warn!(path, "rejected: error reading request body: {e}");
                return Err(Box::new(
                    Rejection::BadRequest {
                        status: axum::http::StatusCode::BAD_REQUEST,
                        message: "could not read request body".to_string(),
                    }
                    .into_response(),
                ));
            }
        }
    }
    Ok(Request::from_parts(parts, Body::from(Bytes::from(buf))))
}

/// Read and discard up to `max_bytes` of `body`, for at most
/// [`BODY_DRAIN_MAX_TIME`].
async fn drain(body: &mut Body, max_bytes: usize) {
    let mut drained = 0usize;
    let result = tokio::time::timeout(BODY_DRAIN_MAX_TIME, async {
        while drained < max_bytes {
            match body.frame().await {
                Some(Ok(frame)) => {
                    if let Some(d) = frame.data_ref() {
                        drained += d.len();
                    }
                }
                _ => return true,
            }
        }
        false
    })
    .await;
    match result {
        Ok(true) => tracing::debug!(drained, "body drain: reached end of body"),
        Ok(false) => tracing::debug!(drained, "body drain: byte cap reached"),
        Err(_) => tracing::debug!(drained, "body drain: time cap reached"),
    }
}

fn body_too_large() -> Response {
    let mut resp = Rejection::BodyTooLarge.into_response();
    resp.headers_mut()
        .insert(header::CONNECTION, HeaderValue::from_static("close"));
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer() -> Option<SocketAddr> {
        Some(SocketAddr::from(([127, 0, 0, 1], 5555)))
    }

    #[test]
    fn xff_ignored_without_trust_proxy() {
        let mut h = HeaderMap::new();
        h.insert(X_FORWARDED_FOR, HeaderValue::from_static("203.0.113.9"));
        assert_eq!(
            client_ip(&h, peer(), false),
            Some(IpAddr::from([127, 0, 0, 1]))
        );
    }

    #[test]
    fn xff_rightmost_entry_used_with_trust_proxy() {
        let mut h = HeaderMap::new();
        h.insert(
            X_FORWARDED_FOR,
            HeaderValue::from_static("1.1.1.1, 203.0.113.9"),
        );
        assert_eq!(
            client_ip(&h, peer(), true),
            Some(IpAddr::from([203, 0, 113, 9]))
        );
    }

    #[test]
    fn garbage_xff_falls_back_to_peer() {
        let mut h = HeaderMap::new();
        h.insert(X_FORWARDED_FOR, HeaderValue::from_static("not-an-ip"));
        assert_eq!(
            client_ip(&h, peer(), true),
            Some(IpAddr::from([127, 0, 0, 1]))
        );
    }

    #[test]
    fn direct_loopback_detection() {
        let empty = HeaderMap::new();
        let mut proxied = HeaderMap::new();
        proxied.insert(X_FORWARDED_FOR, HeaderValue::from_static("203.0.113.9"));
        let remote = Some(SocketAddr::from(([203, 0, 113, 9], 1)));
        assert!(is_direct_loopback(&empty, peer(), false));
        assert!(is_direct_loopback(&empty, peer(), true));
        assert!(!is_direct_loopback(&proxied, peer(), true));
        // Without trust_proxy an XFF header is client noise, ignored.
        assert!(is_direct_loopback(&proxied, peer(), false));
        assert!(!is_direct_loopback(&empty, remote, false));
        assert!(!is_direct_loopback(&empty, None, false));
    }
}
