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
//! - Missions whose Task states the exact output (`PINNED_OUTPUT`) show
//!   every expected line as a code span in the Task text, and the
//!   player-facing solution really prints exactly that (`--ignored`).
//! - Outputs claimed for common mistakes in those missions' Hints
//!   (`HINT_CLAIMS`) are what those programs really print (`--ignored`).

mod common;

use common::PLAYER_EDITION;
use pledge_and_crown::plugins::editor::solutions::canonical_solution;
use pledge_and_crown::plugins::mission::{DEMO_LAST_MISSION_ID, MissionRegistry};
use pledge_and_crown::plugins::progress::MissionProgress;
use std::path::PathBuf;
use std::process::Command;

/// Missions whose starter is meant to fail to compile, with the error
/// code the lesson is about.
const STARTERS_THAT_MUST_FAIL: &[(&str, &str)] = &[("lifetimes", "E0106")];

/// Fresh per-test, per-process scratch dir (the slow tests run in
/// parallel, so each needs its own).
fn scratch_dir(tag: &str) -> PathBuf {
    let dir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{tag}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear stale scratch dir");
    }
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

#[test]
#[ignore = "slow: runs rustc once per mission"]
fn every_starter_compiles_or_fails_for_its_intended_reason() {
    let dir = scratch_dir("starters");
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

/// Missions whose Task text pins the program's output, with that output
/// (lines joined by `\n`). The server's output checks for these missions
/// (`compile-api/src/expectations/data.rs`) were recorded from the same
/// solutions in the real sandbox.
const PINNED_OUTPUT: &[(&str, &str)] = &[
    ("vec_iter", "60"),
    ("closure_basic", "42"),
    ("enum_match", "0\n90\n180\n270"),
    ("and_then", "Some(2)\nNone"),
    ("hashmap_basic", "Some(7)\nNone"),
    ("arc_mutex", "5"),
    ("mpsc_channel", "7\n11"),
    ("atomic", "10\n11"),
    ("rc_basic", "1\n2\n1"),
    ("refcell", "10\n15"),
    ("cell", "1\n5"),
    ("rc_refcell", "15"),
    ("weak_ref", "Some(5)\nNone"),
];

/// `(mission, program, stdout)`: the mistakes the Hint text of a pinned
/// mission describes, and what they print (hard rule 3: the claim must be
/// true).
const HINT_CLAIMS: &[(&str, &str, &str)] = &[
    (
        "rc_basic",
        "use std::rc::Rc;\nfn main() {\n    let original = Rc::new(String::from(\"hoard\"));\n    println!(\"{}\", Rc::strong_count(&original));\n    let shared = Rc::new((*original).clone());\n    println!(\"{}\", Rc::strong_count(&original));\n    drop(shared);\n    println!(\"{}\", Rc::strong_count(&original));\n}\n",
        "1\n1\n1",
    ),
    (
        "refcell",
        "use std::cell::RefCell;\nfn main() {\n    let guarded = RefCell::new(10);\n    let r = &guarded;\n    println!(\"{}\", r.borrow());\n    let mut n = *r.borrow_mut();\n    n += 5;\n    let _ = n;\n    println!(\"{}\", r.borrow());\n}\n",
        "10\n10",
    ),
    (
        "cell",
        "use std::cell::Cell;\nfn main() {\n    let slot = Cell::new(1);\n    let r = &slot;\n    println!(\"{}\", r.get());\n    let mut copy = r.get();\n    copy += 4;\n    let _ = copy;\n    println!(\"{}\", r.get());\n}\n",
        "1\n1",
    ),
    (
        "rc_refcell",
        "use std::cell::RefCell;\nuse std::rc::Rc;\nfn main() {\n    let shared = Rc::new(RefCell::new(0));\n    let other = Rc::new(RefCell::new(*shared.borrow()));\n    *other.borrow_mut() += 10;\n    *shared.borrow_mut() += 5;\n    println!(\"{}\", shared.borrow());\n    println!(\"{}\", other.borrow());\n}\n",
        "5\n10",
    ),
    (
        "weak_ref",
        "use std::rc::{Rc, Weak};\nfn main() {\n    let strong = Rc::new(5);\n    let weak: Weak<i32> = Rc::downgrade(&strong);\n    let alive = weak.upgrade();\n    println!(\"{alive:?}\");\n    drop(strong);\n    println!(\"{:?}\", weak.upgrade());\n}\n",
        "Some(5)\nSome(5)",
    ),
];

/// The `## Task` section of a tutorial.
fn task_section(tutorial: &str) -> &str {
    let start = tutorial
        .find("## Task")
        .expect("tutorial has a Task section");
    let rest = &tutorial[start..];
    let end = rest.find("## Hint").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn pinned_output_lines_are_shown_in_the_task() {
    let reg = MissionRegistry::default();
    for (id, output) in PINNED_OUTPUT {
        let m = reg
            .find(id)
            .unwrap_or_else(|| panic!("`{id}` is not a mission"));
        let task = task_section(m.tutorial);
        for line in output.lines() {
            assert!(
                task.contains(&format!("`{line}`")),
                "`{id}`: the Task must show the expected line `{line}` in a code span\n{task}"
            );
        }
    }
    for (id, _, _) in HINT_CLAIMS {
        assert!(
            PINNED_OUTPUT.iter().any(|(p, _)| p == id),
            "{id} is not pinned"
        );
    }
}

/// Build `src` with rustc (warnings allowed: the mistakes are deliberate)
/// and return its stdout, normalised like the server does.
fn run_native(dir: &std::path::Path, name: &str, src: &str) -> String {
    let src_path = dir.join(format!("{name}.rs"));
    let exe = dir.join(format!("{name}.exe"));
    std::fs::write(&src_path, src).expect("write program");
    let out = Command::new("rustc")
        .args(["--edition", PLAYER_EDITION, "--crate-type", "bin"])
        .arg("-o")
        .arg(&exe)
        .arg(&src_path)
        .output()
        .expect("spawn rustc");
    assert!(
        out.status.success(),
        "{name} does not compile:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe).output().expect("run program");
    assert!(run.status.success(), "{name} did not exit cleanly");
    String::from_utf8_lossy(&run.stdout)
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

#[test]
#[ignore = "slow: runs rustc once per pinned mission and hint claim"]
fn pinned_outputs_and_hint_claims_are_true() {
    let dir = scratch_dir("pinned");
    let mut problems = Vec::new();
    for (id, want) in PINNED_OUTPUT {
        let src = canonical_solution(id).unwrap_or_else(|| panic!("no solution for `{id}`"));
        let got = run_native(&dir, id, src);
        if got != *want {
            problems.push(format!(
                "`{id}` solution prints {got:?}, Task says {want:?}"
            ));
        }
    }
    for (n, (id, src, want)) in HINT_CLAIMS.iter().enumerate() {
        let got = run_native(&dir, &format!("hint_{n}_{id}"), src);
        if got != *want {
            problems.push(format!(
                "`{id}` hint claim prints {got:?}, expected {want:?}"
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
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
