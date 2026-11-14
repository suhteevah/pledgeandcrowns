// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP middleware: request metrics, client-IP derivation, per-IP rate
//! limiting, bearer auth and the pre-buffer body-size check.
//!
//! Layer order (outermost first), assembled in [`crate::build_router`]:
//!
//! ```text
//! TraceLayer -> record_metrics -> CORS -> global_rate_limit
//!   -> [protected routes] content_length_limit -> require_bearer
//!        -> [/compile-real only] real_rate_limit -> handler
//! ```
//!
//! CORS sits outside the limiter so a browser can read a 429. The global
//! limiter sits outside auth so token guessing is throttled too.

use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, MatchedPath, Request, State};
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use subtle::ConstantTimeEq;

use crate::metrics::RejectReason;
use crate::rate_limit::{Decision, RateLimiter};
use crate::{AppState, MAX_BODY_BYTES, Rejection};

/// Header consulted when `trust_proxy` is on.
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";

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

fn request_ip(state: &AppState, req: &Request) -> Result<IpAddr, Rejection> {
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| *addr);
    client_ip(req.headers(), peer, state.config().trust_proxy).ok_or_else(|| {
        tracing::error!(
            "client address unavailable: serve with into_make_service_with_connect_info \
             (use pledge_compile_api::serve)"
        );
        Rejection::ClientAddrUnavailable
    })
}

fn apply_limit(
    state: &AppState,
    limiter: &RateLimiter,
    name: &'static str,
    req: &Request,
) -> Result<(), Rejection> {
    let ip = request_ip(state, req)?;
    match limiter.check(ip) {
        Decision::Allow => Ok(()),
        Decision::Deny { retry_after_secs } => {
            tracing::warn!(
                limiter = name,
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

/// Per-IP budget across every route (design/05 §2: 600/min).
pub async fn global_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    match apply_limit(&state, state.global_limiter(), "global", &req) {
        Ok(()) => next.run(req).await,
        Err(r) => r.into_response(),
    }
}

/// Tighter per-IP budget on `/compile-real` (default 20/min).
pub async fn real_rate_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    match apply_limit(&state, state.real_limiter(), "compile-real", &req) {
        Ok(()) => next.run(req).await,
        Err(r) => r.into_response(),
    }
}

/// Bearer auth for the protected routes. A no-op when no token is
/// configured (the config layer guarantees that only happens on a
/// loopback bind). Every configured token is compared in constant time
/// and the loop never short-circuits, so timing reveals neither which
/// token matched nor how many bytes of a guess were right. (Token
/// *length* can leak through `ct_eq`'s length check; tokens are >= 32
/// random chars so that reveals nothing useful.)
pub async fn require_bearer(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let tokens = &state.config().auth_tokens;
    if tokens.is_empty() {
        return next.run(req).await;
    }
    let presented = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .map(str::trim);

    let Some(presented) = presented else {
        tracing::warn!(path = req.uri().path(), "rejected: missing bearer token");
        state.metrics().record_rejection(RejectReason::Auth);
        return Rejection::Unauthorized.into_response();
    };

    let mut matched = subtle::Choice::from(0u8);
    for t in tokens {
        matched |= t.expose().ct_eq(presented.as_bytes());
    }
    if bool::from(matched) {
        next.run(req).await
    } else {
        tracing::warn!(path = req.uri().path(), "rejected: wrong bearer token");
        state.metrics().record_rejection(RejectReason::Auth);
        Rejection::Unauthorized.into_response()
    }
}

/// Refuse a declared-oversize body before a single byte is buffered.
/// Chunked bodies with no `Content-Length` are still capped while
/// streaming by `DefaultBodyLimit` on the same routes.
pub async fn content_length_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let declared = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    if let Some(len) = declared
        && len > MAX_BODY_BYTES as u64
    {
        tracing::warn!(
            path = req.uri().path(),
            declared = len,
            limit = MAX_BODY_BYTES,
            "rejected: declared body too large"
        );
        state.metrics().record_rejection(RejectReason::Body);
        return Rejection::BodyTooLarge.into_response();
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

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
}
