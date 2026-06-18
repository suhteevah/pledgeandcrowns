// SPDX-License-Identifier: AGPL-3.0-or-later
//! Library surface for the compile-API. Exposes the router builder
//! and the grader so integration tests can drive the same code paths
//! the binary uses.

pub mod cargo_grader;
pub mod grader;
pub mod wasm_builder;
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
async fn compile_real(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    tracing::info!(
        "compile-real: encounter={} source_bytes={}",
        req.encounter_id,
        req.source.len()
    );
    if let Some(rejection) = reject_invalid_source(&req.source) {
        return rejection;
    }
    // Honest end-to-end: build to wasm, then run it in the sandbox. We do
    // NOT stitch in pattern-grader flavor — the two answer different
    // questions and a stitched response can be self-contradictory
    // (ok=true from compile, "missing required: let mut" from pattern).
    // The execution layer carries the real verdict; per-encounter
    // expected-output checks layer on top of this in a later pass.
    match wasm_builder::compile_and_run(&req.source) {
        Ok(outcome) => {
            let stdout = if outcome.ok() {
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
                    ok: outcome.ok(),
                    stdout,
                    stderr: outcome.stderr,
                }),
            )
        }
        Err(e) => {
            tracing::error!("compile-real: wasm_builder setup failure: {e:#}");
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
