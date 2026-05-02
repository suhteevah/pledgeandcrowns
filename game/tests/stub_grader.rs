// SPDX-License-Identifier: MIT
//! Audit: the offline stub grader must accept every canonical solution
//! and reject obviously-wrong samples for every mission shipped in the
//! `MissionRegistry::default()` table. An unknown encounter id must
//! return `None` so the caller can fall back to `[client error]`.
//!
//! Canonical solutions are duplicated here intentionally — the
//! `contract.rs` cross-crate audit keeps the server-side grader honest;
//! this duplication keeps the client-side stub honest. If the two ever
//! drift, both audits will scream.

use pledge_and_crown::plugins::mission::MissionRegistry;
use pledge_and_crown::plugins::stub_grader::stub_verdict;

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
        _ => return None,
    })
}

#[test]
fn stub_covers_every_mission_in_the_registry() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        assert!(
            canonical_solution(m.id).is_some(),
            "stub_grader test fixtures missing canonical_solution for `{}`",
            m.id
        );
        let v = stub_verdict(m.id, "fn main() {}");
        assert!(
            v.is_some(),
            "stub_verdict returned None for shipped mission `{}` — add an arm",
            m.id
        );
    }
}

#[test]
fn stub_passes_every_canonical_solution() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src = canonical_solution(m.id).expect("checked above");
        let v = stub_verdict(m.id, src).expect("registry coverage checked above");
        assert!(
            v.ok,
            "stub rejected canonical solution for `{}`: {v:?}",
            m.id
        );
        // Stub markers are load-bearing for the UI banner contract.
        assert!(
            v.stdout.starts_with("[stub] "),
            "stub pass for `{}` did not carry the [stub] prefix: {v:?}",
            m.id
        );
        assert!(v.stderr.is_empty());
    }
}

#[test]
fn stub_fails_obviously_wrong_input() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let v = stub_verdict(m.id, "fn main() {}").expect("registry coverage checked");
        assert!(
            !v.ok,
            "stub passed empty `fn main() {{}}` for mission `{}` — needles are too lax",
            m.id
        );
        assert!(
            v.stderr.starts_with("[stub] "),
            "stub fail for `{}` did not carry the [stub] prefix: {v:?}",
            m.id
        );
        assert!(v.stdout.is_empty());
    }
}

#[test]
fn stub_returns_none_for_unknown_encounter() {
    assert!(stub_verdict("totally_made_up_id", "fn main() {}").is_none());
    assert!(stub_verdict("", "anything").is_none());
}

#[test]
fn stub_does_not_cross_grade_solutions() {
    // Same anti-leak guarantee the server-side grader has: solving one
    // mission must not pass another.
    let intro = "fn main() { let answer = 42; }";
    assert!(!stub_verdict("double_function", intro).unwrap().ok);
    assert!(!stub_verdict("borrow_preview", intro).unwrap().ok);

    let double = "fn double(n: i32) -> i32 { n * 2 }";
    assert!(!stub_verdict("intro_let_binding", double).unwrap().ok);

    let match_sol = "fn f(x: Option<i32>) -> i32 { match x { Some(n) => n, None => 0 } }";
    assert!(!stub_verdict("loop_break", match_sol).unwrap().ok);
    assert!(!stub_verdict("struct_basic", match_sol).unwrap().ok);
}
