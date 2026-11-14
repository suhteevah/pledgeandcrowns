// SPDX-License-Identifier: MIT
//! Pure parsing of compile results for the editor's result pane and
//! error-line gutter.
//!
//! Two jobs:
//! 1. [`parse_result`] splits the string `compile_client` writes into
//!    `EditorState::last_compile_result` into a verdict (pass / fail /
//!    error / pending) plus the message body.
//! 2. [`error_lines`] pulls the primary `--> file:LINE:COL` location of
//!    every rustc *error* (not warning) diagnostic, mapped back to the
//!    player's own buffer lines.

/// Lines the compile-API inserts above the player's source before it
/// builds: both `compile-api/src/cargo_grader.rs` and
/// `compile-api/src/wasm_builder.rs` prepend
/// `#![allow(dead_code, unused)]\n`. rustc reports `src/main.rs:4` for
/// what the player sees as line 3. If that preamble changes, change this.
pub const SERVER_PREAMBLE_LINES: usize = 1;

/// Verdict class of one compile result string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultKind {
    /// Grader accepted the submission (`[ok] ...`).
    Pass,
    /// Grader or compiler rejected the submission (`[fail] ...`). This is
    /// the only kind that counts toward the show-solution threshold.
    Fail,
    /// Request in flight (the `compiling…` placeholder).
    Pending,
    /// Anything else: transport/client errors, missing grader. Not the
    /// player's fault, so it never counts as a failed attempt.
    Error,
}

/// A compile result split for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResult<'a> {
    pub kind: ResultKind,
    /// True when the native client fell back to the offline stub grader.
    pub offline: bool,
    /// Message with the `[ok]` / `[fail]` tag and offline banner removed.
    pub body: &'a str,
}

const OFFLINE_BANNER_PREFIX: &str = "[offline mode";
const PENDING_TEXT: &str = "compiling…";

/// Classify a `last_compile_result` string. Formats produced by
/// `compile_client.rs`: `[ok] ...`, `[fail] ...`, an optional leading
/// `[offline mode — server unreachable]` line, `[client error] ...`,
/// `[wasm] no stub grader ...`, and the `compiling…` placeholder.
pub fn parse_result(raw: &str) -> ParsedResult<'_> {
    let mut rest = raw;
    let mut offline = false;
    if rest.starts_with(OFFLINE_BANNER_PREFIX) {
        offline = true;
        rest = rest.split_once('\n').map_or("", |(_, tail)| tail);
    }
    let (kind, body) = if let Some(b) = rest.strip_prefix("[ok]") {
        (ResultKind::Pass, b)
    } else if let Some(b) = rest.strip_prefix("[fail]") {
        (ResultKind::Fail, b)
    } else if rest.trim() == PENDING_TEXT {
        (ResultKind::Pending, rest)
    } else {
        (ResultKind::Error, rest)
    };
    ParsedResult {
        kind,
        offline,
        body: body.trim_start_matches(' '),
    }
}

/// 1-based lines in the PLAYER's buffer that carry a rustc error.
///
/// Only the primary location (the first `-->` after an `error` header)
/// is taken; warnings, and `-->` lines belonging to trailing `note:` /
/// `help:` sub-diagnostics, are skipped. Locations are mapped back by
/// subtracting `preamble_lines`; anything that lands in the preamble is
/// dropped. Result is sorted and deduplicated.
pub fn error_lines(output: &str, preamble_lines: usize) -> Vec<usize> {
    #[derive(PartialEq)]
    enum Severity {
        None,
        Error,
        Warning,
    }
    let mut severity = Severity::None;
    let mut primary_pending = false;
    let mut lines = Vec::new();

    for raw in output.lines() {
        let line = raw.trim_end();
        if is_error_header(line) {
            severity = Severity::Error;
            primary_pending = true;
        } else if line.starts_with("warning") {
            severity = Severity::Warning;
            primary_pending = false;
        } else if line.starts_with("note:") || line.starts_with("help:") {
            // Top-level sub-diagnostic: its own `-->` is secondary.
            primary_pending = false;
        } else if let Some(loc) = line.trim_start().strip_prefix("--> ") {
            if primary_pending && severity == Severity::Error {
                if let Some(n) = parse_location_line(loc) {
                    if n > preamble_lines {
                        lines.push(n - preamble_lines);
                    }
                }
            }
            primary_pending = false;
        }
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

/// `error[E0384]: ...` or `error: ...`, but not cargo's trailing
/// `error: could not compile` / rustc's `error: aborting due to ...`
/// (those never carry a location; excluding them just keeps intent clear).
fn is_error_header(line: &str) -> bool {
    if line.starts_with("error[E") {
        return true;
    }
    line.strip_prefix("error:").is_some_and(|rest| {
        let rest = rest.trim_start();
        !rest.starts_with("could not compile") && !rest.starts_with("aborting due to")
    })
}

/// Parse `src/main.rs:12:5` → `Some(12)`. Only the player's crate root
/// (`main.rs` / `lib.rs`) counts; std-library or dependency paths are
/// ignored. Splits from the right so Windows drive letters survive.
fn parse_location_line(loc: &str) -> Option<usize> {
    let mut parts = loc.trim().rsplitn(3, ':');
    let _col: usize = parts.next()?.parse().ok()?;
    let line: usize = parts.next()?.parse().ok()?;
    let path = parts.next()?.replace('\\', "/");
    let file = path.rsplit('/').next().unwrap_or(&path);
    (file == "main.rs" || file == "lib.rs").then_some(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real rustc 1.8x output as captured from `cargo check`
    // (CARGO_TERM_COLOR=never), player source on the left of the gutter.
    const SINGLE_ERROR: &str = "    Checking player v0.1.0 (/sandbox)
error[E0384]: cannot assign twice to immutable variable `x`
 --> src/main.rs:4:5
  |
3 |     let x = 5;
  |         - first assignment to `x`
4 |     x = 6;
  |     ^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
3 |     let mut x = 5;
  |         +++

For more information about this error, try `rustc --explain E0384`.
error: could not compile `player` (bin \"player\") due to 1 previous error
";

    const MULTIPLE_ERRORS: &str = "error[E0425]: cannot find value `y` in this scope
 --> src/main.rs:3:20
  |
3 |     println!(\"{}\", y);
  |                    ^ help: a local variable with a similar name exists: `x`

error[E0308]: mismatched types
 --> src/main.rs:6:18
  |
6 |     let n: i32 = \"five\";
  |            ---   ^^^^^^ expected `i32`, found `&str`
  |            |
  |            expected due to this

error[E0382]: borrow of moved value: `s`
 --> src/main.rs:10:20
   |
8  |     let s = String::from(\"hi\");
   |         - move occurs because `s` has type `String`, which does not implement the `Copy` trait
9  |     take(s);
   |          - value moved here
10 |     println!(\"{}\", s);
   |                    ^ value borrowed here after move
   |
note: consider changing this parameter type in function `take` to borrow instead if owning the value isn't necessary
  --> src/main.rs:13:12
   |
13 | fn take(v: String) {}
   |    ----    ^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0308, E0382, E0425.
";

    const WARNINGS_AND_ERROR: &str = "warning: unused variable: `z`
 --> src/main.rs:2:9
  |
2 |     let z = 1;
  |         ^ help: if this is intentional, prefix it with an underscore: `_z`
  |
  = note: `#[warn(unused_variables)]` on by default

error: expected one of `,`, `.`, `?`, `}`, or an operator, found `y`
 --> src/main.rs:5:13
  |
5 |     let a = x y;
  |              -^ expected one of `,`, `.`, `?`, `}`, or an operator
  |              |
  |              help: missing `,`

warning: `player` (bin \"player\") generated 1 warning
error: could not compile `player` (bin \"player\") due to 1 previous error; 1 warning emitted
";

    const WARNINGS_ONLY: &str = "warning: unused variable: `z`
 --> src/main.rs:2:9
  |
2 |     let z = 1;
  |         ^ help: if this is intentional, prefix it with an underscore: `_z`
";

    #[test]
    fn single_error_line_mapped_through_preamble() {
        assert_eq!(error_lines(SINGLE_ERROR, 0), vec![4]);
        assert_eq!(error_lines(SINGLE_ERROR, SERVER_PREAMBLE_LINES), vec![3]);
    }

    #[test]
    fn multiple_errors_take_primary_locations_only() {
        // The `note:` location (line 13) is secondary and is skipped.
        assert_eq!(error_lines(MULTIPLE_ERRORS, 0), vec![3, 6, 10]);
    }

    #[test]
    fn warnings_are_not_highlighted() {
        assert_eq!(error_lines(WARNINGS_AND_ERROR, 0), vec![5]);
        assert!(error_lines(WARNINGS_ONLY, 0).is_empty());
    }

    #[test]
    fn no_location_yields_nothing() {
        assert!(error_lines("[fail] expected `let answer` binding", 0).is_empty());
        assert!(
            error_lines(
                "error[E0601]: `main` function not found in crate `player`\n",
                0
            )
            .is_empty()
        );
        assert!(error_lines("", 0).is_empty());
    }

    #[test]
    fn preamble_line_itself_is_dropped() {
        let out = "error: crate-level attribute oops\n --> src/lib.rs:1:1\n";
        assert!(error_lines(out, 1).is_empty());
        assert_eq!(error_lines(out, 0), vec![1]);
    }

    #[test]
    fn ignores_non_crate_root_paths_and_handles_windows_paths() {
        let std_loc = "error[E0277]: oops\n --> /rustc/abc/library/core/src/fmt/mod.rs:9:1\n";
        assert!(error_lines(std_loc, 0).is_empty());
        let win = "error[E0308]: mismatched types\n --> C:\\sandbox\\src\\main.rs:7:3\n";
        assert_eq!(error_lines(win, 0), vec![7]);
    }

    #[test]
    fn dedups_repeated_lines() {
        let out = "error[E0425]: a\n --> src/main.rs:2:1\nerror[E0425]: b\n --> src/main.rs:2:9\n";
        assert_eq!(error_lines(out, 0), vec![2]);
    }

    #[test]
    fn works_through_compile_client_wrapper() {
        let wrapped = format!("[fail] build failed\n--stderr--\n{SINGLE_ERROR}");
        assert_eq!(error_lines(&wrapped, SERVER_PREAMBLE_LINES), vec![3]);
    }

    #[test]
    fn parse_result_classifies_every_client_format() {
        let p = parse_result("[ok] answer = 42");
        assert_eq!(
            (p.kind, p.offline, p.body),
            (ResultKind::Pass, false, "answer = 42")
        );

        let p = parse_result("[fail] missing `let answer`\n--stderr--\nx");
        assert_eq!(p.kind, ResultKind::Fail);
        assert_eq!(p.body, "missing `let answer`\n--stderr--\nx");

        let p = parse_result("[offline mode — server unreachable]\n[fail] nope");
        assert_eq!(
            (p.kind, p.offline, p.body),
            (ResultKind::Fail, true, "nope")
        );

        let p = parse_result("[offline mode — server unreachable]\n[ok] yes");
        assert_eq!((p.kind, p.offline), (ResultKind::Pass, true));

        assert_eq!(parse_result("compiling…").kind, ResultKind::Pending);
        assert_eq!(
            parse_result("[client error] connection refused").kind,
            ResultKind::Error
        );
        assert_eq!(
            parse_result("[wasm] no stub grader for encounter `x`").kind,
            ResultKind::Error
        );
    }
}
