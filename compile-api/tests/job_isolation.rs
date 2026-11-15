// SPDX-License-Identifier: AGPL-3.0-or-later
//! Security re-review P1-C: timed-out work must never outlive its
//! accounting.
//!
//! Each job runs in its own `pledge_compile_api job` process (the real
//! binary, `CARGO_BIN_EXE_pledge_compile_api`), killed with its whole
//! process tree at the job deadline. These tests fire several jobs that
//! cannot finish in time, through the HTTP route (API build semaphore)
//! and through the builder's connection handler (builder job slots), and
//! then assert:
//!
//! - every response is the deadline verdict;
//! - every job process is dead, and no process whose command line names
//!   the test's unique build root (cargo, rustc, linker) is left;
//! - no helper thread is left (`helper_threads::live() == 0`);
//! - the semaphore / job slots are back to full.
//!
//! The tests in this file share process-wide counters, so they run one at
//! a time (`SERIAL`).

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use pledge_compile_api::builder::{BuilderCtx, ConnOutcome, JobSlots, handle_connection};
use pledge_compile_api::ipc::{self, JobReply, JobRequest};
use pledge_compile_api::jobs::{self, JobEnvelope};
use pledge_compile_api::proc_tree::pid_alive;
use pledge_compile_api::wasm_builder::BuildSettings;
use pledge_compile_api::{
    AppState, CompileResponse, Config, MissionGrader, build_router, helper_threads, serve,
};

const EXE: &str = env!("CARGO_BIN_EXE_pledge_compile_api");

static SERIAL: Mutex<()> = Mutex::new(());

/// Exponential macro expansion: rustc spins for a long time on this, so a
/// job that reaches rustc cannot finish before a short deadline. Uses no
/// deny-listed construct.
const SLOW_SOURCE: &str = "macro_rules! r { () => {}; ($h:tt $($t:tt)*) => { r!($($t)*); \
     r!($($t)*); }; }\nr!(a a a a a a a a a a a a a a a a a a a a a a a a a a a a);\nfn main() {}";

fn unique_root() -> (PathBuf, String) {
    let tag = format!("pnc-jobiso-{}", uuid_like());
    (std::env::temp_dir().join(&tag), tag)
}

fn uuid_like() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("{:x}{:x}", std::process::id(), nanos)
}

fn settings(root: &Path) -> BuildSettings {
    BuildSettings {
        root: root.to_path_buf(),
        ..BuildSettings::default()
    }
}

fn request() -> JobRequest {
    JobRequest {
        encounter_id: "intro_let_binding".to_string(),
        source: SLOW_SOURCE.to_string(),
        run_max_memory_bytes: 16 << 20,
    }
}

/// Number of live processes (other than this one and the query itself)
/// whose command line contains `needle`.
fn processes_mentioning(needle: &str) -> usize {
    #[cfg(windows)]
    {
        let script = format!(
            "@(Get-CimInstance Win32_Process | Where-Object {{ $_.CommandLine -like '*{needle}*' \
             -and $_.ProcessId -ne $PID }}).Count"
        );
        let out = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .expect("powershell");
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("bad count: {:?}", String::from_utf8_lossy(&out.stdout)))
    }
    #[cfg(unix)]
    {
        let me = std::process::id().to_string();
        std::fs::read_dir("/proc")
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| {
                        let name = e.file_name();
                        let name = name.to_string_lossy();
                        name.chars().all(|c| c.is_ascii_digit()) && name != me
                    })
                    .filter(|e| {
                        std::fs::read(e.path().join("cmdline"))
                            .map(|c| String::from_utf8_lossy(&c).contains(needle))
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0)
    }
}

/// Hang guard for "eventually" checks. These tests failed twice on an
/// overloaded Windows box with the old 5 s / immediate checks: a slow host
/// delays thread exit and process teardown, it does not leave them alive.
/// The assertions are unchanged (zero helpers, zero processes, dead PIDs);
/// only how long we wait for the OS to finish what was already ordered.
const SETTLE: Duration = Duration::from_secs(30);

fn wait_for_helpers_to_exit() -> usize {
    let started = Instant::now();
    while helper_threads::live() > 0 && started.elapsed() < SETTLE {
        std::thread::sleep(Duration::from_millis(20));
    }
    helper_threads::live()
}

/// Build processes (cargo, rustc, linker) still alive for this test.
///
/// Linux: `proc_tree` only returns once every process group it killed is
/// confirmed empty, so this must be zero at once; no waiting.
/// Windows (dev only): `taskkill /T /F` is not confirmed (documented gap
/// in `proc_tree`) and process teardown after TerminateProcess is
/// asynchronous, so a process can still be listed for a moment after the
/// job returned. Poll until it is gone, bounded by [`SETTLE`].
fn surviving_build_processes(tag: &str) -> usize {
    if cfg!(unix) {
        return processes_mentioning(tag);
    }
    let started = Instant::now();
    loop {
        let n = processes_mentioning(tag);
        if n == 0 || started.elapsed() >= SETTLE {
            return n;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

/// Same rule as [`surviving_build_processes`] for one job process.
fn job_process_alive(pid: u32) -> bool {
    if cfg!(unix) {
        return pid_alive(pid);
    }
    let started = Instant::now();
    while pid_alive(pid) && started.elapsed() < SETTLE {
        std::thread::sleep(Duration::from_millis(250));
    }
    pid_alive(pid)
}

fn assert_deadline_verdict(run: &JobReply) {
    match run {
        JobReply::Graded(g) => {
            assert!(!g.compiled && !g.ran_ok, "{g:?}");
            assert!(g.stderr.contains("job budget"), "{g:?}");
        }
        other => panic!("expected the deadline verdict, got {other:?}"),
    }
}

/// HTTP path: N concurrent `/compile-real` requests, 2 build slots, each
/// job killed at a short deadline.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
// The guard only serialises this file's tests; nothing else takes it, and
// the test future runs on the test thread (no Send needed).
#[allow(clippy::await_holding_lock)]
async fn timed_out_jobs_leave_no_process_thread_or_slot_behind() {
    let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    const N: usize = 4;
    const DEADLINE: Duration = Duration::from_millis(400);
    let (root, tag) = unique_root();
    let s = settings(&root);
    let pids = Arc::new(Mutex::new(Vec::new()));
    let pids_c = Arc::clone(&pids);
    let grader: MissionGrader = Arc::new(move |enc, src, limits| {
        let env = JobEnvelope {
            settings: s.clone(),
            request: JobRequest {
                encounter_id: enc.to_string(),
                source: src.to_string(),
                run_max_memory_bytes: limits.max_memory_bytes,
            },
        };
        let run = jobs::run_job_subprocess(Path::new(EXE), &env, DEADLINE)?;
        pids_c.lock().unwrap().push(run.pid);
        assert!(
            run.timed_out,
            "job finished before its deadline: {:?}",
            run.reply
        );
        jobs::reply_to_graded(run.reply)
    });
    let cfg = Config {
        max_concurrent_builds: 2,
        max_queued_builds: 8,
        ..Config::default()
    };
    let state = AppState::with_mission_grader(cfg, grader);
    let probe = state.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { serve(listener, build_router(state)).await });

    let client = reqwest::Client::new();
    let mut tasks = Vec::new();
    for _ in 0..N {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            client
                .post(format!("http://{addr}/compile-real"))
                .json(&serde_json::json!({
                    "source": SLOW_SOURCE,
                    "encounter_id": "intro_let_binding"
                }))
                .send()
                .await
                .unwrap()
        }));
    }
    for t in tasks {
        let resp = t.await.unwrap();
        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        let body: CompileResponse = resp.json().await.unwrap();
        assert!(!body.ok, "{body:?}");
        assert!(body.stderr.contains("job budget"), "{body:?}");
    }

    let pids = pids.lock().unwrap().clone();
    assert_eq!(pids.len(), N);
    for pid in &pids {
        assert!(!job_process_alive(*pid), "job process {pid} survived");
    }
    assert_eq!(
        surviving_build_processes(&tag),
        0,
        "build processes survived"
    );
    assert_eq!(wait_for_helpers_to_exit(), 0, "helper threads survived");
    assert_eq!(probe.available_build_slots(), 2, "semaphore back to full");
    let _ = std::fs::remove_dir_all(&root);
}

/// Builder path: the same over the IPC protocol (loopback TCP stands in
/// for the Unix socket on this platform), with builder job slots.
#[test]
fn builder_slots_return_to_full_after_timed_out_jobs() {
    let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    const N: usize = 3;
    const DEADLINE: Duration = Duration::from_millis(400);
    let (root, tag) = unique_root();
    let pids = Arc::new(Mutex::new(Vec::new()));
    let pids_c = Arc::clone(&pids);
    let ctx = BuilderCtx {
        settings: settings(&root),
        slots: JobSlots::new(N),
        runner: Arc::new(move |env| {
            let run = jobs::run_job_subprocess(Path::new(EXE), env, DEADLINE)?;
            pids_c.lock().unwrap().push(run.pid);
            Ok(run)
        }),
        on_survivors: Arc::new(|| panic!("no survivors expected")),
        request_deadline: ipc::REQUEST_IO_DEADLINE,
        reply_deadline: ipc::REPLY_WRITE_DEADLINE,
    };
    let slots = Arc::clone(&ctx.slots);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let mut outcomes = Vec::new();
        let mut handles = Vec::new();
        for _ in 0..N {
            let (mut stream, _) = listener.accept().unwrap();
            let ctx = ctx.clone();
            handles.push(std::thread::spawn(move || {
                handle_connection(&mut stream, &ctx)
            }));
        }
        for h in handles {
            outcomes.push(h.join().unwrap());
        }
        outcomes
    });
    let clients: Vec<_> = (0..N)
        .map(|_| {
            std::thread::spawn(move || {
                let mut s = TcpStream::connect(addr).unwrap();
                ipc::exchange(&mut s, &request(), Duration::from_secs(30)).unwrap()
            })
        })
        .collect();
    for c in clients {
        assert_deadline_verdict(&c.join().unwrap());
    }
    for outcome in server.join().unwrap() {
        assert_eq!(outcome, ConnOutcome::Replied("graded"));
    }
    for pid in pids.lock().unwrap().iter() {
        assert!(!job_process_alive(*pid), "job process {pid} survived");
    }
    assert_eq!(
        surviving_build_processes(&tag),
        0,
        "build processes survived"
    );
    assert_eq!(wait_for_helpers_to_exit(), 0, "helper threads survived");
    assert_eq!(slots.available(), N, "job slots back to full");
    let _ = std::fs::remove_dir_all(&root);
}

/// Needs the wasm32-wasip1 toolchain: lets the job actually reach rustc
/// (observed running), then kills it at the deadline.
#[test]
#[ignore = "slow: needs cargo + wasm32-wasip1; lets rustc start before the kill"]
fn runaway_rustc_is_killed_with_its_job() {
    let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let (root, tag) = unique_root();
    let env = JobEnvelope {
        settings: settings(&root),
        request: request(),
    };
    let deadline = Duration::from_secs(8);
    let watcher_tag = tag.clone();
    let watcher = std::thread::spawn(move || {
        // Sample while the job runs: proves rustc/cargo were alive.
        let started = Instant::now();
        let mut peak = 0;
        while started.elapsed() < Duration::from_secs(7) {
            peak = peak.max(processes_mentioning(&watcher_tag));
            std::thread::sleep(Duration::from_millis(500));
        }
        peak
    });
    let run = jobs::run_job_subprocess(Path::new(EXE), &env, deadline).unwrap();
    assert!(run.timed_out);
    assert_deadline_verdict(&run.reply);
    let peak = watcher.join().unwrap();
    assert!(peak > 0, "expected build processes while the job ran");
    assert!(!job_process_alive(run.pid));
    assert_eq!(surviving_build_processes(&tag), 0, "rustc survived its job");
    assert_eq!(wait_for_helpers_to_exit(), 0);
    let _ = std::fs::remove_dir_all(&root);
}
