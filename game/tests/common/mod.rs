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

/// The rustc release the compile-API builds player code with: a copy of
/// `pledge_compile_api::toolchain::PINNED_RUSTC_VERSION`. NPC dialogue
/// quotes rustc's wording, which changes between releases, so every test
/// that runs `rustc` on a claim, starter or solution first calls
/// [`assert_pinned_rustc`]. The value is duplicated rather than imported
/// so the audit suites stay free of compile-API types (the game crate may
/// never depend on the AGPL server at run time);
/// `contract.rs::pinned_rustc_copy_matches_the_server` fails if the two
/// copies ever disagree.
#[allow(dead_code)] // not every suite that mounts `common` runs rustc
pub const PINNED_RUSTC_VERSION: &str = "1.99.0";

/// Panic, with the fix in the message, unless the `rustc` on `PATH` is
/// [`PINNED_RUSTC_VERSION`]. A claim checked on another compiler proves
/// nothing about what players are told (hard rule 3). Returns the version
/// line for the test log.
#[allow(dead_code)] // not every suite that mounts `common` runs rustc
pub fn assert_pinned_rustc() -> String {
    let out = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .expect("run `rustc --version`");
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let found = line.split_whitespace().nth(1).unwrap_or("");
    assert_eq!(
        found, PINNED_RUSTC_VERSION,
        "rustc on PATH is `{line}`, but the compile-API (and so every NPC \
         claim) is pinned to {PINNED_RUSTC_VERSION}. Run the slow suites on \
         the pinned toolchain: `rustup toolchain install {PINNED_RUSTC_VERSION} \
         --target wasm32-wasip1`, then `rustup run {PINNED_RUSTC_VERSION} cargo \
         test ... -- --ignored` (scripts/ci-slow.ps1 -Toolchain {PINNED_RUSTC_VERSION})."
    );
    println!("{line}");
    line
}

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
