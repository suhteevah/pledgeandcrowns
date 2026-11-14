// SPDX-License-Identifier: MIT
//! Renderer for the markdown subset the mission tutorials and prompts use.
//!
//! Surveyed across all 57 missions: `## Heading` lines, fenced code
//! blocks (bare ```` ``` ````, no language tag), inline `` `code` ``,
//! `*emphasis*` and `**strong**`. No lists, tables, links or nested
//! markup. Anything else is shown as plain text, never dropped.
//!
//! [`parse`] is pure and unit-tested on the real tutorial strings;
//! [`show`] draws the result with the theme: headings in Oxblood, inline
//! code as monospace on an Aged paper chip, code blocks in an Aged paper
//! box.

use bevy_egui::egui;

use super::{
    BASALT, BODY_SIZE, CODE_CHIP, MONO_SIZE, SUBHEADING_SIZE, TEXT, TEXT_HEADING, WINEFLESH,
};

/// Inline run kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    Text,
    Code,
    Emphasis,
    Strong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub kind: SpanKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// `## Concept` → `Heading("Concept")`.
    Heading(String),
    /// Consecutive non-blank lines, joined with spaces.
    Paragraph(Vec<Span>),
    /// The lines between two fences, verbatim.
    Code(String),
}

fn span(kind: SpanKind, text: &str) -> Span {
    Span {
        kind,
        text: text.to_string(),
    }
}

/// Split one paragraph into inline spans. Code spans win over emphasis
/// (`` `**x` `` is code), an unmatched delimiter is literal text.
pub fn parse_inline(src: &str) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    let mut text = String::new();
    let mut rest = src;
    let flush = |text: &mut String, out: &mut Vec<Span>| {
        if !text.is_empty() {
            out.push(span(SpanKind::Text, text));
            text.clear();
        }
    };
    while let Some(c) = rest.chars().next() {
        let matched = if c == '`' {
            rest[1..].find('`').map(|end| (SpanKind::Code, 1, end, 1))
        } else if let Some(after) = rest.strip_prefix("**") {
            after
                .find("**")
                .filter(|&end| end > 0)
                .map(|end| (SpanKind::Strong, 2, end, 2))
        } else if c == '*' && rest[1..].starts_with(|n: char| !n.is_whitespace() && n != '*') {
            rest[1..]
                .find('*')
                .filter(|&end| !rest[1..1 + end].ends_with(char::is_whitespace))
                .map(|end| (SpanKind::Emphasis, 1, end, 1))
        } else {
            None
        };
        match matched {
            Some((kind, open, len, close)) => {
                flush(&mut text, &mut out);
                out.push(span(kind, &rest[open..open + len]));
                rest = &rest[open + len + close..];
            }
            None => {
                text.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    flush(&mut text, &mut out);
    out
}

/// Parse tutorial/prompt text into blocks.
pub fn parse(src: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let mut code: Option<Vec<&str>> = None;
    let end_para = |para: &mut Vec<&str>, blocks: &mut Vec<Block>| {
        if !para.is_empty() {
            blocks.push(Block::Paragraph(parse_inline(&para.join(" "))));
            para.clear();
        }
    };
    for line in src.lines() {
        if let Some(lines) = code.as_mut() {
            if line.trim_start().starts_with("```") {
                blocks.push(Block::Code(lines.join("\n")));
                code = None;
            } else {
                lines.push(line);
            }
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            end_para(&mut para, &mut blocks);
            code = Some(Vec::new());
        } else if trimmed.starts_with('#') && trimmed.trim_start_matches('#').starts_with(' ') {
            end_para(&mut para, &mut blocks);
            blocks.push(Block::Heading(
                trimmed.trim_start_matches('#').trim().to_string(),
            ));
        } else if trimmed.is_empty() {
            end_para(&mut para, &mut blocks);
        } else {
            para.push(trimmed);
        }
    }
    // An unclosed fence still shows its code.
    if let Some(lines) = code {
        blocks.push(Block::Code(lines.join("\n")));
    }
    end_para(&mut para, &mut blocks);
    blocks
}

/// Lay out one paragraph's spans as an egui text job.
pub fn inline_job(spans: &[Span], size: f32, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let body = egui::FontId::proportional(size);
    for s in spans {
        let format = match s.kind {
            SpanKind::Text => egui::TextFormat::simple(body.clone(), TEXT),
            SpanKind::Code => egui::TextFormat {
                font_id: egui::FontId::monospace(MONO_SIZE.min(size)),
                color: TEXT,
                background: CODE_CHIP,
                expand_bg: 1.0,
                ..Default::default()
            },
            SpanKind::Emphasis => egui::TextFormat {
                font_id: body.clone(),
                color: TEXT,
                italics: true,
                ..Default::default()
            },
            SpanKind::Strong => egui::TextFormat::simple(body.clone(), WINEFLESH),
        };
        job.append(&s.text, 0.0, format);
    }
    job
}

/// Draw a single line of inline markdown (prompts).
pub fn show_inline(ui: &mut egui::Ui, src: &str, size: f32) -> egui::Response {
    let job = inline_job(&parse_inline(src), size, ui.available_width());
    ui.label(job)
}

/// Draw parsed blocks at body size.
pub fn show(ui: &mut egui::Ui, src: &str) {
    let blocks = parse(src);
    tracing::trace!(blocks = blocks.len(), "markdown::show");
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            ui.add_space(4.0);
        }
        match block {
            Block::Heading(h) => {
                ui.label(
                    egui::RichText::new(h)
                        .size(SUBHEADING_SIZE)
                        .color(TEXT_HEADING)
                        .strong(),
                );
            }
            Block::Paragraph(spans) => {
                ui.label(inline_job(spans, BODY_SIZE, ui.available_width()));
            }
            Block::Code(code) => {
                egui::Frame::new()
                    .fill(CODE_CHIP)
                    .stroke(egui::Stroke::new(1.0_f32, BASALT))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            egui::RichText::new(code)
                                .monospace()
                                .size(MONO_SIZE)
                                .color(TEXT),
                        );
                    });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::mission::MissionRegistry;

    fn kinds(spans: &[Span]) -> Vec<(SpanKind, &str)> {
        spans.iter().map(|s| (s.kind, s.text.as_str())).collect()
    }

    #[test]
    fn inline_code_emphasis_and_strong() {
        let got = parse_inline("Use `let` to *bind*, **always**.");
        assert_eq!(
            kinds(&got),
            vec![
                (SpanKind::Text, "Use "),
                (SpanKind::Code, "let"),
                (SpanKind::Text, " to "),
                (SpanKind::Emphasis, "bind"),
                (SpanKind::Text, ", "),
                (SpanKind::Strong, "always"),
                (SpanKind::Text, "."),
            ]
        );
    }

    #[test]
    fn code_wins_over_stars_and_lone_stars_are_text() {
        let got = parse_inline("deref twice (`**x`) then x * 2 and a*b");
        assert_eq!(
            kinds(&got),
            vec![
                (SpanKind::Text, "deref twice ("),
                (SpanKind::Code, "**x"),
                (SpanKind::Text, ") then x * 2 and a*b"),
            ]
        );
    }

    #[test]
    fn unmatched_backtick_is_literal() {
        let got = parse_inline("a ` b");
        assert_eq!(kinds(&got), vec![(SpanKind::Text, "a ` b")]);
    }

    #[test]
    fn first_tutorial_parses_into_the_expected_blocks() {
        let reg = MissionRegistry::default();
        let Some(m) = reg.find("intro_let_binding") else {
            panic!("intro_let_binding missing");
        };
        let blocks = parse(m.tutorial);
        let headings: Vec<&str> = blocks
            .iter()
            .filter_map(|b| match b {
                Block::Heading(h) => Some(h.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(headings, vec!["Concept", "Syntax", "Task", "Hint"]);
        assert_eq!(
            blocks.get(1).map(|b| matches!(b, Block::Paragraph(s) if s.iter().any(|s| s.kind == SpanKind::Code && s.text == "let"))),
            Some(true),
            "concept paragraph keeps `let` as inline code: {blocks:?}"
        );
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, Block::Code(c) if c.starts_with("let name = value;"))),
            "syntax fence becomes a code block: {blocks:?}"
        );
    }

    #[test]
    fn async_fn_tutorial_keeps_strong_and_emphasis() {
        let reg = MissionRegistry::default();
        let Some(m) = reg.find("async_fn") else {
            panic!("async_fn missing");
        };
        let spans: Vec<Span> = parse(m.tutorial)
            .into_iter()
            .filter_map(|b| match b {
                Block::Paragraph(s) => Some(s),
                _ => None,
            })
            .flatten()
            .collect();
        assert!(
            spans
                .iter()
                .any(|s| s.kind == SpanKind::Strong && s.text == "Note:")
        );
        assert!(
            spans
                .iter()
                .any(|s| s.kind == SpanKind::Emphasis && s.text == "executor")
        );
    }

    /// No markup leaks into rendered text for any tutorial or prompt:
    /// every heading is recognised, every fence closes, no stray `##`,
    /// fence or backtick survives in a text span.
    #[test]
    fn every_tutorial_and_prompt_renders_without_raw_markup() {
        let reg = MissionRegistry::default();
        assert!(reg.missions.len() >= 57);
        for m in &reg.missions {
            let fences = m
                .tutorial
                .lines()
                .filter(|l| l.trim().starts_with("```"))
                .count();
            assert_eq!(fences % 2, 0, "{}: unbalanced fences", m.id);
            let blocks = parse(m.tutorial);
            let headings = blocks
                .iter()
                .filter(|b| matches!(b, Block::Heading(_)))
                .count();
            assert!(headings >= 3, "{}: only {headings} headings", m.id);
            for b in &blocks {
                if let Block::Paragraph(spans) = b {
                    for s in spans.iter().filter(|s| s.kind == SpanKind::Text) {
                        assert!(
                            !s.text.contains('`'),
                            "{}: raw backtick in {:?}",
                            m.id,
                            s.text
                        );
                        assert!(
                            !s.text.contains("##"),
                            "{}: raw heading in {:?}",
                            m.id,
                            s.text
                        );
                        assert!(
                            !s.text.contains("**"),
                            "{}: raw strong in {:?}",
                            m.id,
                            s.text
                        );
                    }
                }
            }
            for s in parse_inline(m.prompt)
                .iter()
                .filter(|s| s.kind == SpanKind::Text)
            {
                assert!(
                    !s.text.contains('`'),
                    "{}: raw backtick in prompt {:?}",
                    m.id,
                    s.text
                );
            }
        }
    }
}
