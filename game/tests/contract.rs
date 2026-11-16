// SPDX-License-Identifier: MIT
//! Audit: cross-crate contract between the game's MissionRegistry and
//! the compile-API's grader. For every mission the game ships, this
//! test asserts:
//!
//!   1. A canonical solution exists in the test fixtures (so we can
//!      grade-test it).
//!   2. The grader passes that canonical solution.
//!   3. The grader does NOT pass the unmodified starter_code (which
//!      would mean the mission is trivially won by clicking Compile).
//!   4. The grader does NOT fall through to the freeform-encounter
//!      stub for that mission's id (catches "registered a mission but
//!      forgot to add a grader case").

mod act9_programs;
mod common;

use act9_programs as a9;

use common::PLAYER_EDITION;
use pledge_and_crown::plugins::mission::MissionRegistry;
use pledge_and_crown::plugins::stub_grader::{MAIN_HINT, has_entry_point, lex, stub_verdict};
use pledge_compile_api::cargo_grader;
use pledge_compile_api::grader::grade;

/// Canonical correct solutions. MUST cover every mission id in the
/// registry — `every_mission_has_a_canonical_solution` enforces that.
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
}
fn main() {
    println!("{}", sign(-3));
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
}
fn main() {
    println!("{}", unwrap_or_zero(Some(7)));
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
        "bridge_toll" => {
            r#"fn weigh(cargo: &str) -> usize { cargo.len() }
fn main() {
    let cargo = String::from("lantern oil");
    let toll = weigh(&cargo);
    println!("toll: {toll}");
    println!("cargo: {cargo}");
}"#
        }
        "bridge_quill" => {
            r#"fn inscribe(scroll: &mut Vec<String>, entry: &str) { scroll.push(entry.to_string()); }
fn main() {
    let mut scroll = vec![String::from("north road")];
    println!("reader one: {}", scroll[0]);
    println!("reader two: {}", scroll[0]);
    inscribe(&mut scroll, "east road");
    println!("scroll: {}", scroll.join(", "));
}"#
        }
        "bridge_lantern" => {
            r#"fn before<'a>(line: &'a str, sep: &str) -> &'a str {
    match line.find(sep) { Some(i) => &line[..i], None => line }
}
fn main() {
    let lantern = String::from("oil:wick:glass");
    let part;
    { let sep = String::from(":"); part = before(&lantern, &sep); }
    println!("first: {part}");
    println!("whole: {}", before("glass", ":"));
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
            r#"enum Direction { North, East, South, West }
fn degrees(d: Direction) -> i32 {
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
}"#
        }
        // ── Act 3: Guildhall Quarter.
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
        // ── Act 3 boss: the Variant Wraith.
        "wraith_forms" => {
            r#"#[derive(Debug, Clone, Copy)]
enum Wraith { Solid, Mist, Shadow }
fn counter(form: Wraith) -> &'static str {
    match form {
        Wraith::Solid => "steel",
        Wraith::Mist => "wind",
        Wraith::Shadow => "light",
    }
}
fn main() {
    for form in [Wraith::Solid, Wraith::Mist, Wraith::Shadow] {
        println!("{form:?}: {}", counter(form));
    }
}"#
        }
        "wraith_essence" => {
            r#"#[derive(Debug, Clone, Copy)]
enum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow }
fn damage(form: Wraith, blow: u32) -> u32 {
    match form {
        Wraith::Solid { armor } => blow - armor,
        Wraith::Mist { density } => blow / density,
        Wraith::Shadow => 0,
    }
}
fn main() {
    for form in [Wraith::Solid { armor: 3 }, Wraith::Mist { density: 2 }, Wraith::Shadow] {
        println!("{form:?} takes {}", damage(form, 10));
    }
}"#
        }
        "wraith_possessed" => {
            r#"#[derive(Debug, Clone, Copy)]
enum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow, Possessed { will: u32 } }
fn damage(form: Wraith, blow: u32) -> u32 {
    match form {
        Wraith::Solid { armor } => blow - armor,
        Wraith::Mist { density } => blow / density,
        Wraith::Shadow => 0,
        Wraith::Possessed { will } => blow + will,
    }
}
fn main() {
    let forms = [Wraith::Solid { armor: 3 }, Wraith::Mist { density: 2 }, Wraith::Shadow, Wraith::Possessed { will: 4 }];
    for form in forms {
        println!("{form:?} takes {}", damage(form, 10));
    }
}"#
        }
        // ── Act 4: Trait Mage's Tower.
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
        // ── Act 4 boss: the Chimera of Vexis.
        "chimera_heads" => {
            r#"trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }
struct Fire;
struct Frost;
struct Storm;
impl Enemy for Fire { fn name(&self) -> &'static str { "fire" } fn breath(&self) -> u32 { 7 } }
impl Enemy for Frost { fn name(&self) -> &'static str { "frost" } fn breath(&self) -> u32 { 5 } }
impl Enemy for Storm { fn name(&self) -> &'static str { "storm" } fn breath(&self) -> u32 { 9 } }
fn main() {
    println!("{} breathes {}", Fire.name(), Fire.breath());
    println!("{} breathes {}", Frost.name(), Frost.breath());
    println!("{} breathes {}", Storm.name(), Storm.breath());
}"#
        }
        "chimera_bound" => {
            r#"trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }
struct Fire;
impl Enemy for Fire { fn name(&self) -> &'static str { "fire" } fn breath(&self) -> u32 { 7 } }
fn volley<T: Enemy>(head: &T, rounds: u32) -> String {
    format!("{} x{rounds} = {}", head.name(), head.breath() * rounds)
}
fn main() {
    println!("{}", volley(&Fire, 3));
}"#
        }
        "chimera_plasma" => {
            r#"trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }
struct Fire;
struct Plasma;
impl Enemy for Fire { fn name(&self) -> &'static str { "fire" } fn breath(&self) -> u32 { 7 } }
impl Enemy for Plasma { fn name(&self) -> &'static str { "plasma" } fn breath(&self) -> u32 { 12 } }
fn battle(heads: &[Box<dyn Enemy>]) -> u32 {
    let mut total = 0;
    for head in heads {
        println!("{} breathes {}", head.name(), head.breath());
        total += head.breath();
    }
    total
}
fn main() {
    let heads: Vec<Box<dyn Enemy>> = vec![Box::new(Fire), Box::new(Plasma)];
    println!("total: {}", battle(&heads));
}"#
        }
        // ── Act 5: Tavern of Tribulations.
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
        // ── Act 5 boss: the Tavernkeeper's Wager.
        "wager_stake" => {
            r#"use std::num::ParseIntError;
fn read_stake(text: &str) -> Result<u32, ParseIntError> { text.trim().parse::<u32>() }
fn double_stake(text: &str) -> Result<u32, ParseIntError> {
    let stake = read_stake(text)?;
    Ok(stake * 2)
}
fn main() {
    println!("{:?}", double_stake("20"));
}"#
        }
        "wager_from" => {
            r#"use std::num::ParseIntError;
#[derive(Debug)]
struct Short;
fn pay(purse: u32, stake: u32) -> Result<u32, Short> {
    if stake > purse { Err(Short) } else { Ok(purse - stake) }
}
#[derive(Debug)]
enum WagerError { BadStake, TooPoor }
impl From<ParseIntError> for WagerError { fn from(_: ParseIntError) -> Self { WagerError::BadStake } }
impl From<Short> for WagerError { fn from(_: Short) -> Self { WagerError::TooPoor } }
fn place_bet(text: &str, purse: u32) -> Result<u32, WagerError> {
    let stake = text.trim().parse::<u32>()?;
    let left = pay(purse, stake)?;
    Ok(left)
}
fn main() {
    println!("{:?}", place_bet("15", 50));
}"#
        }
        "wager_quest" => {
            r#"use std::num::ParseIntError;
#[derive(Debug)]
struct WrongKey;
#[derive(Debug)]
struct Short;
struct Reward { gold: u32 }
#[derive(Debug)]
enum QuestError { BadMap, Locked, Broke }
impl From<ParseIntError> for QuestError { fn from(_: ParseIntError) -> Self { QuestError::BadMap } }
impl From<WrongKey> for QuestError { fn from(_: WrongKey) -> Self { QuestError::Locked } }
impl From<Short> for QuestError { fn from(_: Short) -> Self { QuestError::Broke } }
fn read_map(map: &str) -> Result<u32, ParseIntError> { map.trim().parse::<u32>() }
fn open_vault(key: &str) -> Result<u32, WrongKey> { if key == "brass" { Ok(30) } else { Err(WrongKey) } }
fn pay_guide(gold: u32, leagues: u32) -> Result<u32, Short> {
    let fee = leagues * 2;
    if fee > gold { Err(Short) } else { Ok(gold - fee) }
}
fn quest(map: &str, key: &str) -> Result<Reward, QuestError> {
    let leagues = read_map(map)?;
    let gold = open_vault(key)?;
    let left = pay_guide(gold, leagues)?;
    Ok(Reward { gold: left })
}
fn main() {
    println!("{:?}", quest("4", "brass").map(|r| r.gold));
}"#
        }
        // ── Act 6: Iterator Forge.
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
        // ── Act 6 boss: the Forge-Golem.
        "golem_parts" => {
            r#"struct Part { name: &'static str, armoured: bool, weakness: u32 }
fn weak_points(parts: &[Part]) -> Vec<String> {
    parts.iter().filter(|p| !p.armoured).map(|p| format!("{} ({})", p.name, p.weakness)).collect()
}
fn main() {
    let parts = vec![Part { name: "arm", armoured: false, weakness: 3 }];
    println!("{:?}", weak_points(&parts));
}"#
        }
        "golem_sum" => {
            r#"struct Part { armoured: bool, weakness: u32 }
fn core_damage(parts: &[Part]) -> u32 {
    let total: u32 = parts.iter().filter(|p| !p.armoured).map(|p| p.weakness).sum();
    total * 2
}
fn main() {
    let parts = vec![Part { armoured: false, weakness: 3 }];
    println!("{}", core_damage(&parts));
}"#
        }
        "golem_core" => {
            r#"struct Part { armoured: bool, weakness: u32 }
fn total_damage(parts: &[Part], blow: u32) -> u32 {
    parts.iter().filter(|p| !p.armoured).map(|p| blow * p.weakness).fold(0, |total, hit| total + hit)
}
fn main() {
    let parts = vec![Part { armoured: false, weakness: 3 }];
    println!("{}", total_damage(&parts, 2));
}"#
        }
        // ── Act 7: Concurrent Coast.
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
        // ── Act 7 boss: the Tide Hydra.
        "hydra_senders" => {
            r#"use std::sync::mpsc::{self, Sender};
fn fight(head: &'static str, tx: Sender<String>) {
    tx.send(format!("{head} down")).unwrap();
}
fn main() {
    let (tx, rx) = mpsc::channel();
    for head in ["fang", "venom"] {
        fight(head, tx.clone());
    }
    drop(tx);
    let _: Vec<String> = rx.iter().collect();
}"#
        }
        "hydra_threads" => {
            r#"use std::sync::mpsc;
use std::thread;
fn main() {
    let (tx, rx) = mpsc::channel();
    for head in ["fang", "venom"] {
        let tx = tx.clone();
        thread::spawn(move || tx.send(head).unwrap());
    }
    drop(tx);
    let mut reports: Vec<&str> = rx.iter().collect();
    reports.sort();
}"#
        }
        "hydra_body" => {
            r#"use std::sync::mpsc::{self, Receiver};
enum Report { Hit, Down }
fn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 {
    let mut hits = 0;
    let mut down = 0;
    while down < heads {
        match rx.recv() {
            Ok(Report::Hit) => hits += 1,
            Ok(Report::Down) => down += 1,
            Err(_) => break,
        }
    }
    hits
}
fn main() {
    let (tx, rx) = mpsc::channel();
    tx.send(Report::Hit).unwrap();
    tx.send(Report::Down).unwrap();
    println!("{}", await_heads(&rx, 1));
}"#
        }
        // ── Act 8: Vault of Pointers.
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
        // ── Act 8 boss: the Memory Warden.
        "warden_box" => {
            r#"enum Ledger { Entry(u32, Box<Ledger>), Closed }
fn total(ledger: &Ledger) -> u32 {
    match ledger {
        Ledger::Entry(amount, rest) => amount + total(rest),
        Ledger::Closed => 0,
    }
}
fn main() {
    let ledger = Ledger::Entry(5, Box::new(Ledger::Closed));
    println!("{}", total(&ledger));
}"#
        }
        "warden_shared" => {
            r#"use std::cell::RefCell;
use std::rc::Rc;
struct Debtor { vault: Rc<RefCell<u32>> }
fn pay(debtor: &Debtor, coins: u32) {
    *debtor.vault.borrow_mut() += coins;
}
fn main() {
    let vault = Rc::new(RefCell::new(0));
    let miller = Debtor { vault: Rc::clone(&vault) };
    pay(&miller, 5);
    println!("{}", vault.borrow());
}"#
        }
        "warden_weak" => {
            r#"use std::cell::RefCell;
use std::rc::{Rc, Weak};
struct Debtor { name: &'static str, owes: RefCell<Weak<Debtor>> }
fn main() {
    let a = Rc::new(Debtor { name: "a", owes: RefCell::new(Weak::new()) });
    let b = Rc::new(Debtor { name: "b", owes: RefCell::new(Weak::new()) });
    *a.owes.borrow_mut() = Rc::downgrade(&b);
    let next = a.owes.borrow().upgrade().map(|d| d.name);
    println!("{} owes {:?}", a.name, next);
}"#
        }
        // ── Act 9: Forbidden Library (the player-facing solutions; their
        // `unsafe` is text in a string, compiled only as player code).
        "raw_pointer" => {
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
}"#
        }
        "extern_c" => {
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
}"#
        }
        "macro_rules" => {
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
}"#
        }
        "drop_trait" => {
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
} // remaining locals drop in reverse order: `_second`, then `_first`"#
        }
        "send_sync" => {
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
}"#
        }
        "maybe_uninit" => {
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
}"#
        }
        // ── Act 9 boss: the Librarian.
        "librarian_index" => {
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
}"#
        }
        "librarian_demand" => {
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
}"#
        }
        "librarian_seal" => {
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
}"#
        }
        _ => return None,
    })
}

#[test]
fn every_mission_has_a_canonical_solution() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        assert!(
            canonical_solution(m.id).is_some(),
            "mission `{}` has no canonical_solution() entry — add one before the contract suite can grade it",
            m.id
        );
    }
}

#[test]
fn canonical_solutions_pass_the_grader() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src = canonical_solution(m.id).expect("checked by previous test");
        let v = grade(m.id, src);
        assert!(
            v.ok,
            "canonical solution for `{}` was rejected: {v:?}",
            m.id
        );
    }
}

#[test]
fn unmodified_starter_code_does_not_trivially_win() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let v = grade(m.id, m.starter_code);
        assert!(
            !v.ok,
            "mission `{}` is trivially solved by submitting the starter_code unchanged: {v:?}",
            m.id
        );
    }
}

#[test]
fn server_and_stub_flavor_agree_byte_for_byte() {
    // Stub_grader duplicates the server's flavor strings on purpose
    // (game crate must not depend on pledge_compile_api at runtime).
    // The previous integration miss (190fdda) was caused by *wording*
    // drift, not missing arms — the existing coverage test only
    // catches the latter. This test catches both.
    //
    // Strip the `[stub] ` prefix the stub adds at the boundary, then
    // compare to the server's stdout/stderr verbatim.
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src = canonical_solution(m.id).expect("checked elsewhere");
        let server = grade(m.id, src);
        let stub = stub_verdict(m.id, src)
            .unwrap_or_else(|| panic!("stub_verdict missing arm for `{}`", m.id));
        assert_eq!(
            server.ok, stub.ok,
            "ok flag drift on `{}`: server={} stub={}",
            m.id, server.ok, stub.ok
        );
        // Stub messages carry the `[stub] ` marker; strip it before
        // comparing to the server's verbatim flavor text.
        let stub_stdout = stub.stdout.strip_prefix("[stub] ").unwrap_or(&stub.stdout);
        let stub_stderr = stub.stderr.strip_prefix("[stub] ").unwrap_or(&stub.stderr);
        assert_eq!(
            server.stdout, stub_stdout,
            "stdout drift on canonical `{}`",
            m.id
        );
        assert_eq!(
            server.stderr, stub_stderr,
            "stderr drift on canonical `{}`",
            m.id
        );
    }
}

/// Slow audit: every canonical solution must also pass `cargo check`.
///
/// The pattern grader checks token presence, not language correctness —
/// in principle it could approve syntactically-broken Rust as long as
/// the right keywords appear. This test closes that loop by running
/// every canonical solution through the real `cargo_grader::compile_check`
/// and asserting compilation succeeds.
///
/// `#[ignore]` because each invocation forks a real `cargo check` (~1-3s
/// warm × 21 missions ≈ 30-60s total). Runs alongside the other slow
/// audits via:
///
/// ```text
/// cargo test --workspace -- --ignored
/// ```
#[test]
#[ignore = "slow: forks cargo check 21 times"]
fn every_canonical_solution_passes_cargo_check() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let src =
            canonical_solution(m.id).expect("checked by every_mission_has_a_canonical_solution");
        let verdict = cargo_grader::compile_check(src)
            .unwrap_or_else(|e| panic!("cargo_grader setup failed for `{}`: {e:#}", m.id));
        assert!(
            verdict.ok,
            "canonical solution for `{}` is not valid Rust per cargo check.\n--- source ---\n{src}\n--- stderr ---\n{}",
            m.id, verdict.stderr
        );
    }
}

// ── Grader characterisation snapshots ───────────────────────────────
//
// Both graders were refactored from hand-written per-mission `if` chains
// into a rule table. These snapshots pin the exact observable output
// (ok flag + stdout + stderr) for every mission against:
//
//   * the canonical solution above,
//   * the mission's starter code,
//   * an empty string,
//   * near-misses: the canonical solution with one whitespace-delimited
//     token removed, one near-miss per distinct server failure message.
//
// Inputs are stored in the snapshot files themselves, so the replay
// tests (`stub_grader_matches_snapshot` here, and
// `compile-api/tests/grader_snapshot.rs`) are pure replays. To
// regenerate (only when grader output is meant to change):
//
//   PNC_UPDATE_GRADER_SNAPSHOTS=1 cargo test -p pledge_and_crown --test contract

const STUB_SNAPSHOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/snapshots/stub_grader.json"
);
const SERVER_SNAPSHOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../compile-api/tests/snapshots/grader.json"
);

/// Each near-miss is the canonical source with exactly one
/// whitespace-delimited token cut out.
fn single_token_removals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let bytes: Vec<(usize, char)> = src.char_indices().collect();
    for (i, c) in bytes
        .iter()
        .copied()
        .chain(std::iter::once((src.len(), ' ')))
    {
        match (c.is_whitespace(), start) {
            (false, None) => start = Some(i),
            (true, Some(s)) => {
                out.push(format!("{}{}", &src[..s], &src[i..]));
                start = None;
            }
            _ => {}
        }
    }
    out
}

/// (encounter_id, case label, source) for every snapshot case.
fn snapshot_cases() -> Vec<(String, String, String)> {
    let reg = MissionRegistry::default();
    let mut cases = Vec::new();
    for m in &reg.missions {
        let canonical = canonical_solution(m.id).expect("checked elsewhere");
        cases.push((m.id.to_string(), "canonical".into(), canonical.to_string()));
        cases.push((
            m.id.to_string(),
            "starter".into(),
            m.starter_code.to_string(),
        ));
        cases.push((m.id.to_string(), "empty".into(), String::new()));
        let mut seen = std::collections::BTreeSet::new();
        for (n, src) in single_token_removals(canonical).into_iter().enumerate() {
            let v = grade(m.id, &src);
            if !v.ok && seen.insert(v.stderr.clone()) {
                cases.push((m.id.to_string(), format!("near_miss_{n}"), src));
            }
        }
    }
    // Unknown id: server falls back to freeform, stub returns None.
    for (label, src) in [("empty", ""), ("garbage", "fn main() { /* ? */ }")] {
        cases.push(("not_a_real_mission".into(), label.into(), src.into()));
    }
    cases
}

fn load_snapshot(path: &str) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read snapshot {path}: {e} (regenerate, see comment)"));
    let v: serde_json::Value = serde_json::from_str(&text).expect("snapshot is valid JSON");
    v.as_array().expect("snapshot is a JSON array").clone()
}

#[test]
fn stub_grader_matches_snapshot() {
    if std::env::var_os("PNC_UPDATE_GRADER_SNAPSHOTS").is_some() {
        let mut stub_rows = Vec::new();
        let mut server_rows = Vec::new();
        for (id, case, src) in snapshot_cases() {
            let stub = stub_verdict(&id, &src)
                .map(|s| serde_json::json!({ "ok": s.ok, "stdout": s.stdout, "stderr": s.stderr }));
            stub_rows.push(serde_json::json!({
                "id": id, "case": case, "input": src, "verdict": stub,
            }));
            let g = grade(&id, &src);
            server_rows.push(serde_json::json!({
                "id": id, "case": case, "input": src,
                "verdict": { "ok": g.ok, "stdout": g.stdout, "stderr": g.stderr },
            }));
        }
        for (path, rows) in [(STUB_SNAPSHOT, stub_rows), (SERVER_SNAPSHOT, server_rows)] {
            let dir = std::path::Path::new(path).parent().expect("has parent");
            std::fs::create_dir_all(dir).expect("create snapshot dir");
            let body = serde_json::to_string_pretty(&rows).expect("serialize") + "\n";
            std::fs::write(path, body).expect("write snapshot");
        }
    }

    let rows = load_snapshot(STUB_SNAPSHOT);
    assert!(rows.len() > 4 * 57, "snapshot suspiciously small");
    for row in &rows {
        let id = row["id"].as_str().expect("id");
        let case = row["case"].as_str().expect("case");
        let input = row["input"].as_str().expect("input");
        let actual = stub_verdict(id, input)
            .map(|s| serde_json::json!({ "ok": s.ok, "stdout": s.stdout, "stderr": s.stderr }));
        let actual = actual.unwrap_or(serde_json::Value::Null);
        assert_eq!(
            actual, row["verdict"],
            "stub grader drift on `{id}` case `{case}`"
        );
    }
}

/// The two crates cannot share code (MIT vs AGPL), so the matching engine
/// and the rule table are duplicated. Keep both textually identical so a
/// one-sided edit can't hide behind inputs the snapshot happens not to
/// cover.
#[test]
fn server_and_stub_rule_tables_are_textually_identical() {
    fn region(path: &str, name: &str) -> String {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let text = text.replace("\r\n", "\n");
        let begin = format!("// BEGIN {name}");
        let end_marker = format!("// END {name}");
        let start = text
            .find(&begin)
            .unwrap_or_else(|| panic!("{path}: no `{begin}` marker"));
        let body_start = start + text[start..].find('\n').expect("marker line ends") + 1;
        let end = text
            .find(&end_marker)
            .unwrap_or_else(|| panic!("{path}: no `{end_marker}` marker"));
        assert!(body_start < end, "{path}: `{name}` markers out of order");
        text[body_start..end].to_string()
    }
    let stub_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/plugins/stub_grader.rs");
    let server_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../compile-api/src/grader.rs");
    for name in ["MISSION TABLE", "GRADER ENGINE"] {
        let stub = region(stub_path, name);
        let server = region(server_path, name);
        for (n, (a, b)) in stub.lines().zip(server.lines()).enumerate() {
            assert_eq!(a, b, "`{name}` regions diverge at region line {}", n + 1);
        }
        assert_eq!(
            stub.lines().count(),
            server.lines().count(),
            "`{name}` regions differ in length"
        );
    }
}

// ── Evidence corpus ─────────────────────────────────────────────────
//
// The pattern grader is the only grader in the browser build, so "it
// passes the canonical solution" is not enough. For every mission this
// corpus checks, against BOTH graders (which must agree):
//
//   (a) the player-facing solution (editor/solutions/data.rs) passes;
//   (b) the canonical fixture above passes;
//   (c) the player solution hidden in a block comment, in line comments
//       and in a raw string, next to an empty `fn main`, FAILS;
//   (d) correct programs the old substring grader rejected (the audit's
//       examples, plus differently-written solutions for every mission
//       whose checks were tightened) pass;
//   (e) wrong programs the old grader accepted, and that a token check
//       can catch, fail;
//   (f) the unmodified starter code fails.
//
// Run with `--nocapture` to see the counts.

use pledge_and_crown::plugins::editor::solutions::canonical_solution as player_solution;

/// Grade with both graders, insist they agree, return the verdict.
fn graded_ok(id: &str, src: &str) -> bool {
    let server = grade(id, src);
    let stub = stub_verdict(id, src).unwrap_or_else(|| panic!("no stub rule for `{id}`"));
    assert_eq!(
        server.ok, stub.ok,
        "graders disagree on `{id}`:\n{src}\nserver={server:?}\nstub={stub:?}"
    );
    server.ok
}

/// Everything of the Wager's last stage except `quest`, shared by trial
/// programs in `CORRECT` and `WRONG` (mirrors `expectations_proof.rs`).
macro_rules! quest_items {
    () => {
        "use std::num::ParseIntError;\n#[derive(Debug)]\nstruct WrongKey;\n#[derive(Debug)]\nstruct Short;\n#[derive(Debug)]\nstruct Reward { gold: u32 }\n#[derive(Debug)]\nenum QuestError { BadMap, Locked, Broke }\nimpl From<ParseIntError> for QuestError { fn from(_: ParseIntError) -> Self { QuestError::BadMap } }\nimpl From<WrongKey> for QuestError { fn from(_: WrongKey) -> Self { QuestError::Locked } }\nimpl From<Short> for QuestError { fn from(_: Short) -> Self { QuestError::Broke } }\nfn read_map(map: &str) -> Result<u32, ParseIntError> { map.trim().parse::<u32>() }\nfn open_vault(key: &str) -> Result<u32, WrongKey> { if key == \"brass\" { Ok(30) } else { Err(WrongKey) } }\nfn pay_guide(gold: u32, leagues: u32) -> Result<u32, Short> { let fee = leagues * 2; if fee > gold { Err(Short) } else { Ok(gold - fee) } }\nfn main() { for (m, k) in [(\"4\", \"brass\"), (\"far\", \"brass\"), (\"4\", \"bone\"), (\"20\", \"brass\")] { println!(\"{:?}\", quest(m, k).map(|r| r.gold)); } }\n"
    };
}

/// Correct programs, each a complete program that builds with `rustc` at
/// the server's edition (`PLAYER_EDITION`). `(mission, why it is here, source)`.
const CORRECT: &[(&str, &str, &str)] = &[
    // ── audit "correct but rejected" (G1, G3-G7, G10, G11) ──
    (
        "double_function",
        "G1 2 * n",
        "fn double(n: i32) -> i32 {\n    2 * n\n}\nfn main() {\n    println!(\"{}\", double(21));\n}\n",
    ),
    (
        "double_function",
        "G1 n + n",
        "fn double(n: i32) -> i32 {\n    n + n\n}\nfn main() {\n    println!(\"{}\", double(21));\n}\n",
    ),
    (
        "borrow_mut",
        "G3 param named n",
        "fn bump(n: &mut i32) {\n    *n += 1;\n}\nfn main() {\n    let mut v = 0;\n    bump(&mut v);\n    println!(\"{v}\");\n}\n",
    ),
    (
        "closure_basic",
        "G3 |x, y|",
        "fn main() {\n    let add = |x, y| x + y;\n    println!(\"{}\", add(2, 3));\n}\n",
    ),
    (
        "iter_map_collect",
        "G3 |n|",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let d: Vec<i32> = v.iter().map(|n| n * 2).collect();\n    println!(\"{d:?}\");\n}\n",
    ),
    (
        "string_vs_str",
        "G4 to_string",
        "fn greet(name: &str) {\n    println!(\"hi {name}\");\n}\nfn main() {\n    let s = \"you\".to_string();\n    greet(&s);\n    greet(\"anon\");\n}\n",
    ),
    (
        "result_question_mark",
        "G5 Ok(expr?)",
        "fn parse_int(s: &str) -> Result<i32, String> {\n    Ok(s.parse::<i32>().map_err(|e| e.to_string())?)\n}\nfn main() {\n    println!(\"{:?}\", parse_int(\"42\"));\n}\n",
    ),
    (
        "derive_debug",
        "G6 derive(Debug, Clone) + {:#?}",
        "#[derive(Debug, Clone)]\nstruct Item {\n    name: String,\n}\nfn main() {\n    let item = Item { name: String::from(\"ring\") };\n    println!(\"{item:#?}\");\n    println!(\"{}\", item.name);\n}\n",
    ),
    (
        "derive_debug",
        "G6 derive(Clone, Debug) + {:?}",
        "#[derive(Clone, Debug)]\nstruct Item {\n    name: String,\n}\nfn main() {\n    let item = Item { name: String::from(\"ring\") };\n    println!(\"{:?}\", item);\n    println!(\"{}\", item.name);\n}\n",
    ),
    (
        "assoc_new",
        "G7 -> Hero",
        "struct Hero {\n    level: i32,\n}\nimpl Hero {\n    fn new(level: i32) -> Hero {\n        Hero { level }\n    }\n}\nfn main() {\n    let h = Hero::new(5);\n    println!(\"{}\", h.level);\n}\n",
    ),
    (
        "generic_struct",
        "G10 tuple Pair<T>(T, T)",
        "struct Pair<T>(T, T);\nfn main() {\n    let p = Pair(1, 2);\n    println!(\"{} {}\", p.0, p.1);\n}\n",
    ),
    (
        "generic_struct",
        "G10 Pair<U>",
        "struct Pair<U> {\n    a: U,\n    b: U,\n}\nfn main() {\n    let p = Pair { a: 1, b: 2 };\n    println!(\"{} {}\", p.a, p.b);\n}\n",
    ),
    (
        "lifetimes",
        "G10 lifetime 's",
        "fn longest<'s>(x: &'s str, y: &'s str) -> &'s str {\n    if x.len() > y.len() { x } else { y }\n}\nfn main() {\n    println!(\"{}\", longest(\"aa\", \"b\"));\n}\n",
    ),
    (
        "iter_zip",
        "G11 std::iter::zip",
        "fn main() {\n    let a = vec![1, 2];\n    let b = vec![3, 4];\n    for (x, y) in std::iter::zip(&a, &b) {\n        println!(\"{x},{y}\");\n    }\n}\n",
    ),
    (
        "closure_move",
        "audit move|| without space",
        "fn main() {\n    let name = String::from(\"Garin\");\n    let greet = move||println!(\"hail, {name}\");\n    greet();\n}\n",
    ),
    // ── differently-written solutions for every tightened mission ──
    (
        "if_else_sign",
        "tightened: returns",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 {\n        return \"neg\";\n    } else if n == 0 {\n        return \"zero\";\n    } else {\n        return \"pos\";\n    }\n}\nfn main() {\n    println!(\"{}\", sign(-3));\n}\n",
    ),
    (
        "if_else_sign",
        "tightened: > 0 branch, let-bound",
        "fn sign(n: i32) -> &'static str {\n    let label = if n < 0 { \"neg\" } else if n > 0 { \"pos\" } else { \"zero\" };\n    label\n}\nfn main() {\n    println!(\"{}\", sign(4));\n}\n",
    ),
    (
        "if_else_sign",
        "tightened: zero first, no spaces",
        "fn sign(n: i32) -> &'static str {\n    if n==0 {\"zero\"} else if n<0 {\"neg\"} else {\"pos\"}\n}\nfn main() {\n    println!(\"{}\", sign(0));\n}\n",
    ),
    (
        "loop_break",
        "tightened: double before test",
        "fn main() {\n    let mut n = 1;\n    let r = loop {\n        n *= 2;\n        if n >= 100 {\n            break n;\n        }\n    };\n    println!(\"{r}\");\n}\n",
    ),
    (
        "loop_break",
        "tightened: x = x * 2",
        "fn main() {\n    let mut value = 1;\n    let first = loop {\n        if value >= 100 {\n            break value;\n        }\n        value = value * 2;\n    };\n    println!(\"{first}\");\n}\n",
    ),
    (
        "loop_break",
        "tightened: x = 2 * x",
        "fn main() {\n    let mut value = 1;\n    let first = loop {\n        if value >= 100 {\n            break value;\n        }\n        value = 2 * value;\n    };\n    println!(\"{first}\");\n}\n",
    ),
    (
        "match_option",
        "tightened: None arm first",
        "fn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        None => 0,\n        Some(v) => v,\n    }\n}\nfn main() {\n    println!(\"{}\", unwrap_or_zero(Some(7)));\n}\n",
    ),
    (
        "match_option",
        "tightened: block arms",
        "fn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        Some(value) => {\n            value\n        }\n        None => {\n            0\n        }\n    }\n}\nfn main() {\n    println!(\"{}\", unwrap_or_zero(None));\n}\n",
    ),
    (
        "match_option",
        "tightened: return arms",
        "fn unwrap_or_zero(maybe: Option<i32>) -> i32 {\n    match maybe {\n        Some(n) => return n,\n        None => return 0,\n    }\n}\nfn main() {\n    println!(\"{}\", unwrap_or_zero(Some(1)));\n}\n",
    ),
    (
        "tuple_struct",
        "tightened: .0 on a temporary",
        "struct Meters(f64);\nfn main() {\n    let raw: f64 = 3.5;\n    println!(\"{}\", Meters(raw).0);\n}\n",
    ),
    (
        "tuple_struct",
        "tightened: float literal and a real .0",
        "struct Meters(f64);\nfn main() {\n    let d = Meters(3.0);\n    let inner = d.0;\n    println!(\"{inner}\");\n}\n",
    ),
    (
        "trait_def",
        "tightened: trait defined after impl",
        "struct Fire;\nimpl Element for Fire {\n    fn name(&self) -> &str {\n        \"fire\"\n    }\n}\ntrait Element {\n    fn name(&self) -> &str;\n}\nfn main() {\n    println!(\"{}\", Fire.name());\n}\n",
    ),
    (
        "trait_def",
        "tightened: default method, empty impl",
        "trait Element {\n    fn name(&self) -> &str {\n        \"element\"\n    }\n}\nstruct Fire;\nimpl Element for Fire {}\nfn main() {\n    let f = Fire;\n    println!(\"{}\", f.name());\n}\n",
    ),
    (
        "trait_def",
        "tightened: String-returning trait",
        "trait Element {\n    fn name(&self) -> String;\n}\nstruct Fire;\nimpl Element for Fire {\n    fn name(&self) -> String {\n        String::from(\"fire\")\n    }\n}\nfn main() {\n    println!(\"{}\", Fire.name());\n}\n",
    ),
    (
        "iter_fold",
        "tightened: |sum, &n|",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.iter().fold(0, |sum, &n| sum + n);\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "tightened: typed params, block body",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total: i32 = v.into_iter().fold(0, |acc: i32, x: i32| {\n        let next = acc + x;\n        next\n    });\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "tightened: x + acc",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.iter().fold(0, |acc, x| x + acc);\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "named fn defined before the fold",
        "fn add(acc: i32, x: &i32) -> i32 {\n    acc + x\n}\nfn main() {\n    let v = vec![1, 2, 3];\n    println!(\"{}\", v.iter().fold(0, add));\n}\n",
    ),
    (
        "iter_fold",
        "named fn defined after the fold",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.into_iter().fold(0, add);\n    println!(\"{total}\");\n}\nfn add(acc: i32, x: i32) -> i32 {\n    acc + x\n}\n",
    ),
    (
        "iter_fold",
        "closure bound to a name first",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let add = |acc: i32, x: &i32| acc + x;\n    println!(\"{}\", v.iter().fold(0, add));\n}\n",
    ),
    (
        "intro_let_binding",
        "entry point: pub fn main",
        "pub fn main() {\n    let answer = 42;\n    println!(\"{answer}\");\n}\n",
    ),
    (
        "closure_basic",
        "tightened: typed params, b + a",
        "fn main() {\n    let add = |a: i32, b: i32| -> i32 { b + a };\n    println!(\"{}\", add(2, 3));\n}\n",
    ),
    (
        "borrow_mut",
        "tightened: *counter = *counter + 1",
        "fn bump(counter: &mut i32) {\n    *counter = *counter + 1;\n}\nfn main() {\n    let mut c = 0;\n    bump(&mut c);\n    println!(\"{c}\");\n}\n",
    ),
    (
        "generic_struct",
        "tightened: bound on T",
        "struct Pair<T: Copy> {\n    first: T,\n    second: T,\n}\nfn main() {\n    let p = Pair { first: 1, second: 2 };\n    println!(\"{} {}\", p.first, p.second);\n}\n",
    ),
    (
        "lifetimes",
        "tightened: lifetime 'long",
        "fn longest<'long>(a: &'long str, b: &'long str) -> &'long str {\n    if a.len() >= b.len() { a } else { b }\n}\nfn main() {\n    println!(\"{}\", longest(\"x\", \"yy\"));\n}\n",
    ),
    (
        "generic_fn",
        "tightened: type parameter U",
        "fn larger<U: PartialOrd>(a: U, b: U) -> U {\n    if a > b { a } else { b }\n}\nfn main() {\n    println!(\"{}\", larger(3, 7));\n}\n",
    ),
    (
        "atomic",
        "token match: AtomicU32",
        "use std::sync::atomic::{AtomicU32, Ordering};\nfn main() {\n    let c = AtomicU32::new(0);\n    c.fetch_add(1, Ordering::SeqCst);\n    println!(\"{}\", c.load(Ordering::SeqCst));\n}\n",
    ),
    // ── the Borrow Checker's trial: realistic alternatives (the same
    // programs pass the server check in `expectations_proof.rs`) ──
    (
        "bridge_toll",
        "trial alt: chars().count() and as_str()",
        "fn weigh(cargo: &str) -> usize {\n    cargo.chars().count()\n}\nfn main() {\n    let cargo = String::from(\"lantern oil\");\n    let toll = weigh(cargo.as_str());\n    println!(\"toll: {toll}\");\n    println!(\"cargo: {cargo}\");\n}\n",
    ),
    (
        "bridge_quill",
        "trial alt: readers clone, other names",
        "fn inscribe(list: &mut Vec<String>, line: &str) {\n    list.push(String::from(line));\n}\nfn main() {\n    let mut scroll = vec![String::from(\"north road\")];\n    let first = scroll[0].clone();\n    let second = scroll[0].clone();\n    inscribe(&mut scroll, \"east road\");\n    println!(\"reader one: {first}\");\n    println!(\"reader two: {second}\");\n    println!(\"scroll: {}\", scroll.join(\", \"));\n}\n",
    ),
    (
        "bridge_lantern",
        "trial alt: two named lifetimes, split",
        "fn before<'a, 'b>(line: &'a str, sep: &'b str) -> &'a str {\n    line.split(sep).next().unwrap_or(line)\n}\nfn main() {\n    let lantern = String::from(\"oil:wick:glass\");\n    let part;\n    {\n        let sep = String::from(\":\");\n        part = before(&lantern, &sep);\n    }\n    println!(\"first: {part}\");\n    println!(\"whole: {}\", before(\"glass\", \":\"));\n}\n",
    ),
    // ── the Act 3-5 trials: realistic alternatives (the same programs
    // pass the server check in `expectations_proof.rs`) ──
    (
        "wraith_forms",
        "trial alt: arms in another order, {:?} with an argument",
        "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid,\n    Mist,\n    Shadow,\n}\nfn counter(form: Wraith) -> &'static str {\n    match form {\n        Wraith::Shadow => \"light\",\n        Wraith::Mist => \"wind\",\n        Wraith::Solid => \"steel\",\n    }\n}\nfn main() {\n    for form in [Wraith::Solid, Wraith::Mist, Wraith::Shadow] {\n        println!(\"{:?}: {}\", form, counter(form));\n    }\n}\n",
    ),
    (
        "wraith_essence",
        "trial alt: fields bound to other names",
        "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid { armor: u32 },\n    Mist { density: u32 },\n    Shadow,\n}\nfn damage(form: Wraith, blow: u32) -> u32 {\n    match form {\n        Wraith::Solid { armor: a } => blow - a,\n        Wraith::Mist { density: d } => blow / d,\n        Wraith::Shadow => 0,\n    }\n}\nfn main() {\n    let forms = [Wraith::Solid { armor: 3 }, Wraith::Mist { density: 2 }, Wraith::Shadow];\n    for f in forms.iter() {\n        println!(\"{:?} takes {}\", f, damage(*f, 10));\n    }\n}\n",
    ),
    (
        "wraith_possessed",
        "trial alt: new arm first, will renamed",
        "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid { armor: u32 },\n    Mist { density: u32 },\n    Shadow,\n    Possessed { will: u32 },\n}\nfn damage(form: Wraith, blow: u32) -> u32 {\n    match form {\n        Wraith::Possessed { will: w } => w + blow,\n        Wraith::Solid { armor } => blow - armor,\n        Wraith::Mist { density } => blow / density,\n        Wraith::Shadow => 0,\n    }\n}\nfn main() {\n    let forms = [\n        Wraith::Solid { armor: 3 },\n        Wraith::Mist { density: 2 },\n        Wraith::Shadow,\n        Wraith::Possessed { will: 4 },\n    ];\n    for form in forms {\n        println!(\"{form:?} takes {}\", damage(form, 10));\n    }\n}\n",
    ),
    (
        "chimera_heads",
        "trial alt: heads printed through &dyn Enemy",
        "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nfn main() {\n    let heads: [&dyn Enemy; 3] = [&Fire, &Frost, &Storm];\n    for h in heads {\n        println!(\"{} breathes {}\", h.name(), h.breath());\n    }\n}\n",
    ),
    (
        "chimera_bound",
        "trial alt: `&impl Enemy` argument",
        "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nfn volley(head: &impl Enemy, rounds: u32) -> String {\n    let total = head.breath() * rounds;\n    format!(\"{} x{} = {}\", head.name(), rounds, total)\n}\nfn main() {\n    println!(\"{}\", volley(&Fire, 3));\n    println!(\"{}\", volley(&Frost, 3));\n    println!(\"{}\", volley(&Storm, 3));\n}\n",
    ),
    (
        "chimera_bound",
        "trial alt: `where H: Enemy` clause",
        "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nfn volley<H>(head: &H, rounds: u32) -> String\nwhere\n    H: Enemy,\n{\n    format!(\"{} x{rounds} = {}\", head.name(), head.breath() * rounds)\n}\nfn main() {\n    println!(\"{}\", volley(&Fire, 3));\n    println!(\"{}\", volley(&Frost, 3));\n    println!(\"{}\", volley(&Storm, 3));\n}\n",
    ),
    (
        "chimera_plasma",
        "trial alt: iterator map and sum",
        "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nstruct Plasma;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nimpl Enemy for Plasma {\n    fn name(&self) -> &'static str { \"plasma\" }\n    fn breath(&self) -> u32 { 12 }\n}\nfn battle(enemies: &[Box<dyn Enemy>]) -> u32 {\n    enemies\n        .iter()\n        .map(|e| {\n            println!(\"{} breathes {}\", e.name(), e.breath());\n            e.breath()\n        })\n        .sum()\n}\nfn main() {\n    let heads: Vec<Box<dyn Enemy>> =\n        vec![Box::new(Fire), Box::new(Frost), Box::new(Storm), Box::new(Plasma)];\n    let total = battle(&heads);\n    println!(\"total: {total}\");\n}\n",
    ),
    (
        "wager_stake",
        "trial alt: full std path, `?` inside Ok(..)",
        "fn read_stake(text: &str) -> Result<u32, std::num::ParseIntError> {\n    text.trim().parse::<u32>()\n}\nfn double_stake(text: &str) -> Result<u32, std::num::ParseIntError> {\n    Ok(read_stake(text)? * 2)\n}\nfn main() {\n    for stake in [\"20\", \"twenty\"] {\n        println!(\"{:?}\", double_stake(stake));\n    }\n}\n",
    ),
    (
        "wager_from",
        "trial alt: full std path, Self:: variants, inferred parse",
        "#[derive(Debug)]\nstruct Short;\nfn pay(purse: u32, stake: u32) -> Result<u32, Short> {\n    if stake > purse { Err(Short) } else { Ok(purse - stake) }\n}\n#[derive(Debug)]\nenum WagerError {\n    BadStake,\n    TooPoor,\n}\nimpl From<std::num::ParseIntError> for WagerError {\n    fn from(_e: std::num::ParseIntError) -> WagerError {\n        Self::BadStake\n    }\n}\nimpl From<Short> for WagerError {\n    fn from(_e: Short) -> WagerError {\n        Self::TooPoor\n    }\n}\nfn place_bet(text: &str, purse: u32) -> Result<u32, WagerError> {\n    let stake: u32 = text.trim().parse()?;\n    Ok(pay(purse, stake)?)\n}\nfn main() {\n    println!(\"{:?}\", place_bet(\"15\", 50));\n    println!(\"{:?}\", place_bet(\"lots\", 50));\n    println!(\"{:?}\", place_bet(\"80\", 50));\n}\n",
    ),
    (
        "wager_quest",
        "trial alt: steps 2 and 3 nested in one expression",
        concat!(
            quest_items!(),
            "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> {\n    let leagues = read_map(map)?;\n    Ok(Reward { gold: pay_guide(open_vault(key)?, leagues)? })\n}\n"
        ),
    ),
    // ── the Act 6-8 trials: realistic alternatives (the same programs
    // pass the server check in `expectations_proof.rs`) ──
    (
        "golem_parts",
        "trial alt: `== false`, renamed closure argument, turbofish collect",
        GOLEM_PARTS_ALT,
    ),
    (
        "golem_sum",
        "trial alt: `.sum::<u32>()` in one expression",
        GOLEM_SUM_TURBOFISH_ALT,
    ),
    (
        "golem_sum",
        "trial alt: `.fold(0, ..)` instead of `sum`",
        GOLEM_SUM_FOLD_ALT,
    ),
    (
        "golem_core",
        "trial alt: map to weakness, sum, then times the blow",
        GOLEM_CORE_SUM_ALT,
    ),
    (
        "golem_core",
        "trial alt: filter then one fold",
        GOLEM_CORE_FOLD_ALT,
    ),
    (
        "hydra_senders",
        "trial alt: `Sender::clone(&tx)`, `while let` over `recv`",
        HYDRA_SENDERS_CLONE_ALT,
    ),
    (
        "hydra_senders",
        "trial alt: `fight` borrows the one sender (rustc's other suggestion)",
        HYDRA_SENDERS_BORROW_ALT,
    ),
    (
        "hydra_threads",
        "trial alt: handles joined in order, `sort_unstable`",
        HYDRA_THREADS_JOIN_ALT,
    ),
    (
        "hydra_threads",
        "trial alt: block-bodied `move` closure, `into_iter`",
        HYDRA_THREADS_BLOCK_ALT,
    ),
    (
        "hydra_body",
        "trial alt: `for` over `rx.iter()`, break when the last head is down",
        HYDRA_BODY_FOR_ALT,
    ),
    (
        "hydra_body",
        "trial alt: `let ... else` on `recv`",
        HYDRA_BODY_LET_ELSE_ALT,
    ),
    (
        "warden_box",
        "trial alt: a `while let` walk instead of recursion",
        WARDEN_BOX_WHILE_ALT,
    ),
    (
        "warden_box",
        "trial alt: `if let` recursion",
        WARDEN_BOX_IF_LET_ALT,
    ),
    (
        "warden_shared",
        "trial alt: a named `borrow_mut` guard, `vault.clone()`",
        WARDEN_SHARED_ALT,
    ),
    (
        "warden_weak",
        "trial alt: stop on the start's name",
        WARDEN_WEAK_NAMES_ALT,
    ),
    (
        "warden_weak",
        "trial alt: `while let` with `Option::filter`",
        WARDEN_WEAK_WHILE_ALT,
    ),
    // ── Act 9: the spec's alternatives and one per Librarian stage (the
    // same programs pass the server check in `expectations_proof.rs`) ──
    (
        "raw_pointer",
        "alt: `&raw mut` / `&raw const`, no intermediate reference",
        a9::ALT_RAW_POINTER_0,
    ),
    (
        "maybe_uninit",
        "alt: an array of `MaybeUninit<u32>`, safe `write`, then `transmute`",
        a9::ALT_MAYBE_UNINIT_0,
    ),
    (
        "librarian_index",
        "trial alt: early `return None`, `if let` in main",
        a9::ALT_LIBRARIAN_INDEX_0,
    ),
    (
        "librarian_demand",
        "trial alt: `@fold` rules with a `;` separator",
        a9::ALT_LIBRARIAN_DEMAND_0,
    ),
    (
        "librarian_seal",
        "trial alt: one `*mut [u32]` field, `Box::from_raw` on it",
        a9::ALT_LIBRARIAN_SEAL_0,
    ),
];

// Alternatives for the Act 6-8 trials, the same text as in
// `compile-api/tests/expectations_proof.rs`.

const GOLEM_PARTS_ALT: &str = "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\nfn weak_points(parts: &[Part]) -> Vec<String> {\n    parts\n        .iter()\n        .filter(|part| part.armoured == false)\n        .map(|part| format!(\"{} ({})\", part.name, part.weakness))\n        .collect::<Vec<_>>()\n}\nfn main() {\n    let parts = golem();\n    let weak = weak_points(&parts);\n    println!(\"weak points: {weak:?}\");\n}\n";

const GOLEM_SUM_TURBOFISH_ALT: &str = "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\nfn core_damage(parts: &[Part]) -> u32 {\n    parts.iter().filter(|p| !p.armoured).map(|p| p.weakness).sum::<u32>() * 2\n}\nfn main() {\n    let parts = golem();\n    let names: Vec<&str> = parts.iter().map(|p| p.name).collect();\n    println!(\"golem: {}\", names.join(\" \"));\n    println!(\"core damage: {}\", core_damage(&parts));\n}\n";

const GOLEM_SUM_FOLD_ALT: &str = "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\nfn core_damage(parts: &[Part]) -> u32 {\n    let total = parts\n        .iter()\n        .filter(|p| !p.armoured)\n        .fold(0, |acc, p| acc + p.weakness);\n    total * 2\n}\nfn main() {\n    let parts = golem();\n    let names: Vec<&str> = parts.iter().map(|p| p.name).collect();\n    println!(\"golem: {}\", names.join(\" \"));\n    println!(\"core damage: {}\", core_damage(&parts));\n}\n";

const GOLEM_CORE_SUM_ALT: &str = "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\nfn total_damage(parts: &[Part], blow: u32) -> u32 {\n    parts.iter().filter(|p| !p.armoured).map(|p| p.weakness).sum::<u32>() * blow\n}\nfn main() {\n    let parts = golem();\n    let names: Vec<&str> = parts.iter().map(|p| p.name).collect();\n    println!(\"golem: {}\", names.join(\" \"));\n    println!(\"blow 1: {}\", total_damage(&parts, 1));\n    println!(\"blow 3: {}\", total_damage(&parts, 3));\n}\n";

const GOLEM_CORE_FOLD_ALT: &str = "struct Part {\n    name: &'static str,\n    armoured: bool,\n    weakness: u32,\n}\nfn golem() -> Vec<Part> {\n    vec![\n        Part { name: \"arm\", armoured: false, weakness: 3 },\n        Part { name: \"chest\", armoured: true, weakness: 8 },\n        Part { name: \"knee\", armoured: false, weakness: 4 },\n        Part { name: \"head\", armoured: true, weakness: 5 },\n        Part { name: \"eye\", armoured: false, weakness: 6 },\n    ]\n}\nfn total_damage(parts: &[Part], blow: u32) -> u32 {\n    parts\n        .iter()\n        .filter(|p| !p.armoured)\n        .fold(0, |acc, p| acc + blow * p.weakness)\n}\nfn main() {\n    let parts = golem();\n    let names = parts.iter().map(|p| p.name).collect::<Vec<_>>();\n    println!(\"golem: {}\", names.join(\" \"));\n    println!(\"blow 1: {}\", total_damage(&parts, 1));\n    println!(\"blow 3: {}\", total_damage(&parts, 3));\n}\n";

const HYDRA_SENDERS_CLONE_ALT: &str = "use std::sync::mpsc::{self, Sender};\nfn fight(head: &'static str, hp: u32, tx: Sender<String>) {\n    let blows = hp.div_ceil(4);\n    tx.send(format!(\"{head} down after {blows} blows\")).unwrap();\n}\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {\n    let (tx, rx) = mpsc::channel();\n    for (head, hp) in heads.iter().copied() {\n        let own = Sender::clone(&tx);\n        fight(head, hp, own);\n    }\n    drop(tx);\n    let mut reports = Vec::new();\n    while let Ok(r) = rx.recv() {\n        reports.push(r);\n    }\n    reports\n}\nfn main() {\n    let reports = hunt(&[(\"fang\", 9), (\"venom\", 6), (\"brine\", 13)]);\n    for report in &reports {\n        println!(\"{report}\");\n    }\n    println!(\"{} heads down\", reports.len());\n}\n";

const HYDRA_SENDERS_BORROW_ALT: &str = "use std::sync::mpsc::{self, Sender};\nfn fight(head: &'static str, hp: u32, tx: &Sender<String>) {\n    let blows = hp.div_ceil(4);\n    tx.send(format!(\"{head} down after {blows} blows\")).unwrap();\n}\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {\n    let (tx, rx) = mpsc::channel();\n    for &(head, hp) in heads {\n        fight(head, hp, &tx);\n    }\n    drop(tx);\n    rx.iter().collect()\n}\nfn main() {\n    let reports = hunt(&[(\"fang\", 9), (\"venom\", 6), (\"brine\", 13)]);\n    for report in &reports {\n        println!(\"{report}\");\n    }\n    println!(\"{} heads down\", reports.len());\n}\n";

const HYDRA_THREADS_JOIN_ALT: &str = "use std::sync::mpsc::{self, Sender};\nuse std::thread;\nfn fight(head: &'static str, hp: u32, tx: Sender<String>) {\n    let blows = hp.div_ceil(4);\n    tx.send(format!(\"{head} down after {blows} blows\")).unwrap();\n}\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {\n    let (tx, rx) = mpsc::channel();\n    let mut workers = Vec::new();\n    for &(head, hp) in heads {\n        let tx = tx.clone();\n        workers.push(thread::spawn(move || fight(head, hp, tx)));\n    }\n    for w in workers {\n        w.join().expect(\"a head's thread panicked\");\n    }\n    drop(tx);\n    let mut reports: Vec<String> = rx.iter().collect();\n    reports.sort_unstable();\n    reports\n}\nfn main() {\n    let reports = hunt(&[(\"fang\", 9), (\"venom\", 6), (\"brine\", 13)]);\n    for report in &reports {\n        println!(\"{report}\");\n    }\n    println!(\"{} heads down\", reports.len());\n}\n";

const HYDRA_THREADS_BLOCK_ALT: &str = "use std::sync::mpsc::{self, Sender};\nuse std::thread;\nfn fight(head: &'static str, hp: u32, tx: Sender<String>) {\n    let blows = hp.div_ceil(4);\n    tx.send(format!(\"{head} down after {blows} blows\")).unwrap();\n}\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> {\n    let (tx, rx) = mpsc::channel();\n    for &(head, hp) in heads {\n        let tx = tx.clone();\n        thread::spawn(move || {\n            fight(head, hp, tx);\n        });\n    }\n    drop(tx);\n    let mut reports = rx.into_iter().collect::<Vec<_>>();\n    reports.sort();\n    reports\n}\nfn main() {\n    let reports = hunt(&[(\"fang\", 9), (\"venom\", 6), (\"brine\", 13)]);\n    for report in &reports {\n        println!(\"{report}\");\n    }\n    println!(\"{} heads down\", reports.len());\n}\n";

const HYDRA_BODY_FOR_ALT: &str = "use std::sync::mpsc::{self, Receiver};\nuse std::thread;\nenum Report {\n    Hit,\n    Down,\n}\nfn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 {\n    let (mut hits, mut down) = (0u32, 0usize);\n    for report in rx.iter() {\n        match report {\n            Report::Hit => hits += 1,\n            Report::Down => {\n                down += 1;\n                if down == heads {\n                    break;\n                }\n            }\n        }\n    }\n    hits\n}\nfn main() {\n    let blows = [3, 2, 4];\n    let (tx, rx) = mpsc::channel();\n    for n in blows {\n        let tx = tx.clone();\n        thread::spawn(move || {\n            for _ in 0..n {\n                tx.send(Report::Hit).unwrap();\n            }\n            tx.send(Report::Down).unwrap();\n        });\n    }\n    drop(tx);\n    let hits = await_heads(&rx, blows.len());\n    println!(\"{} heads down after {hits} hits\", blows.len());\n    println!(\"strike the body\");\n}\n";

const HYDRA_BODY_LET_ELSE_ALT: &str = "use std::sync::mpsc::{self, Receiver};\nuse std::thread;\nenum Report {\n    Hit,\n    Down,\n}\nfn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 {\n    let mut hits = 0;\n    let mut down = 0;\n    while down < heads {\n        let Ok(report) = rx.recv() else { break };\n        if let Report::Hit = report {\n            hits += 1;\n        } else {\n            down += 1;\n        }\n    }\n    hits\n}\nfn main() {\n    let blows = [3, 2, 4];\n    let (tx, rx) = mpsc::channel();\n    for n in blows {\n        let tx = tx.clone();\n        thread::spawn(move || {\n            for _ in 0..n {\n                tx.send(Report::Hit).unwrap();\n            }\n            tx.send(Report::Down).unwrap();\n        });\n    }\n    drop(tx);\n    let hits = await_heads(&rx, blows.len());\n    println!(\"{} heads down after {hits} hits\", blows.len());\n    println!(\"strike the body\");\n}\n";

const WARDEN_BOX_WHILE_ALT: &str = "#[derive(Debug)]\nenum Ledger {\n    Entry(u32, Box<Ledger>),\n    Closed,\n}\nfn entry(amount: u32, rest: Ledger) -> Ledger {\n    Ledger::Entry(amount, Box::new(rest))\n}\nfn total(ledger: &Ledger) -> u32 {\n    let mut sum = 0;\n    let mut at = ledger;\n    while let Ledger::Entry(amount, rest) = at {\n        sum += amount;\n        at = rest.as_ref();\n    }\n    sum\n}\nfn main() {\n    let ledger = entry(5, entry(7, entry(3, Ledger::Closed)));\n    println!(\"{:?}\", ledger);\n    println!(\"owed: {}\", total(&ledger));\n}\n";

const WARDEN_BOX_IF_LET_ALT: &str = "#[derive(Debug)]\nenum Ledger {\n    Entry(u32, Box<Ledger>),\n    Closed,\n}\nfn entry(amount: u32, rest: Ledger) -> Ledger {\n    Ledger::Entry(amount, Box::new(rest))\n}\nfn total(ledger: &Ledger) -> u32 {\n    if let Ledger::Entry(owed, rest) = ledger {\n        owed + total(rest)\n    } else {\n        0\n    }\n}\nfn main() {\n    let ledger = entry(5, entry(7, entry(3, Ledger::Closed)));\n    println!(\"{ledger:?}\");\n    println!(\"owed: {}\", total(&ledger));\n}\n";

const WARDEN_SHARED_ALT: &str = "use std::cell::RefCell;\nuse std::rc::Rc;\nstruct Debtor {\n    name: &'static str,\n    vault: Rc<RefCell<u32>>,\n}\nfn pay(debtor: &Debtor, coins: u32) {\n    let mut held = debtor.vault.borrow_mut();\n    *held += coins;\n}\nfn main() {\n    let vault: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));\n    let miller = Debtor { name: \"miller\", vault: vault.clone() };\n    let smith = Debtor { name: \"smith\", vault: vault.clone() };\n    pay(&miller, 5);\n    pay(&smith, 7);\n    println!(\"{} and {} paid\", miller.name, smith.name);\n    println!(\"vault holds {}\", *vault.borrow());\n    println!(\"handles: {}\", Rc::strong_count(&vault));\n}\n";

const WARDEN_WEAK_NAMES_ALT: &str = "use std::cell::RefCell;\nuse std::rc::{Rc, Weak};\nstruct Debtor {\n    name: &'static str,\n    owes: RefCell<Weak<Debtor>>,\n}\nfn debtor(name: &'static str) -> Rc<Debtor> {\n    Rc::new(Debtor { name, owes: RefCell::new(Weak::new()) })\n}\nfn circle(start: &Rc<Debtor>) -> Vec<&'static str> {\n    let mut names = vec![start.name];\n    let mut next = start.owes.borrow().upgrade();\n    while let Some(d) = next {\n        if d.name == start.name {\n            break;\n        }\n        names.push(d.name);\n        next = d.owes.borrow().upgrade();\n    }\n    names\n}\nfn main() {\n    let warden = debtor(\"warden\");\n    let you = debtor(\"you\");\n    let teller = debtor(\"teller\");\n    *warden.owes.borrow_mut() = Rc::downgrade(&you);\n    *you.owes.borrow_mut() = Rc::downgrade(&teller);\n    *teller.owes.borrow_mut() = Rc::downgrade(&warden);\n    println!(\"circle: {:?}\", circle(&warden));\n    println!(\n        \"warden: strong {}, weak {}\",\n        Rc::strong_count(&warden),\n        Rc::weak_count(&warden)\n    );\n    let probe = Rc::downgrade(&warden);\n    drop(warden);\n    drop(you);\n    drop(teller);\n    println!(\"after the ledger closes: {:?}\", probe.upgrade().map(|d| d.name));\n}\n";

const WARDEN_WEAK_WHILE_ALT: &str = "use std::cell::RefCell;\nuse std::rc::{Rc, Weak};\nstruct Debtor {\n    name: &'static str,\n    owes: RefCell<Weak<Debtor>>,\n}\nfn debtor(name: &'static str) -> Rc<Debtor> {\n    let owes = RefCell::new(Weak::new());\n    Rc::new(Debtor { name, owes })\n}\nfn circle(start: &Rc<Debtor>) -> Vec<&'static str> {\n    let mut names = Vec::new();\n    let mut at = Some(Rc::clone(start));\n    while let Some(d) = at {\n        names.push(d.name);\n        at = d.owes.borrow().upgrade().filter(|n| !Rc::ptr_eq(n, start));\n    }\n    names\n}\nfn main() {\n    let warden = debtor(\"warden\");\n    let you = debtor(\"you\");\n    let teller = debtor(\"teller\");\n    *warden.owes.borrow_mut() = Rc::downgrade(&you);\n    *you.owes.borrow_mut() = Rc::downgrade(&teller);\n    *teller.owes.borrow_mut() = Rc::downgrade(&warden);\n    println!(\"circle: {:?}\", circle(&warden));\n    let strong = Rc::strong_count(&warden);\n    let weak = Rc::weak_count(&warden);\n    println!(\"warden: strong {strong}, weak {weak}\");\n    let probe = Rc::downgrade(&warden);\n    drop((warden, you, teller));\n    let left = probe.upgrade().map(|d| d.name);\n    println!(\"after the ledger closes: {left:?}\");\n}\n";

/// Wrong programs the old substring grader ACCEPTED and a token-level
/// check now catches. `(mission, audit id, source)`.
const WRONG: &[(&str, &str, &str)] = &[
    // ── the Borrow Checker's trial: the shortcuts his tutorial names ──
    (
        "bridge_toll",
        "trial: pays the toll with a clone",
        "fn weigh(cargo: String) -> usize { cargo.len() }\nfn main() { let cargo = String::from(\"lantern oil\"); let toll = weigh(cargo.clone()); println!(\"toll: {toll}\"); println!(\"cargo: {cargo}\"); }\n",
    ),
    (
        "bridge_quill",
        "trial: inscribe takes the scroll by value",
        "fn inscribe(mut scroll: Vec<String>, entry: &str) -> Vec<String> { scroll.push(entry.to_string()); scroll }\nfn main() { let scroll = vec![String::from(\"north road\")]; let scroll = inscribe(scroll, \"east road\"); println!(\"scroll: {}\", scroll.join(\", \")); }\n",
    ),
    (
        "bridge_lantern",
        "trial: ties the answer to both inputs (rustc's suggestion; E0597 in main)",
        "fn before<'a>(line: &'a str, sep: &'a str) -> &'a str { match line.find(sep) { Some(i) => &line[..i], None => line } }\nfn main() { println!(\"{}\", before(\"oil:wick\", \":\")); }\n",
    ),
    // ── the Act 3-5 trials: the shortcuts their hints name ──
    (
        "wraith_essence",
        "trial: rustc's help, fields bound to `_` and ignored",
        "#[derive(Debug, Clone, Copy)]\nenum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow }\nfn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor: _ } => blow, Wraith::Mist { density: _ } => blow, Wraith::Shadow => 0 } }\nfn main() { println!(\"{}\", damage(Wraith::Solid { armor: 3 }, 10)); }\n",
    ),
    (
        "wraith_possessed",
        "trial: a blanket `_ => 0` arm for the new form",
        "#[derive(Debug, Clone, Copy)]\nenum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow, Possessed { will: u32 } }\nfn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor } => blow - armor, Wraith::Mist { density } => blow / density, Wraith::Shadow => 0, _ => 0 } }\nfn main() { println!(\"{}\", damage(Wraith::Possessed { will: 4 }, 10)); }\n",
    ),
    (
        "chimera_heads",
        "trial: plain `impl Frost` blocks, no trait",
        "trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire { fn name(&self) -> &'static str { \"fire\" } fn breath(&self) -> u32 { 7 } }\nimpl Frost { fn name(&self) -> &'static str { \"frost\" } fn breath(&self) -> u32 { 5 } }\nimpl Storm { fn name(&self) -> &'static str { \"storm\" } fn breath(&self) -> u32 { 9 } }\nfn main() { println!(\"{} {} {}\", Fire.breath(), Frost.breath(), Storm.breath()); let _ = (Fire.name(), Frost.name(), Storm.name()); }\n",
    ),
    (
        "chimera_bound",
        "trial: a trait object instead of a bounded generic",
        "trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }\nstruct Fire;\nimpl Enemy for Fire { fn name(&self) -> &'static str { \"fire\" } fn breath(&self) -> u32 { 7 } }\nfn volley(head: &dyn Enemy, rounds: u32) -> String { format!(\"{} x{rounds} = {}\", head.name(), head.breath() * rounds) }\nfn main() { println!(\"{}\", volley(&Fire, 3)); }\n",
    ),
    (
        "wager_quest",
        "trial: the nested `match` the Tavernkeeper grades against",
        concat!(
            quest_items!(),
            "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { match read_map(map) { Ok(leagues) => match open_vault(key) { Ok(gold) => match pay_guide(gold, leagues) { Ok(left) => Ok(Reward { gold: left }), Err(e) => Err(e.into()) }, Err(e) => Err(e.into()) }, Err(e) => Err(e.into()) } }\n"
        ),
    ),
    // ── the Act 6-8 trials: the shortcuts their hints name ──
    (
        "golem_parts",
        "trial: map first, then drop the chest and the head by name",
        "struct Part { name: &'static str, armoured: bool, weakness: u32 }\nfn weak_points(parts: &[Part]) -> Vec<String> { parts.iter().map(|p| format!(\"{} ({})\", p.name, p.weakness)).filter(|l| !l.starts_with(\"chest\") && !l.starts_with(\"head\")).collect() }\nfn main() { let parts = vec![Part { name: \"arm\", armoured: false, weakness: 3 }]; println!(\"{:?}\", weak_points(&parts)); }\n",
    ),
    (
        "golem_core",
        "trial: `take_while` stops at the first armoured part",
        "struct Part { armoured: bool, weakness: u32 }\nfn total_damage(parts: &[Part], blow: u32) -> u32 { parts.iter().take_while(|p| !p.armoured).map(|p| blow * p.weakness).sum() }\nfn main() { let parts = vec![Part { armoured: false, weakness: 3 }]; println!(\"{}\", total_damage(&parts, 2)); }\n",
    ),
    (
        "hydra_threads",
        "trial: threads, but the reports are never sorted",
        "use std::sync::mpsc;\nuse std::thread;\nfn main() { let (tx, rx) = mpsc::channel(); for head in [\"fang\", \"venom\"] { let tx = tx.clone(); thread::spawn(move || tx.send(head).unwrap()); } drop(tx); let reports: Vec<&str> = rx.iter().collect(); println!(\"{reports:?}\"); }\n",
    ),
    (
        "if_else_sign",
        "G2 neg/pos labels swapped",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 { \"pos\" } else if n == 0 { \"zero\" } else { \"neg\" }\n}\nfn main() {}\n",
    ),
    (
        "if_else_sign",
        "G2 zero/pos labels swapped",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 { \"neg\" } else if n == 0 { \"pos\" } else { \"zero\" }\n}\nfn main() {}\n",
    ),
    (
        "match_option",
        "G2 0 in both arms",
        "fn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        Some(_) => 0,\n        None => 0,\n    }\n}\nfn main() {}\n",
    ),
    (
        "loop_break",
        "G2 break with no value",
        "fn main() {\n    let mut n = 1;\n    loop {\n        if n >= 100 {\n            break;\n        }\n        n *= 2;\n    }\n}\n",
    ),
    (
        "tuple_struct",
        "G8 3.0 literal, field never read",
        "struct Meters(f64);\nfn main() {\n    let _d = Meters(3.0);\n}\n",
    ),
    (
        "trait_def",
        "G9 inherent impl + unused trait + for loop",
        "trait Element {\n    fn name(&self) -> &str;\n}\nstruct Fire;\nimpl Fire {\n    fn name(&self) -> &str {\n        \"fire\"\n    }\n}\nfn main() {\n    for _ in 0..1 {\n        println!(\"{}\", Fire.name());\n    }\n}\n",
    ),
    (
        "trait_def",
        "G9 std trait implemented, own trait unused",
        "trait Element {}\nstruct Fire;\nimpl Clone for Fire {\n    fn clone(&self) -> Self {\n        Fire\n    }\n}\nfn main() {}\n",
    ),
    (
        "iter_fold",
        "fold ignoring its elements",
        "fn main() {\n    let v = vec![1, 2, 3];\n    let total = v.iter().fold(0, |acc, _| acc);\n    println!(\"{total}\");\n}\n",
    ),
    (
        "iter_fold",
        "named fn ignoring its element",
        "fn keep(acc: i32, _x: &i32) -> i32 {\n    acc\n}\nfn main() {\n    let v = vec![1, 2, 3];\n    println!(\"{}\", v.iter().fold(0, keep));\n}\n",
    ),
    (
        "if_else_sign",
        "G0b no fn main (rustc E0601)",
        "fn sign(n: i32) -> &'static str {\n    if n < 0 { \"neg\" } else if n == 0 { \"zero\" } else { \"pos\" }\n}\n",
    ),
    (
        "match_option",
        "G0b main only inside a module",
        "mod game {\n    pub fn main() {}\n}\nfn unwrap_or_zero(x: Option<i32>) -> i32 {\n    match x {\n        Some(n) => n,\n        None => 0,\n    }\n}\n",
    ),
    (
        "impl_method",
        "G0b main only as a method",
        "struct Hero { level: i32 }\nimpl Hero {\n    fn power(&self) -> i32 { self.level * 10 }\n    fn main() {}\n}\n",
    ),
    (
        "intro_let_binding",
        "G0b async fn main (rustc E0752)",
        "async fn main() {\n    let answer = 42;\n}\n",
    ),
    // ── Act 9: right output without the lesson; the tokens catch them ──
    (
        "extern_c",
        "Act 9: calls `crown_seal` directly, never through the extern block",
        a9::LITERAL_EXTERN_C,
    ),
    (
        "send_sync",
        "Act 9: a `Box` field is Send and Sync already, no promise made",
        a9::LITERAL_SEND_SYNC,
    ),
    (
        "librarian_index",
        "trial: safe `get` instead of the `unsafe fn`",
        a9::LITERAL_LIBRARIAN_INDEX,
    ),
    (
        "librarian_seal",
        "trial: prints the seal line, never calls `from_raw`",
        a9::LITERAL_LIBRARIAN_SEAL,
    ),
];

#[test]
fn grader_corpus() {
    let reg = MissionRegistry::default();
    let n = reg.missions.len();
    let (mut a, mut b, mut c, mut f) = (0, 0, 0, 0);
    let mut starters_with_main = 0;
    let mut failures = Vec::new();
    for m in &reg.missions {
        let player = player_solution(m.id).unwrap_or_else(|| panic!("no solution for `{}`", m.id));
        if graded_ok(m.id, player) {
            a += 1;
        } else {
            failures.push(format!("(a) player solution rejected: {}", m.id));
        }
        let fixture = canonical_solution(m.id).expect("checked elsewhere");
        if graded_ok(m.id, fixture) {
            b += 1;
        } else {
            failures.push(format!("(b) contract fixture rejected: {}", m.id));
        }
        assert!(
            !player.contains("*/") && !player.contains("\"##"),
            "wrapper would break"
        );
        let block = format!("fn main() {{}}\n/*\n{player}\n*/\n");
        let lines: String = player.lines().map(|l| format!("// {l}\n")).collect();
        let lines = format!("fn main() {{}}\n{lines}");
        let string = format!("fn main() {{\n    let _s = r##\"{player}\"##;\n}}\n");
        let hidden: Vec<bool> = [&block, &lines, &string]
            .iter()
            .map(|src| graded_ok(m.id, src))
            .collect();
        if hidden.iter().all(|ok| !ok) {
            c += 1;
        } else {
            failures.push(format!(
                "(c) hidden solution passed (block, lines, string) = {hidden:?}: {}",
                m.id
            ));
        }
        if !graded_ok(m.id, m.starter_code) {
            f += 1;
        } else {
            failures.push(format!("(f) starter passes: {}", m.id));
        }
        // A starter's first message must teach its concept, never the
        // shared `fn main` rule (which is checked last).
        let starter_msg = grade(m.id, m.starter_code).stderr;
        if starter_msg.contains(MAIN_HINT) {
            failures.push(format!(
                "(f) starter's first failure is the fn main hint, not its concept: {}",
                m.id
            ));
        }
        if has_entry_point(&lex(m.starter_code)) {
            starters_with_main += 1;
        }
    }
    let mut d = 0;
    for (id, why, src) in CORRECT {
        if graded_ok(id, src) {
            d += 1;
        } else {
            failures.push(format!("(d) correct program rejected: {id} [{why}]"));
        }
    }
    let mut e = 0;
    for (id, why, src) in WRONG {
        if !graded_ok(id, src) {
            e += 1;
        } else {
            failures.push(format!("(e) wrong program accepted: {id} [{why}]"));
        }
    }
    println!(
        "grader corpus: (a) {a}/{n} player solutions pass; (b) {b}/{n} fixtures pass; \
         (c) {c}/{n} hidden solutions fail; (d) {d}/{} correct programs pass; \
         (e) {e}/{} wrong programs fail; (f) {f}/{n} starters fail \
         ({starters_with_main} of them have `fn main`, all fail on a concept hint)",
        CORRECT.len(),
        WRONG.len()
    );
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Slow audit: every `CORRECT` corpus program really is valid Rust at the
/// server's edition.
/// Run with `cargo test -p pledge_and_crown --test contract -- --ignored`.
#[test]
#[ignore = "slow: runs rustc once per corpus program"]
fn corpus_correct_programs_compile() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("grader-corpus-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    let mut failures = Vec::new();
    // The canonical fixtures too, as binaries: a fixture with no `fn main`
    // (audit G0b) fails here with E0601.
    let reg = MissionRegistry::default();
    let fixtures = reg.missions.iter().map(|m| {
        let src = canonical_solution(m.id).expect("checked elsewhere");
        (m.id, "canonical fixture", src)
    });
    let programs: Vec<(&str, &str, &str)> = CORRECT.iter().copied().chain(fixtures).collect();
    for (n, (id, why, src)) in programs.iter().enumerate() {
        let path = dir.join(format!("c{n}.rs"));
        std::fs::write(&path, src).expect("write corpus source");
        let out = std::process::Command::new("rustc")
            .args(["--edition", PLAYER_EDITION, "--crate-type", "bin", "-o"])
            .arg(dir.join(format!("c{n}.out")))
            .arg(&path)
            .output()
            .expect("spawn rustc");
        if !out.status.success() {
            failures.push(format!(
                "{id} [{why}]: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!(
        "corpus compile ({PLAYER_EDITION}): {} of {} programs built",
        programs.len() - failures.len(),
        programs.len()
    );
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn no_mission_falls_through_to_freeform() {
    // The freeform fallthrough path always returns ok=true with a
    // stdout containing the literal "freeform". A real mission's
    // grader path must reject garbage with ok=false (and the freeform
    // marker must NOT appear in either branch).
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        let v = grade(m.id, "fn main() { /* total nonsense */ }");
        let mentions_freeform = v.stdout.contains("freeform") || v.stderr.contains("freeform");
        assert!(
            !mentions_freeform,
            "mission `{}` is hitting the freeform fallthrough — add a grader arm in compile-api/src/grader.rs",
            m.id
        );
    }
}
