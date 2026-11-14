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
//! - WASI capabilities are restricted to stdio. No filesystem (no
//!   `preopened_dir`), no network (no `inherit_network` / `allow_tcp`),
//!   no env, no host args. Random/clocks come from defaults; we do not
//!   add `inherit_env` or `inherit_args`.
//! - Wall-clock timeout via [`Engine::increment_epoch`] from a watchdog
//!   thread that is cancelled as soon as the run returns.
//! - Fuel exhaustion via [`wasmtime::Config::consume_fuel`] +
//!   [`wasmtime::Store::set_fuel`].
//! - Linear-memory cap via [`wasmtime::StoreLimitsBuilder::memory_size`].
//! - stdout/stderr are captured to bounded in-memory pipes
//!   ([`wasmtime_wasi::p2::pipe::MemoryOutputPipe`], capacity
//!   [`PIPE_CAP_BYTES`]).
//! - Fuel/timeout traps are NOT propagated as `Err` to the caller —
//!   they are normal grading outcomes and surface as
//!   `Ok(RunVerdict { ok: false, .. })` with a recognisable stderr
//!   message ("fuel exhausted" or "timeout").
//! - Module size is capped at [`MAX_WASM_BYTES`] before anything is
//!   parsed, and Cranelift compilation is single-threaded
//!   (`parallel_compilation(false)`) so one module cannot occupy every
//!   core.
//! - Module compilation is bounded in *wall time* by
//!   [`MODULE_COMPILE_TIMEOUT`]: it runs on a helper thread and the run
//!   is failed if it does not finish in time. Honest limit: wasmtime
//!   cannot cancel a compile, so on timeout the helper thread is detached
//!   and finishes in the background (its work is bounded by the size
//!   cap), outside the build semaphore. The end state that removes this
//!   gap is running the module in the sandboxed, killable build worker
//!   instead of in the server process; that is not done yet.
//! - Instance/table/memory counts and table growth are capped through the
//!   store's resource limiter, in addition to linear memory.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Result, bail};
use wasmtime::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};
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

/// One instance per run (the player module; WASI p1 host functions are
/// not instances).
const MAX_INSTANCES: usize = 1;
/// Rust emits one table and one memory.
const MAX_TABLES: usize = 2;
const MAX_MEMORIES: usize = 1;
/// Elements per table. Rust's indirect-call table for a small program
/// holds a few hundred entries.
const MAX_TABLE_ELEMENTS: usize = 64 * 1024;

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
        bail!("wasm_runner: fuel={fuel} below floor {MIN_FUEL}; pick a saner budget");
    }
    if max_memory_bytes < MIN_MEMORY_BYTES {
        bail!(
            "wasm_runner: max_memory_bytes={max_memory_bytes} below floor \
             {MIN_MEMORY_BYTES}"
        );
    }
    if timeout_ms < MIN_TIMEOUT_MS {
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
    let mut config = Config::new();
    config.consume_fuel(true);
    config.epoch_interruption(true);
    // Defence-in-depth: deny the exotic / high-surface wasm proposals
    // we don't need, while KEEPING the WebAssembly 2.0 baseline that the
    // Rust `wasm32-wasip1` std actually emits.
    //
    // Hard-won correction (2026-06-18): `bulk_memory` and
    // `reference_types` MUST stay enabled. Current rustc lowers `memcpy`/
    // `memset` in std to `memory.copy`/`memory.fill` (bulk-memory) and
    // emits the reference-types table encoding — both are baseline since
    // the WebAssembly 2.0 spec. Disabling them makes wasmtime's decoder
    // mis-parse and reject EVERY real Rust module with a "zero byte
    // expected" translation error. The original "students use none of
    // these" assumption was wrong: it's the std the linker pulls in, not
    // the student snippet, that uses them. This file's first end-to-end
    // build-and-run test (compile-api/tests/wasm_builder.rs) is what
    // surfaced it.
    //
    // Dependency chain (per wasmtime 44 Config docs):
    //   reference_types depends on bulk_memory
    //   function_references depends on reference_types
    //   gc depends on function_references
    // We enable bulk_memory + reference_types (baseline, required) and
    // stop the chain there: function_references / gc stay off. The
    // genuinely exotic or concurrency-bearing proposals — SIMD,
    // relaxed-SIMD, multi-memory, memory64, tail-call, threads (shared
    // memory + atomics) — remain disabled; the curriculum needs none of
    // them and each is real attack surface. sign-extension, multi-value
    // and non-trapping float-to-int are left at wasmtime's defaults
    // (on): they're baseline and harmless, and rustc emits them too.
    config.wasm_bulk_memory(true);
    config.wasm_reference_types(true);
    config.wasm_simd(false);
    config.wasm_relaxed_simd(false);
    config.wasm_multi_memory(false);
    config.wasm_memory64(false);
    config.wasm_function_references(false);
    config.wasm_gc(false);
    config.wasm_tail_call(false);
    config.wasm_threads(false);
    // One core per compile: a large module must not starve the server.
    config.parallel_compilation(false);

    let engine = Engine::new(&config)?;
    let module = match compile_bounded(&engine, bytes, compile_timeout)? {
        Some(m) => m,
        None => {
            return Ok(RunVerdict {
                ok: false,
                stdout: String::new(),
                stderr: format!(
                    "module compile exceeded {}s; program too large to run",
                    compile_timeout.as_secs()
                ),
            });
        }
    };

    // ---- WASI ctx -------------------------------------------------------
    // No env, no args, no preopens, no network. Stdio only, captured to
    // bounded memory pipes. stdin is left as the default (closed).
    let stdout_pipe = MemoryOutputPipe::new(PIPE_CAP_BYTES);
    let stderr_pipe = MemoryOutputPipe::new(PIPE_CAP_BYTES);

    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder.stdout(stdout_pipe.clone());
    wasi_builder.stderr(stderr_pipe.clone());
    let wasi = wasi_builder.build_p1();

    let limits = StoreLimitsBuilder::new()
        .memory_size(usize::try_from(max_memory_bytes).unwrap_or(usize::MAX))
        .table_elements(MAX_TABLE_ELEMENTS)
        .instances(MAX_INSTANCES)
        .tables(MAX_TABLES)
        .memories(MAX_MEMORIES)
        .build();

    let mut store: Store<StoreState> = Store::new(&engine, StoreState { wasi, limits });
    store.limiter(|s| &mut s.limits);
    store.set_fuel(fuel)?;
    // One epoch tick is enough to interrupt — the watchdog increments
    // the engine epoch once after `timeout_ms` and the store's deadline
    // of 1 epoch past the current value will be exceeded.
    store.set_epoch_deadline(1);

    // ---- Linker ---------------------------------------------------------
    let mut linker: Linker<StoreState> = Linker::new(&engine);
    wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |s: &mut StoreState| &mut s.wasi)?;

    // ---- Watchdog -------------------------------------------------------
    // A thread blocks on `recv_timeout(timeout_ms)`. If the run finishes
    // first we drop `cancel_tx`, the receive returns `Disconnected`
    // immediately and the thread exits without ticking — so a 2 ms run
    // joins in ~2 ms instead of parking a thread for the full timeout
    // (the old `thread::sleep` watchdog, flagged in the 2026-05-02
    // rust-lead review). On `Timeout` it bumps the engine epoch once,
    // which trips the store's 1-epoch deadline. The engine is per-run,
    // so the tick can never interrupt another submission.
    let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
    let watchdog_engine = engine.clone();
    let watchdog = thread::spawn(move || {
        if let Err(mpsc::RecvTimeoutError::Timeout) =
            cancel_rx.recv_timeout(Duration::from_millis(timeout_ms))
        {
            watchdog_engine.increment_epoch();
        }
    });

    // ---- Instantiate + invoke ------------------------------------------
    let outcome = (|| -> Result<()> {
        let instance = linker.instantiate(&mut store, &module)?;
        let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
        start.call(&mut store, ())?;
        Ok(())
    })();

    // Cancel the watchdog (disconnects its receiver) and join.
    drop(cancel_tx);
    if watchdog.join().is_err() {
        tracing::warn!("wasm_runner: watchdog thread panicked");
    }

    // ---- Drain pipes ----------------------------------------------------
    // Per docs: contents() returns the bytes accumulated so far, and
    // the pipe silently drops further writes past capacity (the pipe
    // was constructed with PIPE_CAP_BYTES capacity).
    let stdout_bytes = stdout_pipe.contents();
    let stderr_bytes = stderr_pipe.contents();
    let captured_stdout = String::from_utf8_lossy(&stdout_bytes).into_owned();
    let captured_stderr = String::from_utf8_lossy(&stderr_bytes).into_owned();

    match outcome {
        Ok(()) => {
            tracing::debug!("wasm_runner: _start returned Ok");
            Ok(RunVerdict {
                ok: true,
                stdout: captured_stdout,
                stderr: captured_stderr,
            })
        }
        Err(err) => Ok(classify_trap(err, captured_stdout, captured_stderr)),
    }
}

/// Compile `bytes` on a helper thread, waiting at most `timeout`.
/// `Ok(None)` on timeout (the helper is detached; see module docs),
/// `Err` if compilation itself failed (invalid module, disabled feature).
fn compile_bounded(engine: &Engine, bytes: &[u8], timeout: Duration) -> Result<Option<Module>> {
    let (tx, rx) = mpsc::channel();
    let engine = engine.clone();
    let owned = bytes.to_vec();
    thread::spawn(move || {
        let _ = tx.send(Module::new(&engine, &owned));
    });
    match rx.recv_timeout(timeout) {
        Ok(result) => Ok(Some(result?)),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            tracing::warn!(
                ?timeout,
                bytes = bytes.len(),
                "wasm_runner: module compile timed out; detaching compile thread"
            );
            Ok(None)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            bail!("wasm_runner: compile thread exited without a result")
        }
    }
}

/// Map a wasmtime invocation error onto a `RunVerdict`. Distinguishes
/// fuel exhaustion, epoch / timeout interruption, WASI proc_exit(0),
/// and generic traps so the calling grader can format useful feedback
/// to the player.
fn classify_trap(err: anyhow::Error, stdout: String, stderr: String) -> RunVerdict {
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
                return RunVerdict {
                    ok: false,
                    stdout,
                    stderr: combine_trap_msg("timeout", stderr),
                };
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

    // Fall-through: a non-trap anyhow error, e.g. "_start not found".
    let msg = format!("execution error: {err}");
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
