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
//! - In `bwrap` mode the module then runs in its own run sandbox, a
//!   separate `run-wasm` child that sees only the server binary and its
//!   libraries ([`crate::run_sandbox`]); in `none` mode it runs in-process.
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
use serde::{Deserialize, Serialize};

use crate::build_root::{self, JobDirGuard};
use crate::build_sandbox::{self, SandboxMode, SandboxSettings, WASIP1_TARGET};
use crate::proc_tree;
use crate::source_policy;
use crate::toolchain;
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

/// Text the server places before the player's source in `src/main.rs`,
/// for plain and harness builds alike. Prefixing a crate-level allow keeps
/// unused bindings in a student snippet from failing the build on lints.
pub const PLAYER_PREAMBLE: &str = "#![allow(dead_code, unused)]\n";

/// Lines in [`PLAYER_PREAMBLE`]. rustc line numbers for player code are
/// this many lines past the player's own; the game client subtracts it
/// (`SERVER_PREAMBLE_LINES`).
pub const PLAYER_PREAMBLE_LINES: usize = 1;

/// Line appended after the player's source in a harness build. It comes
/// after the player's last line, so it never shifts player line numbers.
pub const HARNESS_DECLARATION: &str = "\n#[doc(hidden)]\npub(crate) mod __pnc_harness;\n";

/// How the server lays out the crate around the player's source. In both
/// layouts the player's source is the single file `src/main.rs` (after
/// [`PLAYER_PREAMBLE`]) and never touches the manifest.
#[derive(Debug, Clone, Copy)]
pub enum Layout<'a> {
    /// `src/main.rs` is the crate root; the player's `fn main` runs.
    Plain,
    /// Server entry `src/pnc_entry.rs` is the crate root and declares the
    /// player's file as module `main`; the server checker
    /// `src/main/__pnc_harness.rs` is a child module of the player's code.
    /// See `crate::expectations` for the design.
    Harness {
        /// Server entry crate root source.
        entry: &'a str,
        /// Server checker module source.
        checker: &'a str,
    },
}

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
/// sandbox, deny-list on). Serializable because the builder hands it to
/// each one-shot job process ([`crate::jobs`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

    /// Structural checks for settings received over a process boundary.
    pub fn validate(&self) -> std::result::Result<(), String> {
        if !self.root.is_absolute() {
            return Err(format!(
                "build root {} is not absolute",
                self.root.display()
            ));
        }
        self.sandbox.validate()
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
/// the job dir, can't spawn the build, process tree would not die) and
/// for a toolchain crash ([`ToolchainCrash`]). A
/// failed `cargo build` or a deny-listed source is a *successful* call
/// returning `Ok(BuildVerdict { ok: false, .. })`.
pub fn build_wasm_in(source: &str, settings: &BuildSettings) -> Result<BuildVerdict> {
    Ok(build_wasm_inner(source, &Layout::Plain, settings)?.0)
}

/// Test-only hook: run a `build_wasm` and return the job dir path AFTER
/// cleanup so a test can assert the directory no longer exists.
#[doc(hidden)]
pub fn build_wasm_with_sandbox_path(source: &str) -> Result<(BuildVerdict, PathBuf)> {
    let (verdict, path) = build_wasm_inner(source, &Layout::Plain, &BuildSettings::default())?;
    let path = path.ok_or_else(|| anyhow!("wasm_builder: no job dir was created"))?;
    Ok((verdict, path))
}

fn build_wasm_inner(
    source: &str,
    layout: &Layout<'_>,
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
        write_skeleton(guard.path(), source, layout)?;
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
    compile_and_run_layout(source, &Layout::Plain, true, limits, settings)
}

/// Build `source` in `layout` and, when `run` is true and it compiled,
/// run it in the sandbox. With `run == false` the module is never
/// executed: the outcome has `compiled` set, `ran_ok == false` and
/// `run_time == None` (used for missions that cannot run on wasip1).
/// The source deny-list applies to the player's `source` only; server
/// files in the layout are trusted.
pub fn compile_and_run_layout(
    source: &str,
    layout: &Layout<'_>,
    run: bool,
    limits: &RunLimits,
    settings: &BuildSettings,
) -> Result<RealOutcome> {
    compile_and_run_layout_with(source, layout, run, limits, settings, None)
}

/// [`compile_and_run_layout`] for a late-nonce harness build
/// (`expectations::HarnessMode::UnsafeLateNonce`): with `Some(late)` the
/// run links the one-shot nonce import ([`wasm_runner::LateNonce`]), in
/// process or in the run sandbox alike. `None` is exactly
/// [`compile_and_run_layout`].
pub fn compile_and_run_layout_with(
    source: &str,
    layout: &Layout<'_>,
    run: bool,
    limits: &RunLimits,
    settings: &BuildSettings,
    late: Option<&wasm_runner::LateNonce>,
) -> Result<RealOutcome> {
    let build_started = Instant::now();
    let build = build_wasm_inner(source, layout, settings)?.0;
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
    if !run {
        tracing::debug!("wasm_builder: built in {build_time:?}; run skipped by caller");
        return Ok(RealOutcome {
            compiled: true,
            ran_ok: false,
            stdout: String::new(),
            stderr: build.stderr,
            build_time,
            run_time: None,
        });
    }

    wasm_runner::announce_run_phase();
    let run_started = Instant::now();
    // bwrap mode: wasmtime runs in its own run sandbox (a separate
    // `run-wasm` child, see `run_sandbox`), never with this process's view
    // of the host. `none` (dev/tests/Windows): in-process.
    let RunVerdict { ok, stdout, stderr } = if settings.sandbox.mode == SandboxMode::Bwrap {
        crate::run_sandbox::run_wasm_sandboxed_with(&bytes, limits, &settings.sandbox, late)?
    } else {
        wasm_runner::run_wasm_late_nonce(
            &bytes,
            limits.timeout_ms,
            limits.fuel,
            limits.max_memory_bytes,
            late,
        )?
    };
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
    check_toolchain(&settings.sandbox)?;
    if settings.sandbox.mode == SandboxMode::Bwrap {
        build_sandbox::probe_isolation(&settings.sandbox, &settings.root)
            .context("build sandbox isolation probe failed")?;
    }
    build_self_test(settings)
}

/// Log the rustc version the builds will use and compare it with
/// [`toolchain::PINNED_RUSTC_VERSION`]: a mismatch is fatal in `bwrap`
/// mode, a loud warning in `none` (dev) mode.
pub fn check_toolchain(sandbox: &SandboxSettings) -> Result<()> {
    let found = match toolchain::probe_rustc(sandbox) {
        Ok(v) => v,
        Err(e) if sandbox.mode == SandboxMode::None => {
            tracing::warn!("toolchain check skipped (dev, sandbox none): {e}");
            return Ok(());
        }
        Err(e) => return Err(anyhow!(e).context("toolchain check failed")),
    };
    match (toolchain::check_version(&found), sandbox.mode) {
        (Ok(()), _) => {
            tracing::info!(rustc = %found, "toolchain: matches the pinned version");
            Ok(())
        }
        (Err(e), SandboxMode::Bwrap) => {
            tracing::error!("refusing: {e}");
            Err(anyhow!(e))
        }
        (Err(e), SandboxMode::None) => {
            tracing::warn!("dev toolchain differs from the pin (allowed with sandbox none): {e}");
            Ok(())
        }
    }
}

/// Build and run a trivial program through the exact production path.
pub fn build_self_test(settings: &BuildSettings) -> Result<()> {
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

/// Server-owned manifest. Frozen, no dependencies, edition 2024. Player
/// input does NOT flow into this string. `panic = "abort"` keeps panics
/// from unwinding: in wasm a panic becomes a clean trap (the player's panic
/// message is still printed to stderr first), so "your code panicked"
/// surfaces as ran_ok=false rather than a murky partial state. opt-level 1
/// with codegen-units 16 favors build latency (this is graded interactively)
/// over runtime micro-perf. `autobins = false` so a harness build, whose
/// bin root is `src/pnc_entry.rs`, does not also auto-discover
/// `src/main.rs` as a second binary. `bin_path` is one of two literals.
fn manifest(bin_path: &str) -> String {
    format!(
        r#"[package]
name = "player_submission"
version = "0.0.0"
edition = "2024"
publish = false
autobins = false

[[bin]]
name = "player_submission"
path = "{bin_path}"

[profile.release]
opt-level = 1
codegen-units = 16
lto = false
panic = "abort"
"#
    )
}

fn write_skeleton(job: &Path, source: &str, layout: &Layout<'_>) -> Result<()> {
    let src_dir = job.join("src");
    // The player's file: preamble + source, never otherwise modified. The
    // preamble is an inner attribute and must be first; player inner
    // attributes (if any) follow it, which is legal both as a crate root
    // and as a module file.
    let mut main_rs = String::with_capacity(source.len() + 128);
    main_rs.push_str(PLAYER_PREAMBLE);
    main_rs.push_str(source);
    let bin_path = match layout {
        Layout::Plain => "src/main.rs",
        Layout::Harness { entry, checker } => {
            main_rs.push_str(HARNESS_DECLARATION);
            fs::write(src_dir.join("pnc_entry.rs"), entry)
                .context("wasm_builder: failed to write src/pnc_entry.rs")?;
            let module_dir = src_dir.join("main");
            fs::create_dir(&module_dir).context("wasm_builder: failed to create src/main/")?;
            fs::write(module_dir.join("__pnc_harness.rs"), checker)
                .context("wasm_builder: failed to write the checker module")?;
            "src/pnc_entry.rs"
        }
    };
    fs::write(job.join("Cargo.toml"), manifest(bin_path))
        .context("wasm_builder: failed to write Cargo.toml")?;
    fs::write(src_dir.join("main.rs"), main_rs)
        .context("wasm_builder: failed to write src/main.rs")?;
    tracing::debug!(
        harness = matches!(layout, Layout::Harness { .. }),
        "wasm_builder: wrote crate skeleton"
    );
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
        if let Some(crash) = toolchain_crash(&out.stderr) {
            // Not the player's fault: never report it as a compile error.
            // The `Err` becomes the setup-failure outcome (HTTP 500, ok =
            // false, generic "please try again"), which is never a clear.
            // The tail is player-influenced (lld echoes symbol names raw):
            // `stderr_tail` escapes every control character, so it stays
            // one field on one log line.
            tracing::error!(
                kind = crash.kind(),
                exit = ?out.status.code(),
                stderr_tail = %stderr_tail(&out.stderr),
                "wasm_builder: the toolchain crashed while building player code"
            );
            return Err(crash.into());
        }
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
    const ARTIFACT_DIRS: [&str; 3] = ["target", WASIP1_TARGET, "release"];
    let bytes = match read_artifact(job, &ARTIFACT_DIRS, "player_submission.wasm")? {
        ArtifactRead::Bytes(b) => b,
        ArtifactRead::Missing(e) => {
            // Built clean but no artifact — almost always "no `main`"
            // (a lib-shaped snippet built as a bin). Report it honestly
            // rather than pretending the run can proceed.
            tracing::warn!("wasm_builder: build succeeded but artifact missing: {e}");
            return Ok(BuildVerdict {
                ok: false,
                stderr: format!("build produced no runnable binary (is there a `fn main`?): {e}"),
                wasm: None,
            });
        }
        ArtifactRead::Refused(reason) => {
            tracing::warn!("wasm_builder: refused build artifact: {reason}");
            return Ok(BuildVerdict {
                ok: false,
                stderr: format!("built module refused: {reason}"),
                wasm: None,
            });
        }
    };
    tracing::debug!("wasm_builder: read {} wasm bytes", bytes.len());
    Ok(BuildVerdict {
        ok: true,
        stderr: out.stdout,
        wasm: Some(bytes),
    })
}

/// A failed build whose stderr shows the toolchain itself died, as
/// opposed to rejecting the player's code. Returned as an `Err`, so it is
/// reported as a server-side failure, never as a compile error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ToolchainCrash {
    /// A native toolchain component (rust-lld, LLVM) aborted through the
    /// C++ runtime: a line starting `terminate called after throwing`.
    #[error("the toolchain aborted (C++ terminate) while building player code")]
    CxxTerminate,
    /// rustc reported the linker killed by a signal.
    #[error("the linker was killed by a signal while building player code")]
    LinkerSignal,
}

impl ToolchainCrash {
    /// Stable label for logs.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CxxTerminate => "cxx_terminate",
            Self::LinkerSignal => "linker_signal",
        }
    }
}

/// Classify a failed build's stderr (pure). `None` means the failure is
/// the player's (a compile error, an undefined symbol, ...).
///
/// Deliberately narrow: only inside rustc's linker-failure block (a
/// column-0 `error: linking with `...` line and the indented notes under
/// it), either the C++ runtime's abort line or rustc saying the linker
/// died by a signal. Every rustc diagnostic a player can provoke starts
/// with its level at column 0 and indents continuation lines, so a
/// multi-line `compile_error!` cannot open that block. It is NOT
/// unforgeable: lld quotes symbol names raw, so an undefined
/// `#[link_name = "x\n  terminate called after throwing an instance of
/// y"]` lands inside the block (checked on 1.99.0). That code fails to
/// link anyway; forging only turns the player's own failure into the
/// server-error outcome (HTTP 500, `ok: false`), which is never a clear
/// (the client treats it as offline and does not save). So this must
/// never be, and is not, what decides a pass.
pub fn toolchain_crash(stderr: &str) -> Option<ToolchainCrash> {
    const LINK_FAILED: &str = "error: linking with `";
    let mut in_link_block = false;
    for line in stderr.lines() {
        if line.starts_with(LINK_FAILED) {
            if line.contains("` failed: signal: ") {
                return Some(ToolchainCrash::LinkerSignal);
            }
            in_link_block = true;
            continue;
        }
        if line.starts_with(|c: char| !c.is_whitespace()) {
            // Any other column-0 line ends the block (`error:`, `warning:`).
            in_link_block = false;
            continue;
        }
        let note = line.trim_start();
        let note = note.strip_prefix("= note:").unwrap_or(note).trim_start();
        if in_link_block && note.starts_with("terminate called after throwing an instance of ") {
            return Some(ToolchainCrash::CxxTerminate);
        }
    }
    None
}

/// The last ~2 KiB of `stderr` for an error log line: ANSI-stripped, then
/// every control character (`\n`, `\r`, NUL, ESC, ...) and the Unicode
/// line/paragraph separators replaced by their `escape_default` text, so
/// the result is a single line with no control characters at all. The
/// text is player-influenced (lld echoes undefined symbol names, newlines
/// included); without this a player could write whole forged lines into
/// the job log, the builder's relog and the journal, or plant the
/// `pnc-job-phase` marker `jobs.rs` looks for. The bound applies to the
/// escaped result.
fn stderr_tail(stderr: &str) -> String {
    const TAIL_BYTES: usize = 2048;
    let clean = proc_tree::strip_ansi(stderr);
    let mut escaped = String::with_capacity(clean.len());
    for c in clean.chars() {
        if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') {
            escaped.extend(c.escape_default());
        } else {
            escaped.push(c);
        }
    }
    let mut start = escaped.len().saturating_sub(TAIL_BYTES);
    while !escaped.is_char_boundary(start) {
        start += 1;
    }
    escaped.split_off(start)
}

/// Outcome of [`read_artifact`].
#[derive(Debug)]
pub enum ArtifactRead {
    Bytes(Vec<u8>),
    /// Not there (the build produced no binary).
    Missing(String),
    /// There, but not something we will read (symlink, not a regular file,
    /// wrong owner/device, over [`MAX_WASM_BYTES`]).
    Refused(String),
}

/// Read `<job>/<dirs...>/<file>` without following symlinks and with a size
/// cap applied before and while reading.
///
/// The build ran with write access to the job dir, so every component
/// below `job` is untrusted: each directory must be a real directory (not
/// a symlink) on the job dir's device, and the file itself is opened with
/// `O_NOFOLLOW` (Unix), must be a regular file owned by us on the same
/// device, and is read through a `take(MAX + 1)` so a file that grows
/// after the check is still capped. No build process is alive at this
/// point (the tree is confirmed dead), so the checks cannot race the
/// build. Windows (dev only) gets the symlink and size checks.
#[doc(hidden)]
pub fn read_artifact(job: &Path, dirs: &[&str], file: &str) -> Result<ArtifactRead> {
    use std::io::Read;

    let job_meta = fs::symlink_metadata(job)
        .with_context(|| format!("wasm_builder: stat job dir {}", job.display()))?;
    let mut dir = job.to_path_buf();
    for component in dirs {
        dir.push(component);
        let meta = match fs::symlink_metadata(&dir) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ArtifactRead::Missing(format!("{}: {e}", dir.display())));
            }
            Err(e) => return Ok(ArtifactRead::Refused(format!("{}: {e}", dir.display()))),
        };
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Ok(ArtifactRead::Refused(format!(
                "{} is not a plain directory",
                dir.display()
            )));
        }
        if !same_device(&job_meta, &meta) {
            return Ok(ArtifactRead::Refused(format!(
                "{} is on another device",
                dir.display()
            )));
        }
    }
    let path = dir.join(file);
    let opened = open_no_follow(&path);
    let f = match opened {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ArtifactRead::Missing(format!("{}: {e}", path.display())));
        }
        Err(e) => {
            // ELOOP from O_NOFOLLOW on a symlink lands here.
            return Ok(ArtifactRead::Refused(format!("{}: {e}", path.display())));
        }
    };
    let meta = f
        .metadata()
        .with_context(|| format!("wasm_builder: fstat {}", path.display()))?;
    if !meta.is_file() {
        return Ok(ArtifactRead::Refused(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    if !same_device(&job_meta, &meta) {
        return Ok(ArtifactRead::Refused(format!(
            "{} is on another device",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let euid = rustix::process::geteuid().as_raw();
        if meta.uid() != euid {
            return Ok(ArtifactRead::Refused(format!(
                "{} is owned by uid {}, not {euid}",
                path.display(),
                meta.uid()
            )));
        }
    }
    let len = meta.len();
    if len > MAX_WASM_BYTES as u64 {
        return Ok(ArtifactRead::Refused(format!(
            "built module is {len} bytes, over the {MAX_WASM_BYTES}-byte limit"
        )));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(len).unwrap_or(0));
    f.take(MAX_WASM_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("wasm_builder: failed to read {}", path.display()))?;
    if bytes.len() > MAX_WASM_BYTES {
        return Ok(ArtifactRead::Refused(format!(
            "built module grew past the {MAX_WASM_BYTES}-byte limit while being read"
        )));
    }
    Ok(ArtifactRead::Bytes(bytes))
}

#[cfg(unix)]
fn open_no_follow(path: &Path) -> std::io::Result<fs::File> {
    use rustix::fs::{Mode, OFlags};
    let fd = rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NOCTTY,
        Mode::empty(),
    )?;
    Ok(fs::File::from(fd))
}

#[cfg(not(unix))]
fn open_no_follow(path: &Path) -> std::io::Result<fs::File> {
    // No O_NOFOLLOW: refuse a symlink by lstat first (dev only; nothing
    // runs between this check and the open).
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(std::io::Error::other("refusing to follow a symlink"));
    }
    fs::File::open(path)
}

#[cfg(unix)]
fn same_device(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    a.dev() == b.dev()
}

#[cfg(not(unix))]
fn same_device(_a: &fs::Metadata, _b: &fs::Metadata) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The failure seen on the 56-core Linux host (2026-11-16), as cargo
    /// printed it.
    const LLD_EAGAIN: &str = "\
error: linking with `rust-lld` failed: exit status: 1
  |
  = note: \"rust-lld\" \"-flavor\" \"wasm\" \"--threads=4\"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: terminate called after throwing an instance of 'std::system_error'
            what():  Resource temporarily unavailable
          rust-lld: error: failed to write output '/build/target/wasm32-wasip1/release/deps/player_submission-0123.wasm': No such file or directory


error: could not compile `player_submission` (bin \"player_submission\") due to 1 previous error
";

    #[test]
    fn linker_abort_is_a_toolchain_crash() {
        assert_eq!(
            toolchain_crash(LLD_EAGAIN),
            Some(ToolchainCrash::CxxTerminate)
        );
        assert_eq!(
            toolchain_crash(
                "error: linking with `rust-lld` failed: signal: 6 (SIGABRT) (core dumped)\n"
            ),
            Some(ToolchainCrash::LinkerSignal)
        );
    }

    #[test]
    fn player_errors_stay_player_errors() {
        let compile_error = "\
error[E0308]: mismatched types
 --> src/main.rs:3:18
  |
3 |     let x: u32 = \"seven\";
  |            ---   ^^^^^^^ expected `u32`, found `&str`

error: could not compile `player_submission` (bin \"player_submission\") due to 1 previous error
";
        assert_eq!(toolchain_crash(compile_error), None);
        // An ordinary link error (undefined symbol) is the player's.
        let undefined = "\
error: linking with `rust-lld` failed: exit status: 1
  |
  = note: rust-lld: error: player_submission.o: undefined symbol: missing_fn
";
        assert_eq!(toolchain_crash(undefined), None);
        // A one-line `compile_error!` cannot reach it: rustc prefixes the
        // message with `error: `.
        let forged = "error: terminate called after throwing an instance of 'std::system_error'\n";
        assert_eq!(toolchain_crash(forged), None);
        // Nor a multi-line one (rustc 1.99.0 renders the second line
        // indented under the first, as here).
        let forged_multiline = "\
error: a
       = note: terminate called after throwing an instance of x
 --> src/main.rs:1:1
";
        assert_eq!(toolchain_crash(forged_multiline), None);
        // Nor a multi-line warning followed by a real (player-caused) link
        // error: the warning block ends at the column-0 `error: linking`.
        let warning_then_link = format!(
            "warning: a\n       terminate called after throwing an instance of x\n\n{undefined}"
        );
        assert_eq!(toolchain_crash(&warning_then_link), None);
        assert_eq!(toolchain_crash(""), None);
    }

    #[test]
    fn stderr_tail_keeps_the_end_on_a_char_boundary() {
        let long = format!("{}\u{e9}end", "x".repeat(4000));
        let t = stderr_tail(&long);
        assert!(t.ends_with("\u{e9}end") && t.len() <= 2048, "{}", t.len());
        assert_eq!(stderr_tail("short"), "short");
    }

    #[test]
    fn stderr_tail_has_no_control_characters() {
        let t = stderr_tail("a\nINFO forged\rb\x1b[31m\x00");
        assert!(!t.chars().any(char::is_control), "{t:?}");
        assert_eq!(t, "a\\nINFO forged\\rb\\u{0}");
        // A planted job-phase marker cannot start a line of the log.
        let planted = format!(
            "x\n{}\ny\u{2028}z",
            crate::wasm_runner::JOB_RUN_PHASE_MARKER
        );
        let t = stderr_tail(&planted);
        assert_eq!(t.lines().count(), 1, "{t:?}");
        assert!(
            !t.lines()
                .any(|l| l.trim() == crate::wasm_runner::JOB_RUN_PHASE_MARKER),
            "{t:?}"
        );
        assert!(!t.chars().any(|c| c.is_control() || c == '\u{2028}'));
        // The bound holds for the escaped text: 4000 newlines escape to
        // 8000 bytes, cut to the last 2048.
        let t = stderr_tail(&"\n".repeat(4000));
        assert!(t.len() <= 2048 && !t.contains('\n'), "{}", t.len());
    }
}
