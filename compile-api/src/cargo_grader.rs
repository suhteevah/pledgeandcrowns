// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real `cargo check` grader. Drop-in successor to the v0 pattern grader
//! in `grader.rs`, but NOT yet wired into the HTTP route — exposed only
//! as a public function so the security model can be reviewed and
//! regression-tested in isolation.
//!
//! # Security contract (per `design/05-tech-architecture.md` §2)
//!
//! - Server owns the manifest. Player input goes ONLY into `src/lib.rs`.
//!   It never touches `Cargo.toml`, never appears as a path component,
//!   never appears as a process arg.
//! - Each invocation gets a fresh sandbox at
//!   `<temp>/pledge-cargo-grader/<uuid>/`. The directory name is a
//!   freshly-generated UUID v4 — never derived from input.
//! - `cargo check --offline` is run with a 10s wall-clock watchdog.
//!   stderr capture is bounded at 16 KiB so a pathological diagnostic
//!   firehose can't blow memory.
//! - Sandbox auto-cleanup runs on success and failure via a Drop guard.
//!
//! # Test invocation
//!
//! Tests in `compile-api/tests/cargo_grader.rs` are slow (each call
//! shells out to a real `cargo check`, ~1-3s warm) and are gated with
//! `#[ignore]`. They do NOT run in the default `cargo test --workspace`
//! pre-commit pipeline. Run them manually:
//!
//! ```text
//! cargo test --workspace -- --ignored
//! ```

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use uuid::Uuid;

/// Hard cap on stderr capture from `cargo check`. Anything beyond this
/// is truncated with a marker.
const STDERR_CAP_BYTES: usize = 16 * 1024;

/// Wall-clock timeout for the cargo check subprocess.
const CARGO_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

/// Verdict from a real `cargo check` invocation.
#[derive(Debug, Clone)]
pub struct CargoVerdict {
    /// True iff `cargo check` exited 0.
    pub ok: bool,
    /// On success: stdout (usually empty for --quiet).
    /// On failure: cargo stderr, capped at [`STDERR_CAP_BYTES`].
    pub stderr: String,
}

/// RAII guard that deletes the sandbox directory on drop, regardless
/// of how the calling function exits.
struct SandboxGuard {
    path: PathBuf,
}

impl SandboxGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for SandboxGuard {
    fn drop(&mut self) {
        if self.path.exists()
            && let Err(e) = fs::remove_dir_all(&self.path)
        {
            tracing::warn!(
                "cargo_grader: failed to clean sandbox {}: {}",
                self.path.display(),
                e
            );
        }
    }
}

/// Compile-check player source against a server-owned, dependency-free
/// crate skeleton. See module docs for the security contract.
///
/// Returns `Err` only for setup-time failures (empty source, can't
/// create tempdir, can't spawn cargo). A failed `cargo check` is a
/// successful call returning `Ok(CargoVerdict { ok: false, .. })`.
pub fn compile_check(source: &str) -> Result<CargoVerdict> {
    if source.trim().is_empty() {
        return Err(anyhow!("source is empty or whitespace-only"));
    }

    let sandbox = create_sandbox()?;
    let _guard = SandboxGuard::new(sandbox.clone());

    write_skeleton(&sandbox, source)?;

    let manifest = sandbox.join("Cargo.toml");
    run_cargo_check(&manifest)
}

/// Test-only hook: run a compile_check and return the sandbox path
/// AFTER cleanup so a test can verify the directory no longer exists.
#[doc(hidden)]
pub fn compile_check_with_sandbox_path(source: &str) -> Result<(CargoVerdict, PathBuf)> {
    if source.trim().is_empty() {
        return Err(anyhow!("source is empty or whitespace-only"));
    }
    let sandbox = create_sandbox()?;
    let path_for_caller = sandbox.clone();
    let verdict = {
        let _guard = SandboxGuard::new(sandbox.clone());
        write_skeleton(&sandbox, source)?;
        run_cargo_check(&sandbox.join("Cargo.toml"))?
    };
    Ok((verdict, path_for_caller))
}

fn create_sandbox() -> Result<PathBuf> {
    let root = std::env::temp_dir().join("pledge-cargo-grader");
    fs::create_dir_all(&root).with_context(|| {
        format!(
            "cargo_grader: failed to create sandbox root {}",
            root.display()
        )
    })?;
    let dir = root.join(Uuid::new_v4().to_string());
    fs::create_dir(&dir).with_context(|| {
        format!(
            "cargo_grader: failed to create sandbox dir {}",
            dir.display()
        )
    })?;
    fs::create_dir(dir.join("src"))
        .with_context(|| format!("cargo_grader: failed to create src/ in {}", dir.display()))?;
    tracing::debug!("cargo_grader: created sandbox {}", dir.display());
    Ok(dir)
}

fn write_skeleton(sandbox: &Path, source: &str) -> Result<()> {
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
    fs::write(sandbox.join("Cargo.toml"), manifest)
        .context("cargo_grader: failed to write Cargo.toml")?;

    // Wrap player source with a top-level allow attr so a missing
    // `main` / unused vars don't cause spurious lint failures, but
    // do NOT modify the player source otherwise. The attr is on its
    // own line preceding the verbatim source.
    let mut lib_rs = String::with_capacity(source.len() + 64);
    lib_rs.push_str("#![allow(dead_code, unused)]\n");
    lib_rs.push_str(source);
    fs::write(sandbox.join("src").join("lib.rs"), lib_rs)
        .context("cargo_grader: failed to write src/lib.rs")?;
    Ok(())
}

fn run_cargo_check(manifest_path: &Path) -> Result<CargoVerdict> {
    let manifest_str = manifest_path
        .to_str()
        .ok_or_else(|| anyhow!("cargo_grader: manifest path not valid UTF-8"))?;

    // The sandbox is the manifest's parent. We anchor cwd + the build
    // target dir there so the parent process's `CARGO_TARGET_DIR` (and
    // any other cargo-flavored env vars) cannot leak into the player
    // build — that would defeat the per-invocation isolation premise.
    let sandbox = manifest_path
        .parent()
        .ok_or_else(|| anyhow!("cargo_grader: manifest path has no parent"))?;
    let sandbox_target = sandbox.join("target");

    tracing::debug!("cargo_grader: running cargo check on {}", manifest_str);
    let started = Instant::now();
    let mut cmd = Command::new("cargo");
    cmd.arg("check")
        .arg("--offline")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_str)
        .current_dir(sandbox)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Strip the inherited environment and rebuild it from a small
    // allowlist. Without this, `CARGO_TARGET_DIR` (Matt's machine has
    // it set workspace-wide) would point cargo back at the parent
    // build dir, smuggling state between invocations and breaking
    // sandbox cleanup.
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
        if let Ok(v) = std::env::var(name) {
            cmd.env(name, v);
        }
    }
    // Pin the cargo build dir to the sandbox so it lives and dies
    // with the rest of the per-invocation state.
    cmd.env("CARGO_TARGET_DIR", &sandbox_target);
    // Disable colored diagnostics so captured stderr is clean text
    // rather than a stream of ANSI escapes the player would see in
    // the editor.
    cmd.env("CARGO_TERM_COLOR", "never");

    let mut child = cmd
        .spawn()
        .context("cargo_grader: failed to spawn cargo check")?;

    // Read stdout/stderr on dedicated threads with a bounded buffer so
    // a pathological diagnostic firehose can't OOM us. Each reader
    // signals on `tx` when its pipe EOFs — i.e. when cargo's
    // corresponding stream has closed. The watchdog uses these signals
    // to wake immediately on a fast-finishing child rather than busy-
    // polling `try_wait`.
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let (tx, rx) = mpsc::channel::<()>();
    let tx_stdout = tx.clone();
    let tx_stderr = tx.clone();
    let stdout_handle = thread::spawn(move || {
        let s = read_capped(stdout);
        let _ = tx_stdout.send(());
        s
    });
    let stderr_handle = thread::spawn(move || {
        let s = read_capped(stderr);
        let _ = tx_stderr.send(());
        s
    });
    // Drop the original tx so `rx.recv_timeout` returns Disconnected
    // once both readers finish (rather than blocking forever on a
    // sender that nobody else holds).
    drop(tx);

    // Watchdog: block on the reader signals until either both pipes
    // EOF (child exited / streams closed — fast path on quick runs) or
    // the wall-clock budget elapses. On the slow path we give up and
    // kill the child. This is non-cancellable-sleep-free: a 5 ms run
    // joins in 5 ms, not 10 s.
    let mut killed_for_timeout = false;
    loop {
        let elapsed = started.elapsed();
        if elapsed >= CARGO_CHECK_TIMEOUT {
            tracing::warn!(
                "cargo_grader: timing out cargo check after {:?}",
                CARGO_CHECK_TIMEOUT
            );
            let _ = child.kill();
            killed_for_timeout = true;
            break;
        }
        let remaining = CARGO_CHECK_TIMEOUT - elapsed;
        match rx.recv_timeout(remaining) {
            Ok(()) => {
                // One pipe closed; loop and wait for the other (or for
                // the channel to disconnect, signalling both done).
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Both reader threads dropped their tx ends -> child's
                // stdout AND stderr have closed. The child has exited
                // or is about to; fall through to the blocking wait.
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                tracing::warn!(
                    "cargo_grader: timing out cargo check after {:?}",
                    CARGO_CHECK_TIMEOUT
                );
                let _ = child.kill();
                killed_for_timeout = true;
                break;
            }
        }
    }
    let status = child
        .wait()
        .context("cargo_grader: wait after pipes closed / kill")?;

    let stdout_buf = stdout_handle
        .join()
        .map_err(|_| anyhow!("cargo_grader: stdout reader thread panicked"))?;
    let stderr_buf = stderr_handle
        .join()
        .map_err(|_| anyhow!("cargo_grader: stderr reader thread panicked"))?;

    let elapsed = started.elapsed();
    tracing::debug!(
        "cargo_grader: cargo check exit={:?} elapsed={:?} ok_path={}",
        status.code(),
        elapsed,
        status.success()
    );

    if killed_for_timeout {
        return Ok(CargoVerdict {
            ok: false,
            stderr: format!(
                "cargo check timed out after {}s\n--- partial stderr ---\n{}",
                CARGO_CHECK_TIMEOUT.as_secs(),
                stderr_buf
            ),
        });
    }

    if status.success() {
        Ok(CargoVerdict {
            ok: true,
            // On --quiet success cargo prints essentially nothing.
            // Forward stdout (usually empty) for completeness.
            stderr: stdout_buf,
        })
    } else {
        Ok(CargoVerdict {
            ok: false,
            stderr: stderr_buf,
        })
    }
}

/// Read up to [`STDERR_CAP_BYTES`] from `r`, then drain the rest into
/// the void so the writer doesn't block on a full pipe. Returns lossy
/// UTF-8 of the captured prefix.
fn read_capped<R: Read>(mut r: R) -> String {
    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    let mut truncated = false;
    loop {
        match r.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if buf.len() < STDERR_CAP_BYTES {
                    let remaining = STDERR_CAP_BYTES - buf.len();
                    let take = n.min(remaining);
                    buf.extend_from_slice(&chunk[..take]);
                    if take < n {
                        truncated = true;
                    }
                } else {
                    truncated = true;
                }
            }
            Err(_) => break,
        }
    }
    let mut out = String::from_utf8_lossy(&buf).into_owned();
    if truncated {
        out.push_str("\n... [truncated]");
    }
    out
}
