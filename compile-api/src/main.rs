// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pledge & Crown compile API binary. Routes + handlers live in
//! `lib.rs` so the integration test harness can mount the same router.

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt};

const BIND_ADDR: &str = "127.0.0.1:7878";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("pledge_compile_api=debug,tower_http=info,axum::rejection=trace")
    });
    fmt().with_env_filter(filter).with_target(true).init();

    let app = pledge_compile_api::make_router()
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
