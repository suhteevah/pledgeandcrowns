// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real wasm build of player source — the bridge between
//! [`crate::cargo_grader`] (which only `cargo check`s an rlib) and
//! [`crate::wasm_runner`] (which executes raw `.wasm` bytes).
//!
//! `build_wasm` takes player Rust source, wraps it in a server-owned
//! **binary** crate skeleton, and runs
//! `cargo build --target wasm32-wasip1 --release` to produce a runnable
//! `.wasm`. [`compile_and_run`] then hands those bytes to the hardened
//! sandbox in [`crate::wasm_runner`] for an honest "does it compile AND
//! run?" verdict.
//!
//! # Security contract (per `design/05-tech-architecture.md` §2)
//!
//! Identical to [`crate::cargo_grader`]:
//! - Server owns the manifest. Player input goes ONLY into `src/main.rs`.
//!   It never touches `Cargo.toml`, never appears as a path component,
//!   never appears as a process arg.
//! - Fresh UUID-v4 sandbox per call at
//!   `<temp>/pledge-wasm-builder/<uuid>/`; the dir name is never derived
//!   from input.
//! - The cargo subprocess runs with a stripped, allow-listed environment
//!   and a sandbox-pinned `CARGO_TARGET_DIR` so the parent build state
//!   can't leak in (or out).
//! - Wall-clock watchdog kills a runaway build; stderr capture is bounded.
//! - Sandbox auto-cleanup via a Drop guard on every exit path.
//!
//! The sandbox + watchdog logic deliberately mirrors
//! `cargo_grader::run_cargo_check` rather than sharing it: the build
//! target (`wasm32-wasip1`, `bin` crate-type) and the check target
//! (host, `rlib`) answer different questions and are reviewed
//! independently. The duplication is the seam.
//!
//! # Test invocation
//!
//! Tests in `compile-api/tests/wasm_builder.rs` are slow (each shells out
//! to a real `cargo build` for `wasm32-wasip1`) and are `#[ignore]`d.
//! They also require the target to be installed:
//!
//! ```text
//! rustup target add wasm32-wasip1
//! cargo test -p pledge_compile_api --test wasm_builder -- --ignored
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

use crate::wasm_runner::{self, RunVerdict};

/// Rust target triple the player crate is built for. WASI preview1 so the
/// emitted module is `_start`-shaped and runnable by
/// [`crate::wasm_runner`]'s p1 linker.
const WASIP1_TARGET: &str = "wasm32-wasip1";

/// Hard cap on stderr capture from `cargo build`.
const STDERR_CAP_BYTES: usize = 16 * 1024;

/// Wall-clock timeout for the cargo build subprocess. A dependency-free
/// student crate links in ~1-3s warm against the prebuilt wasip1 std;
/// 30s is generous headroom for a cold linker without letting a
/// pathological-macro submission hog a worker indefinitely.
const CARGO_BUILD_TIMEOUT: Duration = Duration::from_secs(30);

// ---- Sandbox execution budget for the run phase --------------------------
// These bound the *execution* of the built wasm in `wasm_runner`. Tuned for
// the student curriculum: programs print a line or two and exit. Generous
// enough that legitimate solutions never trip them; tight enough that an
// infinite loop traps in well under a second of CPU.

/// Wall-clock cap on a single wasm run.
const RUN_TIMEOUT_MS: u64 = 5_000;
/// Fuel budget (~instruction count). 2e9 traps a tight infinite loop in
/// a fraction of a second while leaving ample room for real solutions.
const RUN_FUEL: u64 = 2_000_000_000;
/// Linear-memory cap for the run. 128 MiB.
const RUN_MAX_MEMORY_BYTES: u64 = 128 * 1024 * 1024;

/// Verdict from a single `build_wasm` invocation.
#[derive(Debug, Clone)]
pub struct BuildVerdict {
    /// True iff `cargo build` exited 0 and the artifact was read.
    pub ok: bool,
    /// On failure: cargo stderr (compiler diagnostics), capped at
    /// [`STDERR_CAP_BYTES`]. On success: usually empty.
    pub stderr: String,
    /// The built module bytes on success; `None` on any failure.
    pub wasm: Option<Vec<u8>>,
}

/// Combined compile-and-execute verdict for the `/compile-real` route.
#[derive(Debug, Clone)]
pub struct RealOutcome {
    /// True iff the player source built to a wasm artifact.
    pub compiled: bool,
    /// True iff the artifact then ran to a clean exit (no trap, no
    /// non-zero exit, no fuel/timeout). Meaningless when `compiled` is
    /// false.
    pub ran_ok: bool,
    /// Program stdout on a successful run; empty otherwise.
    pub stdout: String,
    /// Compiler diagnostics (build failure) or trap/exit text + captured
    /// program stderr (run failure).
    pub stderr: String,
}

impl RealOutcome {
    /// The single boolean the grader cares about: it compiled AND ran.
    pub fn ok(&self) -> bool {
        self.compiled && self.ran_ok
    }
}

/// RAII guard that deletes the sandbox directory on drop.
struct SandboxGuard {
    path: PathBuf,
}

impl Drop for SandboxGuard {
    fn drop(&mut self) {
        if self.path.exists()
            && let Err(e) = fs::remove_dir_all(&self.path)
        {
            tracing::warn!(
                "wasm_builder: failed to clean sandbox {}: {}",
                self.path.display(),
                e
            );
        }
    }
}

/// Build player `source` to a `wasm32-wasip1` module. See module docs for
/// the security contract.
///
/// Returns `Err` only for setup-time failures (empty source, can't create
/// tempdir, can't spawn cargo). A failed `cargo build` is a *successful*
/// call returning `Ok(BuildVerdict { ok: false, .. })`.
pub fn build_wasm(source: &str) -> Result<BuildVerdict> {
    if source.trim().is_empty() {
        return Err(anyhow!("source is empty or whitespace-only"));
    }

    let sandbox = create_sandbox()?;
    let _guard = SandboxGuard {
        path: sandbox.clone(),
    };

    write_bin_skeleton(&sandbox, source)?;
    run_cargo_build(&sandbox)
}

/// Test-only hook: run a `build_wasm` and return the sandbox path AFTER
/// cleanup so a test can assert the directory no longer exists.
#[doc(hidden)]
pub fn build_wasm_with_sandbox_path(source: &str) -> Result<(BuildVerdict, PathBuf)> {
    if source.trim().is_empty() {
        return Err(anyhow!("source is empty or whitespace-only"));
    }
    let sandbox = create_sandbox()?;
    let path_for_caller = sandbox.clone();
    let verdict = {
        let _guard = SandboxGuard {
            path: sandbox.clone(),
        };
        write_bin_skeleton(&sandbox, source)?;
        run_cargo_build(&sandbox)?
    };
    Ok((verdict, path_for_caller))
}

/// Build player source to wasm and, if it compiled, run it under the
/// hardened sandbox. The honest end-to-end grader path.
///
/// Returns `Err` only for build-setup failures; a non-compiling or
/// trapping submission is a successful call with the relevant booleans
/// set false.
pub fn compile_and_run(source: &str) -> Result<RealOutcome> {
    let build = build_wasm(source)?;
    let Some(bytes) = build.wasm else {
        tracing::debug!("wasm_builder: build failed, skipping run");
        return Ok(RealOutcome {
            compiled: false,
            ran_ok: false,
            stdout: String::new(),
            stderr: build.stderr,
        });
    };

    let RunVerdict { ok, stdout, stderr } =
        wasm_runner::run_wasm(&bytes, RUN_TIMEOUT_MS, RUN_FUEL, RUN_MAX_MEMORY_BYTES)?;
    tracing::debug!("wasm_builder: run complete ok={ok}");
    Ok(RealOutcome {
        compiled: true,
        ran_ok: ok,
        stdout,
        stderr,
    })
}

fn create_sandbox() -> Result<PathBuf> {
    let root = std::env::temp_dir().join("pledge-wasm-builder");
    fs::create_dir_all(&root).with_context(|| {
        format!(
            "wasm_builder: failed to create sandbox root {}",
            root.display()
        )
    })?;
    let dir = root.join(Uuid::new_v4().to_string());
    fs::create_dir(&dir).with_context(|| {
        format!(
            "wasm_builder: failed to create sandbox dir {}",
            dir.display()
        )
    })?;
    fs::create_dir(dir.join("src"))
        .with_context(|| format!("wasm_builder: failed to create src/ in {}", dir.display()))?;
    tracing::debug!("wasm_builder: created sandbox {}", dir.display());
    Ok(dir)
}

fn write_bin_skeleton(sandbox: &Path, source: &str) -> Result<()> {
    // Server-owned manifest. Frozen, no dependencies, edition 2024.
    // Player input does NOT flow into this string. `panic = "abort"`
    // keeps panics from unwinding: in wasm a panic becomes a clean trap
    // (the player's panic message is still printed to stderr first), so
    // "your code panicked" surfaces as ran_ok=false rather than a murky
    // partial state. opt-level 1 + codegen-units 16 favors build latency
    // (this is graded interactively) over runtime micro-perf.
    let manifest = r#"[package]
name = "player_submission"
version = "0.0.0"
edition = "2024"
publish = false

[[bin]]
name = "player_submission"
path = "src/main.rs"

[profile.release]
opt-level = 1
codegen-units = 16
lto = false
panic = "abort"
"#;
    fs::write(sandbox.join("Cargo.toml"), manifest)
        .context("wasm_builder: failed to write Cargo.toml")?;

    // Prefix a crate-level allow so unused bindings in a student snippet
    // don't fail the build on lints; do NOT otherwise modify the source.
    // The attr is an inner attribute and must be the first thing in the
    // crate — player inner attributes (if any) follow it, which is legal.
    let mut main_rs = String::with_capacity(source.len() + 64);
    main_rs.push_str("#![allow(dead_code, unused)]\n");
    main_rs.push_str(source);
    fs::write(sandbox.join("src").join("main.rs"), main_rs)
        .context("wasm_builder: failed to write src/main.rs")?;
    Ok(())
}

fn run_cargo_build(sandbox: &Path) -> Result<BuildVerdict> {
    let manifest = sandbox.join("Cargo.toml");
    let manifest_str = manifest
        .to_str()
        .ok_or_else(|| anyhow!("wasm_builder: manifest path not valid UTF-8"))?;
    let sandbox_target = sandbox.join("target");

    tracing::debug!("wasm_builder: cargo build wasip1 in {}", sandbox.display());
    let started = Instant::now();
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg(WASIP1_TARGET)
        .arg("--release")
        .arg("--offline")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_str)
        .current_dir(sandbox)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Strip the inherited environment and rebuild it from a small
    // allowlist (mirrors cargo_grader). Without env_clear, a workspace-
    // wide CARGO_TARGET_DIR would point cargo back at the parent build
    // dir, smuggling state between invocations and breaking cleanup.
    cmd.env_clear();
    for name in [
        "PATH",                   // resolve `cargo`, `rustc`, linker, rust-lld
        "CARGO_HOME",             // toolchain + registry cache
        "RUSTUP_HOME",            // rustup root (resolves the wasip1 std)
        "USERPROFILE",            // Windows: HOME analogue
        "HOME",                   // Unix: home
        "TEMP",                   // Windows: temp dir cargo uses for fingerprints
        "TMP",                    // Windows: same, alt name
        "TMPDIR",                 // Unix: temp dir
        "SYSTEMROOT",             // Windows: required for some toolchain bits
        "PROCESSOR_ARCHITECTURE", // Windows: rustc host detection
    ] {
        if let Ok(v) = std::env::var(name) {
            cmd.env(name, v);
        }
    }
    cmd.env("CARGO_TARGET_DIR", &sandbox_target);
    cmd.env("CARGO_TERM_COLOR", "never");

    let mut child = cmd
        .spawn()
        .context("wasm_builder: failed to spawn cargo build")?;

    // Bounded capture on dedicated threads; signal on EOF so the watchdog
    // wakes immediately on a fast build rather than busy-polling.
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
    drop(tx);

    let mut killed_for_timeout = false;
    loop {
        let elapsed = started.elapsed();
        if elapsed >= CARGO_BUILD_TIMEOUT {
            tracing::warn!(
                "wasm_builder: timing out cargo build after {:?}",
                CARGO_BUILD_TIMEOUT
            );
            let _ = child.kill();
            killed_for_timeout = true;
            break;
        }
        let remaining = CARGO_BUILD_TIMEOUT - elapsed;
        match rx.recv_timeout(remaining) {
            Ok(()) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                tracing::warn!(
                    "wasm_builder: timing out cargo build after {:?}",
                    CARGO_BUILD_TIMEOUT
                );
                let _ = child.kill();
                killed_for_timeout = true;
                break;
            }
        }
    }
    let status = child
        .wait()
        .context("wasm_builder: wait after pipes closed / kill")?;

    let stdout_buf = stdout_handle
        .join()
        .map_err(|_| anyhow!("wasm_builder: stdout reader thread panicked"))?;
    let stderr_buf = stderr_handle
        .join()
        .map_err(|_| anyhow!("wasm_builder: stderr reader thread panicked"))?;

    let elapsed = started.elapsed();
    tracing::debug!(
        "wasm_builder: cargo build exit={:?} elapsed={:?} ok_path={}",
        status.code(),
        elapsed,
        status.success()
    );

    if killed_for_timeout {
        return Ok(BuildVerdict {
            ok: false,
            stderr: format!(
                "cargo build timed out after {}s\n--- partial stderr ---\n{}",
                CARGO_BUILD_TIMEOUT.as_secs(),
                stderr_buf
            ),
            wasm: None,
        });
    }

    if !status.success() {
        return Ok(BuildVerdict {
            ok: false,
            // On a build failure cargo's diagnostics are on stderr;
            // forward stdout too on the off chance something useful
            // landed there.
            stderr: if stderr_buf.is_empty() {
                stdout_buf
            } else {
                stderr_buf
            },
            wasm: None,
        });
    }

    // Read the artifact. Bin target `player_submission` -> `<name>.wasm`
    // under the target/<triple>/release dir.
    let artifact = sandbox_target
        .join(WASIP1_TARGET)
        .join("release")
        .join("player_submission.wasm");
    match fs::read(&artifact) {
        Ok(bytes) => {
            tracing::debug!("wasm_builder: read {} wasm bytes", bytes.len());
            Ok(BuildVerdict {
                ok: true,
                stderr: stdout_buf,
                wasm: Some(bytes),
            })
        }
        Err(e) => {
            // Built clean but no artifact — almost always "no `main`"
            // (a lib-shaped snippet built as a bin). Report it honestly
            // rather than pretending the run can proceed.
            tracing::warn!(
                "wasm_builder: build succeeded but artifact missing at {}: {e}",
                artifact.display()
            );
            Ok(BuildVerdict {
                ok: false,
                stderr: format!("build produced no runnable binary (is there a `fn main`?): {e}"),
                wasm: None,
            })
        }
    }
}

/// Read up to [`STDERR_CAP_BYTES`] from `r`, then drain the rest so the
/// writer never blocks on a full pipe. Returns lossy UTF-8 of the prefix.
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
