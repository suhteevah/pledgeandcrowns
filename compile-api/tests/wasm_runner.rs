// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for `wasm_runner`. Tests synthesize wasm bytes
//! from inline WAT via the `wat` crate so they need no external
//! toolchain.
//!
//! Tests that exercise wasmtime instantiation can be slow on a cold
//! cache; the timeout / fuel exhaustion cases are bounded explicitly
//! to keep them fast. If any case starts exceeding ~2s on Matt's
//! kokonoe box, mark it `#[ignore]` and document why like
//! `cargo_grader.rs` does.

use pledge_compile_api::wasm_runner::{PIPE_CAP_BYTES, run_wasm};

/// 200 ms, 10M fuel, 8 MiB memory — comfortable defaults for the
/// trivial / well-behaved cases.
const FAST_TIMEOUT_MS: u64 = 2_000;
const DEFAULT_FUEL: u64 = 10_000_000;
const DEFAULT_MEM: u64 = 8 * 1024 * 1024;

fn compile_wat(src: &str) -> Vec<u8> {
    wat::parse_str(src).expect("test wat must parse")
}

#[test]
fn runs_a_trivial_module() {
    // Minimal WASI-style module: exports a no-op `_start` plus a
    // memory (some WASI implementations require a memory export).
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (func (export "_start"))
        )
        "#,
    );

    let v = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM)
        .expect("run_wasm should succeed for a trivial module");
    assert!(v.ok, "expected ok=true, got verdict {v:?}");
    assert!(
        v.stderr.is_empty(),
        "expected empty stderr, got {:?}",
        v.stderr
    );
}

#[test]
fn traps_on_unreachable() {
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (func (export "_start") unreachable)
        )
        "#,
    );

    let v = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM)
        .expect("trap is a normal verdict, not an Err");
    assert!(!v.ok, "expected ok=false on trap, got {v:?}");
    let s = v.stderr.to_lowercase();
    assert!(
        s.contains("trap") || s.contains("unreachable"),
        "stderr should mention trap/unreachable, got {:?}",
        v.stderr
    );
}

#[test]
fn times_out_on_infinite_loop() {
    // Tight infinite loop. `loop ... br 0` is the canonical wat for it.
    // Give it gobs of fuel so the watchdog wins, not the fuel meter.
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (func (export "_start")
            (loop $l
              br $l)))
        "#,
    );

    let v = run_wasm(
        &bytes,
        200,          // timeout_ms — short for a fast test
        u64::MAX / 2, // fuel — effectively unbounded
        DEFAULT_MEM,
    )
    .expect("timeout is a normal verdict, not an Err");
    assert!(!v.ok, "expected ok=false on timeout, got {v:?}");
    let s = v.stderr.to_lowercase();
    assert!(
        s.contains("timeout") || s.contains("fuel"),
        "stderr should mention timeout or fuel, got {:?}",
        v.stderr
    );
}

#[test]
fn runs_out_of_fuel_on_long_compute() {
    // Same shape as the timeout test but with a tiny fuel budget so
    // fuel is exhausted long before the watchdog could fire.
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (func (export "_start")
            (loop $l
              br $l)))
        "#,
    );

    let v = run_wasm(
        &bytes,
        FAST_TIMEOUT_MS,
        // Above the MIN_FUEL floor (1k) but small enough to exhaust
        // before a 2s watchdog tick.
        50_000,
        DEFAULT_MEM,
    )
    .expect("fuel exhaustion is a normal verdict, not an Err");
    assert!(!v.ok, "expected ok=false on fuel exhaustion, got {v:?}");
    let s = v.stderr.to_lowercase();
    assert!(
        s.contains("fuel"),
        "stderr should mention fuel, got {:?}",
        v.stderr
    );
}

#[test]
fn rejects_empty_bytes() {
    let err = run_wasm(&[], FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM)
        .expect_err("empty bytes must bail");
    let msg = format!("{err}").to_lowercase();
    assert!(
        msg.contains("empty"),
        "error should mention 'empty', got {:?}",
        err
    );
}

#[test]
fn rejects_disabled_bulk_memory_feature() {
    // Defence-in-depth check: bulk-memory is one of several wasm
    // proposals we explicitly disable in `wasm_runner::run_wasm`'s
    // `Config`. A module that uses `memory.copy` (a bulk-memory
    // instruction) should fail to compile/instantiate, NOT silently run.
    //
    // If a future wasmtime release flips a default and we forget to
    // deny it, this test trips before any student submission can.
    //
    // We expect `run_wasm` to return Err(...) at module compile time
    // (`Module::new` fails validation when the feature is disabled).
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (func (export "_start")
            i32.const 0
            i32.const 0
            i32.const 0
            memory.copy))
        "#,
    );

    let result = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM);
    let err = result.expect_err(
        "module using disabled bulk-memory feature must NOT validate; \
         got Ok — has the Config deny-list regressed?",
    );
    let msg = format!("{err:#}").to_lowercase();
    assert!(
        msg.contains("bulk memory")
            || msg.contains("bulk-memory")
            || msg.contains("memory.copy")
            || msg.contains("not enabled")
            || msg.contains("disabled"),
        "error should reference the disabled feature, got {:?}",
        err
    );
}

#[test]
fn pipe_cap_constant_is_64k() {
    // Stability: callers may rely on this for sizing buffers downstream.
    assert_eq!(PIPE_CAP_BYTES, 64 * 1024);
}
