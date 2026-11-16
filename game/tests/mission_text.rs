// SPDX-License-Identifier: MIT
//! Audit: mission starter code and progression gating.
//!
//! - Every starter must itself compile, except the missions whose lesson
//!   is a compile error (`STARTERS_THAT_MUST_FAIL`: `lifetimes` and most
//!   boss-trial stages), which must fail with exactly their one error
//!   code. Checked with plain `rustc` at the server's edition
//!   (`common::PLAYER_EDITION`; `--ignored`, slow).
//! - Hard rule 3 for the boss trials: every error code a trial owner
//!   names, and every compiler message they quote, is a claim compiled
//!   against real rustc (`trial_claims`; the fast test fails on a named
//!   but unclaimed code, the `--ignored` one runs rustc). The same for the
//!   six Act 9 villagers (`act9_villager_claims`), whose claims also
//!   include run-time behaviour (`Claimed::Prints`).
//! - Act 9 starters that fail with a code-less macro error
//!   (`STARTERS_THAT_MUST_FAIL_WITHOUT_A_CODE`) fail with exactly that
//!   message.
//! - Progression is strict-linear: mission N+1 is locked until mission N
//!   is cleared.
//! - With the `demo` feature, nothing past the demo boundary is playable.
//! - Missions whose Task states the exact output (`PINNED_OUTPUT`) show
//!   every expected line as a code span in the Task text, and the
//!   player-facing solution really prints exactly that (`--ignored`).
//! - Outputs claimed for common mistakes in those missions' Hints
//!   (`HINT_CLAIMS`) are what those programs really print (`--ignored`).

mod act10_programs;
mod act9_programs;
mod common;

use act9_programs as a9;
use act10_programs as a10;
use common::PLAYER_EDITION;
use pledge_and_crown::plugins::editor::solutions::canonical_solution;
use pledge_and_crown::plugins::mission::{DEMO_LAST_MISSION_ID, MissionRegistry};
use pledge_and_crown::plugins::progress::MissionProgress;
use std::path::PathBuf;
use std::process::Command;

/// Missions whose starter is meant to fail to compile, with the error
/// code the lesson is about.
const STARTERS_THAT_MUST_FAIL: &[(&str, &str)] = &[
    ("bridge_toll", "E0382"),
    ("bridge_quill", "E0502"),
    ("bridge_lantern", "E0106"),
    ("wraith_essence", "E0533"),
    ("wraith_possessed", "E0004"),
    ("chimera_heads", "E0599"),
    ("chimera_bound", "E0599"),
    ("chimera_plasma", "E0308"),
    ("lifetimes", "E0106"),
    ("wager_stake", "E0277"),
    ("wager_from", "E0277"),
    ("golem_parts", "E0609"),
    ("golem_sum", "E0283"),
    ("hydra_senders", "E0382"),
    ("hydra_threads", "E0373"),
    ("warden_box", "E0072"),
    ("warden_shared", "E0594"),
    ("raw_pointer", "E0133"),
    ("extern_c", "E0133"),
    ("drop_trait", "E0040"),
    ("send_sync", "E0277"),
    ("maybe_uninit", "E0381"),
    ("librarian_index", "E0133"),
    ("const_eval", "E0015"),
    // Also a code-less "cannot find macro `format`"; E0425 is its only code.
    ("no_std", "E0425"),
    ("compiler_lifetime", "E0106"),
    ("compiler_bound", "E0369"),
];

/// Starters meant to fail with an error that has no code (macro errors),
/// with the message they must fail with. Every error in the output must
/// be code-less.
const STARTERS_THAT_MUST_FAIL_WITHOUT_A_CODE: &[(&str, &str)] = &[
    ("macro_rules", "cannot find macro `chant` in this scope"),
    ("librarian_demand", "no rules expected `,`"),
];

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
        if let Some((_, message)) = STARTERS_THAT_MUST_FAIL_WITHOUT_A_CODE
            .iter()
            .find(|(id, _)| *id == m.id)
        {
            let coded = stderr.contains("error[E");
            if out.status.success() || coded || !stderr.contains(message) {
                problems.push(format!(
                    "{}: expected only `error: {message}`, got\n{stderr}",
                    m.id
                ));
            }
            continue;
        }
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
    ("bridge_toll", "toll: 11\ncargo: lantern oil"),
    (
        "bridge_quill",
        "reader one: north road\nreader two: north road\nscroll: north road, east road",
    ),
    ("bridge_lantern", "first: oil\nwhole: glass"),
    ("wraith_forms", "Solid: steel\nMist: wind\nShadow: light"),
    (
        "wraith_essence",
        "Solid { armor: 3 } takes 7\nMist { density: 2 } takes 5\nShadow takes 0",
    ),
    (
        "wraith_possessed",
        "Solid { armor: 3 } takes 7\nMist { density: 2 } takes 5\nShadow takes 0\nPossessed { will: 4 } takes 14",
    ),
    (
        "chimera_heads",
        "fire breathes 7\nfrost breathes 5\nstorm breathes 9",
    ),
    (
        "chimera_bound",
        "fire x3 = 21\nfrost x3 = 15\nstorm x3 = 27",
    ),
    (
        "chimera_plasma",
        "fire breathes 7\nfrost breathes 5\nstorm breathes 9\nplasma breathes 12\ntotal: 33",
    ),
    ("and_then", "Some(2)\nNone"),
    (
        "wager_stake",
        "Ok(40)\nErr(ParseIntError { kind: InvalidDigit })",
    ),
    ("wager_from", "Ok(35)\nErr(BadStake)\nErr(TooPoor)"),
    (
        "wager_quest",
        "Ok(22)\nErr(BadMap)\nErr(Locked)\nErr(Broke)",
    ),
    ("hashmap_basic", "Some(7)\nNone"),
    (
        "golem_parts",
        "weak points: [\"arm (3)\", \"knee (4)\", \"eye (6)\"]",
    ),
    (
        "golem_sum",
        "golem: arm chest knee head eye\ncore damage: 26",
    ),
    (
        "golem_core",
        "golem: arm chest knee head eye\nblow 1: 13\nblow 3: 39",
    ),
    ("arc_mutex", "5"),
    ("mpsc_channel", "7\n11"),
    ("atomic", "10\n11"),
    (
        "hydra_senders",
        "fang down after 3 blows\nvenom down after 2 blows\nbrine down after 4 blows\n3 heads down",
    ),
    // The `main` of these two thread stages cannot run on the server, so
    // this native run is its behaviour proof. Both outputs are fixed
    // whatever order the threads run in: the reports are sorted, and a
    // head's hits reach the channel before its `Down`.
    (
        "hydra_threads",
        "brine down after 4 blows\nfang down after 3 blows\nvenom down after 2 blows\n3 heads down",
    ),
    ("hydra_body", "3 heads down after 9 hits\nstrike the body"),
    ("rc_basic", "1\n2\n1"),
    ("refcell", "10\n15"),
    ("cell", "1\n5"),
    ("rc_refcell", "15"),
    ("weak_ref", "Some(5)\nNone"),
    (
        "warden_box",
        "Entry(5, Entry(7, Entry(3, Closed)))\nowed: 15",
    ),
    (
        "warden_shared",
        "miller and smith paid\nvault holds 12\nhandles: 3",
    ),
    (
        "warden_weak",
        "circle: [\"warden\", \"you\", \"teller\"]\nwarden: strong 1, weak 1\nafter the ledger closes: None",
    ),
    (
        "raw_pointer",
        "pointer is null: false\ngold through the pointer: 15\ngold by name: 15",
    ),
    (
        "extern_c",
        "Rune: 12 bytes, align 4\ndirect call: 44\nthrough the extern block: 44\nthrough a fn pointer: 44",
    ),
    (
        "macro_rules",
        "  verse: 1\nfirst chant: 1\n  verse: 2\n  verse: 3\n  verse: 4\nsecond chant: 9\n  verse: 10\n  verse: 20\nthird chant: 30",
    ),
    (
        "drop_trait",
        "early is released\nthe ritual begins\nunnamed is released\ninner scope ends\ninner is released\nmain ends\nsecond is released\nfirst is released",
    ),
    (
        "send_sync",
        "the courier accepts the tome\npage reads 7\nthe page is returned to the shelf",
    ),
    (
        "maybe_uninit",
        "bound pages: [10, 20, 30, 40]\ntotal ink: 100",
    ),
    (
        "librarian_index",
        "index 0: volume 11\nindex 3: volume 44\nindex 4: no such volume\nindex 99: no such volume",
    ),
    (
        "librarian_demand",
        "the demand: 78\nincant!(1, 2, 3, 4): 78\nthe incantation matches: true\nincant!(7): 21\nincant!(2, 9): 39",
    ),
    (
        "librarian_seal",
        "page 0: Some(5)\npage 2: Some(13)\npage 3: None\nthe Librarian closes the book\nthe codex is sealed (3 pages returned)",
    ),
    // Act 10. The two allocation counts are `1` by std's documented
    // guarantee (exact capacity, no reallocation within it), and the same
    // in the server sandbox (`compile-api/tests/expectations_proof.rs`).
    ("prealloc", "coins: 1000\nallocations: 1"),
    (
        "cow_borrow",
        "Grimoire (borrowed)\nBook_of_Crowns (copied)\nLedger (borrowed)",
    ),
    (
        "const_eval",
        "star chart: [1, 2, 5, 10, 17, 26, 37, 50]\nbrightest: 50",
    ),
    (
        "chunks_exact",
        "choir sum: 55\nplain sum: 55\nempty choir: 0",
    ),
    (
        "no_std",
        "tower 7: lit (sent: true)\noverflow refused: true",
    ),
    (
        "semver_caret",
        "^1.2.3 accepts 1.9.0: true\n^1.2.3 accepts 2.0.0: false\n^1.2.3 accepts 1.2.2: false\n^0.2.3 accepts 0.2.9: true\n^0.2.3 accepts 0.3.0: false\n^0.0.3 accepts 0.0.3: true\n^0.0.3 accepts 0.0.4: false\n^1.2.3 vs 1.x.0: not a version",
    ),
    (
        "compiler_alloc",
        "Ferris, the Borrow Checker, Vexis, the Librarian\nallocations: 1",
    ),
    ("compiler_lifetime", "Ferris (borrowed)\nSir Ferris (owned)"),
    (
        "compiler_bound",
        "Some(9)\nSome(2.5)\nSome(\"vexis\")\nNone",
    ),
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
    let trial_ids: Vec<&str> = trial_hint_claims().iter().map(|(id, _, _)| *id).collect();
    for id in HINT_CLAIMS.iter().map(|(id, _, _)| *id).chain(trial_ids) {
        assert!(
            PINNED_OUTPUT.iter().any(|(p, _)| *p == id),
            "{id} is not pinned"
        );
    }
}

/// The real starter of `id` with each `(from, to)` replacement applied
/// (each `from` must occur in it), the way a tutorial or hint describes.
fn edited_starter(id: &str, edits: &[(&str, &str)]) -> String {
    let reg = MissionRegistry::default();
    let mut s = reg
        .find(id)
        .unwrap_or_else(|| panic!("`{id}` is not a mission"))
        .starter_code
        .to_string();
    for (from, to) in edits {
        assert!(
            s.contains(from),
            "`{id}` starter no longer contains {from:?}"
        );
        s = s.replacen(from, to, 1);
    }
    s
}

/// The player-facing solution of `id` with each `(from, to)` replacement
/// applied (each `from` must occur in it), for claims about a variation
/// of the finished program.
fn edited_solution(id: &str, edits: &[(&str, &str)]) -> String {
    let mut s = canonical_solution(id)
        .unwrap_or_else(|| panic!("no solution for `{id}`"))
        .to_string();
    for (from, to) in edits {
        assert!(
            s.contains(from),
            "`{id}` solution no longer contains {from:?}"
        );
        s = s.replacen(from, to, 1);
    }
    s
}

/// `(mission, program, stdout)` like [`HINT_CLAIMS`], for the trial
/// stages: the programs are the real starters (or solutions), edited the
/// way the text says.
fn trial_hint_claims() -> Vec<(&'static str, String, &'static str)> {
    vec![
        (
            // "As written, the starter compiles and prints `None` three
            // times and the closing line, nothing more."
            "librarian_seal",
            edited_starter("librarian_seal", &[]),
            "page 0: None\npage 2: None\npage 3: None\nthe Librarian closes the book",
        ),
        (
            // "folding from the right gives `48`, not `78`."
            "librarian_demand",
            a9::WRONG_LIBRARIAN_DEMAND_0.to_string(),
            "the demand: 78\nincant!(1, 2, 3, 4): 48\nthe incantation matches: false\nincant!(7): 21\nincant!(2, 9): 39",
        ),
        (
            // "Dropping the chest and the head by name would print the
            // same line here."
            "golem_parts",
            edited_starter(
                "golem_parts",
                &[(
                    ".filter(|p| !p.armoured)",
                    ".filter(|l| !l.starts_with(\"chest\") && !l.starts_with(\"head\"))",
                )],
            ),
            "weak points: [\"arm (3)\", \"knee (4)\", \"eye (6)\"]",
        ),
        (
            // "summing every part, armour and all, prints
            // `core damage: 52`."
            "golem_sum",
            edited_starter(
                "golem_sum",
                &[
                    ("let total = parts", "let total: u32 = parts"),
                    ("        .filter(|p| !p.armoured)\n", ""),
                ],
            ),
            "golem: arm chest knee head eye\ncore damage: 52",
        ),
        (
            // "As written, the starter prints `blow 1: 3` and `blow 3: 9`."
            "golem_core",
            edited_starter("golem_core", &[]),
            "golem: arm chest knee head eye\nblow 1: 3\nblow 3: 9",
        ),
        (
            // "`.take_while(..)` in place of `.filter(..)` makes the same
            // mistake."
            "golem_core",
            edited_solution(
                "golem_core",
                &[(".filter(|p| !p.armoured)", ".take_while(|p| !p.armoured)")],
            ),
            "golem: arm chest knee head eye\nblow 1: 3\nblow 3: 9",
        ),
        (
            // "While any sender is alive ... `rx.recv_timeout(..)` on an
            // empty channel gives `Err(Timeout)`. Once the last sender is
            // dropped it gives `Err(Disconnected)`."
            "hydra_senders",
            "use std::sync::mpsc;\nuse std::time::Duration;\nfn main() {\n    let (tx, rx) = mpsc::channel::<u32>();\n    tx.send(1).unwrap();\n    let wait = Duration::from_millis(50);\n    println!(\"{:?}\", rx.recv_timeout(wait));\n    println!(\"{:?}\", rx.recv_timeout(wait));\n    drop(tx);\n    println!(\"{:?}\", rx.recv_timeout(wait));\n}\n"
                .to_string(),
            "Ok(1)\nErr(Timeout)\nErr(Disconnected)",
        ),
        (
            // "As written, the starter prints `warden: strong 2, weak 0`
            // and `after the ledger closes: Some(\"warden\")`."
            "warden_weak",
            edited_starter("warden_weak", &[]),
            "circle: [\"warden\", \"you\", \"teller\"]\nwarden: strong 2, weak 0\nafter the ledger closes: Some(\"warden\")",
        ),
        (
            // "The compiler's help writes `Wraith::Solid { armor: _ }`. That
            // compiles, but ... the program then prints
            // `Solid { armor: 3 } takes 10`."
            "wraith_essence",
            edited_starter(
                "wraith_essence",
                &[
                    (
                        "Wraith::Solid => blow",
                        "Wraith::Solid { armor: _ } => blow",
                    ),
                    (
                        "Wraith::Mist => blow",
                        "Wraith::Mist { density: _ } => blow",
                    ),
                ],
            ),
            "Solid { armor: 3 } takes 10\nMist { density: 2 } takes 10\nShadow takes 0",
        ),
        (
            // "A `_ => 0` arm compiles, but the program then prints
            // `Possessed { will: 4 } takes 0`."
            "wraith_possessed",
            edited_starter(
                "wraith_possessed",
                &[(
                    "        Wraith::Shadow => 0,\n",
                    "        Wraith::Shadow => 0,\n        _ => 0,\n",
                )],
            ),
            "Solid { armor: 3 } takes 7\nMist { density: 2 } takes 5\nShadow takes 0\nPossessed { will: 4 } takes 0",
        ),
        (
            // "The same methods in a plain `impl Frost` block print the same
            // lines."
            "chimera_heads",
            edited_starter(
                "chimera_heads",
                &[(
                    "\nfn main() {",
                    "\nimpl Frost {\n    fn name(&self) -> &'static str {\n        \"frost\"\n    }\n    fn breath(&self) -> u32 {\n        5\n    }\n}\n\nimpl Storm {\n    fn name(&self) -> &'static str {\n        \"storm\"\n    }\n    fn breath(&self) -> u32 {\n        9\n    }\n}\n\nfn main() {",
                )],
            ),
            "fire breathes 7\nfrost breathes 5\nstorm breathes 9",
        ),
    ]
}

/// Pinned missions whose solution spawns threads. Their output must not
/// depend on scheduling, so the built program is run again
/// `THREADED_RERUNS` times and must print the same lines each time.
const THREADED_PINNED: &[&str] = &["hydra_threads", "hydra_body"];
const THREADED_RERUNS: usize = 20;

/// Run the program `run_native` already built as `name` again.
fn rerun_native(dir: &std::path::Path, name: &str) -> String {
    let run = Command::new(dir.join(format!("{name}.exe")))
        .output()
        .expect("rerun program");
    assert!(
        run.status.success(),
        "{name} did not exit cleanly on a rerun"
    );
    normalise_stdout(&run.stdout)
}

fn normalise_stdout(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
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
    normalise_stdout(&run.stdout)
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
        // Thread stages: the same output on every run, not just the first.
        if THREADED_PINNED.contains(id) {
            for run in 0..THREADED_RERUNS {
                let again = rerun_native(&dir, id);
                if again != *want {
                    problems.push(format!(
                        "`{id}` rerun {run} prints {again:?}, Task says {want:?}"
                    ));
                }
            }
        }
    }
    let trial = trial_hint_claims();
    let all_hints = HINT_CLAIMS
        .iter()
        .map(|(id, src, want)| (*id, *src, *want))
        .chain(
            trial
                .iter()
                .map(|(id, src, want)| (*id, src.as_str(), *want)),
        );
    for (n, (id, src, want)) in all_hints.enumerate() {
        let got = run_native(&dir, &format!("hint_{n}_{id}"), src);
        if got != want {
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
    // Acts 1-2 are the 21 prelude missions plus the three stages of the
    // Borrow Checker's trial (the Act 2 boss), which ends the demo; Act 3
    // opens with `impl_method`.
    assert_eq!(DEMO_LAST_MISSION_ID, "bridge_lantern");
    assert_eq!(idx + 1, 24, "demo mission count");
    assert_eq!(reg.missions[idx - 3].id, "while_loop");
    assert_eq!(reg.missions[idx + 1].id, "impl_method");
    assert_eq!(
        reg.missions.len() - (idx + 1),
        72,
        "missions after the demo"
    );
    assert_eq!(reg.missions.len(), 96, "missions in the full game");
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

// ── Hard rule 3: what trial owners say about the compiler ────────────

use pledge_and_crown::plugins::dialogue::lines::{AUTHORED, TRIAL_STAGES};
use pledge_and_crown::plugins::stub_grader::MISSION_RULES;
use pledge_and_crown::plugins::trial::TRIALS;

/// What a claim says rustc (or the built program) does.
enum Claimed {
    /// Fails with only this error code, and stderr contains every piece
    /// (the first piece is the wording the owner quotes).
    Error(&'static str, &'static [&'static str]),
    /// Fails with errors that carry no code (macro and syntax errors), and
    /// stderr contains every piece (the first is the wording quoted).
    ErrorNoCode(&'static [&'static str]),
    /// Compiles (the claim is "this also compiles").
    Compiles,
    /// Compiles, runs to a clean exit and prints exactly this.
    Prints(&'static str),
    /// Compiles, prints exactly `stdout`, then panics: exits unsuccessfully
    /// with `stderr` in its standard error.
    Panics {
        stdout: &'static str,
        stderr: &'static str,
    },
}

/// Every statement the owner of trial `trial` makes about compiler
/// behaviour in it (tutorials, hints, dialogue, grader messages), with the
/// program it is about. `(label, program, claim)`. Programs are the real
/// starters, edited the way the text describes.
fn trial_claims(trial: &str) -> Vec<(&'static str, String, Claimed)> {
    let s = |id: &str| edited_starter(id, &[]);
    match trial {
        "bridge" => vec![
            (
                "bridge_toll starter: E0382, borrow of moved value",
                s("bridge_toll"),
                Claimed::Error("E0382", &["borrow of moved value: `cargo`"]),
            ),
            (
                "bridge_toll hint: `weigh(cargo.clone())` also compiles",
                edited_starter("bridge_toll", &[("weigh(cargo)", "weigh(cargo.clone())")]),
                Claimed::Compiles,
            ),
            (
                "bridge_quill starter: E0502 with the quoted wording",
                s("bridge_quill"),
                Claimed::Error(
                    "E0502",
                    &["cannot borrow `scroll` as mutable because it is also borrowed as immutable"],
                ),
            ),
            (
                "bridge_lantern starter: E0106, and rustc's help ties `sep` to 'a too",
                s("bridge_lantern"),
                Claimed::Error(
                    "E0106",
                    &[
                        "missing lifetime specifier",
                        "fn before<'a>(line: &'a str, sep: &'a str) -> &'a str",
                    ],
                ),
            ),
            (
                "bridge_lantern: following that help makes `main` fail with E0597",
                edited_starter(
                    "bridge_lantern",
                    &[(
                        "fn before(line: &str, sep: &str) -> &str",
                        "fn before<'a>(line: &'a str, sep: &'a str) -> &'a str",
                    )],
                ),
                Claimed::Error("E0597", &["`sep` does not live long enough"]),
            ),
        ],
        "variant_wraith" => vec![
            (
                "wraith_forms: without the `_` arm and no Shadow arm, E0004",
                edited_starter(
                    "wraith_forms",
                    &[(
                        "        // this runs, but `_` answers a form it never looked at\n        _ => \"steel\",\n",
                        "",
                    )],
                ),
                Claimed::Error(
                    "E0004",
                    &["non-exhaustive patterns: `Wraith::Shadow` not covered"],
                ),
            ),
            (
                "wraith_essence starter: E0533, and rustc's help binds `armor: _`",
                s("wraith_essence"),
                Claimed::Error(
                    "E0533",
                    &[
                        "expected unit struct, unit variant or constant, found struct variant `Wraith::Solid`",
                        "Wraith::Solid { armor: _ }",
                    ],
                ),
            ),
            (
                "wraith_essence hint: following that help compiles",
                edited_starter(
                    "wraith_essence",
                    &[
                        (
                            "Wraith::Solid => blow",
                            "Wraith::Solid { armor: _ } => blow",
                        ),
                        (
                            "Wraith::Mist => blow",
                            "Wraith::Mist { density: _ } => blow",
                        ),
                    ],
                ),
                Claimed::Compiles,
            ),
            (
                "wraith_possessed starter: E0004 names the new form",
                s("wraith_possessed"),
                Claimed::Error(
                    "E0004",
                    &["non-exhaustive patterns: `Wraith::Possessed { .. }` not covered"],
                ),
            ),
            (
                "wraith_possessed: a blanket `_ => 0` arm compiles",
                edited_starter(
                    "wraith_possessed",
                    &[(
                        "        Wraith::Shadow => 0,\n",
                        "        Wraith::Shadow => 0,\n        _ => 0,\n",
                    )],
                ),
                Claimed::Compiles,
            ),
            (
                "wraith_possessed: with `_`, the next new form compiles unnamed too",
                edited_starter(
                    "wraith_possessed",
                    &[
                        (
                            "        Wraith::Shadow => 0,\n",
                            "        Wraith::Shadow => 0,\n        _ => 0,\n",
                        ),
                        (
                            "    Possessed { will: u32 },\n",
                            "    Possessed { will: u32 },\n    Bound,\n",
                        ),
                        (
                            "        Wraith::Possessed { will: 4 },\n",
                            "        Wraith::Possessed { will: 4 },\n        Wraith::Bound,\n",
                        ),
                    ],
                ),
                Claimed::Compiles,
            ),
        ],
        "chimera" => vec![
            (
                "chimera_heads starter: E0599 on a head without the trait",
                s("chimera_heads"),
                Claimed::Error(
                    "E0599",
                    &["no method named `name` found for struct `Frost` in the current scope"],
                ),
            ),
            (
                "chimera_bound starter: E0599 on an unbounded T, help suggests the bound",
                s("chimera_bound"),
                Claimed::Error(
                    "E0599",
                    &[
                        "no method named `name` found for reference `&T` in the current scope",
                        "fn volley<T: Enemy>(head: &T, rounds: u32) -> String",
                    ],
                ),
            ),
            (
                "chimera_plasma starter: E0308, battle expects `&[Head]`",
                s("chimera_plasma"),
                Claimed::Error(
                    "E0308",
                    &[
                        "mismatched types",
                        "expected `&[Head]`, found `&Vec<Box<dyn Enemy>>`",
                    ],
                ),
            ),
        ],
        "wager" => vec![
            (
                "wager_stake starter: E0277, `?` in a function returning u32",
                s("wager_stake"),
                Claimed::Error(
                    "E0277",
                    &[
                        "the `?` operator can only be used in a function that returns `Result` or `Option`",
                    ],
                ),
            ),
            (
                "wager_stake hint: `.unwrap()` compiles with the old signature, then panics on `twenty`",
                edited_starter(
                    "wager_stake",
                    &[("read_stake(text)?", "read_stake(text).unwrap()")],
                ),
                Claimed::Panics {
                    stdout: "40",
                    stderr: "called `Result::unwrap()` on an `Err` value",
                },
            ),
            (
                "wager_from starter: E0277, `?` cannot convert without `From`",
                s("wager_from"),
                Claimed::Error(
                    "E0277",
                    &[
                        "`?` couldn't convert the error to `WagerError`",
                        "the trait `From<ParseIntError>` is not implemented for `WagerError`",
                    ],
                ),
            ),
            (
                "wager_from hint: `.map_err(..)` before each `?` also compiles",
                edited_starter(
                    "wager_from",
                    &[
                        (
                            "text.trim().parse::<u32>()?",
                            "text.trim().parse::<u32>().map_err(|_| WagerError::BadStake)?",
                        ),
                        (
                            "pay(purse, stake)?",
                            "pay(purse, stake).map_err(|_| WagerError::TooPoor)?",
                        ),
                    ],
                ),
                Claimed::Compiles,
            ),
            (
                "wager_quest starter: prints Ok(22), then the unwrap panics",
                s("wager_quest"),
                Claimed::Panics {
                    stdout: "Ok(22)",
                    stderr: "called `Result::unwrap()` on an `Err` value",
                },
            ),
        ],
        "forge_golem" => vec![
            (
                "golem_parts starter: E0609, a label has no `armoured` field",
                s("golem_parts"),
                Claimed::Error("E0609", &["no field `armoured` on type `&String`"]),
            ),
            (
                "golem_sum starter: E0283, and rustc's help names `total: u32`",
                s("golem_sum"),
                Claimed::Error(
                    "E0283",
                    &["type annotations needed", "let total: u32 = parts"],
                ),
            ),
            (
                "golem_sum: `.fold(0, |acc, w| acc + w)` in place of `.sum()` compiles",
                edited_starter("golem_sum", &[(".sum();", ".fold(0, |acc, w| acc + w);")]),
                Claimed::Compiles,
            ),
        ],
        "tide_hydra" => vec![
            (
                "hydra_senders starter: E0382 in the loop, rustc's help clones",
                s("hydra_senders"),
                Claimed::Error(
                    "E0382",
                    &[
                        "use of moved value: `tx`",
                        "value moved here, in previous iteration of loop",
                        "fight(head, hp, tx.clone());",
                    ],
                ),
            ),
            (
                "hydra_senders hint: lending `&tx` to a `fight` that borrows compiles",
                edited_starter(
                    "hydra_senders",
                    &[
                        ("tx: Sender<String>)", "tx: &Sender<String>)"),
                        ("fight(head, hp, tx);", "fight(head, hp, &tx);"),
                    ],
                ),
                Claimed::Compiles,
            ),
            (
                "hydra_threads starter: E0373 on `hp`, rustc's help says `move`",
                s("hydra_threads"),
                Claimed::Error(
                    "E0373",
                    &[
                        "closure may outlive the current function, but it borrows `hp`, which is owned by the current function",
                        "use the `move` keyword",
                    ],
                ),
            ),
        ],
        "memory_warden" => vec![
            (
                "warden_box starter: E0072, and rustc's help quoted in the text",
                s("warden_box"),
                Claimed::Error(
                    "E0072",
                    &[
                        "recursive type `Ledger` has infinite size",
                        "insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle",
                    ],
                ),
            ),
            (
                "warden_shared starter: E0594 through the `Rc`",
                s("warden_shared"),
                Claimed::Error("E0594", &["cannot assign to data in an `Rc`"]),
            ),
            (
                "warden_shared hint: a `borrow()` held across `pay` panics at run time",
                edited_solution(
                    "warden_shared",
                    &[(
                        "    pay(&miller, 5);\n",
                        "    let open = vault.borrow();\n    pay(&miller, 5);\n",
                    )],
                ),
                Claimed::Panics {
                    stdout: "",
                    stderr: "RefCell already borrowed",
                },
            ),
        ],
        "librarian" => vec![
            (
                "librarian_index starter: E0133, the call to the `unsafe fn`",
                s("librarian_index"),
                Claimed::Error(
                    "E0133",
                    &["call to unsafe function `volume_at` is unsafe and requires unsafe block"],
                ),
            ),
            (
                "librarian_index hint: wrapping the call in `unsafe` alone compiles",
                edited_starter(
                    "librarian_index",
                    &[(
                        "Some(volume_at(shelf.as_ptr(), index))",
                        "Some(unsafe { volume_at(shelf.as_ptr(), index) })",
                    )],
                ),
                Claimed::Compiles,
            ),
            (
                "librarian_demand starter: no rules expected `,`",
                s("librarian_demand"),
                Claimed::ErrorNoCode(&["no rules expected `,`"]),
            ),
            (
                "librarian_demand: `@` never starts an expression, so the entry \
                 rule written first still works",
                a9::CLAIM_LIBRARIAN_DEMAND_ORDER.to_string(),
                Claimed::Prints("12"),
            ),
            (
                "librarian_demand syntax: `count!(5, 6, 7)` is `3`",
                a9::CLAIM_COUNT_SYNTAX.to_string(),
                Claimed::Prints("3"),
            ),
        ],
        // The Compiler. His dialogue quotes no compiler output (the result
        // pane shows the player's real stderr); these are the claims in
        // his stages' tutorials and his one framing claim ("my help mends
        // the signature; it does not know your callers": the E0106 help,
        // followed, compiles `herald` and moves the error to `main`).
        "compiler" => vec![
            (
                "compiler_alloc: the starter's count is above 1 (12 on rustc \
                 1.99.0; std does not promise a number, so no text quotes it)",
                s("compiler_alloc"),
                Claimed::Prints(
                    "Ferris, the Borrow Checker, Vexis, the Librarian\nallocations: 12",
                ),
            ),
            (
                "compiler_alloc hint: `names.join(\", \")` also measures 1 here",
                a10::ALT_COMPILER_ALLOC_0.to_string(),
                Claimed::Prints("Ferris, the Borrow Checker, Vexis, the Librarian\nallocations: 1"),
            ),
            (
                "compiler_alloc syntax: 11 bytes fit `with_capacity(11)`, same buffer",
                a10::CLAIM_WITH_CAPACITY.to_string(),
                Claimed::Prints("crown, seal: 11 bytes, same buffer: true"),
            ),
            (
                "compiler_lifetime starter: E0106, and the help ties both inputs",
                s("compiler_lifetime"),
                Claimed::Error(
                    "E0106",
                    &[
                        "missing lifetime specifier",
                        "the signature does not say whether it is borrowed from `name` or `title`",
                        "fn herald<'a>(name: &'a str, title: &'a str) -> Cow<'a, str>",
                    ],
                ),
            ),
            (
                "compiler_lifetime: the help followed literally, `main` fails with E0597",
                a10::WRONG_COMPILER_LIFETIME_1.to_string(),
                Claimed::Error(
                    "E0597",
                    &[
                        "`title` does not live long enough",
                        "`title` dropped here while still borrowed",
                    ],
                ),
            ),
            (
                "compiler_lifetime syntax: `pick` borrows only `keep`",
                a10::CLAIM_PICK_SYNTAX.to_string(),
                Claimed::Prints("[crown]"),
            ),
            (
                "compiler_bound starter: E0369, and the help names `PartialOrd`",
                s("compiler_bound"),
                Claimed::Error(
                    "E0369",
                    &[
                        "binary operation `>` cannot be applied to type `&T`",
                        "consider restricting type parameter `T` with trait `PartialOrd`",
                    ],
                ),
            ),
            (
                "compiler_bound hint: `Ord` with `.iter().max()` refuses the floats",
                a10::WRONG_COMPILER_BOUND_1.to_string(),
                Claimed::Error(
                    "E0277",
                    &["the trait bound `{float}: Ord` is not satisfied"],
                ),
            ),
            (
                "compiler_bound: `sort` and `Iterator::max` need `Ord`",
                a10::CLAIM_FLOAT_ORD.to_string(),
                Claimed::Error(
                    "E0277",
                    &["the trait bound `{float}: Ord` is not satisfied"],
                ),
            ),
            (
                "compiler_bound: NaN is not equal to itself, every comparison is false",
                a10::CLAIM_NAN.to_string(),
                Claimed::Prints(
                    "nan == nan: false\nnan < 1.0: false\nnan > 1.0: false\npartial_cmp: None",
                ),
            ),
            (
                "compiler_bound syntax: one `PartialOrd` bound, three kinds of `T`",
                a10::CLAIM_BIGGER_SYNTAX.to_string(),
                Claimed::Prints("9 2.5 vexis"),
            ),
        ],
        other => panic!("no claims table for trial `{other}`: add one"),
    }
}

/// The error codes each trial owner names, in order. A code added to any
/// owner's text fails `every_error_a_trial_owner_names_is_a_compiled_claim`
/// until it is claimed (and compiled) here.
const NAMED_CODES: &[(&str, &[&str])] = &[
    ("bridge", &["E0106", "E0382", "E0502", "E0597"]),
    ("variant_wraith", &["E0004", "E0533"]),
    ("chimera", &["E0308", "E0599"]),
    ("wager", &["E0277"]),
    ("forge_golem", &["E0283", "E0609"]),
    ("tide_hydra", &["E0373", "E0382"]),
    ("memory_warden", &["E0072", "E0594"]),
    ("librarian", &["E0133"]),
    ("compiler", &["E0106", "E0277", "E0369", "E0597"]),
];

/// Error codes in `text` (`E` followed by exactly four digits).
fn error_codes(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    (0..b.len().saturating_sub(4))
        .filter(|&i| b[i] == b'E' && b[i + 1..i + 5].iter().all(u8::is_ascii_digit))
        .filter(|&i| i + 5 >= b.len() || !b[i + 5].is_ascii_digit())
        .map(|i| text[i..i + 5].to_string())
        .collect()
}

/// Everything said in missions `ids` (each once): prompts and tutorials,
/// authored dialogue rows, grader messages.
fn mission_texts(ids: &[&str]) -> Vec<String> {
    let reg = MissionRegistry::default();
    let mut seen: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for id in ids {
        if seen.contains(id) {
            continue;
        }
        seen.push(id);
        let m = reg.find(id).expect("mission");
        out.push(m.prompt.to_string());
        out.push(m.tutorial.to_string());
        for row in AUTHORED
            .iter()
            .chain(TRIAL_STAGES)
            .filter(|r| r.npc_id == *id)
        {
            for set in [
                row.lines.not_yet,
                row.lines.offer,
                row.lines.in_progress,
                row.lines.cleared,
            ] {
                out.extend(set.iter().map(|l| l.to_string()));
            }
        }
        for rule in MISSION_RULES.iter().filter(|r| r.id == *id) {
            out.push(rule.npc.to_string());
            out.push(rule.pass.to_string());
            out.extend(rule.checks.iter().map(|c| c.hint.to_string()));
        }
    }
    out
}

/// Everything the owner of `trial` says: their prologue and stage
/// missions' prompts and tutorials, their dialogue rows, their grader
/// messages, the trial-complete line.
fn owner_texts(trial: &pledge_and_crown::plugins::trial::Trial) -> Vec<String> {
    let ids: Vec<&str> = std::iter::once(trial.npc)
        .chain(trial.stages.iter().copied())
        .collect();
    let mut out = mission_texts(&ids);
    out.push(trial.complete_line.to_string());
    out
}

/// Every error code named in `texts` has a compiled claim, every claim's
/// code and quoted wording is actually said, and the codes named are
/// exactly `want` (so a newly named code fails until it is claimed).
fn assert_claims_cover_texts(
    subject: &str,
    claims: &[(&'static str, String, Claimed)],
    texts: &[String],
    want: &[&str],
) {
    let claimed: Vec<&str> = claims
        .iter()
        .filter_map(|(_, _, c)| match c {
            Claimed::Error(code, _) => Some(*code),
            Claimed::ErrorNoCode(_)
            | Claimed::Compiles
            | Claimed::Prints(_)
            | Claimed::Panics { .. } => None,
        })
        .collect();
    let mut named = std::collections::BTreeSet::new();
    for t in texts {
        for code in error_codes(t) {
            assert!(
                claimed.contains(&code.as_str()),
                "{subject}: names {code} with no compiled claim:\n{t}"
            );
            named.insert(code);
        }
    }
    // No stale claims: every claimed code and quoted wording is said.
    for (label, _, c) in claims {
        match c {
            Claimed::Error(code, pieces) => {
                assert!(named.contains(*code), "{label}: {code} is never said");
                assert!(
                    texts.iter().any(|t| t.contains(pieces[0])),
                    "{label}: wording {:?} is never said",
                    pieces[0]
                );
            }
            Claimed::ErrorNoCode(pieces) => assert!(
                texts.iter().any(|t| t.contains(pieces[0])),
                "{label}: wording {:?} is never said",
                pieces[0]
            ),
            Claimed::Panics { stderr, .. } => assert!(
                texts.iter().any(|t| t.contains("panic")),
                "{label}: a panic ({stderr}) is claimed but never said"
            ),
            Claimed::Compiles | Claimed::Prints(_) => {}
        }
    }
    assert_eq!(
        named.into_iter().collect::<Vec<_>>(),
        want,
        "codes `{subject}` names"
    );
}

#[test]
fn every_error_a_trial_owner_names_is_a_compiled_claim() {
    assert_eq!(
        TRIALS.iter().map(|t| t.id).collect::<Vec<_>>(),
        NAMED_CODES.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
        "every trial has a claims table"
    );
    for trial in TRIALS {
        let want = NAMED_CODES
            .iter()
            .find(|(t, _)| *t == trial.id)
            .map(|(_, codes)| *codes)
            .unwrap_or_default();
        assert_claims_cover_texts(trial.id, &trial_claims(trial.id), &owner_texts(trial), want);
    }
}

// ── Hard rule 3 for the Act 9 villagers ──────────────────────────────
//
// The six Act 9 villagers teach `unsafe`, FFI, macros, `Drop`, `Send` /
// `Sync` and `MaybeUninit`; their tutorials and hints name compiler errors
// and describe what programs do. Each claim is compiled (and, where it is
// about behaviour, run) here, like the trial owners' above. Statements
// that something is undefined behaviour cannot be run; they follow the
// std and Reference documentation and the spec's soundness notes (no Miri
// run backs them).

/// The Act 9 villager missions, with the error codes their text names.
const ACT9_VILLAGER_CODES: &[(&str, &[&str])] = &[
    ("raw_pointer", &["E0133", "E0499"]),
    ("extern_c", &["E0133"]),
    ("macro_rules", &[]),
    ("drop_trait", &["E0040"]),
    ("send_sync", &["E0277"]),
    ("maybe_uninit", &["E0381"]),
];

/// The solution of `macro_rules` with the macro moved below `main`.
fn macro_defined_after_use() -> String {
    let sol = canonical_solution("macro_rules").expect("macro_rules solution");
    let at = sol.find("fn main()").expect("solution has fn main");
    let (mac, main) = sol.split_at(at);
    format!("{main}\n{mac}")
}

/// Every claim the Act 9 villager of mission `id` makes.
fn act9_villager_claims(id: &str) -> Vec<(&'static str, String, Claimed)> {
    let s = |id: &str| edited_starter(id, &[]);
    match id {
        "raw_pointer" => vec![
            (
                "raw_pointer starter: E0133, deref outside `unsafe`",
                s("raw_pointer"),
                Claimed::Error(
                    "E0133",
                    &["dereference of raw pointer is unsafe and requires unsafe block"],
                ),
            ),
            (
                "raw_pointer: the borrow checker still applies inside `unsafe`",
                a9::CLAIM_RAW_POINTER_BORROWCK.to_string(),
                // The text names the code only; rustc's wording follows.
                Claimed::Error(
                    "E0499",
                    &[
                        "E0499",
                        "cannot borrow `gold` as mutable more than once at a time",
                    ],
                ),
            ),
        ],
        "extern_c" => vec![
            (
                "extern_c starter: E0133, calling the foreign declaration",
                s("extern_c"),
                Claimed::Error(
                    "E0133",
                    &[
                        "E0133",
                        "call to unsafe function `foreign::crown_seal` is unsafe and requires unsafe block",
                    ],
                ),
            ),
            (
                "extern_c: a plain `extern \"C\" {` block fails in edition 2024",
                a9::CLAIM_EXTERN_C_PLAIN_BLOCK.to_string(),
                Claimed::ErrorNoCode(&["extern blocks must be unsafe"]),
            ),
            (
                "extern_c: a plain `#[no_mangle]` fails in edition 2024",
                a9::CLAIM_EXTERN_C_PLAIN_NO_MANGLE.to_string(),
                Claimed::ErrorNoCode(&["unsafe attribute used without unsafe"]),
            ),
        ],
        "macro_rules" => vec![
            (
                "macro_rules starter: the macro does not exist",
                s("macro_rules"),
                Claimed::ErrorNoCode(&["cannot find macro `chant` in this scope"]),
            ),
            (
                "macro_rules: defined below its first use, it is not found",
                macro_defined_after_use(),
                Claimed::ErrorNoCode(&["cannot find macro `chant` in this scope"]),
            ),
            (
                "macro_rules: `$x` written twice runs the expression twice",
                a9::CLAIM_MACRO_RULES_TWICE.to_string(),
                Claimed::Prints("3"),
            ),
            (
                "macro_rules: the template's `total` does not touch the caller's",
                a9::CLAIM_MACRO_RULES_HYGIENE.to_string(),
                Claimed::Prints("100 3"),
            ),
        ],
        "drop_trait" => vec![
            (
                "drop_trait starter: E0040, and rustc's help is `drop(early)`",
                s("drop_trait"),
                Claimed::Error(
                    "E0040",
                    &["explicit use of destructor method", "drop(early);"],
                ),
            ),
            (
                "drop_trait: `std::process::exit` runs no destructor",
                a9::CLAIM_DROP_EXIT.to_string(),
                Claimed::Prints("leaving"),
            ),
        ],
        "send_sync" => vec![
            (
                "send_sync starter: E0277 for `Send` and for `Sync`",
                s("send_sync"),
                Claimed::Error(
                    "E0277",
                    &[
                        "`*mut u32` cannot be sent between threads safely",
                        "`*mut u32` cannot be shared between threads safely",
                    ],
                ),
            ),
            (
                "send_sync: `Rc` is neither `Send` nor `Sync`",
                a9::CLAIM_SEND_SYNC_RC.to_string(),
                Claimed::Error(
                    "E0277",
                    &[
                        "cannot be sent between threads safely",
                        "`Rc<u32>` cannot be sent between threads safely",
                        "`Rc<u32>` cannot be shared between threads safely",
                    ],
                ),
            ),
        ],
        "maybe_uninit" => vec![
            (
                "maybe_uninit starter: E0381, reported at the read in `println!`",
                s("maybe_uninit"),
                Claimed::Error(
                    "E0381",
                    &[
                        "used binding `quire` isn't initialized",
                        "println!(\"bound pages: {quire:?}\");",
                    ],
                ),
            ),
            (
                "maybe_uninit: a `MaybeUninit` never drops what it holds",
                a9::CLAIM_MAYBE_UNINIT_NO_DROP.to_string(),
                Claimed::Prints("slot goes out of scope"),
            ),
        ],
        other => panic!("no claims table for Act 9 mission `{other}`: add one"),
    }
}

#[test]
fn every_error_an_act9_villager_names_is_a_compiled_claim() {
    let reg = MissionRegistry::default();
    let act9: Vec<&str> = reg
        .missions
        .iter()
        .map(|m| m.id)
        .skip_while(|id| *id != "raw_pointer")
        .take_while(|id| *id != "librarian_index")
        .collect();
    assert_eq!(
        act9,
        ACT9_VILLAGER_CODES
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>(),
        "every Act 9 villager mission has a claims table"
    );
    for (id, want) in ACT9_VILLAGER_CODES {
        assert_claims_cover_texts(id, &act9_villager_claims(id), &mission_texts(&[id]), want);
    }
}

/// The code pane shows 72 columns before it scrolls: every line of a trial
/// stage's starter, solution and tutorial code blocks fits.
#[test]
fn trial_code_lines_fit_the_code_pane() {
    const PANE_COLUMNS: usize = 72;
    /// Bridge lines that predate this check. Shortening the starter line
    /// changes the grader snapshots' starter case, so these wait for the
    /// next Bridge text pass; no other line may join them.
    const KNOWN_LONG: &[&str] = &[
        "    // this won't compile yet: weigh takes the cargo, then main reaches for it",
        "// `&mut` is the one exclusive borrow: only `inscribe` may write while it runs.",
        "    } // `sep` is dropped here, but `part` borrows from `lantern`, not `sep`.",
    ];
    let reg = MissionRegistry::default();
    let mut long = Vec::new();
    for trial in TRIALS {
        for id in trial.stages {
            let m = reg.find(id).expect("stage");
            let fenced: Vec<&str> = m
                .tutorial
                .split("```")
                .skip(1)
                .step_by(2)
                .flat_map(str::lines)
                .collect();
            let solution = canonical_solution(id).unwrap_or_default();
            for (what, line) in m
                .starter_code
                .lines()
                .map(|l| ("starter", l))
                .chain(solution.lines().map(|l| ("solution", l)))
                .chain(fenced.into_iter().map(|l| ("tutorial", l)))
            {
                if line.chars().count() > PANE_COLUMNS && !KNOWN_LONG.contains(&line) {
                    long.push(format!("{id} {what} ({}): {line}", line.chars().count()));
                }
            }
        }
    }
    assert!(
        long.is_empty(),
        "lines wider than the code pane:\n{}",
        long.join("\n")
    );
}

/// Build and (where the claim is about behaviour) run one claim; push a
/// line to `problems` if rustc or the program disagrees with it.
fn check_claim(
    dir: &std::path::Path,
    tag: &str,
    label: &str,
    src: &str,
    claim: &Claimed,
    problems: &mut Vec<String>,
) {
    let path = dir.join(format!("{tag}.rs"));
    let exe = dir.join(format!("{tag}.exe"));
    std::fs::write(&path, src).expect("write claim");
    let out = Command::new("rustc")
        .args(["--edition", PLAYER_EDITION, "--crate-type", "bin", "-o"])
        .arg(&exe)
        .arg(&path)
        .output()
        .expect("spawn rustc");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let codes: Vec<&str> = stderr
        .match_indices("error[")
        .map(|(i, _)| &stderr[i + 6..i + 11])
        .collect();
    match claim {
        Claimed::Compiles if out.status.success() => println!("ok  {label}: compiles"),
        Claimed::Compiles => problems.push(format!("{label}: does not compile\n{stderr}")),
        Claimed::Error(code, pieces) => {
            let wording = pieces.iter().all(|p| stderr.contains(p));
            let only = !codes.is_empty() && codes.iter().all(|c| c == code);
            if !out.status.success() && only && wording {
                println!("ok  {label}: {codes:?}");
            } else {
                problems.push(format!(
                    "{label}: expected only {code} with {pieces:?}, got {codes:?}\n{stderr}"
                ));
            }
        }
        Claimed::ErrorNoCode(pieces) => {
            let wording = pieces.iter().all(|p| stderr.contains(p));
            if !out.status.success() && codes.is_empty() && wording {
                println!("ok  {label}: fails without a code");
            } else {
                problems.push(format!(
                    "{label}: expected a code-less error with {pieces:?}, got {codes:?}\n{stderr}"
                ));
            }
        }
        Claimed::Prints(want) => {
            if !out.status.success() {
                problems.push(format!("{label}: does not compile\n{stderr}"));
                return;
            }
            let run = Command::new(&exe).output().expect("run claim");
            let got = normalise_stdout(&run.stdout);
            if run.status.success() && got == *want {
                println!("ok  {label}: prints {want:?}");
            } else {
                problems.push(format!(
                    "{label}: expected {want:?} and a clean exit; exit {:?}, stdout {got:?}",
                    run.status.code()
                ));
            }
        }
        Claimed::Panics {
            stdout,
            stderr: piece,
        } => {
            if !out.status.success() {
                problems.push(format!("{label}: does not compile\n{stderr}"));
                return;
            }
            let run = Command::new(&exe).output().expect("run claim");
            let run_out = String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n");
            let run_err = String::from_utf8_lossy(&run.stderr).into_owned();
            if !run.status.success() && run_out.trim_end() == *stdout && run_err.contains(piece) {
                println!("ok  {label}: panics after {stdout:?}");
            } else {
                problems.push(format!(
                    "{label}: expected {stdout:?} then a panic with {piece:?}; \
                     exit {:?}, stdout {run_out:?}, stderr {run_err}",
                    run.status.code()
                ));
            }
        }
    }
}

fn print_rustc_version() {
    let version = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("rustc --version");
    println!("{}", String::from_utf8_lossy(&version.stdout).trim());
}

#[test]
#[ignore = "slow: runs rustc once per claim"]
fn trial_claims_are_what_rustc_emits() {
    let dir = scratch_dir("trial-claims");
    print_rustc_version();
    let mut problems = Vec::new();
    let mut checked = 0usize;
    for trial in TRIALS {
        for (n, (label, src, claim)) in trial_claims(trial.id).into_iter().enumerate() {
            checked += 1;
            let tag = format!("{}_{n}", trial.id);
            check_claim(&dir, &tag, label, &src, &claim, &mut problems);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!("{checked} claims checked");
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
#[ignore = "slow: runs rustc once per claim"]
fn act9_villager_claims_are_what_rustc_emits() {
    let dir = scratch_dir("act9-claims");
    print_rustc_version();
    let mut problems = Vec::new();
    let mut checked = 0usize;
    for (id, _) in ACT9_VILLAGER_CODES {
        for (n, (label, src, claim)) in act9_villager_claims(id).into_iter().enumerate() {
            checked += 1;
            let tag = format!("{id}_{n}");
            check_claim(&dir, &tag, label, &src, &claim, &mut problems);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!("{checked} claims checked");
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

// ── Hard rule 3 for the Act 10 villagers ─────────────────────────────
//
// The six Act 10 villagers teach measuring allocations, `Cow`, `const fn`,
// `chunks_exact`, `no_std` and Cargo's caret rule. Their tutorials and
// hints name compiler errors and describe what programs print; each claim
// is compiled (and, where it is about behaviour, run) here. Allocation
// counts above 1 are std's growth strategy, which std does not promise:
// the claims pin what rustc 1.99.0 prints, and no player-facing text
// quotes those numbers. What the Cargo text says about Cargo (the caret
// table, `Cargo.lock`, the tools) follows the Cargo Book; nothing here
// runs Cargo.

/// The Act 10 villager missions, with the error codes their text names.
const ACT10_VILLAGER_CODES: &[(&str, &[&str])] = &[
    ("prealloc", &[]),
    ("cow_borrow", &[]),
    ("const_eval", &["E0015", "E0080"]),
    ("chunks_exact", &["E0658"]),
    ("no_std", &["E0425"]),
    ("semver_caret", &[]),
];

/// Every claim the Act 10 villager of mission `id` makes.
fn act10_villager_claims(id: &str) -> Vec<(&'static str, String, Claimed)> {
    let s = |id: &str| edited_starter(id, &[]);
    match id {
        "prealloc" => vec![
            (
                "prealloc hint: the scale reads more than 1 (9 on rustc 1.99.0; \
                 not a std guarantee, so no text quotes it)",
                s("prealloc"),
                Claimed::Prints("coins: 1000\nallocations: 9"),
            ),
            (
                "prealloc hint: collecting a range measures 1 here too",
                a10::ALT_PREALLOC_0.to_string(),
                Claimed::Prints("coins: 1000\nallocations: 1"),
            ),
        ],
        "cow_borrow" => vec![(
            "cow_borrow: `replace` always makes a new String; a Cow derefs to \
             `&str`; `into_owned` copies only a borrowed value",
            a10::CLAIM_COW_FACTS.to_string(),
            Claimed::Prints(
                "replace made a new string: true\nowned kept its buffer: true\n\
                 borrowed was copied: true\nlength through deref: 8",
            ),
        )],
        "const_eval" => vec![
            (
                "const_eval starter: E0015, a `const` calls a non-const fn",
                s("const_eval"),
                Claimed::Error(
                    "E0015",
                    &["cannot call non-const function `star_table` in constants"],
                ),
            ),
            (
                "const_eval hint: `const` added, the `for` loop is a second E0015",
                a10::CLAIM_CONST_FOR.to_string(),
                Claimed::Error(
                    "E0015",
                    &["cannot use `for` loop on `std::ops::Range<usize>` in constant functions"],
                ),
            ),
            (
                "const_eval: a false compile-time `assert!` fails the build, E0080",
                a10::CLAIM_CONST_ASSERT_FAILS.to_string(),
                Claimed::Error(
                    "E0080",
                    &[
                        "evaluation panicked: ",
                        "evaluation panicked: the last star is misplaced",
                    ],
                ),
            ),
            (
                "const_eval syntax: `square`, `NINE` and the assert compile",
                a10::CLAIM_CONST_SYNTAX.to_string(),
                Claimed::Prints("9"),
            ),
        ],
        "chunks_exact" => vec![
            (
                "chunks_exact hint: the starter's choir sings 36",
                s("chunks_exact"),
                Claimed::Prints("choir sum: 36\nplain sum: 55\nempty choir: 0"),
            ),
            (
                "chunks_exact hint: `chunks(4)` is also correct",
                a10::ALT_CHUNKS_EXACT_0.to_string(),
                Claimed::Prints("choir sum: 55\nplain sum: 55\nempty choir: 0"),
            ),
            (
                "chunks_exact: `std::simd` is unstable, E0658 on stable",
                a10::CLAIM_SIMD_UNSTABLE.to_string(),
                Claimed::Error(
                    "E0658",
                    &["use of unstable library feature `portable_simd`"],
                ),
            ),
        ],
        "no_std" => vec![
            (
                "no_std starter: E0425 for `String` and a code-less error for \
                 `format!`; rustc suggests `crate::host::format`",
                s("no_std"),
                Claimed::Error(
                    "E0425",
                    &[
                        "cannot find type `String` in this scope",
                        "cannot find macro `format` in this scope",
                        "use crate::host::format;",
                    ],
                ),
            ),
            (
                "no_std hint: rustc's suggestion compiles",
                edited_starter(
                    "no_std",
                    &[(
                        "mod firmware {\n",
                        "mod firmware {\n    use crate::host::format;\n    use crate::host::string::String;\n",
                    )],
                ),
                Claimed::Compiles,
            ),
            (
                "no_std: no `println!` under `#![no_std]`",
                a10::CLAIM_NO_STD_PRINTLN.to_string(),
                Claimed::ErrorNoCode(&["cannot find macro `println` in this scope"]),
            ),
            (
                "no_std: `extern crate alloc;` brings `String` and `Vec` back",
                a10::CLAIM_NO_STD_ALLOC.to_string(),
                Claimed::Prints("towers: one"),
            ),
        ],
        "semver_caret" => vec![(
            "semver_caret hint: the starter lets 0.3.0 (and 0.0.4) through",
            s("semver_caret"),
            Claimed::Prints(
                "^1.2.3 accepts 1.9.0: true\n^1.2.3 accepts 2.0.0: false\n\
                 ^1.2.3 accepts 1.2.2: false\n^0.2.3 accepts 0.2.9: true\n\
                 ^0.2.3 accepts 0.3.0: true\n^0.0.3 accepts 0.0.3: true\n\
                 ^0.0.3 accepts 0.0.4: true\n^1.2.3 vs 1.x.0: not a version",
            ),
        )],
        other => panic!("no claims table for Act 10 mission `{other}`: add one"),
    }
}

#[test]
fn every_error_an_act10_villager_names_is_a_compiled_claim() {
    let reg = MissionRegistry::default();
    let act10: Vec<&str> = reg
        .missions
        .iter()
        .map(|m| m.id)
        .skip_while(|id| *id != "prealloc")
        .take_while(|id| *id != "compiler_alloc")
        .collect();
    assert_eq!(
        act10,
        ACT10_VILLAGER_CODES
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>(),
        "every Act 10 villager mission has a claims table"
    );
    for (id, want) in ACT10_VILLAGER_CODES {
        assert_claims_cover_texts(id, &act10_villager_claims(id), &mission_texts(&[id]), want);
    }
}

#[test]
#[ignore = "slow: runs rustc once per claim"]
fn act10_villager_claims_are_what_rustc_emits() {
    let dir = scratch_dir("act10-claims");
    print_rustc_version();
    let mut problems = Vec::new();
    let mut checked = 0usize;
    for (id, _) in ACT10_VILLAGER_CODES {
        for (n, (label, src, claim)) in act10_villager_claims(id).into_iter().enumerate() {
            checked += 1;
            let tag = format!("{id}_{n}");
            check_claim(&dir, &tag, label, &src, &claim, &mut problems);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!("{checked} claims checked");
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

/// The epilogue is plain and true for the finished game: it no longer
/// promises more acts, names the company and no person, and the last
/// trial stage is the last mission (clearing it shows the epilogue).
#[test]
fn the_epilogue_closes_the_whole_game() {
    use pledge_and_crown::plugins::mission::EPILOGUE_LINES;
    let all: Vec<&str> = [
        EPILOGUE_LINES.heading,
        EPILOGUE_LINES.body,
        EPILOGUE_LINES.after,
    ]
    .into_iter()
    .chain(EPILOGUE_LINES.credits.iter().copied())
    .collect();
    let text = all.join("\n");
    for stale in ["being forged", "more acts", "coming soon", "@"] {
        assert!(
            !text.to_lowercase().contains(stale),
            "epilogue says {stale:?}"
        );
    }
    assert!(text.contains("Chico Cell Repair LLC"));
    assert!(text.contains("the Compiler"), "{text}");
    assert_eq!(
        TRIALS.last().map(|t| t.stages[t.stages.len() - 1]),
        MissionRegistry::default().missions.last().map(|m| m.id),
        "the last trial stage is the last mission"
    );
}
