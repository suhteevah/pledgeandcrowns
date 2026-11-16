// SPDX-License-Identifier: MIT
//! Pure mapping from an HTTP exchange to what the player sees, and the
//! rule for which passes are saved as clears. Shared by native and web;
//! no I/O here, so every branch is unit-tested.
//!
//! # Honest grades
//!
//! `/compile-real` says how it graded (`grade`, `behaviour_checked`,
//! `note`, and `expected` / `actual` on a wrong answer). A pass is only
//! [`Grade::Passed`] (verified) when the server says `grade: "passed"`
//! AND `behaviour_checked: true`. Every other pass is
//! [`Grade::PassedUnverified`]: the pattern route (`/compile`, which never
//! sends those fields), an old server that omits them, or a mission the
//! server cannot check (`compiled_not_checked` / `ran_not_checked`).
//! Absent fields are never read as "verified".
//!
//! # Fallback and clear policy
//!
//! | Situation                                   | Feedback shown            | Pass saved? |
//! |---------------------------------------------|---------------------------|-------------|
//! | Server graded: verified pass                | PASS (verified)           | yes         |
//! | Server graded: unverified / pattern pass    | PASS, NOT VERIFIED        | yes (policy)|
//! | Server graded: any failure                  | FAIL + reason             | no          |
//! | Server configured but unreachable / errored | offline stub grader       | **no**      |
//! | 429 rate limited / 503 build queue full     | "wait N seconds" message  | no          |
//! | 401 unauthorized / 413 too large            | plain explanation         | no          |
//! | No server configured on purpose (demo mode) | stub grader (pattern only)| yes         |
//!
//! An unverified pass still counts as a clear so the player is never
//! blocked by a mission the server cannot check; the editor labels it
//! NOT VERIFIED instead.
//!
//! The strings produced keep the editor's contract
//! (`editor/diagnostics.rs::parse_result`): a `[ok...]` / `[fail...]`
//! verdict tag (see [`Grade::tag`]), an optional leading `[offline mode
//! ...]` banner line for stub fallback, and any other leading tag for
//! "not the player's fault" errors, which never count as failed attempts.
//! Server text is passed through unchanged so rustc line numbers still
//! map via `SERVER_PREAMBLE_LINES`; the editor strips transport noise
//! only at display time.

use serde::Deserialize;

use super::config::CompileMode;
use crate::plugins::editor::diagnostics::ResultDetail;
use crate::plugins::stub_grader::StubVerdict;

/// Cap on a server-supplied retry delay, so a bogus header can't tell the
/// player to wait a day.
pub const MAX_RETRY_AFTER_SECS: u64 = 600;

/// Banner line for stub feedback while a server is configured. Must keep
/// the `[offline mode` prefix the editor keys on.
pub const OFFLINE_FALLBACK_BANNER: &str = "[offline mode — server unreachable]";

/// A completed HTTP exchange, reduced to what the mapper needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpReply {
    pub status: u16,
    /// Raw `Retry-After` header value, if any.
    pub retry_after: Option<String>,
    pub body: String,
}

/// Why no HTTP reply arrived.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    #[error("the request timed out after {0}s")]
    Timeout(u64),
    #[error("could not connect to the compile server")]
    Connect,
    #[error("network error: {0}")]
    Other(String),
}

/// Why a pass was not verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unverified {
    /// Pattern grader (`/compile` route, or the offline stub): looked for
    /// the right pieces in the source; nothing was compiled or run.
    PatternOnly,
    /// `compiled_not_checked`: built (and passed static checks), not run.
    CompiledNotChecked,
    /// `ran_not_checked`: built and ran cleanly, output not checked.
    RanNotChecked,
    /// `ok: true` but the server did not say how it graded (old server,
    /// unknown `grade` value, or contradictory fields).
    Unknown,
    /// `passed` with `behaviour_checked: true`, from an Act 9 mission the
    /// server grades with `unsafe` allowed (`HarnessMode::UnsafeLateNonce`,
    /// recognised by [`UNSAFE_ALLOWED_NOTE_MARKER`] in its note). The
    /// checker did call the player's code and every result matched, but in
    /// that mode `unsafe` player code can rewrite the checker's stored
    /// results (compile-api/HARNESS-NOTES.md, the documented known limit),
    /// so the pass is not shown as VERIFIED.
    UnsafeAllowed,
}

/// Words the server's pass note carries for a harness mission graded with
/// `unsafe` allowed: `"checked by calling your code (N checks; `unsafe`
/// allowed, results keyed by the server only after your code finished)"`
/// (`compile-api` `expectations::harness_pass_note`). A cross-crate test
/// (`tests/compile_api_client.rs`) feeds the server's real note through
/// [`Grade::from_server`].
pub const UNSAFE_ALLOWED_NOTE_MARKER: &str = "`unsafe` allowed";

/// How the server graded one submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    /// Built, ran, and its behaviour was checked and matched.
    Passed,
    /// Accepted, but behaviour was not checked. Still a clear (policy).
    PassedUnverified(Unverified),
    /// Built and ran, but did not do what the mission asks.
    WrongOutput,
    /// Did not build.
    CompileError,
    /// Built, then crashed / timed out / exited non-zero.
    RuntimeError,
    /// Built, but does not use what a compile-only mission teaches.
    StaticCheckFailed,
    /// Rejected without a grade (pattern grader, or an old server).
    Failed,
}

impl Grade {
    pub fn is_pass(self) -> bool {
        matches!(self, Self::Passed | Self::PassedUnverified(_))
    }

    /// The leading tag of the editor string (`editor/diagnostics.rs`).
    pub fn tag(self) -> &'static str {
        match self {
            Self::Passed => "[ok]",
            Self::PassedUnverified(Unverified::PatternOnly) => "[ok:pattern]",
            Self::PassedUnverified(Unverified::UnsafeAllowed) => "[ok:unsafe]",
            Self::PassedUnverified(_) => "[ok:unverified]",
            Self::WrongOutput => "[fail:output]",
            Self::CompileError => "[fail:compile]",
            Self::RuntimeError => "[fail:runtime]",
            Self::StaticCheckFailed => "[fail:static]",
            Self::Failed => "[fail]",
        }
    }

    /// Map the server's fields. `ok` is already `ok && status == 200`.
    /// Only `passed` + `behaviour_checked: true` is verified, and not even
    /// that when the note says `unsafe` was allowed
    /// ([`Unverified::UnsafeAllowed`]).
    pub fn from_server(
        mode: CompileMode,
        ok: bool,
        grade: Option<&str>,
        behaviour_checked: Option<bool>,
        note: Option<&str>,
    ) -> Self {
        if ok {
            return match (grade, behaviour_checked) {
                (Some("passed"), Some(true))
                    if note.is_some_and(|n| n.contains(UNSAFE_ALLOWED_NOTE_MARKER)) =>
                {
                    Self::PassedUnverified(Unverified::UnsafeAllowed)
                }
                (Some("passed"), Some(true)) => Self::Passed,
                (Some("compiled_not_checked"), _) => {
                    Self::PassedUnverified(Unverified::CompiledNotChecked)
                }
                (Some("ran_not_checked"), _) => Self::PassedUnverified(Unverified::RanNotChecked),
                (None, None) if mode == CompileMode::Pattern => {
                    Self::PassedUnverified(Unverified::PatternOnly)
                }
                _ => Self::PassedUnverified(Unverified::Unknown),
            };
        }
        match grade {
            Some("wrong_output") => Self::WrongOutput,
            Some("compile_error") => Self::CompileError,
            Some("runtime_error") => Self::RuntimeError,
            Some("static_check_failed") => Self::StaticCheckFailed,
            _ => Self::Failed,
        }
    }
}

/// A graded reply: the outcome class plus everything the editor shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradedReply {
    pub grade: Grade,
    /// Editor-ready string (tag + server text verbatim).
    pub message: String,
    /// Server's one-line note on how it graded, if sent.
    pub note: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

/// Classified result of one compile request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiOutcome {
    /// The server graded the code.
    Graded(GradedReply),
    /// 429. Seconds to wait, when the server said.
    RateLimited { retry_after_secs: Option<u64> },
    /// 503: build queue full.
    Busy { retry_after_secs: Option<u64> },
    /// 401: missing or wrong bearer token.
    Unauthorized,
    /// 413: code too large for the server.
    TooLarge,
    /// No usable answer (unreachable, timeout, 5xx, unreadable body,
    /// unexpected status, or a misconfigured client).
    Offline { reason: String },
}

impl ApiOutcome {
    /// The grade, when the server graded the code.
    pub fn grade(&self) -> Option<Grade> {
        match self {
            Self::Graded(g) => Some(g.grade),
            _ => None,
        }
    }
}

/// The server's body. `ok`, `stdout`, `stderr` are the original contract;
/// the rest is optional (absent on `/compile` and on older servers). Its
/// rejection bodies add `error` / `retry_after_seconds`. `grade` is read
/// as a plain string so an unknown future value degrades to "unverified"
/// instead of making the body unreadable.
#[derive(Deserialize, Debug)]
struct CompileResponse {
    ok: bool,
    stdout: String,
    stderr: String,
    #[serde(default)]
    retry_after_seconds: Option<u64>,
    #[serde(default)]
    grade: Option<String>,
    #[serde(default)]
    behaviour_checked: Option<bool>,
    #[serde(default)]
    expected: Option<String>,
    #[serde(default)]
    actual: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

/// Editor-ready text for a server verdict: the grade tag, then the
/// server's stdout and stderr verbatim (the editor's error-line parser
/// reads this text, so it is never rewritten here).
pub fn format_server_verdict(grade: Grade, stdout: &str, stderr: &str) -> String {
    let mut out = format!("{} {stdout}", grade.tag());
    if !stderr.is_empty() {
        out.push_str("\n--stderr--\n");
        out.push_str(stderr);
    }
    out
}

fn parse_retry_after(header: Option<&str>, body: Option<u64>) -> Option<u64> {
    header
        .and_then(|h| h.trim().parse::<u64>().ok())
        .or(body)
        .map(|s| s.min(MAX_RETRY_AFTER_SECS))
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

/// Map a transport result to an [`ApiOutcome`]. Pure. `mode` is the
/// route the request went to (a grade-less pass from `/compile` is
/// "pattern check only").
pub fn classify(mode: CompileMode, result: Result<HttpReply, TransportError>) -> ApiOutcome {
    let reply = match result {
        Ok(r) => r,
        Err(e) => {
            return ApiOutcome::Offline {
                reason: e.to_string(),
            };
        }
    };
    let parsed = serde_json::from_str::<CompileResponse>(&reply.body).ok();
    let retry = || {
        parse_retry_after(
            reply.retry_after.as_deref(),
            parsed.as_ref().and_then(|p| p.retry_after_seconds),
        )
    };
    match reply.status {
        200 | 400 => match parsed {
            // 400 carries a real verdict body ("empty source").
            Some(p) => {
                let ok = p.ok && reply.status == 200;
                let grade = Grade::from_server(
                    mode,
                    ok,
                    p.grade.as_deref(),
                    p.behaviour_checked,
                    p.note.as_deref(),
                );
                tracing::debug!(
                    status = reply.status,
                    ok,
                    server_grade = ?p.grade,
                    behaviour_checked = ?p.behaviour_checked,
                    ?grade,
                    "compile-api: graded reply mapped"
                );
                ApiOutcome::Graded(GradedReply {
                    grade,
                    message: format_server_verdict(grade, &p.stdout, &p.stderr),
                    note: non_empty(p.note),
                    expected: non_empty(p.expected),
                    actual: p.actual,
                })
            }
            None => ApiOutcome::Offline {
                reason: format!(
                    "the server sent a reply the game couldn't read (HTTP {})",
                    reply.status
                ),
            },
        },
        401 => ApiOutcome::Unauthorized,
        413 => ApiOutcome::TooLarge,
        429 => ApiOutcome::RateLimited {
            retry_after_secs: retry(),
        },
        503 => ApiOutcome::Busy {
            retry_after_secs: retry(),
        },
        s => ApiOutcome::Offline {
            reason: format!("the server answered with an error (HTTP {s})"),
        },
    }
}

/// What the main loop applies: text for the editor, pass/fail, whether a
/// pass is saved, and the structured extras the result pane shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub formatted: String,
    pub ok: bool,
    pub persist_clear: bool,
    /// The grade, when something graded the code (server or stub).
    pub grade: Option<Grade>,
    /// Note / expected / actual for the result pane. `for_result` equals
    /// `formatted`.
    pub detail: ResultDetail,
}

impl Verdict {
    fn plain(formatted: String, ok: bool, persist_clear: bool, grade: Option<Grade>) -> Self {
        let detail = ResultDetail::bare(&formatted);
        Self {
            formatted,
            ok,
            persist_clear,
            grade,
            detail,
        }
    }
}

fn wait_phrase(secs: Option<u64>) -> String {
    match secs {
        Some(1) => "about 1 second".to_string(),
        Some(s) if s > 1 => format!("about {s} seconds"),
        _ => "a few seconds".to_string(),
    }
}

/// The stub grader is a pattern grader: its pass is never "verified".
fn stub_grade(v: &StubVerdict) -> Grade {
    if v.ok {
        Grade::PassedUnverified(Unverified::PatternOnly)
    } else {
        Grade::Failed
    }
}

fn stub_text(v: &StubVerdict) -> String {
    let grade = stub_grade(v);
    let body = if v.ok { &v.stdout } else { &v.stderr };
    let mut out = format!("{} {body}", grade.tag());
    if v.ok && !v.stderr.is_empty() {
        out.push_str("\n--stderr--\n");
        out.push_str(&v.stderr);
    }
    out
}

/// Verdict when a server is configured. `stub` is only called for the
/// `Offline` outcome. Fallback passes are never saved.
pub fn verdict_with_server(
    outcome: &ApiOutcome,
    encounter_id: &str,
    stub: impl FnOnce() -> Option<StubVerdict>,
) -> Verdict {
    let refused = |formatted: String| Verdict::plain(formatted, false, false, None);
    match outcome {
        ApiOutcome::Graded(g) => {
            let ok = g.grade.is_pass();
            Verdict {
                formatted: g.message.clone(),
                ok,
                // Policy: verified AND unverified passes are clears.
                persist_clear: ok,
                grade: Some(g.grade),
                detail: ResultDetail {
                    for_result: g.message.clone(),
                    note: g.note.clone(),
                    expected: g.expected.clone(),
                    actual: g.actual.clone(),
                },
            }
        }
        ApiOutcome::RateLimited { retry_after_secs } => refused(format!(
            "[rate limited] You're compiling faster than the server allows. \
             Wait {} and press Compile again. Your code wasn't checked yet.",
            wait_phrase(*retry_after_secs)
        )),
        ApiOutcome::Busy { retry_after_secs } => refused(format!(
            "[server busy] The compile server is busy with other players' builds. \
             Wait {} and press Compile again. Your code wasn't checked yet.",
            wait_phrase(*retry_after_secs)
        )),
        ApiOutcome::Unauthorized => refused(
            "[server refused] The compile server didn't accept this game's access key, \
             so your code wasn't checked. If you run your own server, check \
             PLEDGE_API_TOKEN (desktop) or apiToken in PLEDGE_CONFIG (web)."
                .to_string(),
        ),
        ApiOutcome::TooLarge => refused(
            "[too large] Your code is too long for the compile server to accept \
             (the limit is 64 KB). Trim it down and press Compile again."
                .to_string(),
        ),
        ApiOutcome::Offline { reason } => match stub() {
            Some(v) => {
                let mut formatted = format!("{OFFLINE_FALLBACK_BANNER}\n{}", stub_text(&v));
                if v.ok {
                    formatted.push_str(
                        "\n\n(Offline check only: this pass won't be saved until the \
                         compile server is reachable again.)",
                    );
                }
                Verdict::plain(formatted, v.ok, false, Some(stub_grade(&v)))
            }
            None => refused(format!(
                "[client error] Couldn't reach the compile server ({reason}), and there \
                 is no offline check for `{encounter_id}`. Try again in a moment."
            )),
        },
    }
}

/// Verdict in offline demo mode (no server configured on purpose): the
/// stub grader is canonical, so its passes are saved, but they are still
/// presented as "pattern check only". No offline banner.
pub fn verdict_offline_mode(encounter_id: &str, stub: Option<StubVerdict>) -> Verdict {
    match stub {
        Some(v) => Verdict::plain(stub_text(&v), v.ok, v.ok, Some(stub_grade(&v))),
        None => Verdict::plain(
            format!("[offline] no offline grader for encounter `{encounter_id}`"),
            false,
            false,
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::editor::diagnostics::{ResultKind, ResultTag, parse_result};

    const REAL: CompileMode = CompileMode::Real;
    const PATTERN: CompileMode = CompileMode::Pattern;

    fn reply(status: u16, retry: Option<&str>, body: &str) -> Result<HttpReply, TransportError> {
        Ok(HttpReply {
            status,
            retry_after: retry.map(str::to_string),
            body: body.to_string(),
        })
    }

    fn graded(mode: CompileMode, body: &str) -> GradedReply {
        match classify(mode, reply(200, None, body)) {
            ApiOutcome::Graded(g) => g,
            other => panic!("expected graded, got {other:?}"),
        }
    }

    fn stub(ok: bool) -> Option<StubVerdict> {
        Some(StubVerdict {
            ok,
            stdout: if ok {
                "stub pass".into()
            } else {
                String::new()
            },
            stderr: if ok {
                String::new()
            } else {
                "stub fail".into()
            },
        })
    }

    const REJECT_429: &str = r#"{"ok":false,"stdout":"","stderr":"rate limit exceeded; retry in 7s","error":"rate_limited","retry_after_seconds":7}"#;

    /// One `/compile-real` body per server `grade` value.
    fn real_body(ok: bool, grade: &str, checked: bool) -> String {
        format!(
            r#"{{"ok":{ok},"stdout":"[real] 42","stderr":"","grade":"{grade}","behaviour_checked":{checked},"note":"note for {grade}"}}"#
        )
    }

    #[test]
    fn every_server_grade_value_maps_to_its_own_outcome() {
        let cases = [
            (true, "passed", true, Grade::Passed, ResultTag::Verified),
            (
                true,
                "compiled_not_checked",
                false,
                Grade::PassedUnverified(Unverified::CompiledNotChecked),
                ResultTag::Unverified,
            ),
            (
                true,
                "ran_not_checked",
                false,
                Grade::PassedUnverified(Unverified::RanNotChecked),
                ResultTag::Unverified,
            ),
            (
                false,
                "wrong_output",
                true,
                Grade::WrongOutput,
                ResultTag::WrongOutput,
            ),
            (
                false,
                "compile_error",
                false,
                Grade::CompileError,
                ResultTag::CompileError,
            ),
            (
                false,
                "runtime_error",
                false,
                Grade::RuntimeError,
                ResultTag::RuntimeError,
            ),
            (
                false,
                "static_check_failed",
                false,
                Grade::StaticCheckFailed,
                ResultTag::StaticCheckFailed,
            ),
        ];
        for (ok, name, checked, want, tag) in cases {
            let g = graded(REAL, &real_body(ok, name, checked));
            assert_eq!(g.grade, want, "{name}");
            assert_eq!(g.note.as_deref(), Some(format!("note for {name}").as_str()));
            let v = verdict_with_server(&ApiOutcome::Graded(g), "m", || panic!("no stub"));
            assert_eq!(v.ok, ok, "{name}");
            assert_eq!(
                v.persist_clear, ok,
                "{name}: passes (even unverified) are clears"
            );
            let p = parse_result(&v.formatted);
            assert_eq!(p.tag, tag, "{name}");
            assert_eq!(p.kind == ResultKind::Pass, ok, "{name}");
            assert_eq!(v.detail.for_result, v.formatted);
        }
    }

    /// A late-nonce pass (Act 9, `unsafe` allowed) is a clear but not
    /// VERIFIED: the server's own note names the mode.
    #[test]
    fn unsafe_allowed_pass_is_a_clear_but_not_verified() {
        let body = r#"{"ok":true,"stdout":"[real] 42\n-- checked: does what the mission asks","stderr":"","grade":"passed","behaviour_checked":true,"note":"checked by calling your code (8 checks; `unsafe` allowed, results keyed by the server only after your code finished)"}"#;
        let g = graded(REAL, body);
        assert_eq!(g.grade, Grade::PassedUnverified(Unverified::UnsafeAllowed));
        assert_eq!(g.grade.tag(), "[ok:unsafe]");
        let v = verdict_with_server(&ApiOutcome::Graded(g), "m", || panic!("no stub"));
        assert!(v.ok && v.persist_clear, "still a clear");
        assert_eq!(parse_result(&v.formatted).tag, ResultTag::UnsafeAllowed);
        // The same pass from a ForbidUnsafe mission is verified.
        let plain = body.replace(
            "8 checks; `unsafe` allowed, results keyed by the server only after your code finished",
            "8 checks",
        );
        assert_eq!(graded(REAL, &plain).grade, Grade::Passed);
        // The marker only matters on a checked pass.
        let failed = body
            .replace(r#""ok":true"#, r#""ok":false"#)
            .replace(r#""grade":"passed""#, r#""grade":"wrong_output""#);
        assert_eq!(graded(REAL, &failed).grade, Grade::WrongOutput);
    }

    #[test]
    fn absent_fields_are_never_verified() {
        // Old server on the real route: ok but no grade fields.
        let g = graded(REAL, r#"{"ok":true,"stdout":"[real] 42","stderr":""}"#);
        assert_eq!(g.grade, Grade::PassedUnverified(Unverified::Unknown));
        // Pattern route: never sends grade fields; pattern check only.
        let g = graded(PATTERN, r#"{"ok":true,"stdout":"nice","stderr":""}"#);
        assert_eq!(g.grade, Grade::PassedUnverified(Unverified::PatternOnly));
        assert_eq!(parse_result(&g.message).tag, ResultTag::PatternOnly);
        // `passed` without `behaviour_checked: true` is not verified.
        let g = graded(
            REAL,
            r#"{"ok":true,"stdout":"","stderr":"","grade":"passed"}"#,
        );
        assert_eq!(g.grade, Grade::PassedUnverified(Unverified::Unknown));
        let g = graded(
            REAL,
            r#"{"ok":true,"stdout":"","stderr":"","grade":"passed","behaviour_checked":false}"#,
        );
        assert_eq!(g.grade, Grade::PassedUnverified(Unverified::Unknown));
        // Unknown future grade value: readable, unverified.
        let g = graded(
            REAL,
            r#"{"ok":true,"stdout":"","stderr":"","grade":"vibes"}"#,
        );
        assert_eq!(g.grade, Grade::PassedUnverified(Unverified::Unknown));
        // Failures without a grade are a generic fail.
        let g = graded(PATTERN, r#"{"ok":false,"stdout":"","stderr":"nope"}"#);
        assert_eq!(g.grade, Grade::Failed);
        let g = graded(
            REAL,
            r#"{"ok":false,"stdout":"","stderr":"x","grade":"passed"}"#,
        );
        assert_eq!(g.grade, Grade::Failed);
    }

    #[test]
    fn wrong_output_carries_expected_and_actual() {
        let body = r#"{"ok":false,"stdout":"[real] 41","stderr":"Your program ran, but did not do what this mission asks.\nExpected a line reading `42`\nGot:\n41","grade":"wrong_output","behaviour_checked":true,"expected":"a line reading `42`","actual":"41","note":"output checked against the mission"}"#;
        let g = graded(REAL, body);
        assert_eq!(g.grade, Grade::WrongOutput);
        assert_eq!(g.expected.as_deref(), Some("a line reading `42`"));
        assert_eq!(g.actual.as_deref(), Some("41"));
        let v = verdict_with_server(&ApiOutcome::Graded(g), "m", || None);
        assert_eq!(v.detail.expected.as_deref(), Some("a line reading `42`"));
        assert!(!v.persist_clear);
    }

    #[test]
    fn server_text_passes_through_byte_for_byte() {
        let g = graded(
            REAL,
            r#"{"ok":false,"stdout":"[real]","stderr":"error[E0384]: x\n --> src/main.rs:4:5","grade":"compile_error","behaviour_checked":false}"#,
        );
        assert_eq!(
            g.message,
            "[fail:compile] [real]\n--stderr--\nerror[E0384]: x\n --> src/main.rs:4:5"
        );
    }

    #[test]
    fn bad_request_with_body_is_a_failed_verdict() {
        let o = classify(
            PATTERN,
            reply(
                400,
                None,
                r#"{"ok":false,"stdout":"","stderr":"empty source"}"#,
            ),
        );
        assert_eq!(o.grade(), Some(Grade::Failed));
        // A 400 is never a pass, whatever `ok` says.
        let o = classify(
            PATTERN,
            reply(400, None, r#"{"ok":true,"stdout":"","stderr":""}"#),
        );
        assert_eq!(o.grade(), Some(Grade::Failed));
    }

    #[test]
    fn unreadable_success_body_is_offline() {
        assert!(matches!(
            classify(PATTERN, reply(200, None, "<html>proxy page</html>")),
            ApiOutcome::Offline { .. }
        ));
    }

    #[test]
    fn rate_limited_reads_header_then_body() {
        assert_eq!(
            classify(PATTERN, reply(429, Some("12"), REJECT_429)),
            ApiOutcome::RateLimited {
                retry_after_secs: Some(12)
            }
        );
        // HTTP-date header is not parsed; fall back to the body field.
        assert_eq!(
            classify(
                PATTERN,
                reply(429, Some("Wed, 21 Oct 2026 07:28:00 GMT"), REJECT_429)
            ),
            ApiOutcome::RateLimited {
                retry_after_secs: Some(7)
            }
        );
        assert_eq!(
            classify(PATTERN, reply(429, None, "")),
            ApiOutcome::RateLimited {
                retry_after_secs: None
            }
        );
        assert_eq!(
            classify(PATTERN, reply(429, Some("999999"), "")),
            ApiOutcome::RateLimited {
                retry_after_secs: Some(MAX_RETRY_AFTER_SECS)
            }
        );
    }

    #[test]
    fn busy_unauthorized_too_large() {
        assert_eq!(
            classify(REAL, reply(503, Some("5"), "")),
            ApiOutcome::Busy {
                retry_after_secs: Some(5)
            }
        );
        assert_eq!(
            classify(REAL, reply(401, None, "")),
            ApiOutcome::Unauthorized
        );
        assert_eq!(classify(REAL, reply(413, None, "")), ApiOutcome::TooLarge);
    }

    #[test]
    fn other_statuses_and_transport_errors_are_offline() {
        for s in [404, 405, 415, 422, 500, 502, 504] {
            assert!(
                matches!(
                    classify(REAL, reply(s, None, "{}")),
                    ApiOutcome::Offline { .. }
                ),
                "{s}"
            );
        }
        for e in [
            TransportError::Timeout(10),
            TransportError::Connect,
            TransportError::Other("dns".into()),
        ] {
            assert!(matches!(classify(REAL, Err(e)), ApiOutcome::Offline { .. }));
        }
    }

    #[test]
    fn unreachable_server_falls_back_to_stub_without_saving() {
        let off = ApiOutcome::Offline {
            reason: "could not connect".into(),
        };
        let v = verdict_with_server(&off, "m", || stub(true));
        assert!(v.ok, "stub pass shown as a pass");
        assert!(!v.persist_clear, "fallback pass must not be saved");
        let p = parse_result(&v.formatted);
        assert_eq!((p.kind, p.offline), (ResultKind::Pass, true));
        assert_eq!(p.tag, ResultTag::PatternOnly, "stub is pattern-only");

        let v = verdict_with_server(&off, "m", || stub(false));
        assert!(!v.ok && !v.persist_clear);
        let p = parse_result(&v.formatted);
        assert_eq!((p.kind, p.offline), (ResultKind::Fail, true));

        let v = verdict_with_server(&off, "unknown_mission", || None);
        assert!(!v.ok && !v.persist_clear);
        assert_eq!(parse_result(&v.formatted).kind, ResultKind::Error);
        assert!(v.formatted.contains("could not connect"));
    }

    #[test]
    fn refusals_are_not_counted_as_player_failures() {
        for o in [
            ApiOutcome::RateLimited {
                retry_after_secs: Some(3),
            },
            ApiOutcome::Busy {
                retry_after_secs: None,
            },
            ApiOutcome::Unauthorized,
            ApiOutcome::TooLarge,
        ] {
            let v = verdict_with_server(&o, "m", || panic!("stub must not run for {o:?}"));
            assert!(!v.ok && !v.persist_clear);
            assert_eq!(v.grade, None);
            assert_eq!(
                parse_result(&v.formatted).kind,
                ResultKind::Error,
                "{o:?} must not count toward the show-solution threshold"
            );
        }
        let v = verdict_with_server(
            &ApiOutcome::RateLimited {
                retry_after_secs: Some(3),
            },
            "m",
            || None,
        );
        assert!(v.formatted.contains("about 3 seconds"));
        let v = verdict_with_server(
            &ApiOutcome::Busy {
                retry_after_secs: None,
            },
            "m",
            || None,
        );
        assert!(v.formatted.contains("a few seconds"));
    }

    #[test]
    fn offline_demo_mode_saves_stub_passes_as_pattern_only() {
        let v = verdict_offline_mode("m", stub(true));
        assert!(v.ok && v.persist_clear);
        let p = parse_result(&v.formatted);
        assert_eq!((p.kind, p.offline), (ResultKind::Pass, false));
        assert_eq!(p.tag, ResultTag::PatternOnly);

        let v = verdict_offline_mode("m", stub(false));
        assert!(!v.ok && !v.persist_clear);

        let v = verdict_offline_mode("m", None);
        assert!(!v.ok && !v.persist_clear);
        assert_eq!(parse_result(&v.formatted).kind, ResultKind::Error);
    }
}
