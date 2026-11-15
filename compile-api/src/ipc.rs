// SPDX-License-Identifier: AGPL-3.0-or-later
//! API server <-> builder service protocol.
//!
//! # Why this shape (security re-review P1-A)
//!
//! The token-holding API server must not build or run player code, so
//! `/compile-real` work goes to a separate builder service
//! (`pnc-builder.service`) that holds no secrets. They talk over a **Unix
//! domain socket** in the builder's `RuntimeDirectory`
//! (`/run/pnc-builder/builder.sock`):
//!
//! - Access control is the filesystem: the directory is
//!   `pnc-builder:pnc-ipc 0750` and the socket `0660`, and only the API
//!   user is in `pnc-ipc`. No port, no network reachability, nothing for
//!   the reverse proxy to forward by mistake.
//! - One request per connection. A connection carries one framed request
//!   and one framed reply, then closes. No session state to confuse.
//! - Frames are `b"PNC1"` + a big-endian `u32` length + a JSON document.
//!   The length is checked against a hard cap BEFORE anything is
//!   allocated ([`MAX_REQUEST_FRAME_BYTES`], [`MAX_REPLY_FRAME_BYTES`]).
//!   JSON is parsed with `deny_unknown_fields` everywhere.
//! - Both directions have a TOTAL deadline, not just a per-read timeout:
//!   every read/write recomputes the remaining time, so a peer dripping
//!   one byte at a time cannot stretch it.
//! - Mutual distrust: the builder validates every request field
//!   ([`JobRequest::validate`]) and re-applies the source deny-list
//!   itself; the API caps and validates every reply field
//!   ([`JobReply::validate`]).
//! - A dead or wedged builder costs the API seconds, not a slot for a
//!   minute (third security review, S-2): the API connects without
//!   blocking ([`connect_builder`], bounded by [`BUILDER_CONNECT_TIMEOUT`];
//!   a Unix-socket connect to a full backlog would otherwise block), and
//!   once the builder has read and accepted a request it sends an
//!   [`ACK_PAYLOAD`] frame before starting the job. The API waits only
//!   [`ACK_DEADLINE`] for that first frame (the ack, or an immediate
//!   Busy/Rejected reply); the long [`REPLY_DEADLINE`] applies only after
//!   the ack. A socket file with no listener, a full backlog, a builder
//!   that accepts and never answers, or one that closes the connection
//!   before acknowledging are all "builder unavailable"
//!   ([`IpcError::is_unavailable`]), which the API answers with a 503.
//!
//! Everything here is generic over [`IpcStream`], so the same code runs
//! over `UnixStream` in production and over loopback `TcpStream` in the
//! Windows test suite (production never listens on TCP).

use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::expectations::{GradeState, GradedRun};
use crate::wasm_builder::{MAX_RUN_MAX_MEMORY_BYTES, RealOutcome};

/// Frame magic (protocol version 1).
pub const FRAME_MAGIC: [u8; 4] = *b"PNC1";
/// Largest request frame (64 KiB source, JSON-escaped, plus envelope).
pub const MAX_REQUEST_FRAME_BYTES: usize = 512 * 1024;
/// Largest reply frame.
pub const MAX_REPLY_FRAME_BYTES: usize = 1024 * 1024;
/// Cap on `stdout` / `stderr` in a reply.
pub const MAX_REPLY_TEXT_BYTES: usize = 256 * 1024;
/// Cap on `expected` / `actual` / `note` in a reply.
pub const MAX_REPLY_NOTE_BYTES: usize = 16 * 1024;
/// Cap on a `Rejected` / `Failed` message.
pub const MAX_REPLY_MESSAGE_BYTES: usize = 16 * 1024;
/// Smallest run memory cap a request may ask for (the config minimum).
pub const MIN_RUN_MEMORY_BYTES: u64 = 1024 * 1024;
/// Longest plausible build/run time in a reply.
const MAX_REPLY_MILLIS: u64 = 10 * 60 * 1000;

/// Total time to send or receive a request.
pub const REQUEST_IO_DEADLINE: Duration = Duration::from_secs(5);
/// Total time for the builder to send its reply once the job is done.
pub const REPLY_WRITE_DEADLINE: Duration = Duration::from_secs(5);
/// Total time the API waits for a reply after the builder acknowledged the
/// request: the job deadline plus slack for process start and the write.
pub const REPLY_DEADLINE: Duration = Duration::from_secs(crate::jobs::JOB_DEADLINE.as_secs() + 15);
/// How long the API tries to connect to the builder socket.
pub const BUILDER_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
/// How long the API waits, after sending its request, for the builder's
/// first frame (the ack, or an immediate reply). The builder reads the
/// request under its own [`REQUEST_IO_DEADLINE`] and answers at once.
pub const ACK_DEADLINE: Duration = Duration::from_secs(REQUEST_IO_DEADLINE.as_secs() + 5);
/// Payload of the builder's "request accepted, job starting" frame. Not a
/// valid [`JobReply`] encoding, so it can never be mistaken for one.
pub const ACK_PAYLOAD: &[u8] = b"\"accepted\"";

/// A byte stream with settable timeouts (TCP or Unix socket).
pub trait IpcStream: Read + Write {
    fn set_read_timeout(&self, d: Option<Duration>) -> io::Result<()>;
    fn set_write_timeout(&self, d: Option<Duration>) -> io::Result<()>;
}

impl IpcStream for std::net::TcpStream {
    fn set_read_timeout(&self, d: Option<Duration>) -> io::Result<()> {
        std::net::TcpStream::set_read_timeout(self, d)
    }
    fn set_write_timeout(&self, d: Option<Duration>) -> io::Result<()> {
        std::net::TcpStream::set_write_timeout(self, d)
    }
}

#[cfg(unix)]
impl IpcStream for std::os::unix::net::UnixStream {
    fn set_read_timeout(&self, d: Option<Duration>) -> io::Result<()> {
        std::os::unix::net::UnixStream::set_read_timeout(self, d)
    }
    fn set_write_timeout(&self, d: Option<Duration>) -> io::Result<()> {
        std::os::unix::net::UnixStream::set_write_timeout(self, d)
    }
}

/// Protocol failures. Never carries frame contents.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("ipc {op} failed: {source}")]
    Io {
        op: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("ipc {0} deadline exceeded")]
    Timeout(&'static str),
    #[error("ipc peer closed the connection during {0}")]
    Closed(&'static str),
    #[error("ipc frame of {len} bytes exceeds the {max}-byte cap")]
    FrameTooLarge { len: usize, max: usize },
    #[error("ipc frame has a bad magic number")]
    BadMagic,
    #[error("ipc payload could not be decoded: {0}")]
    Decode(String),
    #[error("ipc payload is invalid: {0}")]
    Invalid(String),
}

impl IpcError {
    /// True when the builder could not be reached or never took the
    /// request (connect failed or timed out, the request could not be
    /// written, no ack in time, closed or reset before acknowledging): the
    /// API reports "builder unavailable" (503), not a server error. After
    /// the ack, failures are server errors (the job may have run).
    pub fn is_unavailable(&self) -> bool {
        const BEFORE_ACK: [&str; 3] = ["connect", "request write", "ack read"];
        match self {
            Self::Io { op, .. } | Self::Timeout(op) | Self::Closed(op) => BEFORE_ACK.contains(op),
            _ => false,
        }
    }
}

fn remaining(deadline: Instant, what: &'static str) -> Result<Duration, IpcError> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        tracing::warn!(what, "ipc: deadline exceeded");
        return Err(IpcError::Timeout(what));
    }
    // A zero timeout means "block forever" to the socket API.
    Ok(left.max(Duration::from_millis(1)))
}

fn is_timeout(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

/// Fill `buf` completely before `deadline`.
fn read_full<S: IpcStream>(
    s: &mut S,
    buf: &mut [u8],
    deadline: Instant,
    what: &'static str,
) -> Result<(), IpcError> {
    let mut filled = 0;
    while filled < buf.len() {
        let left = remaining(deadline, what)?;
        s.set_read_timeout(Some(left))
            .map_err(|source| IpcError::Io { op: what, source })?;
        match s.read(&mut buf[filled..]) {
            Ok(0) => return Err(IpcError::Closed(what)),
            Ok(n) => filled += n,
            Err(e) if is_timeout(&e) || e.kind() == io::ErrorKind::Interrupted => {}
            Err(source) => return Err(IpcError::Io { op: what, source }),
        }
    }
    Ok(())
}

/// Write all of `buf` before `deadline`.
fn write_full<S: IpcStream>(
    s: &mut S,
    buf: &[u8],
    deadline: Instant,
    what: &'static str,
) -> Result<(), IpcError> {
    let mut sent = 0;
    while sent < buf.len() {
        let left = remaining(deadline, what)?;
        s.set_write_timeout(Some(left))
            .map_err(|source| IpcError::Io { op: what, source })?;
        match s.write(&buf[sent..]) {
            Ok(0) => return Err(IpcError::Closed(what)),
            Ok(n) => sent += n,
            Err(e) if is_timeout(&e) || e.kind() == io::ErrorKind::Interrupted => {}
            Err(source) => return Err(IpcError::Io { op: what, source }),
        }
    }
    let _ = s.flush();
    Ok(())
}

/// Send one frame. Refuses (without sending) a payload over `max`.
pub fn write_frame<S: IpcStream>(
    s: &mut S,
    payload: &[u8],
    max: usize,
    deadline: Instant,
    what: &'static str,
) -> Result<(), IpcError> {
    if payload.len() > max {
        tracing::error!(
            what,
            len = payload.len(),
            max,
            "ipc: refusing to send oversized frame"
        );
        return Err(IpcError::FrameTooLarge {
            len: payload.len(),
            max,
        });
    }
    let len = u32::try_from(payload.len()).map_err(|_| IpcError::FrameTooLarge {
        len: payload.len(),
        max,
    })?;
    let mut header = [0u8; 8];
    header[..4].copy_from_slice(&FRAME_MAGIC);
    header[4..].copy_from_slice(&len.to_be_bytes());
    write_full(s, &header, deadline, what)?;
    write_full(s, payload, deadline, what)
}

/// Receive one frame of at most `max` bytes. The length is checked before
/// any payload buffer is allocated.
pub fn read_frame<S: IpcStream>(
    s: &mut S,
    max: usize,
    deadline: Instant,
    what: &'static str,
) -> Result<Vec<u8>, IpcError> {
    let mut header = [0u8; 8];
    read_full(s, &mut header, deadline, what)?;
    if header[..4] != FRAME_MAGIC {
        tracing::warn!(what, "ipc: bad frame magic");
        return Err(IpcError::BadMagic);
    }
    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&header[4..]);
    let len = u32::from_be_bytes(len_bytes) as usize;
    if len > max {
        tracing::warn!(what, len, max, "ipc: oversized frame refused");
        return Err(IpcError::FrameTooLarge { len, max });
    }
    let mut payload = vec![0u8; len];
    read_full(s, &mut payload, deadline, what)?;
    Ok(payload)
}

// ---- messages ----------------------------------------------------------------

/// What the API asks the builder to do: build, run and grade one
/// submission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobRequest {
    pub encounter_id: String,
    pub source: String,
    pub run_max_memory_bytes: u64,
}

impl JobRequest {
    /// Every check the API applies, applied again by the receiver.
    pub fn validate(&self) -> Result<(), IpcError> {
        let bad = |m: String| {
            tracing::warn!("ipc: invalid request: {m}");
            Err(IpcError::Invalid(m))
        };
        if !crate::valid_encounter_id(&self.encounter_id) {
            return bad(format!(
                "encounter_id ({} bytes) is not 1-{} chars of [A-Za-z0-9_-]",
                self.encounter_id.len(),
                crate::MAX_ENCOUNTER_ID_BYTES
            ));
        }
        if self.source.trim().is_empty() {
            return bad("source is empty".to_string());
        }
        if self.source.len() > crate::MAX_SOURCE_BYTES {
            return bad(format!(
                "source is {} bytes, over {}",
                self.source.len(),
                crate::MAX_SOURCE_BYTES
            ));
        }
        if !(MIN_RUN_MEMORY_BYTES..=MAX_RUN_MAX_MEMORY_BYTES).contains(&self.run_max_memory_bytes) {
            return bad(format!(
                "run_max_memory_bytes {} outside {MIN_RUN_MEMORY_BYTES}..={MAX_RUN_MAX_MEMORY_BYTES}",
                self.run_max_memory_bytes
            ));
        }
        Ok(())
    }
}

/// A graded run on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireGradedRun {
    pub compiled: bool,
    pub ran_ok: bool,
    pub stdout: String,
    pub stderr: String,
    pub build_ms: u64,
    pub run_ms: Option<u64>,
    pub state: GradeState,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub note: String,
}

fn millis(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

impl From<GradedRun> for WireGradedRun {
    fn from(g: GradedRun) -> Self {
        Self {
            compiled: g.outcome.compiled,
            ran_ok: g.outcome.ran_ok,
            stdout: g.outcome.stdout,
            stderr: g.outcome.stderr,
            build_ms: millis(g.outcome.build_time),
            run_ms: g.outcome.run_time.map(millis),
            state: g.state,
            expected: g.expected,
            actual: g.actual,
            note: g.note,
        }
    }
}

impl WireGradedRun {
    /// Shrink over-long text to the wire caps (sender side), on a char
    /// boundary, with a marker.
    pub fn clamp(mut self) -> Self {
        clamp_text(&mut self.stdout, MAX_REPLY_TEXT_BYTES);
        clamp_text(&mut self.stderr, MAX_REPLY_TEXT_BYTES);
        clamp_text(&mut self.note, MAX_REPLY_NOTE_BYTES);
        if let Some(e) = self.expected.as_mut() {
            clamp_text(e, MAX_REPLY_NOTE_BYTES);
        }
        if let Some(a) = self.actual.as_mut() {
            clamp_text(a, MAX_REPLY_NOTE_BYTES);
        }
        self
    }

    /// Receiver-side checks.
    pub fn validate(&self) -> Result<(), IpcError> {
        let over = |name: &str, len: usize, max: usize| {
            Err(IpcError::Invalid(format!(
                "{name} is {len} bytes, over {max}"
            )))
        };
        for (name, text, max) in [
            ("stdout", &self.stdout, MAX_REPLY_TEXT_BYTES),
            ("stderr", &self.stderr, MAX_REPLY_TEXT_BYTES),
            ("note", &self.note, MAX_REPLY_NOTE_BYTES),
        ] {
            if text.len() > max {
                return over(name, text.len(), max);
            }
        }
        for (name, text) in [("expected", &self.expected), ("actual", &self.actual)] {
            if let Some(t) = text
                && t.len() > MAX_REPLY_NOTE_BYTES
            {
                return over(name, t.len(), MAX_REPLY_NOTE_BYTES);
            }
        }
        if self.build_ms > MAX_REPLY_MILLIS || self.run_ms.is_some_and(|r| r > MAX_REPLY_MILLIS) {
            return Err(IpcError::Invalid("implausible duration".to_string()));
        }
        if !self.compiled && (self.ran_ok || self.run_ms.is_some()) {
            return Err(IpcError::Invalid(
                "a run is reported for a build that did not compile".to_string(),
            ));
        }
        Ok(())
    }

    pub fn into_graded(self) -> GradedRun {
        GradedRun {
            outcome: RealOutcome {
                compiled: self.compiled,
                ran_ok: self.ran_ok,
                stdout: self.stdout,
                stderr: self.stderr,
                build_time: Duration::from_millis(self.build_ms),
                run_time: self.run_ms.map(Duration::from_millis),
            },
            state: self.state,
            expected: self.expected,
            actual: self.actual,
            note: self.note,
        }
    }
}

fn clamp_text(s: &mut String, max: usize) {
    const MARK: &str = "\n... [truncated]";
    if s.len() <= max {
        return;
    }
    let mut cut = max.saturating_sub(MARK.len());
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    s.truncate(cut);
    s.push_str(MARK);
}

/// The builder's answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobReply {
    /// The submission was built (and maybe run) and graded.
    Graded(WireGradedRun),
    /// Every job slot is busy; retry later (API answers 503).
    Busy,
    /// The request failed validation.
    Rejected(String),
    /// Setup failure (job dir, spawn, ...), not a player error.
    Failed(String),
}

impl JobReply {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Graded(_) => "graded",
            Self::Busy => "busy",
            Self::Rejected(_) => "rejected",
            Self::Failed(_) => "failed",
        }
    }

    /// Receiver-side checks.
    pub fn validate(&self) -> Result<(), IpcError> {
        match self {
            Self::Graded(run) => run.validate(),
            Self::Busy => Ok(()),
            Self::Rejected(m) | Self::Failed(m) => {
                if m.len() > MAX_REPLY_MESSAGE_BYTES {
                    Err(IpcError::Invalid(format!(
                        "message is {} bytes, over {MAX_REPLY_MESSAGE_BYTES}",
                        m.len()
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Sender-side clamp of every text field to the wire caps.
    pub fn clamp(self) -> Self {
        match self {
            Self::Graded(run) => Self::Graded(run.clamp()),
            Self::Rejected(mut m) => {
                clamp_text(&mut m, MAX_REPLY_MESSAGE_BYTES);
                Self::Rejected(m)
            }
            Self::Failed(mut m) => {
                clamp_text(&mut m, MAX_REPLY_MESSAGE_BYTES);
                Self::Failed(m)
            }
            Self::Busy => Self::Busy,
        }
    }
}

/// Strictly decode and validate a reply payload.
pub fn decode_reply(payload: &[u8]) -> Result<JobReply, IpcError> {
    let reply: JobReply = serde_json::from_slice(payload).map_err(|e| {
        tracing::warn!("ipc: undecodable reply: {e}");
        IpcError::Decode(e.to_string())
    })?;
    reply.validate().inspect_err(|e| {
        tracing::warn!("ipc: invalid reply: {e}");
    })?;
    Ok(reply)
}

/// Strictly decode and validate a request payload.
pub fn decode_request(payload: &[u8]) -> Result<JobRequest, IpcError> {
    let req: JobRequest = serde_json::from_slice(payload).map_err(|e| {
        tracing::warn!("ipc: undecodable request: {e}");
        IpcError::Decode(e.to_string())
    })?;
    req.validate()?;
    Ok(req)
}

/// Builder side: tell the client the request was accepted and the job is
/// starting (see the module docs).
pub fn write_ack<S: IpcStream>(stream: &mut S, deadline: Instant) -> Result<(), IpcError> {
    write_frame(
        stream,
        ACK_PAYLOAD,
        MAX_REPLY_FRAME_BYTES,
        deadline,
        "ack write",
    )
}

/// Client side of one exchange: send `req`, wait up to [`ACK_DEADLINE`]
/// (or `reply_deadline`, if shorter) for the builder's first frame, then
/// up to `reply_deadline` for the reply; decode and validate it.
pub fn exchange<S: IpcStream>(
    stream: &mut S,
    req: &JobRequest,
    reply_deadline: Duration,
) -> Result<JobReply, IpcError> {
    exchange_with(
        stream,
        req,
        ACK_DEADLINE.min(reply_deadline),
        reply_deadline,
    )
}

/// [`exchange`] with an explicit ack deadline (tests).
pub fn exchange_with<S: IpcStream>(
    stream: &mut S,
    req: &JobRequest,
    ack_deadline: Duration,
    reply_deadline: Duration,
) -> Result<JobReply, IpcError> {
    req.validate()?;
    let payload = serde_json::to_vec(req).map_err(|e| IpcError::Decode(e.to_string()))?;
    write_frame(
        stream,
        &payload,
        MAX_REQUEST_FRAME_BYTES,
        Instant::now() + REQUEST_IO_DEADLINE,
        "request write",
    )?;
    let first = read_frame(
        stream,
        MAX_REPLY_FRAME_BYTES,
        Instant::now() + ack_deadline,
        "ack read",
    )
    .inspect_err(|e| tracing::warn!("ipc: builder did not acknowledge the request: {e}"))?;
    let reply = if first == ACK_PAYLOAD {
        tracing::debug!("ipc: builder accepted the request");
        read_frame(
            stream,
            MAX_REPLY_FRAME_BYTES,
            Instant::now() + reply_deadline,
            "reply read",
        )?
    } else {
        first
    };
    let reply = decode_reply(&reply)?;
    tracing::debug!(kind = reply.kind(), "ipc: reply received");
    Ok(reply)
}

/// Connect to the builder's Unix socket within `timeout`.
///
/// `UnixStream::connect` has no timeout, and a connect to a listener whose
/// backlog is full (a stopped or wedged builder) blocks. So the socket is
/// created non-blocking: an `AF_UNIX` connect then either completes at
/// once, fails at once (`ECONNREFUSED`: socket file without a listener,
/// `ENOENT`), or reports `EAGAIN` (backlog full), which is retried until
/// `timeout`. The connected stream is switched back to blocking mode
/// (reads and writes use socket timeouts from [`exchange`]).
#[cfg(unix)]
pub fn connect_builder(
    socket: &std::path::Path,
    timeout: Duration,
) -> Result<std::os::unix::net::UnixStream, IpcError> {
    use rustix::io::Errno;
    use rustix::net::{AddressFamily, SocketAddrUnix, SocketFlags, SocketType};

    let io_err = |e: Errno| IpcError::Io {
        op: "connect",
        source: io::Error::from(e),
    };
    let addr = SocketAddrUnix::new(socket).map_err(io_err)?;
    let fd = rustix::net::socket_with(
        AddressFamily::UNIX,
        SocketType::STREAM,
        SocketFlags::NONBLOCK | SocketFlags::CLOEXEC,
        None,
    )
    .map_err(io_err)?;
    let deadline = Instant::now() + timeout;
    loop {
        match rustix::net::connect(&fd, &addr) {
            Ok(()) => break,
            Err(e) if e == Errno::ISCONN => break,
            Err(e) if e == Errno::INTR => continue,
            Err(e) if e == Errno::AGAIN || e == Errno::INPROGRESS || e == Errno::ALREADY => {
                if Instant::now() >= deadline {
                    tracing::error!(
                        ?timeout,
                        "ipc: builder at {} did not accept a connection in time (backlog full?)",
                        socket.display()
                    );
                    return Err(IpcError::Timeout("connect"));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                tracing::error!(
                    "ipc: cannot connect to builder at {}: {e}",
                    socket.display()
                );
                return Err(io_err(e));
            }
        }
    }
    let stream = std::os::unix::net::UnixStream::from(fd);
    stream
        .set_nonblocking(false)
        .map_err(|source| IpcError::Io {
            op: "connect",
            source,
        })?;
    Ok(stream)
}

/// Connect to the builder socket (bounded) and run one [`exchange`].
#[cfg(unix)]
pub fn call_builder(socket: &std::path::Path, req: &JobRequest) -> Result<JobReply, IpcError> {
    let mut stream = connect_builder(socket, BUILDER_CONNECT_TIMEOUT)?;
    exchange(&mut stream, req, REPLY_DEADLINE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> JobRequest {
        JobRequest {
            encounter_id: "intro_let_binding".to_string(),
            source: "fn main() {}".to_string(),
            run_max_memory_bytes: 16 * 1024 * 1024,
        }
    }

    #[test]
    fn request_validation_mirrors_the_api() {
        assert!(req().validate().is_ok());
        for bad in [
            JobRequest {
                encounter_id: "../x".to_string(),
                ..req()
            },
            JobRequest {
                source: "  ".to_string(),
                ..req()
            },
            JobRequest {
                source: "x".repeat(crate::MAX_SOURCE_BYTES + 1),
                ..req()
            },
            JobRequest {
                run_max_memory_bytes: MAX_RUN_MAX_MEMORY_BYTES + 1,
                ..req()
            },
            JobRequest {
                run_max_memory_bytes: 0,
                ..req()
            },
        ] {
            assert!(bad.validate().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn decoding_is_strict() {
        let ok = serde_json::to_vec(&req()).unwrap();
        assert!(decode_request(&ok).is_ok());
        let extra = br#"{"encounter_id":"a","source":"fn main(){}","run_max_memory_bytes":16777216,"skip_denylist":true}"#;
        assert!(matches!(decode_request(extra), Err(IpcError::Decode(_))));
        assert!(matches!(decode_reply(br#""busy""#), Ok(JobReply::Busy)));
        assert!(decode_reply(br#""Busy""#).is_err());
        assert!(decode_reply(br#"{"graded":{"compiled":true}}"#).is_err());
        assert!(decode_reply(br#"{"shell":"rm -rf /"}"#).is_err());
    }

    #[test]
    fn reply_caps_are_enforced_and_clamp_fits_them() {
        let run = WireGradedRun {
            compiled: true,
            ran_ok: true,
            stdout: "x".repeat(MAX_REPLY_TEXT_BYTES + 10),
            stderr: String::new(),
            build_ms: 1,
            run_ms: Some(1),
            state: GradeState::RanNotChecked,
            expected: None,
            actual: None,
            note: "n".to_string(),
        };
        assert!(run.validate().is_err());
        let clamped = run.clamp();
        assert!(clamped.validate().is_ok());
        assert!(clamped.stdout.ends_with("[truncated]"));
        let inconsistent = WireGradedRun {
            compiled: false,
            ran_ok: true,
            stdout: String::new(),
            ..clamped
        };
        assert!(inconsistent.validate().is_err());
    }

    #[test]
    fn clamp_respects_char_boundaries() {
        let mut s = "\u{e9}".repeat(100);
        clamp_text(&mut s, 51);
        assert!(s.len() <= 51);
        assert!(s.ends_with("[truncated]"));
    }
}
