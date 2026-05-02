// SPDX-License-Identifier: AGPL-3.0-or-later
//! End-to-end HTTP integration test. Spins up the real axum router on
//! an OS-assigned port, fires reqwest at it, and verifies the wire
//! contract per mission. Catches regressions the unit tests can't:
//! JSON shape, status codes, content-type negotiation.

use pledge_compile_api::{CompileResponse, HealthResponse, make_router};
use std::net::SocketAddr;

async fn spawn_server() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = make_router();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    // Tiny delay so the server is ready when the first request lands.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

async fn post_compile(
    addr: SocketAddr,
    encounter_id: &str,
    source: &str,
) -> (reqwest::StatusCode, CompileResponse) {
    let body = serde_json::json!({"source": source, "encounter_id": encounter_id});
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/compile"))
        .json(&body)
        .send()
        .await
        .expect("request failed");
    let status = resp.status();
    let body: CompileResponse = resp.json().await.expect("non-json response");
    (status, body)
}

#[tokio::test]
async fn health_returns_ok_with_version() {
    let addr = spawn_server().await;
    let resp: HealthResponse = reqwest::get(format!("http://{addr}/health"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp.status, "ok");
    assert!(!resp.version.is_empty());
}

#[tokio::test]
async fn compile_rejects_empty_source() {
    let addr = spawn_server().await;
    let (status, body) = post_compile(addr, "intro_let_binding", "   ").await;
    assert_eq!(status.as_u16(), 400);
    assert!(!body.ok);
    assert_eq!(body.stderr, "empty source");
}

#[tokio::test]
async fn compile_rejects_oversized_source() {
    let addr = spawn_server().await;
    // 65 KiB — just over the 64 KiB cap. The pattern grader is fast
    // for any input but we cap at the route boundary so the cargo
    // path (and rustc's parser) never sees an unbounded body.
    let big = "// pad\n".repeat(10_000);
    assert!(big.len() > 64 * 1024);
    let (status, body) = post_compile(addr, "intro_let_binding", &big).await;
    assert_eq!(status.as_u16(), 413);
    assert!(!body.ok);
    assert!(
        body.stderr.contains("exceeds limit"),
        "stderr should explain the cap, got: {}",
        body.stderr
    );
}

#[tokio::test]
async fn compile_passes_intro_solution() {
    let addr = spawn_server().await;
    let src = "fn main() { let answer = 42; }";
    let (status, body) = post_compile(addr, "intro_let_binding", src).await;
    assert_eq!(status.as_u16(), 200);
    assert!(body.ok, "expected ok=true, got {body:?}");
    assert!(body.stdout.contains("Ferris"));
}

#[tokio::test]
async fn compile_fails_intro_with_wrong_value() {
    let addr = spawn_server().await;
    let (status, body) =
        post_compile(addr, "intro_let_binding", "fn main() { let answer = 7; }").await;
    assert_eq!(status.as_u16(), 200);
    assert!(!body.ok);
    assert!(body.stderr.contains("42"));
}

#[tokio::test]
async fn compile_passes_double_function() {
    let addr = spawn_server().await;
    let src = "fn double(n: i32) -> i32 { n * 2 }";
    let (_, body) = post_compile(addr, "double_function", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Trait Mage"));
}

#[tokio::test]
async fn compile_passes_borrow_preview() {
    let addr = spawn_server().await;
    let src = r#"fn main() { let value = String::from("x"); let r = &value; println!("{r}"); }"#;
    let (_, body) = post_compile(addr, "borrow_preview", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Borrow Checker"));
}

#[tokio::test]
async fn compile_passes_loop_break() {
    let addr = spawn_server().await;
    let src = "fn main() { let mut n = 1; loop { if n >= 100 { break n; } n *= 2; }; }";
    let (_, body) = post_compile(addr, "loop_break", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Bellringer"));
}

#[tokio::test]
async fn compile_passes_match_option() {
    let addr = spawn_server().await;
    let src = "fn f(x: Option<i32>) -> i32 { match x { Some(n) => n, None => 0 } }";
    let (_, body) = post_compile(addr, "match_option", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Oracle"));
}

#[tokio::test]
async fn compile_passes_struct_basic() {
    let addr = spawn_server().await;
    let src = r#"struct Knight { name: String, hp: i32 }
fn main() { let k = Knight { name: String::from("g"), hp: 1 }; println!("{}", k.name); }"#;
    let (_, body) = post_compile(addr, "struct_basic", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Herald"));
}

#[tokio::test]
async fn compile_passes_vec_iter() {
    let addr = spawn_server().await;
    let src =
        "fn main() { let v = vec![1, 2, 3]; let s: i32 = v.iter().sum(); println!(\"{s}\"); }";
    let (_, body) = post_compile(addr, "vec_iter", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Cooper"));
}

#[tokio::test]
async fn compile_passes_tuple_destructure() {
    let addr = spawn_server().await;
    let src = "fn main() { let (a, b) = (3, 4); println!(\"{a} {b}\"); }";
    let (_, body) = post_compile(addr, "tuple_destructure", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Twin"));
}

#[tokio::test]
async fn compile_passes_while_loop() {
    let addr = spawn_server().await;
    let src = "fn main() { let mut n = 5; while n > 0 { n -= 1; } println!(\"{n}\"); }";
    let (_, body) = post_compile(addr, "while_loop", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Tinker"));
}

#[tokio::test]
async fn compile_passes_borrow_mut() {
    let addr = spawn_server().await;
    let src = "fn bump(x: &mut i32) { *x = 99; }";
    let (_, body) = post_compile(addr, "borrow_mut", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Forgewright"));
}

#[tokio::test]
async fn compile_passes_string_vs_str() {
    let addr = spawn_server().await;
    let src = r#"fn greet(name: &str) { let _ = name; } fn main() { let s = String::from("x"); greet(&s); }"#;
    let (_, body) = post_compile(addr, "string_vs_str", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Linguist"));
}

#[tokio::test]
async fn compile_passes_option_unwrap_or() {
    let addr = spawn_server().await;
    let src = "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }";
    let (_, body) = post_compile(addr, "option_unwrap_or", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Pilgrim"));
}

#[tokio::test]
async fn compile_passes_for_in_range() {
    let addr = spawn_server().await;
    let src = "fn main() { for i in 0..10 { let _ = i; } }";
    let (_, body) = post_compile(addr, "for_in_range", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Drillmaster"));
}

#[tokio::test]
async fn compile_passes_closure_basic() {
    let addr = spawn_server().await;
    let src = "fn main() { let add = |a, b| a + b; let _ = add(1, 2); }";
    let (_, body) = post_compile(addr, "closure_basic", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Reckoner"));
}

#[tokio::test]
async fn compile_passes_slice_basic() {
    let addr = spawn_server().await;
    let src = "fn sum_slice(xs: &[i32]) -> i32 { xs.iter().sum() }";
    let (_, body) = post_compile(addr, "slice_basic", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Quartermaster"));
}

#[tokio::test]
async fn compile_passes_result_question_mark() {
    let addr = spawn_server().await;
    let src = "fn parse_int(s: &str) -> Result<i32, String> { let n = s.parse::<i32>().map_err(|e| e.to_string())?; Ok(n) }";
    let (_, body) = post_compile(addr, "result_question_mark", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Auditor"));
}

#[tokio::test]
async fn compile_passes_derive_debug() {
    let addr = spawn_server().await;
    let src = "#[derive(Debug)] struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{i:?}\"); }";
    let (_, body) = post_compile(addr, "derive_debug", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Chronicler"));
}

#[tokio::test]
async fn compile_passes_iter_map_collect() {
    let addr = spawn_server().await;
    let src =
        "fn main() { let v = vec![1, 2, 3]; let _: Vec<i32> = v.iter().map(|x| x * 2).collect(); }";
    let (_, body) = post_compile(addr, "iter_map_collect", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Alchemist"));
}

#[tokio::test]
async fn compile_passes_enum_match() {
    let addr = spawn_server().await;
    let src = "enum Direction { N, S } fn f(d: Direction) -> i32 { match d { Direction::N => 1, Direction::S => 0 } }";
    let (_, body) = post_compile(addr, "enum_match", src).await;
    assert!(body.ok);
    assert!(body.stdout.contains("Heraldic Sage"));
}

#[tokio::test]
async fn compile_unknown_encounter_falls_through_to_freeform() {
    let addr = spawn_server().await;
    let (_, body) = post_compile(addr, "no_such_mission", "fn main() {}").await;
    assert!(body.ok);
    assert!(body.stdout.contains("freeform"));
    assert!(body.stdout.contains("no_such_mission"));
}
