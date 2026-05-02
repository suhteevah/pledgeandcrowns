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
