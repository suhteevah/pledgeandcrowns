// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pledge & Crown compile API binary. Routes + handlers live in
//! `lib.rs` so the integration test harness can mount the same router.
//!
//! Configuration comes from `PNC_*` environment variables, parsed once
//! into [`pledge_compile_api::Config`] (see `config.rs` for the table and
//! `deploy/env.example` for a commented template).

use pledge_compile_api::build_sandbox::SandboxMode;
use pledge_compile_api::wasm_builder::{self, BuildSettings};
use pledge_compile_api::{AppState, Config, build_root, build_router, serve};
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
        // Only reachable with PNC_ALLOW_TRUST_PROXY_PUBLIC_BIND=true.
        tracing::warn!(
            "PNC_TRUST_PROXY is on with non-loopback bind {} (explicitly overridden): clients \
             that can reach the socket directly can spoof X-Forwarded-For",
            config.bind_addr
        );
    }
    if config.metrics_tokens.is_empty() {
        tracing::info!("metrics: no PNC_METRICS_TOKENS; /metrics served to direct loopback only");
    }
    if !config.source_denylist {
        tracing::warn!(
            "PNC_SOURCE_DENYLIST=false: the second-layer source deny-list is OFF. This is only \
             for deploy/verify-sandbox.sh; turn it back on."
        );
    }

    // Build root: create/verify ownership + mode, then sweep job dirs a
    // previous crash left behind.
    build_root::prepare(&config.build_root)?;
    match build_root::sweep_stale(&config.build_root, build_root::STALE_JOB_AGE) {
        Ok(n) => tracing::info!(
            "build root {}: swept {n} stale job dir(s)",
            config.build_root.display()
        ),
        Err(e) => tracing::warn!("build root sweep failed: {e}"),
    }

    // Build sandbox self-test. Mandatory for bwrap: refuse to serve if the
    // sandbox does not isolate, or a trivial build cannot run inside it.
    let settings = BuildSettings::from_config(&config);
    let mode = settings.sandbox.mode;
    let self_test =
        tokio::task::spawn_blocking(move || wasm_builder::startup_self_test(&settings)).await?;
    match (mode, self_test) {
        (_, Ok(())) => {}
        (SandboxMode::Bwrap, Err(e)) => {
            tracing::error!("refusing to start: build sandbox self-test failed: {e:#}");
            return Err(e);
        }
        (SandboxMode::None, Err(e)) => tracing::warn!(
            "build self-test failed (PNC_BUILD_SANDBOX=none, continuing; /compile-real will \
             fail): {e:#}"
        ),
    }
    if mode == SandboxMode::None {
        tracing::warn!(
            "PNC_BUILD_SANDBOX=none: player builds run on the host and can read any file this \
             user can. Dev only; config refuses this for exposed deployments."
        );
    }

    let addr = config.bind_addr;
    let app = build_router(AppState::new(config));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("pledge compile api listening on http://{addr}");
    serve(listener, app).await?;
    Ok(())
}
