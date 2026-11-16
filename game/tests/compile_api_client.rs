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
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pledge_and_crown::plugins::compile_client::config::{
    ApiEndpoint, ApiToken, CompileMode, DEFAULT_PATTERN_TIMEOUT,
};
use pledge_and_crown::plugins::compile_client::outcome::{
    ApiOutcome, Grade, Unverified, classify, verdict_with_server,
};
use pledge_and_crown::plugins::compile_client::transport::post_compile_blocking;
use pledge_and_crown::plugins::stub_grader::stub_verdict;
use pledge_compile_api::config::Secret;
use pledge_compile_api::expectations::{GradeState, GradedRun, HarnessMode, harness_pass_note};
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
    classify(ep.mode, post_compile_blocking(ep, source, MISSION))
}

#[test]
fn authed_pass_is_graded_and_saved() {
    let addr = start_server(AppState::new(authed_config()));
    let ep = endpoint(addr, Some(TOKEN), CompileMode::Pattern);
    let o = grade(&ep, PASSING);
    // The pattern route never checks behaviour: a pass, never "verified".
    assert_eq!(
        o.grade(),
        Some(Grade::PassedUnverified(Unverified::PatternOnly)),
        "{o:?}"
    );
    let v = verdict_with_server(&o, MISSION, || panic!("no fallback on success"));
    assert!(v.ok && v.persist_clear);
    assert!(v.formatted.starts_with("[ok:pattern]"), "{}", v.formatted);

    let o = grade(&ep, "fn main() {}");
    assert_eq!(o.grade(), Some(Grade::Failed), "{o:?}");
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
    // One build slot, no queue, and a grader that holds the slot until the
    // test releases it. Synchronised on channels, not sleeps: the second
    // request is only sent once the first is provably inside the grader
    // (so holding the slot), however loaded the machine is.
    let cfg = Config {
        max_concurrent_builds: 1,
        max_queued_builds: 0,
        ..authed_config()
    };
    let (entered_tx, entered_rx) = std::sync::mpsc::channel::<()>();
    let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
    let entered_tx = Mutex::new(entered_tx);
    let release_rx = Mutex::new(release_rx);
    let holding: pledge_compile_api::RealGrader = Arc::new(move |_src, _limits| {
        entered_tx
            .lock()
            .expect("entered lock")
            .send(())
            .expect("signal entered");
        release_rx
            .lock()
            .expect("release lock")
            .recv_timeout(Duration::from_secs(60))
            .expect("test never released the build slot");
        Ok(RealOutcome {
            compiled: true,
            ran_ok: true,
            stdout: String::new(),
            stderr: String::new(),
            build_time: Duration::from_millis(1),
            run_time: Some(Duration::from_millis(1)),
        })
    });
    let addr = start_server(AppState::with_real_grader(cfg, holding));
    let ep = endpoint(addr, Some(TOKEN), CompileMode::Real);
    let holder = {
        let ep = ep.clone();
        std::thread::spawn(move || grade(&ep, PASSING))
    };
    entered_rx
        .recv_timeout(Duration::from_secs(60))
        .expect("first build never reached the grader");
    let o = grade(&ep, PASSING);
    release_tx.send(()).expect("release the build slot");
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
    // The injected grader checks nothing: an honest unverified pass.
    assert_eq!(
        first.grade(),
        Some(Grade::PassedUnverified(Unverified::RanNotChecked)),
        "{first:?}"
    );
}

/// A mission grader that returns `state` for any source.
fn fixed_grade(
    state: GradeState,
    expected: Option<&'static str>,
    actual: Option<&'static str>,
) -> pledge_compile_api::MissionGrader {
    Arc::new(move |_enc, _src, _limits| {
        Ok(GradedRun {
            outcome: RealOutcome {
                compiled: !matches!(state, GradeState::CompileError),
                ran_ok: matches!(
                    state,
                    GradeState::Passed | GradeState::RanNotChecked | GradeState::WrongOutput
                ),
                stdout: "41".to_string(),
                stderr: String::new(),
                build_time: Duration::from_millis(1),
                run_time: None,
            },
            state,
            expected: expected.map(str::to_string),
            actual: actual.map(str::to_string),
            note: format!("note for {state:?}"),
        })
    })
}

#[test]
fn real_route_grades_map_end_to_end() {
    let cases = [
        (GradeState::Passed, Grade::Passed, true),
        (
            GradeState::CompiledNotChecked,
            Grade::PassedUnverified(Unverified::CompiledNotChecked),
            true,
        ),
        (
            GradeState::RanNotChecked,
            Grade::PassedUnverified(Unverified::RanNotChecked),
            true,
        ),
        (GradeState::WrongOutput, Grade::WrongOutput, false),
        (GradeState::CompileError, Grade::CompileError, false),
        (GradeState::RuntimeError, Grade::RuntimeError, false),
        (
            GradeState::StaticCheckFailed,
            Grade::StaticCheckFailed,
            false,
        ),
    ];
    for (state, want, saved) in cases {
        let (expected, actual) = if state == GradeState::WrongOutput {
            (Some("a line reading `42`"), Some("41"))
        } else {
            (None, None)
        };
        let addr = start_server(AppState::with_mission_grader(
            authed_config(),
            fixed_grade(state, expected, actual),
        ));
        let o = grade(&endpoint(addr, Some(TOKEN), CompileMode::Real), PASSING);
        assert_eq!(o.grade(), Some(want), "{state:?}: {o:?}");
        let v = verdict_with_server(&o, MISSION, || panic!("no fallback"));
        assert_eq!(v.persist_clear, saved, "{state:?}");
        assert_eq!(v.detail.expected.as_deref(), expected, "{state:?}");
        assert_eq!(v.detail.actual.as_deref(), actual, "{state:?}");
        assert!(
            v.detail
                .note
                .as_deref()
                .is_some_and(|n| n.contains("note for")),
            "{state:?}: {:?}",
            v.detail.note
        );
    }
}

/// A pass whose server note is the real harness pass note for `mode`.
fn harness_pass(mode: HarnessMode) -> pledge_compile_api::MissionGrader {
    Arc::new(move |_enc, _src, _limits| {
        Ok(GradedRun {
            outcome: RealOutcome {
                compiled: true,
                ran_ok: true,
                stdout: "42".to_string(),
                stderr: String::new(),
                build_time: Duration::from_millis(1),
                run_time: None,
            },
            state: GradeState::Passed,
            expected: None,
            actual: None,
            note: harness_pass_note(mode, 8),
        })
    })
}

/// An Act 9 pass graded with `unsafe` allowed (`UnsafeLateNonce`) reaches
/// the player as a clear labelled NOT VERIFIED, keyed on the server's own
/// note; the same pass from a `ForbidUnsafe` mission is verified.
#[test]
fn unsafe_allowed_harness_pass_is_saved_but_not_verified() {
    for (mode, want) in [
        (
            HarnessMode::UnsafeLateNonce,
            Grade::PassedUnverified(Unverified::UnsafeAllowed),
        ),
        (HarnessMode::ForbidUnsafe, Grade::Passed),
    ] {
        let addr = start_server(AppState::with_mission_grader(
            authed_config(),
            harness_pass(mode),
        ));
        let o = grade(&endpoint(addr, Some(TOKEN), CompileMode::Real), PASSING);
        assert_eq!(o.grade(), Some(want), "{mode:?}: {o:?}");
        let v = verdict_with_server(&o, MISSION, || panic!("no fallback"));
        assert!(v.ok && v.persist_clear, "{mode:?}: a pass is a clear");
        assert_eq!(
            v.detail.note.as_deref(),
            Some(harness_pass_note(mode, 8).as_str())
        );
    }
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
