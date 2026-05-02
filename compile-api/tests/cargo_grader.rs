// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for the real `cargo check` grader.
//!
//! These tests are SLOW — every case shells out to `cargo check`, which
//! is ~1-3s warm and 10-30s cold. They are gated with `#[ignore]` so
//! the default `cargo test --workspace` (which runs in the pre-commit
//! hook) skips them.
//!
//! Run them manually:
//!
//! ```text
//! cargo test --workspace -- --ignored
//! ```

use pledge_compile_api::cargo_grader::{compile_check, compile_check_with_sandbox_path};

#[test]
#[ignore = "slow: shells out to cargo check"]
fn compile_check_accepts_trivially_valid_code() {
    let v = compile_check("pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    assert!(v.ok, "expected ok, got fail with stderr:\n{}", v.stderr);
}

#[test]
#[ignore = "slow: shells out to cargo check"]
fn compile_check_rejects_syntax_error() {
    // Missing closing brace.
    let v = compile_check("pub fn add(a: i32, b: i32) -> i32 { a + b ").unwrap();
    assert!(!v.ok, "expected fail, got ok. stderr:\n{}", v.stderr);
    assert!(
        v.stderr.to_lowercase().contains("error"),
        "expected stderr to mention error, got:\n{}",
        v.stderr
    );
}

#[test]
#[ignore = "slow: shells out to cargo check"]
fn compile_check_rejects_type_error() {
    let v = compile_check(r#"pub fn bad() { let _: i32 = "not an int"; }"#).unwrap();
    assert!(!v.ok, "expected type error, got ok. stderr:\n{}", v.stderr);
}

#[test]
fn compile_check_rejects_empty_source() {
    assert!(compile_check("").is_err());
    assert!(compile_check("   \n\t  ").is_err());
}

#[test]
#[ignore = "slow: shells out to cargo check"]
fn compile_check_cleans_up_sandbox() {
    let (verdict, path) = compile_check_with_sandbox_path("pub fn ok() -> i32 { 1 }").unwrap();
    assert!(verdict.ok);
    assert!(!path.exists(), "sandbox dir leaked at {}", path.display());
}

#[test]
#[ignore = "slow: shells out to cargo check"]
fn compile_check_cleans_up_sandbox_on_failure() {
    let (verdict, path) =
        compile_check_with_sandbox_path("pub fn nope() { broken syntax }").unwrap();
    assert!(!verdict.ok);
    assert!(
        !path.exists(),
        "sandbox dir leaked on failure at {}",
        path.display()
    );
}
