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
//!   thread.
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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Result, bail};
use wasmtime::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;

/// Cap on captured stdout / stderr per stream. 64 KiB.
pub const PIPE_CAP_BYTES: usize = 64 * 1024;

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
    if bytes.is_empty() {
        bail!("wasm_runner: bytes is empty");
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
    // Defence-in-depth: deny features we don't need. Threads adds
    // shared-memory shenanigans we explicitly do not want for student
    // submissions.
    config.wasm_threads(false);

    let engine = Engine::new(&config)?;
    let module = Module::new(&engine, bytes)?;

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
        .memory_size(max_memory_bytes as usize)
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
    // Park a thread that bumps the engine epoch after timeout_ms.
    // Use a "done" flag so we don't fire after a fast-finishing run
    // (cosmetic — the tick is harmless once the store is dropped).
    let done = Arc::new(AtomicBool::new(false));
    let watchdog_done = done.clone();
    let watchdog_engine = engine.clone();
    let watchdog = thread::spawn(move || {
        // Sleep in a single shot; epoch interruption is one-tick.
        thread::sleep(Duration::from_millis(timeout_ms));
        if !watchdog_done.load(Ordering::SeqCst) {
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

    // Tell the watchdog it doesn't need to fire (best-effort) and join.
    done.store(true, Ordering::SeqCst);
    let _ = watchdog.join();

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
