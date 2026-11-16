// SPDX-License-Identifier: MIT
//! Player-facing reference solutions, one per mission id.
//!
//! These strings are shown to a stuck player behind the "Show solution"
//! gate, so they are teaching artifacts, not test fixtures:
//!
//! - every entry is a complete program with `fn main` that builds with
//!   plain `rustc` on stable at the server's edition (2024, see
//!   `game/tests/common/mod.rs`), no dependencies, no warnings;
//! - every entry starts from its mission's starter-code shape and does
//!   what the mission's Task section asks;
//! - comments explain the one concept the mission teaches.
//!
//! `game/tests/solutions.rs` proves each entry passes the offline stub
//! grader and the server pattern grader, and (`--ignored`) that each one
//! really compiles natively and for `wasm32-wasip1`.

/// `(mission_id, solution_source)` in registry order.
pub(super) const SOLUTIONS: &[(&str, &str)] = &[
    // ── Prelude ──────────────────────────────────────────────────────
    (
        "intro_let_binding",
        r#"fn main() {
    // `let` gives a value a name. The binding is immutable by default.
    let answer = 42;
    println!("{answer}");
}
"#,
    ),
    (
        "double_function",
        r#"// The last expression in the body (no semicolon) is the return value.
fn double(n: i32) -> i32 {
    n * 2
}

fn main() {
    println!("{}", double(21));
}
"#,
    ),
    (
        "borrow_preview",
        r#"fn main() {
    let value = String::from("sample");
    // `&value` borrows the String. `value` still owns the data.
    let r = &value;
    println!("{r}");
}
"#,
    ),
    (
        "mut_binding",
        r#"fn main() {
    // `mut` opts this binding in to being changed.
    let mut x = 0;
    x += 1;
    println!("{x}");
}
"#,
    ),
    (
        "if_else_sign",
        r#"// `if` is an expression: the branch that runs is the function's value.
// Every branch must produce the same type (here `&'static str`).
fn sign(n: i32) -> &'static str {
    if n < 0 {
        "neg"
    } else if n == 0 {
        "zero"
    } else {
        "pos"
    }
}

fn main() {
    println!("{}", sign(-3));
}
"#,
    ),
    (
        "loop_break",
        r#"fn main() {
    let mut n = 1;
    // `loop` is the only loop that can produce a value: `break n` hands it out.
    let first_big = loop {
        if n >= 100 {
            break n;
        }
        n *= 2;
    };
    println!("{first_big}");
}
"#,
    ),
    (
        "match_option",
        r#"fn unwrap_or_zero(x: Option<i32>) -> i32 {
    // `match` must cover every case. Leave out the `None` arm and it won't compile.
    match x {
        Some(n) => n,
        None => 0,
    }
}

fn main() {
    println!("{}", unwrap_or_zero(Some(7)));
    println!("{}", unwrap_or_zero(None));
}
"#,
    ),
    (
        "struct_basic",
        r#"// A struct groups related values under one type, each with a field name.
struct Knight {
    name: String,
    hp: i32,
}

fn main() {
    let knight = Knight {
        name: String::from("Garin"),
        hp: 30,
    };
    println!("{} has {} hp", knight.name, knight.hp);
}
"#,
    ),
    (
        "vec_iter",
        r#"fn main() {
    let v = vec![10, 20, 30];
    // The `: i32` annotation tells `sum` what type to add into.
    let total: i32 = v.iter().sum();
    println!("{total}");
}
"#,
    ),
    (
        "tuple_destructure",
        r#"fn main() {
    // The pattern on the left has the same shape as the tuple on the right.
    let (a, b) = (3, 4);
    println!("{a} {b}");
}
"#,
    ),
    (
        "borrow_mut",
        r#"// `&mut i32` is an exclusive borrow: `bump` may change the caller's value.
fn bump(x: &mut i32) {
    // `*x` follows the reference so we write to the value it points at.
    *x += 1;
}

fn main() {
    let mut n = 0;
    bump(&mut n);
    println!("{n}");
}
"#,
    ),
    (
        "string_vs_str",
        r#"// Taking `&str` accepts string literals AND borrowed `String`s.
fn greet(name: &str) {
    println!("hi {name}");
}

fn main() {
    let owned = String::from("world");
    greet(&owned); // `&String` coerces to `&str`
    greet("traveller"); // a literal is already a `&'static str`
}
"#,
    ),
    (
        "option_unwrap_or",
        r#"fn safe(x: Option<i32>) -> i32 {
    // The value inside `Some`, or the default `0` when it is `None`.
    x.unwrap_or(0)
}

fn main() {
    println!("{}", safe(Some(5)));
    println!("{}", safe(None));
}
"#,
    ),
    (
        "for_in_range",
        r#"fn main() {
    // `0..10` is half-open: it yields 0 up to 9, never 10.
    for i in 0..10 {
        println!("{i}");
    }
}
"#,
    ),
    (
        "closure_basic",
        r#"fn main() {
    // A closure is an anonymous function stored in a variable.
    // The parameter types are inferred from the call below.
    let add = |a, b| a + b;
    println!("{}", add(19, 23));
}
"#,
    ),
    (
        "slice_basic",
        r#"// `&[i32]` borrows any run of i32s: an array, a Vec, or part of one.
fn sum_slice(xs: &[i32]) -> i32 {
    xs.iter().sum()
}

fn main() {
    println!("{}", sum_slice(&[1, 2, 3]));
    let v = vec![4, 5, 6];
    println!("{}", sum_slice(&v)); // `&Vec<i32>` coerces to `&[i32]`
}
"#,
    ),
    (
        "result_question_mark",
        r#"fn parse_int(s: &str) -> Result<i32, String> {
    // On `Err`, `?` returns it from `parse_int` right away.
    // On `Ok`, `?` unwraps the number and execution continues.
    let n = s.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(n)
}

fn main() {
    println!("{:?}", parse_int("42"));
    println!("{:?}", parse_int("forty-two"));
}
"#,
    ),
    (
        "derive_debug",
        r#"// `derive` writes the `Debug` impl for us from the struct's fields.
#[derive(Debug)]
struct Item {
    name: String,
}

fn main() {
    let item = Item {
        name: String::from("ring"),
    };
    println!("{item:?}");
    println!("the chronicle names a {}", item.name);
}
"#,
    ),
    (
        "iter_map_collect",
        r#"fn main() {
    let v = vec![1, 2, 3];
    // `map` is lazy. `collect` runs the chain and builds the new Vec;
    // the `Vec<i32>` annotation tells it which container to build.
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    println!("{doubled:?}");
}
"#,
    ),
    (
        "enum_match",
        r#"// A value of an enum is exactly one of its variants.
enum Direction {
    North,
    East,
    South,
    West,
}

fn degrees(d: Direction) -> i32 {
    // One arm per variant. Add a variant later and this match stops compiling
    // until you handle it.
    match d {
        Direction::North => 0,
        Direction::East => 90,
        Direction::South => 180,
        Direction::West => 270,
    }
}

fn main() {
    for d in [Direction::North, Direction::East, Direction::South, Direction::West] {
        println!("{}", degrees(d));
    }
}
"#,
    ),
    (
        "while_loop",
        r#"fn main() {
    let mut n = 5;
    // Runs while the condition is true. The body must move `n` toward the exit.
    while n > 0 {
        println!("{n}");
        n -= 1;
    }
    println!("{n}");
}
"#,
    ),
    // ── Act 2 boss: the Borrow Checker's trial, The Bridge ───────────
    (
        "bridge_toll",
        r#"// `&str` borrows: `weigh` reads the cargo and gives it back untouched.
fn weigh(cargo: &str) -> usize {
    cargo.len()
}

fn main() {
    let cargo = String::from("lantern oil");
    // Lend with `&`: `main` still owns `cargo` after the call.
    let toll = weigh(&cargo);
    println!("toll: {toll}");
    println!("cargo: {cargo}");
}
"#,
    ),
    (
        "bridge_quill",
        r#"// `&mut` is the one exclusive borrow: only `inscribe` may write while it runs.
fn inscribe(scroll: &mut Vec<String>, entry: &str) {
    scroll.push(entry.to_string());
}

fn main() {
    let mut scroll = vec![String::from("north road")];
    let first = &scroll[0];
    let second = &scroll[0];
    // Two shared borrows at once are fine. Their last use is here...
    println!("reader one: {first}");
    println!("reader two: {second}");
    // ...so the exclusive borrow may start now.
    inscribe(&mut scroll, "east road");
    println!("scroll: {}", scroll.join(", "));
}
"#,
    ),
    (
        "bridge_lantern",
        r#"// `'a` ties the answer to `line` only; `sep` is just read.
fn before<'a>(line: &'a str, sep: &str) -> &'a str {
    match line.find(sep) {
        Some(i) => &line[..i],
        None => line,
    }
}

fn main() {
    let lantern = String::from("oil:wick:glass");
    let part;
    {
        let sep = String::from(":");
        part = before(&lantern, &sep);
    } // `sep` is dropped here, but `part` borrows from `lantern`, not `sep`.
    println!("first: {part}");
    println!("whole: {}", before("glass", ":"));
}
"#,
    ),
    // ── Act 3: The Guildhall Quarter ─────────────────────────────────
    (
        "impl_method",
        r#"struct Hero {
    level: i32,
}

impl Hero {
    // `&self` borrows the hero, so calling `power` doesn't use it up.
    fn power(&self) -> i32 {
        self.level * 10
    }
}

fn main() {
    let h = Hero { level: 3 };
    println!("{}", h.power());
}
"#,
    ),
    (
        "assoc_new",
        r#"struct Hero {
    level: i32,
}

impl Hero {
    // No `self` parameter: this belongs to the type and is called as `Hero::new`.
    fn new(level: i32) -> Self {
        Self { level }
    }
}

fn main() {
    let h = Hero::new(5);
    println!("{}", h.level);
}
"#,
    ),
    (
        "if_let",
        r#"fn main() {
    let maybe: Option<i32> = Some(7);
    // The block runs only if `maybe` matches `Some(n)`. `None` is skipped.
    if let Some(n) = maybe {
        println!("{n}");
    }
}
"#,
    ),
    (
        "while_let",
        r#"fn main() {
    let mut stack = vec![1, 2, 3];
    // `pop` returns `Some(top)` until the vec is empty, then `None` ends the loop.
    while let Some(top) = stack.pop() {
        println!("{top}");
    }
}
"#,
    ),
    (
        "tuple_struct",
        r#"// A one-field tuple struct (a "newtype"): `Meters` is its own type,
// so the compiler won't mix it up with a bare f64.
struct Meters(f64);

fn main() {
    let raw: f64 = 3.5;
    let distance = Meters(raw);
    println!("{}", distance.0);
}
"#,
    ),
    (
        "enum_data_match",
        r#"// Each variant carries its own data.
enum Item {
    Weapon { damage: i32 },
    Potion { heal: i32 },
}

fn value(item: Item) -> i32 {
    // Each arm names a variant and binds the data inside it.
    match item {
        Item::Weapon { damage } => damage,
        Item::Potion { heal } => heal,
    }
}

fn main() {
    println!("{}", value(Item::Weapon { damage: 10 }));
    println!("{}", value(Item::Potion { heal: 5 }));
}
"#,
    ),
    // ── Act 3 boss: the Guildmaster's trial, The Variant Wraith ──────
    (
        "wraith_forms",
        r#"#[derive(Debug, Clone, Copy)]
enum Wraith {
    Solid,
    Mist,
    Shadow,
}

// One arm per form and no `_`: the compiler checks none is missed.
fn counter(form: Wraith) -> &'static str {
    match form {
        Wraith::Solid => "steel",
        Wraith::Mist => "wind",
        Wraith::Shadow => "light",
    }
}

fn main() {
    let forms = [Wraith::Solid, Wraith::Mist, Wraith::Shadow];
    for form in forms {
        println!("{form:?}: {}", counter(form));
    }
}
"#,
    ),
    (
        "wraith_essence",
        r#"#[derive(Debug, Clone, Copy)]
enum Wraith {
    Solid { armor: u32 },
    Mist { density: u32 },
    Shadow,
}

// Each pattern opens its variant and binds the data it carries.
fn damage(form: Wraith, blow: u32) -> u32 {
    match form {
        Wraith::Solid { armor } => blow - armor,
        Wraith::Mist { density } => blow / density,
        Wraith::Shadow => 0,
    }
}

fn main() {
    let forms = [
        Wraith::Solid { armor: 3 },
        Wraith::Mist { density: 2 },
        Wraith::Shadow,
    ];
    for form in forms {
        println!("{form:?} takes {}", damage(form, 10));
    }
}
"#,
    ),
    (
        "wraith_possessed",
        r#"#[derive(Debug, Clone, Copy)]
enum Wraith {
    Solid { armor: u32 },
    Mist { density: u32 },
    Shadow,
    Possessed { will: u32 },
}

fn damage(form: Wraith, blow: u32) -> u32 {
    match form {
        Wraith::Solid { armor } => blow - armor,
        Wraith::Mist { density } => blow / density,
        Wraith::Shadow => 0,
        // The new form, named. A fifth form would need its own arm too.
        Wraith::Possessed { will } => blow + will,
    }
}

fn main() {
    let forms = [
        Wraith::Solid { armor: 3 },
        Wraith::Mist { density: 2 },
        Wraith::Shadow,
        Wraith::Possessed { will: 4 },
    ];
    for form in forms {
        println!("{form:?} takes {}", damage(form, 10));
    }
}
"#,
    ),
    // ── Act 4: The Trait Mage's Tower ────────────────────────────────
    (
        "trait_def",
        r#"// A trait names a capability. `impl Trait for Type` grants it to a type.
trait Element {
    fn name(&self) -> &str;
}

struct Fire;

impl Element for Fire {
    fn name(&self) -> &str {
        "fire"
    }
}

fn main() {
    let f = Fire;
    println!("{}", f.name());
}
"#,
    ),
    (
        "generic_fn",
        r#"// `T: PartialOrd` means "any type whose values can be compared with `>`".
fn larger<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn main() {
    println!("{}", larger(3, 7)); // T = i32
    println!("{}", larger(1.5, 0.5)); // T = f64
}
"#,
    ),
    (
        "generic_struct",
        r#"// The caller picks `T`. Both fields share that one type.
struct Pair<T> {
    a: T,
    b: T,
}

fn main() {
    let p = Pair { a: 1, b: 2 }; // Pair<i32>, inferred
    println!("{} {}", p.a, p.b);
}
"#,
    ),
    (
        "dyn_trait",
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
    // `Box<dyn Element>` hides the concrete type, so Fire and Water fit in one Vec.
    let elements: Vec<Box<dyn Element>> = vec![Box::new(Fire), Box::new(Water)];
    for e in &elements {
        // Which `name` runs is decided at runtime (dynamic dispatch).
        println!("{}", e.name());
    }
}
"#,
    ),
    (
        "lifetimes",
        r#"// `'a` ties the result to both inputs: the returned borrow can't
// outlive either of them.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    println!("{}", longest("aa", "b"));
}
"#,
    ),
    (
        "assoc_type",
        r#"trait Producer {
    // Each implementor names its own `Output` type.
    type Output;
    fn make(&self) -> Self::Output;
}

struct Coiner;

impl Producer for Coiner {
    type Output = i32;
    fn make(&self) -> Self::Output {
        7
    }
}

fn main() {
    let c = Coiner;
    println!("{}", c.make());
}
"#,
    ),
    // ── Act 4 boss: Vexis's trial, The Chimera of Vexis ──────────────
    (
        "chimera_heads",
        r#"trait Enemy {
    fn name(&self) -> &'static str;
    fn breath(&self) -> u32;
}

struct Fire;
struct Frost;
struct Storm;

impl Enemy for Fire {
    fn name(&self) -> &'static str {
        "fire"
    }
    fn breath(&self) -> u32 {
        7
    }
}

// Each head opts in with its own `impl Enemy for ...` block.
impl Enemy for Frost {
    fn name(&self) -> &'static str {
        "frost"
    }
    fn breath(&self) -> u32 {
        5
    }
}

impl Enemy for Storm {
    fn name(&self) -> &'static str {
        "storm"
    }
    fn breath(&self) -> u32 {
        9
    }
}

fn main() {
    println!("{} breathes {}", Fire.name(), Fire.breath());
    println!("{} breathes {}", Frost.name(), Frost.breath());
    println!("{} breathes {}", Storm.name(), Storm.breath());
}
"#,
    ),
    (
        "chimera_bound",
        r#"trait Enemy {
    fn name(&self) -> &'static str;
    fn breath(&self) -> u32;
}

struct Fire;
struct Frost;
struct Storm;

impl Enemy for Fire {
    fn name(&self) -> &'static str { "fire" }
    fn breath(&self) -> u32 { 7 }
}
impl Enemy for Frost {
    fn name(&self) -> &'static str { "frost" }
    fn breath(&self) -> u32 { 5 }
}
impl Enemy for Storm {
    fn name(&self) -> &'static str { "storm" }
    fn breath(&self) -> u32 { 9 }
}

// `T: Enemy` promises that every T has the trait's methods.
fn volley<T: Enemy>(head: &T, rounds: u32) -> String {
    format!("{} x{rounds} = {}", head.name(), head.breath() * rounds)
}

fn main() {
    println!("{}", volley(&Fire, 3));
    println!("{}", volley(&Frost, 3));
    println!("{}", volley(&Storm, 3));
}
"#,
    ),
    (
        "chimera_plasma",
        r#"trait Enemy {
    fn name(&self) -> &'static str;
    fn breath(&self) -> u32;
}

struct Fire;
struct Frost;
struct Storm;
struct Plasma;

impl Enemy for Fire {
    fn name(&self) -> &'static str { "fire" }
    fn breath(&self) -> u32 { 7 }
}
impl Enemy for Frost {
    fn name(&self) -> &'static str { "frost" }
    fn breath(&self) -> u32 { 5 }
}
impl Enemy for Storm {
    fn name(&self) -> &'static str { "storm" }
    fn breath(&self) -> u32 { 9 }
}
impl Enemy for Plasma {
    fn name(&self) -> &'static str { "plasma" }
    fn breath(&self) -> u32 { 12 }
}

// Every call goes through the trait object, so any head that
// implements `Enemy` works here, even one added after this was written.
fn battle(heads: &[Box<dyn Enemy>]) -> u32 {
    let mut total = 0;
    for head in heads {
        println!("{} breathes {}", head.name(), head.breath());
        total += head.breath();
    }
    total
}

fn main() {
    let heads: Vec<Box<dyn Enemy>> = vec![
        Box::new(Fire),
        Box::new(Frost),
        Box::new(Storm),
        Box::new(Plasma),
    ];
    println!("total: {}", battle(&heads));
}
"#,
    ),
    // ── Act 5: The Tavern of Tribulations ────────────────────────────
    (
        "result_match",
        r#"fn describe(r: Result<i32, String>) -> i32 {
    // Both outcomes must be handled, and both arms must produce an i32.
    match r {
        Ok(v) => v,
        Err(e) => {
            eprintln!("spilled: {e}");
            -1
        }
    }
}

fn main() {
    println!("{}", describe(Ok(5)));
    println!("{}", describe(Err(String::from("dropped the mug"))));
}
"#,
    ),
    (
        "custom_error",
        r#"// One variant per way the brew can go wrong.
#[derive(Debug)]
enum BrewError {
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
    println!("{:?}", check(50));
    println!("{:?}", check(150));
    println!("{:?}", check(-5));
}
"#,
    ),
    (
        "from_error",
        r#"struct ParseFail;
enum AppError {
    Parse,
}

// With this impl, `.into()` and `?` can turn a ParseFail into an AppError.
impl From<ParseFail> for AppError {
    fn from(_: ParseFail) -> Self {
        AppError::Parse
    }
}

fn first_rune(s: &str) -> Result<char, ParseFail> {
    s.chars().next().ok_or(ParseFail)
}

fn read_rune(s: &str) -> Result<char, AppError> {
    // `?` converts the ParseFail into an AppError using the `From` impl above.
    let rune = first_rune(s)?;
    Ok(rune)
}

fn main() {
    let _converted: AppError = ParseFail.into();
    match read_rune("") {
        Ok(rune) => println!("read {rune}"),
        Err(AppError::Parse) => println!("nothing to read"),
    }
}
"#,
    ),
    (
        "option_map",
        r#"fn add_one(o: Option<i32>) -> Option<i32> {
    // `map` changes the value inside `Some`. `None` passes straight through.
    o.map(|x| x + 1)
}

fn main() {
    println!("{:?}", add_one(Some(3)));
    println!("{:?}", add_one(None));
}
"#,
    ),
    (
        "and_then",
        r#"fn half(n: i32) -> Option<i32> {
    if n % 2 == 0 { Some(n / 2) } else { None }
}

fn main() {
    // Each step runs only if the previous one returned `Some`.
    let two = Some(8).and_then(half).and_then(half); // 8 -> 4 -> 2
    println!("{two:?}");
    // 8 -> 4 -> 2 -> 1, then 1 is odd: the chain stops with `None`.
    let gone = Some(8)
        .and_then(half)
        .and_then(half)
        .and_then(half)
        .and_then(half);
    println!("{gone:?}");
}
"#,
    ),
    (
        "unwrap_or_else",
        r#"fn value(o: Option<i32>) -> i32 {
    // The closure runs only on the `None` path, so the fallback costs
    // nothing when there is a value.
    o.unwrap_or_else(|| {
        println!("the cask is dry; broaching a fresh one");
        0
    })
}

fn main() {
    println!("{}", value(Some(3)));
    println!("{}", value(None));
}
"#,
    ),
    // ── Act 5 boss: the Barkeep's trial, The Tavernkeeper's Wager ────
    (
        "wager_stake",
        r#"use std::num::ParseIntError;

fn read_stake(text: &str) -> Result<u32, ParseIntError> {
    text.trim().parse::<u32>()
}

// The return type says it can fail, so `?` may hand an error back.
fn double_stake(text: &str) -> Result<u32, ParseIntError> {
    let stake = read_stake(text)?;
    Ok(stake * 2)
}

fn main() {
    println!("{:?}", double_stake("20"));
    println!("{:?}", double_stake("twenty"));
}
"#,
    ),
    (
        "wager_from",
        r#"use std::num::ParseIntError;

#[derive(Debug)]
struct Short;

fn pay(purse: u32, stake: u32) -> Result<u32, Short> {
    if stake > purse { Err(Short) } else { Ok(purse - stake) }
}

#[derive(Debug)]
enum WagerError {
    BadStake,
    TooPoor,
}

// One `From` per error type that `?` must turn into a WagerError.
impl From<ParseIntError> for WagerError {
    fn from(_: ParseIntError) -> Self {
        WagerError::BadStake
    }
}

impl From<Short> for WagerError {
    fn from(_: Short) -> Self {
        WagerError::TooPoor
    }
}

fn place_bet(text: &str, purse: u32) -> Result<u32, WagerError> {
    let stake = text.trim().parse::<u32>()?;
    let left = pay(purse, stake)?;
    Ok(left)
}

fn main() {
    println!("{:?}", place_bet("15", 50));
    println!("{:?}", place_bet("lots", 50));
    println!("{:?}", place_bet("80", 50));
}
"#,
    ),
    (
        "wager_quest",
        r#"use std::num::ParseIntError;

#[derive(Debug)]
struct WrongKey;

#[derive(Debug)]
struct Short;

#[derive(Debug)]
struct Reward {
    gold: u32,
}

#[derive(Debug)]
enum QuestError {
    BadMap,
    Locked,
    Broke,
}

impl From<ParseIntError> for QuestError {
    fn from(_: ParseIntError) -> Self {
        QuestError::BadMap
    }
}
impl From<WrongKey> for QuestError {
    fn from(_: WrongKey) -> Self {
        QuestError::Locked
    }
}
impl From<Short> for QuestError {
    fn from(_: Short) -> Self {
        QuestError::Broke
    }
}

// Step 1: the map says how many leagues to walk.
fn read_map(map: &str) -> Result<u32, ParseIntError> {
    map.trim().parse::<u32>()
}

// Step 2: only the brass key opens the vault. It holds 30 gold.
fn open_vault(key: &str) -> Result<u32, WrongKey> {
    if key == "brass" { Ok(30) } else { Err(WrongKey) }
}

// Step 3: the guide charges 2 gold a league.
fn pay_guide(gold: u32, leagues: u32) -> Result<u32, Short> {
    let fee = leagues * 2;
    if fee > gold { Err(Short) } else { Ok(gold - fee) }
}

// Each `?` either unwraps its step's value or returns the step's error,
// converted to a QuestError by its `From` impl. The first failure wins.
fn quest(map: &str, key: &str) -> Result<Reward, QuestError> {
    let leagues = read_map(map)?;
    let gold = open_vault(key)?;
    let left = pay_guide(gold, leagues)?;
    Ok(Reward { gold: left })
}

fn main() {
    println!("{:?}", quest("4", "brass").map(|r| r.gold));
    println!("{:?}", quest("far", "brass").map(|r| r.gold));
    println!("{:?}", quest("4", "bone").map(|r| r.gold));
    println!("{:?}", quest("20", "brass").map(|r| r.gold));
}
"#,
    ),
    // ── Act 6: The Iterator Forge ────────────────────────────────────
    (
        "hashmap_basic",
        r#"use std::collections::HashMap;

fn main() {
    let mut stores = HashMap::new();
    stores.insert("vault", 7);
    // `get` returns an Option: the key might not be there.
    println!("{:?}", stores.get("vault"));
    println!("{:?}", stores.get("cellar"));
}
"#,
    ),
    (
        "iter_filter",
        r#"fn main() {
    let v = vec![1, 2, 3, 4];
    // `filter` keeps items where the closure returns true.
    // `iter()` yields `&i32` and `filter` borrows each one again, hence `**x`.
    let evens: Vec<&i32> = v.iter().filter(|x| **x % 2 == 0).collect();
    println!("{evens:?}");
}
"#,
    ),
    (
        "iter_fold",
        r#"fn main() {
    let v = vec![1, 2, 3];
    // Start the accumulator at 0, then combine it with each element in turn.
    let total = v.iter().fold(0, |acc, x| acc + x);
    println!("{total}");
}
"#,
    ),
    (
        "iter_enumerate",
        r#"fn main() {
    let v = vec![10, 20, 30];
    // `enumerate` pairs each item with its index, starting at 0.
    for (i, x) in v.iter().enumerate() {
        println!("{i}: {x}");
    }
}
"#,
    ),
    (
        "iter_zip",
        r#"fn main() {
    let a = vec![1, 2];
    let b = vec![3, 4];
    // `zip` walks both in step and stops at the end of the shorter one.
    for (x, y) in a.iter().zip(b.iter()) {
        println!("{x},{y}");
    }
}
"#,
    ),
    (
        "closure_move",
        r#"fn main() {
    let name = String::from("Garin");
    // `move` makes the closure own `name`. After this line, `name` can't be
    // used here any more: it travels with the closure.
    let greet = move || println!("hail, {name}");
    greet();
}
"#,
    ),
    // ── Act 6 boss: the Keymaster's trial, The Forge-Golem ───────────
    (
        "golem_parts",
        r#"struct Part {
    name: &'static str,
    armoured: bool,
    weakness: u32,
}

fn golem() -> Vec<Part> {
    vec![
        Part { name: "arm", armoured: false, weakness: 3 },
        Part { name: "chest", armoured: true, weakness: 8 },
        Part { name: "knee", armoured: false, weakness: 4 },
        Part { name: "head", armoured: true, weakness: 5 },
        Part { name: "eye", armoured: false, weakness: 6 },
    ]
}

// Filter first, while each item is still a whole Part; label after.
fn weak_points(parts: &[Part]) -> Vec<String> {
    parts
        .iter()
        .filter(|p| !p.armoured)
        .map(|p| format!("{} ({})", p.name, p.weakness))
        .collect()
}

fn main() {
    let parts = golem();
    println!("weak points: {:?}", weak_points(&parts));
}
"#,
    ),
    (
        "golem_sum",
        r#"struct Part {
    name: &'static str,
    armoured: bool,
    weakness: u32,
}

fn golem() -> Vec<Part> {
    vec![
        Part { name: "arm", armoured: false, weakness: 3 },
        Part { name: "chest", armoured: true, weakness: 8 },
        Part { name: "knee", armoured: false, weakness: 4 },
        Part { name: "head", armoured: true, weakness: 5 },
        Part { name: "eye", armoured: false, weakness: 6 },
    ]
}

// The core takes every weak point's weakness, added up and doubled.
fn core_damage(parts: &[Part]) -> u32 {
    // `: u32` tells `sum` which kind of number to make.
    let total: u32 = parts
        .iter()
        .filter(|p| !p.armoured)
        .map(|p| p.weakness)
        .sum();
    total * 2
}

fn main() {
    let parts = golem();
    let names: Vec<&str> = parts.iter().map(|p| p.name).collect();
    println!("golem: {}", names.join(" "));
    println!("core damage: {}", core_damage(&parts));
}
"#,
    ),
    (
        "golem_core",
        r#"struct Part {
    name: &'static str,
    armoured: bool,
    weakness: u32,
}

fn golem() -> Vec<Part> {
    vec![
        Part { name: "arm", armoured: false, weakness: 3 },
        Part { name: "chest", armoured: true, weakness: 8 },
        Part { name: "knee", armoured: false, weakness: 4 },
        Part { name: "head", armoured: true, weakness: 5 },
        Part { name: "eye", armoured: false, weakness: 6 },
    ]
}

// One chain: armour skipped, each weak point mapped to its hit, the
// hits folded into one total. No loop, no mutable total.
fn total_damage(parts: &[Part], blow: u32) -> u32 {
    parts
        .iter()
        .filter(|p| !p.armoured)
        .map(|p| blow * p.weakness)
        .fold(0, |total, hit| total + hit)
}

fn main() {
    let parts = golem();
    let names: Vec<&str> = parts.iter().map(|p| p.name).collect();
    println!("golem: {}", names.join(" "));
    println!("blow 1: {}", total_damage(&parts, 1));
    println!("blow 3: {}", total_damage(&parts, 3));
}
"#,
    ),
    // ── Act 7: The Concurrent Coast ──────────────────────────────────
    (
        "thread_spawn",
        r#"use std::thread;

fn main() {
    // The closure runs on a new OS thread. `spawn` returns a handle at once.
    let handle = thread::spawn(|| 21 * 2);
    // `join` waits for the thread. It is an `Err` only if that thread panicked.
    let answer = handle.join().expect("the worker thread panicked");
    println!("{answer}");
}
"#,
    ),
    (
        "arc_mutex",
        r#"use std::sync::{Arc, Mutex};

fn main() {
    // `Arc` = shared ownership across threads; `Mutex` = one holder at a time.
    let shared = Arc::new(Mutex::new(0));
    let other_owner = Arc::clone(&shared); // what you would move into a thread
    {
        // `lock` is an `Err` only if a thread panicked while holding the lock.
        let mut guard = other_owner.lock().expect("lock poisoned");
        *guard += 5;
    } // the guard is dropped here, which releases the lock
    // Both handles reach the same value, so the first one sees the change.
    println!("{}", shared.lock().expect("lock poisoned"));
}
"#,
    ),
    (
        "mpsc_channel",
        r#"use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    // `send` fails only if the receiver is gone; `recv` only if every sender is.
    tx.send(7).expect("receiver dropped");
    tx.send(11).expect("receiver dropped");
    // Values come out in the order they went in.
    println!("{}", rx.recv().expect("all senders dropped"));
    println!("{}", rx.recv().expect("all senders dropped"));
}
"#,
    ),
    (
        "atomic",
        r#"use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    let counter = AtomicUsize::new(10);
    // One indivisible read-add-write. No lock, no guard.
    // `fetch_add` returns the value from BEFORE the add.
    let before = counter.fetch_add(1, Ordering::SeqCst);
    println!("{before}");
    println!("{}", counter.load(Ordering::SeqCst));
}
"#,
    ),
    (
        "thread_scope",
        r#"use std::thread;

fn main() {
    let data = vec![1, 2, 3];
    // Every thread spawned on `s` is joined before `scope` returns, so the
    // threads may borrow `data`. No `move`, no `Arc`.
    thread::scope(|s| {
        s.spawn(|| {
            println!("{:?}", &data);
        });
    });
    println!("still ours: {data:?}");
}
"#,
    ),
    (
        "async_fn",
        r#"async fn double(x: i32) -> i32 {
    x * 2
}

async fn run() -> i32 {
    // `.await` is only allowed inside async code.
    double(21).await
}

fn main() {
    // Calling an async fn only builds a future. Nothing runs until an
    // executor (such as tokio) polls it, and this sandbox has none.
    let _future = run();
}
"#,
    ),
    // ── Act 7 boss: the Dockmaster's trial, The Tide Hydra ───────────
    (
        "hydra_senders",
        r#"use std::sync::mpsc::{self, Sender};

// A head fights until it falls, then reports on the sender it holds.
fn fight(head: &'static str, hp: u32, tx: Sender<String>) {
    let blows = hp.div_ceil(4);
    let report = format!("{head} down after {blows} blows");
    tx.send(report).expect("receiver dropped");
}

fn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    for &(head, hp) in heads {
        // Each head gets its own sender into the same channel.
        fight(head, hp, tx.clone());
    }
    // No sender left: the receiver knows no more reports are coming.
    drop(tx);
    rx.iter().collect()
}

fn main() {
    let reports = hunt(&[("fang", 9), ("venom", 6), ("brine", 13)]);
    for report in &reports {
        println!("{report}");
    }
    println!("{} heads down", reports.len());
}
"#,
    ),
    (
        "hydra_threads",
        r#"use std::sync::mpsc::{self, Sender};
use std::thread;

// A head fights until it falls, then reports on the sender it holds.
fn fight(head: &'static str, hp: u32, tx: Sender<String>) {
    let blows = hp.div_ceil(4);
    let report = format!("{head} down after {blows} blows");
    tx.send(report).expect("receiver dropped");
}

fn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    for &(head, hp) in heads {
        let tx = tx.clone();
        // `move`: the thread owns its head, its hp and its sender.
        thread::spawn(move || fight(head, hp, tx));
    }
    drop(tx);
    // Threads finish in no promised order: sort what comes back.
    let mut reports: Vec<String> = rx.iter().collect();
    reports.sort();
    reports
}

fn main() {
    let reports = hunt(&[("fang", 9), ("venom", 6), ("brine", 13)]);
    for report in &reports {
        println!("{report}");
    }
    println!("{} heads down", reports.len());
}
"#,
    ),
    (
        "hydra_body",
        r#"use std::sync::mpsc::{self, Receiver};
use std::thread;

enum Report {
    Hit,
    Down,
}

// Count the hits, and return only once every head has reported down.
fn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 {
    let mut hits = 0;
    let mut down = 0;
    while down < heads {
        match rx.recv() {
            Ok(Report::Hit) => hits += 1,
            Ok(Report::Down) => down += 1,
            // Every sender is gone: no more reports can come.
            Err(_) => break,
        }
    }
    hits
}

fn main() {
    // The blows each head takes: fang, venom, brine.
    let blows = [3, 2, 4];
    let (tx, rx) = mpsc::channel();
    for n in blows {
        let tx = tx.clone();
        thread::spawn(move || {
            for _ in 0..n {
                tx.send(Report::Hit).expect("receiver dropped");
            }
            tx.send(Report::Down).expect("receiver dropped");
        });
    }
    drop(tx);
    let hits = await_heads(&rx, blows.len());
    println!("{} heads down after {hits} hits", blows.len());
    println!("strike the body");
}
"#,
    ),
    // ── Act 8: The Vault of Pointers ─────────────────────────────────
    (
        "box_basic",
        r#"// `Box<Node>` is a pointer of fixed size, so the recursive type has a known size.
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

fn main() {
    let list = Node {
        value: 1,
        next: Some(Box::new(Node { value: 2, next: None })),
    };
    println!("{}", list.value);
    if let Some(next) = &list.next {
        println!("{}", next.value);
    }
}
"#,
    ),
    (
        "rc_basic",
        r#"use std::rc::Rc;

fn main() {
    let original = Rc::new(String::from("hoard"));
    println!("{}", Rc::strong_count(&original));
    // `Rc::clone` bumps the count. Both handles point at the same String.
    let shared = Rc::clone(&original);
    println!("{}", Rc::strong_count(&original));
    // Dropping a handle lowers the count; the String lives while any remain.
    drop(shared);
    println!("{}", Rc::strong_count(&original));
}
"#,
    ),
    (
        "refcell",
        r#"use std::cell::RefCell;

fn main() {
    let guarded = RefCell::new(10);
    let shared_ref = &guarded; // only a shared `&`...
    println!("{}", shared_ref.borrow());
    // ...yet we can mutate. The borrow rules are checked at runtime instead.
    *shared_ref.borrow_mut() += 5;
    println!("{}", shared_ref.borrow());
}
"#,
    ),
    (
        "cell",
        r#"use std::cell::Cell;

fn main() {
    let slot = Cell::new(1);
    let shared_ref = &slot;
    println!("{}", shared_ref.get());
    // `set` swaps in a whole new value through a shared `&`. No borrow guard.
    shared_ref.set(5);
    println!("{}", shared_ref.get());
}
"#,
    ),
    (
        "rc_refcell",
        r#"use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // `Rc` shares ownership; `RefCell` allows mutation behind that sharing.
    let shared = Rc::new(RefCell::new(0));
    let clone = Rc::clone(&shared);
    *clone.borrow_mut() += 10;
    *shared.borrow_mut() += 5;
    // Both handles open the same box, so it holds both changes.
    println!("{}", shared.borrow());
}
"#,
    ),
    (
        "weak_ref",
        r#"use std::rc::{Rc, Weak};

fn main() {
    let strong = Rc::new(5);
    // A `Weak` handle does not keep the value alive.
    let weak: Weak<i32> = Rc::downgrade(&strong);
    println!("{:?}", weak.upgrade());
    drop(strong); // the last strong owner is gone, so the value is freed
    println!("{:?}", weak.upgrade());
}
"#,
    ),
    // ── Act 8 boss: the Vaultwright's trial, The Memory Warden ───────
    (
        "warden_box",
        r#"// A ledger is either closed, or one entry and the rest of the ledger.
#[derive(Debug)]
enum Ledger {
    // The Box is a pointer of fixed size; the rest lives on the heap.
    Entry(u32, Box<Ledger>),
    Closed,
}

fn entry(amount: u32, rest: Ledger) -> Ledger {
    Ledger::Entry(amount, Box::new(rest))
}

fn total(ledger: &Ledger) -> u32 {
    match ledger {
        Ledger::Entry(amount, rest) => amount + total(rest),
        Ledger::Closed => 0,
    }
}

fn main() {
    let ledger = entry(5, entry(7, entry(3, Ledger::Closed)));
    println!("{ledger:?}");
    println!("owed: {}", total(&ledger));
}
"#,
    ),
    (
        "warden_shared",
        r#"use std::cell::RefCell;
use std::rc::Rc;

// Every debtor holds a handle to the one vault.
struct Debtor {
    name: &'static str,
    vault: Rc<RefCell<u32>>,
}

// The RefCell lends the vault out for writing, one borrow at a time.
fn pay(debtor: &Debtor, coins: u32) {
    *debtor.vault.borrow_mut() += coins;
}

fn main() {
    let vault = Rc::new(RefCell::new(0));
    let miller = Debtor { name: "miller", vault: Rc::clone(&vault) };
    let smith = Debtor { name: "smith", vault: Rc::clone(&vault) };
    pay(&miller, 5);
    pay(&smith, 7);
    println!("{} and {} paid", miller.name, smith.name);
    println!("vault holds {}", vault.borrow());
    println!("handles: {}", Rc::strong_count(&vault));
}
"#,
    ),
    (
        "warden_weak",
        r#"use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Debtor {
    name: &'static str,
    // Weak: owing someone does not keep them alive.
    owes: RefCell<Weak<Debtor>>,
}

fn debtor(name: &'static str) -> Rc<Debtor> {
    let owes = RefCell::new(Weak::new());
    Rc::new(Debtor { name, owes })
}

// Follow the debts from `start` until they come back round, or end.
fn circle(start: &Rc<Debtor>) -> Vec<&'static str> {
    let mut names = vec![start.name];
    let mut at = Rc::clone(start);
    loop {
        let next = at.owes.borrow().upgrade();
        match next {
            Some(d) if !Rc::ptr_eq(&d, start) => {
                names.push(d.name);
                at = d;
            }
            _ => return names,
        }
    }
}

fn main() {
    let warden = debtor("warden");
    let you = debtor("you");
    let teller = debtor("teller");
    // The Warden owes you, you owe the teller, the teller owes him.
    *warden.owes.borrow_mut() = Rc::downgrade(&you);
    *you.owes.borrow_mut() = Rc::downgrade(&teller);
    *teller.owes.borrow_mut() = Rc::downgrade(&warden);
    println!("circle: {:?}", circle(&warden));
    let strong = Rc::strong_count(&warden);
    let weak = Rc::weak_count(&warden);
    println!("warden: strong {strong}, weak {weak}");
    let probe = Rc::downgrade(&warden);
    drop((warden, you, teller));
    let left = probe.upgrade().map(|d| d.name);
    println!("after the ledger closes: {left:?}");
}
"#,
    ),
    // ── Act 9: The Forbidden Library ────────────────────────────────
    // These programs contain `unsafe`: they are the player's code,
    // shown as text and compiled only by the sandboxed server (hard
    // rule 1 is about game code; nothing here runs in the game).
    (
        "raw_pointer",
        r#"fn main() {
    let mut gold: i32 = 10;

    // Making a raw pointer is safe. Only dereferencing it needs `unsafe`.
    let write_ptr: *mut i32 = &mut gold;
    unsafe {
        // SAFETY: write_ptr came from `&mut gold` on the line above, so it
        // is non-null, aligned and points at a live i32. Nothing else
        // touches `gold` until this write is done.
        *write_ptr += 5;
    }

    let read_ptr: *const i32 = &gold;
    // SAFETY: read_ptr came from `&gold` just above and `gold` is alive.
    let seen = unsafe { *read_ptr };

    println!("pointer is null: {}", read_ptr.is_null());
    println!("gold through the pointer: {seen}");
    println!("gold by name: {gold}");
}
"#,
    ),
    (
        "extern_c",
        r#"/// Laid out by C's rules: fields in declared order, C padding.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rune {
    pub glyph: u8,
    pub power: u32,
    pub tier: u8,
}

/// Exported under its plain, unmangled name with the C calling
/// convention. A C program linked with this one could call it.
/// `no_mangle` is an unsafe attribute: two items exporting the same
/// symbol name would be undefined behaviour, and we promise there is one.
#[unsafe(no_mangle)]
pub extern "C" fn crown_seal(rune: Rune) -> u32 {
    rune.power * 2 + rune.tier as u32 + rune.glyph as u32
}

mod foreign {
    // A foreign declaration: "somewhere in the linked program there is a
    // C-ABI function with this name and signature." The compiler cannot
    // check that claim. Writing `unsafe extern` says we take
    // responsibility for it, and each call is unsafe too.
    unsafe extern "C" {
        pub fn crown_seal(rune: super::Rune) -> u32;
    }
}

fn main() {
    let rune = Rune { glyph: 3, power: 20, tier: 1 };
    println!(
        "Rune: {} bytes, align {}",
        std::mem::size_of::<Rune>(),
        std::mem::align_of::<Rune>()
    );

    // Called by its Rust path it is an ordinary (safe) function call.
    println!("direct call: {}", crown_seal(rune));

    // SAFETY: the declaration in `foreign` matches the definition above
    // exactly (same name, same repr(C) argument, same return type).
    let sealed = unsafe { foreign::crown_seal(rune) };
    println!("through the extern block: {sealed}");

    // A C-ABI function pointer, the shape a C library takes for callbacks.
    let callback: extern "C" fn(Rune) -> u32 = crown_seal;
    println!("through a fn pointer: {}", callback(rune));
}
"#,
    ),
    (
        "macro_rules",
        r#"// `chant!` takes one or more expressions separated by commas, prints each
// verse, and evaluates to their total.
macro_rules! chant {
    ($($verse:expr),+ $(,)?) => {{
        let mut total = 0;
        $(
            // Bind once: a macro pastes tokens, so writing `$verse` twice
            // would evaluate the expression twice.
            let verse = $verse;
            println!("  verse: {verse}");
            total += verse;
        )+
        total
    }};
}

fn main() {
    let first = chant!(1);
    println!("first chant: {first}");
    let second = chant!(2, 3, 4);
    println!("second chant: {second}");
    let third = chant!(10, 20,);
    println!("third chant: {third}");
}
"#,
    ),
    (
        "drop_trait",
        r#"use std::mem;

struct Soul {
    name: &'static str,
}

impl Drop for Soul {
    // Runs automatically when a Soul goes out of scope. You never call it
    // yourself; to drop early, hand the value to `drop(value)`.
    fn drop(&mut self) {
        println!("{} is released", self.name);
    }
}

fn main() {
    let _first = Soul { name: "first" };
    let _second = Soul { name: "second" };
    let early = Soul { name: "early" };
    let bound = Soul { name: "bound" };

    drop(early); // std::mem::drop takes ownership, so `early` drops here
    println!("the ritual begins");

    let _ = Soul { name: "unnamed" }; // `_` does not bind: dropped at once

    {
        let _inner = Soul { name: "inner" };
        println!("inner scope ends");
    } // `_inner` drops here

    mem::forget(bound); // safe, but its destructor will never run
    println!("main ends");
} // remaining locals drop in reverse order: `_second`, then `_first`
"#,
    ),
    (
        "send_sync",
        r#"/// A tome that owns one heap page through a raw pointer, the way `Box`
/// does internally. Raw pointers are neither `Send` nor `Sync`, so the
/// compiler does not derive either for `Tome`. We must decide.
struct Tome {
    page: *mut u32,
}

impl Tome {
    fn new(value: u32) -> Self {
        Tome {
            page: Box::into_raw(Box::new(value)),
        }
    }

    fn read(&self) -> u32 {
        // SAFETY: `page` came from Box::into_raw in `new` and is freed only
        // in Drop, so it is valid for as long as `self` exists.
        unsafe { *self.page }
    }
}

impl Drop for Tome {
    fn drop(&mut self) {
        // SAFETY: we own the allocation and this is the only place it is
        // freed, exactly once.
        unsafe { drop(Box::from_raw(self.page)) };
        println!("the page is returned to the shelf");
    }
}

// SAFETY (Send): a Tome is the only owner of its page, like Box<u32>.
// Moving the Tome to another thread moves that sole ownership with it, and
// u32 itself is Send.
unsafe impl Send for Tome {}

// SAFETY (Sync): through `&Tome` the only operation is `read`, which never
// writes. Many threads reading the same u32 at once is not a data race.
unsafe impl Sync for Tome {}

/// Compiles only for types that may be moved to another thread.
fn courier_accepts<T: Send>(item: T) -> T {
    item
}

/// Compiles only for types that may be shared between threads by `&`.
fn readers_accept<T: Sync>(item: &T) -> &T {
    item
}

fn main() {
    let tome = Tome::new(7);
    let carried = courier_accepts(tome);
    println!("the courier accepts the tome");
    let shared = readers_accept(&carried);
    println!("page reads {}", shared.read());
}
"#,
    ),
    (
        "maybe_uninit",
        r#"use std::mem::MaybeUninit;

const PAGES: usize = 4;

fn main() {
    // Room for four u32s, deliberately not initialised yet.
    let mut quire = MaybeUninit::<[u32; PAGES]>::uninit();
    let first: *mut u32 = quire.as_mut_ptr().cast::<u32>();

    for i in 0..PAGES {
        // SAFETY: i < PAGES, so first.add(i) stays inside the array.
        // `write` stores the value without reading or dropping whatever
        // (uninitialised) bytes were there before.
        unsafe { first.add(i).write((i as u32 + 1) * 10) };
    }

    // SAFETY: the loop above initialised every one of the PAGES elements.
    let quire: [u32; PAGES] = unsafe { quire.assume_init() };

    println!("bound pages: {quire:?}");
    println!("total ink: {}", quire.iter().sum::<u32>());
}
"#,
    ),
    // ── Act 9 boss: the Librarian's trial ───────────────────────────
    (
        "librarian_index",
        r#"/// Returns the volume `index` places past `shelf`.
///
/// # Safety
///
/// `shelf` must point into a live array of initialised `u32`s, and
/// `shelf.add(index)` must still be inside that same array.
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    // SAFETY: the caller upholds the contract above.
    unsafe { *shelf.add(index) }
}

/// The safe door. No argument a caller can pass makes this read out
/// of bounds, so it does not need to be `unsafe` itself.
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    if index < shelf.len() {
        // SAFETY: index < shelf.len(), and the slice is borrowed (so
        // alive) for the whole call.
        Some(unsafe { volume_at(shelf.as_ptr(), index) })
    } else {
        None
    }
}

fn main() {
    let shelf = [11, 22, 33, 44];
    for index in [0, 3, 4, 99] {
        match fetch(&shelf, index) {
            Some(volume) => println!("index {index}: volume {volume}"),
            None => println!("index {index}: no such volume"),
        }
    }
}
"#,
    ),
    (
        "librarian_demand",
        r#"fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// incant!(a, b, c, d) must expand to the Librarian's exact demand:
//     seal(seal(seal(page(a), page(b)), page(c)), page(d))
// The internal `@acc` rules carry the expression built so far. A
// literal `@` can never start an expression, so the entry rule never
// swallows an `@acc` call, and a caller's verses never match `@acc`.
macro_rules! incant {
    (@acc $acc:expr) => { $acc };
    (@acc $acc:expr, $next:expr $(, $rest:expr)*) => {
        incant!(@acc seal($acc, page($next)) $(, $rest)*)
    };
    ($first:expr $(, $rest:expr)*) => {
        incant!(@acc page($first) $(, $rest)*)
    };
}

fn main() {
    let demand = seal(seal(seal(page(1), page(2)), page(3)), page(4));
    let answer = incant!(1, 2, 3, 4);
    println!("the demand: {demand}");
    println!("incant!(1, 2, 3, 4): {answer}");
    println!("the incantation matches: {}", demand == answer);
    println!("incant!(7): {}", incant!(7));
    println!("incant!(2, 9): {}", incant!(2, 9));
}
"#,
    ),
    (
        "librarian_seal",
        r#"/// Owns a run of `len` u32s through a raw pointer, the way `Vec` does
/// underneath. The raw pointer itself owns nothing: Drop must free it.
struct Codex {
    ptr: *mut u32,
    len: usize,
}

impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        let boxed: Box<[u32]> = pages.into_boxed_slice();
        let len = boxed.len();
        // Box::into_raw gives up ownership: nothing frees this until
        // we rebuild the Box in Drop.
        let ptr = Box::into_raw(boxed).cast::<u32>();
        Codex { ptr, len }
    }

    fn get(&self, index: usize) -> Option<u32> {
        if index < self.len {
            // SAFETY: index < len, and ptr points at len initialised
            // u32s that stay alive until Drop.
            Some(unsafe { *self.ptr.add(index) })
        } else {
            None
        }
    }
}

impl Drop for Codex {
    fn drop(&mut self) {
        let slice =
            std::ptr::slice_from_raw_parts_mut(self.ptr, self.len);
        // SAFETY: this is exactly the Box<[u32]> that from_pages turned
        // into a raw pointer (same address, same length), and Drop runs
        // once.
        drop(unsafe { Box::from_raw(slice) });
        println!("the codex is sealed ({} pages returned)", self.len);
    }
}

fn main() {
    let codex = Codex::from_pages(vec![5, 8, 13]);
    println!("page 0: {:?}", codex.get(0));
    println!("page 2: {:?}", codex.get(2));
    println!("page 3: {:?}", codex.get(3));
    println!("the Librarian closes the book");
}
"#,
    ),
    // ── Act 10: The Throne (`unsafe` and `#[global_allocator]` in the
    // two allocator programs are player code, text in a string) ──
    (
        "prealloc",
        r#"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method passes its arguments unchanged to `System`,
// which keeps the GlobalAlloc contract. Adding to an atomic does not
// allocate.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn main() {
    let before = ALLOCATIONS.load(Ordering::Relaxed);

    // Ask for room for every coin up front: one allocation, and `push`
    // never reallocates while the length is below the capacity.
    let mut ledger = Vec::with_capacity(1000);
    for coin in 0..1000u32 {
        ledger.push(coin);
    }

    let after = ALLOCATIONS.load(Ordering::Relaxed);
    // Measure first, print after: printing is not part of the ledger.
    println!("coins: {}", ledger.len());
    println!("allocations: {}", after - before);
}
"#,
    ),
    (
        "cow_borrow",
        r#"use std::borrow::Cow;

/// Shelf names use underscores instead of spaces. Most titles have no
/// spaces, so borrow those as they are and copy only when something
/// must change.
fn shelf_name(title: &str) -> Cow<'_, str> {
    if title.contains(' ') {
        Cow::Owned(title.replace(' ', "_"))
    } else {
        Cow::Borrowed(title)
    }
}

fn main() {
    for title in ["Grimoire", "Book of Crowns", "Ledger"] {
        let name = shelf_name(title);
        let how = match &name {
            Cow::Borrowed(_) => "borrowed",
            Cow::Owned(_) => "copied",
        };
        println!("{name} ({how})");
    }
}
"#,
    ),
    (
        "const_eval",
        r#"/// A `const fn` can run inside the compiler when it is used to compute
/// a constant. `for` loops are not allowed in const fns (they call
/// `Iterator::next`, which is not const), so this uses `while`.
const fn star_table() -> [u32; 8] {
    let mut table = [0u32; 8];
    let mut i = 0;
    while i < 8 {
        table[i] = (i as u32) * (i as u32) + 1;
        i += 1;
    }
    table
}

/// Computed once, during compilation. The finished array is baked into
/// the program; no work happens at run time to build it.
const STARS: [u32; 8] = star_table();

// Checked during compilation: if this were false, the program would not
// build at all.
const _: () = assert!(STARS[7] == 50, "the last star is misplaced");

fn main() {
    println!("star chart: {STARS:?}");
    println!("brightest: {}", STARS[7]);
}
"#,
    ),
    (
        "chunks_exact",
        r#"/// Sums in four independent lanes: the data-parallel shape that SIMD
/// hardware (and the compiler's auto-vectoriser) is built for.
fn choir_sum(voices: &[u32]) -> u32 {
    let mut lanes = [0u32; 4];
    let mut chunks = voices.chunks_exact(4);
    for chunk in &mut chunks {
        for (lane, voice) in lanes.iter_mut().zip(chunk) {
            *lane += voice;
        }
    }
    // chunks_exact only yields full chunks of 4. Whatever is left over
    // (0 to 3 voices) waits in the remainder and must be added by hand.
    let soloists: u32 = chunks.remainder().iter().sum();
    lanes.iter().sum::<u32>() + soloists
}

fn main() {
    let voices: Vec<u32> = (1..=10).collect();
    println!("choir sum: {}", choir_sum(&voices));
    println!("plain sum: {}", voices.iter().sum::<u32>());
    println!("empty choir: {}", choir_sum(&[]));
}
"#,
    ),
    (
        "no_std",
        r#"#![no_std]

// The sandbox still needs std to print, so this program links it under
// another name for `main`. A real firmware crate has no such line.
extern crate std as host;

mod firmware {
    use core::fmt::{self, Write};

    /// A 32-byte text buffer on the stack: no heap, no allocator.
    pub struct Beacon {
        bytes: [u8; 32],
        len: usize,
    }

    impl Beacon {
        pub const fn new() -> Self {
            Beacon { bytes: [0; 32], len: 0 }
        }

        pub fn as_str(&self) -> &str {
            // Only whole &str values are ever copied in, so this is UTF-8.
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("<garbled>")
        }
    }

    impl Write for Beacon {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let end = self.len + s.len();
            if end > self.bytes.len() {
                return Err(fmt::Error); // refuse, never truncate silently
            }
            self.bytes[self.len..end].copy_from_slice(s.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    /// Formats a status line into the beacon. `write!` works without std:
    /// it only needs a type that implements core::fmt::Write.
    pub fn signal(beacon: &mut Beacon, tower: u8, lit: bool) -> fmt::Result {
        write!(beacon, "tower {tower}: {}", if lit { "lit" } else { "dark" })
    }
}

fn main() {
    use core::fmt::Write;
    use firmware::{signal, Beacon};

    let mut beacon = Beacon::new();
    let sent = signal(&mut beacon, 7, true).is_ok();
    host::println!("{} (sent: {sent})", beacon.as_str());

    let mut small = Beacon::new();
    let refused = write!(small, "{}", "a message far longer than thirty-two bytes").is_err();
    host::println!("overflow refused: {refused}");
}
"#,
    ),
    (
        "semver_caret",
        r#"/// A plain MAJOR.MINOR.PATCH version (no pre-release or build tags).
/// Deriving Ord compares fields in declaration order, which is exactly
/// semver precedence for plain versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

fn parse(text: &str) -> Option<Version> {
    let mut parts = text.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Version { major, minor, patch })
}

/// Cargo's default requirement: `serde = "1.2.3"` means `^1.2.3`.
/// The leftmost non-zero part may never change; everything after it
/// may rise.
fn caret_allows(required: Version, candidate: Version) -> bool {
    if candidate < required {
        return false;
    }
    if required.major > 0 {
        candidate.major == required.major
    } else if required.minor > 0 {
        candidate.major == 0 && candidate.minor == required.minor
    } else {
        candidate.major == 0 && candidate.minor == 0 && candidate.patch == required.patch
    }
}

fn main() {
    let checks = [
        ("1.2.3", "1.9.0"),
        ("1.2.3", "2.0.0"),
        ("1.2.3", "1.2.2"),
        ("0.2.3", "0.2.9"),
        ("0.2.3", "0.3.0"),
        ("0.0.3", "0.0.3"),
        ("0.0.3", "0.0.4"),
        ("1.2.3", "1.x.0"),
    ];
    for (req, cand) in checks {
        match (parse(req), parse(cand)) {
            (Some(r), Some(c)) => println!("^{req} accepts {cand}: {}", caret_allows(r, c)),
            _ => println!("^{req} vs {cand}: not a version"),
        }
    }
}
"#,
    ),
    // ── Act 10 boss: the Compiler's trial ──
    (
        "compiler_alloc",
        r#"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method passes its arguments unchanged to `System`,
// which keeps the GlobalAlloc contract. Adding to an atomic does not
// allocate.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// Joins the court's names with ", " in exactly one allocation: the
/// final length is known before the first byte is written.
fn proclaim(names: &[&str]) -> String {
    let letters: usize = names.iter().map(|name| name.len()).sum();
    let separators = names.len().saturating_sub(1) * ", ".len();
    // `len()` counts bytes, so this is the exact size of the result.
    let mut line = String::with_capacity(letters + separators);
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            line.push_str(", ");
        }
        // Still within the capacity: `push_str` never reallocates here.
        line.push_str(name);
    }
    line
}

fn main() {
    let court = [
        "Ferris",
        "the Borrow Checker",
        "Vexis",
        "the Librarian",
    ];
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let line = proclaim(&court);
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    println!("{line}");
    println!("allocations: {}", after - before);
}
"#,
    ),
    (
        "compiler_lifetime",
        r#"use std::borrow::Cow;

/// Only `name` can end up inside the result (when it is borrowed), so
/// only `name` shares the lifetime of the result. `title` is read and
/// let go.
fn herald<'a>(name: &'a str, title: &str) -> Cow<'a, str> {
    if title.is_empty() {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(format!("{title} {name}"))
    }
}

fn main() {
    let name = String::from("Ferris");
    let plain = herald(&name, "");
    let titled = {
        let title = String::from("Sir");
        herald(&name, &title)
    }; // `title` is dropped here, but `titled` does not borrow from it
    for line in [&plain, &titled] {
        let how = match line {
            Cow::Borrowed(_) => "borrowed",
            Cow::Owned(_) => "owned",
        };
        println!("{line} ({how})");
    }
}
"#,
    ),
    (
        "compiler_bound",
        r#"/// Works for any T that can be compared with `>`. PartialOrd, not Ord:
/// floats have a partial order (NaN compares false with everything), so
/// they implement PartialOrd but not Ord.
fn strongest<T: PartialOrd>(champions: &[T]) -> Option<&T> {
    let mut best = champions.first()?;
    for champion in champions {
        if champion > best {
            best = champion;
        }
    }
    Some(best)
}

fn main() {
    println!("{:?}", strongest(&[3, 9, 4]));
    println!("{:?}", strongest(&[0.5, 2.5, 1.0]));
    println!("{:?}", strongest(&["ferris", "vexis", "borrow"]));
    println!("{:?}", strongest::<i32>(&[]));
}
"#,
    ),
];
