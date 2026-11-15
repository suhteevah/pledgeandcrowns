// SPDX-License-Identifier: AGPL-3.0-or-later
//! The run timeout through the REAL build path (Linux verification
//! 2026-11-14, open item 1): a compiled Rust program that sleeps, spins,
//! reads stdin or waits on a clock that never comes must be stopped at the
//! 5 s run timeout (plus `RUN_TIMEOUT_MARGIN`), graded `runtime_error`
//! with a plain message, and give its job slot back, whatever it does.
//!
//! The wat-level cases (no toolchain needed) are in `tests/wasm_runner.rs`.
//! These need cargo + `wasm32-wasip1` and are `#[ignore]`d:
//!
//! ```text
//! cargo test -p pledge_compile_api --test run_timeout -- --ignored
//! ```

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use pledge_compile_api::builder::{BuilderCtx, ConnOutcome, JobSlots, handle_connection};
use pledge_compile_api::expectations::{self, GradeState};
use pledge_compile_api::ipc::{self, JobReply, JobRequest};
use pledge_compile_api::jobs::{self, JOB_DEADLINE, JobEnvelope};
use pledge_compile_api::wasm_builder::{BuildSettings, RUN_TIMEOUT_MS, RunLimits};
use pledge_compile_api::wasm_runner::{RUN_TIMEOUT_MARGIN, TIMEOUT_PREFIX};

const EXE: &str = env!("CARGO_BIN_EXE_pledge_compile_api");

/// Not a curriculum mission: graded "built + ran", no output check.
const PROBE_ID: &str = "run_timeout_probe";

/// Scheduling slack on a box running a 24-job build beside the tests. The
/// bug these tests guard against is a run lasting 3600 s (or the 50 s job
/// deadline), not one that is a second late.
const LOADED_HOST_SLACK: Duration = Duration::from_secs(3);

fn run_bound() -> Duration {
    Duration::from_millis(RUN_TIMEOUT_MS) + RUN_TIMEOUT_MARGIN + LOADED_HOST_SLACK
}

const SLEEP_FOREVER: &str = "fn main() { println!(\"going to sleep\"); \
     std::thread::sleep(std::time::Duration::from_secs(3600)); println!(\"woke\"); }";
/// `Duration::MAX` sleep: std loops on u64-nanosecond clock waits, so the
/// subscription is never ready within any run.
const NEVER_READY: &str = "fn main() { std::thread::sleep(std::time::Duration::MAX); }";
const BUSY_LOOP: &str = "fn main() { let mut x: u64 = 0; \
     loop { x = x.wrapping_add(1); std::hint::black_box(x); } }";
const STDIN_READ: &str = "fn main() { let mut s = String::new(); \
     let n = std::io::stdin().read_line(&mut s).unwrap(); println!(\"read {n} bytes\"); }";

fn grade(src: &str) -> expectations::GradedRun {
    expectations::grade(PROBE_ID, src, &RunLimits::default()).expect("grading ran")
}

fn assert_runtime_error_within_bound(what: &str, g: &expectations::GradedRun) {
    assert!(
        g.outcome.compiled,
        "{what}: must compile: {}",
        g.outcome.stderr
    );
    assert!(!g.outcome.ran_ok, "{what}: must not run cleanly");
    assert_eq!(g.state, GradeState::RuntimeError, "{what}: {g:?}");
    let run = g.outcome.run_time.expect("a run happened");
    assert!(run < run_bound(), "{what}: run took {run:?}");
}

#[test]
#[ignore = "slow: needs cargo + wasm32-wasip1"]
fn sleeping_program_is_a_runtime_error_at_the_run_timeout() {
    let g = grade(SLEEP_FOREVER);
    assert_runtime_error_within_bound("sleep 3600 s", &g);
    assert!(
        g.outcome.stderr.starts_with(TIMEOUT_PREFIX),
        "{}",
        g.outcome.stderr
    );
    assert!(
        g.outcome.stderr.contains("time limit"),
        "{}",
        g.outcome.stderr
    );
    let run = g.outcome.run_time.unwrap();
    assert!(
        run >= Duration::from_millis(RUN_TIMEOUT_MS - 100),
        "stopped too early: {run:?}"
    );
    // Output printed before the sleep is kept.
    assert!(g.outcome.stdout.contains("going to sleep"), "{g:?}");
}

#[test]
#[ignore = "slow: needs cargo + wasm32-wasip1"]
fn never_ready_clock_wait_is_a_runtime_error_at_the_run_timeout() {
    let g = grade(NEVER_READY);
    assert_runtime_error_within_bound("Duration::MAX sleep", &g);
    assert!(g.outcome.stderr.starts_with(TIMEOUT_PREFIX), "{g:?}");
}

#[test]
#[ignore = "slow: needs cargo + wasm32-wasip1"]
fn busy_loop_is_a_runtime_error_within_the_run_timeout() {
    // Fuel (2e9) or the epoch timeout, whichever comes first: both are a
    // runtime error inside the bound.
    let g = grade(BUSY_LOOP);
    assert_runtime_error_within_bound("busy loop", &g);
    let e = &g.outcome.stderr;
    assert!(e.contains("fuel") || e.starts_with(TIMEOUT_PREFIX), "{e}");
}

#[test]
#[ignore = "slow: needs cargo + wasm32-wasip1"]
fn stdin_read_sees_end_of_file_and_finishes() {
    let started = Instant::now();
    let g = grade(STDIN_READ);
    assert!(g.outcome.ok(), "{g:?}");
    assert_eq!(g.state, GradeState::RanNotChecked);
    assert!(g.outcome.stdout.contains("read 0 bytes"), "{g:?}");
    assert!(
        g.outcome.run_time.unwrap() < Duration::from_millis(RUN_TIMEOUT_MS),
        "{g:?}"
    );
    println!("stdin probe took {:?}", started.elapsed());
}

fn probe_request(src: &str) -> JobRequest {
    JobRequest {
        encounter_id: PROBE_ID.to_string(),
        source: src.to_string(),
        run_max_memory_bytes: 16 << 20,
    }
}

/// Through the real job process (the builder's production path): the job
/// must answer on its own (not be killed at the 50 s deadline), with a
/// runtime error and a run time inside the bound.
#[test]
#[ignore = "slow: needs cargo + wasm32-wasip1"]
fn job_process_answers_runtime_error_long_before_its_deadline() {
    let root = std::env::temp_dir().join(format!("pnc-runtimeout-{}", std::process::id()));
    for (what, src) in [
        ("sleep", SLEEP_FOREVER),
        ("never ready", NEVER_READY),
        ("busy loop", BUSY_LOOP),
    ] {
        let env = JobEnvelope {
            settings: BuildSettings {
                root: root.clone(),
                ..BuildSettings::default()
            },
            request: probe_request(src),
        };
        let run = jobs::run_job_subprocess(Path::new(EXE), &env, JOB_DEADLINE).unwrap();
        assert!(
            !run.timed_out,
            "{what}: the job hit its {JOB_DEADLINE:?} deadline (the run timeout did not hold)"
        );
        let JobReply::Graded(g) = run.reply else {
            panic!("{what}: expected a graded reply, got {:?}", run.reply);
        };
        assert!(g.compiled && !g.ran_ok, "{what}: {g:?}");
        assert_eq!(g.state, GradeState::RuntimeError, "{what}: {g:?}");
        let run_ms = Duration::from_millis(g.run_ms.expect("run time"));
        assert!(run_ms < run_bound(), "{what}: run took {run_ms:?}");
        println!(
            "{what}: job elapsed {:?}, run {run_ms:?}, stderr {:?}",
            run.elapsed,
            g.stderr.lines().next()
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// Through the builder's connection handler: the slot is held while the
/// sleeping program runs and is back once the reply is sent.
#[test]
#[ignore = "slow: needs cargo + wasm32-wasip1"]
fn builder_slot_is_freed_after_a_run_timeout() {
    let root = std::env::temp_dir().join(format!("pnc-runtimeout-b-{}", std::process::id()));
    let ctx = BuilderCtx {
        settings: BuildSettings {
            root: root.clone(),
            ..BuildSettings::default()
        },
        slots: JobSlots::new(2),
        runner: Arc::new(|env| jobs::run_job_subprocess(Path::new(EXE), env, JOB_DEADLINE)),
        on_survivors: Arc::new(|| panic!("no survivors expected")),
        request_deadline: ipc::REQUEST_IO_DEADLINE,
        reply_deadline: ipc::REPLY_WRITE_DEADLINE,
    };
    let slots = Arc::clone(&ctx.slots);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        handle_connection(&mut stream, &ctx)
    });
    let started = Instant::now();
    let mut s = TcpStream::connect(addr).unwrap();
    let reply = ipc::exchange(
        &mut s,
        &probe_request(SLEEP_FOREVER),
        Duration::from_secs(60),
    )
    .expect("a reply");
    let took = started.elapsed();
    assert_eq!(server.join().unwrap(), ConnOutcome::Replied("graded"));
    let JobReply::Graded(g) = reply else {
        panic!("expected graded, got {reply:?}");
    };
    assert_eq!(g.state, GradeState::RuntimeError, "{g:?}");
    assert!(g.stderr.starts_with(TIMEOUT_PREFIX), "{g:?}");
    assert!(
        took < JOB_DEADLINE,
        "the reply came at the job deadline ({took:?})"
    );
    assert_eq!(slots.available(), 2, "slot back after the reply");
    let _ = std::fs::remove_dir_all(&root);
}
