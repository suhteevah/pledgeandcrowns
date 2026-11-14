// SPDX-License-Identifier: MIT
//! A small markdown subset for mission tutorials in the goal pane:
//! `#`/`##`/`###` headings, paragraphs (consecutive lines joined),
//! `- ` / `* ` bullets, fenced code (```` ``` ````, optional language tag),
//! and inline `` `code` ``, `*emphasis*` and `**strong**`. Anything else is
//! plain text; an unmatched marker is kept literally.
//!
//! Parsing is pure (this file); drawing lives in `editor/widgets.rs`.
//! NOTE: a similar renderer is being written for mission panels elsewhere;
//! the two should be merged into one shared module later.

/// One inline run of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Span {
    Text(String),
    Code(String),
    Emph(String),
    Strong(String),
}

/// One block of a tutorial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph(Vec<Span>),
    Bullet(Vec<Span>),
    Code(String),
}

/// Parse a tutorial body into blocks.
pub fn parse(src: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let mut code: Option<Vec<&str>> = None;

    let flush_para = |para: &mut Vec<&str>, blocks: &mut Vec<Block>| {
        if !para.is_empty() {
            blocks.push(Block::Paragraph(spans(&para.join(" "))));
            para.clear();
        }
    };

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            match code.take() {
                Some(lines) => blocks.push(Block::Code(lines.join("\n"))),
                None => {
                    flush_para(&mut para, &mut blocks);
                    code = Some(Vec::new());
                }
            }
            continue;
        }
        if let Some(lines) = code.as_mut() {
            lines.push(line);
            continue;
        }
        if trimmed.is_empty() {
            flush_para(&mut para, &mut blocks);
            continue;
        }
        if let Some((level, text)) = heading(trimmed) {
            flush_para(&mut para, &mut blocks);
            blocks.push(Block::Heading {
                level,
                text: text.to_string(),
            });
            continue;
        }
        if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            flush_para(&mut para, &mut blocks);
            blocks.push(Block::Bullet(spans(item)));
            continue;
        }
        para.push(trimmed);
    }
    flush_para(&mut para, &mut blocks);
    if let Some(lines) = code {
        // Unterminated fence: still show it as code.
        blocks.push(Block::Code(lines.join("\n")));
    }
    blocks
}

fn heading(line: &str) -> Option<(u8, &str)> {
    let hashes = line.bytes().take_while(|b| *b == b'#').count();
    if !(1..=3).contains(&hashes) {
        return None;
    }
    let rest = line[hashes..].strip_prefix(' ')?;
    let level = u8::try_from(hashes).ok()?;
    Some((level, rest.trim()))
}

/// Split one line of text into inline spans.
pub fn spans(text: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        let found = match c {
            '`' => delimited(rest, "`").map(|(inner, after)| (Span::Code(inner.into()), after)),
            '*' if rest.starts_with("**") => {
                delimited(rest, "**").map(|(inner, after)| (Span::Strong(inner.into()), after))
            }
            '*' => delimited(rest, "*").map(|(inner, after)| (Span::Emph(inner.into()), after)),
            _ => None,
        };
        match found {
            Some((span, after)) => {
                if !plain.is_empty() {
                    out.push(Span::Text(std::mem::take(&mut plain)));
                }
                out.push(span);
                rest = after;
            }
            None => {
                plain.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    if !plain.is_empty() {
        out.push(Span::Text(plain));
    }
    out
}

/// `rest` starts with `delim`: return the text up to the closing `delim`
/// and what follows it. Empty or space-padded emphasis is not markup.
fn delimited<'a>(rest: &'a str, delim: &str) -> Option<(&'a str, &'a str)> {
    let body = rest.strip_prefix(delim)?;
    let end = body.find(delim)?;
    let inner = &body[..end];
    if inner.is_empty() {
        return None;
    }
    if delim != "`" && (inner.starts_with(' ') || inner.ends_with(' ')) {
        return None;
    }
    Some((inner, &body[end + delim.len()..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Span {
        Span::Text(s.into())
    }
    fn c(s: &str) -> Span {
        Span::Code(s.into())
    }

    #[test]
    fn real_tutorial_shape() {
        let src = "## Concept\nYou give a value a name with `let`. It is *immutable*.\n\n\
                   ## Syntax\n```\nlet name = value;\nlet name: Type = value;  // explicit\n```\n\
                   The annotation is **optional**.";
        let b = parse(src);
        assert_eq!(
            b,
            vec![
                Block::Heading {
                    level: 2,
                    text: "Concept".into()
                },
                Block::Paragraph(vec![
                    t("You give a value a name with "),
                    c("let"),
                    t(". It is "),
                    Span::Emph("immutable".into()),
                    t("."),
                ]),
                Block::Heading {
                    level: 2,
                    text: "Syntax".into()
                },
                Block::Code("let name = value;\nlet name: Type = value;  // explicit".into()),
                Block::Paragraph(vec![
                    t("The annotation is "),
                    Span::Strong("optional".into()),
                    t("."),
                ]),
            ]
        );
    }

    #[test]
    fn paragraph_lines_join_and_blank_lines_split() {
        let b = parse("one\ntwo\n\nthree");
        assert_eq!(
            b,
            vec![
                Block::Paragraph(vec![t("one two")]),
                Block::Paragraph(vec![t("three")]),
            ]
        );
    }

    #[test]
    fn fence_with_language_and_unterminated_fence() {
        assert_eq!(
            parse("```rust\nfn main() {}\n```"),
            vec![Block::Code("fn main() {}".into())]
        );
        assert_eq!(
            parse("```\nlet x = 1;"),
            vec![Block::Code("let x = 1;".into())]
        );
        // Markup inside a fence is literal, indentation kept.
        assert_eq!(
            parse("```\n    ## not a heading `x`\n```"),
            vec![Block::Code("    ## not a heading `x`".into())]
        );
    }

    #[test]
    fn bullets_and_headings_levels() {
        assert_eq!(
            parse("# A\n### C\n- one `x`\n* two"),
            vec![
                Block::Heading {
                    level: 1,
                    text: "A".into()
                },
                Block::Heading {
                    level: 3,
                    text: "C".into()
                },
                Block::Bullet(vec![t("one "), c("x")]),
                Block::Bullet(vec![t("two")]),
            ]
        );
        // Not headings.
        assert_eq!(
            parse("#### four"),
            vec![Block::Paragraph(vec![t("#### four")])]
        );
        assert_eq!(parse("#tag"), vec![Block::Paragraph(vec![t("#tag")])]);
    }

    #[test]
    fn unmatched_markers_stay_literal() {
        assert_eq!(spans("a ` b"), vec![t("a ` b")]);
        assert_eq!(spans("2 * 3 * 4"), vec![t("2 * 3 * 4")]);
        assert_eq!(spans("`*`"), vec![c("*")]);
        assert_eq!(spans("x * 2"), vec![t("x * 2")]);
        assert_eq!(spans("``"), vec![t("``")]);
        assert_eq!(spans("é `ü`"), vec![t("é "), c("ü")]);
    }
}
