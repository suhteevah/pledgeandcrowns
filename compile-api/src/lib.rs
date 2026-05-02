// SPDX-License-Identifier: AGPL-3.0-or-later
//! Library surface for the compile-API. Exposes the router builder
//! and the grader so integration tests can drive the same code paths
//! the binary uses.

pub mod grader;

use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get, routing::post};
use serde::{Deserialize, Serialize};

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

pub fn make_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/compile", post(compile))
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn compile(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    tracing::info!(
        "compile: encounter={} source_bytes={}",
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
    let v = grader::grade(&req.encounter_id, &req.source);
    (
        StatusCode::OK,
        Json(CompileResponse {
            ok: v.ok,
            stdout: v.stdout,
            stderr: v.stderr,
        }),
    )
}
