// SPDX-License-Identifier: MIT
//! Audit: mission starter code and progression gating.
//!
//! - Every starter must itself compile, except the one mission whose
//!   lesson is a compile error (`lifetimes`, which must fail with
//!   exactly `E0106`). Checked with plain `rustc` at the server's edition
//!   (`common::PLAYER_EDITION`; `--ignored`, slow).
//! - Progression is strict-linear: mission N+1 is locked until mission N
//!   is cleared.
//! - With the `demo` feature, nothing past the demo boundary is playable.

mod common;

use common::PLAYER_EDITION;
use pledge_and_crown::plugins::mission::{DEMO_LAST_MISSION_ID, MissionRegistry};
use pledge_and_crown::plugins::progress::MissionProgress;
use std::path::PathBuf;
use std::process::Command;

/// Missions whose starter is meant to fail to compile, with the error
/// code the lesson is about.
const STARTERS_THAT_MUST_FAIL: &[(&str, &str)] = &[("lifetimes", "E0106")];

fn scratch_dir() -> PathBuf {
    let dir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("starters-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear stale scratch dir");
    }
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

#[test]
#[ignore = "slow: runs rustc once per mission"]
fn every_starter_compiles_or_fails_for_its_intended_reason() {
    let dir = scratch_dir();
    let reg = MissionRegistry::default();
    let mut problems = Vec::new();
    for m in &reg.missions {
        let src = dir.join(format!("{}.rs", m.id));
        std::fs::write(&src, m.starter_code).expect("write starter");
        let out = Command::new("rustc")
            .args([
                "--edition",
                PLAYER_EDITION,
                "--crate-type",
                "bin",
                "--crate-name",
            ])
            .arg(m.id)
            .arg("-o")
            .arg(dir.join(format!("{}.out", m.id)))
            .arg(&src)
            .output()
            .expect("spawn rustc");
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        let expected_error = STARTERS_THAT_MUST_FAIL
            .iter()
            .find(|(id, _)| *id == m.id)
            .map(|(_, code)| *code);
        match (expected_error, out.status.success()) {
            (None, true) => {}
            (None, false) => problems.push(format!("{}: starter does not compile\n{stderr}", m.id)),
            (Some(code), false) => {
                let codes: Vec<&str> = stderr
                    .match_indices("error[")
                    .map(|(i, _)| &stderr[i + 6..i + 11])
                    .collect();
                if codes.is_empty() || codes.iter().any(|c| *c != code) {
                    problems.push(format!(
                        "{}: expected only {code}, got {codes:?}\n{stderr}",
                        m.id
                    ));
                }
            }
            (Some(code), true) => problems.push(format!(
                "{}: starter compiled, but should fail with {code}",
                m.id
            )),
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        problems.is_empty(),
        "starter problems:\n{}",
        problems.join("\n")
    );
}

#[test]
fn progression_is_strict_linear() {
    let reg = MissionRegistry::default();
    let mut progress = MissionProgress::default();
    assert!(reg.missions[0].is_unlocked(&progress));
    for i in 0..reg.missions.len() - 1 {
        let (this, next) = (&reg.missions[i], &reg.missions[i + 1]);
        assert!(
            !next.is_unlocked(&progress),
            "`{}` is unlocked before `{}` is cleared",
            next.id,
            this.id
        );
        let expected_next = this.available.then_some(this.id);
        assert_eq!(reg.next_locked(&progress).map(|m| m.id), expected_next);
        progress.mark_cleared(this.id);
        // In a demo build, missions past the boundary never unlock.
        assert_eq!(
            next.is_unlocked(&progress),
            next.available,
            "`{}` unlock state after clearing `{}`",
            next.id,
            this.id
        );
    }
}

#[test]
fn clearing_a_later_mission_does_not_unlock_an_earlier_gap() {
    // Out-of-order clears (e.g. an old save) must not skip the chain:
    // each mission still depends only on its own predecessor.
    let reg = MissionRegistry::default();
    let mut progress = MissionProgress::default();
    progress.mark_cleared(reg.missions[5].id);
    assert!(!reg.missions[2].is_unlocked(&progress));
    assert!(reg.missions[6].is_unlocked(&progress));
    assert_eq!(
        reg.next_locked(&progress).map(|m| m.id),
        Some(reg.missions[0].id)
    );
}

#[test]
fn demo_boundary_is_the_last_act_2_mission() {
    let reg = MissionRegistry::default();
    let idx = reg
        .missions
        .iter()
        .position(|m| m.id == DEMO_LAST_MISSION_ID)
        .expect("demo boundary must be a registered mission");
    // Acts 1-2 are the 21 prelude missions; Act 3 opens with `impl_method`.
    assert_eq!(DEMO_LAST_MISSION_ID, "while_loop");
    assert_eq!(idx + 1, 21, "demo mission count");
    assert_eq!(reg.missions[idx + 1].id, "impl_method");
    assert_eq!(
        reg.missions.len() - (idx + 1),
        36,
        "missions after the demo"
    );
}

#[test]
fn demo_availability_matches_the_feature() {
    let reg = MissionRegistry::default();
    let idx = reg
        .missions
        .iter()
        .position(|m| m.id == DEMO_LAST_MISSION_ID)
        .expect("demo boundary must be a registered mission");
    for (i, m) in reg.missions.iter().enumerate() {
        let expected = !cfg!(feature = "demo") || i <= idx;
        assert_eq!(
            reg.is_available(m.id),
            expected,
            "availability of `{}` (index {i})",
            m.id
        );
    }
    assert!(!reg.is_available("no_such_mission"));
}

#[test]
fn demo_completion_flag() {
    let reg = MissionRegistry::default();
    let mut progress = MissionProgress::default();
    for m in &reg.missions {
        assert!(!reg.demo_complete(&progress));
        progress.mark_cleared(m.id);
        if m.id == DEMO_LAST_MISSION_ID {
            assert_eq!(reg.demo_complete(&progress), cfg!(feature = "demo"));
            if cfg!(feature = "demo") {
                // In the demo, nothing remains to point the player at.
                assert!(reg.next_locked(&progress).is_none());
                return;
            }
        }
    }
    assert!(!reg.demo_complete(&progress));
}
