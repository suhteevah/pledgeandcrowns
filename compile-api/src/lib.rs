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
//!   [`build_sandbox`]) + sandboxed wasm run, graded against the mission's
//!   expectation ([`expectations`]). Source is first checked against the
//!   compile-time file-read deny-list ([`source_policy`]).
//!   Blocking work runs on `spawn_blocking` behind a build semaphore
//!   (`PNC_MAX_CONCURRENT_BUILDS`) with a bounded wait queue
//!   (`PNC_MAX_QUEUED_BUILDS`); overflow gets 503 + `Retry-After`.
//! - `GET /metrics` — Prometheus text format, on a SEPARATE loopback-only
//!   listener (`PNC_METRICS_BIND`, [`build_metrics_router`]) that the
//!   reverse proxy never forwards to. Its own token (`PNC_METRICS_TOKENS`)
//!   or direct-loopback only; player tokens are not accepted.
//!
//! `/compile` and `/compile-real` require `Authorization: Bearer <token>`
//! whenever a token is configured. See [`middleware`] for layer order and
//! [`config`] for every knob.
//!
//! # Where player code runs ([`config::Executor`])
//!
//! - Production: the separate builder service ([`builder`]) over a Unix
//!   socket ([`ipc`]); this process holds the tokens and never builds or
//!   runs player code.
//! - Local dev / tests: in this process, or one child process per job
//!   ([`jobs`]).
//!
//! # Timeouts
//!
//! Request line + headers: `PNC_HEADER_READ_TIMEOUT_SECS` (hyper, see
//! [`serve_with`]); body: `PNC_BODY_READ_TIMEOUT_SECS` (408); whole
//! request including queue wait and build: `PNC_REQUEST_DEADLINE_SECS`
//! (503 `deadline_exceeded`). A build that is cut off by the request
//! deadline keeps its build slot until the build itself has ended.

pub mod build_root;
pub mod build_sandbox;
pub mod builder;
pub mod cargo_grader;
pub mod config;
pub mod expectations;
pub mod grader;
pub mod helper_threads;
pub mod ipc;
pub mod jobs;
pub mod metrics;
pub mod middleware;
pub mod proc_tree;
pub mod rate_limit;
pub mod run_sandbox;
pub mod source_policy;
pub mod toolchain;
pub mod wasm_builder;
pub mod wasm_runner;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::extract::rejection::JsonRejection;
use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub use config::{Config, ConfigError, Executor};
use expectations::{GradeState, GradedRun};
use ipc::JobRequest;
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

/// Body of a graded `/compile` or `/compile-real` response.
///
/// `ok`, `stdout`, `stderr` are the original contract and keep their
/// meaning for the existing client. Everything after them is additive and
/// optional: `/compile` never sets them (its JSON is unchanged), and
/// `/compile-real` always sets `grade`, `behaviour_checked` and `note`.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompileResponse {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    /// How the submission was graded (`/compile-real` only).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub grade: Option<GradeState>,
    /// True only when the program's behaviour was compared with an
    /// expectation. `ok: true` with `behaviour_checked: false` means
    /// "compiled (and maybe ran), but nothing checked what it does".
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub behaviour_checked: Option<bool>,
    /// What the check wanted, on a failed check. Player-facing.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expected: Option<String>,
    /// What the program did instead (truncated). Player-facing.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub actual: Option<String>,
    /// One line on how the submission was graded.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub note: Option<String>,
}

impl CompileResponse {
    /// The original three-field response.
    pub fn basic(ok: bool, stdout: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            ok,
            stdout: stdout.into(),
            stderr: stderr.into(),
            ..Self::default()
        }
    }
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
    /// The builder service is down or not answering (S-2).
    #[error("the build service is temporarily unavailable; retry in {retry_after_secs}s")]
    BuilderUnavailable { retry_after_secs: u64 },
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
    #[error("request body was not received within {secs}s")]
    RequestTimeout { secs: u64 },
    #[error("request did not complete within {secs}s; retry later")]
    DeadlineExceeded { secs: u64 },
    #[error("internal error: {0}")]
    Internal(String),
}

impl Rejection {
    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::QueueFull { .. }
            | Self::BuilderUnavailable { .. }
            | Self::DeadlineExceeded { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::BadRequest { status, .. } => *status,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::SourcePolicy(_) => StatusCode::BAD_REQUEST,
            Self::RequestTimeout { .. } => StatusCode::REQUEST_TIMEOUT,
            Self::ClientAddrUnavailable | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::RateLimited { .. } => "rate_limited",
            Self::QueueFull { .. } => "queue_full",
            Self::BuilderUnavailable { .. } => "builder_unavailable",
            Self::BodyTooLarge => "body_too_large",
            Self::BadRequest { .. } => "bad_request",
            Self::Forbidden(_) => "forbidden",
            Self::SourcePolicy(_) => "source_policy",
            Self::RequestTimeout { .. } => "request_timeout",
            Self::DeadlineExceeded { .. } => "deadline_exceeded",
            Self::ClientAddrUnavailable | Self::Internal(_) => "internal",
        }
    }

    fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_secs }
            | Self::QueueFull { retry_after_secs }
            | Self::BuilderUnavailable { retry_after_secs } => Some(*retry_after_secs),
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
        if matches!(self, Self::RequestTimeout { .. }) {
            resp.headers_mut()
                .insert(header::CONNECTION, HeaderValue::from_static("close"));
        }
        resp
    }
}

/// Blocking build-and-run function with no mission context. Kept as the
/// test-injection hook (`with_real_grader`) so the concurrency/queue path
/// can be exercised without a toolchain; its verdict is used as-is and the
/// response reports `behaviour_checked: false`.
pub type RealGrader = Arc<dyn Fn(&str, &RunLimits) -> anyhow::Result<RealOutcome> + Send + Sync>;

/// Blocking grading function: `(encounter_id, source, limits)`. Production
/// uses [`expectations::grade_in`].
pub type MissionGrader =
    Arc<dyn Fn(&str, &str, &RunLimits) -> anyhow::Result<GradedRun> + Send + Sync>;

struct Inner {
    config: Config,
    metrics: Metrics,
    global_limiter: RateLimiter,
    real_limiter: RateLimiter,
    builds: Arc<Semaphore>,
    queued: AtomicUsize,
    real_grader: MissionGrader,
}

/// Shared server state. Cheap to clone.
#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl AppState {
    /// Production state: real `cargo build` (in the configured build
    /// sandbox, under the configured build root) + wasmtime run, graded
    /// against the mission's expectation.
    ///
    /// Where the work runs follows `config.executor`: in this process,
    /// in one child process per job, or in the builder service.
    pub fn new(config: Config) -> Self {
        let settings = BuildSettings::from_config(&config);
        let grader: MissionGrader = match config.executor.clone() {
            Executor::InProcess => {
                Arc::new(move |encounter: &str, source: &str, limits: &RunLimits| {
                    expectations::grade_in(encounter, source, limits, &settings)
                })
            }
            Executor::Subprocess { exe } => {
                Arc::new(move |encounter: &str, source: &str, limits: &RunLimits| {
                    jobs::grade_via_subprocess(
                        &exe,
                        &settings,
                        job_request(encounter, source, limits),
                        jobs::JOB_DEADLINE,
                    )
                })
            }
            Executor::Builder { socket } => Arc::new(builder_grader(socket)),
        };
        Self::with_mission_grader(config, grader)
    }

    /// State with a custom blocking build-and-run function for
    /// `/compile-real` (test hook). No mission expectation is applied to
    /// its outcome; responses say `behaviour_checked: false`.
    pub fn with_real_grader(config: Config, real_grader: RealGrader) -> Self {
        let grader: MissionGrader =
            Arc::new(move |_encounter: &str, source: &str, limits: &RunLimits| {
                let outcome = real_grader(source, limits)?;
                Ok(GradedRun::unchecked(
                    outcome,
                    "injected grader: no output check applied",
                ))
            });
        Self::with_mission_grader(config, grader)
    }

    /// State with a custom blocking mission grader for `/compile-real`.
    pub fn with_mission_grader(config: Config, real_grader: MissionGrader) -> Self {
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

    /// Free build slots right now (tests assert it returns to
    /// `max_concurrent_builds` once work has really ended).
    #[doc(hidden)]
    pub fn available_build_slots(&self) -> usize {
        self.0.builds.available_permits()
    }
}

/// The IPC request for one grading call.
fn job_request(encounter: &str, source: &str, limits: &RunLimits) -> JobRequest {
    JobRequest {
        encounter_id: encounter.to_string(),
        source: source.to_string(),
        run_max_memory_bytes: limits.max_memory_bytes,
    }
}

#[cfg(unix)]
fn builder_grader(
    socket: std::path::PathBuf,
) -> impl Fn(&str, &str, &RunLimits) -> anyhow::Result<GradedRun> + Send + Sync + 'static {
    move |encounter: &str, source: &str, limits: &RunLimits| {
        let reply = match ipc::call_builder(&socket, &job_request(encounter, source, limits)) {
            Ok(r) => r,
            Err(e) if e.is_unavailable() => {
                tracing::warn!("compile-real: builder unavailable: {e}");
                return Err(jobs::ExecError::Unavailable(e.to_string()).into());
            }
            Err(e) => return Err(e.into()),
        };
        jobs::reply_to_graded(reply)
    }
}

#[cfg(not(unix))]
fn builder_grader(
    socket: std::path::PathBuf,
) -> impl Fn(&str, &str, &RunLimits) -> anyhow::Result<GradedRun> + Send + Sync + 'static {
    move |_encounter: &str, _source: &str, _limits: &RunLimits| {
        Err(anyhow::anyhow!(
            "the builder service ({}) is only supported on Linux",
            socket.display()
        ))
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

    // No /metrics here: it lives on its own loopback listener
    // (`build_metrics_router`), so the reverse proxy cannot reach it.
    let limited = player
        .layer(from_fn_with_state(
            state.clone(),
            middleware::global_rate_limit,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::request_deadline,
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

/// Router for the metrics listener (`PNC_METRICS_BIND`, loopback only):
/// `GET /metrics` and nothing else. Same [`AppState`] as the API router so
/// it reports the same counters.
pub fn build_metrics_router(state: AppState) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware::require_metrics_access,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serve `router` on `listener` with the default header-read timeout
/// ([`config::DEFAULT_HEADER_READ_TIMEOUT`]). See [`serve_with`].
pub async fn serve(listener: tokio::net::TcpListener, router: Router) -> std::io::Result<()> {
    serve_with(listener, router, config::DEFAULT_HEADER_READ_TIMEOUT).await
}

/// Serve `router` on `listener`, HTTP/1.1 only, with:
///
/// - the peer address attached to every request as
///   `ConnectInfo<SocketAddr>` (the per-IP rate limiter needs it);
/// - a header-read timeout: a client that has not sent the full request
///   line + headers within `header_read_timeout` is disconnected (hyper's
///   own timer; `axum::serve` sets no timer, so this was unbounded).
///
/// Body and whole-request deadlines are middleware (see
/// [`middleware::limit_body`] and [`middleware::request_deadline`]).
pub async fn serve_with(
    listener: tokio::net::TcpListener,
    router: Router,
    header_read_timeout: Duration,
) -> std::io::Result<()> {
    use hyper_util::rt::{TokioIo, TokioTimer};
    use hyper_util::service::TowerToHyperService;
    use tower::ServiceExt;

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                // EMFILE and friends: back off instead of spinning.
                tracing::warn!("serve: accept failed: {e}");
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
        };
        let svc = router.clone().map_request(
            move |mut req: axum::http::Request<hyper::body::Incoming>| {
                req.extensions_mut().insert(ConnectInfo(peer));
                req
            },
        );
        let svc = TowerToHyperService::new(svc);
        tokio::spawn(async move {
            let mut builder = hyper::server::conn::http1::Builder::new();
            builder
                .timer(TokioTimer::new())
                .header_read_timeout(header_read_timeout);
            if let Err(e) = builder.serve_connection(TokioIo::new(stream), svc).await {
                tracing::debug!(%peer, "serve: connection ended with error: {e}");
            }
        });
    }
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
        Json(CompileResponse::basic(v.ok, v.stdout, v.stderr)),
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
/// Honest verdict: the player source is built to `wasm32-wasip1`, the
/// resulting module is *actually executed* under the hardened sandbox in
/// [`wasm_runner`], and its behaviour is checked against the mission's
/// expectation ([`expectations`]). The response's `grade` says which kind
/// of verdict it is; `behaviour_checked` is false whenever nothing checked
/// what the program does (unknown mission, player-chosen output, or a
/// compile-only mission). The program's real stdout is returned (checker
/// lines removed). Tags every response with a leading `[real]` marker so a
/// player or test can tell which path produced the verdict; the marker
/// drops when this route becomes the default.
///
/// The build never runs on the async runtime: it moves to
/// `spawn_blocking` holding a semaphore permit, and the permit is held by
/// the blocking task itself, so a client that disconnects (or hits the
/// request deadline) mid-build does not free the slot before the build
/// actually finishes. If the build's processes could not be killed
/// ([`proc_tree::ProcError::Survivors`]) the permit is never released.
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
    let encounter = req.encounter_id.clone();
    let joined = tokio::task::spawn_blocking(move || {
        let _in_flight = InFlight::enter(task_state);
        let result = grader(&encounter, &source, &limits);
        if let Err(e) = &result
            && proc_tree::chain_has_survivors(e)
        {
            // Processes of this build may still be running: never give
            // the slot back (the capacity they use stays accounted for).
            tracing::error!(
                "compile-real: build processes survived SIGKILL; retiring this build slot \
                 permanently (restart the service): {e:#}"
            );
            std::mem::forget(permit);
        } else {
            drop(permit);
        }
        result
    })
    .await;

    // Honest end-to-end: build to wasm, run it in the sandbox, then check
    // its behaviour against the mission. We do NOT stitch in pattern-grader
    // flavor — the two answer different questions and a stitched response
    // can be self-contradictory.
    let graded = match joined {
        Ok(Ok(graded)) => graded,
        Ok(Err(e))
            if matches!(
                e.downcast_ref::<jobs::ExecError>(),
                Some(jobs::ExecError::Busy)
            ) =>
        {
            tracing::warn!("rejected: builder has no free job slot");
            state.metrics().record_rejection(RejectReason::Queue);
            return Rejection::QueueFull {
                retry_after_secs: QUEUE_RETRY_AFTER_SECS,
            }
            .into_response();
        }
        Ok(Err(e))
            if matches!(
                e.downcast_ref::<jobs::ExecError>(),
                Some(jobs::ExecError::Unavailable(_))
            ) =>
        {
            tracing::warn!("rejected: builder unavailable: {e:#}");
            state.metrics().record_rejection(RejectReason::Unavailable);
            return Rejection::BuilderUnavailable {
                retry_after_secs: QUEUE_RETRY_AFTER_SECS,
            }
            .into_response();
        }
        Ok(Err(e)) => return setup_failure_response(&e),
        Err(join_err) => {
            tracing::error!("compile-real: build task failed: {join_err}");
            return Rejection::Internal("build task failed".to_string()).into_response();
        }
    };

    let outcome = &graded.outcome;
    state.metrics().observe_build(outcome.build_time);
    if let Some(run) = outcome.run_time {
        state.metrics().observe_run(run);
    }
    tracing::info!(
        encounter = %req.encounter_id,
        compiled = outcome.compiled,
        ran_ok = outcome.ran_ok,
        grade = ?graded.state,
        behaviour_checked = graded.state.behaviour_checked(),
        build_ms = outcome.build_time.as_millis() as u64,
        run_ms = outcome.run_time.map(|d| d.as_millis() as u64),
        "compile-real: verdict"
    );
    (StatusCode::OK, Json(real_response(graded))).into_response()
}

/// Player-facing text of a setup failure (S-3). The error chain names job
/// dirs, sockets, binaries and toolchain paths, so it goes to the log only,
/// under a request id the player can quote; the client gets this generic
/// line (no path, no internal detail).
pub fn setup_failure_message(request_id: &str) -> String {
    format!(
        "internal error: the server could not build or run your program (request id \
         {request_id}). Please try again; if it keeps happening, report the request id."
    )
}

/// 500 for a grading setup failure: full chain logged with a fresh request
/// id, generic body to the client.
fn setup_failure_response(e: &anyhow::Error) -> Response {
    let request_id = uuid::Uuid::new_v4().simple().to_string();
    tracing::error!(request_id, "compile-real: setup failure: {e:#}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CompileResponse::basic(
            false,
            "[real]",
            setup_failure_message(&request_id),
        )),
    )
        .into_response()
}

/// Build the `/compile-real` body from a grading result. `stdout` keeps
/// the `[real]` marker; a passing response adds one line saying whether
/// behaviour was checked, so the existing client (which shows only
/// `stdout`/`stderr`) never presents an unchecked run as a checked one.
pub fn real_response(graded: GradedRun) -> CompileResponse {
    let ok = graded.ok();
    let state = graded.state;
    let out = graded.outcome;
    let stdout = if ok {
        let body = if out.stdout.is_empty() {
            match state {
                GradeState::CompiledNotChecked => "compiled".to_string(),
                _ => "compiled + ran: ok".to_string(),
            }
        } else {
            out.stdout.trim_end_matches(['\n', '\r']).to_string()
        };
        let verdict_line = match state {
            GradeState::Passed => "-- checked: does what the mission asks".to_string(),
            _ => format!("-- not checked: {}", graded.note),
        };
        format!("[real] {body}\n{verdict_line}")
    } else if matches!(state, GradeState::WrongOutput) && !out.stdout.is_empty() {
        format!("[real] {}", out.stdout)
    } else {
        "[real]".to_string()
    };
    CompileResponse {
        ok,
        stdout,
        stderr: out.stderr,
        grade: Some(state),
        behaviour_checked: Some(state.behaviour_checked()),
        expected: graded.expected,
        actual: graded.actual,
        note: Some(graded.note),
    }
}

/// Shared rejection logic for the two compile routes. Returns `Some`
/// with a 400 response if the source is empty/whitespace or exceeds
/// the byte cap; `None` if the source is acceptable.
fn reject_invalid_source(source: &str) -> Option<(StatusCode, Json<CompileResponse>)> {
    if source.trim().is_empty() {
        tracing::warn!("rejected: empty source");
        return Some((
            StatusCode::BAD_REQUEST,
            Json(CompileResponse::basic(false, "", "empty source")),
        ));
    }
    if source.len() > MAX_SOURCE_BYTES {
        tracing::warn!(source_bytes = source.len(), "rejected: source over cap");
        return Some((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(CompileResponse::basic(
                false,
                "",
                format!(
                    "source exceeds limit of {} bytes (got {})",
                    MAX_SOURCE_BYTES,
                    source.len()
                ),
            )),
        ));
    }
    None
}
