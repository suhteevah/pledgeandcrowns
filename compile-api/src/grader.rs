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
        // Solving intro should not pass double_function or borrow_preview.
        assert!(!grade("double_function", intro_solution).ok);
        assert!(!grade("borrow_preview", intro_solution).ok);

        let double_solution = "fn double(n: i32) -> i32 { n * 2 }";
        assert!(!grade("intro_let_binding", double_solution).ok);
        assert!(!grade("borrow_preview", double_solution).ok);

        let borrow_solution = r#"fn main() { let value = 1; let r = &value; println!("{r}"); }"#;
        assert!(!grade("intro_let_binding", borrow_solution).ok);
        assert!(!grade("double_function", borrow_solution).ok);
    }
}
