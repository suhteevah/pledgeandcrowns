// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pledge & Crown compile API binary. Routes + handlers live in
//! `lib.rs` so the integration test harness can mount the same router.
//!
//! Configuration comes from `PNC_*` environment variables, parsed once
//! into [`pledge_compile_api::Config`] (see `config.rs` for the table and
//! `deploy/env.example` for a commented template).

use pledge_compile_api::{AppState, Config, build_router, serve};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("pledge_compile_api=debug,tower_http=info,axum::rejection=trace")
    });
    fmt().with_env_filter(filter).with_target(true).init();

    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("invalid configuration: {e}");
            return Err(e.into());
        }
    };
    // `Config`'s Debug impl redacts the bearer tokens.
    tracing::info!("config: {config:?}");
    if config.auth_enabled() {
        tracing::info!(
            "auth: bearer token required on /compile, /compile-real, /metrics ({} token(s))",
            config.auth_tokens.len()
        );
    } else {
        tracing::warn!("auth: DISABLED (no PNC_AUTH_TOKENS); loopback bind enforced");
    }
    if config.trust_proxy && !config.bind_addr.ip().is_loopback() {
        tracing::warn!(
            "PNC_TRUST_PROXY is on but the bind address {} is not loopback: clients that can \
             reach the socket directly can spoof X-Forwarded-For and dodge the rate limit",
            config.bind_addr
        );
    }

    let addr = config.bind_addr;
    let app = build_router(AppState::new(config));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("pledge compile api listening on http://{addr}");
    serve(listener, app).await?;
    Ok(())
}
