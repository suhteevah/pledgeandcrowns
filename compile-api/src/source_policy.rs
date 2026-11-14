// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pre-build deny-list for player source: the *second* layer against
//! compile-time file reads.
//!
//! # What this is, and what it is not
//!
//! The real control against "player source makes rustc read a host file"
//! is the filesystem sandbox the build runs in (see
//! [`crate::build_sandbox`]): inside it rustc can only see the toolchain,
//! `/usr`, and the per-request job dir, so there is nothing secret to read.
//! This module is defence in depth for the case where that sandbox is
//! misconfigured, disabled for local dev (`PNC_BUILD_SANDBOX=none`), or
//! has a bug.
//!
//! It is a token-level scan, not a Rust parser and not macro expansion.
//! It strips comments and string/char literals with a small lexer that
//! follows the Rust lexical grammar closely enough for the constructs it
//! looks for, then refuses:
//!
//! - the identifiers `include`, `include_str`, `include_bytes` anywhere in
//!   code, in any position (a call, a path like `std::include_str`, a
//!   `use ... as alias`, a macro argument);
//! - inside any attribute (`#[...]` / `#![...]`): the identifier `path`
//!   (`#[path = ".."]`, also under `cfg_attr`), `debugger_visualizer`
//!   (embeds a named file), `link` (native library search), `feature`
//!   followed by `(` (`#![feature(..)]`, unstable features), and any `$`
//!   (a `macro_rules!` metavariable could otherwise smuggle `path` into an
//!   attribute from a macro argument);
//! - out-of-line modules (`mod name;`), which load a second source file.
//!   Only `src/main.rs` exists in a job dir, so these never compile anyway.
//!
//! What it can NOT guarantee:
//!
//! - It does not expand macros. It relies on the fact that, on stable Rust
//!   with no dependencies, a macro cannot mint the identifier
//!   `include_str` (or `path` in attribute position) without that token
//!   appearing literally in the source. `concat_idents!` and similar are
//!   unstable, and unstable features are refused here and fail on a stable
//!   toolchain anyway (`RUSTC_BOOTSTRAP` is never passed to the build).
//! - Any divergence between this lexer and rustc's is a potential bypass.
//!   Where the lexer is unsure (unterminated literal or comment) it scans
//!   the remainder as code, which can only cause false positives on source
//!   that would not compile anyway.
//! - It does not block `env!` / `option_env!`: they cannot be reliably
//!   caught at token level (`use std::env as e; e!(..)`), and the build
//!   runs with a cleared, allow-listed environment that holds no secrets.
//! - Known false positives: a variable named `include`, and
//!   `macro_rules!` patterns that forward attributes (`#[$m]`). Neither is
//!   needed by the curriculum.
//!
//! Names inside comments, doc comments and string literals are NOT
//! refused, because lessons legitimately talk about them.

use std::fmt;

/// Identifiers that read a file at compile time, refused anywhere in code.
const FILE_INCLUSION_IDENTS: &[&str] = &["include", "include_str", "include_bytes"];

/// Why a submission was refused before any build.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyViolation {
    #[error("compile-time file inclusion (`{name}`) is not allowed")]
    FileInclusion { name: String },
    #[error("module path redirection (`#[path]`) is not allowed")]
    ModulePathRedirect,
    #[error("out-of-line modules (`mod {name};`) are not allowed; put the module body inline")]
    OutOfLineModule { name: String },
    #[error("`#[debugger_visualizer]` embeds a file and is not allowed")]
    DebuggerVisualizer,
    #[error("native library linking (`#[link]`) is not allowed")]
    NativeLink,
    #[error("unstable features (`#![feature]`) are not allowed")]
    UnstableFeature,
    #[error("macro metavariables (`$`) inside attributes are not allowed")]
    MetavarInAttribute,
}

impl PolicyViolation {
    /// Stable label for logs.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::FileInclusion { .. } => "file_inclusion",
            Self::ModulePathRedirect => "module_path_redirect",
            Self::OutOfLineModule { .. } => "out_of_line_module",
            Self::DebuggerVisualizer => "debugger_visualizer",
            Self::NativeLink => "native_link",
            Self::UnstableFeature => "unstable_feature",
            Self::MetavarInAttribute => "metavar_in_attribute",
        }
    }

    /// The text shown to the player.
    pub fn player_message(&self) -> String {
        let headline = match self {
            Self::UnstableFeature => "this server does not allow unstable features",
            _ => "this server does not allow reading files at compile time",
        };
        format!("{headline} ({self})")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Ident(String),
    Punct(char),
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Ident(s) => f.write_str(s),
            Tok::Punct(c) => write!(f, "{c}"),
        }
    }
}

/// Check `source` against the deny-list. `Ok(())` means "no denied
/// construct found", NOT "safe" — see the module docs.
pub fn check(source: &str) -> Result<(), PolicyViolation> {
    let toks = lex(source);
    scan(&toks)
}

fn scan(toks: &[Tok]) -> Result<(), PolicyViolation> {
    let mut i = 0;
    while i < toks.len() {
        match &toks[i] {
            Tok::Ident(name) if FILE_INCLUSION_IDENTS.contains(&name.as_str()) => {
                return Err(PolicyViolation::FileInclusion { name: name.clone() });
            }
            Tok::Ident(kw) if kw == "mod" => {
                if let (Some(Tok::Ident(name)), Some(Tok::Punct(';'))) =
                    (toks.get(i + 1), toks.get(i + 2))
                {
                    return Err(PolicyViolation::OutOfLineModule { name: name.clone() });
                }
            }
            Tok::Punct('#') => {
                // `#[` or `#![` opens an attribute.
                let mut j = i + 1;
                if toks.get(j) == Some(&Tok::Punct('!')) {
                    j += 1;
                }
                if toks.get(j) == Some(&Tok::Punct('[')) {
                    let end = matching_close(toks, j);
                    check_attribute(&toks[j + 1..end])?;
                }
            }
            _ => {}
        }
        i += 1;
    }
    Ok(())
}

/// Index of the token closing the bracket at `open`, or `toks.len()` if
/// unbalanced (then the whole remainder is treated as attribute: strict).
fn matching_close(toks: &[Tok], open: usize) -> usize {
    let mut depth = 0usize;
    for (k, t) in toks.iter().enumerate().skip(open) {
        match t {
            Tok::Punct('[' | '(' | '{') => depth += 1,
            Tok::Punct(']' | ')' | '}') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return k;
                }
            }
            _ => {}
        }
    }
    toks.len()
}

fn check_attribute(body: &[Tok]) -> Result<(), PolicyViolation> {
    for (k, t) in body.iter().enumerate() {
        match t {
            Tok::Punct('$') => return Err(PolicyViolation::MetavarInAttribute),
            Tok::Ident(s) => match s.as_str() {
                "path" => return Err(PolicyViolation::ModulePathRedirect),
                "debugger_visualizer" => return Err(PolicyViolation::DebuggerVisualizer),
                "link" => return Err(PolicyViolation::NativeLink),
                "feature" if body.get(k + 1) == Some(&Tok::Punct('(')) => {
                    return Err(PolicyViolation::UnstableFeature);
                }
                _ => {}
            },
            Tok::Punct(_) => {}
        }
    }
    Ok(())
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// Strip comments and literals; return identifiers and punctuation.
/// Numbers and whitespace are dropped. Raw identifiers (`r#name`) are
/// normalised to `name`.
fn lex(src: &str) -> Vec<Tok> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let c = chars[i];
        // Comments.
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            match skip_block_comment(&chars, i) {
                Some(next) => i = next,
                // Unterminated: scan the rest as code (conservative).
                None => i += 2,
            }
            continue;
        }
        // Prefixed literals: b"..", br"..", r"..", r#".."#, c"..", cr"..",
        // b'x', and raw identifiers r#ident.
        if is_ident_start(c) {
            if let Some(next) = skip_prefixed_literal(&chars, i) {
                i = next;
                continue;
            }
            if c == 'r' && i + 2 < n && chars[i + 1] == '#' && is_ident_start(chars[i + 2]) {
                let start = i + 2;
                let mut j = start;
                while j < n && is_ident_continue(chars[j]) {
                    j += 1;
                }
                out.push(Tok::Ident(chars[start..j].iter().collect()));
                i = j;
                continue;
            }
            let start = i;
            while i < n && is_ident_continue(chars[i]) {
                i += 1;
            }
            out.push(Tok::Ident(chars[start..i].iter().collect()));
            continue;
        }
        if c.is_ascii_digit() {
            // Number literal, including suffixes like `1u8` / `0x1F`.
            while i < n && (is_ident_continue(chars[i]) || chars[i] == '.') {
                // Stop at `..` (range) so `0..include_str` still lexes the
                // identifier after it.
                if chars[i] == '.' && i + 1 < n && chars[i + 1] == '.' {
                    break;
                }
                i += 1;
            }
            continue;
        }
        if c == '"' {
            match skip_quoted(&chars, i + 1, '"') {
                Some(next) => i = next,
                None => i += 1,
            }
            continue;
        }
        if c == '\'' {
            match char_literal_end(&chars, i) {
                Some(next) => i = next,
                // Lifetime or label: drop the quote, lex what follows as code.
                None => i += 1,
            }
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        out.push(Tok::Punct(c));
        i += 1;
    }
    out
}

/// `i` points at `/*`. Rust block comments nest.
fn skip_block_comment(chars: &[char], mut i: usize) -> Option<usize> {
    let n = chars.len();
    let mut depth = 0usize;
    while i + 1 < n {
        if chars[i] == '/' && chars[i + 1] == '*' {
            depth += 1;
            i += 2;
        } else if chars[i] == '*' && chars[i + 1] == '/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return Some(i);
            }
        } else {
            i += 1;
        }
    }
    None
}

/// From just after an opening quote, skip to just after the matching
/// unescaped `close`. `None` if unterminated.
fn skip_quoted(chars: &[char], mut i: usize, close: char) -> Option<usize> {
    let n = chars.len();
    while i < n {
        match chars[i] {
            '\\' => i += 2,
            c if c == close => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

/// Raw string body starting at the `"` after `hashes` `#`s. Returns the
/// index just past the closing `"###`.
fn skip_raw_string(chars: &[char], quote: usize, hashes: usize) -> Option<usize> {
    let n = chars.len();
    let mut i = quote + 1;
    while i < n {
        if chars[i] == '"' {
            let mut k = 0;
            while k < hashes && i + 1 + k < n && chars[i + 1 + k] == '#' {
                k += 1;
            }
            if k == hashes {
                return Some(i + 1 + hashes);
            }
        }
        i += 1;
    }
    None
}

/// If a string/byte/char literal with a prefix (`b`, `br`, `r`, `c`, `cr`)
/// starts at `i`, return the index after it. Raw identifiers are not
/// literals and return `None`.
fn skip_prefixed_literal(chars: &[char], i: usize) -> Option<usize> {
    let n = chars.len();
    let at = |k: usize| chars.get(k).copied();
    // Prefix must not be the tail of a longer identifier: the caller only
    // calls us at an identifier start, so `chars[i]` begins a word.
    let (raw_at, plain_at) = match (at(i), at(i + 1)) {
        (Some('b'), Some('r')) | (Some('c'), Some('r')) => (Some(i + 2), None),
        (Some('r'), _) => (Some(i + 1), None),
        (Some('b'), Some('"')) | (Some('c'), Some('"')) => (None, Some(i + 1)),
        (Some('b'), Some('\'')) => {
            return char_literal_end(chars, i + 1);
        }
        _ => (None, None),
    };
    if let Some(q) = plain_at {
        return Some(skip_quoted(chars, q + 1, '"').unwrap_or(n));
    }
    if let Some(mut k) = raw_at {
        let mut hashes = 0;
        while at(k) == Some('#') {
            hashes += 1;
            k += 1;
        }
        if at(k) == Some('"') {
            // Unterminated raw string: treat the rest as code.
            return Some(skip_raw_string(chars, k, hashes).unwrap_or(k + 1));
        }
    }
    None
}

/// `i` points at `'`. If this is a char literal (`'x'`, `'\n'`,
/// `'\u{1F600}'`, `'\''`), return the index after its closing quote.
/// Otherwise (a lifetime or loop label) return `None`.
fn char_literal_end(chars: &[char], i: usize) -> Option<usize> {
    let n = chars.len();
    let next = *chars.get(i + 1)?;
    if next == '\\' {
        // Escaped char: closing quote within a short window.
        let mut k = i + 3;
        while k < n && k <= i + 12 {
            if chars[k] == '\'' {
                return Some(k + 1);
            }
            k += 1;
        }
        return None;
    }
    if next != '\'' && chars.get(i + 2) == Some(&'\'') {
        return Some(i + 3);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn denied(src: &str) -> PolicyViolation {
        check(src).expect_err(src)
    }

    #[test]
    fn benign_programs_pass() {
        for src in [
            "fn main() { let answer = 42; }",
            "fn main() { let path = String::from(\"a/b\"); println!(\"{path}\"); }",
            "fn f<'a>(x: &'a str) -> &'a str { x }",
            "fn main() { 'outer: loop { break 'outer; } }",
            "fn main() { let c = '\\''; let d = 'x'; let e = '\\u{1F600}'; }",
            "#[derive(Debug, Clone)] struct S { path: u8 }",
            "#[cfg(feature = \"x\")] fn gated() {}",
            "#[allow(clippy::path_buf_push_overwrite)] fn f() {}",
            "mod inline { pub fn f() {} } fn main() { inline::f(); }",
            "fn main() { let args = std::env::args(); for i in 0..10 { let _ = i; } }",
            "macro_rules! sq { ($x:expr) => { $x * $x }; } fn main() { let _ = sq!(3); }",
        ] {
            assert_eq!(check(src), Ok(()), "{src}");
        }
    }

    #[test]
    fn names_in_comments_and_strings_are_not_refused() {
        for src in [
            "// include_str!(\"/etc/passwd\")\nfn main() {}",
            "/* #[path = \"/etc/x\"] mod m; */ fn main() {}",
            "/* nested /* include_bytes!(\"x\") */ still comment */ fn main() {}",
            "/// Use `include_str!` to embed a file.\nfn main() {}",
            "//! #![feature(never_type)] is nightly-only\nfn main() {}",
            "fn main() { println!(\"include_str!(\\\"/etc/passwd\\\")\"); }",
            "fn main() { let s = r#\"#[path = \"x\"] mod y; include!(\"z\")\"#; }",
            "fn main() { let b = b\"include_bytes\"; let c = br##\"include\"##; }",
            "fn main() { let s = c\"include_str\"; }",
            "#[doc = \"see include_str and #[path]\"] fn main() {}",
        ] {
            assert_eq!(check(src), Ok(()), "{src}");
        }
    }

    #[test]
    fn file_inclusion_forms_are_refused() {
        for src in [
            "fn main() { print!(\"{}\", include_str!(\"/etc/hostname\")); }",
            "fn main() { let b = include_bytes!(\"/etc/hostname\"); }",
            "include!(\"/etc/hostname\");",
            "fn main() { let s = std::include_str!(\"x\"); }",
            "fn main() { let s = core :: include_bytes ! (\"x\"); }",
            "use std::include_str as grab; fn main() { grab!(\"x\"); }",
            "macro_rules! m { ($i:ident) => { $i!(\"x\") } } fn main() { m!(include_str); }",
            "fn main() { let s = r#include_str!(\"x\"); }",
            "#[doc = include_str!(\"x\")] fn main() {}",
            "fn main() { for _ in 0..include_bytes!(\"x\").len() {} }",
        ] {
            assert!(
                matches!(denied(src), PolicyViolation::FileInclusion { .. }),
                "{src}"
            );
        }
    }

    #[test]
    fn module_path_redirection_is_refused() {
        for src in [
            "#[path = \"/etc/hostname\"] mod m; fn main() {}",
            "#[path=\"/etc/x\"]\nmod m {}",
            "#[cfg_attr(all(), path = \"/etc/x\")] mod m; fn main() {}",
            "#[ r#path = \"x\" ] mod m {}",
        ] {
            assert_eq!(denied(src), PolicyViolation::ModulePathRedirect, "{src}");
        }
        assert!(matches!(
            denied("mod secrets; fn main() {}"),
            PolicyViolation::OutOfLineModule { .. }
        ));
    }

    #[test]
    fn other_attribute_vectors_are_refused() {
        assert_eq!(
            denied("#![debugger_visualizer(gdb_script_file = \"/etc/x\")] fn main() {}"),
            PolicyViolation::DebuggerVisualizer
        );
        assert_eq!(
            denied("#[link(name = \"x\")] extern \"C\" {} fn main() {}"),
            PolicyViolation::NativeLink
        );
        assert_eq!(
            denied("#![feature(never_type)] fn main() {}"),
            PolicyViolation::UnstableFeature
        );
        assert_eq!(
            denied("#![cfg_attr(all(), feature(never_type))] fn main() {}"),
            PolicyViolation::UnstableFeature
        );
        assert_eq!(
            denied("macro_rules! m { ($a:ident) => { #[$a = \"/etc/x\"] mod q; } } m!(pth);"),
            PolicyViolation::MetavarInAttribute
        );
    }

    #[test]
    fn unterminated_literals_fail_closed() {
        // An unterminated string hides nothing: the rest is scanned.
        assert!(check("fn main() { let s = \"oops include_str!(1) }").is_err());
        assert!(check("fn main() { let s = r#\"oops include_str!(1) }").is_err());
        assert!(check("/* unterminated include_str!(\"x\")").is_err());
    }

    #[test]
    fn player_message_names_the_policy() {
        let v = denied("fn main() { include_str!(\"x\"); }");
        assert!(
            v.player_message()
                .starts_with("this server does not allow reading files at compile time"),
            "{}",
            v.player_message()
        );
        assert_eq!(v.kind(), "file_inclusion");
    }
}
