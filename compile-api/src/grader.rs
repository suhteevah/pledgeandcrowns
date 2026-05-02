// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-encounter source-code grader. Pure function, fully unit-tested.
//!
//! v0 is pattern-matching; v1 will swap the body of `grade` for a
//! real `cargo build --offline` + wasmtime invocation per
//! `design/05-tech-architecture.md` §2 — the test suite below stays
//! valid because it only exercises the public contract.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Verdict {
    pub fn pass(msg: impl Into<String>) -> Self {
        Self {
            ok: true,
            stdout: msg.into(),
            stderr: String::new(),
        }
    }
    pub fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            stdout: String::new(),
            stderr: msg.into(),
        }
    }
}

fn requires_all(source: &str, needles: &[&str]) -> Result<(), String> {
    for n in needles {
        if !source.contains(n) {
            return Err(format!("missing required: `{n}`"));
        }
    }
    Ok(())
}

pub fn grade(encounter_id: &str, source: &str) -> Verdict {
    match encounter_id {
        "intro_let_binding" => match requires_all(source, &["let answer", "42"]) {
            Ok(()) => Verdict::pass("answer bound. Ferris nods approvingly."),
            Err(e) => Verdict::fail(format!("not yet — {e}")),
        },
        "double_function" => match requires_all(source, &["fn double", "i32", "* 2"])
            .or_else(|_| requires_all(source, &["fn double", "i32", "*2"]))
        {
            Ok(()) => Verdict::pass("the Trait Mage smiles: `double(21)` would yield 42."),
            Err(e) => Verdict::fail(format!("not yet — {e}")),
        },
        "borrow_preview" => match requires_all(source, &["&value", "println!"]) {
            Ok(()) => Verdict::pass("the Borrow Checker stirs. \"...acceptable. for now.\""),
            Err(e) => Verdict::fail(format!("the Borrow Checker is silent — {e}")),
        },
        // ── Mission 4: declare a mutable binding and increment it.
        "mut_binding" => {
            if !source.contains("let mut") {
                Verdict::fail("the Smith eyes you — missing required: `let mut`")
            } else if !source.contains("+= 1") && !source.contains("+=1") {
                Verdict::fail("the Smith eyes you — missing required: `+= 1`")
            } else {
                Verdict::pass("the Smith taps the anvil. \"good — that one bends.\"")
            }
        }
        // ── Mission 5: branch on i32 sign with if/else.
        "if_else_sign" => {
            if !source.contains("if ") {
                Verdict::fail("the Cartographer frowns — missing required: `if `")
            } else if !source.contains("else") {
                Verdict::fail("the Cartographer frowns — missing required: `else`")
            } else if !source.contains("< 0") && !source.contains("<0") {
                Verdict::fail("the Cartographer frowns — missing required: `< 0`")
            } else {
                Verdict::pass("the Cartographer nods. \"three roads diverge — well chosen.\"")
            }
        }
        // ── Mission 6: `loop` with `break value;` to return from a loop.
        "loop_break" => {
            if !source.contains("loop") {
                Verdict::fail("the Bellringer waits — missing required: `loop`")
            } else if !source.contains("break") {
                Verdict::fail("the Bellringer waits — missing required: `break`")
            } else if !source.contains("*= 2") && !source.contains("*=2") {
                Verdict::fail(
                    "the Bellringer waits — the value should double each iteration: `*= 2`",
                )
            } else {
                Verdict::pass(
                    "the Bellringer pulls the rope. \"the loop yielded its prize — 128.\"",
                )
            }
        }
        // ── Mission 7: match on Option<i32>, handle Some/None.
        "match_option" => {
            if !source.contains("match") {
                Verdict::fail("the Oracle's eyes are closed — missing required: `match`")
            } else if !source.contains("Some(") {
                Verdict::fail("the Oracle's eyes are closed — missing required: `Some(`")
            } else if !source.contains("None") {
                Verdict::fail("the Oracle's eyes are closed — missing required: `None`")
            } else {
                Verdict::pass("the Oracle exhales. \"both paths walked. nothing slips through.\"")
            }
        }
        // ── Mission 9: iterate a Vec and reduce. Teaches `iter`, `sum`.
        "vec_iter" => {
            if !source.contains("vec!") {
                Verdict::fail("the Cooper waits — missing required: `vec!`")
            } else if !source.contains(".iter()") {
                Verdict::fail("the Cooper waits — call `.iter()` on the vector")
            } else if !source.contains(".sum") {
                Verdict::fail("the Cooper waits — finish with `.sum()` to reduce")
            } else {
                Verdict::pass("the Cooper hoops the barrel. \"every stave counted, sum sealed.\"")
            }
        }
        // ── Mission 10: destructure a tuple in a let binding.
        "tuple_destructure" => {
            if !source.contains("let (") {
                Verdict::fail("the Twin waits — missing required: `let (`")
            } else if !source.contains(",") {
                Verdict::fail("the Twin waits — a tuple needs a comma between its parts")
            } else if !source.contains(") =") && !source.contains(")=") {
                Verdict::fail("the Twin waits — close the pattern with `) =`")
            } else {
                Verdict::pass("the Twin grins. \"two names, one breath. cleanly split.\"")
            }
        }
        // ── Mission 11: `while` loop with a decrementing counter.
        "while_loop" => {
            if !source.contains("while ") {
                Verdict::fail("the Tinker frowns — missing required: `while `")
            } else if !source.contains("> 0") && !source.contains(">0") {
                Verdict::fail("the Tinker frowns — the predicate should test `> 0`")
            } else if !source.contains("-= 1") && !source.contains("-=1") {
                Verdict::fail("the Tinker frowns — decrement with `-= 1` each iteration")
            } else {
                Verdict::pass("the Tinker pockets the spring. \"counted down, gear by gear.\"")
            }
        }
        // ── Mission 8: define a struct with named fields and read one.
        "struct_basic" => {
            if !source.contains("struct Knight") {
                Verdict::fail("the Herald squints — missing required: `struct Knight`")
            } else if !source.contains("name:") {
                Verdict::fail("the Herald squints — Knight needs a `name:` field")
            } else if !source.contains("hp:") {
                Verdict::fail("the Herald squints — Knight needs an `hp:` field")
            } else if !source.contains(".name") {
                Verdict::fail("the Herald squints — read the field with `.name`")
            } else {
                Verdict::pass("the Herald unfurls the scroll. \"so named, so numbered.\"")
            }
        }
        _ => Verdict::pass(format!(
            "[freeform] received {} bytes. encounter `{encounter_id}` has no grader yet.",
            source.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── intro_let_binding ──────────────────────────────────────────

    #[test]
    fn intro_pass_canonical() {
        let src = r#"fn main() {
    let answer = 42;
    println!("{answer}");
}"#;
        let v = grade("intro_let_binding", src);
        assert!(v.ok, "expected pass, got {v:?}");
        assert!(v.stdout.contains("Ferris"));
    }

    #[test]
    fn intro_pass_with_type_annotation() {
        let src = "fn main() { let answer: i32 = 42; }";
        assert!(grade("intro_let_binding", src).ok);
    }

    #[test]
    fn intro_fail_missing_value() {
        let src = "fn main() { let answer = 7; }";
        let v = grade("intro_let_binding", src);
        assert!(!v.ok);
        assert!(
            v.stderr.contains("42"),
            "stderr should name the missing token, got {}",
            v.stderr
        );
    }

    #[test]
    fn intro_fail_wrong_name() {
        let src = "fn main() { let x = 42; }";
        let v = grade("intro_let_binding", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("let answer"));
    }

    #[test]
    fn intro_fail_empty() {
        let v = grade("intro_let_binding", "");
        assert!(!v.ok);
    }

    // ── double_function ────────────────────────────────────────────

    #[test]
    fn double_pass_canonical() {
        let src = "fn double(n: i32) -> i32 { n * 2 }";
        assert!(grade("double_function", src).ok);
    }

    #[test]
    fn double_pass_no_space_in_multiply() {
        let src = "fn double(n: i32) -> i32 { n*2 }";
        assert!(grade("double_function", src).ok, "should accept `n*2`");
    }

    #[test]
    fn double_fail_addition() {
        let src = "fn double(n: i32) -> i32 { n + n }";
        assert!(!grade("double_function", src).ok);
    }

    #[test]
    fn double_fail_wrong_type() {
        let src = "fn double(n: u32) -> u32 { n * 2 }";
        let v = grade("double_function", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("i32"));
    }

    #[test]
    fn double_fail_wrong_name() {
        let src = "fn doubl(n: i32) -> i32 { n * 2 }";
        assert!(!grade("double_function", src).ok);
    }

    // ── borrow_preview ─────────────────────────────────────────────

    #[test]
    fn borrow_pass_canonical() {
        let src = r#"fn main() {
    let value = String::from("borrow me");
    let r = &value;
    println!("{r}");
}"#;
        assert!(grade("borrow_preview", src).ok);
    }

    #[test]
    fn borrow_fail_no_borrow() {
        let src = r#"fn main() {
    let value = String::from("borrow me");
    let r = value;
    println!("{r}");
}"#;
        let v = grade("borrow_preview", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&value"));
    }

    #[test]
    fn borrow_fail_no_println() {
        let src = "fn main() { let value = 1; let _r = &value; }";
        let v = grade("borrow_preview", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("println!"));
    }

    // ── unknown encounter falls back to freeform ───────────────────

    #[test]
    fn unknown_encounter_passes_with_freeform_marker() {
        let v = grade("totally_made_up_id", "fn main() {}");
        assert!(v.ok);
        assert!(v.stdout.contains("freeform"));
        assert!(v.stdout.contains("totally_made_up_id"));
    }

    // ── Sanity: solving one mission must not satisfy another ───────

    #[test]
    fn solutions_are_not_cross_compatible() {
        let intro_solution = "fn main() { let answer = 42; }";
        assert!(!grade("double_function", intro_solution).ok);
        assert!(!grade("borrow_preview", intro_solution).ok);

        let double_solution = "fn double(n: i32) -> i32 { n * 2 }";
        assert!(!grade("intro_let_binding", double_solution).ok);
        assert!(!grade("borrow_preview", double_solution).ok);

        let borrow_solution = r#"fn main() { let value = 1; let r = &value; println!("{r}"); }"#;
        assert!(!grade("intro_let_binding", borrow_solution).ok);
        assert!(!grade("double_function", borrow_solution).ok);

        let loop_solution =
            "fn main() { let mut n = 1; loop { if n >= 100 { break n; } n *= 2; }; }";
        assert!(!grade("intro_let_binding", loop_solution).ok);
        assert!(!grade("match_option", loop_solution).ok);

        let match_solution = "fn f(x: Option<i32>) -> i32 { match x { Some(n) => n, None => 0 } }";
        assert!(!grade("loop_break", match_solution).ok);
        assert!(!grade("struct_basic", match_solution).ok);

        let struct_solution = "struct Knight { name: String, hp: i32 } fn main() { let k = Knight { name: String::new(), hp: 0 }; let _ = k.name; }";
        assert!(!grade("loop_break", struct_solution).ok);
        assert!(!grade("match_option", struct_solution).ok);

        let vec_solution = "fn main() { let v = vec![1,2]; let _: i32 = v.iter().sum(); }";
        assert!(!grade("tuple_destructure", vec_solution).ok);
        assert!(!grade("while_loop", vec_solution).ok);

        let tuple_solution = "fn main() { let (a, b) = (1, 2); let _ = (a, b); }";
        assert!(!grade("vec_iter", tuple_solution).ok);
        assert!(!grade("while_loop", tuple_solution).ok);

        let while_solution = "fn main() { let mut n = 3; while n > 0 { n -= 1; } let _ = n; }";
        assert!(!grade("vec_iter", while_solution).ok);
        assert!(!grade("tuple_destructure", while_solution).ok);
    }

    // ── mut_binding ────────────────────────────────────────────────

    #[test]
    fn mut_pass_canonical() {
        let src = "fn main() { let mut x = 0; x += 1; }";
        assert!(grade("mut_binding", src).ok);
    }

    #[test]
    fn mut_pass_no_space() {
        let src = "fn main() { let mut x = 0; x +=1; }";
        assert!(grade("mut_binding", src).ok);
    }

    #[test]
    fn mut_fail_no_mut_keyword() {
        let src = "fn main() { let x = 0; let y = x + 1; }";
        assert!(!grade("mut_binding", src).ok);
    }

    #[test]
    fn mut_fail_no_increment() {
        let src = "fn main() { let mut x = 0; }";
        let v = grade("mut_binding", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("+= 1"));
    }

    // ── if_else_sign ──────────────────────────────────────────────

    #[test]
    fn if_else_pass_canonical() {
        let src = r#"fn sign(n: i32) -> &'static str { if n < 0 { "neg" } else if n == 0 { "zero" } else { "pos" } }"#;
        assert!(grade("if_else_sign", src).ok);
    }

    #[test]
    fn if_else_fail_no_branch() {
        let src = "fn sign(_: i32) -> &'static str { \"pos\" }";
        assert!(!grade("if_else_sign", src).ok);
    }

    #[test]
    fn if_else_fail_no_negative_check() {
        let src = "fn main() { if true { } else { } }";
        let v = grade("if_else_sign", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("< 0"));
    }

    // ── loop_break ────────────────────────────────────────────────

    #[test]
    fn loop_break_pass_canonical() {
        let src = r#"fn main() {
    let mut n = 1;
    let result = loop {
        if n >= 100 { break n; }
        n *= 2;
    };
    println!("{result}");
}"#;
        assert!(grade("loop_break", src).ok);
    }

    #[test]
    fn loop_break_pass_no_space_in_multiply() {
        let src = "fn main() { let mut n = 1; loop { if n >= 100 { break; } n *=2; } }";
        assert!(grade("loop_break", src).ok);
    }

    #[test]
    fn loop_break_fail_no_loop() {
        let src = "fn main() { let n = 128; println!(\"{n}\"); }";
        let v = grade("loop_break", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("loop"));
    }

    #[test]
    fn loop_break_fail_no_doubling() {
        let src = "fn main() { let mut n = 0; loop { if n >= 100 { break; } n += 1; } }";
        let v = grade("loop_break", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("*= 2"));
    }

    // ── match_option ──────────────────────────────────────────────

    #[test]
    fn match_option_pass_canonical() {
        let src = r#"fn unwrap_or_zero(x: Option<i32>) -> i32 {
    match x {
        Some(n) => n,
        None => 0,
    }
}"#;
        assert!(grade("match_option", src).ok);
    }

    #[test]
    fn match_option_fail_no_match() {
        let src = "fn f(x: Option<i32>) -> i32 { x.unwrap_or(0) }";
        let v = grade("match_option", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("match"));
    }

    #[test]
    fn match_option_fail_no_some_arm() {
        let src = "fn f(x: Option<i32>) -> i32 { match x { _ => 0 } }";
        let v = grade("match_option", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Some("));
    }

    #[test]
    fn match_option_fail_no_none_arm() {
        let src = "fn f(x: Option<i32>) -> i32 { match x { Some(n) => n, _ => 0 } }";
        let v = grade("match_option", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("None"));
    }

    // ── struct_basic ──────────────────────────────────────────────

    #[test]
    fn struct_pass_canonical() {
        let src = r#"struct Knight { name: String, hp: i32 }

fn main() {
    let k = Knight { name: String::from("Garin"), hp: 30 };
    println!("{} {}", k.name, k.hp);
}"#;
        assert!(grade("struct_basic", src).ok);
    }

    #[test]
    fn struct_fail_no_struct() {
        let src = "fn main() { let name = \"x\"; println!(\"{name}\"); }";
        let v = grade("struct_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("struct Knight"));
    }

    #[test]
    fn struct_fail_missing_hp_field() {
        let src = "struct Knight { name: String } fn main() { let k = Knight { name: String::new() }; println!(\"{}\", k.name); }";
        let v = grade("struct_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("hp:"));
    }

    #[test]
    fn struct_fail_no_field_access() {
        let src = "struct Knight { name: String, hp: i32 } fn main() {}";
        let v = grade("struct_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".name"));
    }

    // ── vec_iter ──────────────────────────────────────────────────

    #[test]
    fn vec_iter_pass_canonical() {
        let src =
            "fn main() { let v = vec![1, 2, 3]; let s: i32 = v.iter().sum(); println!(\"{s}\"); }";
        assert!(grade("vec_iter", src).ok);
    }

    #[test]
    fn vec_iter_fail_no_vec_macro() {
        let src = "fn main() { let v = [1, 2, 3]; let _: i32 = v.iter().sum(); }";
        let v = grade("vec_iter", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("vec!"));
    }

    #[test]
    fn vec_iter_fail_no_iter() {
        let src = "fn main() { let v = vec![1, 2, 3]; let _ = v.len(); }";
        let v = grade("vec_iter", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".iter()"));
    }

    #[test]
    fn vec_iter_fail_no_sum() {
        let src = "fn main() { let v = vec![1, 2, 3]; let _ = v.iter().count(); }";
        let v = grade("vec_iter", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".sum"));
    }

    // ── tuple_destructure ────────────────────────────────────────

    #[test]
    fn tuple_destructure_pass_canonical() {
        let src = "fn main() { let (a, b) = (3, 4); println!(\"{a} {b}\"); }";
        assert!(grade("tuple_destructure", src).ok);
    }

    #[test]
    fn tuple_destructure_pass_no_space_in_assign() {
        let src = "fn main() { let (a, b)=(3, 4); let _ = (a, b); }";
        assert!(grade("tuple_destructure", src).ok);
    }

    #[test]
    fn tuple_destructure_fail_single_binding() {
        let src = "fn main() { let pair = (3, 4); }";
        let v = grade("tuple_destructure", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("let ("));
    }

    #[test]
    fn tuple_destructure_fail_no_comma() {
        let src = "fn main() { let (a) = (3); let _ = a; }";
        let v = grade("tuple_destructure", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("comma"));
    }

    // ── while_loop ────────────────────────────────────────────────

    #[test]
    fn while_loop_pass_canonical() {
        let src = "fn main() { let mut n = 5; while n > 0 { n -= 1; } println!(\"{n}\"); }";
        assert!(grade("while_loop", src).ok);
    }

    #[test]
    fn while_loop_pass_no_spaces() {
        let src = "fn main() { let mut n = 5; while n>0 { n -=1; } }";
        assert!(grade("while_loop", src).ok);
    }

    #[test]
    fn while_loop_fail_no_while() {
        let src = "fn main() { let mut n = 5; loop { if n > 0 { n -= 1; } else { break; } } }";
        let v = grade("while_loop", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("while "));
    }

    #[test]
    fn while_loop_fail_no_decrement() {
        let src = "fn main() { let mut n = 5; while n > 0 { n += 1; } }";
        let v = grade("while_loop", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("-= 1"));
    }
}
