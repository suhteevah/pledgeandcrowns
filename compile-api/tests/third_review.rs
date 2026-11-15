// SPDX-License-Identifier: AGPL-3.0-or-later
//! Third security review (2026-11-15) fixes:
//!
//! - S-2: a dead or wedged builder costs the API seconds, not a slot for a
//!   minute (bounded connect, ack deadline, 503 `builder_unavailable`).
//! - S-3: a setup failure answers 500 with a request id and no internal
//!   detail (paths, sockets, binaries) for every failure class.
//! - S-1: the `run-wasm` child protocol through the real binary (all
//!   platforms, no sandbox) and, on Linux with bubblewrap, the run sandbox
//!   itself (`#[ignore]`: needs bwrap; run on the deploy host).

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use pledge_compile_api::builder::{
    BuilderCtx, ConnOutcome, JobRunner, JobSlots, handle_connection,
};
use pledge_compile_api::expectations::GradeState;
use pledge_compile_api::ipc::{self, IpcError, JobReply, JobRequest, WireGradedRun};
use pledge_compile_api::jobs::{ExecError, JobRun};
use pledge_compile_api::proc_tree::{self, ProcError};
use pledge_compile_api::run_sandbox::{self, RunChildReply, RunSandboxError};
use pledge_compile_api::wasm_builder::{BuildSettings, RunLimits};
use pledge_compile_api::{AppState, CompileResponse, Config, MissionGrader, build_router, serve};

const EXE: &str = env!("CARGO_BIN_EXE_pledge_compile_api");

fn request() -> JobRequest {
    JobRequest {
        encounter_id: "intro_let_binding".to_string(),
        source: "fn main() { println!(\"42\"); }".to_string(),
        run_max_memory_bytes: 16 << 20,
    }
}

// ---- S-2: the ack ----------------------------------------------------------------

/// A fake builder that accepts, reads until the client goes away, never
/// writes.
fn silent_builder() -> (SocketAddr, std::thread::JoinHandle<()>) {
    use std::io::Read;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let h = std::thread::spawn(move || {
        if let Ok((mut s, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            while matches!(s.read(&mut buf), Ok(n) if n > 0) {}
        }
    });
    (addr, h)
}

#[test]
fn builder_that_accepts_and_never_answers_is_unavailable_at_the_ack_deadline() {
    let (addr, h) = silent_builder();
    let mut s = TcpStream::connect(addr).unwrap();
    let started = Instant::now();
    let err = ipc::exchange_with(
        &mut s,
        &request(),
        Duration::from_millis(500),
        Duration::from_secs(120),
    )
    .unwrap_err();
    assert!(matches!(err, IpcError::Timeout("ack read")), "{err:?}");
    assert!(err.is_unavailable());
    // Bounded by the ack deadline, not the 65 s reply deadline.
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert!(ipc::ACK_DEADLINE * 4 < ipc::REPLY_DEADLINE);
    drop(s);
    let _ = h.join();
}

#[test]
fn builder_that_closes_before_acknowledging_is_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let h = std::thread::spawn(move || {
        let (s, _) = listener.accept().unwrap();
        drop(s);
    });
    let mut s = TcpStream::connect(addr).unwrap();
    let err = ipc::exchange(&mut s, &request(), Duration::from_secs(20)).unwrap_err();
    assert!(err.is_unavailable(), "{err:?}");
    let _ = h.join();
}

#[test]
fn a_slow_job_after_the_ack_is_not_cut_off_by_the_ack_deadline() {
    let runner: JobRunner = Arc::new(|_| {
        std::thread::sleep(Duration::from_millis(1500));
        Ok(JobRun {
            reply: JobReply::Graded(WireGradedRun {
                compiled: true,
                ran_ok: true,
                stdout: "42\n".to_string(),
                stderr: String::new(),
                build_ms: 1,
                run_ms: Some(1),
                state: GradeState::RanNotChecked,
                expected: None,
                actual: None,
                note: "stub".to_string(),
            }),
            pid: 0,
            timed_out: false,
            elapsed: Duration::from_millis(1500),
        })
    });
    let ctx = BuilderCtx {
        settings: BuildSettings::default(),
        slots: JobSlots::new(1),
        runner,
        on_survivors: Arc::new(|| {}),
        request_deadline: Duration::from_secs(20),
        reply_deadline: Duration::from_secs(20),
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let h = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        handle_connection(&mut s, &ctx)
    });
    let mut s = TcpStream::connect(addr).unwrap();
    let reply = ipc::exchange_with(
        &mut s,
        &request(),
        Duration::from_millis(500),
        Duration::from_secs(20),
    )
    .expect("the ack arrives at once; the reply after the job");
    assert!(matches!(reply, JobReply::Graded(_)), "{reply:?}");
    assert_eq!(h.join().unwrap(), ConnOutcome::Replied("graded"));
}

// ---- S-2: Unix socket connect --------------------------------------------------------

#[cfg(unix)]
mod unix_socket {
    use super::*;
    use std::path::PathBuf;

    fn temp_socket(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pnc-s3-{tag}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("b.sock")
    }

    /// A socket file nobody listens on (what a stopped builder leaves).
    /// Bound but never `listen()`ed: a dropped `UnixListener` is not good
    /// enough in a multi-threaded test binary, because a child forked by a
    /// concurrent test can hold the listening fd until it execs, and the
    /// connect then succeeds (seen once on Linux).
    fn stale_socket(path: &std::path::Path) {
        use rustix::net::{AddressFamily, SocketAddrUnix, SocketFlags, SocketType};
        let fd = rustix::net::socket_with(
            AddressFamily::UNIX,
            SocketType::STREAM,
            SocketFlags::CLOEXEC,
            None,
        )
        .unwrap();
        rustix::net::bind(&fd, &SocketAddrUnix::new(path).unwrap()).unwrap();
        drop(fd);
    }

    #[test]
    fn socket_file_without_a_listener_is_unavailable_at_once() {
        let path = temp_socket("dead");
        stale_socket(&path);
        assert!(path.exists(), "the stale socket file stays behind");
        let started = Instant::now();
        let err = ipc::connect_builder(&path, Duration::from_secs(2)).unwrap_err();
        assert!(err.is_unavailable(), "{err:?}");
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "{:?}",
            started.elapsed()
        );
        let err = ipc::call_builder(&path, &request()).unwrap_err();
        assert!(err.is_unavailable(), "{err:?}");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn full_backlog_times_out_instead_of_blocking() {
        use rustix::net::{AddressFamily, SocketAddrUnix, SocketFlags, SocketType};
        let path = temp_socket("backlog");
        let fd = rustix::net::socket_with(
            AddressFamily::UNIX,
            SocketType::STREAM,
            SocketFlags::CLOEXEC,
            None,
        )
        .unwrap();
        rustix::net::bind(&fd, &SocketAddrUnix::new(&path).unwrap()).unwrap();
        rustix::net::listen(&fd, 0).unwrap();
        // Never accepted: fill the backlog until a connect would block.
        let mut held = Vec::new();
        let mut timed_out = None;
        for _ in 0..64 {
            let started = Instant::now();
            match ipc::connect_builder(&path, Duration::from_millis(300)) {
                Ok(s) => held.push(s),
                Err(e) => {
                    timed_out = Some((e, started.elapsed()));
                    break;
                }
            }
        }
        let (err, took) = timed_out.expect("a full backlog must make connect time out");
        assert!(matches!(err, IpcError::Timeout("connect")), "{err:?}");
        assert!(err.is_unavailable());
        assert!(
            took >= Duration::from_millis(250) && took < Duration::from_secs(5),
            "{took:?}"
        );
        drop(held);
        drop(fd);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    async fn post_real(
        addr: SocketAddr,
    ) -> (reqwest::StatusCode, Option<String>, serde_json::Value) {
        let resp = reqwest::Client::new()
            .post(format!("http://{addr}/compile-real"))
            .json(
                &serde_json::json!({"source": "fn main(){}", "encounter_id": "intro_let_binding"}),
            )
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let retry = resp
            .headers()
            .get("retry-after")
            .map(|v| v.to_str().unwrap_or_default().to_string());
        (status, retry, resp.json().await.unwrap())
    }

    async fn api_with_builder_socket(socket: PathBuf) -> (SocketAddr, AppState) {
        let cfg = Config {
            executor: pledge_compile_api::Executor::Builder { socket },
            ..Config::default()
        };
        let state = AppState::new(cfg);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = build_router(state.clone());
        tokio::spawn(async move { serve(listener, app).await });
        (addr, state)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn api_answers_503_quickly_for_a_stopped_builder_and_frees_the_slot() {
        let path = temp_socket("api-dead");
        stale_socket(&path);
        let (addr, state) = api_with_builder_socket(path.clone()).await;
        let started = Instant::now();
        let (status, retry, body) = post_real(addr).await;
        assert_eq!(status, reqwest::StatusCode::SERVICE_UNAVAILABLE, "{body}");
        assert_eq!(body["error"], "builder_unavailable", "{body}");
        assert!(retry.is_some(), "Retry-After header");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{:?}",
            started.elapsed()
        );
        assert!(!body.to_string().contains(".sock"), "{body}");
        assert_eq!(
            state.available_build_slots(),
            state.config().max_concurrent_builds
        );
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn api_answers_503_for_a_builder_that_accepts_and_never_replies() {
        use std::io::Read;
        let path = temp_socket("api-silent");
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        let holder = std::thread::spawn(move || {
            if let Ok((mut s, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                while matches!(s.read(&mut buf), Ok(n) if n > 0) {}
            }
        });
        let (addr, state) = api_with_builder_socket(path.clone()).await;
        let started = Instant::now();
        let (status, _, body) = post_real(addr).await;
        let took = started.elapsed();
        assert_eq!(status, reqwest::StatusCode::SERVICE_UNAVAILABLE, "{body}");
        assert_eq!(body["error"], "builder_unavailable", "{body}");
        // The ack deadline (10 s), not the 65 s reply deadline.
        assert!(
            took < ipc::ACK_DEADLINE + Duration::from_secs(5),
            "{took:?}"
        );
        assert_eq!(
            state.available_build_slots(),
            state.config().max_concurrent_builds
        );
        let _ = holder.join();
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

// ---- S-3: generic setup-failure responses -----------------------------------------------

type ErrMaker = fn() -> anyhow::Error;

fn setup_failure_classes() -> Vec<(&'static str, ErrMaker)> {
    vec![
        ("job dir", || {
            anyhow::Error::from(std::io::Error::from(std::io::ErrorKind::PermissionDenied)).context(
                "wasm_builder: failed to create job dir /tmp/pledge-wasm-builder/2b0c7d4e-1111",
            )
        }),
        ("spawn", || {
            anyhow::Error::from(ProcError::Spawn {
                label: "cargo build".to_string(),
                source: std::io::Error::other(
                    "/opt/pnc/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin/cargo: not found",
                ),
            })
            .context("wasm_builder: build process supervision failed")
        }),
        ("builder failed", || {
            ExecError::Failed(
                "job process returned an unusable reply: /usr/local/bin/pledge_compile_api exited"
                    .to_string(),
            )
            .into()
        }),
        ("builder rejected", || {
            ExecError::Rejected(
                "ipc payload is invalid: C:\\Users\\dev\\AppData\\Local\\Temp\\x".to_string(),
            )
            .into()
        }),
        ("ipc", || {
            IpcError::Io {
                op: "reply read",
                source: std::io::Error::other("/run/pnc-builder/builder.sock: connection reset"),
            }
            .into()
        }),
        ("run sandbox", || {
            RunSandboxError::Image("/proc/self/maps: /usr/lib64/libc.so.6 (deleted)".to_string())
                .into()
        }),
        ("survivors", || {
            ProcError::Survivors {
                label: "job".to_string(),
                pgid: 4242,
                waited: Duration::from_secs(5),
            }
            .into()
        }),
    ]
}

#[tokio::test]
async fn setup_failures_reach_the_client_as_a_request_id_without_internal_detail() {
    for (class, make) in setup_failure_classes() {
        let grader: MissionGrader = Arc::new(move |_e, _s, _l: &RunLimits| Err(make()));
        let state = AppState::with_mission_grader(Config::default(), grader);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { serve(listener, build_router(state)).await });
        let resp = reqwest::Client::new()
            .post(format!("http://{addr}/compile-real"))
            .json(
                &serde_json::json!({"source": "fn main(){}", "encounter_id": "intro_let_binding"}),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "{class}"
        );
        let text = resp.text().await.unwrap();
        for leak in ['/', '\\'] {
            assert!(
                !text.contains(leak),
                "{class}: path-like `{leak}` reached the client: {text}"
            );
        }
        for leak in [
            "pnc", "pledge", "builder", "cargo", "sock", "libc", "maps", "4242",
        ] {
            assert!(
                !text.to_lowercase().contains(leak),
                "{class}: `{leak}` reached the client: {text}"
            );
        }
        let body: CompileResponse = serde_json::from_str(&text).unwrap();
        assert!(!body.ok);
        assert_eq!(body.stdout, "[real]");
        assert!(body.stderr.contains("request id"), "{class}: {text}");
        let id = body
            .stderr
            .split("request id ")
            .nth(1)
            .and_then(|r| r.split(')').next())
            .unwrap_or_default();
        assert!(
            id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit()),
            "{class}: request id {id:?}"
        );
    }
}

// ---- S-1: the run-wasm child protocol (real binary, no sandbox) --------------------------

/// Prints "hi\n" through WASI `fd_write`.
const HELLO_WAT: &str = r#"
(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 16) "hi\n")
  (func (export "_start")
    (i32.store (i32.const 0) (i32.const 16))
    (i32.store (i32.const 4) (i32.const 3))
    (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 8)))))
"#;

fn run_child_raw(cmd: Command, input: Vec<u8>) -> RunChildReply {
    let out = proc_tree::run_captured_with_input(
        cmd,
        Some(input),
        Duration::from_secs(60),
        4 << 20,
        "test run child",
    )
    .expect("supervision");
    assert!(!out.timed_out, "{out:?}");
    assert!(out.status.success(), "{out:?}");
    run_sandbox::decode_child_reply(&out.stdout).unwrap_or_else(|e| panic!("{e}: {out:?}"))
}

fn run_child(input: Vec<u8>) -> RunChildReply {
    let mut cmd = Command::new(EXE);
    cmd.arg("run-wasm").env("RUST_LOG", "warn");
    run_child_raw(cmd, input)
}

#[test]
fn run_child_runs_a_module_and_returns_its_verdict() {
    let bytes = wat::parse_str(HELLO_WAT).unwrap();
    let input = run_sandbox::encode_run_input(&bytes, &RunLimits::default()).unwrap();
    match run_child(input) {
        RunChildReply::Verdict { ok, stdout, .. } => {
            assert!(ok);
            assert_eq!(stdout, "hi\n");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn run_child_keeps_fuel_and_timeout_and_refuses_bad_input() {
    // A busy loop still stops on fuel / the epoch inside the child.
    let spin = wat::parse_str(
        r#"(module (memory (export "memory") 1) (func (export "_start") (loop (br 0))))"#,
    )
    .unwrap();
    let limits = RunLimits {
        timeout_ms: 1_000,
        fuel: 50_000_000,
        ..RunLimits::default()
    };
    match run_child(run_sandbox::encode_run_input(&spin, &limits).unwrap()) {
        RunChildReply::Verdict { ok, stderr, .. } => {
            assert!(!ok);
            assert!(
                stderr.contains("fuel exhausted") || stderr.starts_with("timeout:"),
                "{stderr}"
            );
        }
        other => panic!("{other:?}"),
    }
    // Limits looser than shipped: refused in the child.
    let bytes = wat::parse_str(HELLO_WAT).unwrap();
    let mut loose = format!(
        "{{\"timeout_ms\":600000,\"fuel\":1000,\"max_memory_bytes\":1048576,\"wasm_bytes\":{}}}\n",
        bytes.len()
    )
    .into_bytes();
    loose.extend_from_slice(&bytes);
    assert!(matches!(run_child(loose), RunChildReply::SetupError(m) if m.contains("timeout_ms")),);
    // Garbage.
    assert!(matches!(
        run_child(b"\0\0\0".to_vec()),
        RunChildReply::SetupError(_)
    ));
    // A module outside the allow-list is a setup error, as in-process.
    let simd = wat::parse_str(
        r#"(module (memory (export "memory") 1) (func (export "_start") (drop (v128.const i64x2 0 0))))"#,
    )
    .unwrap();
    assert!(matches!(
        run_child(run_sandbox::encode_run_input(&simd, &RunLimits::default()).unwrap()),
        RunChildReply::SetupError(_)
    ));
}

#[test]
fn run_probe_reports_existence_only() {
    let req = run_sandbox::ProbeRequest {
        paths: vec!["/definitely/not/here/pnc-s3".to_string()],
        unix_socket: "/definitely/not/here/pnc-s3.sock".to_string(),
        tcp_target: "127.0.0.1:9".to_string(),
        tcp_control: "127.0.0.1:9".to_string(),
        walk_root: false,
    };
    let mut cmd = Command::new(EXE);
    cmd.arg("run-probe").env("RUST_LOG", "warn");
    let out = proc_tree::run_captured_with_input(
        cmd,
        Some(serde_json::to_vec(&req).unwrap()),
        Duration::from_secs(60),
        1 << 20,
        "test probe",
    )
    .unwrap();
    assert!(out.status.success(), "{out:?}");
    let lines: Vec<run_sandbox::ProbeLine> = out
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let has = |c: &str, s: &str, r: &str| {
        lines
            .iter()
            .any(|l| l.check == c && l.subject == s && l.result == r)
    };
    assert!(has("stdin", "request", "ok"), "{lines:?}");
    assert!(
        has("path", "/definitely/not/here/pnc-s3", "not_found"),
        "{lines:?}"
    );
    // Outside any sandbox the probe's in-sandbox binary path does not exist.
    assert!(
        has("self", "/pnc/pledge_compile_api", "not_found"),
        "{lines:?}"
    );
}

// ---- S-1: the run sandbox itself (Linux + bwrap) ---------------------------------------

#[cfg(target_os = "linux")]
mod linux_bwrap {
    use super::*;
    use pledge_compile_api::build_sandbox::{self, RunImage, SandboxMode, SandboxSettings};
    use std::path::PathBuf;

    /// The server binary plus the shared objects THIS test process maps
    /// (the same libc/libm/libgcc_s/ld.so the binary links against).
    fn image() -> RunImage {
        let me = std::env::current_exe().unwrap();
        let maps = std::fs::read_to_string("/proc/self/maps").unwrap();
        let mine = build_sandbox::parse_mapped_objects(&maps, &me).unwrap();
        RunImage {
            exe: PathBuf::from(EXE),
            ..mine
        }
    }

    fn settings() -> SandboxSettings {
        SandboxSettings {
            mode: SandboxMode::Bwrap,
            toolchain_dir: Some(PathBuf::from("/opt/unused-by-the-run-sandbox")),
            ..SandboxSettings::default()
        }
    }

    #[test]
    #[ignore = "needs Linux + bubblewrap; run on the deploy host"]
    fn module_runs_inside_the_run_sandbox() {
        let bytes = wat::parse_str(HELLO_WAT).unwrap();
        let input = run_sandbox::encode_run_input(&bytes, &RunLimits::default()).unwrap();
        let spec = build_sandbox::run_bwrap_spec(&settings(), &image(), "run-wasm", &[]);
        match run_child_raw(spec.to_command(), input) {
            RunChildReply::Verdict { ok, stdout, stderr } => {
                assert!(ok, "{stderr}");
                assert_eq!(stdout, "hi\n");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    #[ignore = "needs Linux + bubblewrap; run on the deploy host"]
    fn run_sandbox_hides_the_host() {
        // `/usr` itself exists inside (bwrap creates the parents of the
        // bound libraries, e.g. /usr/lib64), so probe a sibling instead;
        // the `tree` check below lists everything that is there.
        let host_paths = [
            "/etc",
            "/etc/passwd",
            "/usr/bin",
            "/usr/bin/bwrap",
            "/proc/self",
            "/run",
            "/tmp",
            "/var",
            EXE,
        ];
        for p in host_paths {
            assert!(std::path::Path::new(p).exists(), "host control: {p}");
        }
        let req = run_sandbox::ProbeRequest {
            paths: host_paths.iter().map(|p| p.to_string()).collect(),
            unix_socket: "/run/systemd/journal/socket".to_string(),
            tcp_target: "1.1.1.1:443".to_string(),
            tcp_control: "127.0.0.1:9".to_string(),
            walk_root: true,
        };
        let img = image();
        let spec = build_sandbox::run_bwrap_spec(&settings(), &img, "run-probe", &[]);
        let out = proc_tree::run_captured_with_input(
            spec.to_command(),
            Some(serde_json::to_vec(&req).unwrap()),
            Duration::from_secs(60),
            1 << 20,
            "test sandboxed probe",
        )
        .unwrap();
        assert!(out.status.success(), "{out:?}");
        let has = |l: &str| out.stdout.lines().any(|x| x == l);
        assert!(
            has(r#"{"check":"stdin","subject":"request","result":"ok"}"#),
            "{}",
            out.stdout
        );
        assert!(
            has(r#"{"check":"self","subject":"/pnc/pledge_compile_api","result":"present"}"#),
            "{}",
            out.stdout
        );
        assert!(
            has(r#"{"check":"tcp-control","subject":"127.0.0.1:9","result":"refused"}"#),
            "{}",
            out.stdout
        );
        assert!(
            has(r#"{"check":"tcp","subject":"1.1.1.1:443","result":"network_unreachable"}"#),
            "{}",
            out.stdout
        );
        for p in host_paths {
            assert!(
                has(&format!(
                    r#"{{"check":"path","subject":"{p}","result":"not_found"}}"#
                )),
                "{p} visible in the run sandbox:\n{}",
                out.stdout
            );
        }
        assert!(
            has(r#"{"check":"rlimit","subject":"cpu","result":"30/30"}"#),
            "{}",
            out.stdout
        );
        assert!(
            has(r#"{"check":"rlimit","subject":"fsize","result":"0/0"}"#),
            "{}",
            out.stdout
        );
        // bwrap sets PWD for --chdir; nothing else besides the loader path.
        assert!(
            has(r#"{"check":"env","subject":"keys","result":"LD_LIBRARY_PATH,PWD"}"#),
            "{}",
            out.stdout
        );
        // The whole filesystem: exactly the binary and its libraries.
        let mut want: Vec<String> = img
            .loader
            .iter()
            .chain(img.libs.iter())
            .map(|p| p.display().to_string())
            .chain(["/pnc/pledge_compile_api".to_string()])
            .collect();
        want.sort();
        assert!(
            has(&format!(
                r#"{{"check":"tree","subject":"/","result":"{}"}}"#,
                want.join(",")
            )),
            "{}",
            out.stdout
        );
    }
}
