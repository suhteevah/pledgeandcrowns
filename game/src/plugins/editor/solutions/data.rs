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
    let v = vec![1, 2, 3];
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
    println!("{}", add(2, 3));
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
    South,
    East,
    West,
}

fn heading(d: Direction) -> &'static str {
    // One arm per variant. Add a variant later and this match stops compiling
    // until you handle it.
    match d {
        Direction::North => "up",
        Direction::South => "down",
        Direction::East => "right",
        Direction::West => "left",
    }
}

fn main() {
    for d in [Direction::North, Direction::South, Direction::East, Direction::West] {
        println!("{}", heading(d));
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
    // 8 -> 4 -> 2 -> 1, then 1 is odd: the chain stops with `None`.
    let gone = Some(8)
        .and_then(half)
        .and_then(half)
        .and_then(half)
        .and_then(half);
    println!("{two:?} {gone:?}");
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
    // ── Act 6: The Iterator Forge ────────────────────────────────────
    (
        "hashmap_basic",
        r#"use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();
    scores.insert("ace", 1);
    // `get` returns an Option: the key might not be there.
    match scores.get("ace") {
        Some(score) => println!("ace: {score}"),
        None => println!("no ace on file"),
    }
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
        *guard += 1;
    } // the guard is dropped here, which releases the lock
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
    let got = rx.recv().expect("all senders dropped");
    println!("{got}");
}
"#,
    ),
    (
        "atomic",
        r#"use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    let counter = AtomicUsize::new(0);
    // One indivisible read-add-write. No lock, no guard.
    counter.fetch_add(1, Ordering::SeqCst);
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
    // `Rc::clone` bumps the count. Both handles point at the same String.
    let shared = Rc::clone(&original);
    println!("{shared} has {} owners", Rc::strong_count(&original));
}
"#,
    ),
    (
        "refcell",
        r#"use std::cell::RefCell;

fn main() {
    let guarded = RefCell::new(0);
    let shared_ref = &guarded; // only a shared `&`...
    // ...yet we can mutate. The borrow rules are checked at runtime instead.
    *shared_ref.borrow_mut() += 1;
    println!("{}", guarded.borrow());
}
"#,
    ),
    (
        "cell",
        r#"use std::cell::Cell;

fn main() {
    let slot = Cell::new(1);
    let shared_ref = &slot;
    // `set` swaps in a whole new value through a shared `&`. No borrow guard.
    shared_ref.set(5);
    println!("{}", slot.get());
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
    *clone.borrow_mut() += 1;
    // The change is visible through every handle.
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
    match weak.upgrade() {
        Some(rc) => println!("still here: {rc}"),
        None => println!("gone"),
    }
    drop(strong); // the last strong owner is gone, so the value is freed
    println!("after drop: {:?}", weak.upgrade());
}
"#,
    ),
];
