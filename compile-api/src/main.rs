// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pledge & Crown compile API binary. Routes + handlers live in
//! `lib.rs` so the integration test harness can mount the same router.
//!
//! Subcommands (one binary, two systemd units):
//!
//! | Command | Who runs it | What it does |
//! |---|---|---|
//! | `serve` (default) | `pnc-compile-api.service` | HTTP API; holds the tokens; never builds player code in production |
//! | `builder` | `pnc-builder.service` | builds + runs player code, one process per job; no secrets |
//! | `job` | the builder (internal) | one job: envelope on stdin, reply on stdout |
//! | `run-wasm` | a job, inside the run sandbox (internal) | runs one module: limits + bytes on stdin, verdict on stdout |
//! | `sandbox-verify` | `deploy/verify-sandbox.sh` | one probe build with the deny-list off, bwrap only |
//! | `sandbox-verify-run` | `deploy/verify-sandbox.sh` | runs the native `run-probe` inside the run sandbox, bwrap only |
//! | `run-probe` | `sandbox-verify-run`, inside the run sandbox | reports what exists/connects there (never contents) |
//! | `toolchain` | the deploy README | prints the pinned rustc version |
//!
//! Configuration comes from `PNC_*` environment variables (see
//! `config.rs` and `deploy/env.example`).

use std::io::IsTerminal;
use std::process::ExitCode;

use pledge_compile_api::build_sandbox::SandboxMode;
use pledge_compile_api::builder::{self, BuilderConfig};
use pledge_compile_api::toolchain::PINNED_RUSTC_VERSION;
use pledge_compile_api::wasm_builder::{self, BuildSettings};
use pledge_compile_api::{
    AppState, Config, Executor, build_metrics_router, build_root, build_router, jobs, run_sandbox,
    serve_with,
};
use tracing_subscriber::{EnvFilter, fmt};

const DEFAULT_FILTER: &str = "pledge_compile_api=debug,tower_http=info,axum::rejection=trace";
const USAGE: &str = "usage: pledge_compile_api \
                     [serve|builder|job|run-wasm|sandbox-verify|sandbox-verify-run|run-probe|toolchain]";

/// `to_stderr`: the internal/one-shot commands, whose stdout carries data
/// (job reply, verdict, probe JSON). Colour only on a terminal: services
/// log to the journal and child processes to a pipe that is re-logged, so
/// ANSI escapes would end up as junk in the journal.
fn init_logging(to_stderr: bool) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    if to_stderr {
        fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(std::io::stderr)
            .init();
    } else {
        fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_ansi(std::io::stdout().is_terminal())
            .init();
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("serve");
    if args.len() > 1 {
        eprintln!("{USAGE}");
        return ExitCode::from(64);
    }
    match cmd {
        "toolchain" => {
            println!("{PINNED_RUSTC_VERSION}");
            ExitCode::SUCCESS
        }
        jobs::JOB_SUBCOMMAND => {
            init_logging(true);
            jobs::job_main()
        }
        run_sandbox::RUN_WASM_SUBCOMMAND => {
            init_logging(true);
            run_sandbox::run_wasm_main()
        }
        run_sandbox::RUN_PROBE_SUBCOMMAND => {
            init_logging(true);
            run_sandbox::run_probe_main()
        }
        builder::SANDBOX_VERIFY_SUBCOMMAND => {
            init_logging(true);
            builder::sandbox_verify_main()
        }
        builder::SANDBOX_VERIFY_RUN_SUBCOMMAND => {
            init_logging(true);
            builder::sandbox_verify_run_main()
        }
        builder::BUILDER_SUBCOMMAND => {
            init_logging(false);
            let result = BuilderConfig::from_env()
                .map_err(anyhow::Error::from)
                .and_then(builder::run_builder);
            match result {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    tracing::error!("builder: {e:#}");
                    ExitCode::FAILURE
                }
            }
        }
        "serve" => {
            init_logging(false);
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("cannot start the async runtime: {e}");
                    return ExitCode::FAILURE;
                }
            };
            match rt.block_on(serve_main()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    tracing::error!("serve: {e:#}");
                    ExitCode::FAILURE
                }
            }
        }
        other => {
            eprintln!("unknown command {other:?}; {USAGE}");
            ExitCode::from(64)
        }
    }
}

async fn serve_main() -> anyhow::Result<()> {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("refusing to start: invalid configuration: {e}");
            return Err(e.into());
        }
    };
    // `Config`'s Debug impl redacts the bearer tokens.
    tracing::info!("config: {config:?}");
    if config.auth_enabled() {
        tracing::info!(
            "auth: bearer token required on /compile, /compile-real ({} token(s))",
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
        tracing::info!(
            "metrics: no PNC_METRICS_TOKENS; /metrics on {} answers direct loopback only",
            config.metrics_bind
        );
    }
    if !config.source_denylist {
        tracing::warn!("PNC_SOURCE_DENYLIST=false: the second-layer source deny-list is OFF (dev)");
    }
    if config.dev_unsafe_local_builds {
        tracing::warn!(
            "PNC_DEV_UNSAFE_LOCAL_BUILDS=true: local dev override active (loopback only). Never \
             set this on a server."
        );
    }

    match &config.executor {
        Executor::Builder { socket } => {
            tracing::info!(
                socket = %socket.display(),
                "executor: builder service (this process never builds or runs player code)"
            );
            match std::fs::symlink_metadata(socket) {
                Ok(_) => {}
                Err(e) => tracing::warn!(
                    "builder socket {} not present yet ({e}); /compile-real fails until \
                     pnc-builder.service is up",
                    socket.display()
                ),
            }
        }
        Executor::InProcess | Executor::Subprocess { .. } => {
            tracing::warn!(
                executor = ?config.executor,
                "executor: player code builds and runs on this host as this user (dev only)"
            );
            local_build_self_test(&config).await?;
        }
    }

    let addr = config.bind_addr;
    let metrics_addr = config.metrics_bind;
    let header_timeout = config.header_read_timeout;
    let state = AppState::new(config);
    let app = build_router(state.clone());
    let metrics_app = build_metrics_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let metrics_listener = tokio::net::TcpListener::bind(metrics_addr).await?;
    tracing::info!("pledge compile api listening on http://{addr}");
    tracing::info!("metrics listening on http://{metrics_addr}/metrics (loopback only)");
    tokio::try_join!(
        serve_with(listener, app, header_timeout),
        serve_with(metrics_listener, metrics_app, header_timeout),
    )?;
    Ok(())
}

/// In-process / subprocess executors build on this host: prepare the
/// build root and run the build self-test (mandatory for bwrap).
async fn local_build_self_test(config: &Config) -> anyhow::Result<()> {
    build_root::prepare(&config.build_root)?;
    match build_root::sweep_stale(&config.build_root, build_root::STALE_JOB_AGE) {
        Ok(n) => tracing::info!(
            "build root {}: swept {n} stale job dir(s)",
            config.build_root.display()
        ),
        Err(e) => tracing::warn!("build root sweep failed: {e}"),
    }
    let settings = BuildSettings::from_config(config);
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
             user can. Local dev only; config refuses it for exposed deployments."
        );
    }
    Ok(())
}
