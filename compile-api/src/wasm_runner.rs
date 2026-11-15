// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sandboxed wasm execution.
//!
//! Companion to [`crate::cargo_grader`]: where `cargo_grader` takes
//! player Rust source and runs `cargo check` to a verdict, this module
//! takes the *output* of a successful build (raw `.wasm` bytes) and
//! actually runs them. The two halves are kept separate so the
//! security review of "we run player-controlled wasm" can be scoped to
//! this file alone.
//!
//! # Security contract (per `design/05-tech-architecture.md` §2)
//!
//! - `bytes` is treated as fully adversarial. The whole point of this
//!   module is sandboxing.
//! - WASI capabilities are restricted to stdio: stdin closed (reads see
//!   end-of-file at once), stdout/stderr captured. No filesystem (no
//!   `preopened_dir`), no sockets (TCP, UDP and name lookup explicitly
//!   denied, and WASIp1 has no call that creates a socket), no env, no
//!   args. Clocks and random come from the host defaults.
//! - The wasm proposals the engine accepts are an explicit allow-list
//!   ([`allowed_wasm_features`]): the WebAssembly 2.0 baseline minus SIMD.
//!   Everything else (SIMD, threads, GC, exceptions, tail calls, memory64,
//!   multi-memory, components, ...) is refused at validation. GC, threads,
//!   the pooling allocator, parallel compilation and the module cache are
//!   also compiled out of wasmtime (`default-features = false` in
//!   `Cargo.toml`, pinned by `tests/deploy_files.rs`).
//! - Fuel exhaustion via [`wasmtime::Config::consume_fuel`] +
//!   [`wasmtime::Store::set_fuel`].
//! - Linear-memory cap, instance/table/memory counts and table growth via
//!   the store's resource limiter ([`StoreLimitsBuilder`]).
//! - stdout/stderr are captured to bounded in-memory pipes
//!   ([`wasmtime_wasi::p2::pipe::MemoryOutputPipe`], capacity
//!   [`PIPE_CAP_BYTES`]).
//! - Fuel/timeout traps are NOT propagated as `Err` to the caller —
//!   they are normal grading outcomes and surface as
//!   `Ok(RunVerdict { ok: false, .. })` with a recognisable stderr
//!   message ("fuel exhausted" or [`TIMEOUT_PREFIX`]).
//! - Module size is capped at [`MAX_WASM_BYTES`] before anything is
//!   parsed, and Cranelift compilation is single-threaded (wasmtime's
//!   `parallel-compilation` feature is not compiled in) so one module
//!   cannot occupy every core.
//! - Copy-on-write memory images are off (`memory_init_cow(false)`), so
//!   wasmtime does not need `memfd_create` (smaller syscall surface under
//!   the builder unit's seccomp filter).
//!
//! # Run timeout (Linux verification 2026-11-14, open item 1)
//!
//! The first version ran `_start` on a synchronous store and relied on
//! epoch interruption alone. Epoch checks are compiled into *guest* code
//! (function entries and loop back-edges), so they stop a busy loop, but
//! a guest blocked inside a host call never reaches one: wasmtime-wasi's
//! synchronous `poll_oneoff` (what `std::thread::sleep` lowers to) slept
//! in the host for as long as the guest asked, and fuel is not consumed
//! while the host runs. A `sleep(3600 s)` held its build slot until the
//! 50 s job deadline killed the job process, which then reported it as a
//! compile error.
//!
//! The run now has three independent bounds, all keyed to the run's
//! timeout:
//!
//! 1. **Guest code**: epoch interruption. A watchdog thread bumps the
//!    engine epoch at `timeout`; the store's 1-tick deadline traps the
//!    guest at its next epoch check.
//! 2. **Host calls**: the module runs on an *async* store
//!    ([`wasmtime_wasi::p1::add_to_linker_async`], `call_async`) inside a
//!    current-thread tokio runtime, wrapped in [`tokio::time::timeout`].
//!    Every WASIp1 host call the guest can reach is a future awaited by
//!    that runtime, so when the timer fires the call is dropped (cancelled)
//!    wherever the guest is blocked. `allow_blocking_current_thread` stays
//!    `false`: with it on, wasmtime-wasi would serve a relative sleep with
//!    a real `std::thread::sleep` on the runtime thread and the timer could
//!    not fire.
//! 3. **Backstop**: the run itself executes on a helper thread and the
//!    caller waits at most `timeout + RUN_TIMEOUT_MARGIN`. If (1) and (2)
//!    both failed (a future wasmtime-wasi that blocks a thread inside a
//!    host call), the one-shot job process abandons the run thread and
//!    answers; it exits right after, which ends the thread. A long-lived
//!    process (dev executor, tests) logs and waits instead, so no work
//!    outlives the caller's build slot. The builder's job deadline
//!    ([`crate::jobs::JOB_DEADLINE`]) remains the last line.
//!
//! WASIp1 host functions and whether they can block (wasmtime-wasi 50,
//! with the context built here):
//!
//! | Function(s) | Can block? |
//! |---|---|
//! | `poll_oneoff` with a clock subscription (relative or absolute) | **yes**: sleeps until the deadline. Async future, cancelled by (2) |
//! | `poll_oneoff` on fd 0/1/2 | no: closed stdin and the memory pipes are always ready |
//! | `fd_read`/`fd_pread` on stdin | no: stdin is closed, reads return 0 bytes (EOF) at once |
//! | `fd_write`/`fd_pwrite` on stdout/stderr | no: in-memory pipe, refuses writes past [`PIPE_CAP_BYTES`] |
//! | `sched_yield`, `random_get`, `clock_time_get`, `clock_res_get` | no |
//! | `args_*`, `environ_*` | no (both empty) |
//! | `proc_exit` | no (unwinds the guest) |
//! | `fd_*` on any other fd, `path_*` | no: no preopens, so `EBADF` before any I/O |
//! | `sock_accept/recv/send/shutdown` | no: there is no socket fd to name |
//!
//! Only the clock wait can block, and every function is behind (2) and
//! (3) anyway. `tests/wasm_runner.rs` pins sleep, absolute-time poll,
//! multi-subscription poll, busy loop and stdin reads.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Result, bail};

use crate::helper_threads;

/// Set once by the one-shot job process; see the module docs.
static EXITS_AFTER_JOB: AtomicBool = AtomicBool::new(false);

/// Declare that this process exits as soon as its single job is answered,
/// so a timed-out module compile or a stuck run may be abandoned instead
/// of awaited. Only `pledge_compile_api job` calls this.
pub fn set_process_exits_after_job() {
    EXITS_AFTER_JOB.store(true, Ordering::SeqCst);
}

/// True in the one-shot job process (see [`set_process_exits_after_job`]).
pub fn process_exits_after_job() -> bool {
    EXITS_AFTER_JOB.load(Ordering::SeqCst)
}

/// Line a job process writes to its stderr once the build is over and the
/// run is about to start. If the builder later has to kill the job at its
/// deadline, this tells it the program compiled (so the verdict is a
/// runtime error, not a compile error). Never seen by the player.
pub const JOB_RUN_PHASE_MARKER: &str = "pnc-job-phase: run";

/// Called right before a built module is run. In the job process this
/// writes [`JOB_RUN_PHASE_MARKER`] to stderr; elsewhere it does nothing.
pub fn announce_run_phase() {
    if process_exits_after_job() {
        eprintln!("{JOB_RUN_PHASE_MARKER}");
    }
}

use wasmtime::{
    Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, WasmFeatures,
};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;

/// Cap on captured stdout / stderr per stream. 64 KiB.
pub const PIPE_CAP_BYTES: usize = 64 * 1024;

/// Largest module accepted for compilation. A curriculum program built
/// for `wasm32-wasip1` at opt-level 1 is on the order of 100 KiB; 4 MiB
/// leaves wide headroom while bounding compile work.
pub const MAX_WASM_BYTES: usize = 4 * 1024 * 1024;

/// Wall-clock budget for `Module::new` (Cranelift compile).
pub const MODULE_COMPILE_TIMEOUT: Duration = Duration::from_secs(10);

/// How far past its timeout a run may go before the backstop answers for
/// it. The epoch trap and the async timeout both fire AT the timeout;
/// this only absorbs scheduling delay on a loaded host.
pub const RUN_TIMEOUT_MARGIN: Duration = Duration::from_secs(2);

/// First word of the stderr of a run stopped at its timeout.
pub const TIMEOUT_PREFIX: &str = "timeout:";

/// One instance per run (the player module; WASI p1 host functions are
/// not instances).
pub const MAX_INSTANCES: usize = 1;
/// Rust emits one table and one memory.
pub const MAX_TABLES: usize = 2;
pub const MAX_MEMORIES: usize = 1;
/// Elements per table. Rust's indirect-call table for a small program
/// holds a few hundred entries.
pub const MAX_TABLE_ELEMENTS: usize = 64 * 1024;

/// Floor on fuel; anything below this is rejected up front to catch
/// caller mistakes (zero / one fuel will trap before the module body
/// even starts running).
const MIN_FUEL: u64 = 1_000;

/// Floor on memory cap. WASI's `_start` plus some startup allocations
/// will not fit in less than ~64 KiB; reject anything smaller as a
/// caller bug.
const MIN_MEMORY_BYTES: u64 = 64 * 1024;

/// Floor on timeout. Watchdog needs at least a few ms of slack to be
/// useful; very small values are caller bugs.
const MIN_TIMEOUT_MS: u64 = 10;

/// Verdict from a single wasm run.
#[derive(Debug, Clone)]
pub struct RunVerdict {
    /// True iff `_start` returned successfully (no trap, no exit code
    /// other than 0). For WASI modules a successful run usually means
    /// the program exited via `wasi_proc_exit(0)` which we map to ok.
    pub ok: bool,
    /// Captured stdout, lossy UTF-8, capped at [`PIPE_CAP_BYTES`].
    pub stdout: String,
    /// Captured stderr or trap message, lossy UTF-8, capped at
    /// [`PIPE_CAP_BYTES`] for the captured portion. Trap text is
    /// prepended verbatim and is not subject to that cap.
    pub stderr: String,
}

/// Per-store data. Holds the WASI preview1 context plus the store
/// limits applied to memory/instance growth.
struct StoreState {
    wasi: WasiP1Ctx,
    limits: StoreLimits,
}

/// The only wasm proposals the engine accepts: the WebAssembly 2.0
/// baseline that rustc's `wasm32-wasip1` output uses, without SIMD.
///
/// `bulk_memory` and `reference_types` MUST stay (hard-won, 2026-06-18):
/// rustc lowers `memcpy`/`memset` in std to `memory.copy`/`memory.fill`
/// and emits the reference-types table encoding; without them wasmtime
/// rejects EVERY real Rust module with a "zero byte expected" error. The
/// sub-flags wasmparser folds into them (`bulk_memory_opt`,
/// `call_indirect_overlong`) come along. `floats` is a wasmparser
/// pseudo-feature (float types/instructions at all).
///
/// Everything else is refused: SIMD, relaxed SIMD, threads/shared memory,
/// GC and GC types, function references, exceptions (both encodings),
/// tail calls, multi-memory, memory64, extended-const, wide arithmetic,
/// stack switching, custom page sizes, the component model.
pub fn allowed_wasm_features() -> WasmFeatures {
    WasmFeatures::MUTABLE_GLOBAL
        | WasmFeatures::SATURATING_FLOAT_TO_INT
        | WasmFeatures::SIGN_EXTENSION
        | WasmFeatures::MULTI_VALUE
        | WasmFeatures::BULK_MEMORY
        | WasmFeatures::REFERENCE_TYPES
        | WasmFeatures::FLOATS
}

/// The engine configuration every run uses. Public so tests can build an
/// engine from exactly this and probe it.
pub fn hardened_config() -> Config {
    let mut config = Config::new();
    config.consume_fuel(true);
    config.epoch_interruption(true);
    // Allow-list, not deny-list: start from nothing, add the baseline.
    // A wasmtime upgrade that turns a new proposal on by default cannot
    // reach player code this way (wasmtime 50 already defaults to the
    // whole WebAssembly 3.0 set plus wide-arithmetic).
    config.wasm_features(WasmFeatures::all(), false);
    config.wasm_features(allowed_wasm_features(), true);
    // No CoW memory images: avoids memfd_create (see module docs).
    config.memory_init_cow(false);
    config
}

/// Run `bytes` as a WASI preview1 module under a sandbox configured
/// from the caller's resource budget.
///
/// See module docs for the security contract. Returns `Err` only for
/// setup-time failures (empty bytes, invalid budget, engine init
/// failure, module compile failure, linker setup failure).
pub fn run_wasm(
    bytes: &[u8],
    timeout_ms: u64,
    fuel: u64,
    max_memory_bytes: u64,
) -> Result<RunVerdict> {
    run_wasm_with_compile_timeout(
        bytes,
        timeout_ms,
        fuel,
        max_memory_bytes,
        MODULE_COMPILE_TIMEOUT,
    )
}

/// [`run_wasm`] with an explicit module-compile budget (tests use a zero
/// budget to exercise the timeout path deterministically).
#[doc(hidden)]
pub fn run_wasm_with_compile_timeout(
    bytes: &[u8],
    timeout_ms: u64,
    fuel: u64,
    max_memory_bytes: u64,
    compile_timeout: Duration,
) -> Result<RunVerdict> {
    if bytes.is_empty() {
        tracing::warn!("wasm_runner: rejected empty module");
        bail!("wasm_runner: bytes is empty");
    }
    if bytes.len() > MAX_WASM_BYTES {
        tracing::warn!(
            bytes = bytes.len(),
            limit = MAX_WASM_BYTES,
            "wasm_runner: rejected oversized module"
        );
        bail!(
            "wasm_runner: module is {} bytes, over the {MAX_WASM_BYTES}-byte limit",
            bytes.len()
        );
    }
    if fuel < MIN_FUEL {
        tracing::warn!(fuel, "wasm_runner: rejected fuel budget below floor");
        bail!("wasm_runner: fuel={fuel} below floor {MIN_FUEL}; pick a saner budget");
    }
    if max_memory_bytes < MIN_MEMORY_BYTES {
        tracing::warn!(
            max_memory_bytes,
            "wasm_runner: rejected memory cap below floor"
        );
        bail!(
            "wasm_runner: max_memory_bytes={max_memory_bytes} below floor \
             {MIN_MEMORY_BYTES}"
        );
    }
    if timeout_ms < MIN_TIMEOUT_MS {
        tracing::warn!(timeout_ms, "wasm_runner: rejected timeout below floor");
        bail!("wasm_runner: timeout_ms={timeout_ms} below floor {MIN_TIMEOUT_MS}");
    }

    tracing::debug!(
        "wasm_runner: starting run bytes={} timeout_ms={} fuel={} max_memory_bytes={}",
        bytes.len(),
        timeout_ms,
        fuel,
        max_memory_bytes
    );

    // ---- Engine ---------------------------------------------------------
    let engine = Engine::new(&hardened_config())?;
    let module = match compile_bounded(&engine, bytes, compile_timeout)? {
        Some(m) => m,
        None => {
            let budget = if compile_timeout.subsec_millis() == 0 {
                format!("{}s", compile_timeout.as_secs())
            } else {
                format!("{}ms", compile_timeout.as_millis())
            };
            tracing::warn!(
                ?compile_timeout,
                "wasm_runner: module compile budget exceeded"
            );
            return Ok(RunVerdict {
                ok: false,
                stdout: String::new(),
                stderr: format!("module compile exceeded {budget}; program too large to run"),
            });
        }
    };

    let timeout = Duration::from_millis(timeout_ms);
    let stdout_pipe = MemoryOutputPipe::new(PIPE_CAP_BYTES);
    let stderr_pipe = MemoryOutputPipe::new(PIPE_CAP_BYTES);

    // ---- Watchdog (bound 1: guest code) ---------------------------------
    // A thread blocks on `recv_timeout(timeout)`. If the run finishes
    // first we drop `cancel_tx`, the receive returns `Disconnected`
    // immediately and the thread exits without ticking — so a 2 ms run
    // joins in ~2 ms instead of parking a thread for the full timeout. On
    // `Timeout` it bumps the engine epoch once, which trips the store's
    // 1-epoch deadline. The engine is per-run, so the tick can never
    // interrupt another submission.
    let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
    let watchdog_engine = engine.clone();
    let watchdog = helper_threads::spawn("wasm-watchdog", move || {
        if let Err(mpsc::RecvTimeoutError::Timeout) = cancel_rx.recv_timeout(timeout) {
            tracing::debug!("wasm_runner: watchdog tick (run timeout reached)");
            watchdog_engine.increment_epoch();
        }
    })?;

    // ---- Run thread (bound 2: host calls) + backstop (bound 3) ----------
    let (done_tx, done_rx) = mpsc::channel::<Result<RunEnd>>();
    let run_engine = engine.clone();
    let out_pipe = stdout_pipe.clone();
    let err_pipe = stderr_pipe.clone();
    let runner = helper_threads::spawn("wasm-run", move || {
        let end = run_on_current_thread(
            &run_engine,
            &module,
            out_pipe,
            err_pipe,
            fuel,
            max_memory_bytes,
            timeout,
        );
        let _ = done_tx.send(end);
    });
    let runner = match runner {
        Ok(h) => h,
        Err(e) => {
            drop(cancel_tx);
            let _ = watchdog.join();
            return Err(e.into());
        }
    };

    let backstop = timeout + RUN_TIMEOUT_MARGIN;
    let end = match done_rx.recv_timeout(backstop) {
        Ok(end) => {
            if runner.join().is_err() {
                tracing::warn!("wasm_runner: run thread panicked after sending");
            }
            end
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            if process_exits_after_job() {
                tracing::error!(
                    ?backstop,
                    "wasm_runner: run did not stop at its timeout; abandoning the run thread \
                     (job process exits next)"
                );
            } else {
                tracing::error!(
                    ?backstop,
                    "wasm_runner: run did not stop at its timeout; waiting for it so no work \
                     outlives the caller's build slot"
                );
                if runner.join().is_err() {
                    tracing::warn!("wasm_runner: run thread panicked");
                }
            }
            Ok(RunEnd::Backstop)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            let _ = runner.join();
            drop(cancel_tx);
            let _ = watchdog.join();
            bail!("wasm_runner: run thread exited without a result")
        }
    };

    // Cancel the watchdog (disconnects its receiver) and join.
    drop(cancel_tx);
    if watchdog.join().is_err() {
        tracing::warn!("wasm_runner: watchdog thread panicked");
    }

    // ---- Drain pipes ----------------------------------------------------
    // contents() returns the bytes accumulated so far; the pipe refuses
    // writes past its PIPE_CAP_BYTES capacity.
    let captured_stdout = String::from_utf8_lossy(&stdout_pipe.contents()).into_owned();
    let captured_stderr = String::from_utf8_lossy(&stderr_pipe.contents()).into_owned();

    match end? {
        RunEnd::Returned => {
            tracing::debug!("wasm_runner: _start returned Ok");
            Ok(RunVerdict {
                ok: true,
                stdout: captured_stdout,
                stderr: captured_stderr,
            })
        }
        RunEnd::Trapped(err) => Ok(classify_trap(
            &err,
            timeout_ms,
            captured_stdout,
            captured_stderr,
        )),
        RunEnd::HostTimeout => {
            tracing::info!(
                timeout_ms,
                "wasm_runner: run timeout while the guest was blocked in a host call"
            );
            Ok(timeout_verdict(
                timeout_ms,
                captured_stdout,
                captured_stderr,
            ))
        }
        RunEnd::Backstop => {
            tracing::warn!(timeout_ms, "wasm_runner: run stopped by the backstop");
            Ok(timeout_verdict(
                timeout_ms,
                captured_stdout,
                captured_stderr,
            ))
        }
    }
}

/// How a run ended (before classification).
enum RunEnd {
    /// `_start` returned.
    Returned,
    /// Instantiation or `_start` failed: trap, exit, link error.
    Trapped(wasmtime::Error),
    /// The async timeout fired (guest blocked in a host call).
    HostTimeout,
    /// The backstop answered; the run thread may still be running.
    Backstop,
}

/// Instantiate and run `_start` on an async store inside a fresh
/// current-thread runtime on THIS thread (a fresh helper thread, so it is
/// never nested in the caller's runtime).
fn run_on_current_thread(
    engine: &Engine,
    module: &Module,
    stdout: MemoryOutputPipe,
    stderr: MemoryOutputPipe,
    fuel: u64,
    max_memory_bytes: u64,
    timeout: Duration,
) -> Result<RunEnd> {
    // ---- WASI ctx -------------------------------------------------------
    // No env, no args, no preopens, no network. Stdio only: stdin is the
    // default closed stream, stdout/stderr go to bounded memory pipes.
    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder
        .stdout(stdout)
        .stderr(stderr)
        .allow_tcp(false)
        .allow_udp(false)
        .allow_ip_name_lookup(false)
        // Keep blocking host work off this thread: with this on, a
        // relative sleep becomes `std::thread::sleep` on the runtime
        // thread and the run timeout could not fire (see module docs).
        .allow_blocking_current_thread(false);
    let wasi = wasi_builder.build_p1();

    let limits = StoreLimitsBuilder::new()
        .memory_size(usize::try_from(max_memory_bytes).unwrap_or(usize::MAX))
        .table_elements(MAX_TABLE_ELEMENTS)
        .instances(MAX_INSTANCES)
        .tables(MAX_TABLES)
        .memories(MAX_MEMORIES)
        .build();

    let mut store: Store<StoreState> = Store::new(engine, StoreState { wasi, limits });
    store.limiter(|s| &mut s.limits);
    store.set_fuel(fuel)?;
    // One epoch tick is enough to interrupt: the watchdog increments the
    // engine epoch once after the timeout and this deadline (1 tick past
    // the current epoch) is then exceeded. Trap, never yield-and-continue.
    store.set_epoch_deadline(1);
    store.epoch_deadline_trap();

    let mut linker: Linker<StoreState> = Linker::new(engine);
    wasmtime_wasi::p1::add_to_linker_async(&mut linker, |s: &mut StoreState| &mut s.wasi)?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    let end = runtime.block_on(async {
        let call = async {
            let instance = linker.instantiate_async(&mut store, module).await?;
            let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
            start.call_async(&mut store, ()).await
        };
        match tokio::time::timeout(timeout, call).await {
            Ok(Ok(())) => RunEnd::Returned,
            Ok(Err(e)) => RunEnd::Trapped(e),
            Err(_elapsed) => RunEnd::HostTimeout,
        }
    });
    Ok(end)
}

/// Compile `bytes` on a helper thread, waiting at most `timeout` for the
/// verdict. `Ok(None)` on timeout, `Err` if compilation itself failed
/// (invalid module, disabled feature). On timeout the helper is awaited
/// (long-lived process) or abandoned (one-shot job process); see module
/// docs.
fn compile_bounded(engine: &Engine, bytes: &[u8], timeout: Duration) -> Result<Option<Module>> {
    let (tx, rx) = mpsc::channel();
    let engine = engine.clone();
    let owned = bytes.to_vec();
    let helper = helper_threads::spawn("wasm-compile", move || {
        let _ = tx.send(Module::new(&engine, &owned));
    })?;
    match rx.recv_timeout(timeout) {
        Ok(result) => {
            if helper.join().is_err() {
                tracing::warn!("wasm_runner: compile helper panicked after sending");
            }
            match result {
                Ok(m) => Ok(Some(m)),
                Err(e) => {
                    tracing::warn!("wasm_runner: module refused at compile/validation: {e:#}");
                    Err(e.into())
                }
            }
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            if process_exits_after_job() {
                tracing::warn!(
                    ?timeout,
                    bytes = bytes.len(),
                    "wasm_runner: module compile timed out; abandoning it (job process exits next)"
                );
            } else {
                tracing::warn!(
                    ?timeout,
                    bytes = bytes.len(),
                    "wasm_runner: module compile timed out; waiting for it to finish so no work \
                     outlives the caller's build slot"
                );
                if helper.join().is_err() {
                    tracing::warn!("wasm_runner: compile helper panicked");
                }
            }
            Ok(None)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            bail!("wasm_runner: compile thread exited without a result")
        }
    }
}

/// Player-facing timeout text: starts with [`TIMEOUT_PREFIX`].
pub fn timeout_message(timeout_ms: u64) -> String {
    let limit = if timeout_ms % 1000 == 0 {
        format!("{} s", timeout_ms / 1000)
    } else {
        format!("{timeout_ms} ms")
    };
    format!(
        "{TIMEOUT_PREFIX} your program was still running after the {limit} time limit and was \
         stopped. Look for a loop that never ends, or a sleep or wait that is too long."
    )
}

fn timeout_verdict(timeout_ms: u64, stdout: String, stderr: String) -> RunVerdict {
    RunVerdict {
        ok: false,
        stdout,
        stderr: combine_trap_msg(&timeout_message(timeout_ms), stderr),
    }
}

/// Map a wasmtime invocation error onto a `RunVerdict`. Distinguishes
/// fuel exhaustion, epoch / timeout interruption, WASI proc_exit(0),
/// and generic traps so the calling grader can format useful feedback
/// to the player.
fn classify_trap(
    err: &wasmtime::Error,
    timeout_ms: u64,
    stdout: String,
    stderr: String,
) -> RunVerdict {
    // WASI proc_exit(0) is the canonical "successful" exit for a real
    // WASI program. Treat any non-zero exit code as failure.
    if let Some(exit) = err.downcast_ref::<wasmtime_wasi::I32Exit>() {
        let ok = exit.0 == 0;
        let msg = if ok {
            String::new()
        } else {
            format!("program exited with code {}", exit.0)
        };
        let combined = if msg.is_empty() {
            stderr
        } else if stderr.is_empty() {
            msg
        } else {
            format!("{msg}\n{stderr}")
        };
        return RunVerdict {
            ok,
            stdout,
            stderr: combined,
        };
    }

    // Trap classification.
    if let Some(trap) = err.downcast_ref::<wasmtime::Trap>() {
        match trap {
            wasmtime::Trap::OutOfFuel => {
                tracing::debug!("wasm_runner: fuel exhausted");
                return RunVerdict {
                    ok: false,
                    stdout,
                    stderr: combine_trap_msg("fuel exhausted (likely infinite loop)", stderr),
                };
            }
            wasmtime::Trap::Interrupt => {
                tracing::debug!("wasm_runner: epoch interrupt -> timeout");
                return timeout_verdict(timeout_ms, stdout, stderr);
            }
            other => {
                tracing::debug!("wasm_runner: trap {:?}", other);
                let msg = format!("trap: {other}");
                return RunVerdict {
                    ok: false,
                    stdout,
                    stderr: combine_trap_msg(&msg, stderr),
                };
            }
        }
    }

    // Fall-through: a non-trap error, e.g. "_start not found".
    let msg = format!("execution error: {err:#}");
    tracing::debug!("wasm_runner: non-trap error: {err}");
    RunVerdict {
        ok: false,
        stdout,
        stderr: combine_trap_msg(&msg, stderr),
    }
}

fn combine_trap_msg(prefix: &str, captured: String) -> String {
    if captured.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}\n--- captured stderr ---\n{captured}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_message_is_plain_and_prefixed() {
        let m = timeout_message(5_000);
        assert!(m.starts_with(TIMEOUT_PREFIX), "{m}");
        assert!(m.contains("5 s time limit"), "{m}");
        assert!(timeout_message(250).contains("250 ms"));
    }

    #[test]
    fn allow_list_is_exactly_the_baseline() {
        // Pinned on purpose: widening this set is a security decision,
        // not a refactor. Update the module docs and the behavioural
        // tests in tests/wasm_runner.rs together with it.
        let f = allowed_wasm_features();
        for denied in [
            WasmFeatures::SIMD,
            WasmFeatures::RELAXED_SIMD,
            WasmFeatures::THREADS,
            WasmFeatures::SHARED_EVERYTHING_THREADS,
            WasmFeatures::GC,
            WasmFeatures::GC_TYPES,
            WasmFeatures::FUNCTION_REFERENCES,
            WasmFeatures::EXCEPTIONS,
            WasmFeatures::LEGACY_EXCEPTIONS,
            WasmFeatures::TAIL_CALL,
            WasmFeatures::MULTI_MEMORY,
            WasmFeatures::MEMORY64,
            WasmFeatures::EXTENDED_CONST,
            WasmFeatures::WIDE_ARITHMETIC,
            WasmFeatures::STACK_SWITCHING,
            WasmFeatures::CUSTOM_PAGE_SIZES,
            WasmFeatures::COMPONENT_MODEL,
            WasmFeatures::MEMORY_CONTROL,
        ] {
            assert!(!f.intersects(denied), "{denied:?} must not be allowed");
        }
        assert_eq!(
            f,
            WasmFeatures::MUTABLE_GLOBAL
                | WasmFeatures::SATURATING_FLOAT_TO_INT
                | WasmFeatures::SIGN_EXTENSION
                | WasmFeatures::MULTI_VALUE
                | WasmFeatures::BULK_MEMORY
                | WasmFeatures::REFERENCE_TYPES
                | WasmFeatures::FLOATS
        );
    }

    #[test]
    fn hardened_config_builds_an_engine() {
        assert!(Engine::new(&hardened_config()).is_ok());
    }
}
