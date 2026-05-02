// SPDX-License-Identifier: MIT
//! Pledge & Crown - main entry point.
//!
//! Sets up tracing, then hands off to the Bevy app builder in `lib.rs`.
//! Per global rule: verbose logging everywhere. Default filter is `debug`
//! for our crate, `info` for Bevy internals (which are very chatty).

use tracing_subscriber::{EnvFilter, fmt};

fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("pledge_and_crown=debug,bevy=info,wgpu=warn,naga=warn"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();

    tracing::info!("pledge & crown - boot");
    tracing::debug!("tracing initialized; handing off to bevy app");

    pledge_and_crown::run()
}
