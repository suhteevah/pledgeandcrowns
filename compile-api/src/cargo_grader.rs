// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real `cargo check` grader. Drop-in successor to the v0 pattern grader
//! in `grader.rs`, but NOT wired into any HTTP route — exposed only as a
//! public function so the security model can be reviewed and
//! regression-tested in isolation.
//!
//! # Security contract (per `design/05-tech-architecture.md` §2)
//!
//! - Server owns the manifest. Player input goes ONLY into `src/lib.rs`.
//!   It never touches `Cargo.toml`, never appears as a path component,
//!   never appears as a process arg.
//! - Each invocation gets a fresh job dir under a checked root at
//!   `<temp>/pledge-cargo-grader/<uuid>/` (see [`crate::build_root`]).
//!   The directory name is a freshly-generated UUID v4.
//! - Source is checked against the compile-time file-read deny-list
//!   ([`crate::source_policy`]) before cargo runs.
//! - `cargo check --offline` runs under a 10s watchdog that kills the
//!   whole process tree ([`crate::proc_tree`]); output capture is bounded
//!   at 16 KiB.
//! - NOT sandboxed: this path runs cargo on the host and is not reachable
//!   from the network. If it is ever wired into a route it must go
//!   through [`crate::build_sandbox`] the way `wasm_builder` does.
//! - Job dir auto-cleanup runs on success and failure via a Drop guard.
//!
//! # Test invocation
//!
//! Tests in `compile-api/tests/cargo_grader.rs` are slow (each call
//! shells out to a real `cargo check`, ~1-3s warm) and are gated with
//! `#[ignore]`. Run them manually:
//!
//! ```text
//! cargo test --workspace -- --ignored
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};

use crate::build_root::{self, JobDirGuard};
use crate::{proc_tree, source_policy};

/// Hard cap on stderr capture from `cargo check`. Anything beyond this
/// is truncated with a marker.
const STDERR_CAP_BYTES: usize = 16 * 1024;

/// Wall-clock timeout for the cargo check subprocess tree.
const CARGO_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

/// Root dir name under the system temp dir.
const ROOT_NAME: &str = "pledge-cargo-grader";

/// Verdict from a real `cargo check` invocation.
#[derive(Debug, Clone)]
pub struct CargoVerdict {
    /// True iff `cargo check` exited 0.
    pub ok: bool,
    /// On success: stdout (usually empty for --quiet).
    /// On failure: cargo stderr, capped at [`STDERR_CAP_BYTES`], or the
    /// deny-list message.
    pub stderr: String,
}

/// Compile-check player source against a server-owned, dependency-free
/// crate skeleton. See module docs for the security contract.
///
/// Returns `Err` only for setup-time failures (empty source, can't
/// create the job dir, can't spawn cargo). A failed `cargo check` is a
/// successful call returning `Ok(CargoVerdict { ok: false, .. })`.
pub fn compile_check(source: &str) -> Result<CargoVerdict> {
    Ok(compile_check_inner(source)?.0)
}

/// Test-only hook: run a compile_check and return the job dir path
/// AFTER cleanup so a test can verify the directory no longer exists.
#[doc(hidden)]
pub fn compile_check_with_sandbox_path(source: &str) -> Result<(CargoVerdict, PathBuf)> {
    let (verdict, path) = compile_check_inner(source)?;
    let path = path.ok_or_else(|| anyhow!("cargo_grader: no job dir was created"))?;
    Ok((verdict, path))
}

fn compile_check_inner(source: &str) -> Result<(CargoVerdict, Option<PathBuf>)> {
    if source.trim().is_empty() {
        return Err(anyhow!("source is empty or whitespace-only"));
    }
    if let Err(violation) = source_policy::check(source) {
        tracing::warn!(
            kind = violation.kind(),
            "cargo_grader: source refused by deny-list: {violation}"
        );
        return Ok((
            CargoVerdict {
                ok: false,
                stderr: violation.player_message(),
            },
            None,
        ));
    }
    let root = build_root::default_root(ROOT_NAME);
    let job = build_root::create_job_dir(&root).context("cargo_grader: job dir")?;
    let path_for_caller = job.clone();
    let verdict = {
        let guard = JobDirGuard::new(job);
        write_skeleton(guard.path(), source)?;
        run_cargo_check(guard.path())?
    };
    Ok((verdict, Some(path_for_caller)))
}

fn write_skeleton(job: &Path, source: &str) -> Result<()> {
    // Server-owned manifest. Frozen, no dependencies, edition 2024.
    // Player input does NOT flow into this string.
    let manifest = r#"[package]
name = "player_submission"
version = "0.0.0"
edition = "2024"
publish = false

[lib]
crate-type = ["rlib"]
"#;
    fs::write(job.join("Cargo.toml"), manifest)
        .context("cargo_grader: failed to write Cargo.toml")?;

    // Wrap player source with a top-level allow attr so a missing
    // `main` / unused vars don't cause spurious lint failures, but
    // do NOT modify the player source otherwise. The attr is on its
    // own line preceding the verbatim source.
    let mut lib_rs = String::with_capacity(source.len() + 64);
    lib_rs.push_str("#![allow(dead_code, unused)]\n");
    lib_rs.push_str(source);
    fs::write(job.join("src").join("lib.rs"), lib_rs)
        .context("cargo_grader: failed to write src/lib.rs")?;
    Ok(())
}

fn run_cargo_check(job: &Path) -> Result<CargoVerdict> {
    let manifest = job.join("Cargo.toml");
    tracing::debug!(
        "cargo_grader: running cargo check on {}",
        manifest.display()
    );
    let mut cmd = Command::new("cargo");
    cmd.arg("check")
        .arg("--offline")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest)
        .current_dir(job);

    // Strip the inherited environment and rebuild it from a small
    // allowlist. Without this, `CARGO_TARGET_DIR` (Matt's machine has
    // it set workspace-wide) would point cargo back at the parent
    // build dir, smuggling state between invocations and breaking
    // cleanup — and the server's secrets would reach rustc.
    cmd.env_clear();
    for name in [
        "PATH",                   // resolve `cargo`, `rustc`, linker
        "CARGO_HOME",             // toolchain cache
        "RUSTUP_HOME",            // rustup root
        "USERPROFILE",            // Windows: HOME analogue
        "HOME",                   // Unix: home
        "TEMP",                   // Windows: temp dir cargo uses for fingerprints
        "TMP",                    // Windows: same, alt name
        "TMPDIR",                 // Unix: temp dir
        "SYSTEMROOT",             // Windows: required for some msvc tooling
        "PROCESSOR_ARCHITECTURE", // Windows: rustc host detection
    ] {
        if let Some(v) = std::env::var_os(name) {
            cmd.env(name, v);
        }
    }
    cmd.env("CARGO_TARGET_DIR", job.join("target"));
    cmd.env("CARGO_TERM_COLOR", "never");

    let out = proc_tree::run_captured(cmd, CARGO_CHECK_TIMEOUT, STDERR_CAP_BYTES, "cargo check")
        .context("cargo_grader: cargo check supervision failed")?;
    tracing::debug!(
        "cargo_grader: cargo check exit={:?} elapsed={:?} timed_out={}",
        out.status.code(),
        out.elapsed,
        out.timed_out
    );

    if out.timed_out {
        return Ok(CargoVerdict {
            ok: false,
            stderr: format!(
                "cargo check timed out after {}s\n--- partial stderr ---\n{}",
                CARGO_CHECK_TIMEOUT.as_secs(),
                out.stderr
            ),
        });
    }
    if out.status.success() {
        Ok(CargoVerdict {
            ok: true,
            stderr: out.stdout,
        })
    } else {
        Ok(CargoVerdict {
            ok: false,
            stderr: out.stderr,
        })
    }
}
