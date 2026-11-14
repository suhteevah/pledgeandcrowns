// SPDX-License-Identifier: AGPL-3.0-or-later
//! End-to-end HTTP integration test for the `/compile-real` route.
//!
//! These tests are SLOW — every case shells out to a real
//! `cargo build --target wasm32-wasip1` via `wasm_builder` and runs the
//! result in the wasmtime sandbox (~1-3s warm). They are gated with
//! `#[ignore]` so the default `cargo test --workspace` (pre-commit hook)
//! skips them. Requires `rustup target add wasm32-wasip1`.
//!
//! Run them manually:
//!
//! ```text
//! cargo test -p pledge_compile_api --test http_real -- --ignored
//! ```

use pledge_compile_api::{CompileResponse, make_router, serve};
use std::net::SocketAddr;

async fn spawn_server() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = make_router();
    tokio::spawn(async move {
        serve(listener, app).await.unwrap();
    });
    // Tiny delay so the server is ready when the first request lands.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

async fn post_compile_real(
    addr: SocketAddr,
    encounter_id: &str,
    source: &str,
) -> (reqwest::StatusCode, CompileResponse) {
    let body = serde_json::json!({"source": source, "encounter_id": encounter_id});
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/compile-real"))
        .json(&body)
        .send()
        .await
        .expect("request failed");
    let status = resp.status();
    let body: CompileResponse = resp.json().await.expect("non-json response");
    (status, body)
}

#[tokio::test]
#[ignore = "slow: shells out to cargo check"]
async fn compile_real_passes_canonical_intro() {
    let addr = spawn_server().await;
    // The intro mission's expectation is a printed `42` (the task says to
    // update the println!), so the canonical answer prints it.
    let src = "fn main() { let answer = 42; println!(\"{answer}\"); }";
    let (status, body) = post_compile_real(addr, "intro_let_binding", src).await;
    assert_eq!(status.as_u16(), 200);
    assert!(body.ok, "expected ok=true, got {body:?}");
    assert!(
        body.stdout.starts_with("[real]"),
        "expected [real] marker, got stdout={:?}",
        body.stdout
    );
    // /compile-real returns the build+run verdict plus the mission's
    // output check — no pattern-grader flavor stitched in.
    assert_eq!(
        body.stdout, "[real] 42\n-- checked: does what the mission asks",
        "expected program output + checked marker"
    );
    assert_eq!(body.behaviour_checked, Some(true));
}

#[tokio::test]
#[ignore = "slow: shells out to cargo build + runs wasm"]
async fn compile_real_rejects_a_silent_intro() {
    // Compiles and runs cleanly, but never prints the answer: before
    // per-mission expectations this was a pass.
    let addr = spawn_server().await;
    let (status, body) =
        post_compile_real(addr, "intro_let_binding", "fn main() { let answer = 42; }").await;
    assert_eq!(status.as_u16(), 200);
    assert!(!body.ok, "{body:?}");
    assert_eq!(body.behaviour_checked, Some(true));
}

#[tokio::test]
#[ignore = "slow: shells out to cargo build + runs wasm"]
async fn compile_real_returns_program_stdout() {
    let addr = spawn_server().await;
    let src = r#"fn main() { let v: Vec<i32> = (1..=4).collect(); println!("sum={}", v.iter().sum::<i32>()); }"#;
    let (status, body) = post_compile_real(addr, "vec_iter", src).await;
    assert_eq!(status.as_u16(), 200);
    assert!(body.ok, "expected ok=true, got {body:?}");
    assert!(body.stdout.contains("sum=10"), "{body:?}");
}

#[tokio::test]
async fn compile_real_rejects_empty_source() {
    let addr = spawn_server().await;
    let (status, body) = post_compile_real(addr, "intro_let_binding", "   ").await;
    assert_eq!(status.as_u16(), 400);
    assert!(!body.ok);
    assert_eq!(body.stderr, "empty source");
}

#[tokio::test]
#[ignore = "slow: shells out to cargo check"]
async fn compile_real_rejects_genuine_syntax_error() {
    let addr = spawn_server().await;
    // Unclosed brace — rustc must complain.
    let (status, body) = post_compile_real(addr, "intro_let_binding", "fn main() {").await;
    assert_eq!(status.as_u16(), 200);
    assert!(!body.ok, "expected ok=false, got {body:?}");
    let lower = body.stderr.to_lowercase();
    assert!(
        lower.contains("error"),
        "expected rustc error in stderr, got:\n{}",
        body.stderr
    );
    // Either a "this file contains an unclosed delimiter" or generic
    // parse error — both are acceptable signals of a syntax failure.
    assert!(
        lower.contains("brace") || lower.contains("delimiter") || lower.contains("expected"),
        "expected unmatched-brace / parse hint in stderr, got:\n{}",
        body.stderr
    );
}

#[tokio::test]
#[ignore = "slow: shells out to cargo check"]
async fn compile_real_rejects_type_error() {
    let addr = spawn_server().await;
    let src = r#"fn main() { let _: i32 = "x"; }"#;
    let (status, body) = post_compile_real(addr, "intro_let_binding", src).await;
    assert_eq!(status.as_u16(), 200);
    assert!(!body.ok, "expected ok=false, got {body:?}");
    let lower = body.stderr.to_lowercase();
    assert!(
        lower.contains("mismatched") || lower.contains("expected `i32`") || lower.contains("type"),
        "expected type-mismatch hint in stderr, got:\n{}",
        body.stderr
    );
}
