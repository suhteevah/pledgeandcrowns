// SPDX-License-Identifier: MIT
//! Audit: the offline stub grader must accept every canonical solution
//! and reject obviously-wrong samples for every mission shipped in the
//! `MissionRegistry::default()` table. An unknown encounter id must
//! return `None` so the caller can fall back to `[client error]`.
//!
//! Canonical solutions are duplicated here intentionally — the
//! `contract.rs` cross-crate audit keeps the server-side grader honest;
//! this duplication keeps the client-side stub honest. If the two ever
//! drift, both audits will scream.

use pledge_and_crown::plugins::mission::MissionRegistry;
use pledge_and_crown::plugins::stub_grader::stub_verdict;

fn canonical_solution(encounter_id: &str) -> Option<&'static str> {
    Some(match encounter_id {
        "intro_let_binding" => "fn main() { let answer = 42; }",
        "double_function" => "fn double(n: i32) -> i32 { n * 2 }\nfn main() {}\n",
        "borrow_preview" => {
            r#"fn main() {
    let value = String::from("ok");
    let r = &value;
    println!("{r}");
}"#
        }
        "mut_binding" => "fn main() { let mut x = 0; x += 1; }",
        "if_else_sign" => {
            r#"fn sign(n: i32) -> &'static str {
    if n < 0 { "neg" } else if n == 0 { "zero" } else { "pos" }
}"#
        }
        "loop_break" => {
            r#"fn main() {
    let mut n = 1;
    let _ = loop {
        if n >= 100 { break n; }
        n *= 2;
    };
}"#
        }
        "match_option" => {
            r#"fn unwrap_or_zero(x: Option<i32>) -> i32 {
    match x {
        Some(n) => n,
        None => 0,
    }
}"#
        }
        "struct_basic" => {
            r#"struct Knight { name: String, hp: i32 }
fn main() {
    let k = Knight { name: String::from("Garin"), hp: 30 };
    let _ = k.name;
}"#
        }
        "vec_iter" => {
            r#"fn main() {
    let v = vec![1, 2, 3];
    let s: i32 = v.iter().sum();
    println!("{s}");
}"#
        }
        "tuple_destructure" => {
            r#"fn main() {
    let (a, b) = (3, 4);
    println!("{a} {b}");
}"#
        }
        "while_loop" => {
            r#"fn main() {
    let mut n = 5;
    while n > 0 { n -= 1; }
    println!("{n}");
}"#
        }
        "borrow_mut" => {
            r#"fn bump(x: &mut i32) { *x = 99; }
fn main() { let mut n = 0; bump(&mut n); println!("{n}"); }"#
        }
        "string_vs_str" => {
            r#"fn greet(name: &str) { println!("hi {name}"); }
fn main() { let s = String::from("you"); greet(&s); greet("anon"); }"#
        }
        "option_unwrap_or" => {
            r#"fn safe(x: Option<i32>) -> i32 { x.unwrap_or(0) }
fn main() { println!("{}", safe(Some(7))); }"#
        }
        "for_in_range" => {
            r#"fn main() {
    for i in 0..10 {
        println!("{i}");
    }
}"#
        }
        "closure_basic" => {
            r#"fn main() {
    let add = |a, b| a + b;
    println!("{}", add(2, 3));
}"#
        }
        "slice_basic" => {
            r#"fn sum_slice(xs: &[i32]) -> i32 { xs.iter().sum() }
fn main() { let _ = sum_slice(&[1, 2, 3]); }"#
        }
        "result_question_mark" => {
            r#"fn parse_int(s: &str) -> Result<i32, String> {
    let n = s.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(n)
}
fn main() { let _ = parse_int("42"); }"#
        }
        "derive_debug" => {
            r#"#[derive(Debug)]
struct Item { name: String }
fn main() { let item = Item { name: String::from("ring") }; println!("{item:?}"); }"#
        }
        "iter_map_collect" => {
            r#"fn main() {
    let v = vec![1, 2, 3];
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    let _ = doubled;
}"#
        }
        "enum_match" => {
            r#"enum Direction { North, South }
fn name(d: Direction) -> &'static str {
    match d {
        Direction::North => "n",
        Direction::South => "s",
    }
}
fn main() { let _ = name(Direction::North); }"#
        }
        // ── Act 3: Guildhall Quarter (mirror of contract.rs).
        "impl_method" => {
            r#"struct Hero { level: i32 }
impl Hero {
    fn power(&self) -> i32 { self.level * 10 }
}
fn main() {
    let h = Hero { level: 3 };
    println!("{}", h.power());
}"#
        }
        "assoc_new" => {
            r#"struct Hero { level: i32 }
impl Hero {
    fn new(level: i32) -> Self { Self { level } }
}
fn main() {
    let h = Hero::new(5);
    println!("{}", h.level);
}"#
        }
        "if_let" => {
            r#"fn main() {
    let maybe: Option<i32> = Some(7);
    if let Some(n) = maybe {
        println!("{n}");
    }
}"#
        }
        "while_let" => {
            r#"fn main() {
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("{top}");
    }
}"#
        }
        "tuple_struct" => {
            r#"struct Meters(f64);
fn main() {
    let d = Meters(3.5);
    println!("{}", d.0);
}"#
        }
        "enum_data_match" => {
            r#"enum Item {
    Weapon { damage: i32 },
    Potion { heal: i32 },
}
fn value(item: Item) -> i32 {
    match item {
        Item::Weapon { damage } => damage,
        Item::Potion { heal } => heal,
    }
}
fn main() {
    let _ = value(Item::Weapon { damage: 10 });
}"#
        }
        // ── Act 4: Trait Mage's Tower (mirror of contract.rs).
        "trait_def" => {
            r#"trait Element {
    fn name(&self) -> &str;
}
struct Fire;
impl Element for Fire {
    fn name(&self) -> &str { "fire" }
}
fn main() {
    let f = Fire;
    println!("{}", f.name());
}"#
        }
        "generic_fn" => {
            r#"fn larger<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
fn main() {
    println!("{}", larger(3, 7));
}"#
        }
        "generic_struct" => {
            r#"struct Pair<T> {
    a: T,
    b: T,
}
fn main() {
    let p = Pair { a: 1, b: 2 };
    let _ = (p.a, p.b);
}"#
        }
        "dyn_trait" => {
            r#"trait Element {
    fn name(&self) -> &str;
}
struct Fire;
impl Element for Fire {
    fn name(&self) -> &str { "fire" }
}
struct Water;
impl Element for Water {
    fn name(&self) -> &str { "water" }
}
fn main() {
    let zoo: Vec<Box<dyn Element>> = vec![Box::new(Fire), Box::new(Water)];
    for e in &zoo {
        println!("{}", e.name());
    }
}"#
        }
        "lifetimes" => {
            r#"fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
fn main() {
    let _ = longest("aa", "b");
}"#
        }
        "assoc_type" => {
            r#"trait Producer {
    type Output;
    fn make(&self) -> Self::Output;
}
struct Coiner;
impl Producer for Coiner {
    type Output = i32;
    fn make(&self) -> Self::Output { 7 }
}
fn main() {
    let c = Coiner;
    let _ = c.make();
}"#
        }
        // ── Act 5: Tavern of Tribulations (mirror of contract.rs).
        "result_match" => {
            r#"fn describe(r: Result<i32, String>) -> i32 {
    match r {
        Ok(v) => v,
        Err(_e) => -1,
    }
}
fn main() {
    let _ = describe(Ok(5));
}"#
        }
        "custom_error" => {
            r#"enum BrewError {
    TooHot,
    TooCold,
}
fn check(temp: i32) -> Result<i32, BrewError> {
    if temp > 100 {
        Err(BrewError::TooHot)
    } else if temp < 0 {
        Err(BrewError::TooCold)
    } else {
        Ok(temp)
    }
}
fn main() {
    let _ = check(50);
}"#
        }
        "from_error" => {
            r#"struct ParseFail;
enum AppError {
    Parse,
}
impl From<ParseFail> for AppError {
    fn from(_e: ParseFail) -> Self {
        AppError::Parse
    }
}
fn main() {
    let _e: AppError = ParseFail.into();
}"#
        }
        "option_map" => {
            r#"fn add_one(o: Option<i32>) -> Option<i32> {
    o.map(|x| x + 1)
}
fn main() {
    let _ = add_one(Some(3));
}"#
        }
        "and_then" => {
            r#"fn half(n: i32) -> Option<i32> {
    if n % 2 == 0 { Some(n / 2) } else { None }
}
fn main() {
    let _ = Some(8).and_then(half).and_then(half);
}"#
        }
        "unwrap_or_else" => {
            r#"fn value(o: Option<i32>) -> i32 {
    o.unwrap_or_else(|| 0)
}
fn main() {
    let _ = value(None);
}"#
        }
        // ── Act 6: Iterator Forge (mirror of contract.rs).
        "hashmap_basic" => {
            r#"use std::collections::HashMap;
fn main() {
    let mut scores = HashMap::new();
    scores.insert("ace", 1);
    let _ = scores.get("ace");
}"#
        }
        "iter_filter" => {
            r#"fn main() {
    let v = vec![1, 2, 3, 4];
    let evens: Vec<&i32> = v.iter().filter(|x| **x % 2 == 0).collect();
    let _ = evens;
}"#
        }
        "iter_fold" => {
            r#"fn main() {
    let v = vec![1, 2, 3];
    let total = v.iter().fold(0, |acc, x| acc + x);
    let _ = total;
}"#
        }
        "iter_enumerate" => {
            r#"fn main() {
    let v = vec![10, 20, 30];
    for (i, x) in v.iter().enumerate() {
        let _ = (i, x);
    }
}"#
        }
        "iter_zip" => {
            r#"fn main() {
    let a = vec![1, 2];
    let b = vec![3, 4];
    for (x, y) in a.iter().zip(b.iter()) {
        let _ = (x, y);
    }
}"#
        }
        "closure_move" => {
            r#"fn main() {
    let name = String::from("Garin");
    let greet = move || println!("hail, {name}");
    greet();
}"#
        }
        // ── Act 7: Concurrent Coast (mirror of contract.rs).
        "thread_spawn" => {
            r#"use std::thread;
fn main() {
    let handle = thread::spawn(|| 21 * 2);
    let _ = handle.join();
}"#
        }
        "arc_mutex" => {
            r#"use std::sync::{Arc, Mutex};
fn main() {
    let shared = Arc::new(Mutex::new(0));
    {
        let mut guard = shared.lock().unwrap();
        *guard += 1;
    }
    let _ = shared;
}"#
        }
        "mpsc_channel" => {
            r#"use std::sync::mpsc;
fn main() {
    let (tx, rx) = mpsc::channel();
    tx.send(7).unwrap();
    let _ = rx.recv();
}"#
        }
        "atomic" => {
            r#"use std::sync::atomic::{AtomicUsize, Ordering};
fn main() {
    let counter = AtomicUsize::new(0);
    counter.fetch_add(1, Ordering::SeqCst);
    let _ = counter;
}"#
        }
        "thread_scope" => {
            r#"use std::thread;
fn main() {
    let data = vec![1, 2, 3];
    thread::scope(|s| {
        s.spawn(|| {
            let _ = &data;
        });
    });
}"#
        }
        "async_fn" => {
            r#"async fn double(x: i32) -> i32 {
    x * 2
}
async fn run() -> i32 {
    double(21).await
}
fn main() {
    let _future = run();
}"#
        }
        // ── Act 8: Vault of Pointers (mirror of contract.rs).
        "box_basic" => {
            r#"struct Node {
    value: i32,
    next: Option<Box<Node>>,
}
fn main() {
    let list = Node {
        value: 1,
        next: Some(Box::new(Node { value: 2, next: None })),
    };
    let _ = list;
}"#
        }
        "rc_basic" => {
            r#"use std::rc::Rc;
fn main() {
    let original = Rc::new(String::from("hoard"));
    let shared = Rc::clone(&original);
    let _ = (original, shared);
}"#
        }
        "refcell" => {
            r#"use std::cell::RefCell;
fn main() {
    let guarded = RefCell::new(0);
    *guarded.borrow_mut() += 1;
    let _ = guarded.borrow();
}"#
        }
        "cell" => {
            r#"use std::cell::Cell;
fn main() {
    let slot = Cell::new(1);
    slot.set(5);
    let _ = slot.get();
}"#
        }
        "rc_refcell" => {
            r#"use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let shared = Rc::new(RefCell::new(0));
    let clone = Rc::clone(&shared);
    *clone.borrow_mut() += 1;
    let _ = shared.borrow();
}"#
        }
        "weak_ref" => {
            r#"use std::rc::{Rc, Weak};
fn main() {
    let strong = Rc::new(5);
    let weak: Weak<i32> = Rc::downgrade(&strong);
    let _ = weak.upgrade();
}"#
        }
        _ => return None,
    })
}

#[test]
fn stub_covers_every_mission_in_the_registry() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        assert!(
            canonical_solution(m.id).is_some(),
            "stub_grader test fixtures missing canonical_solution for `{}`",
            m.id
        );
        let v = stub_verdict(m.id, "fn main() {}");
        assert!(
            v.is_some(),
            "stub_verdict returned None for shipped mission `{}` — add an arm",
            m.id
        );
    }
}

#[test]
fn stub_passes_every_canonical_solution() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src = canonical_solution(m.id).expect("checked above");
        let v = stub_verdict(m.id, src).expect("registry coverage checked above");
        assert!(
            v.ok,
            "stub rejected canonical solution for `{}`: {v:?}",
            m.id
        );
        // Stub markers are load-bearing for the UI banner contract.
        assert!(
            v.stdout.starts_with("[stub] "),
            "stub pass for `{}` did not carry the [stub] prefix: {v:?}",
            m.id
        );
        assert!(v.stderr.is_empty());
    }
}

#[test]
fn stub_fails_obviously_wrong_input() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let v = stub_verdict(m.id, "fn main() {}").expect("registry coverage checked");
        assert!(
            !v.ok,
            "stub passed empty `fn main() {{}}` for mission `{}` — needles are too lax",
            m.id
        );
        assert!(
            v.stderr.starts_with("[stub] "),
            "stub fail for `{}` did not carry the [stub] prefix: {v:?}",
            m.id
        );
        assert!(v.stdout.is_empty());
    }
}

#[test]
fn stub_returns_none_for_unknown_encounter() {
    assert!(stub_verdict("totally_made_up_id", "fn main() {}").is_none());
    assert!(stub_verdict("", "anything").is_none());
}

#[test]
fn stub_does_not_cross_grade_solutions() {
    // Same anti-leak guarantee the server-side grader has: solving one
    // mission must not pass another.
    let intro = "fn main() { let answer = 42; }";
    assert!(!stub_verdict("double_function", intro).unwrap().ok);
    assert!(!stub_verdict("borrow_preview", intro).unwrap().ok);

    let double = "fn double(n: i32) -> i32 { n * 2 }";
    assert!(!stub_verdict("intro_let_binding", double).unwrap().ok);

    let match_sol = "fn f(x: Option<i32>) -> i32 { match x { Some(n) => n, None => 0 } }";
    assert!(!stub_verdict("loop_break", match_sol).unwrap().ok);
    assert!(!stub_verdict("struct_basic", match_sol).unwrap().ok);
}

#[test]
fn rule_table_is_well_formed_and_matches_the_registry() {
    use pledge_and_crown::plugins::stub_grader::MISSION_RULES;
    use std::collections::BTreeSet;
    let mut ids = BTreeSet::new();
    for rule in MISSION_RULES {
        assert!(ids.insert(rule.id), "duplicate rule id `{}`", rule.id);
        assert!(!rule.checks.is_empty(), "rule `{}` has no checks", rule.id);
        for c in rule.checks {
            assert!(
                !c.any_of.is_empty(),
                "rule `{}` has an empty check",
                rule.id
            );
        }
    }
    let registry: BTreeSet<&str> = MissionRegistry::default()
        .missions
        .iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, registry, "rule table and MissionRegistry disagree");
}

// ── Lexer ───────────────────────────────────────────────────────────
//
// The grader matches tokens of CODE only, so the lexer must never mistake
// a comment or a literal for code (or the reverse). The server copy of the
// engine is test-enforced textually identical (`contract.rs`), so this
// suite covers both graders.

use pledge_and_crown::plugins::stub_grader::{Token, TokenKind, code_contains, lex};

fn kinds_texts(src: &str) -> Vec<(TokenKind, String)> {
    lex(src).into_iter().map(|t| (t.kind, t.text)).collect()
}

fn texts(src: &str) -> Vec<String> {
    lex(src).into_iter().map(|t| t.text).collect()
}

fn tok(kind: TokenKind, text: &str) -> (TokenKind, String) {
    (kind, text.to_string())
}

use TokenKind::{Ident, Lifetime, Literal, Punct, Str};

#[test]
fn lex_line_and_doc_comments_vanish() {
    assert_eq!(texts("a // b c\nd"), ["a", "d"]);
    assert_eq!(texts("/// doc\n//! inner\nx"), ["x"]);
    assert_eq!(texts("x // no newline at end"), ["x"]);
}

#[test]
fn lex_block_comments_nest() {
    assert_eq!(texts("a /* b /* c */ d */ e"), ["a", "e"]);
    assert_eq!(texts("/*/ still comment */ x"), ["x"]);
    assert_eq!(texts("/** doc */ x /**/ y"), ["x", "y"]);
}

#[test]
fn lex_unterminated_block_comment_runs_to_end() {
    assert_eq!(texts("a /* b /* c */ d"), ["a"]);
    assert_eq!(texts("/*"), Vec::<String>::new());
}

#[test]
fn lex_comment_markers_inside_strings_are_string() {
    assert_eq!(
        kinds_texts("\"a // b\" x"),
        [tok(Str, "a // b"), tok(Ident, "x")]
    );
    assert_eq!(
        kinds_texts("\"/*\" x \"*/\""),
        [tok(Str, "/*"), tok(Ident, "x"), tok(Str, "*/")]
    );
}

#[test]
fn lex_escaped_quote_stays_inside_string() {
    assert_eq!(
        kinds_texts(r#""a\"b" x"#),
        [tok(Str, r#"a\"b"#), tok(Ident, "x")]
    );
    assert_eq!(
        kinds_texts(r#""a\\" x"#),
        [tok(Str, r"a\\"), tok(Ident, "x")]
    );
}

#[test]
fn lex_double_quote_inside_char_literal() {
    assert_eq!(
        kinds_texts(r#"let q = '"'; x "y""#),
        [
            tok(Ident, "let"),
            tok(Ident, "q"),
            tok(Punct, "="),
            tok(Literal, "'\"'"),
            tok(Punct, ";"),
            tok(Ident, "x"),
            tok(Str, "y"),
        ]
    );
}

#[test]
fn lex_char_literal_escapes() {
    for lit in [r"'\''", r"'\\'", r"'\n'", r"'\u{1F600}'", r"'\x41'", "'é'"] {
        let src = format!("{lit} x");
        assert_eq!(
            kinds_texts(&src),
            [tok(Literal, lit), tok(Ident, "x")],
            "{lit}"
        );
    }
}

#[test]
fn lex_lifetimes_are_not_char_literals() {
    assert_eq!(
        kinds_texts("fn f<'a>(x: &'a str) -> &'static str"),
        [
            tok(Ident, "fn"),
            tok(Ident, "f"),
            tok(Punct, "<"),
            tok(Lifetime, "'a"),
            tok(Punct, ">"),
            tok(Punct, "("),
            tok(Ident, "x"),
            tok(Punct, ":"),
            tok(Punct, "&"),
            tok(Lifetime, "'a"),
            tok(Ident, "str"),
            tok(Punct, ")"),
            tok(Punct, "->"),
            tok(Punct, "&"),
            tok(Lifetime, "'static"),
            tok(Ident, "str"),
        ]
    );
    assert_eq!(
        kinds_texts("'outer: loop { break 'outer; }")[0],
        tok(Lifetime, "'outer")
    );
    // One letter between quotes is a char, not a lifetime.
    assert_eq!(
        kinds_texts("'a' 'b"),
        [tok(Literal, "'a'"), tok(Lifetime, "'b")]
    );
    assert_eq!(kinds_texts("'_")[0], tok(Lifetime, "'_"));
}

#[test]
fn lex_raw_strings_with_any_number_of_hashes() {
    assert_eq!(
        kinds_texts(r#"r"a\" x"#),
        [tok(Str, r"a\"), tok(Ident, "x")],
        "raw strings have no escapes"
    );
    assert_eq!(
        kinds_texts(r###"r#"he said "hi""# x"###),
        [tok(Str, r#"he said "hi""#), tok(Ident, "x")]
    );
    assert_eq!(
        kinds_texts(r###"r##"a "# b"## x"###),
        [tok(Str, r##"a "# b"##), tok(Ident, "x")]
    );
    assert_eq!(
        kinds_texts(r###"r#"// not a comment /* nor this"# x"###),
        [tok(Str, "// not a comment /* nor this"), tok(Ident, "x")]
    );
}

#[test]
fn lex_byte_and_c_strings_and_byte_chars() {
    assert_eq!(
        kinds_texts(r###"b"x" br#"y"# c"z" cr"w" b'q' b'\n'"###),
        [
            tok(Str, "x"),
            tok(Str, "y"),
            tok(Str, "z"),
            tok(Str, "w"),
            tok(Literal, "b'q'"),
            tok(Literal, r"b'\n'"),
        ]
    );
    // Identifiers that merely start with b / c / r are identifiers.
    assert_eq!(
        texts("break crate return r b c"),
        ["break", "crate", "return", "r", "b", "c"]
    );
}

#[test]
fn lex_raw_identifiers() {
    assert_eq!(
        kinds_texts("r#type x"),
        [tok(Ident, "type"), tok(Ident, "x")]
    );
}

#[test]
fn lex_unterminated_constructs_run_to_end_without_panicking() {
    assert_eq!(kinds_texts("x \"abc"), [tok(Ident, "x"), tok(Str, "abc")]);
    assert_eq!(kinds_texts("x \"ab\\"), [tok(Ident, "x"), tok(Str, "ab\\")]);
    assert_eq!(
        kinds_texts("x r#\"abc\""),
        [tok(Ident, "x"), tok(Str, "abc\"")]
    );
    assert_eq!(kinds_texts("x '\\"), [tok(Ident, "x"), tok(Literal, "'\\")]);
    assert_eq!(kinds_texts("x '"), [tok(Ident, "x"), tok(Punct, "'")]);
    assert_eq!(kinds_texts("b'"), [tok(Punct, "'")]);
}

#[test]
fn lex_every_prefix_of_a_tricky_program_without_panicking() {
    let tricky = r####"fn f<'a>(s: &'a str) -> char { /* /* */ */ let _ = r##"x"#"##; let _ = b"\"";
    let c = '\''; let d = '"'; let e = 'é'; // tail "
    let _ = br#"y"#; let _ = c"z"; let _ = r#type; 'l: loop { break 'l; } 3.0_f64 + d.0 as f64; c }"####;
    let chars: Vec<char> = tricky.chars().collect();
    for n in 0..=chars.len() {
        let prefix: String = chars[..n].iter().collect();
        let _ = lex(&prefix);
    }
    // And the whole thing lexes the comment and literals away.
    let code = texts(tricky);
    assert!(code.iter().all(|t| !t.contains("/*")));
    assert!(code.contains(&"'l".to_string()));
}

#[test]
fn lex_numbers_ranges_and_tuple_fields() {
    assert_eq!(kinds_texts("3.0"), [tok(Literal, "3.0")]);
    assert_eq!(texts("d.0"), ["d", ".", "0"]);
    assert_eq!(texts("t.0.1"), ["t", ".", "0", ".", "1"]);
    assert_eq!(texts("0..10"), ["0", "..", "10"]);
    assert_eq!(texts("0..=9"), ["0", "..=", "9"]);
    assert_eq!(texts("1_000u32 0x2A"), ["1_000u32", "0x2A"]);
}

#[test]
fn lex_operators() {
    assert_eq!(texts("x += 1"), texts("x+=1"));
    assert_eq!(texts("a::b->c=>d"), ["a", "::", "b", "->", "c", "=>", "d"]);
    // `&&` and `||` stay split so `&value` / `|x|` patterns see through them.
    assert_eq!(texts("&&x || y"), ["&", "&", "x", "|", "|", "y"]);
    assert_eq!(
        texts("Vec<Box<dyn E>>"),
        ["Vec", "<", "Box", "<", "dyn", "E", ">", ">"]
    );
}

#[test]
fn lex_token_type_is_constructible() {
    // Token is public so callers (and these tests) can inspect lexer output.
    let t = Token {
        kind: Ident,
        text: "x".into(),
    };
    assert_eq!(lex("x"), [t]);
}

// ── Matcher ─────────────────────────────────────────────────────────

#[test]
fn match_is_whitespace_insensitive() {
    for src in ["n *= 2", "n*=2", "n *=\n    2"] {
        assert!(code_contains(src, "*= 2"), "{src}");
    }
    assert!(!code_contains("n * = 2", "*= 2"), "`* =` is not `*=`");
}

#[test]
fn match_ignores_comments_and_strings() {
    assert!(!code_contains("// let answer = 42", "let answer"));
    assert!(!code_contains("/* let answer */", "let answer"));
    assert!(!code_contains("\"let answer\"", "let answer"));
    assert!(code_contains("let s = \"neg\";", "\"neg\""));
    assert!(!code_contains("let s = \"negative\";", "\"neg\""));
}

#[test]
fn match_identifiers_whole_not_prefix() {
    assert!(!code_contains("RefCell::new(0)", "Cell::new"));
    assert!(code_contains("Cell::new(0)", "Cell::new"));
    assert!(!code_contains("let answers = 1;", "let answer"));
}

#[test]
fn match_wildcards_and_captures() {
    assert!(code_contains("Some(v) => v", "Some($X) => $X"));
    assert!(!code_contains("Some(v) => 0", "Some($X) => $X"));
    assert!(code_contains("impl Element for Fire", "impl $_ for"));
    assert!(!code_contains("impl Fire {", "impl $_ for"));
    assert!(code_contains("<'s> (x: &'s str)", "<$'a $.. &$'a"));
    assert!(!code_contains("<'s> (x: &'t str)", "<$'a $.. &$'a"));
    assert!(code_contains("<'s>", "<$'_>"));
}

#[test]
fn match_gaps_stay_inside_their_bracket_group() {
    assert!(code_contains(
        "#[derive(Clone, Debug)]",
        "#[derive( $.. Debug"
    ));
    assert!(!code_contains(
        "#[derive(Clone)] struct S; impl Debug for S {}",
        "#[derive( $.. Debug"
    ));
    assert!(code_contains("f(a, (b), c)", "f( $.. c"));
    assert!(!code_contains("f(a) c", "f( $.. c"));
}
