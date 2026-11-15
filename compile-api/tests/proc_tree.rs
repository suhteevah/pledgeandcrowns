// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process-tree supervision: a build that never terminates must be killed
//! together with every descendant, and `run_captured` must not return
//! until that has happened.
//!
//! The "build" is this test binary re-executed as a helper: a parent that
//! spawns a grandchild (which records its PID and holds the output pipes
//! open) and then both sleep forever. That is exactly the shape of
//! `cargo` -> `rustc` hanging. Runs on every platform: Unix uses the
//! process-group kill, Windows `taskkill /T`.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use pledge_compile_api::proc_tree::{TREE_DEATH_TIMEOUT, pid_alive, run_captured};

const ROLE: &str = "PNC_PROC_TREE_HELPER_ROLE";
const PIDFILE: &str = "PNC_PROC_TREE_HELPER_PIDFILE";

fn helper_command(role: &str, pidfile: &PathBuf) -> Command {
    let exe = std::env::current_exe().expect("current_exe");
    let mut cmd = Command::new(exe);
    cmd.args([
        "--exact",
        "proc_tree_helper_entrypoint",
        "--nocapture",
        "--test-threads=1",
    ])
    .env(ROLE, role)
    .env(PIDFILE, pidfile);
    cmd
}

/// Not a real test: when re-executed with `ROLE` set, act as a helper
/// process. Without `ROLE` (the normal test run) it returns immediately.
#[test]
fn proc_tree_helper_entrypoint() {
    let Ok(role) = std::env::var(ROLE) else {
        return;
    };
    let pidfile = PathBuf::from(std::env::var(PIDFILE).expect("pidfile env"));
    match role.as_str() {
        "parent" => {
            // Grandchild inherits our stdout/stderr pipes.
            let mut child = helper_command("grandchild", &pidfile)
                .spawn()
                .expect("spawn grandchild");
            println!("parent running");
            std::thread::sleep(Duration::from_secs(600));
            // Never reached in the test: the supervisor kills us first.
            let _ = child.wait();
        }
        "grandchild" => {
            std::fs::write(&pidfile, std::process::id().to_string()).expect("write pidfile");
            println!("grandchild running");
            std::thread::sleep(Duration::from_secs(600));
        }
        other => panic!("unknown helper role {other}"),
    }
}

#[test]
fn never_terminating_build_tree_is_killed_with_no_survivors() {
    let pidfile = std::env::temp_dir().join(format!(
        "pnc-proc-tree-{}-{}.pid",
        std::process::id(),
        uuid_like()
    ));
    let _ = std::fs::remove_file(&pidfile);

    // The watchdog must not fire before the grandchild has started (two
    // test-binary launches). 10 s is far more than that takes even on a
    // host running a 24-job build; it used to be 6 s.
    const WATCHDOG: Duration = Duration::from_secs(10);
    let started = Instant::now();
    let out = run_captured(
        helper_command("parent", &pidfile),
        WATCHDOG,
        16 * 1024,
        "test hang",
    )
    .expect("supervision must succeed");
    let elapsed = started.elapsed();

    assert!(out.timed_out, "watchdog must fire: {out:?}");
    assert!(!out.status.success());
    // Returned promptly after the watchdog (tree death is bounded),
    // instead of waiting 600 s for the sleepers.
    assert!(
        elapsed < WATCHDOG + TREE_DEATH_TIMEOUT + Duration::from_secs(10),
        "took {elapsed:?}"
    );

    let grandchild: u32 = std::fs::read_to_string(&pidfile)
        .expect("grandchild must have started and written its pid before the watchdog")
        .trim()
        .parse()
        .expect("pid");
    let _ = std::fs::remove_file(&pidfile);

    // On Unix run_captured already confirmed the group empty; on Windows
    // taskkill is not confirmed and teardown is asynchronous. Allow the OS
    // to finish tearing the process down (a hang guard, generous for a
    // loaded host; the assertion below is unchanged).
    let deadline = Instant::now() + Duration::from_secs(30);
    while pid_alive(grandchild) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        !pid_alive(grandchild),
        "grandchild {grandchild} survived the tree kill"
    );
}

#[test]
fn fast_command_returns_output_without_waiting_for_timeout() {
    let exe = std::env::current_exe().expect("current_exe");
    let mut cmd = Command::new(exe);
    cmd.arg("--list");
    let started = Instant::now();
    let out = run_captured(cmd, Duration::from_secs(60), 64 * 1024, "test list").unwrap();
    assert!(!out.timed_out);
    assert!(out.status.success());
    assert!(
        out.stdout.contains("proc_tree_helper_entrypoint"),
        "{}",
        out.stdout
    );
    assert!(started.elapsed() < Duration::from_secs(30));
}

fn uuid_like() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default()
}
