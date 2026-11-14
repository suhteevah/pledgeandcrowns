// SPDX-License-Identifier: MIT
//! Shared support for the audit suites that compile player-facing code.
//!
//! The compile-API builds every player program with a server-owned
//! manifest (`compile-api/src/wasm_builder.rs` and `cargo_grader.rs`). Any
//! test that runs `rustc` on a solution, starter or corpus program must use
//! the same edition, or a program can pass here and fail on the server (an
//! Act 9 FFI draft did exactly that under 2021 vs 2024). The server does not
//! export its edition as a constant and the game crate may not depend on it
//! at runtime, so the value lives here and
//! `solutions.rs::player_edition_matches_the_server_manifests` greps the
//! server's manifests and fails if they ever disagree.

/// Rust edition the compile-API builds player code with.
pub const PLAYER_EDITION: &str = "2024";

/// Server sources that embed a player-code `Cargo.toml` manifest.
#[allow(dead_code)] // only `solutions.rs` checks the manifests
pub const SERVER_MANIFEST_SOURCES: &[&str] = &[
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../compile-api/src/wasm_builder.rs"
    ),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../compile-api/src/cargo_grader.rs"
    ),
];
