// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-mission behaviour checks for `/compile-real`.
//!
//! Building and running the player's program only proves it compiles and
//! exits cleanly. This module turns that into a grade: each mission id
//! (the `encounter_id` the client already sends) maps to a [`Check`]:
//!
//! - [`Check::Stdout`]: run the player's own `main`, normalise its stdout
//!   ([`normalize`]) and apply a small set of [`Matcher`]s. Used when the
//!   mission text pins what the program prints.
//! - [`Check::Harness`]: the mission asks for a function or type, so the
//!   server compiles its own checker next to the player's code, calls the
//!   player's items with fixed inputs and compares what they return with
//!   lines recorded from the canonical solution. See **Harness layout**.
//! - [`Check::RunOnly`]: for a mission that lets the player choose the
//!   values (e.g. "build a Vec and sum it"), so there is nothing fixed to
//!   compare. The program is built and run; the response says the output
//!   was not checked. No mission uses it today: every runnable mission's
//!   text pins its output.
//! - [`Check::CompileOnly`]: the program cannot be run meaningfully on
//!   `wasm32-wasip1` (threads). It is built, never run, and checked with
//!   token-level static checks. The response says behaviour was not
//!   checked on this server.
//!
//! A `Stdout` or `Harness` entry may also list [`MissionCheck::uses`]:
//! token groups that must appear in the code once the behaviour matched
//! (failing them gives [`GradeState::StaticCheckFailed`]). They close the
//! "print the expected lines as literals" shortcut for missions whose
//! output is short, without claiming more than a token check proves.
//! It may also list [`MissionCheck::forbids`]: tokens that must not appear
//! (whole words, comments and strings removed), for a mission whose text
//! rules a construct out.
//!
//! An id with no entry keeps the old behaviour (built + ran = ok) and the
//! response says no output check was applied.
//!
//! # Harness layout (server-owned; the player's source is still one file)
//!
//! ```text
//! Cargo.toml                 server-written, `autobins = false`,
//!                            bin path = src/pnc_entry.rs
//! src/pnc_entry.rs           server: `mod main; fn main() { main::__pnc_harness::run(); }`
//! src/main.rs                PLAYER_PREAMBLE + player source + one appended line:
//!                            `pub(crate) mod __pnc_harness;`
//! src/main/__pnc_harness.rs  server: the checker, a CHILD module of the player's code
//! ```
//!
//! - The player's code is the module `main`, so its file is still
//!   `src/main.rs` and keeps the same one-line preamble: rustc line numbers
//!   for player code are unchanged and the client's
//!   `SERVER_PREAMBLE_LINES = 1` stays correct. The checker declaration is
//!   appended *after* the player's last line, so it shifts nothing.
//! - The checker is a child module, so it can reach the player's private
//!   items through `super::` (Rust privacy is module-scoped and children see
//!   their ancestors), and the player's names can never collide with the
//!   checker's: they live in different modules. The checker uses absolute
//!   paths (`::std::println!`, `::core::fmt::Debug`) because `macro_rules!`
//!   macros the player defines are in textual scope inside child modules.
//! - A player `fn main` is just `main::main`; the checker calls it only for
//!   missions that check what `main` prints ([`Harness::run_main`]).
//! - Identifiers containing [`RESERVED_PREFIX`] are refused up front, so a
//!   player cannot define a `__pnc_harness` module of their own.
//! - Checker output lines carry a per-request random nonce, which the
//!   checker keeps in a private `const` the player's module cannot name.
//!   Only nonce-tagged lines count as checker results, so a player cannot
//!   pass by printing the expected lines. The nonce is stripped before
//!   anything reaches the player.
//! - If the player's code makes the checker fail to compile (wrong
//!   signature, missing item), the diagnostics located in server files are
//!   removed ([`filter_harness_diagnostics`]) and replaced with one
//!   player-facing line describing what the checker needs
//!   ([`Harness::contract`]); diagnostics in `src/main.rs` pass through
//!   untouched.
//!
//! Known limit: player code that names `crate::...` resolves against the
//! server entry crate, not its own file, in harness missions. No mission
//! needs `crate::` paths.
//!
//! # Harness modes ([`HarnessMode`], per mission, chosen by the server)
//!
//! - [`HarnessMode::ForbidUnsafe`] (every registered mission but four):
//!   the entry has `#![forbid(unsafe_code)]` and the nonce is a `const` in
//!   the checker, as described above.
//! - [`HarnessMode::UnsafeLateNonce`] (opt-in, for the four Act 9 harness
//!   missions whose lesson needs `unsafe` in the code the checker calls:
//!   `extern_c`, `send_sync`, `librarian_index`, `librarian_seal`, pinned
//!   by a unit test; HARNESS-NOTES.md design 1): no `forbid`, and NO nonce in the
//!   program. The checker formats every result to owned text while it
//!   calls the player's code, lets every player value drop, resets the
//!   panic hook, and only then fetches the nonce once from a host import
//!   (`wasm_runner::LateNonce`) and prints the tagged lines and the done
//!   mark in one write. The nonce is therefore not in guest memory at any
//!   moment player code runs before the checker's last call returns.
//!   Player `unsafe` code can still corrupt checker state in the shared
//!   memory before that point (stored results, allocator, stdout), so
//!   this raises the bar from "scan memory" to "exploit the checker's
//!   heap"; it is NOT a guarantee. [`UNSAFE_HARNESS_FORBIDS`] adds static
//!   rules against the cheapest ways to run code during the late print.

mod data;

use serde::{Deserialize, Serialize};

use crate::wasm_builder::{self, BuildSettings, Layout, RealOutcome, RunLimits};
use crate::wasm_runner::LateNonce;

/// Identifier prefix reserved for server checker code.
pub const RESERVED_PREFIX: &str = "__pnc";

/// Lines of server text placed before the player's source in
/// `src/main.rs`, for both plain and harness builds. The game client's
/// `SERVER_PREAMBLE_LINES` must equal this.
pub const PLAYER_PREAMBLE_LINES: usize = wasm_builder::PLAYER_PREAMBLE_LINES;

/// Most output lines quoted back to the player in a failure message.
const MAX_QUOTED_LINES: usize = 12;
/// Most characters quoted back to the player in a failure message.
const MAX_QUOTED_CHARS: usize = 600;
/// Most mismatching checker lines listed in one failure message.
const MAX_LISTED_MISMATCHES: usize = 3;

/// One condition on normalised stdout. All matchers of a check must hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Matcher {
    /// The whole normalised output equals this text.
    Exact(&'static str),
    /// Some line equals this text.
    HasLine(&'static str),
    /// Some line contains this text.
    LineContains(&'static str),
    /// Some single line contains all of these pieces, in this order.
    LineContainsAll(&'static [&'static str]),
    /// These exact lines appear in this order (other lines may come between).
    InOrder(&'static [&'static str]),
    /// The normalised output starts with this text.
    StartsWith(&'static str),
    /// The normalised output ends with this text.
    EndsWith(&'static str),
    /// The last line equals this text.
    LastLine(&'static str),
}

impl Matcher {
    /// True when `output` (already normalised) satisfies the matcher.
    pub fn matches(&self, output: &str) -> bool {
        let mut lines = output.lines();
        match *self {
            Self::Exact(want) => output == want,
            Self::HasLine(want) => lines.any(|l| l == want),
            Self::LineContains(want) => lines.any(|l| l.contains(want)),
            Self::LineContainsAll(parts) => lines.any(|l| contains_in_order(l, parts)),
            Self::InOrder(want) => {
                let mut want = want.iter().peekable();
                for line in lines {
                    if want.peek().is_some_and(|w| **w == line) {
                        want.next();
                    }
                }
                want.peek().is_none()
            }
            Self::StartsWith(want) => output.starts_with(want),
            Self::EndsWith(want) => output.ends_with(want),
            Self::LastLine(want) => lines.last() == Some(want),
        }
    }

    /// Player-facing description of what the matcher wants.
    pub fn describe(&self) -> String {
        match *self {
            Self::Exact(want) => format!("exactly this output:\n{want}"),
            Self::HasLine(want) => format!("a line reading `{want}`"),
            Self::LineContains(want) => format!("a line containing `{want}`"),
            Self::LineContainsAll(parts) => {
                let quoted: Vec<String> = parts.iter().map(|p| format!("`{p}`")).collect();
                format!("a line containing {}", quoted.join(" then "))
            }
            Self::InOrder(want) => {
                let quoted: Vec<String> = want.iter().map(|p| format!("`{p}`")).collect();
                format!("these lines in this order: {}", quoted.join(", "))
            }
            Self::StartsWith(want) => format!("output starting with `{want}`"),
            Self::EndsWith(want) => format!("output ending with `{want}`"),
            Self::LastLine(want) => format!("a last line reading `{want}`"),
        }
    }
}

fn contains_in_order(line: &str, parts: &[&str]) -> bool {
    let mut rest = line;
    for part in parts {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    true
}

/// Server checker compiled alongside the player's code.
#[derive(Debug, Clone, Copy)]
pub struct Harness {
    /// Player-facing statement of the items the checker calls, shown when
    /// the player's code does not fit them.
    pub contract: &'static str,
    /// Call the player's `main` first and apply [`Harness::main_matchers`]
    /// to what it prints. Also requires a `fn main` to exist.
    pub run_main: bool,
    /// Rust statements run by the checker. Reach player items with
    /// `super::`; report with `emit(label, value)` (Debug-formatted).
    pub body: &'static str,
    /// The `label = value` lines the canonical solution emits, in order.
    pub expected: &'static [&'static str],
    /// Conditions on the player's `main` output (only with `run_main`).
    pub main_matchers: &'static [Matcher],
}

/// How one mission is graded.
#[derive(Debug, Clone, Copy)]
pub enum Check {
    Stdout {
        matchers: &'static [Matcher],
    },
    Harness(Harness),
    RunOnly {
        reason: &'static str,
    },
    CompileOnly {
        reason: &'static str,
        /// Each group must have at least one token present in the code
        /// (comments and string literals removed).
        required: &'static [&'static [&'static str]],
    },
}

/// How a [`Check::Harness`] mission's checker is built and keyed. Chosen by
/// the server per mission id; nothing in the player's source can change
/// it. Ignored for the other check kinds (their builds never contain a
/// nonce).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessMode {
    /// `#![forbid(unsafe_code)]` in the server entry; the nonce is compiled
    /// into the checker. Every mission but the four Act 9 opt-ins.
    ForbidUnsafe,
    /// `unsafe` allowed (the lesson needs it); the nonce is NOT compiled in
    /// and reaches the checker from the host only after its last call into
    /// player code (HARNESS-NOTES.md design 1). Raises the bar, does not
    /// guarantee: see the module docs and HARNESS-NOTES.md.
    UnsafeLateNonce,
}

/// Tokens refused in every [`HarnessMode::UnsafeLateNonce`] mission on top
/// of its own [`MissionCheck::forbids`] (whole words, comments and strings
/// removed, checked after the behaviour passed). Each names a way for
/// player code to run *during* the checker's late print, when the nonce
/// is in memory: a `#[global_allocator]` is called for every allocation
/// the print makes, and `#[export_name]` can replace a symbol std uses
/// (e.g. `memcpy`). `no_mangle` does the same but is part of the FFI
/// lesson, so missions forbid it individually where they can.
pub const UNSAFE_HARNESS_FORBIDS: &[&str] = &["global_allocator", "export_name"];

/// A mission id and its check.
#[derive(Debug, Clone, Copy)]
pub struct MissionCheck {
    pub id: &'static str,
    pub check: Check,
    /// How a harness check is built (see [`HarnessMode`]).
    /// [`HarnessMode::ForbidUnsafe`] for every registered mission.
    pub harness_mode: HarnessMode,
    /// Static token groups (same rule as [`Check::CompileOnly`]'s
    /// `required`) applied AFTER a [`Check::Stdout`] or [`Check::Harness`]
    /// check has passed. Used where the pinned output alone could be
    /// produced by printing literals: matching output plus these tokens is
    /// evidence (not proof) that the mission's construct produced it.
    /// Empty for every other check kind.
    pub uses: &'static [&'static [&'static str]],
    /// Tokens that must NOT appear in the code (comments and string
    /// literals removed), checked after a passing behaviour check, like
    /// [`MissionCheck::uses`]. For a mission whose text rules a construct
    /// out (the Barkeep's last trial stage: no `match`, `unwrap`, `expect`
    /// or `panic!`; the Keymaster's: no `for`, `while`, `loop` or `mut`).
    /// A token that starts or ends with an identifier character only
    /// matches as a whole word (`match` does not match `matches!`). Empty
    /// for almost every mission.
    pub forbids: &'static [&'static str],
}

/// Every mission with a check, in registry order.
pub fn all() -> &'static [MissionCheck] {
    data::CHECKS
}

/// The check for `encounter_id`, if any.
pub fn lookup(encounter_id: &str) -> Option<&'static MissionCheck> {
    data::CHECKS.iter().find(|m| m.id == encounter_id)
}

/// Grade state reported in the response's `grade` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GradeState {
    /// Built, ran, and its behaviour matched the mission.
    Passed,
    /// Did not build (rustc errors, or refused before building).
    CompileError,
    /// Built, but trapped, panicked, timed out or exited non-zero.
    RuntimeError,
    /// Built and ran, but did not do what the mission asks.
    WrongOutput,
    /// Built, but static checks failed: a compile-only mission's
    /// `required` tokens, or a checked mission's [`MissionCheck::uses`]
    /// tokens (its behaviour matched, but not via the taught construct).
    StaticCheckFailed,
    /// Built and passed static checks; behaviour NOT checked (cannot run
    /// on this server).
    CompiledNotChecked,
    /// Built and ran cleanly; output NOT checked (no fixed expectation).
    RanNotChecked,
}

impl GradeState {
    /// The `ok` the existing client reads.
    pub fn ok(self) -> bool {
        matches!(
            self,
            Self::Passed | Self::CompiledNotChecked | Self::RanNotChecked
        )
    }

    /// True when behaviour was actually compared with an expectation.
    pub fn behaviour_checked(self) -> bool {
        matches!(self, Self::Passed | Self::WrongOutput)
    }
}

/// Result of grading one submission.
#[derive(Debug, Clone)]
pub struct GradedRun {
    /// Build + run outcome. `stdout` is what the player may see (checker
    /// lines removed); `stderr` is player-facing (checker diagnostics
    /// replaced).
    pub outcome: RealOutcome,
    pub state: GradeState,
    /// What the check wanted, when it failed. Player-facing.
    pub expected: Option<String>,
    /// What the program did instead, truncated. Player-facing.
    pub actual: Option<String>,
    /// One-line explanation of how the submission was graded.
    pub note: String,
}

impl GradedRun {
    pub fn ok(&self) -> bool {
        self.state.ok()
    }

    /// Wrap a raw outcome whose output was not checked (unknown mission,
    /// run-only mission, or an injected test grader).
    pub fn unchecked(outcome: RealOutcome, note: impl Into<String>) -> Self {
        let state = if !outcome.compiled {
            GradeState::CompileError
        } else if !outcome.ran_ok {
            GradeState::RuntimeError
        } else {
            GradeState::RanNotChecked
        };
        Self {
            outcome,
            state,
            expected: None,
            actual: None,
            note: note.into(),
        }
    }
}

/// Normalise program output for comparison: `\r\n` and lone `\r` become
/// `\n`, trailing whitespace is removed from every line, and trailing
/// blank lines are dropped. Leading whitespace and blank lines between
/// lines are kept.
pub fn normalize(output: &str) -> String {
    let unified = output.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = unified.lines().map(str::trim_end).collect();
    let end = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map_or(0, |i| i + 1);
    lines[..end].join("\n")
}

/// Truncate text quoted back to the player.
pub fn quote_for_player(text: &str) -> String {
    if text.is_empty() {
        return "(nothing)".to_string();
    }
    let mut out = String::new();
    let mut truncated = false;
    for (n, line) in text.lines().enumerate() {
        if n >= MAX_QUOTED_LINES || out.len() + line.len() > MAX_QUOTED_CHARS {
            truncated = true;
            break;
        }
        if n > 0 {
            out.push('\n');
        }
        out.push_str(line);
    }
    if out.is_empty() {
        // A single line longer than the cap.
        out = text.chars().take(MAX_QUOTED_CHARS).collect();
        truncated = true;
    }
    if truncated {
        out.push_str("\n... (truncated)");
    }
    out
}

/// Result of applying matchers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchFailure {
    pub expected: String,
    pub actual: String,
}

/// Apply every matcher to `raw` stdout. `Err` names the first failure.
pub fn check_stdout(matchers: &[Matcher], raw: &str) -> Result<(), MatchFailure> {
    let output = normalize(raw);
    for m in matchers {
        if !m.matches(&output) {
            tracing::debug!(matcher = ?m, "expectations: stdout matcher failed");
            return Err(MatchFailure {
                expected: m.describe(),
                actual: quote_for_player(&output),
            });
        }
    }
    Ok(())
}

/// Checker output split into the player's own prints and checker results.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Transcript {
    /// Lines the player's code printed (all of them, checker lines removed).
    pub program: String,
    /// What the player's `main` printed, when the checker ran it.
    pub main_output: Option<String>,
    /// `label = value` lines from the checker, nonce removed.
    pub results: Vec<String>,
    /// The checker reached its end.
    pub finished: bool,
    /// A nonce-tagged line appeared after the `<done>` mark (or a second
    /// `<done>`). The honest checker never does that, so the run is not
    /// graded as a pass. Such lines are not counted as results.
    pub after_done: bool,
}

/// Split raw harness stdout on the request nonce.
pub fn parse_transcript(raw: &str, nonce: &str) -> Transcript {
    parse_transcript_tagged(raw, nonce, nonce)
}

/// Split raw harness stdout: `<main>`/`</main>` marks carry `mark`, results
/// and `<done>` carry `nonce`. In [`HarnessMode::ForbidUnsafe`] both are
/// the nonce. In [`HarnessMode::UnsafeLateNonce`] the main marks are
/// printed while player code runs, before the nonce exists, so they carry
/// a separate non-secret per-request mark: forging one only moves the
/// boundary of the player's own `main` output, which the player controls
/// anyway.
pub fn parse_transcript_tagged(raw: &str, nonce: &str, mark: &str) -> Transcript {
    let tag = format!("{nonce} ");
    let mark_tag = format!("{mark} ");
    let mut t = Transcript::default();
    let mut program = Vec::new();
    let mut main_lines: Option<Vec<&str>> = None;
    let mut in_main = false;
    let unified = raw.replace("\r\n", "\n");
    for line in unified.lines() {
        if let Some(rest) = line.strip_prefix(&mark_tag) {
            match rest.trim_end() {
                "<main>" => {
                    in_main = true;
                    main_lines = Some(Vec::new());
                    continue;
                }
                "</main>" => {
                    in_main = false;
                    continue;
                }
                _ => {}
            }
        }
        let Some(rest) = line.strip_prefix(&tag) else {
            program.push(line);
            if in_main && let Some(m) = main_lines.as_mut() {
                m.push(line);
            }
            continue;
        };
        if t.finished {
            t.after_done = true;
            continue;
        }
        match rest.trim_end() {
            "<done>" => t.finished = true,
            result => t.results.push(result.to_string()),
        }
    }
    t.program = program.join("\n");
    t.main_output = main_lines.map(|l| l.join("\n"));
    t
}

/// Compare checker results with the recorded lines. `Err` lists up to
/// [`MAX_LISTED_MISMATCHES`] differences in player-facing form.
pub fn compare_results(expected: &[&str], got: &[String]) -> Result<(), MatchFailure> {
    let mut want_lines = Vec::new();
    let mut got_lines = Vec::new();
    for i in 0..expected.len().max(got.len()) {
        let want = expected.get(i).copied();
        let have = got.get(i).map(String::as_str);
        if want == have {
            continue;
        }
        if want_lines.len() >= MAX_LISTED_MISMATCHES {
            break;
        }
        match (want, have) {
            (Some(w), Some(h)) => {
                let (label, w_val) = split_result(w);
                let (_, h_val) = split_result(h);
                want_lines.push(format!("{label} should give {w_val}"));
                got_lines.push(format!("{label} gave {h_val}"));
            }
            (Some(w), None) => {
                let (label, w_val) = split_result(w);
                want_lines.push(format!("{label} should give {w_val}"));
                got_lines.push(format!("{label}: the checker never got that far"));
            }
            (None, Some(h)) => {
                want_lines.push("no further results".to_string());
                got_lines.push(format!("unexpected extra result: {h}"));
            }
            (None, None) => {}
        }
    }
    if want_lines.is_empty() {
        Ok(())
    } else {
        Err(MatchFailure {
            expected: want_lines.join("\n"),
            actual: got_lines.join("\n"),
        })
    }
}

fn split_result(line: &str) -> (&str, &str) {
    line.split_once(" = ").unwrap_or((line, ""))
}

/// Source of the checker module for `h`, tagged with `nonce`.
pub fn harness_module_source(h: &Harness, nonce: &str) -> String {
    let main_call = if h.run_main {
        "    mark(\"main\");\n    let _ = super::main();\n    mark(\"/main\");\n"
    } else {
        ""
    };
    format!(
        "// Server-owned mission checker (pledge_compile_api). Not player code.\n\
         #![allow(unused, dead_code, non_snake_case, unused_must_use, clippy::all)]\n\
         const NONCE: &str = \"{nonce}\";\n\
         fn emit<T: ::core::fmt::Debug>(label: &str, value: T) {{\n    \
             ::std::println!(\"{{}} {{}} = {{:?}}\", NONCE, label, value);\n\
         }}\n\
         fn mark(tag: &str) {{\n    ::std::println!(\"{{}} <{{}}>\", NONCE, tag);\n}}\n\
         pub(crate) fn run() {{\n{main_call}{body}\n    mark(\"done\");\n}}\n",
        body = h.body
    )
}

/// Source of the checker module for `h` in [`HarnessMode::UnsafeLateNonce`].
///
/// No nonce appears in it. `mark` (non-secret, per request) tags the
/// `<main>` marks; `import` is the per-request name of the host's one-shot
/// nonce function (`wasm_runner::LateNonce`). Order of events in `run`:
///
/// 1. `calls()`: the player's `main` (if `run_main`) and the mission body.
///    `emit` formats each result to an owned `String` and drops the player
///    value right there; every other player value is a local of `calls`
///    and is dropped when it returns.
/// 2. `finish()`: reset the panic hook (dropping any the player installed),
///    take the stored lines, THEN fetch the nonce (the first moment it is
///    in guest memory), build the whole tagged block and print it in one
///    write, flushed. Nothing in `finish` calls player code (barring the
///    corruptions HARNESS-NOTES.md lists as the known limit).
///
/// The one `unsafe` block is the FFI call to the host import (hard rule 1's
/// FFI exception); it lives in server-written guest code, never in the
/// server process.
pub fn harness_module_source_late(h: &Harness, mark: &str, import: &str) -> String {
    let main_call = if h.run_main {
        "    mark(\"main\");\n    let _ = super::main();\n    mark(\"/main\");\n"
    } else {
        ""
    };
    format!(
        "// Server-owned mission checker (pledge_compile_api). Not player code.\n\
         // Harness mode UnsafeLateNonce: the result key is fetched from the host\n\
         // after the last call into player code; it is not in this file.\n\
         #![allow(unused, dead_code, non_snake_case, unused_must_use, clippy::all)]\n\
         const MARK: &str = \"{mark}\";\n\
         const KEY_LEN: usize = {key_len};\n\
         static RESULTS: ::std::sync::Mutex<::std::vec::Vec<::std::string::String>> =\n    \
             ::std::sync::Mutex::new(::std::vec::Vec::new());\n\
         #[link(wasm_import_module = \"{module}\")]\n\
         unsafe extern \"C\" {{\n    \
             #[link_name = \"{import}\"]\n    \
             fn take_key(ptr: *mut u8, len: usize) -> i32;\n\
         }}\n\
         fn emit<T: ::core::fmt::Debug>(label: &str, value: T) {{\n    \
             let line = ::std::format!(\"{{}} = {{:?}}\", label, value);\n    \
             ::core::mem::drop(value);\n    \
             if let ::core::result::Result::Ok(mut r) = RESULTS.lock() {{\n        \
                 r.push(line);\n    \
             }}\n\
         }}\n\
         fn mark(tag: &str) {{\n    ::std::println!(\"{{}} <{{}}>\", MARK, tag);\n}}\n\
         fn calls() {{\n{main_call}{body}\n}}\n\
         fn finish() {{\n    \
             ::core::mem::drop(::std::panic::take_hook());\n    \
             let results = match RESULTS.lock() {{\n        \
                 ::core::result::Result::Ok(mut r) => ::core::mem::take(&mut *r),\n        \
                 ::core::result::Result::Err(_) => return,\n    \
             }};\n    \
             let mut key = [0u8; KEY_LEN];\n    \
             // SAFETY: `take_key` is the host's late-nonce import. It writes at\n    \
             // most `len` bytes at `ptr` through bounds-checked host code, and\n    \
             // `key` is a live local buffer of exactly `len` bytes.\n    \
             let rc = unsafe {{ take_key(key.as_mut_ptr(), key.len()) }};\n    \
             if rc != 0 {{\n        \
                 ::std::eprintln!(\"mission checker: the server refused its result key ({{}})\", rc);\n        \
                 return;\n    \
             }}\n    \
             let ::core::result::Result::Ok(key) = ::core::str::from_utf8(&key) else {{ return; }};\n    \
             let mut block = ::std::string::String::new();\n    \
             for r in &results {{\n        \
                 block.push_str(key);\n        \
                 block.push(' ');\n        \
                 block.push_str(r);\n        \
                 block.push('\\n');\n    \
             }}\n    \
             block.push_str(key);\n    \
             block.push_str(\" <done>\\n\");\n    \
             ::std::print!(\"{{}}\", block);\n    \
             let _ = ::std::io::Write::flush(&mut ::std::io::stdout());\n\
         }}\n\
         pub(crate) fn run() {{\n    calls();\n    finish();\n}}\n",
        key_len = crate::wasm_runner::NONCE_LEN,
        module = crate::wasm_runner::LATE_NONCE_MODULE,
        body = h.body
    )
}

/// Server entry crate root for `mode`: [`HARNESS_ENTRY_SOURCE`] (with
/// `#![forbid(unsafe_code)]`) for [`HarnessMode::ForbidUnsafe`],
/// [`HARNESS_ENTRY_SOURCE_UNSAFE_ALLOWED`] only for
/// [`HarnessMode::UnsafeLateNonce`].
pub fn harness_entry_source(mode: HarnessMode) -> &'static str {
    match mode {
        HarnessMode::ForbidUnsafe => HARNESS_ENTRY_SOURCE,
        HarnessMode::UnsafeLateNonce => HARNESS_ENTRY_SOURCE_UNSAFE_ALLOWED,
    }
}

/// [`HARNESS_ENTRY_SOURCE`] without the `forbid`, for
/// [`HarnessMode::UnsafeLateNonce`] missions only. Same line count and
/// layout otherwise, so diagnostics behave the same.
pub const HARNESS_ENTRY_SOURCE_UNSAFE_ALLOWED: &str = "// Server-owned entry point (pledge_compile_api). The player's code is\n\
// the `main` module, src/main.rs.\n\
// unsafe allowed: harness mode UnsafeLateNonce (see HARNESS-NOTES.md).\n\
#[allow(special_module_name)]\n\
mod main;\n\
fn main() {\n    main::__pnc_harness::run();\n}\n";

/// Source of the server entry crate root for harness builds.
///
/// `#![forbid(unsafe_code)]` (third security review, C-1): the checker's
/// nonce is a constant in the program's own memory, so `unsafe` player code
/// could scan linear memory for it, print forged checker lines and exit.
/// `forbid` at the crate root covers every module, the player's `main`
/// module included, and cannot be relaxed by an inner `#![allow]` there.
/// It lives in this server file, so `src/main.rs` (and the client's
/// `SERVER_PREAMBLE_LINES = 1`) is unchanged.
pub const HARNESS_ENTRY_SOURCE: &str = "// Server-owned entry point (pledge_compile_api). The player's code is\n\
// the `main` module, src/main.rs.\n\
#![forbid(unsafe_code)]\n\
#[allow(special_module_name)]\n\
mod main;\n\
fn main() {\n    main::__pnc_harness::run();\n}\n";

/// First line of the compile error shown when a harness mission's code
/// uses `unsafe`.
pub const UNSAFE_REFUSED_MESSAGE: &str = "error: this mission is checked by calling your code, \
     and its checker does not allow `unsafe` code. Remove the `unsafe` block, function, impl or \
     attribute: every mission before Act 9 can be solved in safe Rust.";

/// True if rustc refused the build because of the `unsafe_code` lint
/// (`#![forbid(unsafe_code)]` in [`HARNESS_ENTRY_SOURCE`]).
pub fn refused_for_unsafe(stderr: &str) -> bool {
    stderr.contains("forbid(unsafe_code)")
        || stderr.lines().any(|l| {
            l.starts_with("error: usage of an `unsafe`")
                || l.starts_with("error: declaration of an `unsafe`")
                || l.starts_with("error: implementation of an `unsafe`")
        })
}

/// Make an `unsafe_code` refusal readable: lead with
/// [`UNSAFE_REFUSED_MESSAGE`] and drop rustc's notes that point into the
/// server's entry file (`the lint level is defined here --> src/pnc_entry.rs`),
/// which a learner cannot see. Player diagnostics (`--> src/main.rs:L:C`)
/// are kept verbatim, so their line numbers are unchanged.
pub fn explain_unsafe_refusal(stderr: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut lines = stderr.lines().peekable();
    while let Some(l) = lines.next() {
        let t = l.trim_start();
        // The lint note (`--> src/pnc_entry.rs`), and E0453's secondary
        // span (`::: src/pnc_entry.rs`, "`forbid` level set here").
        if t.starts_with("note: the lint level is defined here")
            || (t.starts_with(":::") && t.contains("pnc_entry.rs"))
        {
            // Skip the note and its snippet (`-->`, gutter, caret lines).
            while let Some(next) = lines.peek() {
                let n = next.trim_start();
                let snippet = n.starts_with("-->")
                    || n.starts_with('|')
                    || n.starts_with('^')
                    || n.chars().next().is_some_and(|c| c.is_ascii_digit());
                if !snippet {
                    break;
                }
                lines.next();
            }
            continue;
        }
        kept.push(l);
    }
    format!("{UNSAFE_REFUSED_MESSAGE}\n{}", kept.join("\n"))
}

/// Fresh per-request nonce.
fn new_nonce() -> String {
    format!("PNC{}", uuid::Uuid::new_v4().simple())
}

/// Fresh per-request `<main>` mark for [`HarnessMode::UnsafeLateNonce`]
/// (not a secret; see [`parse_transcript_tagged`]).
fn new_mark() -> String {
    format!("PNM{}", uuid::Uuid::new_v4().simple())
}

/// Remove rustc diagnostics whose primary location is in a server file
/// (anything but `src/main.rs`). Returns the filtered text and whether
/// anything was removed. Text before the first diagnostic (cargo's
/// `Compiling ...` lines) and diagnostics without a location are kept.
pub fn filter_harness_diagnostics(stderr: &str) -> (String, bool) {
    let mut blocks: Vec<Vec<&str>> = vec![Vec::new()];
    for line in stderr.lines() {
        let starts_block = line.starts_with("error") || line.starts_with("warning");
        if starts_block {
            blocks.push(Vec::new());
        }
        if let Some(b) = blocks.last_mut() {
            b.push(line);
        }
    }
    let mut removed = false;
    let mut kept: Vec<&str> = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 && block_in_server_file(block) {
            removed = true;
            continue;
        }
        kept.extend(block.iter().copied());
    }
    (kept.join("\n"), removed)
}

fn block_in_server_file(block: &[&str]) -> bool {
    let Some(loc) = block
        .iter()
        .find_map(|l| l.trim_start().strip_prefix("--> "))
    else {
        return false;
    };
    let path = loc.replace('\\', "/");
    let file = path.rsplitn(3, ':').nth(2).unwrap_or(&path).to_string();
    !(file == "src/main.rs" || file.ends_with("/src/main.rs"))
}

/// Remove comments and string/char literal contents, so static checks
/// only see code. Lifetimes are kept. Unterminated constructs consume the
/// rest of the input (the build fails on such source anyway).
pub fn strip_comments_and_strings(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        let next = c.get(i + 1).copied();
        if ch == '/' && next == Some('/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '/' && next == Some('*') {
            let mut depth = 1;
            i += 2;
            while i < c.len() && depth > 0 {
                if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            out.push(' ');
        } else if ch == 'r' && (next == Some('"') || next == Some('#')) && !prev_is_ident(&c, i) {
            let mut j = i + 1;
            let mut hashes = 0;
            while c.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if c.get(j) != Some(&'"') {
                out.push(ch);
                i += 1;
                continue;
            }
            j += 1;
            loop {
                if j >= c.len() {
                    break;
                }
                if c[j] == '"' && (0..hashes).all(|k| c.get(j + 1 + k) == Some(&'#')) {
                    j += 1 + hashes;
                    break;
                }
                j += 1;
            }
            out.push_str("\"\"");
            i = j;
        } else if ch == '"' {
            i += 1;
            while i < c.len() && c[i] != '"' {
                if c[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            out.push_str("\"\"");
        } else if ch == '\'' {
            // Char literal ('x', '\n', '\u{..}') vs lifetime ('a).
            if next == Some('\\') {
                let mut j = i + 2;
                while j < c.len() && c[j] != '\'' {
                    j += 1;
                }
                i = j + 1;
                out.push_str("' '");
            } else if c.get(i + 2) == Some(&'\'') {
                i += 3;
                out.push_str("' '");
            } else {
                out.push(ch);
                i += 1;
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

fn prev_is_ident(c: &[char], i: usize) -> bool {
    i > 0 && (c[i - 1].is_alphanumeric() || c[i - 1] == '_')
}

/// Static token check: every group needs one token present in `code`.
/// Returns the first unmet group.
pub fn static_check(
    source: &str,
    required: &'static [&'static [&'static str]],
) -> Result<(), &'static [&'static str]> {
    let code = strip_comments_and_strings(source);
    for group in required {
        if !group.iter().any(|tok| code.contains(tok)) {
            return Err(group);
        }
    }
    Ok(())
}

/// Grade `source` for `encounter_id` with the dev [`BuildSettings`].
pub fn grade(encounter_id: &str, source: &str, limits: &RunLimits) -> anyhow::Result<GradedRun> {
    grade_in(encounter_id, source, limits, &BuildSettings::default())
}

/// The server's grading path. Blocking (builds and runs).
pub fn grade_in(
    encounter_id: &str,
    source: &str,
    limits: &RunLimits,
    settings: &BuildSettings,
) -> anyhow::Result<GradedRun> {
    let Some(m) = lookup(encounter_id) else {
        let outcome = wasm_builder::compile_and_run_in(source, limits, settings)?;
        let graded = GradedRun::unchecked(
            outcome,
            format!(
                "no output check exists for mission `{encounter_id}`; \
                 ok means it compiled and ran cleanly"
            ),
        );
        tracing::info!(
            encounter = encounter_id,
            state = ?graded.state,
            checked = false,
            "expectations: grading decision"
        );
        return Ok(graded);
    };
    grade_check(m, source, limits, settings)
}

/// Grade `source` against the check `m` (what [`grade_in`] does for a
/// registered id). Public so test fixtures (e.g. the Act 9
/// `UnsafeLateNonce` attack proofs and documented known limits) run
/// the exact production path.
pub fn grade_check(
    m: &MissionCheck,
    source: &str,
    limits: &RunLimits,
    settings: &BuildSettings,
) -> anyhow::Result<GradedRun> {
    if matches!(m.check, Check::Harness(_)) && source.contains(RESERVED_PREFIX) {
        tracing::warn!(
            encounter = m.id,
            "expectations: refused reserved identifier"
        );
        return Ok(refused_reserved());
    }
    let graded = match m.check {
        Check::RunOnly { reason } => {
            let outcome = wasm_builder::compile_and_run_in(source, limits, settings)?;
            GradedRun::unchecked(outcome, format!("output not checked: {reason}"))
        }
        Check::Stdout { matchers } => grade_stdout(source, matchers, limits, settings)?,
        Check::Harness(h) => grade_harness(source, &h, m.harness_mode, limits, settings)?,
        Check::CompileOnly { reason, required } => {
            grade_compile_only(source, reason, required, limits, settings)?
        }
    };
    let graded = if graded.state == GradeState::Passed && !m.uses.is_empty() {
        apply_uses(source, graded, m.uses)
    } else {
        graded
    };
    let late =
        matches!(m.check, Check::Harness(_)) && m.harness_mode == HarnessMode::UnsafeLateNonce;
    let graded = if graded.state == GradeState::Passed && late {
        apply_forbids(source, graded, UNSAFE_HARNESS_FORBIDS)
    } else {
        graded
    };
    let graded = if graded.state == GradeState::Passed && !m.forbids.is_empty() {
        apply_forbids(source, graded, m.forbids)
    } else {
        graded
    };
    tracing::info!(
        encounter = m.id,
        state = ?graded.state,
        checked = graded.state.behaviour_checked(),
        late_nonce = late,
        "expectations: grading decision"
    );
    Ok(graded)
}

fn refused_reserved() -> GradedRun {
    GradedRun {
        outcome: RealOutcome {
            compiled: false,
            ran_ok: false,
            stdout: String::new(),
            stderr: format!(
                "names containing `{RESERVED_PREFIX}` are reserved by the mission checker; \
                 rename that item"
            ),
            build_time: std::time::Duration::ZERO,
            run_time: None,
        },
        state: GradeState::CompileError,
        expected: None,
        actual: None,
        note: "refused before building".to_string(),
    }
}

fn grade_stdout(
    source: &str,
    matchers: &'static [Matcher],
    limits: &RunLimits,
    settings: &BuildSettings,
) -> anyhow::Result<GradedRun> {
    let outcome = wasm_builder::compile_and_run_in(source, limits, settings)?;
    if !outcome.ok() {
        return Ok(GradedRun::unchecked(
            outcome,
            "did not build and run cleanly",
        ));
    }
    Ok(match check_stdout(matchers, &outcome.stdout) {
        Ok(()) => {
            tracing::debug!("expectations: stdout check passed");
            GradedRun {
                outcome,
                state: GradeState::Passed,
                expected: None,
                actual: None,
                note: "output checked against the mission".to_string(),
            }
        }
        Err(f) => wrong_output(outcome, f),
    })
}

fn wrong_output(mut outcome: RealOutcome, f: MatchFailure) -> GradedRun {
    let mut msg = format!(
        "Your program ran, but did not do what this mission asks.\nExpected {}\nGot:\n{}",
        f.expected, f.actual
    );
    if !outcome.stderr.is_empty() {
        msg.push_str("\n--- your program's stderr ---\n");
        msg.push_str(&quote_for_player(&outcome.stderr));
    }
    outcome.stderr = msg;
    GradedRun {
        outcome,
        state: GradeState::WrongOutput,
        expected: Some(f.expected),
        actual: Some(f.actual),
        note: "output checked against the mission".to_string(),
    }
}

fn grade_harness(
    source: &str,
    h: &Harness,
    mode: HarnessMode,
    limits: &RunLimits,
    settings: &BuildSettings,
) -> anyhow::Result<GradedRun> {
    // ForbidUnsafe: the nonce is compiled into the checker. UnsafeLateNonce:
    // only the import name and the non-secret main mark are; the nonce
    // itself goes to the run (never the build) and the checker fetches it
    // after its last call into player code.
    let (checker, nonce, mark, late) = match mode {
        HarnessMode::ForbidUnsafe => {
            let nonce = new_nonce();
            (harness_module_source(h, &nonce), nonce.clone(), nonce, None)
        }
        HarnessMode::UnsafeLateNonce => {
            let late = LateNonce::generate();
            let mark = new_mark();
            let checker = harness_module_source_late(h, &mark, late.import());
            (checker, late.nonce().to_string(), mark, Some(late))
        }
    };
    tracing::debug!(?mode, "expectations: building harness");
    let layout = Layout::Harness {
        entry: harness_entry_source(mode),
        checker: &checker,
    };
    let mut outcome = wasm_builder::compile_and_run_layout_with(
        source,
        &layout,
        true,
        limits,
        settings,
        late.as_ref(),
    )?;
    let transcript = parse_transcript_tagged(&outcome.stdout, &nonce, &mark);
    // Never hand checker lines (or the nonce) to the player.
    outcome.stdout = transcript.program.clone();

    if !outcome.compiled {
        if mode == HarnessMode::ForbidUnsafe && refused_for_unsafe(&outcome.stderr) {
            tracing::info!("expectations: harness build refused: player code uses `unsafe`");
            let (filtered, _) = filter_harness_diagnostics(&outcome.stderr);
            outcome.stderr = explain_unsafe_refusal(&filtered);
            let mut g = GradedRun::unchecked(outcome, "did not build: `unsafe` is not allowed");
            g.expected = Some(h.contract.to_string());
            return Ok(g);
        }
        let (filtered, removed) = filter_harness_diagnostics(&outcome.stderr);
        if removed {
            tracing::debug!("expectations: checker could not compile against player code");
            outcome.stderr = format!(
                "error: the mission checker could not use your code. It needs: {}\n{}",
                h.contract, filtered
            );
        } else {
            outcome.stderr = filtered;
        }
        let mut g = GradedRun::unchecked(outcome, "did not build");
        g.expected = Some(h.contract.to_string());
        return Ok(g);
    }
    if !outcome.ran_ok {
        return Ok(GradedRun::unchecked(outcome, "crashed while being checked"));
    }
    if !transcript.finished {
        let f = MatchFailure {
            expected: "the checker to finish calling your code".to_string(),
            actual: "your program exited before the checker finished \
                     (did it call std::process::exit?)"
                .to_string(),
        };
        return Ok(wrong_output(outcome, f));
    }
    if transcript.after_done {
        tracing::warn!(
            ?mode,
            "expectations: checker-tagged output after the done mark; not a pass"
        );
        let f = MatchFailure {
            expected: "nothing from the checker after it finished".to_string(),
            actual: "checker output appeared after the checker had finished".to_string(),
        };
        return Ok(wrong_output(outcome, f));
    }
    if h.run_main {
        let main_out = transcript.main_output.as_deref().unwrap_or_default();
        if let Err(f) = check_stdout(h.main_matchers, main_out) {
            return Ok(wrong_output(outcome, f));
        }
    }
    if let Err(f) = compare_results(h.expected, &transcript.results) {
        return Ok(wrong_output(outcome, f));
    }
    tracing::debug!(
        results = transcript.results.len(),
        "expectations: checker results matched"
    );
    Ok(GradedRun {
        outcome,
        state: GradeState::Passed,
        expected: None,
        actual: None,
        note: harness_pass_note(mode, h.expected.len()),
    })
}

/// The note on a passing harness run. In [`HarnessMode::UnsafeLateNonce`]
/// it says `unsafe` was allowed; the game client keys on those words
/// (`UNSAFE_ALLOWED_NOTE_MARKER`) to label the pass NOT VERIFIED, because
/// in that mode player code can rewrite the checker's results
/// (HARNESS-NOTES.md, "What it cannot stop"). Unchanged wording.
pub fn harness_pass_note(mode: HarnessMode, checks: usize) -> String {
    match mode {
        HarnessMode::ForbidUnsafe => format!("checked by calling your code ({checks} checks)"),
        HarnessMode::UnsafeLateNonce => format!(
            "checked by calling your code ({checks} checks; `unsafe` allowed, results keyed \
             by the server only after your code finished)"
        ),
    }
}

fn grade_compile_only(
    source: &str,
    reason: &str,
    required: &'static [&'static [&'static str]],
    limits: &RunLimits,
    settings: &BuildSettings,
) -> anyhow::Result<GradedRun> {
    let outcome =
        wasm_builder::compile_and_run_layout(source, &Layout::Plain, false, limits, settings)?;
    if !outcome.compiled {
        return Ok(GradedRun::unchecked(outcome, "did not build"));
    }
    let note = format!("behaviour not checked on this server: {reason}");
    match static_check(source, required) {
        Ok(()) => Ok(GradedRun {
            outcome,
            state: GradeState::CompiledNotChecked,
            expected: None,
            actual: None,
            note,
        }),
        Err(group) => Ok(static_check_failed(outcome, group, note)),
    }
}

/// Apply a passing run's [`MissionCheck::uses`]: keep the pass when every
/// group is present, otherwise downgrade to `StaticCheckFailed`.
fn apply_uses(
    source: &str,
    graded: GradedRun,
    uses: &'static [&'static [&'static str]],
) -> GradedRun {
    match static_check(source, uses) {
        Ok(()) => {
            tracing::debug!("expectations: construct tokens present");
            graded
        }
        Err(group) => {
            tracing::debug!(
                ?group,
                "expectations: output matched but construct tokens missing"
            );
            static_check_failed(
                graded.outcome,
                group,
                "output matched, but the code does not use the mission's construct".to_string(),
            )
        }
    }
}

/// The first token of `forbids` present in the code of `source` (comments
/// and string literals removed). An edge of a token that is an identifier
/// character must sit next to a non-identifier character (or the end of
/// the code), so `match` is found in `match x {` but not in `matches!`
/// or `rematch`.
pub fn forbidden_token(source: &str, forbids: &'static [&'static str]) -> Option<&'static str> {
    let code = strip_comments_and_strings(source);
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    forbids.iter().copied().find(|tok| {
        let starts_ident = tok.chars().next().is_some_and(ident);
        let ends_ident = tok.chars().last().is_some_and(ident);
        code.match_indices(tok).any(|(at, _)| {
            let before_ok = !starts_ident || !code[..at].chars().next_back().is_some_and(ident);
            let after_ok = !ends_ident || !code[at + tok.len()..].chars().next().is_some_and(ident);
            before_ok && after_ok
        })
    })
}

fn apply_forbids(source: &str, graded: GradedRun, forbids: &'static [&'static str]) -> GradedRun {
    let Some(tok) = forbidden_token(source, forbids) else {
        tracing::debug!("expectations: no forbidden tokens");
        return graded;
    };
    tracing::debug!(
        tok,
        "expectations: output matched but a forbidden token is used"
    );
    let mut outcome = graded.outcome;
    let banned = forbids
        .iter()
        .map(|t| format!("`{t}`"))
        .collect::<Vec<_>>()
        .join(", ");
    outcome.stderr = format!(
        "Your code works, but this mission rules out {banned}, and it uses `{tok}` \
         (comments and strings do not count)."
    );
    GradedRun {
        outcome,
        state: GradeState::StaticCheckFailed,
        expected: Some(format!("no {banned} in your code")),
        actual: None,
        note: format!("output matched, but the code uses `{tok}`, which this mission rules out"),
    }
}

fn static_check_failed(
    mut outcome: RealOutcome,
    group: &'static [&'static str],
    note: String,
) -> GradedRun {
    let want = group
        .iter()
        .map(|t| format!("`{t}`"))
        .collect::<Vec<_>>()
        .join(" or ");
    outcome.stderr = format!(
        "Your code compiles, but it does not use what this mission teaches: \
         expected {want} in your code (comments and strings do not count)."
    );
    GradedRun {
        outcome,
        state: GradeState::StaticCheckFailed,
        expected: Some(want),
        actual: None,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_entry_forbids_unsafe_without_touching_the_player_file() {
        // The lint is a crate-level inner attribute of the SERVER entry...
        let attrs: Vec<&str> = HARNESS_ENTRY_SOURCE
            .lines()
            .filter(|l| l.starts_with("#!["))
            .collect();
        assert_eq!(attrs, ["#![forbid(unsafe_code)]"]);
        // ...placed before any item (inner attributes must be).
        let first_item = HARNESS_ENTRY_SOURCE
            .lines()
            .position(|l| !l.starts_with("//") && !l.starts_with("#!["))
            .unwrap();
        let forbid = HARNESS_ENTRY_SOURCE
            .lines()
            .position(|l| l == "#![forbid(unsafe_code)]")
            .unwrap();
        assert!(forbid < first_item);
        // The player's file still gets exactly the one-line preamble.
        assert_eq!(PLAYER_PREAMBLE_LINES, 1);
        assert_eq!(wasm_builder::PLAYER_PREAMBLE.lines().count(), 1);
        assert!(!wasm_builder::PLAYER_PREAMBLE.contains("unsafe"));
    }

    /// The Act 9 harness missions that opt into `UnsafeLateNonce` (review
    /// finding P2-3 of `.claude/reviews/security-2026-11-15-cf9bc47.md`).
    /// Adding a mission here is a security-reviewed change.
    const LATE_NONCE_ALLOW_LIST: &[&str] =
        &["extern_c", "send_sync", "librarian_index", "librarian_seal"];

    /// Exactly the allow-listed missions build without the `forbid`; every
    /// other registered mission keeps `ForbidUnsafe`, so any mission gaining
    /// the mode fails here. The mode is a field of the server's table,
    /// picked by mission id: nothing in a player's source reaches
    /// `harness_entry_source`.
    #[test]
    fn only_the_act9_allow_list_uses_unsafe_late_nonce() {
        let mut opted = Vec::new();
        for m in all() {
            if LATE_NONCE_ALLOW_LIST.contains(&m.id) {
                assert_eq!(m.harness_mode, HarnessMode::UnsafeLateNonce, "{}", m.id);
                assert!(
                    matches!(m.check, Check::Harness(_)),
                    "{}: the mode only means something for a harness check",
                    m.id
                );
                assert!(
                    !harness_entry_source(m.harness_mode).contains("forbid"),
                    "{}",
                    m.id
                );
                opted.push(m.id);
            } else {
                assert_eq!(m.harness_mode, HarnessMode::ForbidUnsafe, "{}", m.id);
                assert!(
                    harness_entry_source(m.harness_mode).contains("#![forbid(unsafe_code)]"),
                    "{}",
                    m.id
                );
            }
        }
        assert_eq!(
            opted, LATE_NONCE_ALLOW_LIST,
            "allow-list ids are registered, in order"
        );
        assert_eq!(
            harness_entry_source(HarnessMode::ForbidUnsafe),
            HARNESS_ENTRY_SOURCE
        );
    }

    /// The pass note of each mode. The game keys on "`unsafe` allowed" to
    /// label a late-nonce pass NOT VERIFIED.
    #[test]
    fn harness_pass_notes_name_the_mode() {
        assert_eq!(
            harness_pass_note(HarnessMode::ForbidUnsafe, 3),
            "checked by calling your code (3 checks)"
        );
        let late = harness_pass_note(HarnessMode::UnsafeLateNonce, 8);
        assert!(late.starts_with("checked by calling your code (8 checks; "));
        assert!(late.contains("`unsafe` allowed"), "{late}");
        assert!(!harness_pass_note(HarnessMode::ForbidUnsafe, 8).contains("unsafe"));
    }

    #[test]
    fn unsafe_allowed_entry_differs_only_by_the_forbid() {
        let late = harness_entry_source(HarnessMode::UnsafeLateNonce);
        assert!(!late.contains("forbid"), "{late}");
        assert!(!late.lines().any(|l| l.starts_with("#![")), "{late}");
        // Same items, same line count (the forbid line became a comment).
        let code = |s: &str| -> Vec<String> {
            s.lines()
                .filter(|l| !l.starts_with("//") && !l.starts_with("#!["))
                .map(str::to_string)
                .collect()
        };
        assert_eq!(code(late), code(HARNESS_ENTRY_SOURCE));
        assert_eq!(late.lines().count(), HARNESS_ENTRY_SOURCE.lines().count());
    }

    fn looks_like_nonce(s: &str) -> bool {
        s.match_indices("PNC").any(|(at, _)| {
            let rest = &s[at + 3..];
            rest.len() >= 32
                && rest[..32]
                    .bytes()
                    .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        })
    }

    #[test]
    fn late_checker_holds_no_nonce_and_fetches_it_after_the_calls() {
        let h = Harness {
            contract: "fn f()",
            run_main: true,
            body: "    emit(\"f(1)\", super::f(1));",
            expected: &["f(1) = 2"],
            main_matchers: &[],
        };
        let late = LateNonce::generate();
        let src = harness_module_source_late(&h, "PNMmark", late.import());
        assert!(!looks_like_nonce(&src), "{src}");
        assert!(!src.contains(late.nonce()));
        assert!(src.contains(&format!("#[link_name = \"{}\"]", late.import())));
        assert!(src.contains("#[link(wasm_import_module = \"pnc_harness\")]"));
        assert!(src.contains("const MARK: &str = \"PNMmark\";"));
        // Order: player calls, then finish; inside finish the hook reset
        // and the stored results come before the key, the key before the
        // print.
        let at = |needle: &str| {
            src.find(needle)
                .unwrap_or_else(|| panic!("missing {needle}"))
        };
        assert!(at("fn calls()") < at("super::main()"));
        assert!(at("super::main()") < at("emit(\"f(1)\""));
        assert!(at("emit(\"f(1)\"") < at("fn finish()"));
        assert!(at("fn finish()") < at("take_hook()"));
        assert!(at("take_hook()") < at("::core::mem::take(&mut *r)"));
        assert!(at("::core::mem::take(&mut *r)") < at("take_key(key.as_mut_ptr()"));
        assert!(at("take_key(key.as_mut_ptr()") < at("::std::print!"));
        assert!(src.contains("pub(crate) fn run() {\n    calls();\n    finish();\n}"));
        // Exactly one unsafe block: the FFI call.
        assert_eq!(src.matches("unsafe {").count(), 1, "{src}");
        // The sealed checker, by contrast, does compile the nonce in.
        assert!(looks_like_nonce(&harness_module_source(&h, late.nonce())));
    }

    #[test]
    fn late_transcript_uses_the_mark_for_main_and_rejects_lines_after_done() {
        let raw = "M <main>\nhello\nM </main>\nside\nN f(1) = 2\nN <done>\n";
        let t = parse_transcript_tagged(raw, "N", "M");
        assert_eq!(t.main_output.as_deref(), Some("hello"));
        assert_eq!(t.program, "hello\nside");
        assert_eq!(t.results, vec!["f(1) = 2".to_string()]);
        assert!(t.finished && !t.after_done);
        // A main mark carrying the nonce is not a main mark in late mode,
        // and a result carrying the mark is just program output.
        let t = parse_transcript_tagged("N <main>\nM f(1) = 2\nN <done>\n", "N", "M");
        assert_eq!(t.results, vec!["<main>".to_string()]);
        assert!(t.main_output.is_none());
        assert_eq!(t.program, "M f(1) = 2");
        // Tagged lines after the done mark (or a second done) are flagged
        // and never counted.
        let t = parse_transcript_tagged("N f = 1\nN <done>\nN f = 2\nN <done>\n", "N", "M");
        assert!(t.finished && t.after_done);
        assert_eq!(t.results, vec!["f = 1".to_string()]);
        // ForbidUnsafe parsing is unchanged (mark == nonce).
        let t = parse_transcript("N <main>\nx\nN </main>\nN a = 1\nN <done>\n", "N");
        assert_eq!(t.main_output.as_deref(), Some("x"));
        assert_eq!(t.results, vec!["a = 1".to_string()]);
        assert!(!t.after_done);
    }

    #[test]
    fn unsafe_harness_forbids_catch_late_print_hooks() {
        assert_eq!(
            forbidden_token(
                "#[global_allocator]\nstatic A: Spy = Spy;",
                UNSAFE_HARNESS_FORBIDS
            ),
            Some("global_allocator")
        );
        assert_eq!(
            forbidden_token(
                "#[unsafe(export_name = \"memcpy\")]\nextern \"C\" fn m() {}",
                UNSAFE_HARNESS_FORBIDS
            ),
            Some("export_name")
        );
        assert_eq!(
            forbidden_token(
                "// global_allocator export_name\nfn main() {}",
                UNSAFE_HARNESS_FORBIDS
            ),
            None
        );
    }

    #[test]
    fn unsafe_refusal_is_detected_and_explained_with_player_lines_intact() {
        // rustc 1.99's shape for an `unsafe` block under the entry's forbid.
        let stderr = "error: usage of an `unsafe` block\n \
                      --> src/main.rs:4:5\n  \
                      |\n\
                      4 |     unsafe { core::ptr::read_volatile(p) }\n  \
                      |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n  \
                      |\n\
                      note: the lint level is defined here\n \
                      --> src/pnc_entry.rs:3:11\n  \
                      |\n\
                      3 | #![forbid(unsafe_code)]\n  \
                      |           ^^^^^^^^^^^\n\n\
                      error: could not compile `player_submission` (bin \"player_submission\") due to 1 previous error";
        assert!(refused_for_unsafe(stderr));
        let shown = explain_unsafe_refusal(stderr);
        assert!(shown.starts_with(UNSAFE_REFUSED_MESSAGE), "{shown}");
        assert!(
            shown.contains("--> src/main.rs:4:5"),
            "player location kept: {shown}"
        );
        assert!(!shown.contains("pnc_entry"), "server file hidden: {shown}");
        assert!(!shown.contains("forbid(unsafe_code)"), "{shown}");
        // An ordinary type error is not an unsafe refusal.
        assert!(!refused_for_unsafe(
            "error[E0308]: mismatched types\n --> src/main.rs:3:9\n"
        ));
    }

    #[test]
    fn normalize_unifies_line_endings_and_trailing_space() {
        assert_eq!(normalize("a  \r\nb\t\r\n\r\n\n"), "a\nb");
        assert_eq!(normalize("a\rb"), "a\nb");
        assert_eq!(normalize("  lead\n\nmid\n"), "  lead\n\nmid");
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("\n\n"), "");
    }

    #[test]
    fn matchers_behave() {
        let out = normalize("0: 10\n1: 20\nfire\n");
        assert!(Matcher::Exact("0: 10\n1: 20\nfire").matches(&out));
        assert!(!Matcher::Exact("0: 10").matches(&out));
        assert!(Matcher::HasLine("fire").matches(&out));
        assert!(!Matcher::HasLine("fir").matches(&out));
        assert!(Matcher::LineContains("fir").matches(&out));
        assert!(Matcher::LineContainsAll(&["1", "20"]).matches(&out));
        assert!(!Matcher::LineContainsAll(&["20", "1"]).matches(&out));
        assert!(Matcher::InOrder(&["0: 10", "fire"]).matches(&out));
        assert!(!Matcher::InOrder(&["fire", "0: 10"]).matches(&out));
        assert!(Matcher::StartsWith("0: 10\n1").matches(&out));
        assert!(Matcher::EndsWith("20\nfire").matches(&out));
        assert!(Matcher::LastLine("fire").matches(&out));
        assert!(!Matcher::LastLine("1: 20").matches(&out));
        assert!(!Matcher::LastLine("x").matches(""));
    }

    #[test]
    fn check_stdout_reports_first_failure() {
        let err = check_stdout(&[Matcher::HasLine("42")], "41\n").unwrap_err();
        assert_eq!(err.expected, "a line reading `42`");
        assert_eq!(err.actual, "41");
        assert_eq!(
            check_stdout(&[Matcher::HasLine("42")], "")
                .unwrap_err()
                .actual,
            "(nothing)"
        );
        assert!(check_stdout(&[Matcher::HasLine("42")], "42\r\n").is_ok());
    }

    #[test]
    fn quote_truncates_long_output() {
        let long = (0..100)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let q = quote_for_player(&long);
        assert!(q.lines().count() <= MAX_QUOTED_LINES + 1, "{q}");
        assert!(q.ends_with("(truncated)"));
        let wide = "x".repeat(5000);
        let q = quote_for_player(&wide);
        assert!(q.len() < MAX_QUOTED_CHARS + 32);
    }

    #[test]
    fn transcript_separates_checker_lines() {
        let raw = "N <main>\nhello\nN </main>\nside effect\nN f(1) = 2\nN <done>\n";
        let t = parse_transcript(raw, "N");
        assert_eq!(t.program, "hello\nside effect");
        assert_eq!(t.main_output.as_deref(), Some("hello"));
        assert_eq!(t.results, vec!["f(1) = 2".to_string()]);
        assert!(t.finished);
        // A forged line without the nonce is just program output.
        let t = parse_transcript("f(1) = 2\nX f(1) = 2\n", "N");
        assert!(t.results.is_empty());
        assert!(!t.finished);
    }

    #[test]
    fn compare_results_explains_mismatches() {
        let got = vec!["sign(0) = \"pos\"".to_string()];
        let err = compare_results(&["sign(0) = \"zero\""], &got).unwrap_err();
        assert_eq!(err.expected, "sign(0) should give \"zero\"");
        assert_eq!(err.actual, "sign(0) gave \"pos\"");
        assert!(compare_results(&["a = 1"], &["a = 1".to_string()]).is_ok());
        let err = compare_results(&["a = 1", "b = 2"], &["a = 1".to_string()]).unwrap_err();
        assert!(err.actual.contains("never got that far"));
    }

    #[test]
    fn harness_diagnostics_are_filtered() {
        let stderr = "   Compiling player_submission v0.0.0 (/build)\n\
error[E0425]: cannot find function `sign` in module `super`\n \
--> src/main/__pnc_harness.rs:9:12\n  |\n9 | x\n\n\
error[E0308]: mismatched types\n --> src/main.rs:4:5\n  |\n4 | y\n\n\
error: could not compile `player_submission` (bin \"player_submission\") due to 2 previous errors";
        let (out, removed) = filter_harness_diagnostics(stderr);
        assert!(removed);
        assert!(!out.contains("__pnc_harness"), "{out}");
        assert!(out.contains("src/main.rs:4:5"));
        assert!(out.contains("Compiling"));
        assert!(out.contains("could not compile"));
        let (out, removed) =
            filter_harness_diagnostics("error[E0308]: x\n --> /build/src/pnc_entry.rs:6:5\n");
        assert!(removed && out.is_empty(), "{out}");
        let (_, removed) =
            filter_harness_diagnostics("error[E0308]: x\n --> /build/src/main.rs:6:5\n");
        assert!(!removed);
    }

    #[test]
    fn strip_removes_comments_and_strings_only() {
        let s = strip_comments_and_strings(
            "// thread::spawn\nfn f<'a>(x: &'a str) { let c = 'x'; let s = \"spawn(\"; \
             /* .join( /* nested */ */ let r = r#\"x\"#; g.join(); }",
        );
        assert!(!s.contains("thread::spawn"));
        assert!(!s.contains("spawn("));
        assert!(s.contains("g.join()"));
        assert!(s.contains("'a"));
        assert!(!s.contains("nested"));
    }

    #[test]
    fn static_check_ignores_comments() {
        const REQ: &[&[&str]] = &[&["spawn("], &[".join("]];
        assert!(static_check("fn main() { let h = spawn(|| 1); h.join(); }", REQ).is_ok());
        assert_eq!(
            static_check("// spawn( .join(\nfn main() {}", REQ),
            Err(&["spawn("][..])
        );
    }

    #[test]
    fn grade_states_map_to_ok() {
        assert!(GradeState::Passed.ok());
        assert!(GradeState::CompiledNotChecked.ok());
        assert!(GradeState::RanNotChecked.ok());
        assert!(!GradeState::WrongOutput.ok());
        assert!(!GradeState::StaticCheckFailed.ok());
        assert!(!GradeState::CompiledNotChecked.behaviour_checked());
        assert!(GradeState::Passed.behaviour_checked());
    }

    #[test]
    fn every_check_is_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for m in all() {
            assert!(seen.insert(m.id), "duplicate id {}", m.id);
            assert!(crate::valid_encounter_id(m.id), "{}", m.id);
            if !m.uses.is_empty() {
                assert!(
                    matches!(m.check, Check::Stdout { .. } | Check::Harness(_)),
                    "{}: `uses` only applies to behaviour checks",
                    m.id
                );
                assert!(m.uses.iter().all(|g| !g.is_empty()), "{}", m.id);
            }
            if !m.forbids.is_empty() {
                assert!(
                    matches!(m.check, Check::Stdout { .. } | Check::Harness(_)),
                    "{}: `forbids` only applies to behaviour checks",
                    m.id
                );
                assert!(m.forbids.iter().all(|t| !t.trim().is_empty()), "{}", m.id);
            }
            match m.check {
                Check::Stdout { matchers } => assert!(!matchers.is_empty(), "{}", m.id),
                Check::Harness(h) => {
                    assert!(!h.expected.is_empty(), "{}: no recorded results", m.id);
                    assert!(!h.contract.is_empty());
                    assert!(h.run_main || h.main_matchers.is_empty(), "{}", m.id);
                    assert!(!h.body.contains(RESERVED_PREFIX) || h.body.contains("emit"));
                    for e in h.expected {
                        assert!(e.contains(" = "), "{}: {e}", m.id);
                    }
                }
                Check::RunOnly { reason } => assert!(!reason.is_empty()),
                Check::CompileOnly { reason, required } => {
                    assert!(!reason.is_empty() && !required.is_empty())
                }
            }
        }
        assert_eq!(all().len(), 96, "one entry per registry mission");
    }

    #[test]
    fn forbidden_tokens_match_whole_words_in_code_only() {
        const BANNED: &[&str] = &["match", ".unwrap(", ".expect(", "panic!"];
        let found = |src: &str| forbidden_token(src, BANNED);
        assert_eq!(found("fn f() { match x { _ => 1 } }"), Some("match"));
        assert_eq!(
            found("fn f() { let n = s.parse::<u32>().unwrap(); }"),
            Some(".unwrap(")
        );
        assert_eq!(found("fn f() { o.expect(\"x\"); }"), Some(".expect("));
        assert_eq!(found("fn f() { panic!(\"no\"); }"), Some("panic!"));
        // Whole words only, and never inside comments or strings.
        assert_eq!(
            found("fn f() { assert!(matches!(x, Some(_))); let rematch = 1; }"),
            None
        );
        assert_eq!(
            found("fn f() { o.unwrap_or(0); o.unwrap_or_else(|| 1); }"),
            None
        );
        assert_eq!(
            found("// match x.unwrap() panic!\nfn f() { let s = \"match\"; }"),
            None
        );
        assert_eq!(found("fn f() { let x = r#\"panic!\"#; }"), None);
        // Every passing wager_quest check uses `?` and none of the banned tokens.
        let quest = lookup("wager_quest").expect("wager_quest check");
        assert_eq!(quest.forbids, BANNED);
        assert_eq!(quest.uses, &[&["?"][..]]);
    }

    /// The Keymaster's last stage rules out loops and `mut` as whole
    /// words only: `format!`, `mutate` and `for_each` are other words.
    #[test]
    fn golem_core_forbids_loops_and_mut_as_whole_words() {
        let core = lookup("golem_core").expect("golem_core check");
        assert_eq!(core.forbids, &["for", "while", "loop", "mut"]);
        assert_eq!(core.uses, &[&[".fold(", ".sum"][..]]);
        let found = |src: &str| forbidden_token(src, core.forbids);
        assert_eq!(found("fn f() { for p in ps {} }"), Some("for"));
        assert_eq!(found("fn f() { while go() {} }"), Some("while"));
        assert_eq!(found("fn f() { loop { break; } }"), Some("loop"));
        assert_eq!(found("fn f() { let mut t = 0; }"), Some("mut"));
        assert_eq!(
            found("fn f() { let s = format!(\"for\"); mutate(); v.iter().for_each(g); }"),
            None
        );
    }

    /// The headline numbers in `docs/mission-audit.md`. Change both together.
    #[test]
    fn classification_totals() {
        let count = |f: fn(&Check) -> bool| all().iter().filter(|m| f(&m.check)).count();
        let behaviour = count(|c| matches!(c, Check::Stdout { .. } | Check::Harness(_)));
        let static_only = count(|c| matches!(c, Check::CompileOnly { .. }));
        let unverified = count(|c| matches!(c, Check::RunOnly { .. }));
        assert_eq!(
            (behaviour, static_only, unverified),
            (92, 4, 0),
            "behaviour-checked / static-only / run-only"
        );
    }

    #[test]
    fn harness_source_shapes() {
        let h = Harness {
            contract: "fn f()",
            run_main: true,
            body: "    emit(\"f\", 1);",
            expected: &["f = 1"],
            main_matchers: &[],
        };
        let src = harness_module_source(&h, "NONCE1");
        assert!(src.contains("const NONCE: &str = \"NONCE1\";"));
        assert!(src.contains("super::main()"));
        assert!(src.contains("emit(\"f\", 1);"));
        assert!(src.contains("mark(\"done\")"));
        let h2 = Harness {
            run_main: false,
            ..h
        };
        assert!(!harness_module_source(&h2, "N").contains("super::main"));
    }
}
