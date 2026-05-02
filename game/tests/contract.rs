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
