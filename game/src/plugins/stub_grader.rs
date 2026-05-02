// SPDX-License-Identifier: MIT
//! Offline fallback grader for the in-game compile client.
//!
//! When the live `compile-api` server at 127.0.0.1:7878 is unreachable
//! (Hetzner hiccup, demo machine without server, kiosk mode, etc.) the
//! game must not show a dead-end `[client error]`. Instead, this module
//! re-implements the same pattern checks the server-side grader uses,
//! returning a `[stub]`-prefixed verdict so the player can still feel
//! progression while knowing the grade isn't authoritative.
//!
//! IMPORTANT: this MUST NOT add a runtime dependency on
//! `pledge_compile_api`. The pattern table is duplicated on purpose.
//! The cross-crate contract test in `game/tests/contract.rs` keeps the
//! server-side grader honest; the audit harness ensures the duplicated
//! table doesn't drift in observable behaviour.
//!
//! Hard rule: a stub pass is NEVER persisted as a real mission clear.
//! The caller in `compile_client.rs` is responsible for gating
//! `MissionProgress::mark_cleared` on a "this came from the live API"
//! flag — see `CompileOutcome::from_stub`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StubVerdict {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl StubVerdict {
    fn pass(msg: impl Into<String>) -> Self {
        Self {
            ok: true,
            stdout: format!("[stub] {}", msg.into()),
            stderr: String::new(),
        }
    }
    fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            stdout: String::new(),
            stderr: format!("[stub] {}", msg.into()),
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

/// Re-implementation of the server's per-encounter pattern checks.
///
/// Returns `Some(verdict)` for any mission currently shipped in
/// `MissionRegistry::default()`, or `None` for an unknown encounter id
/// (the caller falls back to the existing `[client error]` path so
/// unexpected ids aren't silently graded as a freeform pass). The
/// `stub_covers_every_mission_in_the_registry` test in `tests/stub_grader.rs`
/// will scream if a new mission is added without a stub arm here.
pub fn stub_verdict(encounter_id: &str, source: &str) -> Option<StubVerdict> {
    let v = match encounter_id {
        "intro_let_binding" => match requires_all(source, &["let answer", "42"]) {
            Ok(()) => StubVerdict::pass("answer bound. Ferris nods approvingly."),
            Err(e) => StubVerdict::fail(format!("not yet — {e}")),
        },
        "double_function" => match requires_all(source, &["fn double", "i32", "* 2"])
            .or_else(|_| requires_all(source, &["fn double", "i32", "*2"]))
        {
            Ok(()) => StubVerdict::pass("the Trait Mage smiles: `double(21)` would yield 42."),
            Err(e) => StubVerdict::fail(format!("not yet — {e}")),
        },
        "borrow_preview" => match requires_all(source, &["&value", "println!"]) {
            Ok(()) => StubVerdict::pass("the Borrow Checker stirs. \"...acceptable. for now.\""),
            Err(e) => StubVerdict::fail(format!("the Borrow Checker is silent — {e}")),
        },
        "mut_binding" => {
            if !source.contains("let mut") {
                StubVerdict::fail("the Smith eyes you — missing required: `let mut`")
            } else if !source.contains("+= 1") && !source.contains("+=1") {
                StubVerdict::fail("the Smith eyes you — missing required: `+= 1`")
            } else {
                StubVerdict::pass("the Smith taps the anvil. \"good — that one bends.\"")
            }
        }
        "if_else_sign" => {
            if !source.contains("if ") {
                StubVerdict::fail("the Cartographer frowns — missing required: `if `")
            } else if !source.contains("else") {
                StubVerdict::fail("the Cartographer frowns — missing required: `else`")
            } else if !source.contains("< 0") && !source.contains("<0") {
                StubVerdict::fail("the Cartographer frowns — missing required: `< 0`")
            } else {
                StubVerdict::pass("the Cartographer nods. \"three roads diverge — well chosen.\"")
            }
        }
        "loop_break" => {
            if !source.contains("loop") {
                StubVerdict::fail("the Bellringer waits — missing required: `loop`")
            } else if !source.contains("break") {
                StubVerdict::fail("the Bellringer waits — missing required: `break`")
            } else if !source.contains("*= 2") && !source.contains("*=2") {
                StubVerdict::fail(
                    "the Bellringer waits — the value should double each iteration: `*= 2`",
                )
            } else {
                StubVerdict::pass(
                    "the Bellringer pulls the rope. \"the loop yielded its prize — 128.\"",
                )
            }
        }
        "match_option" => {
            if !source.contains("match") {
                StubVerdict::fail("the Oracle's eyes are closed — missing required: `match`")
            } else if !source.contains("Some(") {
                StubVerdict::fail("the Oracle's eyes are closed — missing required: `Some(`")
            } else if !source.contains("None") {
                StubVerdict::fail("the Oracle's eyes are closed — missing required: `None`")
            } else {
                StubVerdict::pass(
                    "the Oracle exhales. \"both paths walked. nothing slips through.\"",
                )
            }
        }
        "vec_iter" => {
            if !source.contains("vec!") {
                StubVerdict::fail("the Cooper waits — missing required: `vec!`")
            } else if !source.contains(".iter()") {
                StubVerdict::fail("the Cooper waits — call `.iter()` on the vector")
            } else if !source.contains(".sum") {
                StubVerdict::fail("the Cooper waits — finish with `.sum()` to reduce")
            } else {
                StubVerdict::pass(
                    "the Cooper hoops the barrel. \"every stave counted, sum sealed.\"",
                )
            }
        }
        "tuple_destructure" => {
            if !source.contains("let (") {
                StubVerdict::fail("the Twin waits — missing required: `let (`")
            } else if !source.contains(",") {
                StubVerdict::fail("the Twin waits — a tuple needs a comma between its parts")
            } else if !source.contains(") =") && !source.contains(")=") {
                StubVerdict::fail("the Twin waits — close the pattern with `) =`")
            } else {
                StubVerdict::pass("the Twin grins. \"two names, one breath. cleanly split.\"")
            }
        }
        "while_loop" => {
            if !source.contains("while ") {
                StubVerdict::fail("the Tinker frowns — missing required: `while `")
            } else if !source.contains("> 0") && !source.contains(">0") {
                StubVerdict::fail("the Tinker frowns — the predicate should test `> 0`")
            } else if !source.contains("-= 1") && !source.contains("-=1") {
                StubVerdict::fail("the Tinker frowns — decrement with `-= 1` each iteration")
            } else {
                StubVerdict::pass("the Tinker pockets the spring. \"counted down, gear by gear.\"")
            }
        }
        "struct_basic" => {
            if !source.contains("struct Knight") {
                StubVerdict::fail("the Herald squints — missing required: `struct Knight`")
            } else if !source.contains("name:") {
                StubVerdict::fail("the Herald squints — Knight needs a `name:` field")
            } else if !source.contains("hp:") {
                StubVerdict::fail("the Herald squints — Knight needs an `hp:` field")
            } else if !source.contains(".name") {
                StubVerdict::fail("the Herald squints — read the field with `.name`")
            } else {
                StubVerdict::pass("the Herald unfurls the scroll. \"so named, so numbered.\"")
            }
        }
        "borrow_mut" => {
            if !source.contains("fn bump") {
                StubVerdict::fail("the Forgewright glares — missing required: `fn bump`")
            } else if !source.contains("&mut i32") {
                StubVerdict::fail(
                    "the Forgewright glares — the parameter must be `&mut i32` (exclusive borrow)",
                )
            } else if !source.contains("*x") {
                StubVerdict::fail(
                    "the Forgewright glares — write through the reference with a `*x` deref",
                )
            } else {
                StubVerdict::pass(
                    "the Forgewright nods. \"one writer, one anvil — the borrow holds.\"",
                )
            }
        }
        "string_vs_str" => {
            if !source.contains("fn greet") {
                StubVerdict::fail("the Linguist tilts her head — missing required: `fn greet`")
            } else if !source.contains("&str") {
                StubVerdict::fail(
                    "the Linguist tilts her head — the parameter type should be `&str`",
                )
            } else if !source.contains("String::from") {
                StubVerdict::fail(
                    "the Linguist tilts her head — call `greet` once with a `String::from(...)` value",
                )
            } else {
                StubVerdict::pass(
                    "the Linguist smiles. \"one signature, two callers — &str unifies them.\"",
                )
            }
        }
        "option_unwrap_or" => {
            if !source.contains("Option<") {
                StubVerdict::fail("the Pilgrim shakes his head — missing required: `Option<`")
            } else if !source.contains(".unwrap_or(") {
                StubVerdict::fail(
                    "the Pilgrim shakes his head — collapse the absent case with `.unwrap_or(default)`",
                )
            } else {
                StubVerdict::pass(
                    "the Pilgrim raises his lantern. \"the absent path lit a default.\"",
                )
            }
        }
        "for_in_range" => {
            if !source.contains("for ") {
                StubVerdict::fail("the Drillmaster barks — missing required: `for `")
            } else if !source.contains(" in ") {
                StubVerdict::fail("the Drillmaster barks — `for` needs a binding `in` an iterable")
            } else if !source.contains("0..10") && !source.contains("0 .. 10") {
                StubVerdict::fail("the Drillmaster barks — iterate the exact range `0..10`")
            } else {
                StubVerdict::pass("the Drillmaster claps once. \"ten paces, exact and ordered.\"")
            }
        }
        "closure_basic" => {
            if !source.contains("let add") {
                StubVerdict::fail("the Reckoner's quill hovers — missing required: `let add`")
            } else if !source.contains("= |") {
                StubVerdict::fail(
                    "the Reckoner's quill hovers — bind a closure literal with `= |...|`",
                )
            } else if !source.contains("+ b") && !source.contains("+b") {
                StubVerdict::fail("the Reckoner's quill hovers — the body should compute `a + b`")
            } else {
                StubVerdict::pass("the Reckoner inks the ledger. \"summed in a single stroke.\"")
            }
        }
        _ => return None,
    };
    Some(v)
}
