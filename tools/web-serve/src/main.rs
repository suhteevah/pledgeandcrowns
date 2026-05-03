// SPDX-License-Identifier: MIT
//! Minimal local dev server for the Pledge & Crown wasm web build.
//! Serves `<workspace>/web/` on `127.0.0.1:8080` (override with
//! `WEB_SERVE_ADDR` and `WEB_SERVE_DIR` env vars).
//!
//! Replaces the previous `python -m http.server -d web` hint per the
//! no-Python rule. Pure Rust, axum, ~50 LOC.

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("web_serve=info")),
        )
        .init();

    let addr: SocketAddr = std::env::var("WEB_SERVE_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()
        .context("parsing WEB_SERVE_ADDR")?;

    let dir = match std::env::var("WEB_SERVE_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => default_web_dir()?,
    };
    if !dir.is_dir() {
        bail!(
            "web dir `{}` does not exist — run `scripts/web-build.ps1` first",
            dir.display()
        );
    }

    tracing::info!("serving {} on http://{}", dir.display(), addr);
    let app = Router::new()
        .fallback_service(ServeDir::new(dir))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Walk up from CWD looking for the workspace marker, then return
/// `<workspace>/web/`. Mirrors the logic in `tools/render-refs`.
fn default_web_dir() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("tools").join("web-serve").is_dir() {
            return Ok(dir.join("web"));
        }
        if !dir.pop() {
            bail!("could not locate workspace root; set WEB_SERVE_DIR to override");
        }
    }
}
