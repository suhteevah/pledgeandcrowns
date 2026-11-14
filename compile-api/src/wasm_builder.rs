// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real wasm build of player source — the bridge between
//! [`crate::cargo_grader`] (which only `cargo check`s an rlib) and
//! [`crate::wasm_runner`] (which executes raw `.wasm` bytes).
//!
//! `build_wasm` takes player Rust source, wraps it in a server-owned
//! **binary** crate skeleton, and runs
//! `cargo build --target wasm32-wasip1 --release --offline` to produce a
//! runnable `.wasm`. [`compile_and_run`] then hands those bytes to the
//! hardened sandbox in [`crate::wasm_runner`] for an honest "does it
//! compile AND run?" verdict.
//!
//! # Security contract (per `design/05-tech-architecture.md` §2)
//!
//! - Server owns the manifest. Player input goes ONLY into `src/main.rs`.
//!   It never touches `Cargo.toml`, never appears as a path component,
//!   never appears as a process arg.
//! - Fresh UUID-v4 job dir per call under a checked build root (see
//!   [`crate::build_root`]); the dir name is never derived from input.
//! - Before any build, the source is checked against the compile-time
//!   file-read deny-list ([`crate::source_policy`]) — the second layer.
//! - The build runs in the filesystem sandbox selected by
//!   `PNC_BUILD_SANDBOX` ([`crate::build_sandbox`]) — the first layer —
//!   with an exact, allow-listed environment and a job-local
//!   `CARGO_TARGET_DIR`.
//! - A wall-clock watchdog kills the whole build process tree
//!   ([`crate::proc_tree`]) and returns only once it is confirmed dead;
//!   output capture is bounded.
//! - The built module is size-capped before it is read into memory.
//! - Job dir auto-cleanup via a Drop guard on every exit path, after the
//!   process tree is dead.
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
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};

use crate::build_root::{self, JobDirGuard};
use crate::build_sandbox::{self, SandboxSettings, WASIP1_TARGET};
use crate::proc_tree;
use crate::source_policy;
use crate::wasm_runner::{self, MAX_WASM_BYTES, RunVerdict};

/// Hard cap on stdout/stderr capture from `cargo build`.
const STDERR_CAP_BYTES: usize = 16 * 1024;

/// Wall-clock timeout for the cargo build subprocess tree. A
/// dependency-free student crate links in ~1-3s warm against the prebuilt
/// wasip1 std; 30s is generous headroom for a cold linker without letting
/// a pathological-macro submission hog a worker indefinitely.
pub const CARGO_BUILD_TIMEOUT: Duration = Duration::from_secs(30);

// ---- Sandbox execution budget for the run phase --------------------------
// These bound the *execution* of the built wasm in `wasm_runner`. Tuned for
// the student curriculum: programs print a line or two and exit. Generous
// enough that legitimate solutions never trip them; tight enough that an
// infinite loop traps in well under a second of CPU.

/// Wall-clock cap on a single wasm run.
pub const RUN_TIMEOUT_MS: u64 = 5_000;
/// Fuel budget (~instruction count). 2e9 traps a tight infinite loop in
/// a fraction of a second while leaving ample room for real solutions.
pub const RUN_FUEL: u64 = 2_000_000_000;
/// Default linear-memory cap for the run: 16 MiB, the design/05 §2
/// value. A `wasm32-wasip1` Rust binary declares ~1.1 MiB of initial
/// memory (1 MiB shadow stack + data), so curriculum programs fit with a
/// wide margin. Configurable via `PNC_RUN_MEMORY_MIB`.
pub const DEFAULT_RUN_MAX_MEMORY_BYTES: u64 = 16 * 1024 * 1024;
/// Ceiling on the configurable memory cap: the previous hard-coded
/// 128 MiB. Configuration may tighten the limit, never loosen it past
/// what shipped before.
pub const MAX_RUN_MAX_MEMORY_BYTES: u64 = 128 * 1024 * 1024;

/// Execution budget handed to [`wasm_runner::run_wasm`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunLimits {
    pub timeout_ms: u64,
    pub fuel: u64,
    pub max_memory_bytes: u64,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            timeout_ms: RUN_TIMEOUT_MS,
            fuel: RUN_FUEL,
            max_memory_bytes: DEFAULT_RUN_MAX_MEMORY_BYTES,
        }
    }
}

/// Where and how builds run. Built from [`crate::Config`] by the server;
/// [`BuildSettings::default`] is the dev setup (temp-dir root, no
/// sandbox, deny-list on).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSettings {
    pub root: PathBuf,
    pub sandbox: SandboxSettings,
    pub source_denylist: bool,
}

impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            root: build_root::default_root(crate::config::DEFAULT_BUILD_ROOT_NAME),
            sandbox: SandboxSettings::default(),
            source_denylist: true,
        }
    }
}

impl BuildSettings {
    pub fn from_config(cfg: &crate::Config) -> Self {
        Self {
            root: cfg.build_root.clone(),
            sandbox: cfg.sandbox.clone(),
            source_denylist: cfg.source_denylist,
        }
    }
}

/// Verdict from a single `build_wasm` invocation.
#[derive(Debug, Clone)]
pub struct BuildVerdict {
    /// True iff `cargo build` exited 0 and the artifact was read.
    pub ok: bool,
    /// On failure: cargo stderr (compiler diagnostics), capped at
    /// [`STDERR_CAP_BYTES`], or the deny-list message. On success:
    /// usually empty.
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
    /// Wall time spent in `cargo build` (setup included).
    pub build_time: Duration,
    /// Wall time of the sandboxed run; `None` when the build failed.
    pub run_time: Option<Duration>,
}

impl RealOutcome {
    /// The single boolean the grader cares about: it compiled AND ran.
    pub fn ok(&self) -> bool {
        self.compiled && self.ran_ok
    }
}

/// Build player `source` to a `wasm32-wasip1` module with the dev
/// [`BuildSettings`].
pub fn build_wasm(source: &str) -> Result<BuildVerdict> {
    build_wasm_in(source, &BuildSettings::default())
}

/// Build player `source` to a `wasm32-wasip1` module. See module docs for
/// the security contract.
///
/// Returns `Err` only for setup-time failures (empty source, can't create
/// the job dir, can't spawn the build, process tree would not die). A
/// failed `cargo build` or a deny-listed source is a *successful* call
/// returning `Ok(BuildVerdict { ok: false, .. })`.
pub fn build_wasm_in(source: &str, settings: &BuildSettings) -> Result<BuildVerdict> {
    Ok(build_wasm_inner(source, settings)?.0)
}

/// Test-only hook: run a `build_wasm` and return the job dir path AFTER
/// cleanup so a test can assert the directory no longer exists.
#[doc(hidden)]
pub fn build_wasm_with_sandbox_path(source: &str) -> Result<(BuildVerdict, PathBuf)> {
    let (verdict, path) = build_wasm_inner(source, &BuildSettings::default())?;
    let path = path.ok_or_else(|| anyhow!("wasm_builder: no job dir was created"))?;
    Ok((verdict, path))
}

fn build_wasm_inner(
    source: &str,
    settings: &BuildSettings,
) -> Result<(BuildVerdict, Option<PathBuf>)> {
    if source.trim().is_empty() {
        return Err(anyhow!("source is empty or whitespace-only"));
    }
    if settings.source_denylist
        && let Err(violation) = source_policy::check(source)
    {
        tracing::warn!(
            kind = violation.kind(),
            "wasm_builder: source refused by deny-list: {violation}"
        );
        return Ok((
            BuildVerdict {
                ok: false,
                stderr: violation.player_message(),
                wasm: None,
            },
            None,
        ));
    }

    let job = build_root::create_job_dir(&settings.root)
        .context("wasm_builder: failed to create job dir")?;
    let path_for_caller = job.clone();
    let verdict = {
        // Dropped after run_cargo_build returns, i.e. after the build's
        // process tree is confirmed dead.
        let guard = JobDirGuard::new(job);
        write_bin_skeleton(guard.path(), source)?;
        run_cargo_build(guard.path(), &settings.sandbox)?
    };
    Ok((verdict, Some(path_for_caller)))
}

/// Build player source to wasm and, if it compiled, run it under the
/// hardened sandbox with the default [`RunLimits`] and dev settings.
///
/// Returns `Err` only for build-setup failures; a non-compiling or
/// trapping submission is a successful call with the relevant booleans
/// set false.
pub fn compile_and_run(source: &str) -> Result<RealOutcome> {
    compile_and_run_with(source, &RunLimits::default())
}

/// [`compile_and_run`] with an explicit execution budget (dev settings).
pub fn compile_and_run_with(source: &str, limits: &RunLimits) -> Result<RealOutcome> {
    compile_and_run_in(source, limits, &BuildSettings::default())
}

/// The server's build-and-run path. Blocking: it spawns the build and
/// runs wasmtime synchronously, so async callers must put it on a
/// blocking thread (the HTTP route uses `spawn_blocking`).
pub fn compile_and_run_in(
    source: &str,
    limits: &RunLimits,
    settings: &BuildSettings,
) -> Result<RealOutcome> {
    let build_started = Instant::now();
    let build = build_wasm_in(source, settings)?;
    let build_time = build_started.elapsed();
    let Some(bytes) = build.wasm else {
        tracing::debug!("wasm_builder: build failed after {build_time:?}, skipping run");
        return Ok(RealOutcome {
            compiled: false,
            ran_ok: false,
            stdout: String::new(),
            stderr: build.stderr,
            build_time,
            run_time: None,
        });
    };

    let run_started = Instant::now();
    let RunVerdict { ok, stdout, stderr } = wasm_runner::run_wasm(
        &bytes,
        limits.timeout_ms,
        limits.fuel,
        limits.max_memory_bytes,
    )?;
    let run_time = run_started.elapsed();
    tracing::debug!(
        "wasm_builder: run complete ok={ok} build={build_time:?} run={run_time:?} mem_cap={}",
        limits.max_memory_bytes
    );
    Ok(RealOutcome {
        compiled: true,
        ran_ok: ok,
        stdout,
        stderr,
        build_time,
        run_time: Some(run_time),
    })
}

/// Startup self-test: for `bwrap`, the isolation probe in
/// [`build_sandbox::probe_isolation`]; for every mode, a real build and
/// run of a trivial program through the exact production path, which
/// proves the toolchain, the binds and the rlimits work together.
pub fn startup_self_test(settings: &BuildSettings) -> Result<()> {
    if settings.sandbox.mode == build_sandbox::SandboxMode::Bwrap {
        build_sandbox::probe_isolation(&settings.sandbox, &settings.root)
            .context("build sandbox isolation probe failed")?;
    }
    let outcome = compile_and_run_in(
        "fn main() { println!(\"pnc-self-test\"); }",
        &RunLimits::default(),
        settings,
    )
    .context("build self-test could not run")?;
    if !outcome.ok() || !outcome.stdout.contains("pnc-self-test") {
        return Err(anyhow!(
            "build self-test failed (compiled={}, ran_ok={}): {}",
            outcome.compiled,
            outcome.ran_ok,
            outcome.stderr
        ));
    }
    tracing::info!(
        sandbox = %settings.sandbox.mode,
        build_ms = outcome.build_time.as_millis() as u64,
        "build self-test passed"
    );
    Ok(())
}

fn write_bin_skeleton(job: &Path, source: &str) -> Result<()> {
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
    fs::write(job.join("Cargo.toml"), manifest)
        .context("wasm_builder: failed to write Cargo.toml")?;

    // Prefix a crate-level allow so unused bindings in a student snippet
    // don't fail the build on lints; do NOT otherwise modify the source.
    // The attr is an inner attribute and must be the first thing in the
    // crate — player inner attributes (if any) follow it, which is legal.
    let mut main_rs = String::with_capacity(source.len() + 64);
    main_rs.push_str("#![allow(dead_code, unused)]\n");
    main_rs.push_str(source);
    fs::write(job.join("src").join("main.rs"), main_rs)
        .context("wasm_builder: failed to write src/main.rs")?;
    Ok(())
}

fn run_cargo_build(job: &Path, sandbox: &SandboxSettings) -> Result<BuildVerdict> {
    let spec = build_sandbox::cargo_build_spec(sandbox, job)
        .context("wasm_builder: could not construct the build command")?;
    tracing::debug!(
        mode = %sandbox.mode,
        argv = ?spec.argv_lossy(),
        "wasm_builder: cargo build wasip1 in {}",
        job.display()
    );
    let out = proc_tree::run_captured(
        spec.to_command(),
        CARGO_BUILD_TIMEOUT,
        STDERR_CAP_BYTES,
        "cargo build",
    )
    .context("wasm_builder: build process supervision failed")?;
    tracing::debug!(
        "wasm_builder: cargo build exit={:?} elapsed={:?} timed_out={}",
        out.status.code(),
        out.elapsed,
        out.timed_out
    );

    if out.timed_out {
        return Ok(BuildVerdict {
            ok: false,
            stderr: format!(
                "cargo build timed out after {}s\n--- partial stderr ---\n{}",
                CARGO_BUILD_TIMEOUT.as_secs(),
                out.stderr
            ),
            wasm: None,
        });
    }

    if !out.status.success() {
        return Ok(BuildVerdict {
            ok: false,
            // On a build failure cargo's diagnostics are on stderr;
            // forward stdout too on the off chance something useful
            // landed there.
            stderr: if out.stderr.is_empty() {
                out.stdout
            } else {
                out.stderr
            },
            wasm: None,
        });
    }

    // Read the artifact. Bin target `player_submission` -> `<name>.wasm`
    // under the target/<triple>/release dir.
    let artifact = job
        .join("target")
        .join(WASIP1_TARGET)
        .join("release")
        .join("player_submission.wasm");
    let len = match fs::metadata(&artifact) {
        Ok(m) => m.len(),
        Err(e) => {
            // Built clean but no artifact — almost always "no `main`"
            // (a lib-shaped snippet built as a bin). Report it honestly
            // rather than pretending the run can proceed.
            tracing::warn!(
                "wasm_builder: build succeeded but artifact missing at {}: {e}",
                artifact.display()
            );
            return Ok(BuildVerdict {
                ok: false,
                stderr: format!("build produced no runnable binary (is there a `fn main`?): {e}"),
                wasm: None,
            });
        }
    };
    if len > MAX_WASM_BYTES as u64 {
        tracing::warn!(
            bytes = len,
            limit = MAX_WASM_BYTES,
            "wasm_builder: rejected oversized artifact"
        );
        return Ok(BuildVerdict {
            ok: false,
            stderr: format!("built module is {len} bytes, over the {MAX_WASM_BYTES}-byte limit"),
            wasm: None,
        });
    }
    let bytes = fs::read(&artifact)
        .with_context(|| format!("wasm_builder: failed to read {}", artifact.display()))?;
    tracing::debug!("wasm_builder: read {} wasm bytes", bytes.len());
    Ok(BuildVerdict {
        ok: true,
        stderr: out.stdout,
        wasm: Some(bytes),
    })
}
