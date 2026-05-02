// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pledge & Crown compile API.
//!
//! Day-6 cut: axum server with `/health` and `/compile` endpoints.
//! `/compile` accepts a JSON body `{source: string, encounter_id: string}`
//! and returns a stub `{ok: bool, stdout: string, stderr: string}`.
//! The real wasmtime sandbox + cargo-build pipeline lands per
//! `design/05-tech-architecture.md` §2 (manifest control, vendored
//! deps, `--offline`, seccomp). Until then this proves wire format,
//! routing, CORS, and tracing-instrumented request handling.

use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get, routing::post};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt};

const BIND_ADDR: &str = "127.0.0.1:7878";

#[derive(Deserialize)]
struct CompileRequest {
    source: String,
    encounter_id: String,
}

#[derive(Serialize)]
struct CompileResponse {
    ok: bool,
    stdout: String,
    stderr: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("pledge_compile_api=debug,tower_http=info,axum::rejection=trace")
    });
    fmt().with_env_filter(filter).with_target(true).init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/compile", post(compile))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = BIND_ADDR.parse()?;
    tracing::info!("pledge compile api listening on http://{addr}");
    tracing::warn!(
        "compile endpoint is a stub. real wasmtime sandbox per design/05-tech-architecture.md sec 2."
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn compile(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    tracing::info!(
        "compile request: encounter={} source_bytes={}",
        req.encounter_id,
        req.source.len()
    );
    if req.source.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CompileResponse {
                ok: false,
                stdout: String::new(),
                stderr: "empty source".to_string(),
            }),
        );
    }
    (
        StatusCode::OK,
        Json(CompileResponse {
            ok: true,
            stdout: format!(
                "[stub] would compile {} bytes for encounter {}",
                req.source.len(),
                req.encounter_id
            ),
            stderr: String::new(),
        }),
    )
}
