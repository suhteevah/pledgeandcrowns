// SPDX-License-Identifier: AGPL-3.0-or-later
//! One process per job (security re-review P1-C, original P1-5).
//!
//! The builder never builds or runs player code in its own long-lived
//! process. For every request it starts `pledge_compile_api job` (this
//! same binary), writes a [`JobEnvelope`] (build settings + the validated
//! request) to the child's stdin, and reads one JSON [`JobReply`] from its
//! stdout. The child does the cargo build (inside bwrap) AND the wasmtime
//! run, then exits.
//!
//! The child is supervised by [`crate::proc_tree`]: it leads its own
//! process group and the whole group is SIGKILLed at [`JOB_DEADLINE`]
//! (sum of the build, module-compile and run budgets plus slack). So a
//! timed-out build, a runaway Cranelift compile or a stuck run ends by
//! killing processes, never by abandoning a thread, and the caller's job
//! slot is released only once the group is confirmed empty. If the group
//! cannot be emptied, the caller gets [`ProcError::Survivors`] and must
//! not release the slot (the builder exits so systemd kills its cgroup).
//!
//! The child's environment is cleared down to the build allow-list
//! (`PATH`, `CARGO_HOME`, `RUSTUP_HOME`, temp dirs, `RUST_LOG`): never a
//! `PNC_*` variable. Its logs go to stderr (captured, capped, re-logged
//! at debug); stdout carries only the reply.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::build_sandbox::HOST_ENV_ALLOWLIST;
use crate::expectations::{self, GradeState, GradedRun};
use crate::ipc::{
    JobReply, JobRequest, MAX_REPLY_FRAME_BYTES, MAX_REQUEST_FRAME_BYTES, WireGradedRun,
};
use crate::proc_tree::{self, ProcError};
use crate::wasm_builder::{BuildSettings, CARGO_BUILD_TIMEOUT, RUN_TIMEOUT_MS, RunLimits};
use crate::wasm_runner::{self, JOB_RUN_PHASE_MARKER, MODULE_COMPILE_TIMEOUT, RUN_TIMEOUT_MARGIN};

/// Subcommand that runs one job (stdin -> stdout).
pub const JOB_SUBCOMMAND: &str = "job";

/// Hard wall-clock budget for one job process tree: build + module
/// compile + run budgets + 5 s slack. The whole group is killed at this
/// deadline.
pub const JOB_DEADLINE: Duration = Duration::from_secs(50);
const _: () = assert!(
    JOB_DEADLINE.as_secs()
        >= CARGO_BUILD_TIMEOUT.as_secs()
            + MODULE_COMPILE_TIMEOUT.as_secs()
            + RUN_TIMEOUT_MS / 1000
            + RUN_TIMEOUT_MARGIN.as_secs()
);

/// Cap on captured child stdout/stderr (each).
pub const JOB_OUTPUT_CAP: usize = MAX_REPLY_FRAME_BYTES;

/// What the builder sends a job process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobEnvelope {
    pub settings: BuildSettings,
    pub request: JobRequest,
}

impl JobEnvelope {
    pub fn validate(&self) -> Result<(), String> {
        self.settings.validate()?;
        self.request.validate().map_err(|e| e.to_string())
    }
}

/// Failures of the job mechanism itself (not of the player's code).
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error(transparent)]
    Proc(#[from] ProcError),
    #[error("job process returned an unusable reply: {0}")]
    BadReply(String),
    #[error("could not encode the job: {0}")]
    Encode(String),
}

impl JobError {
    pub fn is_survivors(&self) -> bool {
        matches!(self, Self::Proc(p) if p.is_survivors())
    }
}

/// Errors surfaced to the HTTP handler from a remote/subprocess grader.
#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    /// The builder has no free job slot (maps to 503 + Retry-After).
    #[error("builder is busy")]
    Busy,
    #[error("builder rejected the request: {0}")]
    Rejected(String),
    #[error("builder failed: {0}")]
    Failed(String),
}

/// Result of one supervised job.
#[derive(Debug)]
pub struct JobRun {
    pub reply: JobReply,
    /// PID of the job process (for logs and tests).
    pub pid: u32,
    /// The deadline fired and the job's process tree was killed.
    pub timed_out: bool,
    pub elapsed: Duration,
}

/// The graded run reported when a job is killed at its deadline while it
/// was still building (or before it said otherwise).
pub fn deadline_graded(deadline: Duration) -> WireGradedRun {
    WireGradedRun {
        compiled: false,
        ran_ok: false,
        stdout: String::new(),
        stderr: format!(
            "build and run exceeded the {}s job budget and were stopped",
            deadline.as_secs()
        ),
        build_ms: u64::try_from(deadline.as_millis()).unwrap_or(u64::MAX),
        run_ms: None,
        state: GradeState::CompileError,
        expected: None,
        actual: None,
        note: "stopped at the job deadline".to_string(),
    }
}

/// The graded run reported when a job is killed at its deadline after its
/// program had compiled and started running ([`JOB_RUN_PHASE_MARKER`]
/// seen). The run has its own timeout, so this should not happen; if it
/// does, the honest grade is a runtime error.
pub fn deadline_graded_while_running(deadline: Duration) -> WireGradedRun {
    WireGradedRun {
        compiled: true,
        ran_ok: false,
        stdout: String::new(),
        stderr: format!(
            "your program built, but building and running it exceeded the {}s job budget and \
             it was stopped",
            deadline.as_secs()
        ),
        build_ms: 0,
        run_ms: Some(u64::try_from(deadline.as_millis()).unwrap_or(u64::MAX)),
        state: GradeState::RuntimeError,
        expected: None,
        actual: None,
        note: "stopped at the job deadline while running".to_string(),
    }
}

/// Run one job in a fresh `<exe> job` process, killing its whole process
/// tree at `deadline`. Returns only once that tree is confirmed dead.
pub fn run_job_subprocess(
    exe: &Path,
    envelope: &JobEnvelope,
    deadline: Duration,
) -> Result<JobRun, JobError> {
    let input = serde_json::to_vec(envelope).map_err(|e| JobError::Encode(e.to_string()))?;
    let mut cmd = Command::new(exe);
    cmd.arg(JOB_SUBCOMMAND).env_clear();
    for k in HOST_ENV_ALLOWLIST.iter().chain(["RUST_LOG"].iter()) {
        if let Some(v) = std::env::var_os(k) {
            cmd.env(k, v);
        }
    }
    tracing::debug!(
        exe = %exe.display(),
        encounter = %envelope.request.encounter_id,
        ?deadline,
        "jobs: starting job process"
    );
    let out =
        proc_tree::run_captured_with_input(cmd, Some(input), deadline, JOB_OUTPUT_CAP, "job")?;
    if !out.stderr.is_empty() {
        let shown: String = out.stderr.chars().take(16 * 1024).collect();
        tracing::debug!(pid = out.pid, "jobs: job process log:\n{shown}");
    }
    if out.timed_out {
        let running = out.stderr.lines().any(|l| l.trim() == JOB_RUN_PHASE_MARKER);
        tracing::warn!(
            pid = out.pid,
            ?deadline,
            running,
            "jobs: job hit its deadline; process tree killed"
        );
        let graded = if running {
            deadline_graded_while_running(deadline)
        } else {
            deadline_graded(deadline)
        };
        return Ok(JobRun {
            reply: JobReply::Graded(graded),
            pid: out.pid,
            timed_out: true,
            elapsed: out.elapsed,
        });
    }
    if !out.status.success() {
        tracing::error!(pid = out.pid, status = ?out.status, "jobs: job process failed");
        return Err(JobError::BadReply(format!(
            "job process exited with {:?}",
            out.status
        )));
    }
    let reply = crate::ipc::decode_reply(out.stdout.trim().as_bytes())
        .map_err(|e| JobError::BadReply(e.to_string()))?;
    tracing::debug!(
        pid = out.pid,
        kind = reply.kind(),
        elapsed = ?out.elapsed,
        "jobs: job finished"
    );
    Ok(JobRun {
        reply,
        pid: out.pid,
        timed_out: false,
        elapsed: out.elapsed,
    })
}

/// Map a reply to the handler's result type.
pub fn reply_to_graded(reply: JobReply) -> anyhow::Result<GradedRun> {
    match reply {
        JobReply::Graded(run) => Ok(run.into_graded()),
        JobReply::Busy => Err(ExecError::Busy.into()),
        JobReply::Rejected(m) => Err(ExecError::Rejected(m).into()),
        JobReply::Failed(m) => Err(ExecError::Failed(m).into()),
    }
}

/// Grade through a per-job subprocess (the `Executor::Subprocess` path).
pub fn grade_via_subprocess(
    exe: &Path,
    settings: &BuildSettings,
    request: JobRequest,
    deadline: Duration,
) -> anyhow::Result<GradedRun> {
    let envelope = JobEnvelope {
        settings: settings.clone(),
        request,
    };
    let run = run_job_subprocess(exe, &envelope, deadline)?;
    reply_to_graded(run.reply)
}

/// Grade in this process (the job process itself, and the in-process
/// dev executor).
pub fn grade_request(settings: &BuildSettings, req: &JobRequest) -> JobReply {
    let limits = RunLimits {
        max_memory_bytes: req.run_max_memory_bytes,
        ..RunLimits::default()
    };
    let reply = match expectations::grade_in(&req.encounter_id, &req.source, &limits, settings) {
        Ok(graded) => JobReply::Graded(WireGradedRun::from(graded)),
        Err(e) => {
            tracing::error!("jobs: grading setup failure: {e:#}");
            JobReply::Failed(format!("{e:#}"))
        }
    };
    reply.clamp()
}

/// Entry point of `pledge_compile_api job`: read one envelope from stdin
/// (capped), validate it, grade, write one reply to stdout, exit.
pub fn job_main() -> ExitCode {
    // This process exits right after replying, so a timed-out wasm
    // compile may be abandoned (see `wasm_runner`).
    wasm_runner::set_process_exits_after_job();
    let reply = read_envelope().map_or_else(
        |msg| {
            tracing::warn!("job: rejected envelope: {msg}");
            JobReply::Rejected(msg)
        },
        |env| {
            tracing::info!(encounter = %env.request.encounter_id, "job: grading");
            grade_request(&env.settings, &env.request)
        },
    );
    let mut out = std::io::stdout().lock();
    let written = serde_json::to_writer(&mut out, &reply)
        .map_err(|e| e.to_string())
        .and_then(|()| out.flush().map_err(|e| e.to_string()));
    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("job: could not write reply: {e}");
            ExitCode::FAILURE
        }
    }
}

fn read_envelope() -> Result<JobEnvelope, String> {
    let mut buf = Vec::new();
    std::io::stdin()
        .lock()
        .take(MAX_REQUEST_FRAME_BYTES as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|e| format!("reading stdin: {e}"))?;
    if buf.len() > MAX_REQUEST_FRAME_BYTES {
        return Err(format!("envelope over {MAX_REQUEST_FRAME_BYTES} bytes"));
    }
    let env: JobEnvelope = serde_json::from_slice(&buf).map_err(|e| format!("decode: {e}"))?;
    env.validate()?;
    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_round_trips_and_is_strict() {
        let env = JobEnvelope {
            settings: BuildSettings::default(),
            request: JobRequest {
                encounter_id: "intro_let_binding".to_string(),
                source: "fn main() {}".to_string(),
                run_max_memory_bytes: 16 << 20,
            },
        };
        let json = serde_json::to_string(&env).unwrap();
        let back: JobEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, env);
        assert!(back.validate().is_ok());
        let tampered = json.replacen(
            "\"source_denylist\":true",
            "\"source_denylist\":true,\"x\":1",
            1,
        );
        assert!(serde_json::from_str::<JobEnvelope>(&tampered).is_err());
    }

    #[test]
    fn deadline_verdict_is_consistent() {
        let g = deadline_graded(JOB_DEADLINE);
        assert!(g.validate().is_ok());
        assert!(!g.into_graded().ok());
        let r = deadline_graded_while_running(JOB_DEADLINE);
        assert!(r.validate().is_ok(), "{:?}", r.validate());
        assert_eq!(r.state, GradeState::RuntimeError);
        assert!(r.compiled && !r.ran_ok);
        assert!(r.stderr.contains("job budget"));
        assert!(!r.into_graded().ok());
    }
}
