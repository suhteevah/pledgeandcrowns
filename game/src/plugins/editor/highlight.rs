// SPDX-License-Identifier: MIT
//! Hand-rolled Rust syntax highlighter. Split into a pure tokenizer
//! (`tokenize`, unit-tested) and an egui `LayoutJob` builder that maps
//! each token role to a colour from [`super::palette`] and underlines
//! error lines in Alarm scarlet.
//!
//! Hand-rolled because `egui_code_editor` 0.2 pins egui 0.34 while
//! `bevy_egui` 0.39 ships egui 0.33 (unfixable diamond as of 2026-05-02).

use super::palette;
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
            TokenRole::Keyword => palette::KEYWORD,
            TokenRole::Type => palette::TYPE,
            TokenRole::Str => palette::STRING,
            TokenRole::Comment => palette::COMMENT,
            TokenRole::Number => palette::NUMBER,
            TokenRole::Plain => palette::PLAIN,
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
        egui::Stroke::new(1.5_f32, palette::ERROR)
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

#[cfg(test)]
mod tests {
    use super::*;

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
