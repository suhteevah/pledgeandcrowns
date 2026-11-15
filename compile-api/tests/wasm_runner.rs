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
    MAX_WASM_BYTES, PIPE_CAP_BYTES, RUN_TIMEOUT_MARGIN, TIMEOUT_PREFIX, run_wasm,
    run_wasm_with_compile_timeout,
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

// ---- Run timeout (Linux verification 2026-11-14, open item 1) -----------
//
// A guest blocked in a host call used to ignore the run timeout entirely
// (only the 50 s job deadline stopped it). Each case below must end within
// its run timeout plus RUN_TIMEOUT_MARGIN and say "timeout:". The extra
// LOADED_HOST_SLACK covers engine creation + compiling the tiny module on
// a box running a 24-job build next to the tests; the failure these tests
// exist for is a run that lasts 3600 s (or 50 s), not one that is late by
// a second.

const RUN_TIMEOUT_MS: u64 = 500;
const LOADED_HOST_SLACK: Duration = Duration::from_secs(2);

/// WASI imports + one page of memory; `$body` is spliced into `_start`.
fn wasi_module(body: &str) -> Vec<u8> {
    compile_wat(&format!(
        r#"
        (module
          (import "wasi_snapshot_preview1" "poll_oneoff"
            (func $poll (param i32 i32 i32 i32) (result i32)))
          (import "wasi_snapshot_preview1" "fd_read"
            (func $fd_read (param i32 i32 i32 i32) (result i32)))
          (import "wasi_snapshot_preview1" "sched_yield"
            (func $yield (result i32)))
          (memory (export "memory") 1)
          (func (export "_start") {body})
        )
        "#
    ))
}

/// Run with a short timeout and huge fuel; return (verdict, elapsed).
fn run_timed(bytes: &[u8]) -> (pledge_compile_api::wasm_runner::RunVerdict, Duration) {
    let started = std::time::Instant::now();
    let v = run_wasm(bytes, RUN_TIMEOUT_MS, u64::MAX / 2, DEFAULT_MEM).expect("a verdict");
    (v, started.elapsed())
}

fn assert_stopped_at_timeout(
    what: &str,
    v: &pledge_compile_api::wasm_runner::RunVerdict,
    took: Duration,
) {
    let bound = Duration::from_millis(RUN_TIMEOUT_MS) + RUN_TIMEOUT_MARGIN + LOADED_HOST_SLACK;
    assert!(!v.ok, "{what}: must not succeed: {v:?}");
    assert!(
        v.stderr.starts_with(TIMEOUT_PREFIX),
        "{what}: expected the timeout verdict, got {v:?}"
    );
    assert!(took < bound, "{what}: took {took:?}, bound {bound:?}");
    assert!(
        took >= Duration::from_millis(RUN_TIMEOUT_MS) - Duration::from_millis(50),
        "{what}: stopped before its timeout ({took:?})"
    );
}

/// One relative monotonic-clock subscription of `ns` nanoseconds at
/// offset 0 (userdata 0, tag 0 = clock, clock id 1, flags 0). This is
/// exactly what Rust's `std::thread::sleep` sends on wasip1.
const SLEEP_SUB_3600S: &str = r#"
    (i32.store (i32.const 16) (i32.const 1))
    (i64.store (i32.const 24) (i64.const 3600000000000))
"#;

#[test]
fn sleep_forever_in_a_host_call_stops_at_the_run_timeout() {
    let bytes = wasi_module(&format!(
        "{SLEEP_SUB_3600S}
         (drop (call $poll (i32.const 0) (i32.const 256) (i32.const 1) (i32.const 512)))"
    ));
    let (v, took) = run_timed(&bytes);
    assert_stopped_at_timeout("relative sleep", &v, took);
}

#[test]
fn absolute_time_poll_on_a_never_ready_deadline_stops_at_the_run_timeout() {
    // Monotonic clock, ABSTIME flag, deadline ~146 years from boot: never
    // ready within any run.
    let bytes = wasi_module(
        r#"
        (i32.store (i32.const 16) (i32.const 1))
        (i64.store (i32.const 24) (i64.const 4611686018427387904))
        (i32.store16 (i32.const 40) (i32.const 1))
        (drop (call $poll (i32.const 0) (i32.const 256) (i32.const 1) (i32.const 512)))
        "#,
    );
    let (v, took) = run_timed(&bytes);
    assert_stopped_at_timeout("absolute-time poll", &v, took);
}

#[test]
fn multi_subscription_poll_stops_at_the_run_timeout() {
    // Two clock subscriptions: skips wasmtime-wasi's single-sleep fast
    // path and goes through the general pollable path.
    let bytes = wasi_module(
        r#"
        (i32.store (i32.const 16) (i32.const 1))
        (i64.store (i32.const 24) (i64.const 3600000000000))
        (i64.store (i32.const 48) (i64.const 1))
        (i32.store (i32.const 64) (i32.const 1))
        (i64.store (i32.const 72) (i64.const 3600000000000))
        (drop (call $poll (i32.const 0) (i32.const 256) (i32.const 2) (i32.const 512)))
        "#,
    );
    let (v, took) = run_timed(&bytes);
    assert_stopped_at_timeout("two-subscription poll", &v, took);
}

#[test]
fn repeated_short_sleeps_stop_at_the_run_timeout() {
    // A loop of 1 ms sleeps: each host call is short, the total is not.
    let bytes = wasi_module(
        r#"
        (i32.store (i32.const 16) (i32.const 1))
        (i64.store (i32.const 24) (i64.const 1000000))
        (loop $l
          (drop (call $poll (i32.const 0) (i32.const 256) (i32.const 1) (i32.const 512)))
          (br $l))
        "#,
    );
    let (v, took) = run_timed(&bytes);
    assert_stopped_at_timeout("loop of short sleeps", &v, took);
}

#[test]
fn busy_loop_with_unlimited_fuel_stops_at_the_run_timeout() {
    let bytes = wasi_module("(loop $l (br $l))");
    let (v, took) = run_timed(&bytes);
    assert_stopped_at_timeout("busy loop", &v, took);
}

#[test]
fn sched_yield_loop_stops_at_the_run_timeout() {
    let bytes = wasi_module("(loop $l (drop (call $yield)) (br $l))");
    let (v, took) = run_timed(&bytes);
    assert_stopped_at_timeout("sched_yield loop", &v, took);
}

#[test]
fn stdin_is_closed_so_reads_return_eof_at_once() {
    // iovec {buf=1024, len=64} at 0; nread at 16. fd_read(0) must succeed
    // with 0 bytes (end of file) without waiting; anything else traps.
    let bytes = wasi_module(
        r#"
        (i32.store (i32.const 0) (i32.const 1024))
        (i32.store (i32.const 4) (i32.const 64))
        (i32.store (i32.const 16) (i32.const 99))
        (if (i32.ne (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 16))
                    (i32.const 0))
          (then unreachable))
        (if (i32.ne (i32.load (i32.const 16)) (i32.const 0))
          (then unreachable))
        "#,
    );
    let (v, took) = run_timed(&bytes);
    assert!(v.ok, "closed stdin must read as EOF: {v:?}");
    assert!(
        took < Duration::from_millis(RUN_TIMEOUT_MS),
        "took {took:?}"
    );
}

#[test]
fn stdin_read_subscription_is_ready_at_once() {
    // poll_oneoff on fd 0 (tag 1 = fd_read): a closed stream is always
    // ready, so this returns immediately instead of waiting.
    let bytes = wasi_module(
        r#"
        (i32.store8 (i32.const 8) (i32.const 1))
        (i32.store (i32.const 16) (i32.const 0))
        (if (i32.ne (call $poll (i32.const 0) (i32.const 256) (i32.const 1) (i32.const 512))
                    (i32.const 0))
          (then unreachable))
        "#,
    );
    let (v, took) = run_timed(&bytes);
    assert!(v.ok, "{v:?}");
    assert!(
        took < Duration::from_millis(RUN_TIMEOUT_MS),
        "took {took:?}"
    );
}

#[test]
fn a_short_sleep_inside_the_budget_still_works() {
    // 100 ms relative sleep under a 2 s timeout: the async path must still
    // sleep for real and then let the program finish cleanly.
    let bytes = wasi_module(
        r#"
        (i32.store (i32.const 16) (i32.const 1))
        (i64.store (i32.const 24) (i64.const 100000000))
        (if (i32.ne (call $poll (i32.const 0) (i32.const 256) (i32.const 1) (i32.const 512))
                    (i32.const 0))
          (then unreachable))
        "#,
    );
    let started = std::time::Instant::now();
    let v = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM).unwrap();
    let took = started.elapsed();
    assert!(v.ok, "{v:?}");
    assert!(took >= Duration::from_millis(95), "slept only {took:?}");
}

// ---- Security settings pinned against silent wasmtime default changes ----

#[test]
fn guest_sees_no_args_no_env_no_preopens_and_no_sockets() {
    let bytes = compile_wat(
        r#"
        (module
          (import "wasi_snapshot_preview1" "args_sizes_get"
            (func $args (param i32 i32) (result i32)))
          (import "wasi_snapshot_preview1" "environ_sizes_get"
            (func $env (param i32 i32) (result i32)))
          (import "wasi_snapshot_preview1" "fd_prestat_get"
            (func $prestat (param i32 i32) (result i32)))
          (import "wasi_snapshot_preview1" "path_open"
            (func $open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
          (import "wasi_snapshot_preview1" "sock_accept"
            (func $accept (param i32 i32 i32) (result i32)))
          (memory (export "memory") 1)
          (data (i32.const 2048) "etc/passwd")
          (func (export "_start")
            ;; args: count 0, buffer 0
            (i32.store (i32.const 0) (i32.const 7))
            (i32.store (i32.const 4) (i32.const 7))
            (if (call $args (i32.const 0) (i32.const 4)) (then unreachable))
            (if (i32.load (i32.const 0)) (then unreachable))
            (if (i32.load (i32.const 4)) (then unreachable))
            ;; env: count 0, buffer 0
            (i32.store (i32.const 0) (i32.const 7))
            (i32.store (i32.const 4) (i32.const 7))
            (if (call $env (i32.const 0) (i32.const 4)) (then unreachable))
            (if (i32.load (i32.const 0)) (then unreachable))
            (if (i32.load (i32.const 4)) (then unreachable))
            ;; fd 3 (the first preopen slot) does not exist: EBADF = 8
            (if (i32.ne (call $prestat (i32.const 3) (i32.const 0)) (i32.const 8))
              (then unreachable))
            ;; path_open relative to fd 3: EBADF
            (if (i32.ne (call $open (i32.const 3) (i32.const 0) (i32.const 2048) (i32.const 10)
                                    (i32.const 0) (i64.const 0) (i64.const 0) (i32.const 0)
                                    (i32.const 0))
                        (i32.const 8))
              (then unreachable))
            ;; no socket fd exists to accept on
            (if (i32.eqz (call $accept (i32.const 3) (i32.const 0) (i32.const 0)))
              (then unreachable))
          )
        )
        "#,
    );
    let v = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM).unwrap();
    assert!(v.ok, "capability probe failed (a check trapped): {v:?}");
}

#[test]
fn linear_memory_growth_is_capped_by_the_resource_limiter() {
    // DEFAULT_MEM = 8 MiB = 128 pages. Growing by 200 pages must fail.
    let bytes = compile_wat(
        r#"
        (module
          (memory (export "memory") 1)
          (func (export "_start")
            (if (i32.ne (memory.grow (i32.const 200)) (i32.const -1))
              (then unreachable))))
        "#,
    );
    let v = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM).unwrap();
    assert!(v.ok, "memory.grow past the cap must return -1, got {v:?}");
}

#[test]
fn table_count_and_initial_size_are_capped_by_the_resource_limiter() {
    use pledge_compile_api::wasm_runner::{MAX_TABLE_ELEMENTS, MAX_TABLES};
    assert_eq!((MAX_TABLES, MAX_TABLE_ELEMENTS), (2, 64 * 1024));
    // Two tables are fine (reference types allows several).
    let two = compile_wat(
        r#"(module (memory (export "memory") 1) (table 1 funcref) (table 1 funcref)
             (func (export "_start")))"#,
    );
    assert!(
        run_wasm(&two, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM)
            .unwrap()
            .ok
    );
    // A third table, or one table over the element cap, fails to
    // instantiate: a verdict, not a crash.
    for wat_src in [
        r#"(module (memory (export "memory") 1) (table 1 funcref) (table 1 funcref)
             (table 1 funcref) (func (export "_start")))"#,
        r#"(module (memory (export "memory") 1) (table 100000 funcref)
             (func (export "_start")))"#,
    ] {
        let v = run_wasm(
            &compile_wat(wat_src),
            FAST_TIMEOUT_MS,
            DEFAULT_FUEL,
            DEFAULT_MEM,
        )
        .expect("instantiation failure is a verdict");
        assert!(!v.ok, "must be refused by the limiter: {v:?}");
    }
}

#[test]
fn every_proposal_outside_the_allow_list_is_refused() {
    // Each module uses exactly one proposal the runner must refuse. If a
    // wasmtime upgrade (or a Config edit) lets one through, this names it.
    let cases: &[(&str, &str)] = &[
        (
            "threads (shared memory)",
            r#"(module (memory 1 1 shared) (func (export "_start")))"#,
        ),
        (
            "threads (atomics)",
            r#"(module (memory 1) (func (export "_start")
                 (drop (i32.atomic.load (i32.const 0)))))"#,
        ),
        (
            "gc",
            r#"(module (type $s (struct)) (func (export "_start")
                 (drop (struct.new $s))))"#,
        ),
        (
            "function references",
            r#"(module (type $t (func)) (func $f (type $t))
                 (elem declare func $f)
                 (func (export "_start") (call_ref $t (ref.func $f))))"#,
        ),
        (
            "exceptions",
            r#"(module (tag $e) (func (export "_start") (throw $e)))"#,
        ),
        (
            "tail calls",
            r#"(module (func $f) (func (export "_start") (return_call $f)))"#,
        ),
        (
            "memory64",
            r#"(module (memory i64 1) (func (export "_start")))"#,
        ),
        (
            "multi-memory",
            r#"(module (memory 1) (memory 1) (func (export "_start")))"#,
        ),
        (
            "extended const",
            r#"(module (global i32 (i32.add (i32.const 1) (i32.const 2)))
                 (func (export "_start")))"#,
        ),
        (
            "simd",
            r#"(module (func (export "_start") (drop (v128.const i64x2 0 0))))"#,
        ),
        (
            "wide arithmetic",
            r#"(module (func (export "_start")
                 (drop (drop (i64.add128 (i64.const 0) (i64.const 0)
                                         (i64.const 0) (i64.const 0))))))"#,
        ),
        (
            "custom page sizes",
            r#"(module (memory 1 (pagesize 1)) (func (export "_start")))"#,
        ),
        ("component model", r#"(component)"#),
    ];
    for (name, src) in cases {
        let bytes = compile_wat(src);
        let r = run_wasm(&bytes, FAST_TIMEOUT_MS, DEFAULT_FUEL, DEFAULT_MEM);
        assert!(
            r.is_err(),
            "{name}: must be refused at validation, got {r:?}"
        );
    }
}
