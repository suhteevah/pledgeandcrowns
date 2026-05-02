// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pledge & Crown compile API - stub.
//!
//! Real implementation per `design/05-tech-architecture.md` section 2:
//! - server owns Cargo.toml; player input never touches it
//! - vendored deps; cargo --offline; single player file
//! - non-root user, no-network container, ephemeral fs, seccomp, dropped caps
//! - three-layer cache: redis source-hash, sccache, pre-warmed target/
//!
//! For now this is a hello-world to prove the workspace compiles.

use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("pledge_compile_api=debug,tower_http=info"));

    fmt().with_env_filter(filter).init();

    tracing::info!("pledge compile api - stub boot");
    tracing::warn!(
        "server is a stub. real implementation pending design/05-tech-architecture.md sec 2."
    );
    Ok(())
}
