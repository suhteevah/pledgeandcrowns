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

/// What exactly a result says, for the result banner. Finer than
/// [`ResultKind`]; written by `compile_client/outcome.rs` as the tag in
/// front of the message (`[ok]`, `[ok:unverified]`, `[fail:compile]`...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultTag {
    /// `[ok]`: behaviour was checked and matched.
    Verified,
    /// `[ok:unverified]` (or any unknown `[ok:...]`): accepted, behaviour
    /// not checked.
    Unverified,
    /// `[ok:pattern]`: pattern grader (server `/compile` or offline stub).
    PatternOnly,
    /// `[fail]` (or any unknown `[fail:...]`): rejected, no detail.
    Failed,
    /// `[fail:compile]`
    CompileError,
    /// `[fail:runtime]`
    RuntimeError,
    /// `[fail:output]`
    WrongOutput,
    /// `[fail:static]`
    StaticCheckFailed,
    /// Not graded: transport / client errors (any other leading tag).
    NotChecked,
    /// `compiling…`
    Pending,
}

impl ResultTag {
    fn from_tag(tag: &str) -> Option<Self> {
        Some(match tag {
            "ok" => Self::Verified,
            "ok:pattern" => Self::PatternOnly,
            "fail" => Self::Failed,
            "fail:compile" => Self::CompileError,
            "fail:runtime" => Self::RuntimeError,
            "fail:output" => Self::WrongOutput,
            "fail:static" => Self::StaticCheckFailed,
            // Never read an unknown pass tag as verified.
            t if t.starts_with("ok:") => Self::Unverified,
            t if t.starts_with("fail:") => Self::Failed,
            _ => return None,
        })
    }

    pub fn kind(self) -> ResultKind {
        match self {
            Self::Verified | Self::Unverified | Self::PatternOnly => ResultKind::Pass,
            Self::Failed
            | Self::CompileError
            | Self::RuntimeError
            | Self::WrongOutput
            | Self::StaticCheckFailed => ResultKind::Fail,
            Self::NotChecked => ResultKind::Error,
            Self::Pending => ResultKind::Pending,
        }
    }
}

/// A compile result split for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResult<'a> {
    pub kind: ResultKind,
    pub tag: ResultTag,
    /// True when the native client fell back to the offline stub grader.
    pub offline: bool,
    /// Message with the verdict tag and offline banner removed. Still
    /// carries server markers; [`error_lines`] reads this, and
    /// [`clean_for_display`] tidies it for the player.
    pub body: &'a str,
}

/// Structured extras of a server verdict that don't fit the string
/// protocol: the server's note and expected/actual on a wrong answer.
/// `for_result` is the `last_compile_result` string this belongs to; the
/// editor ignores a detail whose string no longer matches (another plugin
/// cleared or replaced the result).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResultDetail {
    pub for_result: String,
    pub note: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

impl ResultDetail {
    /// No extras, for `formatted`.
    pub fn bare(formatted: &str) -> Self {
        Self {
            for_result: formatted.to_string(),
            ..Self::default()
        }
    }
}

const OFFLINE_BANNER_PREFIX: &str = "[offline mode";
const PENDING_TEXT: &str = "compiling…";

/// Classify a `last_compile_result` string. Formats produced by
/// `compile_client`: a verdict tag (`[ok]`, `[ok:unverified]`,
/// `[ok:pattern]`, `[fail]`, `[fail:compile|runtime|output|static]`), an
/// optional leading `[offline mode — server unreachable]` line,
/// `[client error] ...` and other refusal tags, and the `compiling…`
/// placeholder.
pub fn parse_result(raw: &str) -> ParsedResult<'_> {
    let mut rest = raw;
    let mut offline = false;
    if rest.starts_with(OFFLINE_BANNER_PREFIX) {
        offline = true;
        rest = rest.split_once('\n').map_or("", |(_, tail)| tail);
    }
    let verdict = rest
        .strip_prefix('[')
        .and_then(|r| r.split_once(']'))
        .and_then(|(tag, body)| ResultTag::from_tag(tag).map(|t| (t, body)));
    let (tag, body) = match verdict {
        Some(v) => v,
        None if rest.trim() == PENDING_TEXT => (ResultTag::Pending, rest),
        None => (ResultTag::NotChecked, rest),
    };
    ParsedResult {
        kind: tag.kind(),
        tag,
        offline,
        body: body.trim_start_matches(' '),
    }
}

/// Words that start a whole line of transport/tooling noise. Display only.
const NOISE_LINES: &[&str] = &["--stderr--", "--stdout--"];
/// Server verdict lines (`compile-api` `real_response`); the banner says
/// the same thing.
const VERDICT_LINE_PREFIXES: &[&str] = &["-- checked:", "-- not checked:"];
/// Route / grader markers at the start of a line (`[real]` from
/// `/compile-real`, `[stub]` from the offline grader, `[freeform]` from the
/// pattern grader's no-grader reply).
const LINE_MARKERS: &[&str] = &["[real]", "[stub]", "[freeform]"];
/// Cargo progress lines (indented status words); rustc diagnostics follow.
const CARGO_PROGRESS: &[&str] = &[
    "Compiling ",
    "Checking ",
    "Finished ",
    "Running ",
    "Blocking waiting",
    "Updating ",
    "Downloaded ",
];
/// Section separators the server uses, mapped to plain headings.
const SECTION_RENAMES: &[(&str, &str)] = &[
    ("--- captured stderr ---", "Your program's error output:"),
    (
        "--- your program's stderr ---",
        "Your program's error output:",
    ),
    (
        "--- partial stderr ---",
        "Build output before the time limit:",
    ),
];

/// Every marker [`clean_for_display`] removes or rewrites (for docs/tests).
pub fn noise_markers() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = Vec::new();
    v.extend(NOISE_LINES);
    v.extend(VERDICT_LINE_PREFIXES);
    v.extend(LINE_MARKERS);
    v.extend(SECTION_RENAMES.iter().map(|(from, _)| *from));
    v.push("error: could not compile");
    v
}

/// Player-facing copy of a result body: transport markers, server verdict
/// lines and cargo progress noise removed, server section separators
/// renamed, blank runs collapsed. Never used for [`error_lines`], which
/// must see the server text exactly.
pub fn clean_for_display(body: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for raw in body.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if NOISE_LINES.contains(&trimmed)
            || VERDICT_LINE_PREFIXES.iter().any(|p| trimmed.starts_with(p))
            || trimmed.starts_with("error: could not compile")
            || (line.starts_with(' ') && CARGO_PROGRESS.iter().any(|p| trimmed.starts_with(p)))
        {
            continue;
        }
        if let Some((_, to)) = SECTION_RENAMES.iter().find(|(from, _)| trimmed == *from) {
            out.push(String::new());
            out.push((*to).to_string());
            continue;
        }
        let mut kept = line;
        if let Some(rest) = LINE_MARKERS.iter().find_map(|m| kept.strip_prefix(m)) {
            kept = rest.strip_prefix(' ').unwrap_or(rest);
            if kept.trim().is_empty() {
                continue;
            }
        }
        out.push(kept.to_string());
    }
    // Collapse blank runs and trim blank edges.
    let mut cleaned: Vec<String> = Vec::with_capacity(out.len());
    for line in out {
        let blank = line.trim().is_empty();
        if blank && cleaned.last().is_none_or(|l: &String| l.trim().is_empty()) {
            continue;
        }
        cleaned.push(line);
    }
    while cleaned.last().is_some_and(|l| l.trim().is_empty()) {
        cleaned.pop();
    }
    cleaned.join("\n")
}

/// For a wrong-output result whose expected/actual are shown on their own:
/// the part of the cleaned body that adds something (the program's error
/// output), or `None` when the body only repeats expected/actual.
pub fn wrong_output_extra(cleaned: &str) -> Option<&str> {
    const HEADING: &str = "Your program's error output:";
    cleaned
        .find(HEADING)
        .map(|i| cleaned[i..].trim())
        .filter(|s| !s.is_empty())
}

/// Visual class of the result banner (colours in `palette`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerStyle {
    /// Verified pass.
    Pass,
    /// Pass whose behaviour was not checked (calm, not an alarm).
    Unverified,
    Fail,
    /// Not graded at all (busy, offline, refused).
    NotChecked,
    Pending,
}

/// What the result banner says: a short title and one plain line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Banner {
    pub title: &'static str,
    pub style: BannerStyle,
    pub reason: String,
}

/// Capitalise the first letter and end with a full stop.
fn sentence(s: &str) -> String {
    let s = s.trim();
    let mut chars = s.chars();
    let mut out: String = match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => return String::new(),
    };
    if !out.ends_with(['.', '!', '?']) {
        out.push('.');
    }
    out
}

/// Banner for a parsed result. `note` is the server's one-line note, if
/// it sent one (shown as the reason for an unverified pass).
pub fn banner(p: &ParsedResult<'_>, note: Option<&str>) -> Banner {
    let note = note.map(sentence).filter(|n| !n.is_empty());
    let (title, style, reason) = match (p.tag, p.offline) {
        (ResultTag::Pending, _) => (
            "COMPILING…",
            BannerStyle::Pending,
            "Sending your code to the compile server.".to_string(),
        ),
        (ResultTag::NotChecked, _) => (
            "NOT CHECKED",
            BannerStyle::NotChecked,
            "Your code was not graded this time. This does not count as a failed attempt."
                .to_string(),
        ),
        (ResultTag::Verified, _) => (
            "PASS · VERIFIED",
            BannerStyle::Pass,
            "Checked: your program does what this mission asks.".to_string(),
        ),
        (t, true) if t.kind() == ResultKind::Pass => (
            "PASS · OFFLINE PATTERN CHECK",
            BannerStyle::Unverified,
            "The server was unreachable, so only a pattern check ran. This pass is not saved yet."
                .to_string(),
        ),
        (ResultTag::PatternOnly, false) => (
            "PASS · PATTERN CHECK ONLY",
            BannerStyle::Unverified,
            note.unwrap_or_else(|| {
                "Your code has what this mission looks for, but it was not compiled or run."
                    .to_string()
            }),
        ),
        (ResultTag::Unverified, false) => (
            "PASS · NOT VERIFIED",
            BannerStyle::Unverified,
            note.unwrap_or_else(|| {
                "It was accepted, but nothing checked what your code does.".to_string()
            }),
        ),
        (_, true) => (
            "FAIL · OFFLINE PATTERN CHECK",
            BannerStyle::Fail,
            "The server was unreachable; an offline pattern check rejected this.".to_string(),
        ),
        (ResultTag::CompileError, false) => (
            "FAIL · DOES NOT COMPILE",
            BannerStyle::Fail,
            "rustc rejected the code. Fix the first error first; later ones often follow from it."
                .to_string(),
        ),
        (ResultTag::RuntimeError, false) => (
            "FAIL · CRASHED WHILE RUNNING",
            BannerStyle::Fail,
            "It compiled, but stopped with an error when it ran.".to_string(),
        ),
        (ResultTag::WrongOutput, false) => (
            "FAIL · WRONG OUTPUT",
            BannerStyle::Fail,
            "It ran, but did not do what this mission asks.".to_string(),
        ),
        (ResultTag::StaticCheckFailed, false) => (
            "FAIL · LESSON NOT USED",
            BannerStyle::Fail,
            "It compiles, but does not use what this mission teaches.".to_string(),
        ),
        (_, false) => (
            "FAIL",
            BannerStyle::Fail,
            "Not accepted yet. The details are below.".to_string(),
        ),
    };
    Banner {
        title,
        style,
        reason,
    }
}

/// How the result pane colours one cleaned line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineRole {
    /// `error[E..]: ...` / `error: ...` header.
    ErrorHeader,
    /// `warning: ...` header.
    WarningHeader,
    /// ` --> file:line:col`.
    Location,
    /// `help:` / `note:` / `= note:`.
    Hint,
    Plain,
}

pub fn line_role(line: &str) -> LineRole {
    let t = line.trim_start();
    if line.starts_with("error") {
        LineRole::ErrorHeader
    } else if line.starts_with("warning") {
        LineRole::WarningHeader
    } else if t.starts_with("--> ") {
        LineRole::Location
    } else if t.starts_with("help:")
        || t.starts_with("note:")
        || t.starts_with("= note:")
        || t.starts_with("= help:")
    {
        LineRole::Hint
    } else {
        LineRole::Plain
    }
}

/// The first rustc error header in cleaned output (`error[E0384]: ...`),
/// pinned above the result pane's scrolling block so it is always visible.
pub fn first_error_line(cleaned: &str) -> Option<&str> {
    cleaned
        .lines()
        .find(|l| line_role(l) == LineRole::ErrorHeader)
        .map(str::trim_end)
}

/// `cleaned` without its first non-blank line when that line is `pinned`
/// (the headline the result pane pins above the block), so the headline
/// is shown once. Anything else is returned unchanged.
pub fn without_leading_line<'a>(cleaned: &'a str, pinned: &str) -> &'a str {
    let lead = cleaned.len() - cleaned.trim_start_matches(['\n', '\r']).len();
    let rest = &cleaned[lead..];
    let (first, after) = rest.split_once('\n').unwrap_or((rest, ""));
    if first.trim_end() == pinned.trim_end() {
        tracing::trace!("result pane: pinned headline dropped from block");
        after
    } else {
        cleaned
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

    #[test]
    fn pinned_headline_is_shown_once() {
        let out = "error[E0384]: cannot assign twice  \n --> src/main.rs:4:5\n  |";
        let pinned = first_error_line(out).unwrap_or_default();
        assert_eq!(
            without_leading_line(out, pinned),
            " --> src/main.rs:4:5\n  |"
        );
        // Leading blank lines are skipped; a headline further down stays.
        assert_eq!(without_leading_line("\nerror: x\nrest", "error: x"), "rest");
        let warn_first = "warning: w\nerror: x";
        assert_eq!(without_leading_line(warn_first, "error: x"), warn_first);
        assert_eq!(without_leading_line("error: x", "error: x"), "");
    }

    #[test]
    fn first_error_line_skips_warnings_and_art() {
        let out = "warning: unused variable: `n`\n --> src/main.rs:7:9\n  |\nerror[E0425]: cannot find value `y` in this scope  \n  --> src/main.rs:10:28\nerror[E0308]: mismatched types";
        assert_eq!(
            first_error_line(out),
            Some("error[E0425]: cannot find value `y` in this scope")
        );
        assert_eq!(first_error_line("all good\n42"), None);
    }

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
    fn parse_result_reads_every_verdict_tag() {
        for (raw, tag, kind) in [
            ("[ok] 42", ResultTag::Verified, ResultKind::Pass),
            (
                "[ok:unverified] 42",
                ResultTag::Unverified,
                ResultKind::Pass,
            ),
            ("[ok:pattern] 42", ResultTag::PatternOnly, ResultKind::Pass),
            ("[ok:someday] 42", ResultTag::Unverified, ResultKind::Pass),
            ("[fail] x", ResultTag::Failed, ResultKind::Fail),
            (
                "[fail:compile] x",
                ResultTag::CompileError,
                ResultKind::Fail,
            ),
            (
                "[fail:runtime] x",
                ResultTag::RuntimeError,
                ResultKind::Fail,
            ),
            ("[fail:output] x", ResultTag::WrongOutput, ResultKind::Fail),
            (
                "[fail:static] x",
                ResultTag::StaticCheckFailed,
                ResultKind::Fail,
            ),
            ("[fail:new] x", ResultTag::Failed, ResultKind::Fail),
            (
                "[rate limited] wait",
                ResultTag::NotChecked,
                ResultKind::Error,
            ),
        ] {
            let p = parse_result(raw);
            assert_eq!((p.tag, p.kind), (tag, kind), "{raw}");
        }
        assert_eq!(parse_result("[fail:compile] x").body, "x");
    }

    #[test]
    fn error_lines_survive_every_failure_tag() {
        let wrapped = format!("[fail:compile] [real]\n--stderr--\n{SINGLE_ERROR}");
        let p = parse_result(&wrapped);
        assert_eq!(error_lines(p.body, SERVER_PREAMBLE_LINES), vec![3]);
    }

    #[test]
    fn clean_for_display_strips_noise_but_keeps_the_error() {
        let raw = format!(
            "[real]\n--stderr--\n{SINGLE_ERROR}\n--- captured stderr ---\npanicked\n-- not checked: x\n[stub] fine"
        );
        let c = clean_for_display(&raw);
        for gone in [
            "[real]",
            "--stderr--",
            "Checking player",
            "could not compile",
            "-- not checked",
            "[stub]",
            "--- captured",
        ] {
            assert!(!c.contains(gone), "{gone} left in:\n{c}");
        }
        assert!(c.starts_with("error[E0384]"), "{c}");
        assert!(c.contains(" --> src/main.rs:4:5"));
        assert!(c.contains("rustc --explain E0384"));
        assert!(c.contains("Your program's error output:\npanicked"));
        assert!(c.ends_with("fine"));
        assert!(!c.contains("\n\n\n"));
        // The parser still reads the raw body, not the cleaned one.
        assert_eq!(error_lines(&raw, SERVER_PREAMBLE_LINES), vec![3]);
        assert_eq!(
            clean_for_display("[real] 42\n-- checked: does what the mission asks"),
            "42"
        );
        assert_eq!(clean_for_display("[real]"), "");
        assert!(noise_markers().contains(&"[real]"));
    }

    #[test]
    fn wrong_output_extra_keeps_only_new_information() {
        let only =
            "41\nYour program ran, but did not do what this mission asks.\nExpected `42`\nGot:\n41";
        assert_eq!(wrong_output_extra(only), None);
        let more = format!("{only}\n\nYour program's error output:\nboom");
        assert_eq!(
            wrong_output_extra(&more),
            Some("Your program's error output:\nboom")
        );
    }

    #[test]
    fn banner_distinguishes_verified_unverified_and_pattern() {
        let b = banner(&parse_result("[ok] 42"), None);
        assert_eq!((b.title, b.style), ("PASS · VERIFIED", BannerStyle::Pass));

        let b = banner(
            &parse_result("[ok:unverified] compiled"),
            Some("behaviour not checked on this server: needs a GPU"),
        );
        assert_eq!(
            (b.title, b.style),
            ("PASS · NOT VERIFIED", BannerStyle::Unverified)
        );
        assert_eq!(
            b.reason,
            "Behaviour not checked on this server: needs a GPU."
        );
        let b = banner(&parse_result("[ok:unverified] x"), None);
        assert!(!b.reason.is_empty());

        let b = banner(&parse_result("[ok:pattern] nice"), None);
        assert_eq!(
            (b.title, b.style),
            ("PASS · PATTERN CHECK ONLY", BannerStyle::Unverified)
        );

        let b = banner(
            &parse_result("[offline mode — server unreachable]\n[ok:pattern] x"),
            None,
        );
        assert_eq!(b.style, BannerStyle::Unverified);
        assert!(b.title.contains("OFFLINE"));
    }

    #[test]
    fn banner_failures_and_non_grades() {
        for (raw, title) in [
            ("[fail:compile] x", "FAIL · DOES NOT COMPILE"),
            ("[fail:runtime] x", "FAIL · CRASHED WHILE RUNNING"),
            ("[fail:output] x", "FAIL · WRONG OUTPUT"),
            ("[fail:static] x", "FAIL · LESSON NOT USED"),
            ("[fail] x", "FAIL"),
        ] {
            let b = banner(&parse_result(raw), None);
            assert_eq!((b.title, b.style), (title, BannerStyle::Fail), "{raw}");
        }
        let b = banner(&parse_result("[server busy] wait"), None);
        assert_eq!(b.style, BannerStyle::NotChecked);
        assert_eq!(
            banner(&parse_result("compiling…"), None).style,
            BannerStyle::Pending
        );
    }

    #[test]
    fn line_roles() {
        assert_eq!(line_role("error[E0384]: x"), LineRole::ErrorHeader);
        assert_eq!(line_role("warning: unused"), LineRole::WarningHeader);
        assert_eq!(line_role(" --> src/main.rs:4:5"), LineRole::Location);
        assert_eq!(line_role("help: consider"), LineRole::Hint);
        assert_eq!(line_role("  = note: `#[warn]`"), LineRole::Hint);
        assert_eq!(line_role("3 |     let x = 5;"), LineRole::Plain);
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
