// SPDX-License-Identifier: MIT
//! Hand-rolled Rust syntax highlighter, shared by the editor's code page,
//! the revealed solution and fenced code blocks in tutorials. Split into a
//! pure tokenizer (`tokenize`, unit-tested) and an egui `LayoutJob` builder
//! that maps each token role to a `SYNTAX_*` role of [`super`] (bible v2.1)
//! and underlines error lines with [`super::ERROR_MARK`].
//!
//! Hand-rolled because `egui_code_editor` 0.2 pins egui 0.34 while
//! `bevy_egui` 0.39 ships egui 0.33 (unfixable diamond as of 2026-05-02).

use super as theme;
use bevy_egui::egui;
use std::collections::BTreeSet;
use std::ops::Range;

/// What a span of source text is, for colouring purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenRole {
    Keyword,
    Type,
    Str,
    Comment,
    Number,
    Plain,
}

impl TokenRole {
    pub fn color(self) -> egui::Color32 {
        match self {
            TokenRole::Keyword => theme::SYNTAX_KEYWORD,
            TokenRole::Type => theme::SYNTAX_TYPE,
            TokenRole::Str => theme::SYNTAX_STRING,
            TokenRole::Comment => theme::SYNTAX_COMMENT,
            TokenRole::Number => theme::SYNTAX_NUMBER,
            TokenRole::Plain => theme::SYNTAX_PLAIN,
        }
    }
}

/// One coloured run. `range` is a byte range that always lies on UTF-8
/// char boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub range: Range<usize>,
    pub role: TokenRole,
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

/// Split `src` into coloured runs covering every byte exactly once.
pub fn tokenize(src: &str) -> Vec<Token> {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut out = Vec::new();
    let mut i = 0;

    while i < n {
        let b = bytes[i];
        let start = i;

        let role = if b == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            // Line comment to end of line ('\n' is ASCII, so the stop is
            // always a char boundary).
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            TokenRole::Comment
        } else if b == b'"' {
            // String literal with simple escape skipping. The scan only
            // stops on an ASCII '"' or at end of input, both boundaries.
            i += 1;
            while i < n && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < n {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            i = (i + 1).min(n);
            TokenRole::Str
        } else if b.is_ascii_digit() {
            while i < n
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
            {
                i += 1;
            }
            TokenRole::Number
        } else if b == b'_' || b.is_ascii_alphabetic() {
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &src[start..i];
            if KEYWORDS.contains(&word) {
                TokenRole::Keyword
            } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                TokenRole::Type
            } else {
                TokenRole::Plain
            }
        } else {
            // Whitespace, punctuation or a non-ASCII char: advance one
            // whole char. (The old byte-at-a-time step sliced through
            // multi-byte chars like `é` and panicked.)
            i += src[i..].chars().next().map_or(1, char::len_utf8);
            TokenRole::Plain
        };

        // Defensive: if a string escape skipped past the end, clamp.
        let end = i.min(n);
        out.push(Token {
            range: start..end,
            role,
        });
        i = end;
    }
    out
}

/// Build the egui layout job for the editor. `error_lines` holds
/// 1-based line numbers to underline in Alarm scarlet. No wrapping: the
/// gutter assumes one visual row per source line.
pub fn layout_job(
    src: &str,
    font_id: egui::FontId,
    error_lines: &BTreeSet<usize>,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mut line = 1usize;
    for token in tokenize(src) {
        // A token may span lines (multi-line string literal); emit one
        // section per line so per-line underlines stay accurate.
        let text = &src[token.range];
        let mut first = true;
        for piece in text.split('\n') {
            if !first {
                append(&mut job, "\n", &font_id, token.role, false);
                line += 1;
            }
            first = false;
            if !piece.is_empty() {
                append(
                    &mut job,
                    piece,
                    &font_id,
                    token.role,
                    error_lines.contains(&line),
                );
            }
        }
    }
    job.wrap.max_width = f32::INFINITY;
    job
}

fn append(
    job: &mut egui::text::LayoutJob,
    text: &str,
    font_id: &egui::FontId,
    role: TokenRole,
    error_line: bool,
) {
    let underline = if error_line {
        egui::Stroke::new(1.5_f32, theme::ERROR_MARK)
    } else {
        egui::Stroke::NONE
    };
    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id: font_id.clone(),
            color: role.color(),
            underline,
            ..Default::default()
        },
    );
}

/// Display-only copy of `src` for a narrow, read-only code view (the
/// revealed solution): every comment-only line longer than `max_cols`
/// chars is re-flowed onto several comment lines with the same indent and
/// comment marker. Lines with code on them (including code followed by a
/// trailing comment) are never touched, so the code a player reads is the
/// code they would copy. Copy actions must still use the ORIGINAL text.
pub fn wrap_comment_lines(src: &str, max_cols: usize) -> String {
    let mut out = String::with_capacity(src.len() + 16);
    for (i, line) in src.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        let body = match trimmed.strip_prefix("//") {
            Some(body) if line.chars().count() > max_cols => body,
            _ => {
                out.push_str(line);
                continue;
            }
        };
        let (marker, text) = if let Some(t) = body.strip_prefix('/') {
            ("///", t)
        } else if let Some(t) = body.strip_prefix('!') {
            ("//!", t)
        } else {
            ("//", body)
        };
        let prefix = format!("{indent}{marker} ");
        let width = max_cols.saturating_sub(prefix.chars().count()).max(16);
        let mut rows: Vec<String> = Vec::new();
        let mut cur = String::new();
        for word in text.split_whitespace() {
            if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > width {
                rows.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
        rows.push(cur);
        let joined: Vec<String> = rows.iter().map(|r| format!("{prefix}{r}")).collect();
        out.push_str(&joined.join("\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_comment_lines_reflows_only_long_comment_lines() {
        let src = "fn main() {\n    // `let` gives a value a name. The binding is immutable by default, always.\n    let answer = 42; // a trailing comment that is also rather long, but sits on code\n    println!(\"{answer}\");\n}\n";
        let shown = wrap_comment_lines(src, 40);
        // Code lines, including the one with a trailing comment, unchanged.
        for code_line in src.lines().filter(|l| !l.trim_start().starts_with("//")) {
            assert!(shown.lines().any(|l| l == code_line), "lost {code_line:?}");
        }
        // The long comment is now several `    // ` lines, each within width.
        let comment_rows: Vec<&str> = shown
            .lines()
            .filter(|l| l.trim_start().starts_with("//"))
            .collect();
        assert!(comment_rows.len() >= 2, "{shown}");
        for row in &comment_rows {
            assert!(row.starts_with("    // "), "{row:?}");
            assert!(row.chars().count() <= 40, "{row:?} too wide");
        }
        // Same words, same order.
        let words = |s: &str| -> Vec<String> {
            s.split_whitespace()
                .filter(|w| *w != "//")
                .map(str::to_string)
                .collect()
        };
        assert_eq!(words(&shown), words(src));
        // Trailing newline kept.
        assert!(shown.ends_with("}\n"));
    }

    #[test]
    fn wrap_comment_lines_is_identity_when_everything_fits() {
        let src = "// short\nfn main() {}\n";
        assert_eq!(wrap_comment_lines(src, 80), src);
        let doc = "/// a doc comment long enough to wrap at twenty columns";
        let shown = wrap_comment_lines(doc, 24);
        assert!(shown.lines().all(|l| l.starts_with("/// ")), "{shown}");
    }

    fn roles(src: &str) -> Vec<(&str, TokenRole)> {
        tokenize(src)
            .into_iter()
            .filter(|t| !src[t.range.clone()].trim().is_empty())
            .map(|t| (&src[t.range], t.role))
            .collect()
    }

    #[test]
    fn classifies_each_role() {
        let src = "fn main() { let x: Spell = 42; // hi\n \"s\" }";
        let r = roles(src);
        assert!(r.contains(&("fn", TokenRole::Keyword)));
        assert!(r.contains(&("let", TokenRole::Keyword)));
        assert!(r.contains(&("main", TokenRole::Plain)));
        assert!(r.contains(&("Spell", TokenRole::Type)));
        assert!(r.contains(&("42", TokenRole::Number)));
        assert!(r.contains(&("// hi", TokenRole::Comment)));
        assert!(r.contains(&("\"s\"", TokenRole::Str)));
    }

    #[test]
    fn tokens_cover_every_byte_exactly_once() {
        let src = "let s = \"a\\\"b\"; // c\nlet n = 1_000;";
        let toks = tokenize(src);
        let mut pos = 0;
        for t in &toks {
            assert_eq!(t.range.start, pos);
            pos = t.range.end;
        }
        assert_eq!(pos, src.len());
    }

    #[test]
    fn non_ascii_outside_strings_does_not_panic() {
        // Regression: byte-wise stepping used to slice inside `é`.
        let src = "let café = \"naïve\"; // ünïcode — ok\nlet λ = 1;";
        let toks = tokenize(src);
        let rebuilt: String = toks.iter().map(|t| &src[t.range.clone()]).collect();
        assert_eq!(rebuilt, src);
    }

    #[test]
    fn unterminated_string_runs_to_end() {
        let src = "let s = \"oops\\";
        let toks = tokenize(src);
        let last = toks.last().map(|t| (t.range.clone(), t.role));
        assert_eq!(last, Some((8..src.len(), TokenRole::Str)));
    }

    #[test]
    fn layout_job_underlines_only_error_lines() {
        let src = "let a = 1;\nlet b = \"x\ny\";\nlet c = 3;";
        let errs: BTreeSet<usize> = [3].into_iter().collect();
        let job = layout_job(src, egui::FontId::monospace(12.0), &errs);
        assert_eq!(job.text, src);
        for section in &job.sections {
            let text = &job.text[section.byte_range.clone()];
            let line = job.text[..section.byte_range.start].matches('\n').count() + 1;
            let underlined = section.format.underline != egui::Stroke::NONE;
            if text != "\n" {
                assert_eq!(underlined, line == 3, "section {text:?} on line {line}");
            }
        }
    }
}
