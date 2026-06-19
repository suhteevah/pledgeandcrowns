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
