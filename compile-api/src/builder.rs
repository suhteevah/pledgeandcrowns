// SPDX-License-Identifier: AGPL-3.0-or-later
//! The builder service (`pledge_compile_api builder`, unit
//! `pnc-builder.service`): the only place player code is built and run.
//!
//! - Holds **no secrets**: it refuses to start if `PNC_AUTH_TOKENS` or
//!   `PNC_METRICS_TOKENS` is set ([`ConfigError::BuilderHoldsSecrets`]).
//! - Listens on a Unix socket (see [`crate::ipc`] for the protocol and
//!   why), one request per connection, with a cap on concurrent
//!   connections and a total deadline on reading the request.
//! - Treats the API server as untrusted: every request is decoded strictly
//!   and validated, and the source deny-list is applied again here
//!   (`source_denylist` is always on in the builder).
//! - Runs each job in its own process ([`crate::jobs`]) while holding a
//!   job slot ([`JobSlots`]). The slot is released only after the job's
//!   process tree is confirmed dead. If it cannot be confirmed
//!   ([`crate::proc_tree::ProcError::Survivors`]) the slot is retired, not
//!   released, and the builder exits non-zero so systemd kills every
//!   process left in its cgroup and restarts it (escalation).
//!
//! `sandbox-verify` (used by `deploy/verify-sandbox.sh`) is also here: a
//! one-shot CLI that builds a probe program from stdin with the deny-list
//! OFF, under the builder's sandbox settings, to prove the sandbox itself
//! blocks file reads. It is not reachable over any socket, refuses to run
//! with a token in its environment, and refuses `PNC_BUILD_SANDBOX=none`.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::build_sandbox::SandboxMode;
use crate::config::{self, ConfigError, vars};
use crate::ipc::{
    self, IpcError, IpcStream, JobReply, JobRequest, MAX_REPLY_FRAME_BYTES,
    MAX_REQUEST_FRAME_BYTES, REPLY_WRITE_DEADLINE, REQUEST_IO_DEADLINE,
};
use crate::jobs::{self, JOB_DEADLINE, JobEnvelope, JobError, JobRun};
use crate::wasm_builder::{self, BuildSettings, DEFAULT_RUN_MAX_MEMORY_BYTES, RunLimits};

/// Subcommand that runs the builder service.
pub const BUILDER_SUBCOMMAND: &str = "builder";
/// Subcommand for `deploy/verify-sandbox.sh`.
pub const SANDBOX_VERIFY_SUBCOMMAND: &str = "sandbox-verify";
/// Exit code when a job's process tree could not be killed.
pub const EXIT_SURVIVORS: u8 = 70;
/// Default concurrent jobs (matches the API's default build slots).
pub const DEFAULT_MAX_JOBS: usize = 2;
const MAX_JOBS_CAP: usize = 64;
/// Concurrent connections allowed per job slot (the rest wait in the
/// API's own queue; extra connections are closed at once).
pub const CONNECTIONS_PER_JOB: usize = 4;

/// Builder configuration (environment, no secrets).
#[derive(Debug, Clone)]
pub struct BuilderConfig {
    pub socket: PathBuf,
    pub settings: BuildSettings,
    pub max_jobs: usize,
    pub dev_unsafe_local_builds: bool,
}

impl BuilderConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    /// Parse and validate. Refuses any token variable, `none` without the
    /// dev override, and `PNC_SOURCE_DENYLIST=false`.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |k: &str| {
            lookup(k)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        for var in [vars::AUTH_TOKENS, vars::METRICS_TOKENS] {
            if get(var).is_some() {
                tracing::error!(var, "builder: refusing to start with a token variable set");
                return Err(ConfigError::BuilderHoldsSecrets { var });
            }
        }
        config::reject_removed_override(&get)?;
        let dev_unsafe_local_builds = config::parse_dev_override(&get)?;
        let sandbox = config::parse_sandbox(&get)?;
        if let Some(v) = get(vars::SOURCE_DENYLIST)
            && !config::parse_bool(vars::SOURCE_DENYLIST, &v)?
        {
            return Err(ConfigError::Invalid {
                var: vars::SOURCE_DENYLIST,
                value: v,
                reason: "the builder always applies the deny-list".to_string(),
            });
        }
        let socket = match get(vars::BUILDER_SOCKET) {
            None => PathBuf::from(config::DEFAULT_BUILDER_SOCKET),
            Some(v) => config::parse_socket_path(&v)?,
        };
        let root = get(vars::BUILD_ROOT)
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::build_root::default_root(config::DEFAULT_BUILD_ROOT_NAME));
        let max_jobs = config::parse_ranged(
            vars::BUILDER_MAX_JOBS,
            get(vars::BUILDER_MAX_JOBS),
            DEFAULT_MAX_JOBS,
            1,
            MAX_JOBS_CAP,
        )?;
        let cfg = Self {
            socket,
            settings: BuildSettings {
                root,
                sandbox,
                source_denylist: true,
            },
            max_jobs,
            dev_unsafe_local_builds,
        };
        cfg.validate_for(cfg!(target_os = "linux"))?;
        Ok(cfg)
    }

    /// Cross-field rules, platform explicit for tests.
    pub fn validate_for(&self, linux: bool) -> Result<(), ConfigError> {
        match self.settings.sandbox.mode {
            SandboxMode::Bwrap => config::check_bwrap(&self.settings.sandbox, linux),
            SandboxMode::None if self.dev_unsafe_local_builds => Ok(()),
            SandboxMode::None => Err(ConfigError::UnsandboxedExposure {
                reason: "the builder is the production build service".to_string(),
            }),
        }
    }
}

/// Counting semaphore for job slots, held across the whole job.
#[derive(Debug)]
pub struct JobSlots {
    max: usize,
    used: Mutex<usize>,
}

/// A held job slot. Dropping it releases the slot; [`SlotGuard::retire`]
/// keeps it consumed forever (survivors).
#[derive(Debug)]
pub struct SlotGuard {
    slots: Arc<JobSlots>,
    live: bool,
}

impl JobSlots {
    pub fn new(max: usize) -> Arc<Self> {
        Arc::new(Self {
            max,
            used: Mutex::new(0),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, usize> {
        self.used.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn try_acquire(self: &Arc<Self>) -> Option<SlotGuard> {
        let mut used = self.lock();
        if *used >= self.max {
            return None;
        }
        *used += 1;
        Some(SlotGuard {
            slots: Arc::clone(self),
            live: true,
        })
    }

    pub fn available(&self) -> usize {
        self.max.saturating_sub(*self.lock())
    }

    pub fn max(&self) -> usize {
        self.max
    }
}

impl SlotGuard {
    /// Never give this slot back: processes of its job may still be
    /// alive, so the capacity they use must stay accounted for.
    pub fn retire(mut self) {
        self.live = false;
        tracing::error!(
            available = self.slots.available(),
            "builder: job slot retired (job processes may survive)"
        );
    }
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        if self.live {
            let mut used = self.slots.lock();
            *used = used.saturating_sub(1);
        }
    }
}

/// Runs one job (production: [`jobs::run_job_subprocess`]).
pub type JobRunner = Arc<dyn Fn(&JobEnvelope) -> Result<JobRun, JobError> + Send + Sync>;

/// Everything a connection handler needs.
#[derive(Clone)]
pub struct BuilderCtx {
    pub settings: BuildSettings,
    pub slots: Arc<JobSlots>,
    pub runner: JobRunner,
    /// Called after a slot is retired for survivors. Production: log and
    /// exit [`EXIT_SURVIVORS`].
    pub on_survivors: Arc<dyn Fn() + Send + Sync>,
    pub request_deadline: Duration,
    pub reply_deadline: Duration,
}

impl BuilderCtx {
    /// Production context: subprocess jobs via `exe`, exit on survivors.
    pub fn production(settings: BuildSettings, max_jobs: usize, exe: PathBuf) -> Self {
        Self {
            settings,
            slots: JobSlots::new(max_jobs),
            runner: Arc::new(move |env| jobs::run_job_subprocess(&exe, env, JOB_DEADLINE)),
            on_survivors: Arc::new(|| {
                tracing::error!(
                    "builder: a job's processes survived SIGKILL; exiting so systemd kills the \
                     whole cgroup and restarts the builder"
                );
                std::process::exit(i32::from(EXIT_SURVIVORS));
            }),
            request_deadline: REQUEST_IO_DEADLINE,
            reply_deadline: REPLY_WRITE_DEADLINE,
        }
    }
}

/// What happened on one connection (for logs and tests).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnOutcome {
    /// A reply of this kind was sent.
    Replied(&'static str),
    /// The client broke the protocol or timed out; nothing (more) sent.
    ClientError(String),
    /// A job left survivors; its slot was retired.
    Survivors,
}

fn send<S: IpcStream>(stream: &mut S, reply: &JobReply, ctx: &BuilderCtx) -> ConnOutcome {
    let kind = reply.kind();
    let payload = match serde_json::to_vec(&reply.clone().clamp()) {
        Ok(p) => p,
        Err(e) => return ConnOutcome::ClientError(format!("encode reply: {e}")),
    };
    match ipc::write_frame(
        stream,
        &payload,
        MAX_REPLY_FRAME_BYTES,
        Instant::now() + ctx.reply_deadline,
        "reply write",
    ) {
        Ok(()) => ConnOutcome::Replied(kind),
        Err(e) => {
            tracing::warn!("builder: could not send {kind} reply: {e}");
            ConnOutcome::ClientError(e.to_string())
        }
    }
}

/// Serve one connection: read one request (bounded), validate it, run the
/// job while holding a slot, send one reply.
pub fn handle_connection<S: IpcStream>(stream: &mut S, ctx: &BuilderCtx) -> ConnOutcome {
    let payload = match ipc::read_frame(
        stream,
        MAX_REQUEST_FRAME_BYTES,
        Instant::now() + ctx.request_deadline,
        "request read",
    ) {
        Ok(p) => p,
        Err(e @ (IpcError::FrameTooLarge { .. } | IpcError::BadMagic)) => {
            tracing::warn!("builder: rejected request frame: {e}");
            let _ = send(stream, &JobReply::Rejected(e.to_string()), ctx);
            return ConnOutcome::ClientError(e.to_string());
        }
        Err(e) => {
            tracing::warn!("builder: request not received: {e}");
            return ConnOutcome::ClientError(e.to_string());
        }
    };
    let request: JobRequest = match ipc::decode_request(&payload) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("builder: rejected request: {e}");
            return send(stream, &JobReply::Rejected(e.to_string()), ctx);
        }
    };
    let Some(slot) = ctx.slots.try_acquire() else {
        tracing::warn!(max = ctx.slots.max(), "builder: all job slots busy");
        return send(stream, &JobReply::Busy, ctx);
    };
    tracing::info!(
        encounter = %request.encounter_id,
        source_bytes = request.source.len(),
        free_slots = ctx.slots.available(),
        "builder: job accepted"
    );
    let envelope = JobEnvelope {
        settings: ctx.settings.clone(),
        request,
    };
    let reply = match (ctx.runner)(&envelope) {
        Ok(run) => {
            tracing::info!(
                pid = run.pid,
                kind = run.reply.kind(),
                timed_out = run.timed_out,
                elapsed_ms = u64::try_from(run.elapsed.as_millis()).unwrap_or(u64::MAX),
                "builder: job done"
            );
            run.reply
        }
        Err(e) if e.is_survivors() => {
            tracing::error!("builder: {e}");
            slot.retire();
            (ctx.on_survivors)();
            return ConnOutcome::Survivors;
        }
        Err(e) => {
            tracing::error!("builder: job failed: {e}");
            JobReply::Failed(e.to_string())
        }
    };
    // The job's process tree is confirmed dead: release before replying.
    drop(slot);
    send(stream, &reply, ctx)
}

/// Builder startup self-test: toolchain pin, sandbox isolation probe,
/// then a trivial program built AND run through the real job-process
/// path.
pub fn self_test(ctx: &BuilderCtx) -> anyhow::Result<()> {
    wasm_builder::check_toolchain(&ctx.settings.sandbox)?;
    if ctx.settings.sandbox.mode == SandboxMode::Bwrap {
        crate::build_sandbox::probe_isolation(&ctx.settings.sandbox, &ctx.settings.root)?;
    }
    let envelope = JobEnvelope {
        settings: ctx.settings.clone(),
        request: JobRequest {
            encounter_id: "pnc_self_test".to_string(),
            source: "fn main() { println!(\"pnc-self-test\"); }".to_string(),
            run_max_memory_bytes: DEFAULT_RUN_MAX_MEMORY_BYTES,
        },
    };
    let run = (ctx.runner)(&envelope)?;
    match run.reply {
        JobReply::Graded(g) if g.compiled && g.ran_ok && g.stdout.contains("pnc-self-test") => {
            tracing::info!(
                sandbox = %ctx.settings.sandbox.mode,
                build_ms = g.build_ms,
                "build self-test passed (job process: build + run)"
            );
            Ok(())
        }
        other => Err(anyhow::anyhow!("build self-test failed: {other:?}")),
    }
}

/// Run the builder service until it fails.
#[cfg(unix)]
pub fn run_builder(cfg: BuilderConfig) -> anyhow::Result<()> {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};
    use std::os::unix::net::UnixListener;
    use std::sync::atomic::{AtomicUsize, Ordering};

    tracing::info!("builder: config {cfg:?}");
    crate::build_root::prepare(&cfg.settings.root)?;
    match crate::build_root::sweep_stale(&cfg.settings.root, crate::build_root::STALE_JOB_AGE) {
        Ok(n) => tracing::info!("builder: swept {n} stale job dir(s)"),
        Err(e) => tracing::warn!("builder: build root sweep failed: {e}"),
    }
    let exe = std::env::current_exe()?;
    let ctx = BuilderCtx::production(cfg.settings.clone(), cfg.max_jobs, exe);
    if let Err(e) = self_test(&ctx) {
        tracing::error!("refusing to start: builder self-test failed: {e:#}");
        return Err(e);
    }

    // Replace a stale socket from a previous run; refuse anything else.
    match std::fs::symlink_metadata(&cfg.socket) {
        Ok(m) if m.file_type().is_socket() => std::fs::remove_file(&cfg.socket)?,
        Ok(_) => anyhow::bail!(
            "{} exists and is not a socket; refusing to replace it",
            cfg.socket.display()
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let listener = UnixListener::bind(&cfg.socket)?;
    // Created under UMask=0077 (0600), then opened to the pnc-ipc group.
    std::fs::set_permissions(&cfg.socket, std::fs::Permissions::from_mode(0o660))?;
    tracing::info!(
        socket = %cfg.socket.display(),
        max_jobs = cfg.max_jobs,
        "builder: listening"
    );

    let max_conns = cfg.max_jobs * CONNECTIONS_PER_JOB;
    let active = Arc::new(AtomicUsize::new(0));
    for conn in listener.incoming() {
        let mut stream = match conn {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("builder: accept failed: {e}");
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
        };
        if active.fetch_add(1, Ordering::SeqCst) >= max_conns {
            active.fetch_sub(1, Ordering::SeqCst);
            tracing::warn!(max_conns, "builder: too many connections; closing one");
            continue;
        }
        let ctx = ctx.clone();
        let active_c = Arc::clone(&active);
        let spawned = crate::helper_threads::spawn("builder-conn", move || {
            let outcome = handle_connection(&mut stream, &ctx);
            tracing::debug!(?outcome, "builder: connection closed");
            active_c.fetch_sub(1, Ordering::SeqCst);
        });
        if spawned.is_err() {
            active.fetch_sub(1, Ordering::SeqCst);
        }
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn run_builder(_cfg: BuilderConfig) -> anyhow::Result<()> {
    anyhow::bail!("the builder service needs Linux (Unix domain sockets + bubblewrap)")
}

/// Output of `sandbox-verify`.
#[derive(Debug, Serialize)]
struct VerifyOutput {
    compiled: bool,
    ran_ok: bool,
    stdout: String,
    stderr: String,
}

/// `pledge_compile_api sandbox-verify`: build + run the program on stdin
/// with the deny-list OFF under the builder's (bwrap) sandbox settings and
/// print `{compiled, ran_ok, stdout, stderr}` as JSON. See module docs for
/// why this cannot become a production bypass.
pub fn sandbox_verify_main() -> ExitCode {
    match sandbox_verify() {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            tracing::error!("sandbox-verify: {e:#}");
            eprintln!("sandbox-verify: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn sandbox_verify() -> anyhow::Result<String> {
    let cfg = BuilderConfig::from_env()?;
    sandbox_verify_check(&cfg)?;
    let mut source = String::new();
    std::io::stdin()
        .lock()
        .take(crate::MAX_SOURCE_BYTES as u64 + 1)
        .read_to_string(&mut source)?;
    if source.len() > crate::MAX_SOURCE_BYTES || source.trim().is_empty() {
        anyhow::bail!("probe source must be 1..={} bytes", crate::MAX_SOURCE_BYTES);
    }
    let mut settings = cfg.settings;
    // The whole point: test the sandbox, not the deny-list.
    settings.source_denylist = false;
    crate::build_root::prepare(&settings.root)?;
    tracing::warn!("sandbox-verify: deny-list OFF for this one probe build");
    let outcome = wasm_builder::compile_and_run_in(&source, &RunLimits::default(), &settings)?;
    Ok(serde_json::to_string(&VerifyOutput {
        compiled: outcome.compiled,
        ran_ok: outcome.ran_ok,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
    })?)
}

/// `sandbox-verify` only ever verifies bwrap.
pub fn sandbox_verify_check(cfg: &BuilderConfig) -> anyhow::Result<()> {
    if cfg.settings.sandbox.mode != SandboxMode::Bwrap {
        anyhow::bail!("sandbox-verify only runs with PNC_BUILD_SANDBOX=bwrap");
    }
    Ok(())
}

/// For tests: does `path` look like a socket path the builder accepts?
#[doc(hidden)]
pub fn socket_path_ok(path: &Path) -> bool {
    config::parse_socket_path(&path.to_string_lossy()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";
    const TC: &str = "/opt/rust/rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu";

    fn parse(pairs: &[(&str, &str)]) -> Result<BuilderConfig, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        BuilderConfig::from_lookup(|k| map.get(k).cloned())
    }

    #[test]
    fn builder_refuses_any_token_variable() {
        for var in [vars::AUTH_TOKENS, vars::METRICS_TOKENS] {
            assert!(matches!(
                parse(&[(var, TOKEN), (vars::BUILD_SANDBOX, "none")]),
                Err(ConfigError::BuilderHoldsSecrets { .. })
            ));
        }
    }

    #[test]
    fn builder_refuses_none_without_dev_override_and_denylist_off() {
        assert!(matches!(
            parse(&[(vars::BUILD_SANDBOX, "none")]),
            Err(ConfigError::UnsandboxedExposure { .. })
        ));
        assert!(
            parse(&[
                (vars::BUILD_SANDBOX, "none"),
                (vars::DEV_UNSAFE_LOCAL_BUILDS, "true")
            ])
            .is_ok()
        );
        assert!(
            parse(&[
                (vars::BUILD_SANDBOX, "none"),
                (vars::DEV_UNSAFE_LOCAL_BUILDS, "true"),
                (vars::SOURCE_DENYLIST, "false")
            ])
            .is_err()
        );
        let c = BuilderConfig {
            socket: PathBuf::from(config::DEFAULT_BUILDER_SOCKET),
            settings: BuildSettings {
                sandbox: crate::build_sandbox::SandboxSettings {
                    mode: SandboxMode::Bwrap,
                    toolchain_dir: Some(PathBuf::from(TC)),
                    ..Default::default()
                },
                ..BuildSettings::default()
            },
            max_jobs: 2,
            dev_unsafe_local_builds: false,
        };
        assert!(c.validate_for(true).is_ok());
        assert!(matches!(
            c.validate_for(false),
            Err(ConfigError::SandboxUnsupported)
        ));
        assert!(sandbox_verify_check(&c).is_ok());
        let none = BuilderConfig {
            settings: BuildSettings::default(),
            dev_unsafe_local_builds: true,
            ..c
        };
        assert!(
            sandbox_verify_check(&none).is_err(),
            "verify never runs unsandboxed"
        );
        assert!(socket_path_ok(Path::new("/run/pnc-builder/builder.sock")));
        assert!(!socket_path_ok(Path::new("builder.sock")));
    }

    #[test]
    fn slots_release_on_drop_and_stay_taken_when_retired() {
        let slots = JobSlots::new(2);
        let a = slots.try_acquire().unwrap();
        let b = slots.try_acquire().unwrap();
        assert!(slots.try_acquire().is_none());
        drop(a);
        assert_eq!(slots.available(), 1);
        b.retire();
        assert_eq!(slots.available(), 1, "retired slot is never returned");
    }
}
