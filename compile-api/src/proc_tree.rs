// SPDX-License-Identifier: AGPL-3.0-or-later
//! Run a build subprocess with bounded output capture and a wall-clock
//! watchdog that kills the **whole process tree**, not just the direct
//! child.
//!
//! `cargo` spawns `rustc`, which spawns the linker. Killing only `cargo`
//! on timeout leaves `rustc` running outside the build semaphore, after
//! the job dir has been deleted. This module fixes that:
//!
//! - **Linux / Unix:** the child is spawned as the leader of a new process
//!   group (`CommandExt::process_group(0)`, safe std API). On timeout the
//!   whole group gets `SIGKILL` via `rustix::process::kill_process_group`
//!   (a safe wrapper, no `unsafe` here). After every run — timeout or
//!   normal exit — we sweep the group with `SIGKILL` and then poll
//!   `kill(-pgid, 0)` until it reports `ESRCH` ("no process left in the
//!   group"), bounded by [`TREE_DEATH_TIMEOUT`]. Only then does
//!   [`run_captured`] return, so the caller's semaphore permit is held
//!   until the tree is confirmed dead. A survivor past the deadline is an
//!   error ([`ProcError::Survivors`]), never silently ignored.
//!   Limitation: a process that deliberately leaves the group (`setsid`)
//!   escapes the sweep. No player code runs during a dependency-free build
//!   (no `build.rs`, no proc-macros), and under `PNC_BUILD_SANDBOX=bwrap`
//!   the build runs in its own PID namespace that the kernel tears down
//!   when the sandbox's init exits, which covers that case.
//! - **Windows (dev only):** on timeout we run `taskkill /T /F /PID <pid>`,
//!   which kills the child and every descendant it can find by parent PID.
//!   That needs no `unsafe` (a Job Object with kill-on-close would, via
//!   raw `windows-sys` FFI). Gap: we do not independently confirm that no
//!   descendant survived on Windows; `taskkill`'s exit status is logged.
//!   Windows is never a production target.
//!
//! The wall-clock `timeout` covers the whole run: output collection AND the
//! wait for the child to exit. A child that closes its stdout/stderr but
//! keeps running is killed at the deadline like any other (it used to be
//! waited for without a bound). Readers and the optional stdin writer are
//! counted helper threads ([`crate::helper_threads`]).
//!
//! Callers that hold a concurrency slot must treat
//! [`ProcError::Survivors`] as "the work is NOT over": never release the
//! slot on that error (see `lib.rs` and `builder.rs`).

use std::io::{Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::helper_threads;

/// How long to wait, after `SIGKILL`, for every process in the group to
/// disappear (or, on Windows, for `taskkill` to finish).
pub const TREE_DEATH_TIMEOUT: Duration = Duration::from_secs(5);

/// How long to wait for the output reader threads after the tree is dead.
/// A reader only outlives this if some process outside our group still
/// holds the pipe; it is then detached and its output dropped.
const READER_JOIN_TIMEOUT: Duration = Duration::from_secs(5);

/// Process supervision failures (setup or teardown, not "the build
/// failed" — that is a normal [`Captured`] with a non-zero status).
#[derive(Debug, thiserror::Error)]
pub enum ProcError {
    #[error("{label}: failed to spawn: {source}")]
    Spawn {
        label: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{label}: stdout/stderr were not piped")]
    NotPiped { label: String },
    #[error("{label}: wait failed: {source}")]
    Wait {
        label: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{label}: process group {pgid} still has live members {waited:?} after SIGKILL")]
    Survivors {
        label: String,
        pgid: u32,
        waited: Duration,
    },
    #[error("{label}: could not start a helper thread: {source}")]
    Thread {
        label: String,
        #[source]
        source: std::io::Error,
    },
}

impl ProcError {
    /// True for [`ProcError::Survivors`]: processes of this run may still
    /// be alive, so its concurrency slot must not be released.
    pub fn is_survivors(&self) -> bool {
        matches!(self, Self::Survivors { .. })
    }
}

/// True if `err` (or anything in its cause chain) is
/// [`ProcError::Survivors`].
pub fn chain_has_survivors(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause
            .downcast_ref::<ProcError>()
            .is_some_and(ProcError::is_survivors)
    })
}

/// Result of a supervised run.
#[derive(Debug)]
pub struct Captured {
    /// OS process id of the direct child (for logs and tests).
    pub pid: u32,
    /// Exit status of the direct child.
    pub status: ExitStatus,
    /// Captured stdout, lossy UTF-8, capped.
    pub stdout: String,
    /// Captured stderr, lossy UTF-8, capped.
    pub stderr: String,
    /// True if the watchdog fired and the tree was killed.
    pub timed_out: bool,
    /// Wall time from spawn to confirmed tree death.
    pub elapsed: Duration,
}

/// Spawn `cmd` in its own process group, capture stdout/stderr (each capped
/// at `cap_bytes`), and kill the whole tree if it runs past `timeout`.
/// Returns only after the tree is confirmed dead (see module docs).
///
/// `cmd`'s stdin/stdout/stderr are overridden (null / piped / piped).
pub fn run_captured(
    cmd: Command,
    timeout: Duration,
    cap_bytes: usize,
    label: &str,
) -> Result<Captured, ProcError> {
    run_captured_with_input(cmd, None, timeout, cap_bytes, label)
}

/// [`run_captured`], optionally feeding `input` to the child's stdin (then
/// closing it). With `input == None` stdin is `/dev/null`.
pub fn run_captured_with_input(
    mut cmd: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
    cap_bytes: usize,
    label: &str,
) -> Result<Captured, ProcError> {
    let stdin_mode = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    cmd.stdin(stdin_mode)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    let started = Instant::now();
    let deadline = started + timeout;
    let mut child = cmd.spawn().map_err(|source| {
        tracing::error!(label, "proc_tree: spawn failed: {source}");
        ProcError::Spawn {
            label: label.to_string(),
            source,
        }
    })?;
    let pid = child.id();
    tracing::debug!(label, pid, "proc_tree: spawned");

    let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
        kill_tree_now(&mut child, label);
        let _ = child.wait();
        tracing::error!(label, "proc_tree: stdout/stderr not piped");
        return Err(ProcError::NotPiped {
            label: label.to_string(),
        });
    };

    let thread_err = |source: std::io::Error| ProcError::Thread {
        label: label.to_string(),
        source,
    };

    // Feed stdin from a helper so a child that does not read it cannot
    // block us; the pipe closes when the writer finishes.
    let mut writer = None;
    if let Some(bytes) = input {
        match child.stdin.take() {
            Some(mut sink) => {
                let wlabel = label.to_string();
                let spawned = helper_threads::spawn("stdin-writer", move || {
                    if let Err(e) = sink.write_all(&bytes) {
                        tracing::debug!(label = wlabel, "proc_tree: stdin write ended early: {e}");
                    }
                });
                match spawned {
                    Ok(h) => writer = Some(h),
                    Err(e) => {
                        kill_tree_now(&mut child, label);
                        let _ = child.wait();
                        return Err(thread_err(e));
                    }
                }
            }
            None => {
                kill_tree_now(&mut child, label);
                let _ = child.wait();
                tracing::error!(label, "proc_tree: stdin not piped");
                return Err(ProcError::NotPiped {
                    label: label.to_string(),
                });
            }
        }
    }

    // Each reader sends its capped output when its pipe hits EOF; the
    // watchdog wakes on those messages instead of busy-polling.
    let (tx, rx) = mpsc::channel::<(usize, String)>();
    for (idx, pipe) in [
        (0usize, Box::new(stdout) as Box<dyn Read + Send>),
        (1usize, Box::new(stderr) as Box<dyn Read + Send>),
    ] {
        let tx = tx.clone();
        let spawned = helper_threads::spawn("output-reader", move || {
            let s = read_capped(pipe, cap_bytes);
            let _ = tx.send((idx, s));
        });
        if let Err(e) = spawned {
            kill_tree_now(&mut child, label);
            let _ = child.wait();
            return Err(thread_err(e));
        }
    }
    drop(tx);

    let mut outputs: [Option<String>; 2] = [None, None];
    let mut timed_out = false;
    while outputs.iter().any(Option::is_none) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            timed_out = true;
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok((idx, s)) => outputs[idx] = Some(s),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                timed_out = true;
                break;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // Pipes closed does not mean the child exited: bound the wait by the
    // same deadline (a child can close stdout/stderr and keep running).
    let mut exited = None;
    if !timed_out {
        exited = wait_until(&mut child, deadline, label)?;
        if exited.is_none() {
            tracing::warn!(
                label,
                ?timeout,
                "proc_tree: child closed its output but did not exit before the deadline"
            );
            timed_out = true;
        }
    }

    let status = match exited {
        Some(status) => {
            // Normal exit: sweep any straggler that kept running after the
            // leader exited, then confirm.
            sweep_group_after_exit(&child, label);
            confirm_tree_dead(&child, label)?;
            status
        }
        None => {
            tracing::warn!(
                label,
                ?timeout,
                "proc_tree: watchdog fired, killing process tree"
            );
            kill_tree_now(&mut child, label);
            // SIGKILL cannot be ignored, so this wait ends unless the
            // kernel itself is stuck; it is still bounded below.
            let status = match wait_until(&mut child, Instant::now() + TREE_DEATH_TIMEOUT, label)? {
                Some(s) => s,
                None => {
                    tracing::error!(label, pid, "proc_tree: child survived SIGKILL");
                    return Err(ProcError::Survivors {
                        label: label.to_string(),
                        pgid: pid,
                        waited: TREE_DEATH_TIMEOUT,
                    });
                }
            };
            confirm_tree_dead(&child, label)?;
            status
        }
    };

    if let Some(h) = writer
        && h.join().is_err()
    {
        tracing::warn!(label, "proc_tree: stdin writer panicked");
    }

    // Collect whatever the readers have; with the tree dead the pipes are
    // closed, so this is immediate unless an escaped process holds them.
    let deadline = Instant::now() + READER_JOIN_TIMEOUT;
    while outputs.iter().any(Option::is_none) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(remaining) {
            Ok((idx, s)) => outputs[idx] = Some(s),
            Err(_) => {
                tracing::error!(
                    label,
                    "proc_tree: output pipe still open after tree death; detaching reader"
                );
                break;
            }
        }
    }
    let [out, err] = outputs;
    let elapsed = started.elapsed();
    tracing::debug!(
        label,
        code = ?status.code(),
        timed_out,
        ?elapsed,
        "proc_tree: finished"
    );
    Ok(Captured {
        pid,
        status,
        stdout: out.unwrap_or_default(),
        stderr: err.unwrap_or_default(),
        timed_out,
        elapsed,
    })
}

/// Poll `child` until it exits or `deadline` passes. `Ok(None)` means it
/// is still running at the deadline.
fn wait_until(
    child: &mut Child,
    deadline: Instant,
    label: &str,
) -> Result<Option<ExitStatus>, ProcError> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(Some(status)),
            Ok(None) => {}
            Err(source) => {
                tracing::error!(label, "proc_tree: wait failed: {source}");
                return Err(ProcError::Wait {
                    label: label.to_string(),
                    source,
                });
            }
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(5));
    }
}

/// Kill the child and all its descendants right now (timeout path).
fn kill_tree_now(child: &mut Child, label: &str) {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process_group};
        let pgid = Pid::from_child(child);
        match kill_process_group(pgid, Signal::KILL) {
            Ok(()) => tracing::debug!(label, pgid = child.id(), "proc_tree: SIGKILL sent to group"),
            Err(e) if e == rustix::io::Errno::SRCH => {
                tracing::debug!(label, "proc_tree: group already gone")
            }
            Err(e) => {
                tracing::error!(
                    label,
                    "proc_tree: kill(-pgid) failed: {e}; killing leader only"
                );
                let _ = child.kill();
            }
        }
    }
    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        match Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(s) => tracing::debug!(label, pid, code = ?s.code(), "proc_tree: taskkill /T /F"),
            Err(e) => tracing::error!(label, pid, "proc_tree: taskkill failed to run: {e}"),
        }
        // Belt and braces: make sure the direct child is gone even if
        // taskkill could not run.
        let _ = child.kill();
    }
    #[cfg(not(any(unix, windows)))]
    {
        tracing::warn!(
            label,
            "proc_tree: no tree kill on this platform; killing leader"
        );
        let _ = child.kill();
    }
}

/// After the leader exited normally, SIGKILL anything left in its group.
fn sweep_group_after_exit(child: &Child, label: &str) {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process_group};
        match kill_process_group(Pid::from_child(child), Signal::KILL) {
            Ok(()) => tracing::warn!(
                label,
                "proc_tree: processes outlived the build leader; SIGKILLed the group"
            ),
            Err(e) if e == rustix::io::Errno::SRCH => {}
            Err(e) => tracing::error!(label, "proc_tree: post-exit group sweep failed: {e}"),
        }
    }
    #[cfg(not(unix))]
    {
        // Windows: cargo waits for rustc before exiting; nothing to sweep.
        let _ = (child, label);
    }
}

/// Poll until no process remains in the child's process group.
fn confirm_tree_dead(child: &Child, label: &str) -> Result<(), ProcError> {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, test_kill_process_group};
        let pgid = Pid::from_child(child);
        let started = Instant::now();
        loop {
            match test_kill_process_group(pgid) {
                Err(e) if e == rustix::io::Errno::SRCH => return Ok(()),
                // EPERM would mean a member changed uid; treat as alive.
                _ => {}
            }
            let waited = started.elapsed();
            if waited >= TREE_DEATH_TIMEOUT {
                tracing::error!(
                    label,
                    pgid = child.id(),
                    ?waited,
                    "proc_tree: group survivors"
                );
                return Err(ProcError::Survivors {
                    label: label.to_string(),
                    pgid: child.id(),
                    waited,
                });
            }
            // Orphaned members are reaped by init/the subreaper; give it
            // a moment.
            thread::sleep(Duration::from_millis(10));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (child, label);
        Ok(())
    }
}

/// Best-effort liveness check for a PID, for tests that assert a
/// grandchild was killed.
#[doc(hidden)]
pub fn pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, test_kill_process};
        let Some(pid) = i32::try_from(pid).ok().and_then(Pid::from_raw) else {
            return false;
        };
        !matches!(test_kill_process(pid), Err(e) if e == rustix::io::Errno::SRCH)
    }
    #[cfg(windows)]
    {
        let filter = format!("PID eq {pid}");
        match Command::new("tasklist")
            .args(["/FI", &filter, "/NH", "/FO", "CSV"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\"")),
            Err(_) => false,
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        false
    }
}

/// Read up to `cap` bytes, then drain the rest so the writer never blocks
/// on a full pipe. Lossy UTF-8 of the prefix, with a marker if truncated.
pub fn read_capped<R: Read>(mut r: R, cap: usize) -> String {
    let mut buf = Vec::with_capacity(4096.min(cap));
    let mut chunk = [0u8; 4096];
    let mut truncated = false;
    loop {
        match r.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                let room = cap.saturating_sub(buf.len());
                let take = n.min(room);
                buf.extend_from_slice(&chunk[..take]);
                if take < n {
                    truncated = true;
                }
            }
            Err(_) => break,
        }
    }
    let mut out = String::from_utf8_lossy(&buf).into_owned();
    if truncated {
        out.push_str("\n... [truncated]");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_capped_truncates_and_marks() {
        let data = vec![b'a'; 10_000];
        let s = read_capped(&data[..], 100);
        assert!(s.starts_with(&"a".repeat(100)));
        assert!(s.ends_with("[truncated]"));
        assert_eq!(read_capped(&b"short"[..], 100), "short");
    }
}
