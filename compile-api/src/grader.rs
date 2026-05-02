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
        // ── Mission 12: take `&mut T`, mutate through the deref. Act 2:
        // exclusive borrow + interior mutation through a reference.
        "borrow_mut" => {
            if !source.contains("fn bump") {
                Verdict::fail("the Forgewright glares — missing required: `fn bump`")
            } else if !source.contains("&mut i32") {
                Verdict::fail(
                    "the Forgewright glares — the parameter must be `&mut i32` (exclusive borrow)",
                )
            } else if !source.contains("*x") {
                Verdict::fail(
                    "the Forgewright glares — write through the reference with a `*x` deref",
                )
            } else {
                Verdict::pass("the Forgewright nods. \"one writer, one anvil — the borrow holds.\"")
            }
        }
        // ── Mission 13: function takes `&str`, callable with both
        // `String` (via `&s`) and a string literal. Act 2: deref coercion
        // and the `String` vs `&str` distinction.
        "string_vs_str" => {
            if !source.contains("fn greet") {
                Verdict::fail("the Linguist tilts her head — missing required: `fn greet`")
            } else if !source.contains("&str") {
                Verdict::fail("the Linguist tilts her head — the parameter type should be `&str`")
            } else if !source.contains("String::from") {
                Verdict::fail(
                    "the Linguist tilts her head — call `greet` once with a `String::from(...)` value",
                )
            } else {
                Verdict::pass(
                    "the Linguist smiles. \"one signature, two callers — &str unifies them.\"",
                )
            }
        }
        // ── Mission 14: handle a missing `Option<T>` with `.unwrap_or`.
        // Act 2: idiomatic non-`match` Option handling.
        "option_unwrap_or" => {
            if !source.contains("Option<") {
                Verdict::fail("the Pilgrim shakes his head — missing required: `Option<`")
            } else if !source.contains(".unwrap_or(") {
                Verdict::fail(
                    "the Pilgrim shakes his head — collapse the absent case with `.unwrap_or(default)`",
                )
            } else {
                Verdict::pass("the Pilgrim raises his lantern. \"the absent path lit a default.\"")
            }
        }
        // ── Mission 15: `for i in 0..N` range iteration. Act 2: the
        // for-loop is the iterator-protocol entry point.
        "for_in_range" => {
            if !source.contains("for ") {
                Verdict::fail("the Drillmaster barks — missing required: `for `")
            } else if !source.contains(" in ") {
                Verdict::fail("the Drillmaster barks — `for` needs a binding `in` an iterable")
            } else if !source.contains("0..10") && !source.contains("0 .. 10") {
                Verdict::fail("the Drillmaster barks — iterate the exact range `0..10`")
            } else {
                Verdict::pass("the Drillmaster claps once. \"ten paces, exact and ordered.\"")
            }
        }
        // ── Mission 16: bind a two-argument closure to a name and
        // invoke it. Act 2 entry to closures + `Fn` traits.
        "closure_basic" => {
            if !source.contains("let add") {
                Verdict::fail("the Reckoner's quill hovers — missing required: `let add`")
            } else if !source.contains("= |") {
                Verdict::fail("the Reckoner's quill hovers — bind a closure literal with `= |...|`")
            } else if !source.contains("+ b") && !source.contains("+b") {
                Verdict::fail("the Reckoner's quill hovers — the body should compute `a + b`")
            } else {
                Verdict::pass("the Reckoner inks the ledger. \"summed in a single stroke.\"")
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
        // ── Mission 17: take a `&[i32]` slice. Act 2: borrowing a
        // contiguous view without owning the buffer.
        "slice_basic" => {
            if !source.contains("fn sum_slice") {
                Verdict::fail("the Quartermaster squints — missing required: `fn sum_slice`")
            } else if !source.contains("&[i32]") {
                Verdict::fail("the Quartermaster squints — the parameter must be a slice `&[i32]`")
            } else if !source.contains(".iter()") {
                Verdict::fail("the Quartermaster squints — walk the slice with `.iter()`")
            } else {
                Verdict::pass(
                    "the Quartermaster ticks the manifest. \"a window into the stores — counted, not claimed.\"",
                )
            }
        }
        // ── Mission 18: propagate a `Result` with the `?` operator.
        // Act 2: error short-circuiting without `match`.
        "result_question_mark" => {
            if !source.contains("Result<") {
                Verdict::fail("the Auditor's pen taps — missing required: `Result<`")
            } else if !source.contains(".parse") {
                Verdict::fail("the Auditor's pen taps — call `.parse` on a string source")
            } else if !source.contains("?;") && !source.contains("?\n") {
                Verdict::fail("the Auditor's pen taps — propagate the error with the `?` operator")
            } else {
                Verdict::pass(
                    "the Auditor closes the ledger. \"the Auditor accepts: errors travel upward.\"",
                )
            }
        }
        // ── Mission 19: derive Debug on a struct, then `{:?}` print it.
        "derive_debug" => {
            if !source.contains("#[derive(Debug)]") {
                Verdict::fail(
                    "the Chronicler shakes his head — missing required: `#[derive(Debug)]`",
                )
            } else if !source.contains("struct Item") {
                Verdict::fail("the Chronicler shakes his head — define `struct Item`")
            } else if !source.contains(":?") {
                Verdict::fail(
                    "the Chronicler shakes his head — print with the debug formatter `{:?}`",
                )
            } else {
                Verdict::pass(
                    "the Chronicler dips his quill. \"derived, not authored — the Chronicler approves.\"",
                )
            }
        }
        // ── Mission 20: iter().map().collect() into a Vec.
        "iter_map_collect" => {
            if !source.contains(".map(") {
                Verdict::fail("the Alchemist stirs — missing required: `.map(`")
            } else if !source.contains(".collect") {
                Verdict::fail("the Alchemist stirs — finish with `.collect` to gather the result")
            } else if !source.contains("|x|") && !source.contains("|x |") {
                Verdict::fail("the Alchemist stirs — the closure should bind one element as `|x|`")
            } else {
                Verdict::pass(
                    "the Alchemist decants the flask. \"the Alchemist measures: every drop transmuted.\"",
                )
            }
        }
        // ── Mission 21: define an enum, match on its variants.
        "enum_match" => {
            if !source.contains("enum Direction") {
                Verdict::fail("the Heraldic Sage frowns — missing required: `enum Direction`")
            } else if !source.contains("match ") {
                Verdict::fail(
                    "the Heraldic Sage frowns — inspect the variant with a `match ` expression",
                )
            } else if !source.contains("Direction::") {
                Verdict::fail(
                    "the Heraldic Sage frowns — name a variant with the `Direction::` path",
                )
            } else {
                Verdict::pass(
                    "the Heraldic Sage raises a banner. \"every quarter named, the Heraldic Sage salutes.\"",
                )
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

        // ── Act 2 additions ──
        let borrow_mut_solution = "fn bump(x: &mut i32) { *x = 99; }";
        assert!(!grade("borrow_preview", borrow_mut_solution).ok);
        assert!(!grade("mut_binding", borrow_mut_solution).ok);
        assert!(!grade("string_vs_str", borrow_mut_solution).ok);

        let str_solution = r#"fn greet(name: &str) { let _ = name; } fn main() { let s = String::from("x"); greet(&s); }"#;
        assert!(!grade("borrow_preview", str_solution).ok);
        assert!(!grade("borrow_mut", str_solution).ok);
        assert!(!grade("option_unwrap_or", str_solution).ok);

        let unwrap_or_solution = "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }";
        assert!(!grade("match_option", unwrap_or_solution).ok);
        assert!(!grade("for_in_range", unwrap_or_solution).ok);

        let for_solution = "fn main() { for i in 0..10 { let _ = i; } }";
        assert!(!grade("while_loop", for_solution).ok);
        assert!(!grade("loop_break", for_solution).ok);
        assert!(!grade("closure_basic", for_solution).ok);

        let closure_solution = "fn main() { let add = |a, b| a + b; let _ = add(1, 2); }";
        assert!(!grade("intro_let_binding", closure_solution).ok);
        assert!(!grade("for_in_range", closure_solution).ok);
        assert!(!grade("borrow_mut", closure_solution).ok);

        // Existing solutions must not pass new graders.
        assert!(!grade("borrow_mut", borrow_solution).ok);
        assert!(!grade("borrow_mut", "fn main() { let mut x = 0; x += 1; }").ok);
        assert!(!grade("string_vs_str", borrow_solution).ok);
        assert!(!grade("option_unwrap_or", match_solution).ok);
        assert!(!grade("for_in_range", while_solution).ok);
        assert!(!grade("closure_basic", "fn main() { let answer = 42; }").ok);

        // ── Act 2 wave 2 additions ──
        let slice_solution = "fn sum_slice(xs: &[i32]) -> i32 { xs.iter().sum() }";
        assert!(!grade("vec_iter", slice_solution).ok);
        assert!(!grade("borrow_mut", slice_solution).ok);
        assert!(!grade("string_vs_str", slice_solution).ok);

        let result_qm_solution = "fn parse_int(s: &str) -> Result<i32, String> { let n = s.parse::<i32>().map_err(|e| e.to_string())?; Ok(n) }";
        assert!(!grade("option_unwrap_or", result_qm_solution).ok);
        assert!(!grade("match_option", result_qm_solution).ok);

        let derive_debug_solution = "#[derive(Debug)] struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{i:?}\"); }";
        assert!(!grade("struct_basic", derive_debug_solution).ok);
        assert!(!grade("string_vs_str", derive_debug_solution).ok);

        let iter_map_solution = "fn main() { let v = vec![1, 2]; let _: Vec<i32> = v.iter().map(|x| x * 2).collect(); }";
        assert!(!grade("vec_iter", iter_map_solution).ok);
        assert!(!grade("closure_basic", iter_map_solution).ok);

        let enum_match_solution = "enum Direction { N, S } fn f(d: Direction) -> i32 { match d { Direction::N => 1, Direction::S => 0 } }";
        assert!(!grade("match_option", enum_match_solution).ok);
        assert!(!grade("struct_basic", enum_match_solution).ok);

        // Existing solutions must not pass the new graders.
        assert!(!grade("slice_basic", borrow_solution).ok);
        assert!(!grade("slice_basic", vec_solution).ok);
        assert!(!grade("result_question_mark", match_solution).ok);
        assert!(!grade("derive_debug", struct_solution).ok);
        assert!(!grade("iter_map_collect", vec_solution).ok);
        assert!(!grade("iter_map_collect", closure_solution).ok);
        assert!(!grade("enum_match", match_solution).ok);
        assert!(!grade("enum_match", struct_solution).ok);
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

    // ── borrow_mut ────────────────────────────────────────────────

    #[test]
    fn borrow_mut_pass_canonical() {
        let src = "fn bump(x: &mut i32) { *x = 99; }";
        assert!(grade("borrow_mut", src).ok);
    }

    #[test]
    fn borrow_mut_pass_with_assignment_expr() {
        let src = "fn bump(x: &mut i32) { *x = *x + 1; } fn main() {}";
        assert!(grade("borrow_mut", src).ok);
    }

    #[test]
    fn borrow_mut_fail_no_mut_borrow() {
        let src = "fn bump(x: &i32) -> i32 { *x }";
        let v = grade("borrow_mut", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&mut i32"));
    }

    #[test]
    fn borrow_mut_fail_no_deref() {
        let src = "fn bump(x: &mut i32) { let _ = x; }";
        let v = grade("borrow_mut", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("*x"));
    }

    // ── string_vs_str ─────────────────────────────────────────────

    #[test]
    fn string_vs_str_pass_canonical() {
        let src = r#"fn greet(name: &str) { println!("hi {name}"); }
fn main() { let s = String::from("you"); greet(&s); greet("anon"); }"#;
        assert!(grade("string_vs_str", src).ok);
    }

    #[test]
    fn string_vs_str_pass_minimal() {
        let src = "fn greet(n: &str) { let _ = n; } fn main() { let s = String::from(\"x\"); greet(&s); }";
        assert!(grade("string_vs_str", src).ok);
    }

    #[test]
    fn string_vs_str_fail_no_str_param() {
        let src = "fn greet(name: String) { let _ = name; } fn main() { let s = String::from(\"x\"); greet(s); }";
        let v = grade("string_vs_str", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&str"));
    }

    #[test]
    fn string_vs_str_fail_no_string_caller() {
        let src = "fn greet(name: &str) { let _ = name; } fn main() { greet(\"anon\"); }";
        let v = grade("string_vs_str", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("String::from"));
    }

    // ── option_unwrap_or ──────────────────────────────────────────

    #[test]
    fn option_unwrap_or_pass_canonical() {
        let src = "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }";
        assert!(grade("option_unwrap_or", src).ok);
    }

    #[test]
    fn option_unwrap_or_pass_with_nonzero_default() {
        let src = "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(42) }";
        assert!(grade("option_unwrap_or", src).ok);
    }

    #[test]
    fn option_unwrap_or_fail_no_option_type() {
        let src = "fn safe(x: i32) -> i32 { x }";
        let v = grade("option_unwrap_or", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Option<"));
    }

    #[test]
    fn option_unwrap_or_fail_used_match_instead() {
        let src = "fn safe(x: Option<i32>) -> i32 { match x { Some(n) => n, None => 0 } }";
        let v = grade("option_unwrap_or", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".unwrap_or("));
    }

    // ── for_in_range ──────────────────────────────────────────────

    #[test]
    fn for_in_range_pass_canonical() {
        let src = "fn main() { for i in 0..10 { println!(\"{i}\"); } }";
        assert!(grade("for_in_range", src).ok);
    }

    #[test]
    fn for_in_range_pass_spaced_range() {
        let src = "fn main() { for i in 0 .. 10 { let _ = i; } }";
        assert!(grade("for_in_range", src).ok);
    }

    #[test]
    fn for_in_range_fail_used_while_instead() {
        let src = "fn main() { let mut i = 0; while i < 10 { i += 1; } }";
        let v = grade("for_in_range", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("for "));
    }

    #[test]
    fn for_in_range_fail_wrong_range() {
        let src = "fn main() { for i in 0..5 { let _ = i; } }";
        let v = grade("for_in_range", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("0..10"));
    }

    // ── closure_basic ─────────────────────────────────────────────

    #[test]
    fn closure_basic_pass_canonical() {
        let src = "fn main() { let add = |a, b| a + b; println!(\"{}\", add(2, 3)); }";
        assert!(grade("closure_basic", src).ok);
    }

    #[test]
    fn closure_basic_pass_no_space_before_b() {
        let src = "fn main() { let add = |a, b| a +b; let _ = add(1, 2); }";
        assert!(grade("closure_basic", src).ok);
    }

    #[test]
    fn closure_basic_fail_used_fn_instead() {
        let src = "fn add(a: i32, b: i32) -> i32 { a + b } fn main() { let _ = add(1, 2); }";
        let v = grade("closure_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("let add"));
    }

    #[test]
    fn closure_basic_fail_wrong_body() {
        let src = "fn main() { let add = |a, b| a * 2 + 0; let _ = add(1, 2); }";
        let v = grade("closure_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("+ b"));
    }

    // ── slice_basic ───────────────────────────────────────────────

    #[test]
    fn slice_basic_pass_canonical() {
        let src = "fn sum_slice(xs: &[i32]) -> i32 { xs.iter().sum() }";
        assert!(grade("slice_basic", src).ok);
    }

    #[test]
    fn slice_basic_pass_with_loop_body() {
        let src =
            "fn sum_slice(xs: &[i32]) -> i32 { let mut s = 0; for x in xs.iter() { s += *x; } s }";
        assert!(grade("slice_basic", src).ok);
    }

    #[test]
    fn slice_basic_fail_no_slice_param() {
        let src = "fn sum_slice(xs: Vec<i32>) -> i32 { xs.iter().sum() }";
        let v = grade("slice_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("&[i32]"));
    }

    #[test]
    fn slice_basic_fail_used_indexing_only() {
        let src = "fn sum_slice(xs: &[i32]) -> i32 { xs[0] }";
        let v = grade("slice_basic", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".iter()"));
    }

    // ── result_question_mark ──────────────────────────────────────

    #[test]
    fn result_qm_pass_canonical() {
        let src = r#"fn parse_int(s: &str) -> Result<i32, String> {
    let n = s.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(n)
}"#;
        assert!(grade("result_question_mark", src).ok);
    }

    #[test]
    fn result_qm_pass_inline_qmark_newline() {
        let src = "fn f() -> Result<i32, String> { let n: i32 = \"1\".parse().map_err(|e: std::num::ParseIntError| e.to_string())?\n;Ok(n) }";
        assert!(grade("result_question_mark", src).ok);
    }

    #[test]
    fn result_qm_fail_no_result_type() {
        let src = "fn f() -> i32 { \"42\".parse::<i32>().unwrap() }";
        let v = grade("result_question_mark", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Result<"));
    }

    #[test]
    fn result_qm_fail_no_question_mark() {
        let src = "fn f() -> Result<i32, String> { let n = \"1\".parse::<i32>().unwrap(); Ok(n) }";
        let v = grade("result_question_mark", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("?"));
    }

    // ── derive_debug ──────────────────────────────────────────────

    #[test]
    fn derive_debug_pass_canonical() {
        let src = r#"#[derive(Debug)]
struct Item { name: String }
fn main() { let item = Item { name: String::from("ring") }; println!("{item:?}"); }"#;
        assert!(grade("derive_debug", src).ok);
    }

    #[test]
    fn derive_debug_pass_with_qmark_format() {
        let src = "#[derive(Debug)] struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{:?}\", i); }";
        assert!(grade("derive_debug", src).ok);
    }

    #[test]
    fn derive_debug_fail_no_derive() {
        let src = "struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{i:?}\"); }";
        let v = grade("derive_debug", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("#[derive(Debug)]"));
    }

    #[test]
    fn derive_debug_fail_used_display_format() {
        let src = "#[derive(Debug)] struct Item { name: String } fn main() { let i = Item { name: String::new() }; println!(\"{}\", i.name); }";
        let v = grade("derive_debug", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(":?"));
    }

    // ── iter_map_collect ──────────────────────────────────────────

    #[test]
    fn iter_map_collect_pass_canonical() {
        let src = "fn main() { let v = vec![1, 2, 3]; let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect(); let _ = doubled; }";
        assert!(grade("iter_map_collect", src).ok);
    }

    #[test]
    fn iter_map_collect_pass_with_turbofish() {
        let src =
            "fn main() { let v = [1, 2]; let _ = v.iter().map(|x| x + 1).collect::<Vec<_>>(); }";
        assert!(grade("iter_map_collect", src).ok);
    }

    #[test]
    fn iter_map_collect_fail_no_map() {
        let src = "fn main() { let v = vec![1, 2, 3]; let _: Vec<&i32> = v.iter().collect(); }";
        let v = grade("iter_map_collect", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".map("));
    }

    #[test]
    fn iter_map_collect_fail_no_collect() {
        let src = "fn main() { let v = vec![1, 2, 3]; let s: i32 = v.iter().map(|x| x * 2).sum(); let _ = s; }";
        let v = grade("iter_map_collect", src);
        assert!(!v.ok);
        assert!(v.stderr.contains(".collect"));
    }

    // ── enum_match ────────────────────────────────────────────────

    #[test]
    fn enum_match_pass_canonical() {
        let src = r#"enum Direction { North, South, East, West }
fn name(d: Direction) -> &'static str {
    match d {
        Direction::North => "n",
        Direction::South => "s",
        Direction::East => "e",
        Direction::West => "w",
    }
}"#;
        assert!(grade("enum_match", src).ok);
    }

    #[test]
    fn enum_match_pass_minimal_two_variants() {
        let src = "enum Direction { Up, Down } fn f(d: Direction) -> i32 { match d { Direction::Up => 1, Direction::Down => 0 } }";
        assert!(grade("enum_match", src).ok);
    }

    #[test]
    fn enum_match_fail_no_enum_decl() {
        let src = "fn name(d: i32) -> &'static str { match d { 0 => \"n\", _ => \"x\" } }";
        let v = grade("enum_match", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("enum Direction"));
    }

    #[test]
    fn enum_match_fail_no_variant_path() {
        let src = "enum Direction { North } fn f(_d: Direction) -> i32 { match 0 { _ => 0 } }";
        let v = grade("enum_match", src);
        assert!(!v.ok);
        assert!(v.stderr.contains("Direction::"));
    }
}
