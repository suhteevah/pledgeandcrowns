// SPDX-License-Identifier: AGPL-3.0-or-later
//! Library surface for the compile-API. Exposes the router builder,
//! configuration and the graders so integration tests drive the same
//! code paths the binary uses.
//!
//! # Request path
//!
//! - `GET /health` — open, and never rate limited.
//! - `POST /compile` — pattern grader; cheap, runs on the async path.
//! - `POST /compile-real` — real `cargo build` (in the build sandbox, see
//!   [`build_sandbox`]) + sandboxed wasm run. Source is first checked
//!   against the compile-time file-read deny-list ([`source_policy`]).
//!   Blocking work runs on `spawn_blocking` behind a build semaphore
//!   (`PNC_MAX_CONCURRENT_BUILDS`) with a bounded wait queue
//!   (`PNC_MAX_QUEUED_BUILDS`); overflow gets 503 + `Retry-After`.
//! - `GET /metrics` — Prometheus text format. Its own token
//!   (`PNC_METRICS_TOKENS`) or direct-loopback only; player tokens are
//!   not accepted.
//!
//! `/compile` and `/compile-real` require `Authorization: Bearer <token>`
//! whenever a token is configured. See [`middleware`] for layer order and
//! [`config`] for every knob.

pub mod build_root;
pub mod build_sandbox;
pub mod cargo_grader;
pub mod config;
pub mod grader;
pub mod metrics;
pub mod middleware;
pub mod proc_tree;
pub mod rate_limit;
pub mod source_policy;
pub mod wasm_builder;
pub mod wasm_runner;

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub use config::{Config, ConfigError};
use metrics::{Metrics, RejectReason};
use rate_limit::{OnFull, RateLimiter};
use wasm_builder::{BuildSettings, RealOutcome, RunLimits};

/// Hard cap on player source bytes accepted by `/compile` and
/// `/compile-real`. The pattern grader is fast for any input but the
/// cargo path hands the buffer to rustc's parser; an unbounded body
/// is a cheap DoS surface (10 MB submissions exhaust ram + cpu in the
/// parser before any sandbox limit kicks in). 64 KiB is roughly
/// 10x the largest reasonable mission solution and an order of
/// magnitude under any size that meaningfully stresses the parser.
pub const MAX_SOURCE_BYTES: usize = 64 * 1024;

/// Transport-level cap on the JSON request body, enforced before the
/// body is buffered: the 64 KiB source cap plus 4 KiB for the JSON
/// envelope (`encounter_id`, keys, quotes). Because JSON escaping only
/// ever *grows* a string, every body that fits here decodes to a source
/// that is checked again against [`MAX_SOURCE_BYTES`]; a source just
/// under 64 KiB that is escape-heavy can be refused here first, which is
/// a tightening, never a loosening.
pub const MAX_BODY_BYTES: usize = MAX_SOURCE_BYTES + 4 * 1024;

/// `Retry-After` sent with a 503 when the build queue is full. Roughly
/// one warm build.
pub const QUEUE_RETRY_AFTER_SECS: u64 = 5;

/// Longest accepted `encounter_id`, in bytes.
pub const MAX_ENCOUNTER_ID_BYTES: usize = 64;

/// `encounter_id` must be 1..=64 bytes of `[A-Za-z0-9_-]`. Checked before
/// the value is logged, echoed or used, so it can carry neither log
/// injection nor a path.
pub fn valid_encounter_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ENCOUNTER_ID_BYTES
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[derive(Deserialize)]
pub struct CompileRequest {
    pub source: String,
    pub encounter_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompileResponse {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Body of every refusal (401/413/415/422/429/500/503). A superset of
/// [`CompileResponse`] so the game client, which deserializes
/// `{ok, stdout, stderr}`, can show `stderr` without special-casing, while
/// `error` / `retry_after_seconds` follow the design/05 §2 429 shape.
#[derive(Serialize, Deserialize, Debug)]
pub struct RejectionBody {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub retry_after_seconds: Option<u64>,
}

/// Request refusals, each mapped to one status code and JSON body.
#[derive(Debug, thiserror::Error)]
pub enum Rejection {
    #[error("missing or invalid bearer token")]
    Unauthorized,
    #[error("rate limit exceeded; retry in {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("build queue is full; retry in {retry_after_secs}s")]
    QueueFull { retry_after_secs: u64 },
    #[error("request body exceeds limit of {MAX_BODY_BYTES} bytes")]
    BodyTooLarge,
    #[error("{message}")]
    BadRequest { status: StatusCode, message: String },
    #[error("{0}")]
    Forbidden(String),
    /// Player source hit the compile-time file-read deny-list. The
    /// message is the player-facing text from
    /// [`source_policy::PolicyViolation::player_message`].
    #[error("{0}")]
    SourcePolicy(String),
    #[error("client address unavailable")]
    ClientAddrUnavailable,
    #[error("internal error: {0}")]
    Internal(String),
}

impl Rejection {
    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::QueueFull { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::BadRequest { status, .. } => *status,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::SourcePolicy(_) => StatusCode::BAD_REQUEST,
            Self::ClientAddrUnavailable | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::RateLimited { .. } => "rate_limited",
            Self::QueueFull { .. } => "queue_full",
            Self::BodyTooLarge => "body_too_large",
            Self::BadRequest { .. } => "bad_request",
            Self::Forbidden(_) => "forbidden",
            Self::SourcePolicy(_) => "source_policy",
            Self::ClientAddrUnavailable | Self::Internal(_) => "internal",
        }
    }

    fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_secs } | Self::QueueFull { retry_after_secs } => {
                Some(*retry_after_secs)
            }
            _ => None,
        }
    }
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        let status = self.status();
        let retry = self.retry_after();
        let body = RejectionBody {
            ok: false,
            stdout: String::new(),
            stderr: self.to_string(),
            error: self.code().to_string(),
            retry_after_seconds: retry,
        };
        let mut resp = (status, Json(body)).into_response();
        if let Some(secs) = retry {
            resp.headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(secs));
        }
        if matches!(self, Self::Unauthorized) {
            resp.headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        resp
    }
}

/// Blocking build-and-run function. Production uses
/// [`wasm_builder::compile_and_run_with`]; tests inject a stub so the
/// concurrency/queue path can be exercised without a toolchain.
pub type RealGrader = Arc<dyn Fn(&str, &RunLimits) -> anyhow::Result<RealOutcome> + Send + Sync>;

struct Inner {
    config: Config,
    metrics: Metrics,
    global_limiter: RateLimiter,
    real_limiter: RateLimiter,
    builds: Arc<Semaphore>,
    queued: AtomicUsize,
    real_grader: RealGrader,
}

/// Shared server state. Cheap to clone.
#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl AppState {
    /// Production state: real `cargo build` (in the configured build
    /// sandbox, under the configured build root) + wasmtime grader.
    pub fn new(config: Config) -> Self {
        let settings = BuildSettings::from_config(&config);
        let grader: RealGrader = Arc::new(move |source: &str, limits: &RunLimits| {
            wasm_builder::compile_and_run_in(source, limits, &settings)
        });
        Self::with_real_grader(config, grader)
    }

    /// State with a custom blocking grader for `/compile-real`.
    pub fn with_real_grader(config: Config, real_grader: RealGrader) -> Self {
        // Global limiter fails open under table pressure; the expensive
        // route evicts least-recently-seen buckets instead (it is also
        // bounded by the build semaphore + queue). See `rate_limit`.
        let global_limiter = RateLimiter::with_max_entries(
            "global",
            config.rate_limit_per_min,
            config.rate_limit_max_clients,
            OnFull::FailOpen,
        );
        let real_limiter = RateLimiter::with_max_entries(
            "compile-real",
            config.real_rate_limit_per_min,
            config.rate_limit_max_clients,
            OnFull::EvictLru,
        );
        let builds = Arc::new(Semaphore::new(config.max_concurrent_builds));
        Self(Arc::new(Inner {
            config,
            metrics: Metrics::new(),
            global_limiter,
            real_limiter,
            builds,
            queued: AtomicUsize::new(0),
            real_grader,
        }))
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

    pub fn metrics(&self) -> &Metrics {
        &self.0.metrics
    }

    pub(crate) fn global_limiter(&self) -> &RateLimiter {
        &self.0.global_limiter
    }

    pub(crate) fn real_limiter(&self) -> &RateLimiter {
        &self.0.real_limiter
    }

    fn run_limits(&self) -> RunLimits {
        RunLimits {
            max_memory_bytes: self.0.config.run_max_memory_bytes,
            ..RunLimits::default()
        }
    }
}

/// Router with dev-default configuration (loopback, no auth). Serve it
/// with [`serve`] so the per-IP limiter can see the peer address.
pub fn make_router() -> Router {
    build_router(AppState::new(Config::default()))
}

/// Assemble the full router for `state`. Layer order is documented in
/// [`middleware`].
pub fn build_router(state: AppState) -> Router {
    let cfg = state.config();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(cfg.cors_origins.clone()))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .max_age(Duration::from_secs(600));

    let player = Router::new()
        .route("/compile", post(compile))
        .route(
            "/compile-real",
            post(compile_real).route_layer(from_fn_with_state(
                state.clone(),
                middleware::real_rate_limit,
            )),
        )
        .route_layer(from_fn_with_state(state.clone(), middleware::limit_body))
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware::require_bearer,
        ))
        // Backstop only: `limit_body` has already enforced the cap.
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES));

    let metrics = Router::new()
        .route("/metrics", get(metrics_handler))
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware::require_metrics_access,
        ));

    let limited = player.merge(metrics).layer(from_fn_with_state(
        state.clone(),
        middleware::global_rate_limit,
    ));

    Router::new()
        .merge(limited)
        // Added after the limiter layer, so /health is never limited.
        .route("/health", get(health))
        .layer(cors)
        .layer(from_fn_with_state(
            state.clone(),
            middleware::record_metrics,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serve `router` on `listener` with peer-address info attached (needed
/// by the per-IP rate limiter).
pub async fn serve(listener: tokio::net::TcpListener, router: Router) -> std::io::Result<()> {
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("metrics: scrape");
    let limiters = [
        ("global", state.global_limiter().stats()),
        ("compile-real", state.real_limiter().stats()),
    ];
    (
        [(header::CONTENT_TYPE, metrics::CONTENT_TYPE)],
        state.metrics().render_with_limiters(&limiters),
    )
}

/// Refuse a malformed `encounter_id` before it is logged or used.
fn reject_bad_encounter(state: &AppState, route: &str, id: &str) -> Option<Response> {
    if valid_encounter_id(id) {
        return None;
    }
    // Log the length only: the value is attacker-controlled.
    tracing::warn!(
        route,
        encounter_id_bytes = id.len(),
        "rejected: invalid encounter_id"
    );
    state.metrics().record_rejection(RejectReason::Input);
    Some(
        Rejection::BadRequest {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "invalid encounter_id: expected 1-{MAX_ENCOUNTER_ID_BYTES} characters of \
                 [A-Za-z0-9_-]"
            ),
        }
        .into_response(),
    )
}

/// Map an extractor failure to a JSON refusal. A 413 from the streaming
/// body limit counts as a body rejection.
fn json_rejection(state: &AppState, route: &str, rej: JsonRejection) -> Response {
    let status = rej.status();
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        tracing::warn!(route, "rejected: streamed body exceeded limit");
        state.metrics().record_rejection(RejectReason::Body);
        return Rejection::BodyTooLarge.into_response();
    }
    tracing::warn!(route, %status, "rejected: bad request body: {}", rej.body_text());
    Rejection::BadRequest {
        status,
        message: rej.body_text(),
    }
    .into_response()
}

async fn compile(
    State(state): State<AppState>,
    payload: Result<Json<CompileRequest>, JsonRejection>,
) -> Response {
    let req = match payload {
        Ok(Json(req)) => req,
        Err(rej) => return json_rejection(&state, "/compile", rej),
    };
    if let Some(r) = reject_bad_encounter(&state, "/compile", &req.encounter_id) {
        return r;
    }
    tracing::info!(
        encounter = %req.encounter_id,
        source_bytes = req.source.len(),
        "compile: request"
    );
    if let Some(rejection) = reject_invalid_source(&req.source) {
        return rejection.into_response();
    }
    let v = grader::grade(&req.encounter_id, &req.source);
    tracing::debug!(encounter = %req.encounter_id, ok = v.ok, "compile: graded");
    (
        StatusCode::OK,
        Json(CompileResponse {
            ok: v.ok,
            stdout: v.stdout,
            stderr: v.stderr,
        }),
    )
        .into_response()
}

/// Decrements the queued counter + gauge when a waiter leaves the queue
/// (permit acquired, rejected, or the client disconnected mid-wait).
struct QueueSlot<'a> {
    state: &'a AppState,
}

impl<'a> QueueSlot<'a> {
    /// Join the queue; returns the slot and the depth *before* joining.
    fn enter(state: &'a AppState) -> (Self, usize) {
        let prev = state.0.queued.fetch_add(1, Ordering::SeqCst);
        state.metrics().builds_queued_add(1);
        (Self { state }, prev)
    }
}

impl Drop for QueueSlot<'_> {
    fn drop(&mut self) {
        self.state.0.queued.fetch_sub(1, Ordering::SeqCst);
        self.state.metrics().builds_queued_add(-1);
    }
}

/// Keeps `pnc_builds_in_flight` accurate across every exit of the
/// blocking build, including a panic.
struct InFlight(AppState);

impl InFlight {
    fn enter(state: AppState) -> Self {
        state.metrics().builds_in_flight_add(1);
        Self(state)
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        self.0.metrics().builds_in_flight_add(-1);
    }
}

/// Take a build slot, waiting in a bounded queue if all slots are busy.
async fn acquire_build_slot(state: &AppState) -> Result<OwnedSemaphorePermit, Rejection> {
    match state.0.builds.clone().try_acquire_owned() {
        Ok(permit) => return Ok(permit),
        Err(TryAcquireError::Closed) => {
            return Err(Rejection::Internal("build semaphore closed".to_string()));
        }
        Err(TryAcquireError::NoPermits) => {}
    }
    let max_queued = state.config().max_queued_builds;
    let (_slot, depth) = QueueSlot::enter(state);
    if depth >= max_queued {
        tracing::warn!(depth, max_queued, "rejected: build queue full");
        state.metrics().record_rejection(RejectReason::Queue);
        return Err(Rejection::QueueFull {
            retry_after_secs: QUEUE_RETRY_AFTER_SECS,
        });
    }
    tracing::debug!(depth = depth + 1, "compile-real: queued for a build slot");
    state
        .0
        .builds
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| Rejection::Internal("build semaphore closed".to_string()))
}

/// Real build-and-run route. Slow (build ~1-3s warm + a sub-second
/// sandboxed run) — available but not the game client's default yet
/// (see `compile_client.rs`; the prod switch is gated on a wasip1
/// toolchain being installed on the server). Same request/response shape
/// as `/compile` so the client toggle is a one-line change.
///
/// Honest verdict: the player source is built to `wasm32-wasip1` and the
/// resulting module is *actually executed* under the hardened sandbox in
/// [`wasm_runner`]. `ok` means it compiled AND ran to a clean exit. The
/// program's real stdout is returned. Tags every response with a leading
/// `[real]` marker so a player or test can tell which path produced the
/// verdict; the marker drops when this route becomes the default.
///
/// The build never runs on the async runtime: it moves to
/// `spawn_blocking` holding a semaphore permit, and the permit is held by
/// the blocking task itself, so a client that disconnects mid-build does
/// not free the slot before the build actually finishes.
async fn compile_real(
    State(state): State<AppState>,
    payload: Result<Json<CompileRequest>, JsonRejection>,
) -> Response {
    let req = match payload {
        Ok(Json(req)) => req,
        Err(rej) => return json_rejection(&state, "/compile-real", rej),
    };
    if let Some(r) = reject_bad_encounter(&state, "/compile-real", &req.encounter_id) {
        return r;
    }
    tracing::info!(
        encounter = %req.encounter_id,
        source_bytes = req.source.len(),
        "compile-real: request"
    );
    if let Some(rejection) = reject_invalid_source(&req.source) {
        return rejection.into_response();
    }
    // Second layer against compile-time file reads; the build sandbox is
    // the first. Checked before a build slot is taken.
    if state.config().source_denylist {
        if let Err(violation) = source_policy::check(&req.source) {
            tracing::warn!(
                encounter = %req.encounter_id,
                kind = violation.kind(),
                "rejected: source policy: {violation}"
            );
            state.metrics().record_rejection(RejectReason::SourcePolicy);
            return Rejection::SourcePolicy(violation.player_message()).into_response();
        }
    } else {
        tracing::warn!("compile-real: source deny-list is DISABLED (PNC_SOURCE_DENYLIST=false)");
    }

    let permit = match acquire_build_slot(&state).await {
        Ok(p) => p,
        Err(r) => return r.into_response(),
    };

    let grader = state.0.real_grader.clone();
    let limits = state.run_limits();
    let task_state = state.clone();
    let source = req.source;
    let joined = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let _in_flight = InFlight::enter(task_state);
        grader(&source, &limits)
    })
    .await;

    // Honest end-to-end: build to wasm, then run it in the sandbox. We do
    // NOT stitch in pattern-grader flavor — the two answer different
    // questions and a stitched response can be self-contradictory
    // (ok=true from compile, "missing required: let mut" from pattern).
    // The execution layer carries the real verdict; per-encounter
    // expected-output checks layer on top of this in a later pass.
    let outcome = match joined {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(e)) => {
            tracing::error!("compile-real: wasm_builder setup failure: {e:#}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CompileResponse {
                    ok: false,
                    stdout: "[real]".to_string(),
                    stderr: format!("{e:#}"),
                }),
            )
                .into_response();
        }
        Err(join_err) => {
            tracing::error!("compile-real: build task failed: {join_err}");
            return Rejection::Internal("build task failed".to_string()).into_response();
        }
    };

    state.metrics().observe_build(outcome.build_time);
    if let Some(run) = outcome.run_time {
        state.metrics().observe_run(run);
    }
    tracing::info!(
        encounter = %req.encounter_id,
        compiled = outcome.compiled,
        ran_ok = outcome.ran_ok,
        build_ms = outcome.build_time.as_millis() as u64,
        run_ms = outcome.run_time.map(|d| d.as_millis() as u64),
        "compile-real: verdict"
    );

    let ok = outcome.ok();
    let stdout = if ok {
        // Surface the program's real output, marked [real].
        if outcome.stdout.is_empty() {
            "[real] compiled + ran: ok".to_string()
        } else {
            format!("[real] {}", outcome.stdout)
        }
    } else {
        "[real]".to_string()
    };
    (
        StatusCode::OK,
        Json(CompileResponse {
            ok,
            stdout,
            stderr: outcome.stderr,
        }),
    )
        .into_response()
}

/// Shared rejection logic for the two compile routes. Returns `Some`
/// with a 400 response if the source is empty/whitespace or exceeds
/// the byte cap; `None` if the source is acceptable.
fn reject_invalid_source(source: &str) -> Option<(StatusCode, Json<CompileResponse>)> {
    if source.trim().is_empty() {
        tracing::warn!("rejected: empty source");
        return Some((
            StatusCode::BAD_REQUEST,
            Json(CompileResponse {
                ok: false,
                stdout: String::new(),
                stderr: "empty source".to_string(),
            }),
        ));
    }
    if source.len() > MAX_SOURCE_BYTES {
        tracing::warn!(source_bytes = source.len(), "rejected: source over cap");
        return Some((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(CompileResponse {
                ok: false,
                stdout: String::new(),
                stderr: format!(
                    "source exceeds limit of {} bytes (got {})",
                    MAX_SOURCE_BYTES,
                    source.len()
                ),
            }),
        ));
    }
    None
}
