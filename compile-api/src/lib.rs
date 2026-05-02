// SPDX-License-Identifier: AGPL-3.0-or-later
//! Library surface for the compile-API. Exposes the router builder
//! and the grader so integration tests can drive the same code paths
//! the binary uses.

pub mod cargo_grader;
pub mod grader;
pub mod wasm_runner;

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
        .route("/compile-real", post(compile_real))
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

/// Real `cargo check` route. Slow (1-3s warm) — not yet wired into the
/// game client. Same request/response shape as `/compile` so swapping
/// is a one-line client change once we're confident in the path.
///
/// Tags every response with a leading `[real]` marker in `stdout` so a
/// player or test can tell which path generated the verdict. The marker
/// will be removed when this route becomes the default.
async fn compile_real(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    tracing::info!(
        "compile-real: encounter={} source_bytes={}",
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
    match cargo_grader::compile_check(&req.source) {
        Ok(verdict) => {
            if verdict.ok {
                // cargo check passed — also run the pattern grader so the
                // player still gets the per-encounter flavor message.
                let pattern = grader::grade(&req.encounter_id, &req.source);
                (
                    StatusCode::OK,
                    Json(CompileResponse {
                        ok: true,
                        stdout: format!("[real] {}", pattern.stdout),
                        stderr: verdict.stderr,
                    }),
                )
            } else {
                (
                    StatusCode::OK,
                    Json(CompileResponse {
                        ok: false,
                        stdout: "[real]".to_string(),
                        stderr: verdict.stderr,
                    }),
                )
            }
        }
        Err(e) => {
            tracing::error!("compile-real: cargo_grader setup failure: {e:#}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CompileResponse {
                    ok: false,
                    stdout: "[real]".to_string(),
                    stderr: format!("{e:#}"),
                }),
            )
        }
    }
}
