// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pledge & Crown compile API.
//!
//! Day-8 cut: per-encounter validation. The endpoint matches on
//! `encounter_id` and runs a hand-coded checker that pattern-matches
//! the player's source. Real `cargo build --offline` + wasmtime
//! sandbox per `design/05-tech-architecture.md` §2 still pending —
//! pattern-matching is the v0 grader so the in-game mission loop
//! works end-to-end without paying the security tax yet.

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
        "v0 grader is pattern-based. real cargo build + wasmtime sandbox per design/05 sec 2 pending."
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

    let verdict = grade(&req.encounter_id, &req.source);
    (
        StatusCode::OK,
        Json(CompileResponse {
            ok: verdict.ok,
            stdout: verdict.stdout,
            stderr: verdict.stderr,
        }),
    )
}

struct Verdict {
    ok: bool,
    stdout: String,
    stderr: String,
}

impl Verdict {
    fn pass(msg: impl Into<String>) -> Self {
        Self {
            ok: true,
            stdout: msg.into(),
            stderr: String::new(),
        }
    }
    fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            stdout: String::new(),
            stderr: msg.into(),
        }
    }
}

/// Source contains all of the listed substrings (whitespace-flexible
/// only on individual checks; we trim & lowercase neither — Rust is
/// case-sensitive and the curriculum should reward correctness).
fn requires_all(source: &str, needles: &[&str]) -> Result<(), String> {
    for n in needles {
        if !source.contains(n) {
            return Err(format!("missing required: `{n}`"));
        }
    }
    Ok(())
}

fn grade(encounter_id: &str, source: &str) -> Verdict {
    match encounter_id {
        // ── Mission 1: bind 42 to `answer`.
        "intro_let_binding" => match requires_all(source, &["let answer", "42"]) {
            Ok(()) => Verdict::pass("answer bound. Ferris nods approvingly."),
            Err(e) => Verdict::fail(format!("not yet — {e}")),
        },

        // ── Mission 2: define `fn double(n: i32) -> i32 { n * 2 }`.
        "double_function" => match requires_all(source, &["fn double", "i32", "* 2"])
            .or_else(|_| requires_all(source, &["fn double", "i32", "*2"]))
        {
            Ok(()) => Verdict::pass("the Trait Mage smiles: `double(21)` would yield 42."),
            Err(e) => Verdict::fail(format!("not yet — {e}")),
        },

        // ── Mission 3: take `&value` and print it.
        "borrow_preview" => match requires_all(source, &["&value", "println!"]) {
            Ok(()) => Verdict::pass("the Borrow Checker stirs. \"...acceptable. for now.\""),
            Err(e) => Verdict::fail(format!("the Borrow Checker is silent — {e}")),
        },

        // ── Freeform sandbox / unknown encounter: accept anything,
        // report length so the round-trip is visibly working.
        _ => Verdict::pass(format!(
            "[freeform] received {} bytes. encounter `{encounter_id}` has no grader yet.",
            source.len()
        )),
    }
}
