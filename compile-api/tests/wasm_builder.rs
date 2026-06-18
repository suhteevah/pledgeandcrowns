// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for the real wasm build-and-run grader.
//!
//! These are SLOW and have a toolchain prerequisite — every case shells
//! out to `cargo build --target wasm32-wasip1` and then executes the
//! result in the wasmtime sandbox. They are gated with `#[ignore]` so the
//! default `cargo test --workspace` (run by the pre-commit hook) skips
//! them.
//!
//! Prerequisite + manual run:
//!
//! ```text
//! rustup target add wasm32-wasip1
//! cargo test -p pledge_compile_api --test wasm_builder -- --ignored
//! ```

use pledge_compile_api::wasm_builder::{build_wasm, build_wasm_with_sandbox_path, compile_and_run};

#[test]
fn build_wasm_rejects_empty_source() {
    assert!(build_wasm("").is_err());
    assert!(build_wasm("   \n\t  ").is_err());
}

#[test]
#[ignore = "slow: shells out to cargo build for wasm32-wasip1"]
fn build_wasm_produces_artifact_for_hello_world() {
    let v = build_wasm("fn main() { println!(\"hello\"); }").unwrap();
    assert!(v.ok, "expected ok build, stderr:\n{}", v.stderr);
    let bytes = v.wasm.expect("ok build must carry wasm bytes");
    assert!(!bytes.is_empty(), "wasm artifact was empty");
    // A wasm module starts with the magic header b"\0asm".
    assert_eq!(&bytes[..4], b"\0asm", "artifact is not a wasm module");
}

#[test]
#[ignore = "slow: shells out to cargo build + runs wasm"]
fn compile_and_run_executes_and_captures_stdout() {
    let outcome = compile_and_run("fn main() { println!(\"forty-two\"); }").unwrap();
    assert!(
        outcome.ok(),
        "expected compile+run ok; compiled={} ran_ok={} stderr:\n{}",
        outcome.compiled,
        outcome.ran_ok,
        outcome.stderr
    );
    assert!(
        outcome.stdout.contains("forty-two"),
        "program stdout not captured, got: {:?}",
        outcome.stdout
    );
}

#[test]
#[ignore = "slow: shells out to cargo build"]
fn compile_and_run_rejects_compile_error() {
    // Type error: assign a &str to an i32 binding.
    let outcome = compile_and_run(r#"fn main() { let _: i32 = "nope"; }"#).unwrap();
    assert!(!outcome.compiled, "type error must not compile");
    assert!(!outcome.ok());
    assert!(
        outcome.stderr.to_lowercase().contains("error"),
        "expected a compiler diagnostic, got:\n{}",
        outcome.stderr
    );
}

#[test]
#[ignore = "slow: shells out to cargo build + runs wasm"]
fn compile_and_run_flags_a_panic_as_run_failure() {
    // Compiles fine, but panics at runtime -> trap -> ran_ok=false.
    let outcome = compile_and_run(r#"fn main() { panic!("boom"); }"#).unwrap();
    assert!(outcome.compiled, "panicking program still compiles");
    assert!(!outcome.ran_ok, "a panic must fail the run");
    assert!(!outcome.ok());
    // The panic message is printed to stderr before the abort trap.
    assert!(
        outcome.stderr.contains("boom") || outcome.stderr.contains("panic"),
        "panic message should reach stderr, got:\n{}",
        outcome.stderr
    );
}

#[test]
#[ignore = "slow: shells out to cargo build + runs wasm"]
fn compile_and_run_traps_an_infinite_loop() {
    // Compiles and runs, but never exits -> fuel exhaustion / timeout.
    let outcome = compile_and_run("fn main() { loop {} }").unwrap();
    assert!(outcome.compiled, "infinite loop still compiles");
    assert!(!outcome.ran_ok, "an infinite loop must not pass");
    let s = outcome.stderr.to_lowercase();
    assert!(
        s.contains("fuel") || s.contains("timeout"),
        "expected fuel/timeout trap, got:\n{}",
        outcome.stderr
    );
}

#[test]
#[ignore = "slow: shells out to cargo build"]
fn compile_and_run_flags_missing_main() {
    // A lib-shaped snippet (no `fn main`) can't build as a bin.
    let outcome = compile_and_run("pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    assert!(!outcome.compiled, "a binary needs a main");
    assert!(!outcome.ok());
}

#[test]
#[ignore = "slow: shells out to cargo build"]
fn build_wasm_cleans_up_sandbox() {
    let (verdict, path) = build_wasm_with_sandbox_path("fn main() { println!(\"ok\"); }").unwrap();
    assert!(verdict.ok, "stderr:\n{}", verdict.stderr);
    assert!(!path.exists(), "sandbox dir leaked at {}", path.display());
}

#[test]
#[ignore = "slow: shells out to cargo build"]
fn build_wasm_cleans_up_sandbox_on_failure() {
    let (verdict, path) = build_wasm_with_sandbox_path("fn main() { broken syntax }").unwrap();
    assert!(!verdict.ok);
    assert!(
        !path.exists(),
        "sandbox dir leaked on failure at {}",
        path.display()
    );
}
