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

use std::time::Duration;

use pledge_compile_api::wasm_runner::{
    MAX_WASM_BYTES, PIPE_CAP_BYTES, run_wasm, run_wasm_with_compile_timeout,
};

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
fn accepts_baseline_bulk_memory_feature() {
    // bulk-memory is WebAssembly 2.0 baseline and the Rust wasm32-wasip1
    // std emits it (memcpy/memset lower to memory.copy/memory.fill), so
    // `run_wasm` MUST accept it — disabling it rejected every real Rust
    // module with a "zero byte expected" translation error (the bug the
    // build-and-run path's first end-to-end test surfaced, 2026-06-18).
    // A zero-length memory.copy is a valid no-op under the proposal.
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

    let v = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM)
        .expect("bulk-memory is baseline and must validate + run");
    assert!(v.ok, "expected ok=true for a bulk-memory no-op, got {v:?}");
}

#[test]
fn rejects_disabled_simd_feature() {
    // Defence-in-depth check, re-pointed from bulk-memory (which we now
    // correctly allow) at a proposal we genuinely keep DENIED: SIMD.
    // The curriculum needs no `v128`; it's real attack surface, so a
    // module using a SIMD instruction must fail validation, NOT run.
    //
    // If a future wasmtime release flips the SIMD default on and we
    // forget to re-deny it, this test trips before any student
    // submission can. (Same guard for multi-memory/memory64/threads/
    // tail-call/gc — SIMD is the cheap single-instruction canary.)
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (func (export "_start")
            (drop (v128.const i32x4 0 0 0 0))))
        "#,
    );

    let result = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM);
    let err = result.expect_err(
        "module using disabled SIMD feature must NOT validate; \
         got Ok — has the Config deny-list regressed?",
    );
    let msg = format!("{err:#}").to_lowercase();
    assert!(
        msg.contains("simd")
            || msg.contains("v128")
            || msg.contains("not enabled")
            || msg.contains("disabled")
            || msg.contains("unsupported"),
        "error should reference the disabled SIMD feature, got {:?}",
        err
    );
}

#[test]
fn rejects_oversized_module_before_compiling() {
    // A syntactically valid module padded with a custom section past the
    // cap. Refused on size alone.
    let mut bytes = compile_wat(r#"(module (func (export "_start")))"#);
    let pad = MAX_WASM_BYTES + 1 - bytes.len();
    // Custom section: id 0, LEB128 size, name length 1, name "p", payload.
    let payload_len = pad + 2;
    bytes.push(0);
    let mut n = payload_len;
    loop {
        let mut b = (n & 0x7f) as u8;
        n >>= 7;
        if n != 0 {
            b |= 0x80;
        }
        bytes.push(b);
        if n == 0 {
            break;
        }
    }
    bytes.push(1);
    bytes.push(b'p');
    bytes.resize(bytes.len() + pad, 0);
    assert!(bytes.len() > MAX_WASM_BYTES);
    let err = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM)
        .expect_err("oversized module must be refused");
    assert!(format!("{err}").contains("limit"), "{err}");
}

#[test]
fn module_compile_timeout_fails_the_run() {
    // A zero compile budget can never be met: exercises the timeout path
    // deterministically without needing a pathological module.
    let bytes = compile_wat(r#"(module (memory (export "memory") 1) (func (export "_start")))"#);
    let v = run_wasm_with_compile_timeout(
        &bytes,
        FAST_TIMEOUT_MS,
        DEFAULT_FUEL,
        DEFAULT_MEM,
        Duration::ZERO,
    )
    .expect("compile timeout is a verdict, not an Err");
    assert!(!v.ok);
    assert!(v.stderr.contains("module compile exceeded"), "{v:?}");
}

#[test]
fn table_growth_is_capped_by_the_resource_limiter() {
    // table.grow by 1M elements must fail (-1) under the element cap,
    // and the module traps on that result.
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (table 1 funcref)
          (func (export "_start")
            (if (i32.ne
                  (table.grow (ref.null func) (i32.const 1000000))
                  (i32.const -1))
              (then unreachable))))
        "#,
    );
    let v = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM).unwrap();
    assert!(v.ok, "table.grow past the cap must return -1, got {v:?}");
}

#[test]
fn second_memory_or_instance_is_refused() {
    // multi-memory is disabled at the engine level, so a two-memory
    // module fails validation; the store limit (1 memory) backs it up.
    let bytes = compile_wat(r#"(module (memory 1) (memory 1) (func (export "_start")))"#);
    assert!(run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM).is_err());
}

#[test]
fn pipe_cap_constant_is_64k() {
    // Stability: callers may rely on this for sizing buffers downstream.
    assert_eq!(PIPE_CAP_BYTES, 64 * 1024);
}
