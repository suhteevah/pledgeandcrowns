// SPDX-License-Identifier: MIT
//! Audit: the player-facing "Show solution" text.
//!
//! A reference solution that does not compile, or that the game's own
//! grader rejects, is a P0 for a game that teaches Rust (CLAUDE.md hard
//! rules 3 and 4). For every mission in `MissionRegistry::default()` this
//! suite proves, rather than asserts, that:
//!
//!   1. a solution exists (or the id is listed in `KNOWN_GAPS` with why);
//!   2. the offline stub grader passes it;
//!   3. the server pattern grader passes it;
//!   4. (`--ignored`) it builds as a binary with plain `rustc`, no warnings;
//!   5. (`--ignored`) it builds as a binary for `wasm32-wasip1`, the target
//!      the server's `/compile-real` path uses;
//!   6. (`--ignored`) it builds AND runs to a clean exit in the server's
//!      real wasm sandbox, except the ids in `EXPECTED_SANDBOX_RUN_FAILURES`.
//!
//! Every `rustc` build here uses `common::PLAYER_EDITION`, the edition the
//! server builds player code with.
//!
//! Run the slow half with:
//!
//! ```text
//! cargo test -p pledge_and_crown --test solutions -- --ignored
//! ```

mod common;

use common::{PLAYER_EDITION, SERVER_MANIFEST_SOURCES};
use pledge_and_crown::plugins::editor::solutions::{all_solutions, canonical_solution};
use pledge_and_crown::plugins::mission::MissionRegistry;
use pledge_and_crown::plugins::stub_grader::stub_verdict;
use pledge_compile_api::grader::grade;
use pledge_compile_api::wasm_builder::compile_and_run;
use std::path::PathBuf;
use std::process::Command;

/// Missions with no player-facing solution, and why. Empty means every
/// mission is covered. An id belongs here only when no program can both
/// compile and pass the grader (i.e. the grader or mission text is wrong).
const KNOWN_GAPS: &[(&str, &str)] = &[];

/// Missions whose solution builds for `wasm32-wasip1` but cannot run to a
/// clean exit in the server sandbox, and why. Each must actually fail
/// there; the sandbox test fails if one starts passing, so this list
/// cannot go stale silently.
const EXPECTED_SANDBOX_RUN_FAILURES: &[(&str, &str)] = &[
    (
        "thread_spawn",
        "wasm32-wasip1 std has no thread support: thread::spawn panics \
         (observed as a wasm `unreachable` trap in the sandbox)",
    ),
    (
        "thread_scope",
        "wasm32-wasip1 std has no thread support: Scope::spawn panics \
         (observed as a wasm `unreachable` trap in the sandbox)",
    ),
];

fn is_known_gap(id: &str) -> bool {
    KNOWN_GAPS.iter().any(|(gap, _)| *gap == id)
}

/// `(id, solution)` for every registry mission that has one, in order.
fn covered() -> Vec<(&'static str, &'static str)> {
    MissionRegistry::default()
        .missions
        .iter()
        .filter_map(|m| canonical_solution(m.id).map(|s| (m.id, s)))
        .collect()
}

#[test]
fn every_mission_has_a_solution_or_a_documented_gap() {
    let reg = MissionRegistry::default();
    let mut missing = Vec::new();
    for m in &reg.missions {
        let has = canonical_solution(m.id).is_some();
        let gap = is_known_gap(m.id);
        assert!(
            !(has && gap),
            "`{}` has a solution but is still listed in KNOWN_GAPS — remove it",
            m.id
        );
        if !has && !gap {
            missing.push(m.id);
        }
    }
    assert!(
        missing.is_empty(),
        "missions with no solution and no KNOWN_GAPS entry: {missing:?}"
    );
    for (gap, why) in KNOWN_GAPS {
        assert!(
            reg.find(gap).is_some(),
            "KNOWN_GAPS lists `{gap}` ({why}) but the registry has no such mission"
        );
    }
    println!(
        "solutions: {} of {} missions covered, {} known gaps",
        covered().len(),
        reg.missions.len(),
        KNOWN_GAPS.len()
    );
}

#[test]
fn no_solution_for_an_unknown_mission() {
    let reg = MissionRegistry::default();
    for (id, _) in all_solutions() {
        assert!(
            reg.find(id).is_some(),
            "solution authored for `{id}`, which is not in the registry"
        );
    }
}

#[test]
fn every_solution_is_a_complete_program() {
    for (id, src) in covered() {
        assert!(
            src.contains("fn main()"),
            "solution for `{id}` has no `fn main()`"
        );
    }
}

/// `PLAYER_EDITION` is a copy of the server's manifest edition (the server
/// does not export it). Every `edition = "..."` in the server's embedded
/// manifests must equal it, so the two cannot drift silently.
#[test]
fn player_edition_matches_the_server_manifests() {
    let mut found = 0;
    for path in SERVER_MANIFEST_SOURCES {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        for line in text.lines() {
            let Some(rest) = line.trim().strip_prefix("edition") else {
                continue;
            };
            let Some(value) = rest.trim_start().strip_prefix('=') else {
                continue;
            };
            let value = value.trim().trim_matches('"');
            assert_eq!(
                value, PLAYER_EDITION,
                "{path} builds player code with edition {value}, but the tests use \
                 PLAYER_EDITION = {PLAYER_EDITION}; update game/tests/common/mod.rs"
            );
            found += 1;
        }
    }
    assert_eq!(
        found,
        SERVER_MANIFEST_SOURCES.len(),
        "expected exactly one `edition = ...` manifest line per server source"
    );
}

#[test]
fn solutions_pass_the_stub_grader() {
    for (id, src) in covered() {
        let v = stub_verdict(id, src).unwrap_or_else(|| panic!("no stub arm for `{id}`"));
        assert!(v.ok, "stub grader rejected the solution for `{id}`: {v:?}");
    }
}

#[test]
fn solutions_pass_the_server_pattern_grader() {
    for (id, src) in covered() {
        let v = grade(id, src);
        assert!(
            v.ok,
            "server grader rejected the solution for `{id}`: {v:?}"
        );
    }
}

#[test]
fn solutions_differ_from_the_starter() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        if let Some(src) = canonical_solution(m.id) {
            assert_ne!(
                src, m.starter_code,
                "solution for `{}` is the starter",
                m.id
            );
        }
    }
}

/// Fresh per-process scratch dir under cargo's per-target temp area, so
/// concurrent runs (or worktrees sharing a target dir) never collide.
fn scratch_dir(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("solutions-{tag}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear stale scratch dir");
    }
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Build every solution with `rustc` and the given extra args. Returns the
/// list of `(id, stderr)` failures; prints a pass count.
fn rustc_build_all(tag: &str, extra: &[&str]) -> Vec<(&'static str, String)> {
    let dir = scratch_dir(tag);
    let solutions = covered();
    let mut failures = Vec::new();
    for (id, src) in &solutions {
        let src_path = dir.join(format!("{id}.rs"));
        std::fs::write(&src_path, src).expect("write solution source");
        let out = Command::new("rustc")
            .args([
                "--edition",
                PLAYER_EDITION,
                "--crate-type",
                "bin",
                "--crate-name",
            ])
            .arg(id)
            .args(["-D", "warnings"])
            .args(extra)
            .arg("-o")
            .arg(dir.join(format!("{id}.out")))
            .arg(&src_path)
            .output()
            .expect("spawn rustc");
        if !out.status.success() {
            failures.push((*id, String::from_utf8_lossy(&out.stderr).into_owned()));
        }
    }
    println!(
        "[{tag}] rustc build: {} of {} solutions passed",
        solutions.len() - failures.len(),
        solutions.len()
    );
    // Best-effort cleanup; a leftover dir under the target tmp is harmless.
    let _ = std::fs::remove_dir_all(&dir);
    failures
}

#[test]
#[ignore = "slow: runs rustc once per mission"]
fn every_solution_compiles_natively_without_warnings() {
    let failures = rustc_build_all("native", &[]);
    assert!(
        failures.is_empty(),
        "solutions that do not compile cleanly with rustc:\n{failures:#?}"
    );
}

fn wasip1_installed() -> bool {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| l.trim() == "wasm32-wasip1")
        })
        .unwrap_or(false)
}

#[test]
#[ignore = "slow: runs rustc once per mission for wasm32-wasip1"]
fn every_solution_builds_for_wasm32_wasip1() {
    assert!(
        wasip1_installed(),
        "wasm32-wasip1 is not installed: rustup target add wasm32-wasip1"
    );
    let failures = rustc_build_all("wasip1", &["--target", "wasm32-wasip1"]);
    assert!(
        failures.is_empty(),
        "solutions that do not build for wasm32-wasip1:\n{failures:#?}"
    );
}

#[test]
#[ignore = "slow: one real cargo build + sandboxed wasm run per mission"]
fn solutions_build_and_run_in_the_real_server_sandbox() {
    let solutions = covered();
    let mut unexpected = Vec::new();
    let mut ran_ok = 0usize;
    for (id, src) in &solutions {
        let outcome = compile_and_run(src)
            .unwrap_or_else(|e| panic!("sandbox setup failed for `{id}`: {e:#}"));
        let expected_fail = EXPECTED_SANDBOX_RUN_FAILURES
            .iter()
            .any(|(fail, _)| fail == id);
        if outcome.ok() {
            ran_ok += 1;
        }
        println!(
            "[sandbox] {id}: compiled={} ran_ok={} stderr={:?}",
            outcome.compiled,
            outcome.ran_ok,
            outcome.stderr.lines().take(3).collect::<Vec<_>>()
        );
        match (outcome.ok(), expected_fail) {
            (true, true) => unexpected.push(format!(
                "`{id}` now runs cleanly: remove it from EXPECTED_SANDBOX_RUN_FAILURES"
            )),
            (false, false) => unexpected.push(format!(
                "`{id}` failed: compiled={} stderr={}",
                outcome.compiled, outcome.stderr
            )),
            (false, true) => assert!(
                outcome.compiled,
                "`{id}` is expected to fail only at runtime, but did not compile: {}",
                outcome.stderr
            ),
            (true, false) => {}
        }
    }
    println!(
        "[sandbox] {ran_ok} of {} solutions built and ran cleanly",
        solutions.len()
    );
    assert!(unexpected.is_empty(), "{unexpected:#?}");
}
