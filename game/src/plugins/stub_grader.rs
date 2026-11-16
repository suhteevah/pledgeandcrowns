// SPDX-License-Identifier: MIT
//! Offline fallback grader for the in-game compile client.
//!
//! When the live `compile-api` server at 127.0.0.1:7878 is unreachable
//! (Hetzner hiccup, demo machine without server, kiosk mode, etc.) the
//! game must not show a dead-end `[client error]`. Instead, this module
//! re-implements the same pattern checks the server-side grader uses,
//! returning a `[stub]`-prefixed verdict so the player can still feel
//! progression while knowing the grade isn't authoritative. On wasm it
//! is the ONLY grader.
//!
//! IMPORTANT: this MUST NOT add a runtime dependency on
//! `pledge_compile_api` (MIT game vs AGPL server). The matching engine and
//! the rule table are duplicated on purpose and kept TEXTUALLY IDENTICAL
//! to `compile-api/src/grader.rs` between the `BEGIN GRADER ENGINE` /
//! `END GRADER ENGINE` and `BEGIN MISSION TABLE` / `END MISSION TABLE`
//! markers. `game/tests/contract.rs` enforces both the textual parity and
//! byte-for-byte output parity.
//!
//! Hard rule: a stub pass is NEVER persisted as a real mission clear.
//! The caller in `compile_client.rs` is responsible for gating
//! `MissionProgress::mark_cleared` on a "this came from the live API"
//! flag — see `CompileOutcome::from_stub`.
//!
//! # Adding a mission
//!
//! Add ONE [`MissionRule`] entry to [`MISSION_RULES`] here, and the
//! identical entry to `MISSION_RULES` in `compile-api/src/grader.rs`:
//!
//! ```text
//! MissionRule {
//!     id: "mut_binding",                  // MissionRegistry encounter id
//!     npc: "the Smith eyes you",          // fail = "{npc} — {hint}"
//!     checks: &[
//!         // Checked in order; the first check none of whose `any_of`
//!         // patterns occurs in the code fails with its hint.
//!         check(&["let mut"], "missing required: `let mut`"),
//!         check(&["+= 1"], "missing required: `+= 1`"),
//!         // check_in_strings(..) also searches string literals.
//!     ],
//!     pass: "the Smith strikes the work. \"good — that one bends.\"",
//! },
//! ```
//!
//! # How a check matches
//!
//! The source is lexed first ([`lex`]): comments (line, doc and nested
//! block) vanish, and every string, raw string, byte string and char
//! literal becomes ONE token, so code pasted inside a comment or a string
//! is not code. Lifetimes (`'a`) are told apart from char literals.
//!
//! Each pattern in `any_of` is lexed the same way and matched as a
//! contiguous run of tokens, so whitespace never matters: `"*= 2"` matches
//! `n *= 2`, `n*=2` and `n *=\n 2`. A string literal in a pattern matches a
//! string literal with the same contents (`"\"neg\""`). Plain checks look
//! only at code; a check built with [`check_in_strings`] also looks inside
//! string literals, which is how `{:?}` in a format string is found.
//!
//! Pattern wildcards (`$` never appears in a normal program outside
//! `macro_rules!`, so it is free to use):
//!
//! | pattern | matches |
//! |---|---|
//! | `$_` | any one identifier |
//! | `$X` (any name) | any one identifier; every `$X` in the same pattern must be the same identifier |
//! | `$'_` | any one lifetime |
//! | `$'a` (any name) | any one lifetime; repeats must be the same lifetime |
//! | `$..` | any run of tokens, possibly empty, that stays inside the bracket group it starts in |
//!
//! So `"|$X $.., $Y $..| $.. $X + $Y"` accepts `|a, b| a + b` and
//! `|x: i32, y: i32| -> i32 { x + y }`, and `"<$'a $.. &$'a"` accepts any
//! lifetime name used consistently. A pattern grader checks shape, never
//! behaviour: real compilation with an output check is the only thing that
//! proves a program correct. If a future mission genuinely needs logic the
//! table cannot express, give it a named custom fn rather than contorting
//! the table.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StubVerdict {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl StubVerdict {
    fn pass(msg: impl Into<String>) -> Self {
        Self {
            ok: true,
            stdout: format!("[stub] {}", msg.into()),
            stderr: String::new(),
        }
    }
    fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            stdout: String::new(),
            stderr: format!("[stub] {}", msg.into()),
        }
    }
}

/// Re-implementation of the server's per-encounter pattern checks.
///
/// Returns `Some(verdict)` for any mission currently shipped in
/// `MissionRegistry::default()`, or `None` for an unknown encounter id
/// (the caller falls back to the existing `[client error]` path so
/// unexpected ids aren't silently graded as a freeform pass). The
/// `stub_covers_every_mission_in_the_registry` test in `tests/stub_grader.rs`
/// will scream if a new mission is added without a table entry here.
pub fn stub_verdict(encounter_id: &str, source: &str) -> Option<StubVerdict> {
    let Some(rule) = mission_rule(encounter_id) else {
        tracing::debug!(encounter_id, "stub grader: unknown encounter id");
        return None;
    };
    let verdict = match rule.evaluate(source) {
        Ok(msg) => StubVerdict::pass(msg),
        Err(msg) => StubVerdict::fail(msg),
    };
    tracing::debug!(
        encounter_id,
        ok = verdict.ok,
        bytes = source.len(),
        "stub grader verdict"
    );
    Some(verdict)
}

// BEGIN GRADER ENGINE — must stay byte-identical to compile-api/src/grader.rs.

/// One ordered grading step: satisfied when the lexed source contains ANY
/// of the `any_of` patterns; otherwise the mission fails with
/// `"{npc} — {hint}"`. See the module docs for the pattern language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Check {
    pub any_of: &'static [&'static str],
    pub hint: &'static str,
    /// Also look inside string literals (format strings such as `{:?}`).
    /// Off for almost every check: code in a string is not code.
    pub in_strings: bool,
}

/// Shorthand constructor so table entries stay on one line each.
pub const fn check(any_of: &'static [&'static str], hint: &'static str) -> Check {
    Check {
        any_of,
        hint,
        in_strings: false,
    }
}

/// Like [`check`], but the patterns may also match inside string literals.
pub const fn check_in_strings(any_of: &'static [&'static str], hint: &'static str) -> Check {
    Check {
        any_of,
        hint,
        in_strings: true,
    }
}

/// The full grading rule for one mission. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionRule {
    pub id: &'static str,
    pub npc: &'static str,
    pub checks: &'static [Check],
    pub pass: &'static str,
}

impl MissionRule {
    /// Run the checks in order. `Ok(pass message)` or `Err(fail message)`.
    pub fn evaluate(&self, source: &str) -> Result<&'static str, String> {
        let code = lex(source);
        let mut budget = EVAL_BUDGET;
        let mut strings: Option<Vec<Vec<Token>>> = None;
        for c in self.checks {
            let patterns: Vec<Vec<Pat>> = c.any_of.iter().map(|p| compile(p)).collect();
            let mut ok = patterns.iter().any(|p| contains(p, &code, &mut budget));
            if !ok && c.in_strings {
                let lexed = strings.get_or_insert_with(|| {
                    code.iter()
                        .filter(|t| t.kind == TokenKind::Str)
                        .map(|t| lex(&t.text))
                        .collect()
                });
                ok = patterns
                    .iter()
                    .any(|p| lexed.iter().any(|s| contains(p, s, &mut budget)));
            }
            if !ok {
                if budget == 0 {
                    tracing::debug!(id = self.id, "grader: match budget exhausted");
                }
                return Err(format!("{} — {}", self.npc, c.hint));
            }
        }
        // One shared rule for every mission, checked last so a missing
        // concept is always reported first: the program needs an entry point.
        if !has_entry_point(&code) {
            tracing::debug!(id = self.id, "grader: no top-level `fn main`");
            return Err(format!("{} — {}", self.npc, MAIN_HINT));
        }
        Ok(self.pass)
    }
}

/// Fail hint when every mission check passes but there is no `fn main`.
pub const MAIN_HINT: &str = "every Rust program starts at `fn main()`: add one at the top level, outside any `impl` or `mod`";

/// True when the code has a top-level `fn main(` item, optionally `pub`,
/// `pub(..)` or `const`. A `main` nested in a `mod`, `impl`, trait or fn
/// body is not the entry point, and rustc rejects an `async`, `unsafe` or
/// `extern` main, so none of those count. (A `main` re-exported with `use`
/// from a module is valid Rust but is not recognised.)
pub fn has_entry_point(code: &[Token]) -> bool {
    let mut depth = 0usize;
    for (i, t) in code.iter().enumerate() {
        if t.kind == TokenKind::Punct {
            match t.text.as_str() {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth = depth.saturating_sub(1),
                _ => {}
            }
            continue;
        }
        let ident = |k: usize, text: &str| {
            code.get(k)
                .is_some_and(|t| t.kind == TokenKind::Ident && t.text == text)
        };
        let is_main = depth == 0
            && ident(i, "fn")
            && ident(i + 1, "main")
            && code
                .get(i + 2)
                .is_some_and(|t| t.kind == TokenKind::Punct && t.text == "(");
        let bad_qualifier = i.checked_sub(1).and_then(|p| code.get(p)).is_some_and(|p| {
            p.kind == TokenKind::Str
                || (p.kind == TokenKind::Ident
                    && matches!(p.text.as_str(), "async" | "unsafe" | "extern"))
        });
        if is_main && !bad_qualifier {
            return true;
        }
    }
    false
}

/// Look up the rule for an encounter id.
pub fn mission_rule(encounter_id: &str) -> Option<&'static MissionRule> {
    MISSION_RULES.iter().find(|r| r.id == encounter_id)
}

/// True when `pattern` (same language as the mission table) occurs in the
/// code of `source`, ignoring comments and string contents.
pub fn code_contains(source: &str, pattern: &str) -> bool {
    let mut budget = EVAL_BUDGET;
    contains(&compile(pattern), &lex(source), &mut budget)
}

/// What kind of lexeme a [`Token`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifier or keyword (`let`, `answer`, `_`). A raw `r#x` is stored as `x`.
    Ident,
    /// Lifetime or loop label, stored with its quote (`'a`).
    Lifetime,
    /// Number, char or byte literal, stored verbatim (`42`, `3.0`, `'x'`).
    Literal,
    /// String literal of any flavour (plain, raw, byte, C); `text` is the
    /// raw contents between the quotes, escapes left as written.
    Str,
    /// Punctuation. `::` `->` `=>` `==` `!=` `<=` `>=` `+=` `-=` `*=` `/=`
    /// `%=` `^=` `..` `..=` `...` are single tokens; everything else
    /// (including `&&`, `||`, `<<`, `>>`) is one token per character.
    Punct,
}

/// One lexeme of Rust source. Comments and whitespace produce no tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

/// Total matcher steps allowed for one `evaluate` call. Real programs use a
/// tiny fraction of this; it only bounds pathological input. When it runs
/// out, remaining patterns count as "not found".
const EVAL_BUDGET: u32 = 5_000_000;

const MULTI_PUNCT: &[&str] = &[
    "..=", "...", "::", "->", "=>", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "^=",
    "..",
];

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

fn token(kind: TokenKind, chars: &[char]) -> Token {
    Token {
        kind,
        text: chars.iter().collect(),
    }
}

/// Lex Rust source just enough to tell code from comments and literals.
///
/// Never panics: unterminated comments, strings and char literals run to
/// the end of the input; a stray `'` becomes a punctuation token.
pub fn lex(source: &str) -> Vec<Token> {
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut toks: Vec<Token> = Vec::new();
    let mut i = 0;
    while i < len {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c.is_whitespace() {
            i += 1;
        } else if c == '/' && next == Some('/') {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            i = skip_block_comment(&chars, i);
        } else if let Some((tok, end)) = lex_prefixed_literal(&chars, i) {
            toks.push(tok);
            i = end;
        } else if c == '"' {
            let (text, end) = lex_quoted(&chars, i + 1);
            toks.push(Token {
                kind: TokenKind::Str,
                text,
            });
            i = end;
        } else if c == '\'' {
            let (tok, end) = lex_quote(&chars, i, i);
            toks.push(tok);
            i = end;
        } else if c == 'r'
            && next == Some('#')
            && chars.get(i + 2).is_some_and(|&n| is_ident_start(n))
        {
            let start = i + 2;
            let mut j = start;
            while j < len && is_ident_continue(chars[j]) {
                j += 1;
            }
            toks.push(token(TokenKind::Ident, &chars[start..j]));
            i = j;
        } else if is_ident_start(c) {
            let mut j = i;
            while j < len && is_ident_continue(chars[j]) {
                j += 1;
            }
            toks.push(token(TokenKind::Ident, &chars[i..j]));
            i = j;
        } else if c.is_ascii_digit() {
            let after_dot = toks
                .last()
                .is_some_and(|t| t.kind == TokenKind::Punct && t.text == ".");
            let mut j = i;
            while j < len && is_ident_continue(chars[j]) {
                j += 1;
            }
            // `3.5` is one literal; `d.0` (tuple field) and `0..10` are not.
            if !after_dot
                && chars.get(j) == Some(&'.')
                && chars.get(j + 1).is_some_and(|d| d.is_ascii_digit())
            {
                j += 1;
                while j < len && is_ident_continue(chars[j]) {
                    j += 1;
                }
            }
            toks.push(token(TokenKind::Literal, &chars[i..j]));
            i = j;
        } else {
            let ahead: String = chars[i..(i + 3).min(len)].iter().collect();
            let op = MULTI_PUNCT
                .iter()
                .find(|op| ahead.starts_with(**op))
                .map_or(1, |op| op.len());
            toks.push(token(TokenKind::Punct, &chars[i..i + op]));
            i += op;
        }
    }
    toks
}

/// `i` is at the `/` of `/*`. Block comments nest. Returns the index just
/// past the closing `*/`, or the end of input.
fn skip_block_comment(chars: &[char], i: usize) -> usize {
    let len = chars.len();
    let mut depth = 0usize;
    let mut j = i;
    while j < len {
        let pair = (chars[j], chars.get(j + 1).copied());
        if pair == ('/', Some('*')) {
            depth += 1;
            j += 2;
        } else if pair == ('*', Some('/')) {
            depth = depth.saturating_sub(1);
            j += 2;
            if depth == 0 {
                return j;
            }
        } else {
            j += 1;
        }
    }
    len
}

/// Contents of a `"`-delimited literal whose body starts at `start`, plus
/// the index just past the closing quote (or the end of input).
fn lex_quoted(chars: &[char], start: usize) -> (String, usize) {
    let len = chars.len();
    let mut j = start;
    while j < len {
        match chars[j] {
            '\\' => j += 2,
            '"' => return (chars[start..j].iter().collect(), j + 1),
            _ => j += 1,
        }
    }
    (chars[start.min(len)..].iter().collect(), len)
}

/// Raw string body: `p` is just past the `r`. `None` if this is not a raw
/// string (no `"` after the `#`s).
fn lex_raw(chars: &[char], p: usize) -> Option<(String, usize)> {
    let len = chars.len();
    let mut q = p;
    while q < len && chars[q] == '#' {
        q += 1;
    }
    if chars.get(q) != Some(&'"') {
        return None;
    }
    let hashes = q - p;
    let body = q + 1;
    let mut j = body;
    while j < len {
        if chars[j] == '"' && (1..=hashes).all(|k| chars.get(j + k) == Some(&'#')) {
            return Some((chars[body..j].iter().collect(), j + 1 + hashes));
        }
        j += 1;
    }
    Some((chars[body.min(len)..].iter().collect(), len))
}

/// String and char literals that start with a letter: `b"…"`, `c"…"`,
/// `r"…"`, `r#"…"#`, `br"…"`, `cr#"…"#`, `b'x'`.
fn lex_prefixed_literal(chars: &[char], i: usize) -> Option<(Token, usize)> {
    let str_tok = |(text, end): (String, usize)| {
        (
            Token {
                kind: TokenKind::Str,
                text,
            },
            end,
        )
    };
    let next = chars.get(i + 1).copied();
    match chars[i] {
        'b' | 'c' if next == Some('"') => Some(str_tok(lex_quoted(chars, i + 2))),
        'b' if next == Some('\'') => Some(lex_quote(chars, i, i + 1)),
        'b' | 'c' if next == Some('r') => lex_raw(chars, i + 2).map(str_tok),
        'r' => lex_raw(chars, i + 1).map(str_tok),
        _ => None,
    }
}

/// `q` is at a `'` (`start` is where the token begins, before any `b`).
/// Decides between a char literal (`'x'`, `'\n'`, `'"'`), a lifetime or
/// label (`'a`, `'static`) and a stray quote.
fn lex_quote(chars: &[char], start: usize, q: usize) -> (Token, usize) {
    let len = chars.len();
    match chars.get(q + 1).copied() {
        Some('\\') => {
            let mut j = (q + 3).min(len);
            while j < len && chars[j] != '\'' && chars[j] != '\n' {
                j += 1;
            }
            let end = if j < len && chars[j] == '\'' {
                j + 1
            } else {
                j
            };
            (token(TokenKind::Literal, &chars[start..end]), end)
        }
        Some(_) if chars.get(q + 2) == Some(&'\'') => {
            (token(TokenKind::Literal, &chars[start..q + 3]), q + 3)
        }
        Some(a) if is_ident_start(a) && start == q => {
            let mut j = q + 1;
            while j < len && is_ident_continue(chars[j]) {
                j += 1;
            }
            (token(TokenKind::Lifetime, &chars[q..j]), j)
        }
        _ => (token(TokenKind::Punct, &chars[q..q + 1]), q + 1),
    }
}

/// One element of a compiled pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pat {
    /// A literal token (string literals compare by contents).
    Tok(Token),
    /// `$_`: any one identifier.
    AnyIdent,
    /// `$X`: any one identifier; every `$X` in the pattern must be the same.
    Ident(String),
    /// `$'_`: any one lifetime.
    AnyLifetime,
    /// `$'x`: any one lifetime; every `$'x` in the pattern must be the same.
    Lifetime(String),
    /// `$..`: any run of tokens (possibly none) that stays inside the
    /// bracket group it starts in.
    Gap,
}

fn compile(pattern: &str) -> Vec<Pat> {
    let mut out = Vec::new();
    let mut toks = lex(pattern).into_iter().peekable();
    while let Some(t) = toks.next() {
        if t.kind == TokenKind::Punct && t.text == "$" {
            let wildcard = toks.peek().and_then(|n| match (n.kind, n.text.as_str()) {
                (TokenKind::Ident, "_") => Some(Pat::AnyIdent),
                (TokenKind::Ident, name) => Some(Pat::Ident(name.to_string())),
                (TokenKind::Lifetime, "'_") => Some(Pat::AnyLifetime),
                (TokenKind::Lifetime, name) => Some(Pat::Lifetime(name.to_string())),
                (TokenKind::Punct, "..") => Some(Pat::Gap),
                _ => None,
            });
            if let Some(w) = wildcard {
                toks.next();
                out.push(w);
                continue;
            }
        }
        out.push(Pat::Tok(t));
    }
    out
}

/// True when `pat` matches a contiguous run of `toks`.
fn contains(pat: &[Pat], toks: &[Token], budget: &mut u32) -> bool {
    let mut m = Matcher {
        toks,
        binds: Vec::new(),
        budget,
    };
    (0..=toks.len()).any(|start| {
        m.binds.clear();
        m.at(pat, start)
    })
}

struct Matcher<'a> {
    toks: &'a [Token],
    binds: Vec<(String, String)>,
    budget: &'a mut u32,
}

impl Matcher<'_> {
    fn at(&mut self, pat: &[Pat], ti: usize) -> bool {
        let Some((first, rest)) = pat.split_first() else {
            return true;
        };
        if *self.budget == 0 {
            return false;
        }
        *self.budget -= 1;
        let toks = self.toks;
        if *first == Pat::Gap {
            let mut depth = 0usize;
            let mut j = ti;
            loop {
                if self.at(rest, j) {
                    return true;
                }
                let Some(t) = toks.get(j) else {
                    return false;
                };
                if t.kind == TokenKind::Punct {
                    match t.text.as_str() {
                        "(" | "[" | "{" => depth += 1,
                        ")" | "]" | "}" if depth == 0 => return false,
                        ")" | "]" | "}" => depth -= 1,
                        _ => {}
                    }
                }
                j += 1;
            }
        }
        let Some(t) = toks.get(ti) else {
            return false;
        };
        let mark = self.binds.len();
        let ok = match first {
            Pat::Tok(p) => p == t,
            Pat::AnyIdent => t.kind == TokenKind::Ident,
            Pat::AnyLifetime => t.kind == TokenKind::Lifetime,
            Pat::Ident(name) => t.kind == TokenKind::Ident && self.bind(name, &t.text),
            Pat::Lifetime(name) => t.kind == TokenKind::Lifetime && self.bind(name, &t.text),
            Pat::Gap => false,
        };
        if ok && self.at(rest, ti + 1) {
            return true;
        }
        self.binds.truncate(mark);
        false
    }

    fn bind(&mut self, name: &str, text: &str) -> bool {
        match self.binds.iter().find(|(n, _)| n == name) {
            Some((_, bound)) => bound == text,
            None => {
                self.binds.push((name.to_string(), text.to_string()));
                true
            }
        }
    }
}
// END GRADER ENGINE

// BEGIN MISSION TABLE — must stay byte-identical to compile-api/src/grader.rs.
#[rustfmt::skip]
pub static MISSION_RULES: &[MissionRule] = &[
    // ── Act 1 (Ferris's Village).
    MissionRule {
        id: "intro_let_binding",
        npc: "not yet",
        checks: &[
            check(&["let answer"], "missing required: `let answer`"),
            check(&["42"], "missing required: `42`"),
        ],
        pass: "answer bound. Ferris nods approvingly.",
    },
    MissionRule {
        id: "double_function",
        npc: "not yet",
        checks: &[
            check(&["fn double"], "missing required: `fn double`"),
            check(&["i32"], "missing required: `i32`"),
            check(&["* 2", "2 *", "$X + $X"], "missing required: `*2`"),
        ],
        pass: "the Trait Mage's staff hums. \"named once, doubled twice over.\"",
    },
    MissionRule {
        id: "borrow_preview",
        npc: "the Borrow Checker is silent",
        checks: &[
            check(&["&value"], "missing required: `&value`"),
            check(&["println!"], "missing required: `println!`"),
        ],
        pass: "the Borrow Checker stirs. \"...acceptable. for now.\"",
    },
    MissionRule {
        id: "mut_binding",
        npc: "the Smith eyes you",
        checks: &[
            check(&["let mut"], "missing required: `let mut`"),
            check(&["+= 1"], "missing required: `+= 1`"),
        ],
        pass: "the Smith strikes the work. \"good — that one bends.\"",
    },
    MissionRule {
        id: "if_else_sign",
        npc: "the Cartographer frowns",
        checks: &[
            check(&["if"], "missing required: `if `"),
            check(&["else"], "missing required: `else`"),
            check(&["< 0"], "missing required: `< 0`"),
            check(&["< 0 { \"neg\"", "< 0 { return \"neg\"", "< 0 => \"neg\""], "the `< 0` branch should give `\"neg\"`"),
            check(&["== 0 { \"zero\"", "== 0 { return \"zero\"", "0 => \"zero\"", "> 0 { \"pos\" } else { \"zero\"", "< 0 { \"neg\" } else { \"zero\""], "zero should give `\"zero\"`"),
            check(&["> 0 { \"pos\"", "> 0 { return \"pos\"", "_ => \"pos\"", "== 0 { \"zero\" } else { \"pos\"", "== 0 { return \"zero\"; } else { return \"pos\"", "< 0 { \"neg\" } else { \"pos\""], "a positive number should give `\"pos\"`"),
        ],
        pass: "the Cartographer nods. \"three roads diverge — well chosen.\"",
    },
    MissionRule {
        id: "loop_break",
        npc: "the Bellringer waits",
        checks: &[
            check(&["loop"], "missing required: `loop`"),
            check(&["break"], "missing required: `break`"),
            check(&["*= 2", "$X = $X * 2", "$X = 2 * $X"], "the value should double each iteration: `*= 2`"),
            check(&["break $_"], "hand the value out of the loop with `break n`"),
        ],
        pass: "the Bellringer pulls the rope — the bell sounds. \"the loop returned its number.\"",
    },
    MissionRule {
        id: "match_option",
        npc: "the Oracle's eyes are closed",
        checks: &[
            check(&["match"], "missing required: `match`"),
            check(&["Some("], "missing required: `Some(`"),
            check(&["None"], "missing required: `None`"),
            check(&["Some($X) => $X", "Some($X) => { $X", "Some($X) => return $X"], "the `Some(n)` arm should give back the `n` it binds"),
            check(&["None => 0", "None => { 0", "None => return 0"], "the `None` arm should give `0`"),
        ],
        pass: "the Oracle exhales. \"both paths walked. nothing slips through.\"",
    },
    MissionRule {
        id: "vec_iter",
        npc: "the Cooper waits",
        checks: &[
            check(&["vec!"], "missing required: `vec!`"),
            check(&[".iter()"], "call `.iter()` on the vector"),
            check(&[".sum"], "finish with `.sum()` to reduce"),
        ],
        pass: "the Cooper hoops the barrel. \"every stave counted, sum sealed.\"",
    },
    MissionRule {
        id: "tuple_destructure",
        npc: "the Twin waits",
        checks: &[
            check(&["let ("], "missing required: `let (`"),
            check(&[","], "a tuple needs a comma between its parts"),
            check(&[") ="], "close the pattern with `) =`"),
        ],
        pass: "the Twin grins. \"two names, one breath. cleanly split.\"",
    },
    MissionRule {
        id: "while_loop",
        npc: "the Tinker frowns",
        checks: &[
            check(&["while"], "missing required: `while `"),
            check(&["> 0"], "the predicate should test `> 0`"),
            check(&["-= 1"], "decrement with `-= 1` each iteration"),
        ],
        pass: "the Tinker pockets the spring. \"counted down, gear by gear.\"",
    },
    MissionRule {
        id: "struct_basic",
        npc: "the Herald squints",
        checks: &[
            check(&["struct Knight"], "missing required: `struct Knight`"),
            check(&["name:"], "Knight needs a `name:` field"),
            check(&["hp:"], "Knight needs an `hp:` field"),
            check(&[".name"], "read the field with `.name`"),
        ],
        pass: "the Herald reads the tabard. \"by name and by number — both fields announced.\"",
    },
    // ── Act 2 (Borrow Checker's Bridge).
    MissionRule {
        id: "borrow_mut",
        npc: "the Forgewright glares",
        checks: &[
            check(&["fn bump"], "missing required: `fn bump`"),
            check(&["&mut i32"], "the parameter must be `&mut i32` (exclusive borrow)"),
            check(&["fn bump($X: &mut i32) $.. *$X"], "write through the reference with a `*x` deref"),
        ],
        pass: "the Forgewright nods. \"one writer at the forge — the borrow holds.\"",
    },
    MissionRule {
        id: "string_vs_str",
        npc: "the Linguist tilts her head",
        checks: &[
            check(&["fn greet"], "missing required: `fn greet`"),
            check(&["&str"], "the parameter type should be `&str`"),
            check(&["String::from", ".to_string()", ".to_owned()", "String::new()", "format!"], "call `greet` once with a `String::from(...)` value"),
        ],
        pass: "the Linguist smiles. \"one tongue, two voices — &str carries both.\"",
    },
    MissionRule {
        id: "option_unwrap_or",
        npc: "the Pilgrim shakes his head",
        checks: &[
            check(&["Option<"], "missing required: `Option<`"),
            check(&[".unwrap_or("], "collapse the absent case with `.unwrap_or(default)`"),
        ],
        pass: "the Pilgrim raises his lantern. \"the absent path lit a default.\"",
    },
    MissionRule {
        id: "for_in_range",
        npc: "the Drillmaster barks",
        checks: &[
            check(&["for"], "missing required: `for `"),
            check(&["in"], "`for` needs a binding `in` an iterable"),
            check(&["0..10"], "iterate the exact range `0..10`"),
        ],
        pass: "the Drillmaster claps once. \"ten paces, exact and ordered.\"",
    },
    MissionRule {
        id: "closure_basic",
        npc: "the Reckoner's quill hovers",
        checks: &[
            check(&["let add"], "missing required: `let add`"),
            check(&["= |", "= move |"], "bind a closure literal with `= |...|`"),
            check(&["|$X $.., $Y $..| $.. $X + $Y", "|$X $.., $Y $..| $.. $Y + $X"], "the body should compute `a + b`"),
        ],
        pass: "the Reckoner inks the ledger. \"summed in a single stroke.\"",
    },
    MissionRule {
        id: "slice_basic",
        npc: "the Quartermaster squints",
        checks: &[
            check(&["fn sum_slice"], "missing required: `fn sum_slice`"),
            check(&["&[i32]"], "the parameter must be a slice `&[i32]`"),
            check(&[".iter()"], "walk the slice with `.iter()`"),
        ],
        pass: "the Quartermaster ticks the manifest. \"a window into the stores — counted, not claimed.\"",
    },
    MissionRule {
        id: "result_question_mark",
        npc: "the Auditor's pen taps",
        checks: &[
            check(&["Result<"], "missing required: `Result<`"),
            check(&[".parse"], "call `.parse` on a string source"),
            check(&["?"], "propagate the error with the `?` operator"),
        ],
        pass: "the Auditor marks the tally. \"errors travel upward — the books balance.\"",
    },
    MissionRule {
        id: "derive_debug",
        npc: "the Chronicler shakes his head",
        checks: &[
            check(&["#[derive( $.. Debug"], "missing required: `#[derive(Debug)]`"),
            check(&["struct Item"], "define `struct Item`"),
            check_in_strings(&[":?", ":#?"], "print with the debug formatter `{:?}`"),
        ],
        pass: "the Chronicler dips his quill. \"derived, not authored.\"",
    },
    MissionRule {
        id: "iter_map_collect",
        npc: "the Alchemist stirs",
        checks: &[
            check(&[".map("], "missing required: `.map(`"),
            check(&[".collect"], "finish with `.collect` to gather the result"),
            check(&["|$_|", "|&$_|", "|$_: $..|"], "the closure should bind one element as `|x|`"),
        ],
        pass: "the Alchemist decants the flask. \"every drop transmuted.\"",
    },
    MissionRule {
        id: "enum_match",
        npc: "the Heraldic Sage frowns",
        checks: &[
            check(&["enum Direction"], "missing required: `enum Direction`"),
            check(&["match"], "inspect the variant with a `match ` expression"),
            check(&["Direction::"], "name a variant with the `Direction::` path"),
        ],
        pass: "the Heraldic Sage walks the sash. \"every variant a panel, every arm a blazon read.\"",
    },
    // ── Act 2 boss: the Borrow Checker's trial, The Bridge (stages 1-3).
    MissionRule {
        id: "bridge_toll",
        npc: "the Borrow Checker bars the bridge",
        checks: &[
            check(&["fn weigh($_: &str)"], "weigh must borrow its cargo: `fn weigh(cargo: &str)`"),
            check(&["weigh(&$_)", "weigh($_.as_str())"], "lend the cargo: call `weigh(&cargo)`"),
        ],
        pass: "the Borrow Checker steps aside one pace. \"the cargo is still yours. first toll paid.\"",
    },
    MissionRule {
        id: "bridge_quill",
        npc: "the Borrow Checker holds the quill back",
        checks: &[
            check(&["fn inscribe($S: &mut Vec<String>, $_: &str)"], "inscribe takes the scroll as `&mut Vec<String>` and the entry as `&str`"),
            check(&["fn inscribe($S: &mut Vec<String>, $_: &str) $.. $S.push("], "inscribe must `push` the entry onto the scroll"),
            check(&["inscribe(&mut $_"], "hand over the quill exclusively: call `inscribe(&mut scroll, ...)`"),
        ],
        pass: "the Borrow Checker releases the quill. \"readers first, then the writer. second toll paid.\"",
    },
    MissionRule {
        id: "bridge_lantern",
        npc: "the Borrow Checker shades the lantern",
        checks: &[
            check(&["fn before<$'a>($_: &$'a str, $_: &str) -> &$'a str", "fn before<$'a, $'b>($_: &$'a str, $_: &$'b str) -> &$'a str"], "tie the answer to `line` only: `fn before<'a>(line: &'a str, sep: &str) -> &'a str`"),
        ],
        pass: "the Borrow Checker lifts the lantern. \"the answer borrows from the line alone. you may cross.\"",
    },
    // ── Act 3 (Guildhall Quarter).
    MissionRule {
        id: "impl_method",
        npc: "the Guildmaster waits",
        checks: &[
            check(&["impl"], "missing required: `impl`"),
            check(&["&self"], "a method borrows the instance with `&self`"),
            check(&["self."], "read the instance's data with `self.`"),
        ],
        pass: "the Guildmaster nods. \"duties bound to a name — that is a method.\"",
    },
    MissionRule {
        id: "assoc_new",
        npc: "the Recruiter shakes her head",
        checks: &[
            check(&["fn new"], "missing required: `fn new`"),
            check(&["Self", "-> Hero"], "a constructor returns `Self`"),
            check(&["::new"], "call it with the `::new` path"),
        ],
        pass: "the Recruiter stamps the roll. \"Self::new — a member forged from a name.\"",
    },
    MissionRule {
        id: "if_let",
        npc: "the Locksmith frowns",
        checks: &[
            check(&["if let"], "missing required: `if let`"),
            check(&["Some("], "match the present case with `Some(`"),
        ],
        pass: "the Locksmith turns the one key that fits. \"Some — and only Some — opens.\"",
    },
    MissionRule {
        id: "while_let",
        npc: "the Porter sets the crate down",
        checks: &[
            check(&["while let"], "missing required: `while let`"),
            check(&[".pop("], "drain the stack with `.pop()`"),
        ],
        pass: "the Porter empties the cart, crate by crate. \"while there is a Some, keep unloading.\"",
    },
    MissionRule {
        id: "tuple_struct",
        npc: "the Surveyor squints down the line",
        checks: &[
            check(&["struct Meters("], "missing required: `struct Meters(`"),
            check(&[".0"], "read the wrapped value with `.0`"),
        ],
        pass: "the Surveyor reads the rod. \"a bare number, now named Meters — a newtype.\"",
    },
    MissionRule {
        id: "enum_data_match",
        npc: "the Armorer taps the anvil",
        checks: &[
            check(&["enum Item"], "missing required: `enum Item`"),
            check(&["Weapon"], "give Item a `Weapon` variant"),
            check(&["match"], "inspect the item with `match`"),
            check(&["=>"], "each variant needs a `=>` arm"),
        ],
        pass: "the Armorer lays out the rack. \"weapon or potion — name every kind, miss none.\"",
    },
    // ── Act 3 boss: the Guildmaster's trial, The Variant Wraith (stages 1-3).
    MissionRule {
        id: "wraith_forms",
        npc: "the Guildmaster raises the ledger",
        checks: &[
            check(&["Wraith::Shadow =>"], "give the Shadow its own arm: `Wraith::Shadow => \"light\"`"),
        ],
        pass: "the Guildmaster ticks the ledger. \"three forms, three arms. first stage passed.\"",
    },
    MissionRule {
        id: "wraith_essence",
        npc: "the Guildmaster taps the ledger",
        checks: &[
            check(&["Wraith::Solid { armor } => $.. - armor", "Wraith::Solid { armor: $A } => $.. - $A"], "open the Solid form and use its armor: `Wraith::Solid { armor } => blow - armor`"),
            check(&["Wraith::Mist { density } => $.. / density", "Wraith::Mist { density: $D } => $.. / $D"], "open the Mist and use its density: `Wraith::Mist { density } => blow / density`"),
        ],
        pass: "the Guildmaster nods. \"each form opened, each measure used. second stage passed.\"",
    },
    MissionRule {
        id: "wraith_possessed",
        npc: "the Guildmaster holds the ledger shut",
        checks: &[
            check(&["Wraith::Possessed { will } => $.. + will", "Wraith::Possessed { will: $W } => $.. + $W", "Wraith::Possessed { will } => will +", "Wraith::Possessed { will: $W } => $W +"], "name the new form and add its will: `Wraith::Possessed { will } => blow + will`"),
        ],
        pass: "the Guildmaster closes the ledger. \"four forms, every one named. the Wraith is unmade.\"",
    },
    // ── Act 4 (Trait Mage's Tower).
    MissionRule {
        id: "trait_def",
        npc: "Vexis waits",
        checks: &[
            check(&["trait"], "missing required: a `trait ` definition"),
            check(&["impl"], "implement it with an `impl ` block"),
            check(&["trait $T $.. impl $T for", "trait $T $.. impl<$..> $T for", "impl $T for $.. trait $T"], "bind the trait to a type: `impl Trait for Type`"),
        ],
        pass: "Vexis lowers his staff. \"a capability named once, granted to a type — that is a trait.\"",
    },
    MissionRule {
        id: "generic_fn",
        npc: "the Wandwright frowns",
        checks: &[
            check(&["<T", "fn $_ < $_"], "missing required: a type parameter `<T`"),
            check(&["PartialOrd"], "bound the parameter so it can be compared: `T: PartialOrd`"),
        ],
        pass: "the Wandwright sights down the blank. \"one wand, any element — bounded by what it can compare.\"",
    },
    MissionRule {
        id: "generic_struct",
        npc: "the Conjurer waits",
        checks: &[
            check(&["struct"], "missing required: a `struct ` definition"),
            check(&["struct $_ < $_ >", "struct $_ < $_ :"], "make it generic with a `<T>` parameter"),
            check(&["struct $_ < $T $.. > { $.. : $T", "struct $_ < $T $.. > ( $.. $T"], "give it a field of the generic type `: T`"),
        ],
        pass: "the Conjurer cups two matching lights. \"a vessel for any type — so long as both halves agree.\"",
    },
    MissionRule {
        id: "dyn_trait",
        npc: "the Familiar flickers",
        checks: &[
            check(&["Box<dyn"], "missing required: `Box<dyn ...>`"),
            check(&["Box::new"], "box each value onto the heap with `Box::new`"),
        ],
        pass: "the Familiar shifts through its forms. \"many shapes, one cage — dispatched at a touch.\"",
    },
    MissionRule {
        id: "lifetimes",
        npc: "the Lanternkeeper waits",
        checks: &[
            check(&["<$'_>", "<$'_,"], "missing required: a lifetime parameter `<'a>`"),
            check(&["<$'a $.. &$'a"], "annotate the references with `&'a`"),
        ],
        pass: "the Lanternkeeper keeps the flame. \"the borrow lives exactly as long as 'a — no shorter.\"",
    },
    MissionRule {
        id: "assoc_type",
        npc: "the Loremaster turns the page",
        checks: &[
            check(&["type Output"], "missing required: `type Output`"),
            check(&["Self::Output"], "refer to it as `Self::Output` in the method"),
        ],
        pass: "the Loremaster turns the page. \"each producer names its own yield — the type follows the trait.\"",
    },
    // ── Act 4 boss: Vexis's trial, The Chimera of Vexis (stages 1-3).
    MissionRule {
        id: "chimera_heads",
        npc: "Vexis raises his staff",
        checks: &[
            check(&["impl Enemy for Frost"], "teach the Frost head the trait: `impl Enemy for Frost`"),
            check(&["impl Enemy for Storm"], "teach the Storm head the trait: `impl Enemy for Storm`"),
        ],
        pass: "Vexis lowers his staff. \"three heads, one trait. first stage passed.\"",
    },
    MissionRule {
        id: "chimera_bound",
        npc: "Vexis studies the spell",
        checks: &[
            check(&["fn volley<$T: Enemy>", "fn volley<$T>($..) -> String where $T: Enemy", "fn volley($_: &impl Enemy"], "bound the type parameter: `fn volley<T: Enemy>(head: &T, rounds: u32)`"),
        ],
        pass: "Vexis traces the bound. \"any head that holds the trait may answer. second stage passed.\"",
    },
    MissionRule {
        id: "chimera_plasma",
        npc: "Vexis bars the stair",
        checks: &[
            check(&["fn battle($_: &[Box<dyn Enemy>])"], "battle takes trait objects: `fn battle(heads: &[Box<dyn Enemy>]) -> u32`"),
        ],
        pass: "Vexis steps aside. \"any head, known or not. the Chimera falls.\"",
    },
    // ── Act 5 (Tavern of Tribulations).
    MissionRule {
        id: "result_match",
        npc: "the Barkeep waits",
        checks: &[
            check(&["match"], "missing required: `match `"),
            check(&["Ok("], "handle the success arm `Ok(`"),
            check(&["Err("], "handle the failure arm `Err(`"),
        ],
        pass: "the Barkeep nods. \"poured or spilled — you accounted for both.\"",
    },
    MissionRule {
        id: "custom_error",
        npc: "the Bouncer crosses his arms",
        checks: &[
            check(&["enum"], "missing required: an error `enum `"),
            check(&["Result<"], "return a `Result<...>` from check"),
            check(&["Err("], "produce a failure with `Err(...)`"),
        ],
        pass: "the Bouncer grunts. \"every kind of trouble, named and on the list.\"",
    },
    MissionRule {
        id: "from_error",
        npc: "the Interpreter pauses",
        checks: &[
            check(&["impl From<"], "missing required: `impl From<...>`"),
            check(&["for"], "implement it `for` your error type"),
            check(&["fn from"], "the conversion is a `fn from` method"),
        ],
        pass: "the Interpreter inclines her head. \"one tongue's failure, spoken in another.\"",
    },
    MissionRule {
        id: "option_map",
        npc: "the Mixologist waits",
        checks: &[
            check(&["Option<"], "keep it an `Option<...>`"),
            check(&[".map("], "transform the inside with `.map(`"),
        ],
        pass: "the Mixologist swirls the glass. \"changed within — if there was anything to change.\"",
    },
    MissionRule {
        id: "and_then",
        npc: "the Tabkeeper waits",
        checks: &[
            check(&["Option"], "the steps should yield an `Option`"),
            check(&[".and_then("], "chain the fallible steps with `.and_then(`"),
        ],
        pass: "the Tabkeeper closes the ledger. \"one failed round and the tab is cut.\"",
    },
    MissionRule {
        id: "unwrap_or_else",
        npc: "the Cellarer waits",
        checks: &[
            check(&[".unwrap_or_else("], "missing required: `.unwrap_or_else(`"),
            check(&["||"], "the lazy default is a closure `|| ...`"),
        ],
        pass: "the Cellarer taps the cask. \"a fresh draught — drawn only when the old runs dry.\"",
    },
    // ── Act 5 boss: the Barkeep's trial, The Tavernkeeper's Wager (stages 1-3).
    MissionRule {
        id: "wager_stake",
        npc: "the Barkeep holds the stake",
        checks: &[
            check(&["fn double_stake($_: &str) -> Result<u32, ParseIntError>", "fn double_stake($_: &str) -> Result<u32, std::num::ParseIntError>"], "let it fail out loud: `fn double_stake(text: &str) -> Result<u32, ParseIntError>`"),
            check(&["Ok("], "hand back the doubled stake wrapped in `Ok(...)`"),
        ],
        pass: "the Barkeep pockets the stake. \"poured or spilled, it comes back to the table. first round yours.\"",
    },
    MissionRule {
        id: "wager_from",
        npc: "the Barkeep taps the tally",
        checks: &[
            check(&["impl From<ParseIntError> for WagerError", "impl From<std::num::ParseIntError> for WagerError"], "teach `?` the bad stake: `impl From<ParseIntError> for WagerError`"),
            check(&["impl From<Short> for WagerError"], "teach `?` the light purse: `impl From<Short> for WagerError`"),
        ],
        pass: "the Barkeep chalks the tally. \"every error in the house's coin. second round yours.\"",
    },
    MissionRule {
        id: "wager_quest",
        npc: "the Barkeep keeps the purse",
        checks: &[
            check(&["read_map($..)?"], "chain step 1 with `?`: `read_map(map)?`"),
            check(&["open_vault($..)?"], "chain step 2 with `?`: `open_vault(key)?`"),
            check(&["pay_guide($..)?"], "chain step 3 with `?`: `pay_guide(gold, leagues)?`"),
        ],
        pass: "the Barkeep pays out. \"every step chained, every failure handed back. the wager is yours.\"",
    },
    // ── Act 6 (Iterator Forge).
    MissionRule {
        id: "hashmap_basic",
        npc: "the Keymaster waits",
        checks: &[
            check(&["HashMap"], "missing required: `HashMap`"),
            check(&[".insert("], "store a value with `.insert(`"),
            check(&[".get("], "look it up with `.get(`"),
        ],
        pass: "the Keymaster turns the ring. \"every store under its own key.\"",
    },
    MissionRule {
        id: "iter_filter",
        npc: "the Sifter shakes the screen",
        checks: &[
            check(&[".filter("], "missing required: `.filter(`"),
            check(&[".collect"], "gather the survivors with `.collect`"),
        ],
        pass: "the Sifter shakes the screen. \"only what fits the mesh falls through.\"",
    },
    MissionRule {
        id: "iter_fold",
        npc: "the Smelter waits",
        checks: &[
            check(&[".fold("], "missing required: `.fold(`"),
            check(&[".fold( $.. |$_ $.., $X $..| $.. $X", ".fold( $.. |$_ $.., &$X $..| $.. $X", "let $F = |$_ $.., $X $..| $.. $X $.. .fold( $.. , $F )", "fn $F ( $_ $.., $X : $.. ) -> $_ { $.. $X $.. } $.. .fold( $.. , $F )", ".fold( $.. , $F ) $.. } $.. fn $F ( $_ $.., $X : $.. ) -> $_ { $.. $X $.. }"], "the closure must use each element: `|acc, x| acc + x`"),
        ],
        pass: "the Smelter tips the crucible. \"many ores, one ingot.\"",
    },
    MissionRule {
        id: "iter_enumerate",
        npc: "the Tallywright waits",
        checks: &[
            check(&[".enumerate("], "missing required: `.enumerate(`"),
        ],
        pass: "the Tallywright stamps the row. \"each item numbered as it passes.\"",
    },
    MissionRule {
        id: "iter_zip",
        npc: "the Riveter waits",
        checks: &[
            check(&["zip("], "missing required: `.zip(`"),
        ],
        pass: "the Riveter drives the rivet. \"two plates, aligned pair by pair.\"",
    },
    MissionRule {
        id: "closure_move",
        npc: "the Bondsmith waits",
        checks: &[
            check(&["move |"], "missing required: a `move` closure (`move |...|`)"),
        ],
        pass: "the Bondsmith seals the charge. \"owned, and carried where I go.\"",
    },
    // ── Act 6 boss: the Keymaster's trial, The Forge-Golem (stages 1-3).
    MissionRule {
        id: "golem_parts",
        npc: "the Keymaster turns a forge key",
        checks: &[
            check(&[".filter($..).map("], "filter while you still hold each whole part: `.filter(..)` before `.map(..)`"),
        ],
        pass: "the Keymaster hangs up the key. \"every weak point found. first stage passed.\"",
    },
    MissionRule {
        id: "golem_sum",
        npc: "the Keymaster checks the label",
        checks: &[
            check(&[": u32 = $.. .sum(", ".sum::<u32>(", ".fold("], "tell `sum` what to make: `let total: u32 = ...sum();`"),
        ],
        pass: "the Keymaster files the total. \"labelled and doubled. second stage passed.\"",
    },
    MissionRule {
        id: "golem_core",
        npc: "the Keymaster bars the forge",
        checks: &[
            check(&[".filter("], "skip the armour with `.filter(..)`: it passes a plate by and keeps going"),
            check(&[".fold(", ".sum"], "gather the hits into one total with `.fold(0, ..)` or `.sum()`"),
        ],
        pass: "the Keymaster lowers the bar. \"one chain, every seam struck. the Golem falls.\"",
    },
    // ── Act 7 (Concurrent Coast).
    MissionRule {
        id: "thread_spawn",
        npc: "the Dockmaster waits",
        checks: &[
            check(&["thread::spawn"], "missing required: `thread::spawn`"),
            check(&[".join("], "wait for the worker with `.join(`"),
        ],
        pass: "the Dockmaster waves them off. \"sent to work — and met again at the gate.\"",
    },
    MissionRule {
        id: "arc_mutex",
        npc: "the Lighthouse Keeper waits",
        checks: &[
            check(&["Arc"], "share ownership with `Arc`"),
            check(&["Mutex"], "guard the value with `Mutex`"),
            check(&[".lock("], "take exclusive access with `.lock(`"),
        ],
        pass: "the Lighthouse Keeper turns the key. \"one hand on the lamp at a time.\"",
    },
    MissionRule {
        id: "mpsc_channel",
        npc: "the Signaler waits",
        checks: &[
            check(&["mpsc::channel"], "missing required: `mpsc::channel`"),
            check(&[".send("], "push a value in with `.send(`"),
            check(&[".recv("], "read it at the far end with `.recv(`"),
        ],
        pass: "the Signaler flashes the lamp. \"sent down the coast, read at the next tower.\"",
    },
    MissionRule {
        id: "atomic",
        npc: "the Tidewatch waits",
        checks: &[
            check(&["AtomicBool", "AtomicI8", "AtomicI16", "AtomicI32", "AtomicI64", "AtomicIsize", "AtomicU8", "AtomicU16", "AtomicU32", "AtomicU64", "AtomicUsize", "AtomicPtr"], "use an `Atomic` type"),
            check(&[".fetch_add("], "bump it atomically with `.fetch_add(`"),
        ],
        pass: "the Tidewatch clicks the gauge. \"one notch up — no gatekeeper.\"",
    },
    MissionRule {
        id: "thread_scope",
        npc: "the Harbormaster waits",
        checks: &[
            check(&["thread::scope"], "missing required: `thread::scope`"),
            check(&[".spawn("], "launch a scoped thread with `.spawn(`"),
        ],
        pass: "the Harbormaster rings the bell. \"every boat in before the harbor closes.\"",
    },
    MissionRule {
        id: "async_fn",
        npc: "the Tideforecaster waits",
        checks: &[
            check(&["async fn"], "missing required: `async fn`"),
            check(&[".await"], "drive the future with `.await`"),
        ],
        pass: "the Tideforecaster lowers the glass. \"the tide will come — awaited, not forced.\"",
    },
    // ── Act 7 boss: the Dockmaster's trial, The Tide Hydra (stages 1-3).
    MissionRule {
        id: "hydra_senders",
        npc: "the Dockmaster raises a signal flag",
        checks: &[
            check(&[".clone()", "Sender::clone(", "&tx)"], "give each head its own sender: `fight(head, hp, tx.clone())`"),
        ],
        pass: "the Dockmaster reads the flags. \"every head reporting. first stage passed.\"",
    },
    MissionRule {
        id: "hydra_threads",
        npc: "the Dockmaster holds the gate",
        checks: &[
            check(&["spawn(move"], "send each worker off owning its load: `thread::spawn(move || ...)`"),
            check(&[".sort", ".sort_unstable", ".sort_by", ".sort_by_key", ".sort_unstable_by", ".sort_unstable_by_key"], "nothing fixes the order the reports arrive in: sort them before you return"),
        ],
        pass: "the Dockmaster opens the gate. \"every worker away with its own sender. second stage passed.\"",
    },
    MissionRule {
        id: "hydra_body",
        npc: "the Dockmaster watches the water",
        checks: &[
            check(&["< heads", "<= heads", "== heads", ">= heads", "!= heads", "heads >", "heads ==", "heads !="], "count the heads that report `Down`, and wait until the count reaches `heads`"),
        ],
        pass: "the Dockmaster signals the strike. \"every head down, then the body. the Hydra sinks.\"",
    },
    // ── Act 8 (Vault of Pointers).
    MissionRule {
        id: "box_basic",
        npc: "the Vaultwright waits",
        checks: &[
            check(&["Box<"], "missing required: `Box<...>`"),
            check(&["Box::new"], "put the value on the heap with `Box::new`"),
        ],
        pass: "the Vaultwright seals the box. \"on the heap it holds what the stack cannot.\"",
    },
    MissionRule {
        id: "rc_basic",
        npc: "the Sharekeeper waits",
        checks: &[
            check(&["Rc::new"], "missing required: `Rc::new`"),
            check(&["Rc::clone"], "hand out a second handle with `Rc::clone`"),
        ],
        pass: "the Sharekeeper tallies the claims. \"one hoard, many holders.\"",
    },
    MissionRule {
        id: "refcell",
        npc: "the Warden waits",
        checks: &[
            check(&["RefCell"], "missing required: `RefCell`"),
            check(&[".borrow_mut("], "mutate the inner value with `.borrow_mut(`"),
        ],
        pass: "the Warden checks the ledger. \"changed within — by the runtime's leave.\"",
    },
    MissionRule {
        id: "cell",
        npc: "the Swapwarden waits",
        checks: &[
            check(&["Cell::new"], "missing required: `Cell::new`"),
            check(&[".set("], "swap the whole value with `.set(`"),
        ],
        pass: "the Swapwarden trades the coin. \"the whole value, swapped clean.\"",
    },
    MissionRule {
        id: "rc_refcell",
        npc: "the Strongbox waits",
        checks: &[
            check(&["Rc::new"], "share ownership with `Rc::new`"),
            check(&["RefCell"], "make it mutable inside with `RefCell`"),
            check(&[".borrow_mut("], "open and change it via `.borrow_mut(`"),
        ],
        pass: "the Strongbox turns the keys. \"many owners, and the box still opens.\"",
    },
    MissionRule {
        id: "weak_ref",
        npc: "the Ghostkeeper waits",
        checks: &[
            check(&["Weak"], "missing required: a `Weak` reference"),
            check(&["downgrade"], "make the weak handle with `Rc::downgrade`"),
            check(&[".upgrade("], "reclaim it (or not) with `.upgrade(`"),
        ],
        pass: "the Ghostkeeper tugs the tether. \"a hold that holds nothing alive.\"",
    },
    // ── Act 8 boss: the Vaultwright's trial, The Memory Warden (stages 1-3).
    MissionRule {
        id: "warden_box",
        npc: "the Vaultwright opens the ledger",
        checks: &[
            check(&["Box<Ledger>"], "give the rest of the ledger a fixed size: `Entry(u32, Box<Ledger>)`"),
            check(&["Box::new("], "put the rest on the heap where you build an entry: `Box::new(rest)`"),
        ],
        pass: "the Vaultwright seals the box. \"a ledger of any length, in a size the compiler knows. first phase passed.\"",
    },
    MissionRule {
        id: "warden_shared",
        npc: "the Vaultwright weighs the vault",
        checks: &[
            check(&["Rc<RefCell<u32>>"], "share a vault you can write to: `vault: Rc<RefCell<u32>>`"),
            check(&[".borrow_mut()"], "borrow the vault to add the coins: `*debtor.vault.borrow_mut() += coins`"),
        ],
        pass: "the Vaultwright counts the coins. \"one vault, many hands. second phase passed.\"",
    },
    MissionRule {
        id: "warden_weak",
        npc: "the Vaultwright traces the ring",
        checks: &[
            check(&["Weak<Debtor>"], "make each debt a link that owns nothing: `owes: RefCell<Weak<Debtor>>`"),
            check(&["downgrade("], "link the debtors with `Rc::downgrade(&creditor)`"),
            check(&[".upgrade()"], "follow a link with `.upgrade()`: `Some` while the debtor lives"),
        ],
        pass: "the Vaultwright closes the vault. \"walked once, nothing left behind. the Warden is unmade.\"",
    },
    // ── Act 9 (Forbidden Library). `unsafe` appears only inside these
    // string literals: grader patterns, never code.
    MissionRule {
        id: "raw_pointer",
        npc: "the Shelfmark taps the card",
        checks: &[
            check(&["unsafe {", "unsafe{"], "dereference the pointer inside an `unsafe` block"),
            check(&["*mut", "&raw mut", "addr_of_mut!"], "write through a `*mut` raw pointer"),
            check(&["*const", "&raw const", "addr_of!"], "read back through a `*const` raw pointer"),
        ],
        pass: "the Shelfmark files the card. \"an address is not a book — but you fetched the book.\"",
    },
    MissionRule {
        id: "extern_c",
        npc: "the Envoy waits at the border",
        checks: &[
            check(&["repr(C)"], "lay `Rune` out by C's rules with `#[repr(C)]`"),
            check(&["no_mangle"], "export `crown_seal` under its plain name with `#[unsafe(no_mangle)]`"),
            check(&["extern \"C\" fn"], "give `crown_seal` the C calling convention: `pub extern \"C\" fn`"),
            check(&["unsafe {", "unsafe{"], "call through the foreign declaration inside `unsafe`"),
        ],
        pass: "the Envoy seals the treaty. \"two tongues, one calling convention.\"",
    },
    MissionRule {
        id: "macro_rules",
        npc: "the Macro Monk keeps his silence",
        checks: &[
            check(&["macro_rules!"], "define the macro with `macro_rules!`"),
            check(&["$("], "accept a list of verses with a repetition: `$( ... ),+`"),
        ],
        pass: "the Macro Monk lifts the woodblock and presses it again, without a word.",
    },
    MissionRule {
        id: "drop_trait",
        npc: "the Sexton holds the snuffer",
        checks: &[
            check(&["impl Drop for"], "give `Soul` a destructor: `impl Drop for Soul`"),
            check(&["forget("], "keep `bound` from ever being released with `std::mem::forget`"),
        ],
        pass: "the Sexton snuffs the candles in order. \"last lit, first out — and one left burning.\"",
    },
    MissionRule {
        id: "send_sync",
        npc: "the Courier will not carry it",
        checks: &[
            check(&["unsafe impl Send"], "promise the tome may move between threads: `unsafe impl Send for Tome {}`"),
            check(&["unsafe impl Sync"], "promise it may be shared by reference: `unsafe impl Sync for Tome {}`"),
        ],
        pass: "the Courier shoulders the satchel. \"you vouched for it, so I carry it.\"",
    },
    MissionRule {
        id: "maybe_uninit",
        npc: "the Bookbinder holds up a blank page",
        checks: &[
            check(&["MaybeUninit"], "bind the quire blank with `MaybeUninit`"),
            check(&["write("], "fill each page with `write`, which never reads or drops the old bytes"),
            check(&["assume_init", "transmute"], "declare it filled with `assume_init` once every page is written"),
        ],
        pass: "the Bookbinder closes the quire. \"every page written before anyone read one.\"",
    },
    // ── Act 9 boss: the Librarian's trial (stages 1-3).
    MissionRule {
        id: "librarian_index",
        npc: "the Librarian blocks the stacks",
        checks: &[
            check(&["unsafe fn"], "keep `volume_at` an `unsafe fn` with its `# Safety` contract"),
            check(&[".len()"], "check the index against the shelf's `.len()` before the unsafe call"),
        ],
        pass: "the Librarian steps aside. \"a door no index can break. first test passed.\"",
    },
    MissionRule {
        id: "librarian_demand",
        npc: "the Librarian repeats his demand",
        checks: &[
            check(&["macro_rules!"], "answer with a `macro_rules!` macro"),
            check(&["$("], "take any number of arguments with a repetition: `$( ... )*`"),
        ],
        pass: "the Librarian hears his words returned. \"the incantation, exactly. second test passed.\"",
    },
    MissionRule {
        id: "librarian_seal",
        npc: "the Librarian holds the codex open",
        checks: &[
            check(&["impl Drop for"], "seal the codex: `impl Drop for Codex`"),
            check(&["from_raw"], "give the pages back to the heap with `Box::from_raw`"),
        ],
        pass: "the Librarian closes the codex. \"borrowed from the heap, returned to it. the Library is yours.\"",
    },
    // ── Act 10 (The Throne). `unsafe` and `#[global_allocator]` appear
    // only in the players' programs, never in these patterns' crate.
    MissionRule {
        id: "prealloc",
        npc: "the Assayer reads the scale",
        checks: &[
            check(&["with_capacity", "reserve", "collect"], "size the ledger before the loop: `Vec::with_capacity(1000)`"),
        ],
        pass: "the Assayer sets down the weights. \"one allocation. an honest measure.\"",
    },
    MissionRule {
        id: "cow_borrow",
        npc: "the Copyist lifts his quill",
        checks: &[
            check(&["Cow<", "Cow::"], "return a `Cow<'_, str>` instead of a fresh `String`"),
            check(&["Borrowed"], "hand back titles that need no change as `Cow::Borrowed`"),
        ],
        pass: "the Copyist lays down his quill. \"copy only what must change.\"",
    },
    MissionRule {
        id: "const_eval",
        npc: "the Astronomer squints at an empty sky",
        checks: &[
            check(&["const fn"], "let the compiler run `star_table`: make it a `const fn`"),
            check(&["const _", ": () = assert!"], "check the chart at compile time: `const _: () = assert!(...)`"),
        ],
        pass: "the Astronomer unrolls the chart. \"drawn before the night began.\"",
    },
    MissionRule {
        id: "chunks_exact",
        npc: "the Choirmaster cups an ear",
        checks: &[
            check(&["remainder", "chunks("], "the last voices fill no whole chunk: add `chunks.remainder()`"),
        ],
        pass: "the Choirmaster lowers the pipes. \"four lanes, and every soloist heard.\"",
    },
    MissionRule {
        id: "no_std",
        npc: "the Hermit shakes his head",
        checks: &[
            check(&["#![no_std]"], "the outpost has no standard library: keep `#![no_std]`"),
            check(&["core::"], "build the line with `core::fmt::Write` into a fixed buffer, not a `String`"),
        ],
        pass: "the Hermit trims the lantern. \"no heap, no harbour, and still it shines.\"",
    },
    MissionRule {
        id: "semver_caret",
        npc: "the Registrar checks the register",
        checks: &[
            check(&["> 0", "== 0", "!= 0", "(0,", ">= 1"], "below 1.0 the caret pins the first non-zero part: handle a zero major"),
        ],
        pass: "the Registrar stamps the entry. \"compatible, by the caret's rule.\"",
    },
    // ── Act 10 boss: the Compiler's trial (stages 1-3).
    MissionRule {
        id: "compiler_alloc",
        npc: "the Compiler weighs the proclamation",
        checks: &[
            check(&["with_capacity", "reserve", "join("], "know the length first: `String::with_capacity`"),
        ],
        pass: "the Compiler inclines its head. \"one allocation. proceed.\"",
    },
    MissionRule {
        id: "compiler_lifetime",
        npc: "the Compiler waits for an honest signature",
        checks: &[
            check(&["<$'_>", "<$'_,", "&$'_"], "name the lifetime the result borrows from: `fn herald<'a>(name: &'a str, ...)`"),
        ],
        pass: "the Compiler is silent, which is its highest praise.",
    },
    MissionRule {
        id: "compiler_bound",
        npc: "the Compiler waits on the Throne",
        checks: &[
            check(&["PartialOrd"], "bound `T` by what `>` needs: `PartialOrd`"),
        ],
        pass: "the Compiler rises from the Throne. \"it compiles. it was always yours.\"",
    },
];
// END MISSION TABLE
