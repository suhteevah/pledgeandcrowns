// SPDX-License-Identifier: AGPL-3.0-or-later
//! Library surface for the compile-API. Exposes the router builder
//! and the grader so integration tests can drive the same code paths
//! the binary uses.

pub mod cargo_grader;
pub mod grader;
pub mod wasm_runner;

use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get, routing::post};
use serde::{Deserialize, Serialize};

/// Hard cap on player source bytes accepted by `/compile` and
/// `/compile-real`. The pattern grader is fast for any input but the
/// cargo path hands the buffer to rustc's parser; an unbounded body
/// is a cheap DoS surface (10 MB submissions exhaust ram + cpu in the
/// parser before any sandbox limit kicks in). 64 KiB is roughly
/// 10x the largest reasonable mission solution and an order of
/// magnitude under any size that meaningfully stresses the parser.
const MAX_SOURCE_BYTES: usize = 64 * 1024;

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
    if let Some(rejection) = reject_invalid_source(&req.source) {
        return rejection;
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
    if let Some(rejection) = reject_invalid_source(&req.source) {
        return rejection;
    }
    // Real verdict only — do NOT stitch in pattern-grader flavor here.
    // The two graders answer different questions: the pattern grader
    // says "did the player use the right construct?", cargo says "does
    // it compile?". A solution can be syntactically correct (ok=true
    // from cargo) but use the wrong construct (ok=false from pattern),
    // and the previous stitched response could ship `ok=true` with a
    // "missing required: let mut" stderr — incoherent. When wasmtime
    // execution lands, that layer carries the per-encounter flavor;
    // for now we ship cargo's verdict as-is, marked [real].
    match cargo_grader::compile_check(&req.source) {
        Ok(verdict) => (
            StatusCode::OK,
            Json(CompileResponse {
                ok: verdict.ok,
                stdout: if verdict.ok {
                    "[real] cargo check: ok".to_string()
                } else {
                    "[real]".to_string()
                },
                stderr: verdict.stderr,
            }),
        ),
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

/// Shared rejection logic for the two compile routes. Returns `Some`
/// with a 400 response if the source is empty/whitespace or exceeds
/// the byte cap; `None` if the source is acceptable.
fn reject_invalid_source(source: &str) -> Option<(StatusCode, Json<CompileResponse>)> {
    if source.trim().is_empty() {
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
