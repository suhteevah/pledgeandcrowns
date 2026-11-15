// SPDX-License-Identifier: AGPL-3.0-or-later
//! The run phase in its own bubblewrap sandbox (third security review,
//! S-1).
//!
//! Before this module the built module ran in the `pledge_compile_api job`
//! process itself, with the builder unit's view of the host: read-only,
//! but every world-readable file, the builder's socket and the host's
//! filesystem Unix sockets were reachable from a wasmtime escape. Now, in
//! `bwrap` mode:
//!
//! ```text
//! job process (supervisor, host view)            run child (sandboxed)
//!   prlimit -- bwrap <see build_sandbox::run_bwrap_spec> -- [ld.so] /pnc/pledge_compile_api run-wasm
//!   stdin : header line (limits, JSON) + raw module bytes   ->  read, capped
//!   stdout: one JSON RunChildReply                          <-  written once
//!   stderr: the child's logs (relayed at debug, capped)
//! ```
//!
//! - The child runs exactly the in-process code ([`wasm_runner::run_wasm`]):
//!   fuel, epoch interruption, the async run timeout, the store limiter and
//!   the backstop all still apply inside it.
//! - The job process supervises it through [`proc_tree`] with a wall-clock
//!   deadline of compile budget + run timeout + margin + [`RUN_CHILD_SLACK`];
//!   on expiry the whole tree is SIGKILLed (bwrap's PID namespace and
//!   `--die-with-parent` take every process inside with it) and the run is
//!   graded as a timeout. The builder's 50 s job deadline stays the outer
//!   bound.
//! - Every input crossing into the child is validated again there (limits
//!   no looser than the shipped ones, module size, exact length), and the
//!   child's reply is decoded strictly with a size cap.
//!
//! `none` mode (Windows, local dev, the test suite) keeps running the
//! module in-process: there is no bwrap there.
//!
//! The `run-probe` subcommand is the server-owned native probe that
//! `deploy/verify-sandbox.sh` runs in place of wasmtime (through
//! `sandbox-verify-run`, see `builder.rs`): a guest cannot be the prober,
//! wasmtime itself confines it, so the script needs a native program in
//! exactly this sandbox to show what an escape would find there.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::ExitCode;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::build_sandbox::{self, RunImage, SANDBOX_RUN_EXE, SandboxSettings};
use crate::proc_tree;
use crate::wasm_builder::{MAX_RUN_MAX_MEMORY_BYTES, RUN_FUEL, RUN_TIMEOUT_MS, RunLimits};
use crate::wasm_runner::{
    self, MAX_WASM_BYTES, MODULE_COMPILE_TIMEOUT, RUN_TIMEOUT_MARGIN, RunVerdict,
};

/// Subcommand that runs one module inside the run sandbox.
pub const RUN_WASM_SUBCOMMAND: &str = "run-wasm";
/// Subcommand of the native probe (`verify-sandbox.sh`).
pub const RUN_PROBE_SUBCOMMAND: &str = "run-probe";

/// Extra wall-clock time the supervisor allows the run child beyond its
/// own budgets (process start-up, bwrap namespace setup, the reply).
pub const RUN_CHILD_SLACK: Duration = Duration::from_secs(2);

/// Largest header line on the child's stdin.
const MAX_HEADER_BYTES: usize = 4 * 1024;
/// Largest stdin the child accepts: header + newline + module.
pub const MAX_RUN_INPUT_BYTES: usize = MAX_HEADER_BYTES + 1 + MAX_WASM_BYTES;
/// Cap on the child's stdout (the JSON reply). The guest's own output is
/// at most 2 x 64 KiB before JSON escaping (at most 6x), plus the trap
/// text; 2 MiB is above anything a real reply can reach.
const RUN_REPLY_CAP: usize = 2 * 1024 * 1024;
/// Cap on the child's stderr (its logs).
const RUN_LOG_CAP: usize = 64 * 1024;
/// Most child log text relayed into this process's log.
const RUN_LOG_RELAY: usize = 16 * 1024;
/// Largest probe request.
pub const MAX_PROBE_REQUEST_BYTES: usize = 64 * 1024;
/// Wall-clock bound on one probe run.
const PROBE_DEADLINE: Duration = Duration::from_secs(30);
/// Timeout of the probe's TCP connection attempts.
const PROBE_TCP_TIMEOUT: Duration = Duration::from_secs(3);

/// Run-sandbox failures (setup, not the player's program).
#[derive(Debug, thiserror::Error)]
pub enum RunSandboxError {
    #[error("run sandbox is only available on Linux")]
    Unsupported,
    #[error("could not locate the server binary or its libraries: {0}")]
    Image(String),
    #[error("run input is invalid: {0}")]
    BadInput(String),
    #[error("run child: {0}")]
    Child(String),
    #[error(transparent)]
    Proc(#[from] proc_tree::ProcError),
}

/// Limits and size sent ahead of the module bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunHeader {
    pub timeout_ms: u64,
    pub fuel: u64,
    pub max_memory_bytes: u64,
    pub wasm_bytes: u64,
}

impl RunHeader {
    /// The child's checks: never looser than the shipped run budget.
    pub fn validate(&self) -> Result<(), RunSandboxError> {
        let bad = |m: String| {
            tracing::warn!("run_sandbox: rejected header: {m}");
            Err(RunSandboxError::BadInput(m))
        };
        if self.timeout_ms == 0 || self.timeout_ms > RUN_TIMEOUT_MS {
            return bad(format!(
                "timeout_ms {} outside 1..={RUN_TIMEOUT_MS}",
                self.timeout_ms
            ));
        }
        if self.fuel == 0 || self.fuel > RUN_FUEL {
            return bad(format!("fuel {} outside 1..={RUN_FUEL}", self.fuel));
        }
        if self.max_memory_bytes == 0 || self.max_memory_bytes > MAX_RUN_MAX_MEMORY_BYTES {
            return bad(format!(
                "max_memory_bytes {} outside 1..={MAX_RUN_MAX_MEMORY_BYTES}",
                self.max_memory_bytes
            ));
        }
        if self.wasm_bytes == 0 || self.wasm_bytes > MAX_WASM_BYTES as u64 {
            return bad(format!(
                "wasm_bytes {} outside 1..={MAX_WASM_BYTES}",
                self.wasm_bytes
            ));
        }
        Ok(())
    }
}

/// What the run child writes to stdout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RunChildReply {
    /// The module was compiled and run (or stopped): the verdict.
    Verdict {
        ok: bool,
        stdout: String,
        stderr: String,
    },
    /// The run could not be set up (invalid module, bad input, ...).
    SetupError(String),
}

/// Encode the child's stdin: one JSON header line, `\n`, the module bytes.
pub fn encode_run_input(bytes: &[u8], limits: &RunLimits) -> Result<Vec<u8>, RunSandboxError> {
    let header = RunHeader {
        timeout_ms: limits.timeout_ms,
        fuel: limits.fuel,
        max_memory_bytes: limits.max_memory_bytes,
        wasm_bytes: bytes.len() as u64,
    };
    header.validate()?;
    let mut out =
        serde_json::to_vec(&header).map_err(|e| RunSandboxError::BadInput(e.to_string()))?;
    out.push(b'\n');
    out.extend_from_slice(bytes);
    Ok(out)
}

/// Decode and validate the child's stdin.
pub fn decode_run_input(input: &[u8]) -> Result<(RunHeader, &[u8]), RunSandboxError> {
    if input.len() > MAX_RUN_INPUT_BYTES {
        return Err(RunSandboxError::BadInput(format!(
            "input is {} bytes, over {MAX_RUN_INPUT_BYTES}",
            input.len()
        )));
    }
    let nl = input
        .iter()
        .take(MAX_HEADER_BYTES + 1)
        .position(|&b| b == b'\n')
        .ok_or_else(|| RunSandboxError::BadInput("no header line".to_string()))?;
    let header: RunHeader = serde_json::from_slice(&input[..nl])
        .map_err(|e| RunSandboxError::BadInput(format!("header: {e}")))?;
    header.validate()?;
    let wasm = &input[nl + 1..];
    if wasm.len() as u64 != header.wasm_bytes {
        return Err(RunSandboxError::BadInput(format!(
            "header says {} module bytes, got {}",
            header.wasm_bytes,
            wasm.len()
        )));
    }
    Ok((header, wasm))
}

/// Decode the child's stdout strictly.
pub fn decode_child_reply(stdout: &str) -> Result<RunChildReply, RunSandboxError> {
    serde_json::from_str(stdout.trim()).map_err(|e| {
        tracing::warn!("run_sandbox: undecodable child reply: {e}");
        RunSandboxError::Child(format!("undecodable reply: {e}"))
    })
}

/// The server binary and its mapped shared objects (Linux).
#[cfg(target_os = "linux")]
pub fn current_run_image() -> Result<RunImage, RunSandboxError> {
    let exe =
        std::env::current_exe().map_err(|e| RunSandboxError::Image(format!("current_exe: {e}")))?;
    let maps = std::fs::read_to_string("/proc/self/maps")
        .map_err(|e| RunSandboxError::Image(format!("/proc/self/maps: {e}")))?;
    let image = build_sandbox::parse_mapped_objects(&maps, &exe).map_err(RunSandboxError::Image)?;
    tracing::debug!(?image, "run_sandbox: run image");
    Ok(image)
}

/// Not available off Linux (there is no bwrap).
#[cfg(not(target_os = "linux"))]
pub fn current_run_image() -> Result<RunImage, RunSandboxError> {
    Err(RunSandboxError::Unsupported)
}

/// `RUST_LOG` for the child, if the supervisor has one (never a `PNC_*`).
fn child_env() -> Vec<(&'static str, OsString)> {
    std::env::var_os("RUST_LOG")
        .map(|v| vec![("RUST_LOG", v)])
        .unwrap_or_default()
}

/// Wall-clock deadline the supervisor gives a run child.
pub const fn run_child_deadline(timeout_ms: u64) -> Duration {
    Duration::from_secs(
        MODULE_COMPILE_TIMEOUT.as_secs()
            + timeout_ms.div_ceil(1000)
            + RUN_TIMEOUT_MARGIN.as_secs()
            + RUN_CHILD_SLACK.as_secs(),
    )
}

fn relay_child_log(label: &str, pid: u32, stderr: &str) {
    if stderr.is_empty() {
        return;
    }
    let clean = proc_tree::strip_ansi(stderr);
    let shown: String = clean.chars().take(RUN_LOG_RELAY).collect();
    tracing::debug!(pid, "run_sandbox: {label} log:\n{shown}");
}

/// Run `bytes` in a fresh run sandbox (the `bwrap`-mode run phase). Same
/// contract as [`wasm_runner::run_wasm`]: `Err` only for setup failures.
pub fn run_wasm_sandboxed(
    bytes: &[u8],
    limits: &RunLimits,
    sandbox: &SandboxSettings,
) -> anyhow::Result<RunVerdict> {
    let image = current_run_image()?;
    let input = encode_run_input(bytes, limits)?;
    let spec = build_sandbox::run_bwrap_spec(sandbox, &image, RUN_WASM_SUBCOMMAND, &child_env());
    let deadline = run_child_deadline(limits.timeout_ms);
    tracing::debug!(
        argv = ?spec.argv_lossy(),
        ?deadline,
        bytes = bytes.len(),
        "run_sandbox: starting run child"
    );
    let out = proc_tree::run_captured_with_input(
        spec.to_command(),
        Some(input),
        deadline,
        RUN_REPLY_CAP.max(RUN_LOG_CAP),
        "wasm run",
    )?;
    relay_child_log("run child", out.pid, &out.stderr);
    if out.timed_out {
        tracing::warn!(
            pid = out.pid,
            ?deadline,
            "run_sandbox: run child hit its deadline; tree killed, graded as a timeout"
        );
        return Ok(RunVerdict {
            ok: false,
            stdout: String::new(),
            stderr: wasm_runner::timeout_message(limits.timeout_ms),
        });
    }
    if !out.status.success() {
        if let Some(sig) = killed_by_signal(&out.status) {
            // SIGKILL from the kernel (cgroup OOM) or a crash of the
            // runtime under the player's module: the program did not
            // finish, which is a runtime error, not a server fault.
            tracing::warn!(
                pid = out.pid,
                sig,
                "run_sandbox: run child killed by a signal"
            );
            return Ok(RunVerdict {
                ok: false,
                stdout: String::new(),
                stderr: format!(
                    "your program was stopped by the server (signal {sig}); it may have used \
                     too much memory"
                ),
            });
        }
        let tail: String = proc_tree::strip_ansi(&out.stderr)
            .chars()
            .rev()
            .take(2000)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        tracing::error!(pid = out.pid, status = ?out.status, "run_sandbox: run child failed: {tail}");
        return Err(RunSandboxError::Child(format!("exited with {:?}", out.status)).into());
    }
    match decode_child_reply(&out.stdout)? {
        RunChildReply::Verdict { ok, stdout, stderr } => Ok(RunVerdict { ok, stdout, stderr }),
        RunChildReply::SetupError(msg) => {
            tracing::warn!("run_sandbox: run child reported a setup error: {msg}");
            Err(RunSandboxError::Child(msg).into())
        }
    }
}

#[cfg(unix)]
fn killed_by_signal(status: &std::process::ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn killed_by_signal(_status: &std::process::ExitStatus) -> Option<i32> {
    None
}

fn write_stdout(bytes: &[u8]) -> ExitCode {
    let mut out = std::io::stdout().lock();
    match out.write_all(bytes).and_then(|()| out.flush()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("run_sandbox: could not write to stdout: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Entry point of `pledge_compile_api run-wasm` (inside the run sandbox):
/// read the header and module from stdin, run it with
/// [`wasm_runner::run_wasm`], write one [`RunChildReply`], exit.
pub fn run_wasm_main() -> ExitCode {
    // One-shot: a timed-out compile or a stuck run may be abandoned.
    wasm_runner::set_process_exits_after_job();
    let mut input = Vec::new();
    let read = std::io::stdin()
        .lock()
        .take(MAX_RUN_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut input);
    let reply = match read {
        Err(e) => RunChildReply::SetupError(format!("reading stdin: {e}")),
        Ok(_) => match decode_run_input(&input) {
            Err(e) => RunChildReply::SetupError(e.to_string()),
            Ok((h, wasm)) => {
                tracing::debug!(bytes = wasm.len(), ?h, "run-wasm: running module");
                match wasm_runner::run_wasm(wasm, h.timeout_ms, h.fuel, h.max_memory_bytes) {
                    Ok(v) => RunChildReply::Verdict {
                        ok: v.ok,
                        stdout: v.stdout,
                        stderr: v.stderr,
                    },
                    Err(e) => {
                        tracing::warn!("run-wasm: setup failure: {e:#}");
                        RunChildReply::SetupError(format!("{e:#}"))
                    }
                }
            }
        },
    };
    match serde_json::to_vec(&reply) {
        Ok(bytes) => write_stdout(&bytes),
        Err(e) => {
            tracing::error!("run-wasm: could not encode reply: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---- native probe -------------------------------------------------------------

/// What `verify-sandbox.sh` asks the probe to try.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeRequest {
    /// Host paths that must NOT exist inside the sandbox.
    pub paths: Vec<String>,
    /// A Unix socket to connect to (the builder socket).
    pub unix_socket: String,
    /// An outside address to connect to (must fail: no network).
    pub tcp_target: String,
    /// A loopback address with nothing listening: the positive control for
    /// the network check (the probe can make connections at all).
    pub tcp_control: String,
    /// List every file reachable from `/` (names only). Only meaningful,
    /// and only requested, inside the run sandbox.
    #[serde(default)]
    pub walk_root: bool,
}

impl ProbeRequest {
    fn validate(&self) -> Result<(), String> {
        if self.paths.len() > 64
            || self
                .paths
                .iter()
                .any(|p| !p.starts_with('/') || p.len() > 4096)
        {
            return Err("paths: at most 64 absolute paths".to_string());
        }
        if !self.unix_socket.starts_with('/') {
            return Err("unix_socket must be absolute".to_string());
        }
        Ok(())
    }
}

/// One probe result line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeLine {
    pub check: String,
    pub subject: String,
    pub result: String,
}

fn line(check: &str, subject: &str, result: impl Into<String>) -> ProbeLine {
    ProbeLine {
        check: check.to_string(),
        subject: subject.to_string(),
        result: result.into(),
    }
}

fn io_result(r: std::io::Result<()>) -> String {
    match r {
        Ok(()) => "present".to_string(),
        // EAFNOSUPPORT: the unit's RestrictAddressFamilies= refuses IP
        // sockets outright (seen in the [unit] pass).
        #[cfg(unix)]
        Err(e) if e.raw_os_error() == Some(rustix::io::Errno::AFNOSUPPORT.raw_os_error()) => {
            "af_refused".to_string()
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => "not_found".to_string(),
            std::io::ErrorKind::PermissionDenied => "denied".to_string(),
            std::io::ErrorKind::ConnectionRefused => "refused".to_string(),
            std::io::ErrorKind::NetworkUnreachable => "network_unreachable".to_string(),
            std::io::ErrorKind::HostUnreachable => "host_unreachable".to_string(),
            std::io::ErrorKind::TimedOut => "timed_out".to_string(),
            _ => format!("error: {e}"),
        },
    }
}

fn tcp_result(target: &str) -> String {
    use std::net::{SocketAddr, TcpStream};
    match target.parse::<SocketAddr>() {
        Err(e) => format!("error: bad address: {e}"),
        Ok(addr) => match TcpStream::connect_timeout(&addr, PROBE_TCP_TIMEOUT) {
            Ok(_) => "connected".to_string(),
            Err(e) => io_result(Err(e)),
        },
    }
}

/// Most entries [`walk`] lists, and how deep it goes.
const WALK_MAX_ENTRIES: usize = 256;
const WALK_MAX_DEPTH: usize = 8;

/// Collect the paths of non-directory entries below `dir` (no symlink is
/// followed; unreadable directories are listed as `<path>/?`).
fn walk(dir: &std::path::Path, depth: usize, out: &mut Vec<String>) {
    if depth > WALK_MAX_DEPTH || out.len() >= WALK_MAX_ENTRIES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        out.push(format!("{}/?", dir.display()));
        return;
    };
    for e in entries.flatten() {
        if out.len() >= WALK_MAX_ENTRIES {
            out.push("...".to_string());
            return;
        }
        let path = e.path();
        match e.file_type() {
            Ok(t) if t.is_dir() => walk(&path, depth + 1, out),
            _ => out.push(path.display().to_string()),
        }
    }
}

/// Run every probe of `req` in THIS process (whatever sandbox it is in).
pub fn probe(req: &ProbeRequest) -> Vec<ProbeLine> {
    let mut out = vec![line("stdin", "request", "ok")];
    // Positive controls: the probe's own binary is inside the sandbox.
    out.push(line(
        "self",
        SANDBOX_RUN_EXE,
        io_result(std::fs::symlink_metadata(SANDBOX_RUN_EXE).map(|_| ())),
    ));
    for p in &req.paths {
        out.push(line(
            "path",
            p,
            io_result(std::fs::symlink_metadata(p).map(|_| ())),
        ));
    }
    #[cfg(unix)]
    out.push(line(
        "unix",
        &req.unix_socket,
        io_result(std::os::unix::net::UnixStream::connect(&req.unix_socket).map(|_| ())),
    ));
    out.push(line("tcp", &req.tcp_target, tcp_result(&req.tcp_target)));
    out.push(line(
        "tcp-control",
        &req.tcp_control,
        tcp_result(&req.tcp_control),
    ));
    out.push(line("pid", "self", std::process::id().to_string()));
    // Every non-directory entry reachable from `/` (names only, bounded):
    // inside the run sandbox this must be exactly the binary and its
    // libraries. Not run outside a sandbox (it would walk the host).
    if req.walk_root {
        let mut files = Vec::new();
        walk("/".as_ref(), 0, &mut files);
        files.sort();
        out.push(line("tree", "/", files.join(",")));
    }
    let mut keys: Vec<String> = std::env::vars_os()
        .map(|(k, _)| k.to_string_lossy().into_owned())
        .collect();
    keys.sort();
    out.push(line("env", "keys", keys.join(",")));
    #[cfg(target_os = "linux")]
    {
        use rustix::process::{Resource, getrlimit};
        for (name, res) in [
            ("cpu", Resource::Cpu),
            ("nofile", Resource::Nofile),
            ("fsize", Resource::Fsize),
            ("core", Resource::Core),
        ] {
            let l = getrlimit(res);
            let show = |v: Option<u64>| v.map_or("unlimited".to_string(), |n| n.to_string());
            out.push(line(
                "rlimit",
                name,
                format!("{}/{}", show(l.current), show(l.maximum)),
            ));
        }
        out.push(line(
            "uid",
            "self",
            rustix::process::getuid().as_raw().to_string(),
        ));
    }
    out
}

/// Entry point of `pledge_compile_api run-probe`: read a [`ProbeRequest`]
/// from stdin, print one JSON [`ProbeLine`] per line. It reports only
/// whether things exist or connect, never file contents.
pub fn run_probe_main() -> ExitCode {
    let mut input = Vec::new();
    if let Err(e) = std::io::stdin()
        .lock()
        .take(MAX_PROBE_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut input)
    {
        tracing::error!("run-probe: reading stdin: {e}");
        return ExitCode::FAILURE;
    }
    let req = match decode_probe_request(&input) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("run-probe: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut text = String::new();
    for l in probe(&req) {
        match serde_json::to_string(&l) {
            Ok(s) => {
                text.push_str(&s);
                text.push('\n');
            }
            Err(e) => {
                tracing::error!("run-probe: encode: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    write_stdout(text.as_bytes())
}

/// Decode and validate a probe request.
pub fn decode_probe_request(input: &[u8]) -> Result<ProbeRequest, RunSandboxError> {
    if input.len() > MAX_PROBE_REQUEST_BYTES {
        return Err(RunSandboxError::BadInput(
            "probe request too large".to_string(),
        ));
    }
    let req: ProbeRequest = serde_json::from_slice(input)
        .map_err(|e| RunSandboxError::BadInput(format!("probe request: {e}")))?;
    req.validate().map_err(RunSandboxError::BadInput)?;
    Ok(req)
}

/// Run the native probe inside the run sandbox (as `sandbox-verify-run`
/// does) and return its stdout lines.
pub fn run_probe_sandboxed(
    sandbox: &SandboxSettings,
    request: &ProbeRequest,
) -> anyhow::Result<String> {
    let image = current_run_image()?;
    let input = serde_json::to_vec(request)?;
    let spec = build_sandbox::run_bwrap_spec(sandbox, &image, RUN_PROBE_SUBCOMMAND, &child_env());
    tracing::debug!(argv = ?spec.argv_lossy(), "run_sandbox: starting probe child");
    let out = proc_tree::run_captured_with_input(
        spec.to_command(),
        Some(input),
        PROBE_DEADLINE,
        RUN_REPLY_CAP,
        "run probe",
    )?;
    relay_child_log("probe child", out.pid, &out.stderr);
    if out.timed_out || !out.status.success() {
        anyhow::bail!(
            "probe child failed (timed_out={}, status={:?}): {}",
            out.timed_out,
            out.status,
            proc_tree::strip_ansi(&out.stderr)
        );
    }
    Ok(out.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> RunLimits {
        RunLimits::default()
    }

    #[test]
    fn run_input_round_trips_and_is_exact() {
        let wasm = b"\0asm\x01\0\0\0".to_vec();
        let enc = encode_run_input(&wasm, &limits()).unwrap();
        let (h, body) = decode_run_input(&enc).unwrap();
        assert_eq!(body, &wasm[..]);
        assert_eq!(h.timeout_ms, RUN_TIMEOUT_MS);
        assert_eq!(h.fuel, RUN_FUEL);
        // Trailing or missing bytes are refused.
        let mut longer = enc.clone();
        longer.push(0);
        assert!(decode_run_input(&longer).is_err());
        assert!(decode_run_input(&enc[..enc.len() - 1]).is_err());
        // No header line at all.
        assert!(decode_run_input(&wasm).is_err());
        // Unknown header field.
        let tampered = String::from_utf8_lossy(&enc)
            .replacen("{\"timeout_ms\"", "{\"x\":1,\"timeout_ms\"", 1)
            .into_bytes();
        assert!(decode_run_input(&tampered).is_err());
    }

    #[test]
    fn child_refuses_limits_looser_than_shipped() {
        let base = RunHeader {
            timeout_ms: RUN_TIMEOUT_MS,
            fuel: RUN_FUEL,
            max_memory_bytes: MAX_RUN_MAX_MEMORY_BYTES,
            wasm_bytes: 8,
        };
        assert!(base.validate().is_ok());
        for bad in [
            RunHeader {
                timeout_ms: RUN_TIMEOUT_MS + 1,
                ..base
            },
            RunHeader {
                timeout_ms: 0,
                ..base
            },
            RunHeader {
                fuel: RUN_FUEL + 1,
                ..base
            },
            RunHeader {
                max_memory_bytes: MAX_RUN_MAX_MEMORY_BYTES + 1,
                ..base
            },
            RunHeader {
                wasm_bytes: MAX_WASM_BYTES as u64 + 1,
                ..base
            },
            RunHeader {
                wasm_bytes: 0,
                ..base
            },
        ] {
            assert!(bad.validate().is_err(), "{bad:?}");
        }
        let too_big = vec![0u8; MAX_WASM_BYTES + 1];
        assert!(encode_run_input(&too_big, &limits()).is_err());
    }

    #[test]
    fn child_reply_decoding_is_strict() {
        let v = RunChildReply::Verdict {
            ok: true,
            stdout: "hi\n".to_string(),
            stderr: String::new(),
        };
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(decode_child_reply(&s).unwrap(), v);
        assert!(
            decode_child_reply(r#"{"verdict":{"ok":true,"stdout":"","stderr":"","x":1}}"#).is_err()
        );
        assert!(decode_child_reply("garbage").is_err());
        assert!(matches!(
            decode_child_reply(r#"{"setup_error":"bad module"}"#),
            Ok(RunChildReply::SetupError(_))
        ));
    }

    #[test]
    fn child_deadline_fits_inside_the_job_budget() {
        let d = run_child_deadline(RUN_TIMEOUT_MS);
        assert!(
            d >= MODULE_COMPILE_TIMEOUT
                + Duration::from_millis(RUN_TIMEOUT_MS)
                + RUN_TIMEOUT_MARGIN
        );
        assert!(
            crate::wasm_builder::CARGO_BUILD_TIMEOUT + d <= crate::jobs::JOB_DEADLINE,
            "{d:?}"
        );
    }

    #[test]
    fn probe_request_is_validated_and_probe_reports_without_contents() {
        let req = ProbeRequest {
            paths: vec!["/definitely/not/here/pnc".to_string()],
            unix_socket: "/definitely/not/here/pnc.sock".to_string(),
            tcp_target: "not-an-address".to_string(),
            tcp_control: "not-an-address".to_string(),
            walk_root: false,
        };
        let enc = serde_json::to_vec(&req).unwrap();
        assert_eq!(decode_probe_request(&enc).unwrap(), req);
        assert!(
            decode_probe_request(
                br#"{"paths":["relative"],"unix_socket":"/x","tcp_target":"","tcp_control":""}"#
            )
            .is_err()
        );
        let lines = probe(&req);
        assert!(lines.contains(&line("stdin", "request", "ok")));
        assert!(lines.contains(&line("path", "/definitely/not/here/pnc", "not_found")));
        assert!(
            lines
                .iter()
                .any(|l| l.check == "tcp" && l.result.starts_with("error: bad address"))
        );
        // Field order is what the verify script greps for.
        let s = serde_json::to_string(&line("path", "/x", "not_found")).unwrap();
        assert_eq!(s, r#"{"check":"path","subject":"/x","result":"not_found"}"#);
    }
}
