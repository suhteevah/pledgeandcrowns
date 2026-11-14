// SPDX-License-Identifier: MIT
//! Pure mapping from an HTTP exchange to what the player sees, and the
//! rule for which passes are saved as clears. Shared by native and web;
//! no I/O here, so every branch is unit-tested.
//!
//! # Fallback and clear policy
//!
//! | Situation                                   | Feedback shown            | Pass saved? |
//! |---------------------------------------------|---------------------------|-------------|
//! | Server graded the code (200, or 400 body)   | server verdict            | yes         |
//! | Server configured but unreachable / errored | offline stub grader       | **no**      |
//! | 429 rate limited / 503 build queue full     | "wait N seconds" message  | no          |
//! | 401 unauthorized / 413 too large            | plain explanation         | no          |
//! | No server configured on purpose (demo mode) | stub grader, no banner    | yes         |
//!
//! "Configured but misconfigured" (bad URL/token in the config) is handled
//! by the caller as `Offline`, i.e. stub feedback that is not saved.
//!
//! The strings produced keep the editor's contract
//! (`editor/diagnostics.rs::parse_result`): `[ok] ...` / `[fail] ...` for
//! verdicts, an optional leading `[offline mode ...]` banner line for stub
//! fallback, and any other leading tag for "not the player's fault"
//! errors, which never count as failed attempts. Server text is passed
//! through unchanged so rustc line numbers still map via
//! `SERVER_PREAMBLE_LINES`.

use serde::Deserialize;

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

/// Classified result of one compile request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiOutcome {
    /// The server graded the code. `message` is the editor-ready string.
    Graded { ok: bool, message: String },
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

/// The server's `{ok, stdout, stderr}` body (also the base of its
/// rejection bodies, which add `error` / `retry_after_seconds`).
#[derive(Deserialize, Debug)]
struct CompileResponse {
    ok: bool,
    stdout: String,
    stderr: String,
    #[serde(default)]
    retry_after_seconds: Option<u64>,
}

/// Editor-ready text for a server verdict. Unchanged from the original
/// client so the editor's parsing and line mapping still hold.
pub fn format_server_verdict(ok: bool, stdout: &str, stderr: &str) -> String {
    let tag = if ok { "ok" } else { "fail" };
    let mut out = format!("[{tag}] {stdout}");
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

/// Map a transport result to an [`ApiOutcome`]. Pure.
pub fn classify(result: Result<HttpReply, TransportError>) -> ApiOutcome {
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
            Some(p) => ApiOutcome::Graded {
                ok: p.ok && reply.status == 200,
                message: format_server_verdict(p.ok && reply.status == 200, &p.stdout, &p.stderr),
            },
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

/// What the main loop applies: text for the editor, pass/fail, and
/// whether a pass is saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub formatted: String,
    pub ok: bool,
    pub persist_clear: bool,
}

fn wait_phrase(secs: Option<u64>) -> String {
    match secs {
        Some(1) => "about 1 second".to_string(),
        Some(s) if s > 1 => format!("about {s} seconds"),
        _ => "a few seconds".to_string(),
    }
}

fn stub_text(v: &StubVerdict) -> String {
    let tag = if v.ok { "ok" } else { "fail" };
    let body = if v.ok { &v.stdout } else { &v.stderr };
    let mut out = format!("[{tag}] {body}");
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
    let refused = |formatted: String| Verdict {
        formatted,
        ok: false,
        persist_clear: false,
    };
    match outcome {
        ApiOutcome::Graded { ok, message } => Verdict {
            formatted: message.clone(),
            ok: *ok,
            persist_clear: *ok,
        },
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
                Verdict {
                    formatted,
                    ok: v.ok,
                    persist_clear: false,
                }
            }
            None => refused(format!(
                "[client error] Couldn't reach the compile server ({reason}), and there \
                 is no offline check for `{encounter_id}`. Try again in a moment."
            )),
        },
    }
}

/// Verdict in offline demo mode (no server configured on purpose): the
/// stub grader is canonical, so its passes are saved. No banner.
pub fn verdict_offline_mode(encounter_id: &str, stub: Option<StubVerdict>) -> Verdict {
    match stub {
        Some(v) => Verdict {
            formatted: stub_text(&v),
            ok: v.ok,
            persist_clear: v.ok,
        },
        None => Verdict {
            formatted: format!("[offline] no offline grader for encounter `{encounter_id}`"),
            ok: false,
            persist_clear: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::editor::diagnostics::{ResultKind, parse_result};

    fn reply(status: u16, retry: Option<&str>, body: &str) -> Result<HttpReply, TransportError> {
        Ok(HttpReply {
            status,
            retry_after: retry.map(str::to_string),
            body: body.to_string(),
        })
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

    #[test]
    fn graded_pass_and_fail() {
        let o = classify(reply(200, None, r#"{"ok":true,"stdout":"42","stderr":""}"#));
        assert_eq!(
            o,
            ApiOutcome::Graded {
                ok: true,
                message: "[ok] 42".into()
            }
        );
        let o = classify(reply(
            200,
            None,
            r#"{"ok":false,"stdout":"[real]","stderr":"error[E0384]: x\n --> src/main.rs:4:5"}"#,
        ));
        let ApiOutcome::Graded { ok, message } = o else {
            panic!("expected graded")
        };
        assert!(!ok);
        // Server text passes through byte for byte.
        assert_eq!(
            message,
            "[fail] [real]\n--stderr--\nerror[E0384]: x\n --> src/main.rs:4:5"
        );
    }

    #[test]
    fn bad_request_with_body_is_a_failed_verdict() {
        let o = classify(reply(
            400,
            None,
            r#"{"ok":false,"stdout":"","stderr":"empty source"}"#,
        ));
        assert!(matches!(o, ApiOutcome::Graded { ok: false, .. }));
    }

    #[test]
    fn unreadable_success_body_is_offline() {
        assert!(matches!(
            classify(reply(200, None, "<html>proxy page</html>")),
            ApiOutcome::Offline { .. }
        ));
    }

    #[test]
    fn rate_limited_reads_header_then_body() {
        assert_eq!(
            classify(reply(429, Some("12"), REJECT_429)),
            ApiOutcome::RateLimited {
                retry_after_secs: Some(12)
            }
        );
        // HTTP-date header is not parsed; fall back to the body field.
        assert_eq!(
            classify(reply(
                429,
                Some("Wed, 21 Oct 2026 07:28:00 GMT"),
                REJECT_429
            )),
            ApiOutcome::RateLimited {
                retry_after_secs: Some(7)
            }
        );
        assert_eq!(
            classify(reply(429, None, "")),
            ApiOutcome::RateLimited {
                retry_after_secs: None
            }
        );
        assert_eq!(
            classify(reply(429, Some("999999"), "")),
            ApiOutcome::RateLimited {
                retry_after_secs: Some(MAX_RETRY_AFTER_SECS)
            }
        );
    }

    #[test]
    fn busy_unauthorized_too_large() {
        assert_eq!(
            classify(reply(503, Some("5"), "")),
            ApiOutcome::Busy {
                retry_after_secs: Some(5)
            }
        );
        assert_eq!(classify(reply(401, None, "")), ApiOutcome::Unauthorized);
        assert_eq!(classify(reply(413, None, "")), ApiOutcome::TooLarge);
    }

    #[test]
    fn other_statuses_and_transport_errors_are_offline() {
        for s in [404, 405, 415, 422, 500, 502, 504] {
            assert!(
                matches!(classify(reply(s, None, "{}")), ApiOutcome::Offline { .. }),
                "{s}"
            );
        }
        for e in [
            TransportError::Timeout(10),
            TransportError::Connect,
            TransportError::Other("dns".into()),
        ] {
            assert!(matches!(classify(Err(e)), ApiOutcome::Offline { .. }));
        }
    }

    #[test]
    fn server_pass_is_saved_server_fail_is_not() {
        let pass = ApiOutcome::Graded {
            ok: true,
            message: "[ok] hi".into(),
        };
        let v = verdict_with_server(&pass, "m", || panic!("stub must not run"));
        assert!(v.ok && v.persist_clear);
        assert_eq!(parse_result(&v.formatted).kind, ResultKind::Pass);

        let fail = ApiOutcome::Graded {
            ok: false,
            message: "[fail] no".into(),
        };
        let v = verdict_with_server(&fail, "m", || panic!("stub must not run"));
        assert!(!v.ok && !v.persist_clear);
        assert_eq!(parse_result(&v.formatted).kind, ResultKind::Fail);
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
    fn offline_demo_mode_saves_stub_passes() {
        let v = verdict_offline_mode("m", stub(true));
        assert!(v.ok && v.persist_clear);
        let p = parse_result(&v.formatted);
        assert_eq!((p.kind, p.offline), (ResultKind::Pass, false));

        let v = verdict_offline_mode("m", stub(false));
        assert!(!v.ok && !v.persist_clear);

        let v = verdict_offline_mode("m", None);
        assert!(!v.ok && !v.persist_clear);
        assert_eq!(parse_result(&v.formatted).kind, ResultKind::Error);
    }
}
