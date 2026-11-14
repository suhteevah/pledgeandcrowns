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

use pledge_and_crown::plugins::mission::MissionRegistry;
use pledge_and_crown::plugins::stub_grader::stub_verdict;
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
            r#"enum Direction { North, South }
fn name(d: Direction) -> &'static str {
    match d {
        Direction::North => "n",
        Direction::South => "s",
    }
}
fn main() { let _ = name(Direction::North); }"#
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

/// The two crates cannot share code (MIT vs AGPL), so the rule tables are
/// duplicated. Keep them textually identical so a one-sided edit can't
/// hide behind inputs the snapshot happens not to cover.
#[test]
fn server_and_stub_rule_tables_are_textually_identical() {
    fn table_body(path: &str) -> String {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let text = text.replace("\r\n", "\n");
        let start = text
            .find("// BEGIN MISSION TABLE")
            .unwrap_or_else(|| panic!("{path}: no BEGIN MISSION TABLE marker"));
        let body_start = start + text[start..].find('\n').expect("marker line ends") + 1;
        let end = text
            .find("// END MISSION TABLE")
            .unwrap_or_else(|| panic!("{path}: no END MISSION TABLE marker"));
        text[body_start..end].to_string()
    }
    let stub = table_body(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/plugins/stub_grader.rs"
    ));
    let server = table_body(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../compile-api/src/grader.rs"
    ));
    for (n, (a, b)) in stub.lines().zip(server.lines()).enumerate() {
        assert_eq!(a, b, "rule tables diverge at table line {}", n + 1);
    }
    assert_eq!(
        stub.lines().count(),
        server.lines().count(),
        "rule tables differ in length"
    );
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
