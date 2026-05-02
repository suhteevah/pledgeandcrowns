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
/// Returns `Some(verdict)` for any of the 11 missions currently shipped
/// in `MissionRegistry::default()`, or `None` for an unknown encounter
/// id (the caller falls back to the existing `[client error]` path so
/// unexpected ids aren't silently graded as a freeform pass).
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
        _ => return None,
    };
    Some(v)
}
