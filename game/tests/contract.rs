// SPDX-License-Identifier: MIT
//! Audit: cross-crate contract between the game's MissionRegistry and
//! the compile-API's grader. For every mission the game ships, this
//! test asserts:
//!
//!   1. A canonical solution exists in the test fixtures (so we can
//!      grade-test it).
//!   2. The grader passes that canonical solution.
//!   3. The grader does NOT pass the unmodified starter_code (which
//!      would mean the mission is trivially won by clicking Compile).
//!   4. The grader does NOT fall through to the freeform-encounter
//!      stub for that mission's id (catches "registered a mission but
//!      forgot to add a grader case").

mod common;

use common::PLAYER_EDITION;
use pledge_and_crown::plugins::mission::MissionRegistry;
use pledge_and_crown::plugins::stub_grader::{MAIN_HINT, has_entry_point, lex, stub_verdict};
use pledge_compile_api::cargo_grader;
use pledge_compile_api::grader::grade;

/// Canonical correct solutions. MUST cover every mission id in the
/// registry — `every_mission_has_a_canonical_solution` enforces that.
fn canonical_solution(encounter_id: &str) -> Option<&'static str> {
    Some(match encounter_id {
        "intro_let_binding" => "fn main() { let answer = 42; }",
        "double_function" => "fn double(n: i32) -> i32 { n * 2 }\nfn main() {}\n",
        "borrow_preview" => {
            r#"fn main() {
    let value = String::from("ok");
    let r = &value;
    println!("{r}");
}"#
        }
        "mut_binding" => "fn main() { let mut x = 0; x += 1; }",
        "if_else_sign" => {
            r#"fn sign(n: i32) -> &'static str {
    if n < 0 { "neg" } else if n == 0 { "zero" } else { "pos" }
}
fn main() {
    println!("{}", sign(-3));
}"#
        }
        "loop_break" => {
            r#"fn main() {
    let mut n = 1;
    let _ = loop {
        if n >= 100 { break n; }
        n *= 2;
    };
}"#
        }
        "match_option" => {
            r#"fn unwrap_or_zero(x: Option<i32>) -> i32 {
    match x {
        Some(n) => n,
        None => 0,
    }
}
fn main() {
    println!("{}", unwrap_or_zero(Some(7)));
}"#
        }
        "struct_basic" => {
            r#"struct Knight { name: String, hp: i32 }
fn main() {
    let k = Knight { name: String::from("Garin"), hp: 30 };
    let _ = k.name;
}"#
        }
        "vec_iter" => {
            r#"fn main() {
    let v = vec![1, 2, 3];
    let s: i32 = v.iter().sum();
    println!("{s}");
}"#
        }
        "tuple_destructure" => {
            r#"fn main() {
    let (a, b) = (3, 4);
    println!("{a} {b}");
}"#
        }
        "while_loop" => {
            r#"fn main() {
    let mut n = 5;
    while n > 0 { n -= 1; }
    println!("{n}");
}"#
        }
        "bridge_toll" => {
            r#"fn weigh(cargo: &str) -> usize { cargo.len() }
fn main() {
    let cargo = String::from("lantern oil");
    let toll = weigh(&cargo);
    println!("toll: {toll}");
    println!("cargo: {cargo}");
}"#
        }
        "bridge_quill" => {
            r#"fn inscribe(scroll: &mut Vec<String>, entry: &str) { scroll.push(entry.to_string()); }
fn main() {
    let mut scroll = vec![String::from("north road")];
    println!("reader one: {}", scroll[0]);
    println!("reader two: {}", scroll[0]);
    inscribe(&mut scroll, "east road");
    println!("scroll: {}", scroll.join(", "));
}"#
        }
        "bridge_lantern" => {
            r#"fn before<'a>(line: &'a str, sep: &str) -> &'a str {
    match line.find(sep) { Some(i) => &line[..i], None => line }
}
fn main() {
    let lantern = String::from("oil:wick:glass");
    let part;
    { let sep = String::from(":"); part = before(&lantern, &sep); }
    println!("first: {part}");
    println!("whole: {}", before("glass", ":"));
}"#
        }
        "borrow_mut" => {
            r#"fn bump(x: &mut i32) { *x = 99; }
fn main() { let mut n = 0; bump(&mut n); println!("{n}"); }"#
        }
        "string_vs_str" => {
            r#"fn greet(name: &str) { println!("hi {name}"); }
fn main() { let s = String::from("you"); greet(&s); greet("anon"); }"#
        }
        "option_unwrap_or" => {
            r#"fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }
fn main() { println!("{}", safe(Some(7))); }"#
        }
        "for_in_range" => {
            r#"fn main() {
    for i in 0..10 {
        println!("{i}");
    }
}"#
        }
        "closure_basic" => {
            r#"fn main() {
    let add = |a, b| a + b;
    println!("{}", add(2, 3));
}"#
        }
        "slice_basic" => {
            r#"fn sum_slice(xs: &[i32]) -> i32 { xs.iter().sum() }
fn main() { let _ = sum_slice(&[1, 2, 3]); }"#
        }
        "result_question_mark" => {
            r#"fn parse_int(s: &str) -> Result<i32, String> {
    let n = s.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(n)
}
fn main() { let _ = parse_int("42"); }"#
        }
        "derive_debug" => {
            r#"#[derive(Debug)]
struct Item { name: String }
fn main() { let item = Item { name: String::from("ring") }; println!("{item:?}"); }"#
        }
        "iter_map_collect" => {
            r#"fn main() {
    let v = vec![1, 2, 3];
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    let _ = doubled;
}"#
        }
        "enum_match" => {
            r#"enum Direction { North, East, South, West }
fn degrees(d: Direction) -> i32 {
    match d {
        Direction::North => 0,
        Direction::East => 90,
        Direction::South => 180,
        Direction::West => 270,
    }
}
fn main() {
    for d in [Direction::North, Direction::East, Direction::South, Direction::West] {
        println!("{}", degrees(d));
    }
}"#
        }
        // ── Act 3: Guildhall Quarter.
        "impl_method" => {
            r#"struct Hero { level: i32 }
impl Hero {
    fn power(&self) -> i32 { self.level * 10 }
}
fn main() {
    let h = Hero { level: 3 };
    println!("{}", h.power());
}"#
        }
        "assoc_new" => {
            r#"struct Hero { level: i32 }
impl Hero {
    fn new(level: i32) -> Self { Self { level } }
}
fn main() {
    let h = Hero::new(5);
    println!("{}", h.level);
}"#
        }
        "if_let" => {
            r#"fn main() {
    let maybe: Option<i32> = Some(7);
    if let Some(n) = maybe {
        println!("{n}");
    }
}"#
        }
        "while_let" => {
            r#"fn main() {
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("{top}");
    }
}"#
        }
        "tuple_struct" => {
            r#"struct Meters(f64);
fn main() {
    let d = Meters(3.5);
    println!("{}", d.0);
}"#
        }
        "enum_data_match" => {
            r#"enum Item {
    Weapon { damage: i32 },
    Potion { heal: i32 },
}
fn value(item: Item) -> i32 {
    match item {
        Item::Weapon { damage } => damage,
        Item::Potion { heal } => heal,
    }
}
fn main() {
    let _ = value(Item::Weapon { damage: 10 });
}"#
        }
        // ── Act 4: Trait Mage's Tower.
        "trait_def" => {
            r#"trait Element {
    fn name(&self) -> &str;
}
struct Fire;
impl Element for Fire {
    fn name(&self) -> &str { "fire" }
}
fn main() {
    let f = Fire;
    println!("{}", f.name());
}"#
        }
        "generic_fn" => {
            r#"fn larger<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
fn main() {
    println!("{}", larger(3, 7));
}"#
        }
        "generic_struct" => {
            r#"struct Pair<T> {
    a: T,
    b: T,
}
fn main() {
    let p = Pair { a: 1, b: 2 };
    let _ = (p.a, p.b);
}"#
        }
        "dyn_trait" => {
            r#"trait Element {
    fn name(&self) -> &str;
}
struct Fire;
impl Element for Fire {
    fn name(&self) -> &str { "fire" }
}
struct Water;
impl Element for Water {
    fn name(&self) -> &str { "water" }
}
fn main() {
    let zoo: Vec<Box<dyn Element>> = vec![Box::new(Fire), Box::new(Water)];
    for e in &zoo {
        println!("{}", e.name());
    }
}"#
        }
        "lifetimes" => {
            r#"fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
fn main() {
    let _ = longest("aa", "b");
}"#
        }
        "assoc_type" => {
            r#"trait Producer {
    type Output;
    fn make(&self) -> Self::Output;
}
struct Coiner;
impl Producer for Coiner {
    type Output = i32;
    fn make(&self) -> Self::Output { 7 }
}
fn main() {
    let c = Coiner;
    let _ = c.make();
}"#
        }
        // ── Act 5: Tavern of Tribulations.
        "result_match" => {
            r#"fn describe(r: Result<i32, String>) -> i32 {
    match r {
        Ok(v) => v,
        Err(_e) => -1,
    }
}
fn main() {
    let _ = describe(Ok(5));
}"#
        }
        "custom_error" => {
            r#"enum BrewError {
    TooHot,
    TooCold,
}
fn check(temp: i32) -> Result<i32, BrewError> {
    if temp > 100 {
        Err(BrewError::TooHot)
    } else if temp < 0 {
        Err(BrewError::TooCold)
    } else {
        Ok(temp)
    }
}
fn main() {
    let _ = check(50);
}"#
        }
        "from_error" => {
            r#"struct ParseFail;
enum AppError {
    Parse,
}
impl From<ParseFail> for AppError {
    fn from(_e: ParseFail) -> Self {
        AppError::Parse
    }
}
fn main() {
    let _e: AppError = ParseFail.into();
}"#
        }
        "option_map" => {
            r#"fn add_one(o: Option<i32>) -> Option<i32> {
    o.map(|x| x + 1)
}
fn main() {
    let _ = add_one(Some(3));
}"#
        }
        "and_then" => {
            r#"fn half(n: i32) -> Option<i32> {
    if n % 2 == 0 { Some(n / 2) } else { None }
}
fn main() {
    let _ = Some(8).and_then(half).and_then(half);
}"#
        }
        "unwrap_or_else" => {
            r#"fn value(o: Option<i32>) -> i32 {
    o.unwrap_or_else(|| 0)
}
fn main() {
    let _ = value(None);
}"#
        }
        // ── Act 6: Iterator Forge.
        "hashmap_basic" => {
            r#"use std::collections::HashMap;
fn main() {
    let mut scores = HashMap::new();
    scores.insert("ace", 1);
    let _ = scores.get("ace");
}"#
        }
        "iter_filter" => {
            r#"fn main() {
    let v = vec![1, 2, 3, 4];
    let evens: Vec<&i32> = v.iter().filter(|x| **x % 2 == 0).collect();
    let _ = evens;
}"#
        }
        "iter_fold" => {
            r#"fn main() {
    let v = vec![1, 2, 3];
    let total = v.iter().fold(0, |acc, x| acc + x);
    let _ = total;
}"#
        }
        "iter_enumerate" => {
            r#"fn main() {
    let v = vec![10, 20, 30];
    for (i, x) in v.iter().enumerate() {
        let _ = (i, x);
    }
}"#
        }
        "iter_zip" => {
            r#"fn main() {
    let a = vec![1, 2];
    let b = vec![3, 4];
    for (x, y) in a.iter().zip(b.iter()) {
        let _ = (x, y);
    }
}"#
        }
        "closure_move" => {
            r#"fn main() {
    let name = String::from("Garin");
    let greet = move || println!("hail, {name}");
    greet();
}"#
        }
        // ── Act 7: Concurrent Coast.
        "thread_spawn" => {
            r#"use std::thread;
fn main() {
    let handle = thread::spawn(|| 21 * 2);
    let _ = handle.join();
}"#
        }
        "arc_mutex" => {
            r#"use std::sync::{Arc, Mutex};
fn main() {
    let shared = Arc::new(Mutex::new(0));
    {
        let mut guard = shared.lock().unwrap();
        *guard += 1;
    }
    let _ = shared;
}"#
        }
        "mpsc_channel" => {
            r#"use std::sync::mpsc;
fn main() {
    let (tx, rx) = mpsc::channel();
    tx.send(7).unwrap();
    let _ = rx.recv();
}"#
        }
        "atomic" => {
            r#"use std::sync::atomic::{AtomicUsize, Ordering};
fn main() {
    let counter = AtomicUsize::new(0);
    counter.fetch_add(1, Ordering::SeqCst);
    let _ = counter;
}"#
        }
        "thread_scope" => {
            r#"use std::thread;
fn main() {
    let data = vec![1, 2, 3];
    thread::scope(|s| {
        s.spawn(|| {
            let _ = &data;
        });
    });
}"#
        }
        "async_fn" => {
            r#"async fn double(x: i32) -> i32 {
    x * 2
}
async fn run() -> i32 {
    double(21).await
}
fn main() {
    let _future = run();
}"#
        }
        // ── Act 8: Vault of Pointers.
        "box_basic" => {
            r#"struct Node {
    value: i32,
    next: Option<Box<Node>>,
}
fn main() {
    let list = Node {
        value: 1,
        next: Some(Box::new(Node { value: 2, next: None })),
    };
    let _ = list;
}"#
        }
        "rc_basic" => {
            r#"use std::rc::Rc;
fn main() {
    let original = Rc::new(String::from("hoard"));
    let shared = Rc::clone(&original);
    let _ = (original, shared);
}"#
        }
        "refcell" => {
            r#"use std::cell::RefCell;
fn main() {
    let guarded = RefCell::new(0);
    *guarded.borrow_mut() += 1;
    let _ = guarded.borrow();
}"#
        }
        "cell" => {
            r#"use std::cell::Cell;
fn main() {
    let slot = Cell::new(1);
    slot.set(5);
    let _ = slot.get();
}"#
        }
        "rc_refcell" => {
            r#"use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let shared = Rc::new(RefCell::new(0));
    let clone = Rc::clone(&shared);
    *clone.borrow_mut() += 1;
    let _ = shared.borrow();
}"#
        }
        "weak_ref" => {
            r#"use std::rc::{Rc, Weak};
fn main() {
    let strong = Rc::new(5);
    let weak: Weak<i32> = Rc::downgrade(&strong);
    let _ = weak.upgrade();
}"#
        }
        _ => return None,
    })
}

#[test]
fn every_mission_has_a_canonical_solution() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        assert!(
            canonical_solution(m.id).is_some(),
            "mission `{}` has no canonical_solution() entry — add one before the contract suite can grade it",
            m.id
        );
    }
}

#[test]
fn canonical_solutions_pass_the_grader() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src = canonical_solution(m.id).expect("checked by previous test");
        let v = grade(m.id, src);
        assert!(
            v.ok,
            "canonical solution for `{}` was rejected: {v:?}",
            m.id
        );
    }
}

#[test]
fn unmodified_starter_code_does_not_trivially_win() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let v = grade(m.id, m.starter_code);
        assert!(
            !v.ok,
            "mission `{}` is trivially solved by submitting the starter_code unchanged: {v:?}",
            m.id
        );
    }
}

#[test]
fn server_and_stub_flavor_agree_byte_for_byte() {
    // Stub_grader duplicates the server's flavor strings on purpose
    // (game crate must not depend on pledge_compile_api at runtime).
    // The previous integration miss (190fdda) was caused by *wording*
    // drift, not missing arms — the existing coverage test only
    // catches the latter. This test catches both.
    //
    // Strip the `[stub] ` prefix the stub adds at the boundary, then
    // compare to the server's stdout/stderr verbatim.
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src = canonical_solution(m.id).expect("checked elsewhere");
        let server = grade(m.id, src);
        let stub = stub_verdict(m.id, src)
            .unwrap_or_else(|| panic!("stub_verdict missing arm for `{}`", m.id));
        assert_eq!(
            server.ok, stub.ok,
            "ok flag drift on `{}`: server={} stub={}",
            m.id, server.ok, stub.ok
        );
        // Stub messages carry the `[stub] ` marker; strip it before
        // comparing to the server's verbatim flavor text.
        let stub_stdout = stub.stdout.strip_prefix("[stub] ").unwrap_or(&stub.stdout);
        let stub_stderr = stub.stderr.strip_prefix("[stub] ").unwrap_or(&stub.stderr);
        assert_eq!(
            server.stdout, stub_stdout,
            "stdout drift on canonical `{}`",
            m.id
        );
        assert_eq!(
            server.stderr, stub_stderr,
            "stderr drift on canonical `{}`",
            m.id
        );
    }
}

/// Slow audit: every canonical solution must also pass `cargo check`.
///
/// The pattern grader checks token presence, not language correctness —
/// in principle it could approve syntactically-broken Rust as long as
/// the right keywords appear. This test closes that loop by running
/// every canonical solution through the real `cargo_grader::compile_check`
/// and asserting compilation succeeds.
///
/// `#[ignore]` because each invocation forks a real `cargo check` (~1-3s
/// warm × 21 missions ≈ 30-60s total). Runs alongside the other slow
/// audits via:
///
/// ```text
/// cargo test --workspace -- --ignored
/// ```
#[test]
#[ignore = "slow: forks cargo check 21 times"]
fn every_canonical_solution_passes_cargo_check() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src =
            canonical_solution(m.id).expect("checked by every_mission_has_a_canonical_solution");
        let verdict = cargo_grader::compile_check(src)
            .unwrap_or_else(|e| panic!("cargo_grader setup failed for `{}`: {e:#}", m.id));
        assert!(
            verdict.ok,
            "canonical solution for `{}` is not valid Rust per cargo check.\n--- source ---\n{src}\n--- stderr ---\n{}",
            m.id, verdict.stderr
        );
    }
}

// ── Grader characterisation snapshots ───────────────────────────────
//
// Both graders were refactored from hand-written per-mission `if` chains
// into a rule table. These snapshots pin the exact observable output
// (ok flag + stdout + stderr) for every mission against:
//
//   * the canonical solution above,
//   * the mission's starter code,
//   * an empty string,
//   * near-misses: the canonical solution with one whitespace-delimited
//     token removed, one near-miss per distinct server failure message.
//
// Inputs are stored in the snapshot files themselves, so the replay
// tests (`stub_grader_matches_snapshot` here, and
// `compile-api/tests/grader_snapshot.rs`) are pure replays. To
// regenerate (only when grader output is meant to change):
//
//   PNC_UPDATE_GRADER_SNAPSHOTS=1 cargo test -p pledge_and_crown --test contract

const STUB_SNAPSHOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/snapshots/stub_grader.json"
);
const SERVER_SNAPSHOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../compile-api/tests/snapshots/grader.json"
);

/// Each near-miss is the canonical source with exactly one
/// whitespace-delimited token cut out.
fn single_token_removals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let bytes: Vec<(usize, char)> = src.char_indices().collect();
    for (i, c) in bytes
        .iter()
        .copied()
        .chain(std::iter::once((src.len(), ' ')))
    {
        match (c.is_whitespace(), start) {
            (false, None) => start = Some(i),
            (true, Some(s)) => {
                out.push(format!("{}{}", &src[..s], &src[i..]));
                start = None;
            }
            _ => {}
        }
    }
    out
}

/// (encounter_id, case label, source) for every snapshot case.
fn snapshot_cases() -> Vec<(String, String, String)> {
    let reg = MissionRegistry::default();
    let mut cases = Vec::new();
    for m in &reg.missions {
        let canonical = canonical_solution(m.id).expect("checked elsewhere");
        cases.push((m.id.to_string(), "canonical".into(), canonical.to_string()));
        cases.push((
            m.id.to_string(),
            "starter".into(),
            m.starter_code.to_string(),
        ));
        cases.push((m.id.to_string(), "empty".into(), String::new()));
        let mut seen = std::collections::BTreeSet::new();
        for (n, src) in single_token_removals(canonical).into_iter().enumerate() {
            let v = grade(m.id, &src);
            if !v.ok && seen.insert(v.stderr.clone()) {
                cases.push((m.id.to_string(), format!("near_miss_{n}"), src));
            }
        }
    }
    // Unknown id: server falls back to freeform, stub returns None.
    for (label, src) in [("empty", ""), ("garbage", "fn main() { /* ? */ }")] {
        cases.push(("not_a_real_mission".into(), label.into(), src.into()));
    }
    cases
}

fn load_snapshot(path: &str) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read snapshot {path}: {e} (regenerate, see comment)"));
    let v: serde_json::Value = serde_json::from_str(&text).expect("snapshot is valid JSON");
    v.as_array().expect("snapshot is a JSON array").clone()
}

#[test]
fn stub_grader_matches_snapshot() {
    if std::env::var_os("PNC_UPDATE_GRADER_SNAPSHOTS").is_some() {
        let mut stub_rows = Vec::new();
        let mut server_rows = Vec::new();
        for (id, case, src) in snapshot_cases() {
            let stub = stub_verdict(&id, &src)
                .map(|s| serde_json::json!({ "ok": s.ok, "stdout": s.stdout, "stderr": s.stderr }));
            stub_rows.push(serde_json::json!({
                "id": id, "case": case, "input": src, "verdict": stub,
            }));
            let g = grade(&id, &src);
            server_rows.push(serde_json::json!({
                "id": id, "case": case, "input": src,
                "verdict": { "ok": g.ok, "stdout": g.stdout, "stderr": g.stderr },
            }));
        }
        for (path, rows) in [(STUB_SNAPSHOT, stub_rows), (SERVER_SNAPSHOT, server_rows)] {
            let dir = std::path::Path::new(path).parent().expect("has parent");
            std::fs::create_dir_all(dir).expect("create snapshot dir");
            let body = serde_json::to_string_pretty(&rows).expect("serialize") + "\n";
            std::fs::write(path, body).expect("write snapshot");
        }
    }

    let rows = load_snapshot(STUB_SNAPSHOT);
    assert!(rows.len() > 4 * 57, "snapshot suspiciously small");
    for row in &rows {
        let id = row["id"].as_str().expect("id");
        let case = row["case"].as_str().expect("case");
        let input = row["input"].as_str().expect("input");
        let actual = stub_verdict(id, input)
            .map(|s| serde_json::json!({ "ok": s.ok, "stdout": s.stdout, "stderr": s.stderr }));
        let actual = actual.unwrap_or(serde_json::Value::Null);
        assert_eq!(
            actual, row["verdict"],
            "stub grader drift on `{id}` case `{case}`"
        );
    }
}

/// The two crates cannot share code (MIT vs AGPL), so the matching engine
/// and the rule table are duplicated. Keep both textually identical so a
/// one-sided edit can't hide behind inputs the snapshot happens not to
/// cover.
#[test]
fn server_and_stub_rule_tables_are_textually_identical() {
    fn region(path: &str, name: &str) -> String {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let text = text.replace("\r\n", "\n");
        let begin = format!("// BEGIN {name}");
        let end_marker = format!("// END {name}");
        let start = text
            .find(&begin)
            .unwrap_or_else(|| panic!("{path}: no `{begin}` marker"));
        let body_start = start + text[start..].find('\n').expect("marker line ends") + 1;
        let end = text
            .find(&end_marker)
            .unwrap_or_else(|| panic!("{path}: no `{end_marker}` marker"));
        assert!(body_start < end, "{path}: `{name}` markers out of order");
        text[body_start..end].to_string()
    }
    let stub_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/plugins/stub_grader.rs");
    let server_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../compile-api/src/grader.rs");
    for name in ["MISSION TABLE", "GRADER ENGINE"] {
        let stub = region(stub_path, name);
        let server = region(server_path, name);
        for (n, (a, b)) in stub.lines().zip(server.lines()).enumerate() {
            assert_eq!(a, b, "`{name}` regions diverge at region line {}", n + 1);
        }
        assert_eq!(
            stub.lines().count(),
            server.lines().count(),
            "`{name}` regions differ in length"
        );
    }
}

// ── Evidence corpus ─────────────────────────────────────────────────
//
// The pattern grader is the only grader in the browser build, so "it
// passes the canonical solution" is not enough. For every mission this
// corpus checks, against BOTH graders (which must agree):
//
//   (a) the player-facing solution (editor/solutions/data.rs) passes;
//   (b) the canonical fixture above passes;
//   (c) the player solution hidden in a block comment, in line comments
//       and in a raw string, next to an empty `fn main`, FAILS;
//   (d) correct programs the old substring grader rejected (the audit's
//       examples, plus differently-written solutions for every mission
//       whose checks were tightened) pass;
//   (e) wrong programs the old grader accepted, and that a token check
//       can catch, fail;
//   (f) the unmodified starter code fails.
//
// Run with `--nocapture` to see the counts.

use pledge_and_crown::plugins::editor::solutions::canonical_solution as player_solution;

/// Grade with both graders, insist they agree, return the verdict.
fn graded_ok(id: &str, src: &str) -> bool {
    let server = grade(id, src);
    let stub = stub_verdict(id, src).unwrap_or_else(|| panic!("no stub rule for `{id}`"));
    assert_eq!(
        server.ok, stub.ok,
        "graders disagree on `{id}`:\n{src}\nserver={server:?}\nstub={stub:?}"
    );
    server.ok
}

/// Correct programs, each a complete program that builds with `rustc` at
/// the server's edition (`PLAYER_EDITION`). `(mission, why it is here, source)`.
const CORRECT: &[(&str, &str, &str)] = &[
    // ── audit "correct but rejected" (G1, G3-G7, G10, G11) ──
    (
        "double_function",
        "G1 2 * n",
        "fn double(n: i32) -> i32 {\n    2 * n\n}\nfn main() {\n    println!(\"{}\", double(21));\n}\n",
    ),
    (
        "double_function",
        "G1 n + n",
        "fn double(n: i32) -> i32 {\n    n + n\n}\nfn main() {\n    println!(\"{}\", double(21));\n}\n",
    ),
    (
        "borrow_mut",
        "G3 param named n",
        "fn bump(n: &mut i32) {\n    *n += 1;\n}\nfn main() {\n    let mut v = 0;\n    bump(&mut v);\n    println!(\"{v}\");\n}\n",
    ),
    (
        "closure_basic",
        "G3 |x, y|",
        "fn main() {\n    let add = |x, y| x + y;\n    println!(\"{}\", add(2, 3));\n}\n",
    ),
    (
        "iter_map_collect",
        "G3 |n|",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let d: Vec<i32> = v.iter().map(|n| n * 2).collect();\n    println!(\"{d:?}\");\n}\n",
    ),
    (
        "string_vs_str",
        "G4 to_string",
        "fn greet(name: &str) {\n    println!(\"hi {name}\");\n}\nfn main() {\n    let s = \"you\".to_string();\n    greet(&s);\n    greet(\"anon\");\n}\n",
    ),
    (
        "result_question_mark",
        "G5 Ok(expr?)",
        "fn parse_int(s: &str) -> Result<i32, String> {\n    Ok(s.parse::<i32>().map_err(|e| e.to_string())?)\n}\nfn main() {\n    println!(\"{:?}\", parse_int(\"42\"));\n}\n",
    ),
    (
        "derive_debug",
        "G6 derive(Debug, Clone) + {:#?}",
        "#[derive(Debug, Clone)]\nstruct Item {\n    name: String,\n}\nfn main() {\n    let item = Item { name: String::from(\"ring\") };\n    println!(\"{item:#?}\");\n    println!(\"{}\", item.name);\n}\n",
    ),
    (
        "derive_debug",
        "G6 derive(Clone, Debug) + {:?}",
        "#[derive(Clone, Debug)]\nstruct Item {\n    name: String,\n}\nfn main() {\n    let item = Item { name: String::from(\"ring\") };\n    println!(\"{:?}\", item);\n    println!(\"{}\", item.name);\n}\n",
    ),
    (
        "assoc_new",
        "G7 -> Hero",
        "struct Hero {\n    level: i32,\n}\nimpl Hero {\n    fn new(level: i32) -> Hero {\n        Hero { level }\n    }\n}\nfn main() {\n    let h = Hero::new(5);\n    println!(\"{}\", h.level);\n}\n",
    ),
    (
        "generic_struct",
        "G10 tuple Pair<T>(T, T)",
        "struct Pair<T>(T, T);\nfn main() {\n    let p = Pair(1, 2);\n    println!(\"{} {}\", p.0, p.1);\n}\n",
    ),
    (
        "generic_struct",
        "G10 Pair<U>",
        "struct Pair<U> {\n    a: U,\n    b: U,\n}\nfn main() {\n    let p = Pair { a: 1, b: 2 };\n    println!(\"{} {}\", p.a, p.b);\n}\n",
    ),
    (
        "lifetimes",
        "G10 lifetime 's",
        "fn longest<'s>(x: &'s str, y: &'s str) -> &'s str {\n    if x.len() > y.len() { x } else { y }\n}\nfn main() {\n    println!(\"{}\", longest(\"aa\", \"b\"));\n}\n",
    ),
    (
        "iter_zip",
        "G11 std::iter::zip",
        "fn main() {\n    let a = vec![1, 2];\n    let b = vec![3, 4];\n    for (x, y) in std::iter::zip(&a, &b) {\n        println!(\"{x},{y}\");\n    }\n}\n",
    ),
    (
        "closure_move",
        "audit move|| without space",
        "fn main() {\n    let name = String::from(\"Garin\");\n    let greet = move||println!(\"hail, {name}\");\n    greet();\n}\n",
    ),
    // ── differently-written solutions for every tightened mission ──
    (
        "if_else_sign",
        "tightened: returns",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 {\n        return \"neg\";\n    } else if n == 0 {\n        return \"zero\";\n    } else {\n        return \"pos\";\n    }\n}\nfn main() {\n    println!(\"{}\", sign(-3));\n}\n",
    ),
    (
        "if_else_sign",
        "tightened: > 0 branch, let-bound",
        "fn sign(n: i32) -> &'static str {\n    let label = if n < 0 { \"neg\" } else if n > 0 { \"pos\" } else { \"zero\" };\n    label\n}\nfn main() {\n    println!(\"{}\", sign(4));\n}\n",
    ),
    (
        "if_else_sign",
        "tightened: zero first, no spaces",
        "fn sign(n: i32) -> &'static str {\n    if n==0 {\"zero\"} else if n<0 {\"neg\"} else {\"pos\"}\n}\nfn main() {\n    println!(\"{}\", sign(0));\n}\n",
    ),
    (
        "loop_break",
        "tightened: double before test",
        "fn main() {\n    let mut n = 1;\n    let r = loop {\n        n *= 2;\n        if n >= 100 {\n            break n;\n        }\n    };\n    println!(\"{r}\");\n}\n",
    ),
    (
        "loop_break",
        "tightened: x = x * 2",
        "fn main() {\n    let mut value = 1;\n    let first = loop {\n        if value >= 100 {\n            break value;\n        }\n        value = value * 2;\n    };\n    println!(\"{first}\");\n}\n",
    ),
    (
        "loop_break",
        "tightened: x = 2 * x",
        "fn main() {\n    let mut value = 1;\n    let first = loop {\n        if value >= 100 {\n            break value;\n        }\n        value = 2 * value;\n    };\n    println!(\"{first}\");\n}\n",
    ),
    (
        "match_option",
        "tightened: None arm first",
        "fn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        None => 0,\n        Some(v) => v,\n    }\n}\nfn main() {\n    println!(\"{}\", unwrap_or_zero(Some(7)));\n}\n",
    ),
    (
        "match_option",
        "tightened: block arms",
        "fn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        Some(value) => {\n            value\n        }\n        None => {\n            0\n        }\n    }\n}\nfn main() {\n    println!(\"{}\", unwrap_or_zero(None));\n}\n",
    ),
    (
        "match_option",
        "tightened: return arms",
        "fn unwrap_or_zero(maybe: Option<i32>) -> i32 {\n    match maybe {\n        Some(n) => return n,\n        None => return 0,\n    }\n}\nfn main() {\n    println!(\"{}\", unwrap_or_zero(Some(1)));\n}\n",
    ),
    (
        "tuple_struct",
        "tightened: .0 on a temporary",
        "struct Meters(f64);\nfn main() {\n    let raw: f64 = 3.5;\n    println!(\"{}\", Meters(raw).0);\n}\n",
    ),
    (
        "tuple_struct",
        "tightened: float literal and a real .0",
        "struct Meters(f64);\nfn main() {\n    let d = Meters(3.0);\n    let inner = d.0;\n    println!(\"{inner}\");\n}\n",
    ),
    (
        "trait_def",
        "tightened: trait defined after impl",
        "struct Fire;\nimpl Element for Fire {\n    fn name(&self) -> &str {\n        \"fire\"\n    }\n}\ntrait Element {\n    fn name(&self) -> &str;\n}\nfn main() {\n    println!(\"{}\", Fire.name());\n}\n",
    ),
    (
        "trait_def",
        "tightened: default method, empty impl",
        "trait Element {\n    fn name(&self) -> &str {\n        \"element\"\n    }\n}\nstruct Fire;\nimpl Element for Fire {}\nfn main() {\n    let f = Fire;\n    println!(\"{}\", f.name());\n}\n",
    ),
    (
        "trait_def",
        "tightened: String-returning trait",
        "trait Element {\n    fn name(&self) -> String;\n}\nstruct Fire;\nimpl Element for Fire {\n    fn name(&self) -> String {\n        String::from(\"fire\")\n    }\n}\nfn main() {\n    println!(\"{}\", Fire.name());\n}\n",
    ),
    (
        "iter_fold",
        "tightened: |sum, &n|",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.iter().fold(0, |sum, &n| sum + n);\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "tightened: typed params, block body",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total: i32 = v.into_iter().fold(0, |acc: i32, x: i32| {\n        let next = acc + x;\n        next\n    });\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "tightened: x + acc",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.iter().fold(0, |acc, x| x + acc);\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "named fn defined before the fold",
        "fn add(acc: i32, x: &i32) -> i32 {\n    acc + x\n}\nfn main() {\n    let v = vec![1, 2, 3];\n    println!(\"{}\", v.iter().fold(0, add));\n}\n",
    ),
    (
        "iter_fold",
        "named fn defined after the fold",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.into_iter().fold(0, add);\n    println!(\"{total}\");\n}\nfn add(acc: i32, x: i32) -> i32 {\n    acc + x\n}\n",
    ),
    (
        "iter_fold",
        "closure bound to a name first",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let add = |acc: i32, x: &i32| acc + x;\n    println!(\"{}\", v.iter().fold(0, add));\n}\n",
    ),
    (
        "intro_let_binding",
        "entry point: pub fn main",
        "pub fn main() {\n    let answer = 42;\n    println!(\"{answer}\");\n}\n",
    ),
    (
        "closure_basic",
        "tightened: typed params, b + a",
        "fn main() {\n    let add = |a: i32, b: i32| -> i32 { b + a };\n    println!(\"{}\", add(2, 3));\n}\n",
    ),
    (
        "borrow_mut",
        "tightened: *counter = *counter + 1",
        "fn bump(counter: &mut i32) {\n    *counter = *counter + 1;\n}\nfn main() {\n    let mut c = 0;\n    bump(&mut c);\n    println!(\"{c}\");\n}\n",
    ),
    (
        "generic_struct",
        "tightened: bound on T",
        "struct Pair<T: Copy> {\n    first: T,\n    second: T,\n}\nfn main() {\n    let p = Pair { first: 1, second: 2 };\n    println!(\"{} {}\", p.first, p.second);\n}\n",
    ),
    (
        "lifetimes",
        "tightened: lifetime 'long",
        "fn longest<'long>(a: &'long str, b: &'long str) -> &'long str {\n    if a.len() >= b.len() { a } else { b }\n}\nfn main() {\n    println!(\"{}\", longest(\"x\", \"yy\"));\n}\n",
    ),
    (
        "generic_fn",
        "tightened: type parameter U",
        "fn larger<U: PartialOrd>(a: U, b: U) -> U {\n    if a > b { a } else { b }\n}\nfn main() {\n    println!(\"{}\", larger(3, 7));\n}\n",
    ),
    (
        "atomic",
        "token match: AtomicU32",
        "use std::sync::atomic::{AtomicU32, Ordering};\nfn main() {\n    let c = AtomicU32::new(0);\n    c.fetch_add(1, Ordering::SeqCst);\n    println!(\"{}\", c.load(Ordering::SeqCst));\n}\n",
    ),
    // ── the Borrow Checker's trial: realistic alternatives (the same
    // programs pass the server check in `expectations_proof.rs`) ──
    (
        "bridge_toll",
        "trial alt: chars().count() and as_str()",
        "fn weigh(cargo: &str) -> usize {\n    cargo.chars().count()\n}\nfn main() {\n    let cargo = String::from(\"lantern oil\");\n    let toll = weigh(cargo.as_str());\n    println!(\"toll: {toll}\");\n    println!(\"cargo: {cargo}\");\n}\n",
    ),
    (
        "bridge_quill",
        "trial alt: readers clone, other names",
        "fn inscribe(list: &mut Vec<String>, line: &str) {\n    list.push(String::from(line));\n}\nfn main() {\n    let mut scroll = vec![String::from(\"north road\")];\n    let first = scroll[0].clone();\n    let second = scroll[0].clone();\n    inscribe(&mut scroll, \"east road\");\n    println!(\"reader one: {first}\");\n    println!(\"reader two: {second}\");\n    println!(\"scroll: {}\", scroll.join(\", \"));\n}\n",
    ),
    (
        "bridge_lantern",
        "trial alt: two named lifetimes, split",
        "fn before<'a, 'b>(line: &'a str, sep: &'b str) -> &'a str {\n    line.split(sep).next().unwrap_or(line)\n}\nfn main() {\n    let lantern = String::from(\"oil:wick:glass\");\n    let part;\n    {\n        let sep = String::from(\":\");\n        part = before(&lantern, &sep);\n    }\n    println!(\"first: {part}\");\n    println!(\"whole: {}\", before(\"glass\", \":\"));\n}\n",
    ),
];

/// Wrong programs the old substring grader ACCEPTED and a token-level
/// check now catches. `(mission, audit id, source)`.
const WRONG: &[(&str, &str, &str)] = &[
    // ── the Borrow Checker's trial: the shortcuts his tutorial names ──
    (
        "bridge_toll",
        "trial: pays the toll with a clone",
        "fn weigh(cargo: String) -> usize { cargo.len() }\nfn main() { let cargo = String::from(\"lantern oil\"); let toll = weigh(cargo.clone()); println!(\"toll: {toll}\"); println!(\"cargo: {cargo}\"); }\n",
    ),
    (
        "bridge_quill",
        "trial: inscribe takes the scroll by value",
        "fn inscribe(mut scroll: Vec<String>, entry: &str) -> Vec<String> { scroll.push(entry.to_string()); scroll }\nfn main() { let scroll = vec![String::from(\"north road\")]; let scroll = inscribe(scroll, \"east road\"); println!(\"scroll: {}\", scroll.join(\", \")); }\n",
    ),
    (
        "bridge_lantern",
        "trial: ties the answer to both inputs (rustc's suggestion; E0597 in main)",
        "fn before<'a>(line: &'a str, sep: &'a str) -> &'a str { match line.find(sep) { Some(i) => &line[..i], None => line } }\nfn main() { println!(\"{}\", before(\"oil:wick\", \":\")); }\n",
    ),
    (
        "if_else_sign",
        "G2 neg/pos labels swapped",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 { \"pos\" } else if n == 0 { \"zero\" } else { \"neg\" }\n}\nfn main() {}\n",
    ),
    (
        "if_else_sign",
        "G2 zero/pos labels swapped",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 { \"neg\" } else if n == 0 { \"pos\" } else { \"zero\" }\n}\nfn main() {}\n",
    ),
    (
        "match_option",
        "G2 0 in both arms",
        "fn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        Some(_) => 0,\n        None => 0,\n    }\n}\nfn main() {}\n",
    ),
    (
        "loop_break",
        "G2 break with no value",
        "fn main() {\n    let mut n = 1;\n    loop {\n        if n >= 100 {\n            break;\n        }\n        n *= 2;\n    }\n}\n",
    ),
    (
        "tuple_struct",
        "G8 3.0 literal, field never read",
        "struct Meters(f64);\nfn main() {\n    let _d = Meters(3.0);\n}\n",
    ),
    (
        "trait_def",
        "G9 inherent impl + unused trait + for loop",
        "trait Element {\n    fn name(&self) -> &str;\n}\nstruct Fire;\nimpl Fire {\n    fn name(&self) -> &str {\n        \"fire\"\n    }\n}\nfn main() {\n    for _ in 0..1 {\n        println!(\"{}\", Fire.name());\n    }\n}\n",
    ),
    (
        "trait_def",
        "G9 std trait implemented, own trait unused",
        "trait Element {}\nstruct Fire;\nimpl Clone for Fire {\n    fn clone(&self) -> Self {\n        Fire\n    }\n}\nfn main() {}\n",
    ),
    (
        "iter_fold",
        "fold ignoring its elements",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.iter().fold(0, |acc, _| acc);\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "named fn ignoring its element",
        "fn keep(acc: i32, _x: &i32) -> i32 {\n    acc\n}\nfn main() {\n    let v = vec![1, 2, 3];\n    println!(\"{}\", v.iter().fold(0, keep));\n}\n",
    ),
    (
        "if_else_sign",
        "G0b no fn main (rustc E0601)",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 { \"neg\" } else if n == 0 { \"zero\" } else { \"pos\" }\n}\n",
    ),
    (
        "match_option",
        "G0b main only inside a module",
        "mod game {\n    pub fn main() {}\n}\nfn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        Some(n) => n,\n        None => 0,\n    }\n}\n",
    ),
    (
        "impl_method",
        "G0b main only as a method",
        "struct Hero { level: i32 }\nimpl Hero {\n    fn power(&self) -> i32 { self.level * 10 }\n    fn main() {}\n}\n",
    ),
    (
        "intro_let_binding",
        "G0b async fn main (rustc E0752)",
        "async fn main() {\n    let answer = 42;\n}\n",
    ),
];

#[test]
fn grader_corpus() {
    let reg = MissionRegistry::default();
    let n = reg.missions.len();
    let (mut a, mut b, mut c, mut f) = (0, 0, 0, 0);
    let mut starters_with_main = 0;
    let mut failures = Vec::new();
    for m in &reg.missions {
        let player = player_solution(m.id).unwrap_or_else(|| panic!("no solution for `{}`", m.id));
        if graded_ok(m.id, player) {
            a += 1;
        } else {
            failures.push(format!("(a) player solution rejected: {}", m.id));
        }
        let fixture = canonical_solution(m.id).expect("checked elsewhere");
        if graded_ok(m.id, fixture) {
            b += 1;
        } else {
            failures.push(format!("(b) contract fixture rejected: {}", m.id));
        }
        assert!(
            !player.contains("*/") && !player.contains("\"##"),
            "wrapper would break"
        );
        let block = format!("fn main() {{}}\n/*\n{player}\n*/\n");
        let lines: String = player.lines().map(|l| format!("// {l}\n")).collect();
        let lines = format!("fn main() {{}}\n{lines}");
        let string = format!("fn main() {{\n    let _s = r##\"{player}\"##;\n}}\n");
        let hidden: Vec<bool> = [&block, &lines, &string]
            .iter()
            .map(|src| graded_ok(m.id, src))
            .collect();
        if hidden.iter().all(|ok| !ok) {
            c += 1;
        } else {
            failures.push(format!(
                "(c) hidden solution passed (block, lines, string) = {hidden:?}: {}",
                m.id
            ));
        }
        if !graded_ok(m.id, m.starter_code) {
            f += 1;
        } else {
            failures.push(format!("(f) starter passes: {}", m.id));
        }
        // A starter's first message must teach its concept, never the
        // shared `fn main` rule (which is checked last).
        let starter_msg = grade(m.id, m.starter_code).stderr;
        if starter_msg.contains(MAIN_HINT) {
            failures.push(format!(
                "(f) starter's first failure is the fn main hint, not its concept: {}",
                m.id
            ));
        }
        if has_entry_point(&lex(m.starter_code)) {
            starters_with_main += 1;
        }
    }
    let mut d = 0;
    for (id, why, src) in CORRECT {
        if graded_ok(id, src) {
            d += 1;
        } else {
            failures.push(format!("(d) correct program rejected: {id} [{why}]"));
        }
    }
    let mut e = 0;
    for (id, why, src) in WRONG {
        if !graded_ok(id, src) {
            e += 1;
        } else {
            failures.push(format!("(e) wrong program accepted: {id} [{why}]"));
        }
    }
    println!(
        "grader corpus: (a) {a}/{n} player solutions pass; (b) {b}/{n} fixtures pass; \
         (c) {c}/{n} hidden solutions fail; (d) {d}/{} correct programs pass; \
         (e) {e}/{} wrong programs fail; (f) {f}/{n} starters fail \
         ({starters_with_main} of them have `fn main`, all fail on a concept hint)",
        CORRECT.len(),
        WRONG.len()
    );
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Slow audit: every `CORRECT` corpus program really is valid Rust at the
/// server's edition.
/// Run with `cargo test -p pledge_and_crown --test contract -- --ignored`.
#[test]
#[ignore = "slow: runs rustc once per corpus program"]
fn corpus_correct_programs_compile() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("grader-corpus-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    let mut failures = Vec::new();
    // The canonical fixtures too, as binaries: a fixture with no `fn main`
    // (audit G0b) fails here with E0601.
    let reg = MissionRegistry::default();
    let fixtures = reg.missions.iter().map(|m| {
        let src = canonical_solution(m.id).expect("checked elsewhere");
        (m.id, "canonical fixture", src)
    });
    let programs: Vec<(&str, &str, &str)> = CORRECT.iter().copied().chain(fixtures).collect();
    for (n, (id, why, src)) in programs.iter().enumerate() {
        let path = dir.join(format!("c{n}.rs"));
        std::fs::write(&path, src).expect("write corpus source");
        let out = std::process::Command::new("rustc")
            .args(["--edition", PLAYER_EDITION, "--crate-type", "bin", "-o"])
            .arg(dir.join(format!("c{n}.out")))
            .arg(&path)
            .output()
            .expect("spawn rustc");
        if !out.status.success() {
            failures.push(format!(
                "{id} [{why}]: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!(
        "corpus compile ({PLAYER_EDITION}): {} of {} programs built",
        programs.len() - failures.len(),
        programs.len()
    );
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn no_mission_falls_through_to_freeform() {
    // The freeform fallthrough path always returns ok=true with a
    // stdout containing the literal "freeform". A real mission's
    // grader path must reject garbage with ok=false (and the freeform
    // marker must NOT appear in either branch).
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let v = grade(m.id, "fn main() { /* total nonsense */ }");
        let mentions_freeform = v.stdout.contains("freeform") || v.stderr.contains("freeform");
        assert!(
            !mentions_freeform,
            "mission `{}` is hitting the freeform fallthrough — add a grader arm in compile-api/src/grader.rs",
            m.id
        );
    }
}
