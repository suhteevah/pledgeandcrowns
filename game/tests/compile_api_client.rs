// SPDX-License-Identifier: MIT
//! Native end-to-end: the game's compile client against the real
//! compile-API router, served in-process on a loopback port.
//!
//! The server crate is a dev-dependency only (AGPL code never reaches the
//! shipped MIT binary). Each test gets its own server so rate-limit and
//! queue state never leak between tests.
//!
//! The client is driven exactly as the plugin drives it on native:
//! `post_compile_blocking` from a plain thread with no Tokio context
//! (the server's runtime lives on a separate thread).

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use pledge_and_crown::plugins::compile_client::config::{
    ApiEndpoint, ApiToken, CompileMode, DEFAULT_PATTERN_TIMEOUT,
};
use pledge_and_crown::plugins::compile_client::outcome::{
    ApiOutcome, classify, verdict_with_server,
};
use pledge_and_crown::plugins::compile_client::transport::post_compile_blocking;
use pledge_and_crown::plugins::stub_grader::stub_verdict;
use pledge_compile_api::config::Secret;
use pledge_compile_api::wasm_builder::RealOutcome;
use pledge_compile_api::{AppState, Config, build_router, serve};

const TOKEN: &str = "0123456789abcdef0123456789abcdef-test";
const PASSING: &str = "fn main() { let answer = 42; }";
const MISSION: &str = "intro_let_binding";

/// Start `state`'s router on 127.0.0.1:0 in a background runtime thread.
fn start_server(state: AppState) -> SocketAddr {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("server runtime");
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind");
            tx.send(listener.local_addr().expect("addr"))
                .expect("send addr");
            serve(listener, build_router(state)).await.expect("serve");
        });
    });
    rx.recv_timeout(Duration::from_secs(10))
        .expect("server did not start")
}

fn authed_config() -> Config {
    Config {
        auth_tokens: vec![Secret::new(TOKEN)],
        ..Config::default()
    }
}

fn endpoint(addr: SocketAddr, token: Option<&str>, mode: CompileMode) -> ApiEndpoint {
    ApiEndpoint {
        base_url: format!("http://{addr}"),
        token: token.map(|t| ApiToken::new(t).expect("token")),
        mode,
        timeout: DEFAULT_PATTERN_TIMEOUT,
    }
}

fn grade(ep: &ApiEndpoint, source: &str) -> ApiOutcome {
    classify(post_compile_blocking(ep, source, MISSION))
}

#[test]
fn authed_pass_is_graded_and_saved() {
    let addr = start_server(AppState::new(authed_config()));
    let ep = endpoint(addr, Some(TOKEN), CompileMode::Pattern);
    let o = grade(&ep, PASSING);
    assert!(matches!(o, ApiOutcome::Graded { ok: true, .. }), "{o:?}");
    let v = verdict_with_server(&o, MISSION, || panic!("no fallback on success"));
    assert!(v.ok && v.persist_clear);
    assert!(v.formatted.starts_with("[ok]"), "{}", v.formatted);

    let o = grade(&ep, "fn main() {}");
    assert!(matches!(o, ApiOutcome::Graded { ok: false, .. }), "{o:?}");
}

#[test]
fn missing_or_wrong_token_is_unauthorized() {
    let addr = start_server(AppState::new(authed_config()));
    assert_eq!(
        grade(&endpoint(addr, None, CompileMode::Pattern), PASSING),
        ApiOutcome::Unauthorized
    );
    assert_eq!(
        grade(
            &endpoint(addr, Some("wrong-token"), CompileMode::Pattern),
            PASSING
        ),
        ApiOutcome::Unauthorized
    );
}

#[test]
fn oversize_source_is_too_large() {
    let addr = start_server(AppState::new(authed_config()));
    let big = format!("fn main() {{}}\n// {}", "x".repeat(80 * 1024));
    assert_eq!(
        grade(&endpoint(addr, Some(TOKEN), CompileMode::Pattern), &big),
        ApiOutcome::TooLarge
    );
}

#[test]
fn rate_limit_reports_retry_after() {
    let cfg = Config {
        rate_limit_per_min: 2,
        ..authed_config()
    };
    let addr = start_server(AppState::new(cfg));
    let ep = endpoint(addr, Some(TOKEN), CompileMode::Pattern);
    let mut last = None;
    for _ in 0..4 {
        last = Some(grade(&ep, PASSING));
    }
    match last {
        Some(ApiOutcome::RateLimited {
            retry_after_secs: Some(s),
        }) => assert!(s >= 1, "retry_after {s}"),
        other => panic!("expected RateLimited with Retry-After, got {other:?}"),
    }
}

#[test]
fn full_build_queue_is_busy() {
    // One build slot, no queue, and a grader that holds the slot.
    let cfg = Config {
        max_concurrent_builds: 1,
        max_queued_builds: 0,
        ..authed_config()
    };
    let slow: pledge_compile_api::RealGrader = Arc::new(|_src, _limits| {
        std::thread::sleep(Duration::from_secs(3));
        Ok(RealOutcome {
            compiled: true,
            ran_ok: true,
            stdout: String::new(),
            stderr: String::new(),
            build_time: Duration::from_millis(1),
            run_time: Some(Duration::from_millis(1)),
        })
    });
    let addr = start_server(AppState::with_real_grader(cfg, slow));
    let ep = endpoint(addr, Some(TOKEN), CompileMode::Real);
    let holder = {
        let ep = ep.clone();
        std::thread::spawn(move || grade(&ep, PASSING))
    };
    std::thread::sleep(Duration::from_millis(700));
    let o = grade(&ep, PASSING);
    assert!(
        matches!(
            o,
            ApiOutcome::Busy {
                retry_after_secs: Some(_)
            }
        ),
        "{o:?}"
    );
    let first = holder.join().expect("holder thread");
    assert!(
        matches!(first, ApiOutcome::Graded { ok: true, .. }),
        "{first:?}"
    );
}

#[test]
fn unreachable_server_falls_back_to_unsaved_stub() {
    // Bind then drop a listener to get a port nothing is serving.
    let addr = std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .expect("free port");
    let ep = endpoint(addr, Some(TOKEN), CompileMode::Pattern);
    let o = grade(&ep, PASSING);
    assert!(matches!(o, ApiOutcome::Offline { .. }), "{o:?}");
    let v = verdict_with_server(&o, MISSION, || stub_verdict(MISSION, PASSING));
    assert!(v.ok, "stub feedback still shows the pass");
    assert!(!v.persist_clear, "fallback pass must not be saved");
    assert!(v.formatted.starts_with("[offline mode"), "{}", v.formatted);
}
