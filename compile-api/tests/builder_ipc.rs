// SPDX-License-Identifier: AGPL-3.0-or-later
//! API <-> builder protocol, both ends treating the other as untrusted.
//!
//! Production uses a Unix socket; std has none on Windows, so these tests
//! run the SAME generic code (`ipc::exchange`, `builder::handle_connection`)
//! over loopback TCP. No toolchain needed: the job runner is injected.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use pledge_compile_api::builder::{
    BuilderCtx, ConnOutcome, JobRunner, JobSlots, handle_connection,
};
use pledge_compile_api::expectations::GradeState;
use pledge_compile_api::ipc::{
    self, FRAME_MAGIC, IpcError, JobReply, JobRequest, MAX_REPLY_FRAME_BYTES,
    MAX_REQUEST_FRAME_BYTES, WireGradedRun,
};
use pledge_compile_api::jobs::{JobError, JobRun};
use pledge_compile_api::proc_tree::ProcError;
use pledge_compile_api::wasm_builder::BuildSettings;

fn request() -> JobRequest {
    JobRequest {
        encounter_id: "intro_let_binding".to_string(),
        source: "fn main() { println!(\"42\"); }".to_string(),
        run_max_memory_bytes: 16 << 20,
    }
}

fn graded() -> WireGradedRun {
    WireGradedRun {
        compiled: true,
        ran_ok: true,
        stdout: "42\n".to_string(),
        stderr: String::new(),
        build_ms: 5,
        run_ms: Some(1),
        state: GradeState::RanNotChecked,
        expected: None,
        actual: None,
        note: "stub".to_string(),
    }
}

fn ok_runner() -> JobRunner {
    Arc::new(|env| {
        assert!(
            env.settings.source_denylist,
            "builder always applies the deny-list"
        );
        Ok(JobRun {
            reply: JobReply::Graded(graded()),
            pid: 0,
            timed_out: false,
            elapsed: Duration::from_millis(5),
        })
    })
}

fn ctx(slots: usize, runner: JobRunner) -> BuilderCtx {
    BuilderCtx {
        settings: BuildSettings::default(),
        slots: JobSlots::new(slots),
        runner,
        on_survivors: Arc::new(|| {}),
        request_deadline: Duration::from_millis(500),
        reply_deadline: Duration::from_millis(500),
    }
}

/// Serve exactly one connection with `ctx`; returns the address and a
/// handle yielding the outcome.
fn serve_one(ctx: BuilderCtx) -> (SocketAddr, std::thread::JoinHandle<ConnOutcome>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let h = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        handle_connection(&mut s, &ctx)
    });
    (addr, h)
}

fn frame(payload: &[u8]) -> Vec<u8> {
    let mut v = FRAME_MAGIC.to_vec();
    v.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    v.extend_from_slice(payload);
    v
}

/// Read the single reply frame the builder sends back.
fn read_reply(s: &mut TcpStream) -> Result<JobReply, IpcError> {
    let payload = ipc::read_frame(
        s,
        MAX_REPLY_FRAME_BYTES,
        Instant::now() + Duration::from_secs(5),
        "test reply",
    )?;
    ipc::decode_reply(&payload)
}

#[test]
fn valid_request_round_trips() {
    let (addr, h) = serve_one(ctx(1, ok_runner()));
    let mut s = TcpStream::connect(addr).unwrap();
    let reply = ipc::exchange(&mut s, &request(), Duration::from_secs(5)).unwrap();
    assert_eq!(reply, JobReply::Graded(graded()));
    assert_eq!(h.join().unwrap(), ConnOutcome::Replied("graded"));
}

#[test]
fn full_builder_answers_busy() {
    let (addr, h) = serve_one(ctx(0, ok_runner()));
    let mut s = TcpStream::connect(addr).unwrap();
    let reply = ipc::exchange(&mut s, &request(), Duration::from_secs(5)).unwrap();
    assert_eq!(reply, JobReply::Busy);
    assert_eq!(h.join().unwrap(), ConnOutcome::Replied("busy"));
    // The API maps Busy to the handler's queue-full error.
    let err = pledge_compile_api::jobs::reply_to_graded(JobReply::Busy).unwrap_err();
    assert!(matches!(
        err.downcast_ref::<pledge_compile_api::jobs::ExecError>(),
        Some(pledge_compile_api::jobs::ExecError::Busy)
    ));
}

#[test]
fn hostile_requests_are_rejected_before_any_job() {
    let ran = Arc::new(AtomicBool::new(false));
    let bad_payloads: Vec<Vec<u8>> = vec![
        // Unknown field (e.g. an attempt to switch off the deny-list).
        br#"{"encounter_id":"a","source":"fn main(){}","run_max_memory_bytes":16777216,"source_denylist":false}"#.to_vec(),
        // Path-like encounter id.
        br#"{"encounter_id":"../../etc","source":"fn main(){}","run_max_memory_bytes":16777216}"#.to_vec(),
        // Memory cap above the ceiling.
        br#"{"encounter_id":"a","source":"fn main(){}","run_max_memory_bytes":999999999999}"#.to_vec(),
        // Not JSON.
        b"\x00\x01\x02".to_vec(),
    ];
    for payload in bad_payloads {
        let ran_c = Arc::clone(&ran);
        let runner: JobRunner = Arc::new(move |_| {
            ran_c.store(true, Ordering::SeqCst);
            Err(JobError::BadReply("must not run".to_string()))
        });
        let (addr, h) = serve_one(ctx(1, runner));
        let mut s = TcpStream::connect(addr).unwrap();
        s.write_all(&frame(&payload)).unwrap();
        let reply = read_reply(&mut s).unwrap();
        assert!(matches!(reply, JobReply::Rejected(_)), "{reply:?}");
        assert_eq!(h.join().unwrap(), ConnOutcome::Replied("rejected"));
    }
    assert!(
        !ran.load(Ordering::SeqCst),
        "no job may run for a bad request"
    );
}

#[test]
fn bad_magic_and_oversized_frames_are_refused_unread() {
    // Bad magic.
    let (addr, h) = serve_one(ctx(1, ok_runner()));
    let mut s = TcpStream::connect(addr).unwrap();
    // Exactly one 8-byte header's worth: the builder refuses on the header
    // alone. (Extra unread bytes would make the server's close a TCP
    // reset, which can discard the reply before this client reads it.)
    s.write_all(b"HTTP/1.1").unwrap();
    assert!(matches!(read_reply(&mut s).unwrap(), JobReply::Rejected(_)));
    assert!(matches!(h.join().unwrap(), ConnOutcome::ClientError(_)));

    // Declared length over the cap: refused from the header alone, the
    // 512 KiB+ payload is never sent or allocated.
    let (addr, h) = serve_one(ctx(1, ok_runner()));
    let mut s = TcpStream::connect(addr).unwrap();
    let mut header = FRAME_MAGIC.to_vec();
    header.extend_from_slice(&((MAX_REQUEST_FRAME_BYTES as u32) + 1).to_be_bytes());
    s.write_all(&header).unwrap();
    let reply = read_reply(&mut s).unwrap();
    assert!(
        matches!(&reply, JobReply::Rejected(m) if m.contains("exceeds")),
        "{reply:?}"
    );
    assert!(matches!(h.join().unwrap(), ConnOutcome::ClientError(_)));
}

#[test]
fn slow_client_is_dropped_at_the_request_deadline() {
    let (addr, h) = serve_one(ctx(1, ok_runner()));
    let mut s = TcpStream::connect(addr).unwrap();
    let started = Instant::now();
    // Half a header, then nothing: the server's TOTAL deadline (500 ms)
    // must fire even though bytes did arrive.
    s.write_all(&FRAME_MAGIC[..2]).unwrap();
    let outcome = h.join().unwrap();
    assert!(
        matches!(&outcome, ConnOutcome::ClientError(m) if m.contains("deadline")),
        "{outcome:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(3));
    let mut buf = [0u8; 1];
    s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    assert!(
        matches!(s.read(&mut buf), Ok(0) | Err(_)),
        "connection closed"
    );
}

#[test]
fn survivors_retire_the_slot_and_escalate() {
    let escalated = Arc::new(AtomicBool::new(false));
    let esc = Arc::clone(&escalated);
    let runner: JobRunner = Arc::new(|_| {
        Err(JobError::Proc(ProcError::Survivors {
            label: "job".to_string(),
            pgid: 4242,
            waited: Duration::from_secs(5),
        }))
    });
    let mut c = ctx(2, runner);
    c.on_survivors = Arc::new(move || esc.store(true, Ordering::SeqCst));
    let slots = Arc::clone(&c.slots);
    let (addr, h) = serve_one(c);
    let mut s = TcpStream::connect(addr).unwrap();
    let err = ipc::exchange(&mut s, &request(), Duration::from_secs(5)).unwrap_err();
    assert!(
        matches!(err, IpcError::Closed(_) | IpcError::Io { .. }),
        "{err:?}"
    );
    assert_eq!(h.join().unwrap(), ConnOutcome::Survivors);
    assert!(escalated.load(Ordering::SeqCst), "builder must escalate");
    assert_eq!(
        slots.available(),
        1,
        "the survivors' slot is never released"
    );
}

/// A fake builder for client-side tests: reads one request, then runs
/// `respond` on the raw stream.
fn fake_builder(respond: impl FnOnce(&mut TcpStream) + Send + 'static) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let _ = ipc::read_frame(
            &mut s,
            MAX_REQUEST_FRAME_BYTES,
            Instant::now() + Duration::from_secs(5),
            "fake read",
        );
        respond(&mut s);
        std::thread::sleep(Duration::from_millis(1500));
    });
    addr
}

#[test]
fn api_side_rejects_hostile_builder_replies() {
    // Oversized declared reply.
    let addr = fake_builder(|s| {
        let mut h = FRAME_MAGIC.to_vec();
        h.extend_from_slice(&u32::MAX.to_be_bytes());
        let _ = s.write_all(&h);
    });
    let mut s = TcpStream::connect(addr).unwrap();
    let err = ipc::exchange(&mut s, &request(), Duration::from_secs(5)).unwrap_err();
    assert!(matches!(err, IpcError::FrameTooLarge { .. }), "{err:?}");

    // Unknown variant / extra fields.
    for body in [
        br#"{"exec":"sh"}"#.to_vec(),
        br#"{"graded":{"compiled":true,"ran_ok":true,"stdout":"","stderr":"","build_ms":1,"run_ms":1,"state":"passed","expected":null,"actual":null,"note":"","admin":true}}"#.to_vec(),
    ] {
        let addr = fake_builder(move |s| {
            let _ = s.write_all(&frame(&body));
        });
        let mut s = TcpStream::connect(addr).unwrap();
        let err = ipc::exchange(&mut s, &request(), Duration::from_secs(5)).unwrap_err();
        assert!(matches!(err, IpcError::Decode(_)), "{err:?}");
    }

    // Well-formed but over the per-field cap.
    let addr = fake_builder(|s| {
        let mut run = graded();
        run.stdout = "x".repeat(ipc::MAX_REPLY_TEXT_BYTES + 1);
        let body = serde_json::to_vec(&JobReply::Graded(run)).unwrap();
        let _ = s.write_all(&frame(&body));
    });
    let mut s = TcpStream::connect(addr).unwrap();
    let err = ipc::exchange(&mut s, &request(), Duration::from_secs(5)).unwrap_err();
    assert!(matches!(err, IpcError::Invalid(_)), "{err:?}");

    // A builder that drips its reply past the total reply deadline.
    let addr = fake_builder(|s| {
        for b in FRAME_MAGIC {
            let _ = s.write_all(&[b]);
            std::thread::sleep(Duration::from_millis(200));
        }
    });
    let mut s = TcpStream::connect(addr).unwrap();
    let started = Instant::now();
    let err = ipc::exchange(&mut s, &request(), Duration::from_millis(500)).unwrap_err();
    assert!(matches!(err, IpcError::Timeout(_)), "{err:?}");
    assert!(started.elapsed() < Duration::from_secs(2));
}
