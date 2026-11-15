// SPDX-License-Identifier: AGPL-3.0-or-later
//! Proof that the `/compile-real` expectations discriminate.
//!
//! For every mission with a behaviour check (`Stdout` / `Harness`):
//! the canonical solution passes, the mission's starter code fails, and
//! every hand-written wrong-but-compiling variant below fails. For
//! `CompileOnly` missions the same holds for the static checks. For
//! `RunOnly` missions (none today) the canonical solution must report
//! "ran, not checked". For every mission with `uses` tokens, a program
//! that prints the pinned lines as literals must be `StaticCheckFailed`.
//!
//! The canonical solutions and starters are MIT game code. This crate is
//! AGPL and must not depend on the game crate, so this dev-only test reads
//! them as TEXT from the game sources; nothing here ships.
//!
//! All tests are `#[ignore]`d: each case is a real `cargo build` for
//! `wasm32-wasip1` plus a sandboxed run.
//!
//! ```text
//! cargo test -p pledge_compile_api --test expectations_proof -- --ignored --nocapture
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use pledge_compile_api::expectations::{self, Check, GradeState};
use pledge_compile_api::wasm_builder::{self, BuildSettings, Layout, RunLimits};

/// Parallel build workers: `CARGO_BUILD_JOBS` when set (so a run on a
/// shared machine can be kept light), else 6.
fn workers() -> usize {
    std::env::var("CARGO_BUILD_JOBS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(6)
}

fn game_file(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("game")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// `(id, source)` from the game's `solutions/data.rs` raw-string table.
fn canonical_solutions() -> HashMap<String, String> {
    let text = game_file("src/plugins/editor/solutions/data.rs");
    let mut out = HashMap::new();
    let mut rest = text.as_str();
    while let Some(open) = rest.find("r#\"") {
        let before = &rest[..open];
        let id_end = before.rfind('"').expect("id close quote");
        let id_start = before[..id_end].rfind('"').expect("id open quote") + 1;
        let id = before[id_start..id_end].to_string();
        let body_start = open + 3;
        let body_len = rest[body_start..].find("\"#").expect("raw string end");
        out.insert(id, rest[body_start..body_start + body_len].to_string());
        rest = &rest[body_start + body_len + 2..];
    }
    out
}

/// Unescape a normal Rust string literal body (the subset mission.rs uses).
fn unescape(lit: &str) -> String {
    let mut out = String::new();
    let mut chars = lit.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            Some('\n') | Some('\r') => {
                while chars.peek().is_some_and(|c| c.is_whitespace()) {
                    chars.next();
                }
            }
            Some(other) => panic!("unhandled escape \\{other}"),
            None => {}
        }
    }
    out
}

/// `(id, starter_code)` from the game's mission registry.
fn starters() -> HashMap<String, String> {
    let text = game_file("src/plugins/mission.rs");
    let mut out = HashMap::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("            id: \"") {
        let after = &rest[at + 17..];
        let id_end = after.find('"').expect("id end");
        let id = after[..id_end].to_string();
        let s_at = after.find("starter_code: \"").expect("starter") + 15;
        let body = &after[s_at..];
        let mut end = 0;
        let bytes = body.as_bytes();
        while end < bytes.len() {
            if bytes[end] == b'\\' {
                end += 2;
                continue;
            }
            if bytes[end] == b'"' {
                break;
            }
            end += 1;
        }
        out.insert(id, unescape(&body[..end]));
        rest = &after[s_at + end..];
    }
    out
}

/// Wrong-but-compiling variants per mission. Each builds as a standalone
/// program, and each must be rejected.
fn wrong_variants() -> HashMap<&'static str, Vec<&'static str>> {
    HashMap::from([
        (
            "intro_let_binding",
            vec!["fn main() { let answer = 41; println!(\"{answer}\"); }"],
        ),
        (
            "double_function",
            vec![
                "fn double(n: i32) -> i32 { n + 2 } fn main() { println!(\"{}\", double(21)); }",
                "fn double(n: i32) -> i32 { 42 } fn main() { println!(\"{}\", double(21)); }",
            ],
        ),
        (
            "borrow_preview",
            vec![
                "fn main() { let value = String::from(\"sample\"); let r = &value; println!(\"r\"); }",
            ],
        ),
        (
            "mut_binding",
            vec!["fn main() { let mut x = 0; x += 2; println!(\"{x}\"); }"],
        ),
        (
            "if_else_sign",
            vec![
                "fn sign(n: i32) -> &'static str { if n < 0 { \"pos\" } else if n == 0 { \"zero\" } else { \"neg\" } }\nfn main() { println!(\"{}\", sign(-3)); }",
                "fn sign(n: i32) -> &'static str { if n <= 0 { \"neg\" } else { \"pos\" } }\nfn main() { println!(\"{}\", sign(-3)); }",
            ],
        ),
        (
            "loop_break",
            vec![
                "fn main() { let mut n = 1; let r = loop { if n > 100 { break n; } n *= 3; }; println!(\"{r}\"); }",
            ],
        ),
        (
            "match_option",
            vec![
                "fn unwrap_or_zero(x: Option<i32>) -> i32 { match x { Some(_) => 0, None => 0 } }\nfn main() { println!(\"{}\", unwrap_or_zero(Some(7))); }",
            ],
        ),
        (
            "struct_basic",
            vec![
                "struct Knight { name: String, hp: u32 }\nfn main() { let k = Knight { name: String::from(\"G\"), hp: 3 }; println!(\"{}\", k.name); }",
            ],
        ),
        (
            "tuple_destructure",
            vec!["fn main() { let (a, b) = (3, 4); println!(\"{a}\"); }"],
        ),
        (
            "borrow_mut",
            vec![
                "fn bump(x: &mut i32) { let _ = x; }\nfn main() { let mut n = 0; bump(&mut n); println!(\"{n}\"); }",
            ],
        ),
        (
            "string_vs_str",
            vec![
                "fn greet(name: String) { println!(\"hi {name}\"); }\nfn main() { greet(String::from(\"a\")); }",
            ],
        ),
        (
            "option_unwrap_or",
            vec![
                "fn safe(x: Option<i32>) -> i32 { x.unwrap_or(1) }\nfn main() { println!(\"{}\", safe(None)); }",
            ],
        ),
        (
            "for_in_range",
            vec!["fn main() { for i in 1..10 { println!(\"{i}\"); } }"],
        ),
        (
            "slice_basic",
            vec![
                "fn sum_slice(xs: &[i32]) -> i32 { xs.len() as i32 }\nfn main() { println!(\"{}\", sum_slice(&[1, 2, 3])); }",
            ],
        ),
        (
            "result_question_mark",
            vec![
                "fn parse_int(s: &str) -> Result<i32, String> { Ok(s.parse::<i32>().unwrap_or(0)) }\nfn main() { println!(\"{:?}\", parse_int(\"42\")); }",
            ],
        ),
        (
            "derive_debug",
            vec![
                "#[derive(Debug)]\nstruct Item { name: String }\nfn main() { let item = Item { name: String::from(\"ring\") }; println!(\"{}\", item.name); }",
                "struct Item { name: String }\nimpl std::fmt::Display for Item { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, \"Item {{ {} }}\", self.name) } }\nfn main() { let item = Item { name: String::from(\"ring\") }; println!(\"{item}\"); }",
            ],
        ),
        (
            "iter_map_collect",
            vec![
                "fn main() { let v = vec![1, 2, 3]; let d: Vec<i32> = v.iter().map(|x| x + 2).collect(); println!(\"{d:?}\"); }",
            ],
        ),
        (
            "while_loop",
            vec![
                "fn main() { let mut n = 5; while n > 1 { println!(\"{n}\"); n -= 1; } println!(\"{n}\"); }",
            ],
        ),
        // The Borrow Checker's trial. Every variant compiles on its own.
        (
            "bridge_toll",
            vec![
                // Pays the toll with a clone: `weigh` still takes `String`.
                "fn weigh(cargo: String) -> usize { cargo.len() }\nfn main() { let cargo = String::from(\"lantern oil\"); let toll = weigh(cargo.clone()); println!(\"toll: {toll}\"); println!(\"cargo: {cargo}\"); }",
                // Borrows, but weighs nothing: always 11.
                "fn weigh(_cargo: &str) -> usize { 11 }\nfn main() { let cargo = String::from(\"lantern oil\"); let toll = weigh(&cargo); println!(\"toll: {toll}\"); println!(\"cargo: {cargo}\"); }",
                // Never shows the cargo was kept.
                "fn weigh(cargo: &str) -> usize { cargo.len() }\nfn main() { let cargo = String::from(\"lantern oil\"); let toll = weigh(&cargo); println!(\"toll: {toll}\"); }",
            ],
        ),
        (
            "bridge_quill",
            vec![
                // Writes at the front of the scroll.
                "fn inscribe(scroll: &mut Vec<String>, entry: &str) { scroll.insert(0, entry.to_string()); }\nfn main() { let mut scroll = vec![String::from(\"north road\")]; println!(\"reader one: {}\", scroll[0]); println!(\"reader two: {}\", scroll[0]); inscribe(&mut scroll, \"east road\"); println!(\"scroll: {}\", scroll.join(\", \")); }",
                // Writes the entry twice.
                "fn inscribe(scroll: &mut Vec<String>, entry: &str) { scroll.push(entry.to_string()); scroll.push(entry.to_string()); }\nfn main() { let mut scroll = vec![String::from(\"north road\")]; println!(\"reader one: {}\", scroll[0]); println!(\"reader two: {}\", scroll[0]); inscribe(&mut scroll, \"east road\"); println!(\"scroll: {}\", scroll.join(\", \")); }",
                // Writes first, reads after: the lines come out in the wrong order.
                "fn inscribe(scroll: &mut Vec<String>, entry: &str) { scroll.push(entry.to_string()); }\nfn main() { let mut scroll = vec![String::from(\"north road\")]; inscribe(&mut scroll, \"east road\"); println!(\"scroll: {}\", scroll.join(\", \")); println!(\"reader one: {}\", scroll[0]); println!(\"reader two: {}\", scroll[0]); }",
            ],
        ),
        (
            "bridge_lantern",
            vec![
                // Returns the part AFTER the separator.
                "fn before<'a>(line: &'a str, sep: &str) -> &'a str { match line.find(sep) { Some(i) => &line[i + sep.len()..], None => line } }\nfn main() { let lantern = String::from(\"oil:wick:glass\"); let part; { let sep = String::from(\":\"); part = before(&lantern, &sep); } println!(\"first: {part}\"); println!(\"whole: {}\", before(\"glass\", \":\")); }",
                // Ignores the separator.
                "fn before<'a>(line: &'a str, _sep: &str) -> &'a str { line }\nfn main() { let lantern = String::from(\"oil:wick:glass\"); let part; { let sep = String::from(\":\"); part = before(&lantern, &sep); } println!(\"first: {part}\"); println!(\"whole: {}\", before(\"glass\", \":\")); }",
                // Dodges the lifetime with an owned String: right output, wrong signature.
                "fn before(line: &str, sep: &str) -> String { line.split(sep).next().unwrap_or(line).to_string() }\nfn main() { let lantern = String::from(\"oil:wick:glass\"); let part; { let sep = String::from(\":\"); part = before(&lantern, &sep); } println!(\"first: {part}\"); println!(\"whole: {}\", before(\"glass\", \":\")); }",
            ],
        ),
        (
            "impl_method",
            vec![
                "struct Hero { level: i32 }\nimpl Hero { fn power(self) -> i32 { self.level * 10 } }\nfn main() { let h = Hero { level: 3 }; println!(\"{}\", h.power()); }",
            ],
        ),
        (
            "assoc_new",
            vec![
                "struct Hero { level: i32 }\nimpl Hero { fn new(level: i32) -> Self { Self { level: 1 } } }\nfn main() { let h = Hero::new(5); println!(\"{}\", h.level); }",
            ],
        ),
        (
            "if_let",
            vec![
                "fn main() { let maybe: Option<i32> = Some(7); if let None = maybe { println!(\"7\"); } else { println!(\"nothing\"); } }",
            ],
        ),
        (
            "while_let",
            vec![
                "fn main() { let mut stack = vec![1, 2, 3]; while let Some(top) = stack.pop() { if top == 2 { break; } println!(\"{top}\"); } }",
            ],
        ),
        (
            "tuple_struct",
            vec!["struct Meters(f64); fn main() { let _d = Meters(3.0); }"],
        ),
        (
            "enum_data_match",
            vec![
                "enum Item { Weapon { damage: i32 }, Potion { heal: i32 } }\nfn value(item: Item) -> i32 { match item { Item::Weapon { .. } => 0, Item::Potion { .. } => 0 } }\nfn main() { println!(\"{}\", value(Item::Potion { heal: 5 })); }",
            ],
        ),
        (
            "trait_def",
            vec![
                "trait Element { fn name(&self) -> &str; }\nstruct Fire;\nimpl Fire { fn name(&self) -> &str { \"fire\" } }\nfn main() { let f = Fire; for _ in 0..1 { println!(\"{}\", f.name()); } }",
            ],
        ),
        (
            "generic_fn",
            vec![
                "fn larger<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }\nfn main() { println!(\"{}\", larger(3, 7)); }",
            ],
        ),
        (
            "generic_struct",
            vec![
                "struct Pair { a: i32, b: i32 }\nfn main() { let p = Pair { a: 1, b: 2 }; println!(\"{} {}\", p.a, p.b); }",
            ],
        ),
        (
            "dyn_trait",
            vec![
                "trait Element { fn name(&self) -> &str; }\nstruct Fire;\nimpl Element for Fire { fn name(&self) -> &str { \"fire\" } }\nstruct Water;\nimpl Element for Water { fn name(&self) -> &str { \"water\" } }\nfn main() { let e: Vec<Box<dyn Element>> = vec![Box::new(Fire)]; for x in &e { println!(\"{}\", x.name()); } let _ = Water; }",
            ],
        ),
        (
            "lifetimes",
            vec![
                "fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { y }\nfn main() { println!(\"{}\", longest(\"aa\", \"b\")); }",
            ],
        ),
        (
            "assoc_type",
            vec![
                "trait Producer { fn make(&self) -> i32; }\nstruct Coiner;\nimpl Producer for Coiner { fn make(&self) -> i32 { 7 } }\nfn main() { println!(\"{}\", Coiner.make()); }",
            ],
        ),
        (
            "result_match",
            vec![
                "fn describe(r: Result<i32, String>) -> i32 { match r { Ok(v) => v, Err(_) => 0 } }\nfn main() { println!(\"{}\", describe(Ok(5))); }",
            ],
        ),
        (
            "custom_error",
            vec![
                "#[derive(Debug)]\nenum BrewError { TooHot, TooCold }\nfn check(temp: i32) -> Result<i32, BrewError> { Ok(temp) }\nfn main() { println!(\"{:?}\", check(50)); }",
            ],
        ),
        (
            "from_error",
            vec![
                "struct ParseFail;\nenum AppError { Parse, Other }\nimpl From<ParseFail> for AppError { fn from(_: ParseFail) -> Self { AppError::Other } }\nfn main() { let _e: AppError = ParseFail.into(); }",
            ],
        ),
        (
            "option_map",
            vec![
                "fn add_one(o: Option<i32>) -> Option<i32> { o.map(|x| x) }\nfn main() { println!(\"{:?}\", add_one(Some(3))); }",
            ],
        ),
        (
            "unwrap_or_else",
            vec![
                "fn value(o: Option<i32>) -> i32 { o.map(|x| x + 1).unwrap_or_else(|| 0) }\nfn main() { println!(\"{}\", value(Some(3))); }",
            ],
        ),
        (
            "iter_filter",
            vec![
                "fn main() { let v = vec![1, 2, 3, 4]; let odds: Vec<&i32> = v.iter().filter(|x| **x % 2 == 1).collect(); println!(\"{odds:?}\"); }",
            ],
        ),
        (
            "iter_fold",
            vec![
                "fn main() { let v = vec![1, 2, 3]; let total = v.iter().fold(0, |acc, _x| acc); println!(\"{total}\"); }",
            ],
        ),
        (
            "iter_enumerate",
            vec![
                "fn main() { let v = vec![10, 20, 30]; for (i, x) in v.iter().enumerate() { println!(\"{}: {x}\", i + 1); } }",
            ],
        ),
        (
            "iter_zip",
            vec![
                "fn main() { let a = vec![1, 2]; let b = vec![3, 4]; for (x, y) in a.iter().zip(a.iter()) { println!(\"{x},{y}\"); } let _ = b; }",
            ],
        ),
        (
            "closure_move",
            vec![
                "fn main() { let name = String::from(\"Garin\"); let greet = move || println!(\"hail, {}\", name.len()); greet(); }",
            ],
        ),
        (
            "thread_spawn",
            vec!["use std::thread;\nfn main() { let _h = thread::spawn(|| 21 * 2); }"],
        ),
        (
            "thread_scope",
            vec![
                "fn main() { let data = vec![1, 2, 3]; let h = std::thread::spawn(move || data.len()); let _ = h.join(); }",
            ],
        ),
        (
            "async_fn",
            vec![
                "fn double(x: i32) -> i32 { x * 2 }\n// async fn double(x) .await\nfn main() { let _ = double(2); }",
            ],
        ),
        (
            "box_basic",
            vec![
                "struct Node { value: i32, next: Option<i32> }\nfn main() { let n = Node { value: 1, next: Some(2) }; println!(\"{}\", n.value); }",
            ],
        ),
        // ── Formerly run-only: the mission text now pins the output.
        (
            "vec_iter",
            vec![
                // Left an element out.
                "fn main() { let v = vec![10, 20]; let total: i32 = v.iter().sum(); println!(\"{total}\"); }",
                // Counted the elements instead of adding them.
                "fn main() { let v = vec![10, 20, 30]; let total = v.iter().count(); println!(\"{total}\"); }",
            ],
        ),
        (
            "closure_basic",
            vec![
                "fn main() { let add = |a: i32, b: i32| a - b; println!(\"{}\", add(19, 23)); }",
                "fn main() { let add = |a: i32, _b: i32| a + a; println!(\"{}\", add(19, 23)); }",
            ],
        ),
        (
            "enum_match",
            vec![
                // East and West swapped.
                "enum Direction { North, East, South, West }\nfn degrees(d: Direction) -> i32 { match d { Direction::North => 0, Direction::East => 270, Direction::South => 180, Direction::West => 90 } }\nfn main() { for d in [Direction::North, Direction::East, Direction::South, Direction::West] { println!(\"{}\", degrees(d)); } }",
                // A catch-all arm swallows two variants; main still prints the right lines.
                "enum Direction { North, East, South, West }\nfn degrees(d: Direction) -> i32 { match d { Direction::North => 0, _ => 90 } }\nfn main() { let _ = (Direction::South, Direction::West); for line in [0, 90, 180, 270] { println!(\"{line}\"); } let _ = degrees(Direction::East); }",
            ],
        ),
        (
            "and_then",
            vec![
                // Three steps instead of four: 8 -> 4 -> 2 -> 1 is still Some.
                "fn half(n: i32) -> Option<i32> { if n % 2 == 0 { Some(n / 2) } else { None } }\nfn main() { println!(\"{:?}\", Some(8).and_then(half).and_then(half)); println!(\"{:?}\", Some(8).and_then(half).and_then(half).and_then(half)); }",
            ],
        ),
        (
            "hashmap_basic",
            vec![
                // Unwrapped instead of printing the Option.
                "use std::collections::HashMap;\nfn main() { let mut m = HashMap::new(); m.insert(\"vault\", 7); println!(\"{}\", m.get(\"vault\").copied().unwrap_or(0)); println!(\"{:?}\", m.get(\"cellar\")); }",
                // Looked up the wrong key twice.
                "use std::collections::HashMap;\nfn main() { let mut m = HashMap::new(); m.insert(\"vault\", 7); println!(\"{:?}\", m.get(\"cellar\")); println!(\"{:?}\", m.get(\"cellar\")); }",
            ],
        ),
        (
            "arc_mutex",
            vec![
                // Copied the number out of the guard and changed the copy.
                "use std::sync::{Arc, Mutex};\nfn main() { let shared = Arc::new(Mutex::new(0)); let other = Arc::clone(&shared); let mut n = *other.lock().expect(\"lock\"); n += 5; let _ = n; println!(\"{}\", shared.lock().expect(\"lock\")); }",
                // A second Arc around a copy, not a second handle to the same one.
                "use std::sync::{Arc, Mutex};\nfn main() { let shared = Arc::new(Mutex::new(0)); let other = Arc::new(Mutex::new(*shared.lock().expect(\"lock\"))); *other.lock().expect(\"lock\") += 5; println!(\"{}\", shared.lock().expect(\"lock\")); }",
            ],
        ),
        (
            "mpsc_channel",
            vec![
                // Received only once.
                "use std::sync::mpsc;\nfn main() { let (tx, rx) = mpsc::channel(); tx.send(7).expect(\"send\"); tx.send(11).expect(\"send\"); println!(\"{}\", rx.recv().expect(\"recv\")); }",
            ],
        ),
        (
            "atomic",
            vec![
                // Printed the new value twice.
                "use std::sync::atomic::{AtomicUsize, Ordering};\nfn main() { let c = AtomicUsize::new(10); c.fetch_add(1, Ordering::SeqCst); println!(\"{}\", c.load(Ordering::SeqCst)); println!(\"{}\", c.load(Ordering::SeqCst)); }",
                // Assumed fetch_add returns the NEW value.
                "use std::sync::atomic::{AtomicUsize, Ordering};\nfn main() { let c = AtomicUsize::new(10); let now = c.fetch_add(1, Ordering::SeqCst) + 1; println!(\"{now}\"); println!(\"{}\", c.load(Ordering::SeqCst)); }",
            ],
        ),
        (
            "rc_basic",
            vec![
                // Cloned the String into a new Rc: a second hoard, counts stay 1.
                "use std::rc::Rc;\nfn main() { let original = Rc::new(String::from(\"hoard\")); println!(\"{}\", Rc::strong_count(&original)); let shared = Rc::new((*original).clone()); println!(\"{}\", Rc::strong_count(&original)); drop(shared); println!(\"{}\", Rc::strong_count(&original)); let _ = Rc::clone(&original); }",
                // Forgot to drop the second handle.
                "use std::rc::Rc;\nfn main() { let original = Rc::new(String::from(\"hoard\")); println!(\"{}\", Rc::strong_count(&original)); let shared = Rc::clone(&original); println!(\"{}\", Rc::strong_count(&original)); println!(\"{}\", Rc::strong_count(&shared)); }",
            ],
        ),
        (
            "refcell",
            vec![
                // Copied the value out of the guard and changed the copy.
                "use std::cell::RefCell;\nfn main() { let g = RefCell::new(10); let r = &g; println!(\"{}\", r.borrow()); let mut n = *r.borrow_mut(); n += 5; let _ = n; println!(\"{}\", r.borrow()); }",
            ],
        ),
        (
            "cell",
            vec![
                // Changed the copy that get() returned.
                "use std::cell::Cell;\nfn main() { let slot = Cell::new(1); let r = &slot; println!(\"{}\", r.get()); let mut copy = r.get(); copy += 4; let _ = copy; println!(\"{}\", r.get()); r.set(r.get()); }",
            ],
        ),
        (
            "rc_refcell",
            vec![
                // Two boxes: a new Rc<RefCell> around a copy instead of Rc::clone.
                "use std::cell::RefCell;\nuse std::rc::Rc;\nfn main() { let shared = Rc::new(RefCell::new(0)); let other = Rc::new(RefCell::new(*shared.borrow())); *other.borrow_mut() += 10; *shared.borrow_mut() += 5; println!(\"{}\", shared.borrow()); }",
                // Printed through the clone before the original's change.
                "use std::cell::RefCell;\nuse std::rc::Rc;\nfn main() { let shared = Rc::new(RefCell::new(0)); let clone = Rc::clone(&shared); *clone.borrow_mut() += 10; println!(\"{}\", clone.borrow()); *shared.borrow_mut() += 5; }",
            ],
        ),
        (
            "weak_ref",
            vec![
                // Kept the upgraded Rc alive across the drop.
                "use std::rc::{Rc, Weak};\nfn main() { let strong = Rc::new(5); let weak: Weak<i32> = Rc::downgrade(&strong); let alive = weak.upgrade(); println!(\"{alive:?}\"); drop(strong); println!(\"{:?}\", weak.upgrade()); }",
            ],
        ),
    ])
}

/// Programs that print exactly the pinned lines as literals, without the
/// construct the mission teaches. Each must fail the mission's `uses`
/// tokens (`StaticCheckFailed`), proving the token check closes that
/// shortcut. Exactly the missions with non-empty `uses` have entries.
fn literal_prints() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        (
            "vec_iter",
            "fn main() { let v = vec![10, 20, 30]; let _ = v; println!(\"60\"); }",
        ),
        ("closure_basic", "fn main() { println!(\"42\"); }"),
        (
            "enum_match",
            "enum Direction { North, East, South, West }\nfn degrees(d: Direction) -> i32 { if let Direction::North = d { 0 } else if let Direction::East = d { 90 } else if let Direction::South = d { 180 } else { 270 } }\nfn main() { for d in [Direction::North, Direction::East, Direction::South, Direction::West] { println!(\"{}\", degrees(d)); } }",
        ),
        (
            "and_then",
            "fn main() { println!(\"Some(2)\"); println!(\"None\"); }",
        ),
        (
            "hashmap_basic",
            "fn main() { println!(\"Some(7)\"); println!(\"None\"); }",
        ),
        (
            "arc_mutex",
            "use std::sync::{Arc, Mutex};\nfn main() { let _m = Arc::new(Mutex::new(0)); println!(\"5\"); }",
        ),
        (
            "mpsc_channel",
            "use std::sync::mpsc;\nfn main() { let (tx, _rx) = mpsc::channel(); tx.send(7).expect(\"send\"); println!(\"7\"); println!(\"11\"); }",
        ),
        (
            "atomic",
            "fn main() { println!(\"10\"); println!(\"11\"); }",
        ),
        (
            "rc_basic",
            "use std::rc::Rc;\nfn main() { let _r = Rc::new(String::from(\"hoard\")); println!(\"1\"); println!(\"2\"); println!(\"1\"); }",
        ),
        (
            "refcell",
            "use std::cell::RefCell;\nfn main() { let _g = RefCell::new(10); println!(\"10\"); println!(\"15\"); }",
        ),
        (
            "cell",
            "use std::cell::Cell;\nfn main() { let _s = Cell::new(1); println!(\"1\"); println!(\"5\"); }",
        ),
        (
            "rc_refcell",
            "use std::cell::RefCell;\nuse std::rc::Rc;\nfn main() { let _s = Rc::new(RefCell::new(0)); println!(\"15\"); }",
        ),
        (
            "weak_ref",
            "use std::rc::Rc;\nfn main() { let _s = Rc::new(5); println!(\"Some(5)\"); println!(\"None\"); }",
        ),
    ])
}

/// Realistic correct solutions that differ from the canonical one. Each
/// must pass (`Passed`). The game's `tests/contract.rs` `CORRECT` corpus
/// holds the same programs, so they are proven against both graders.
fn alternatives() -> HashMap<&'static str, Vec<&'static str>> {
    HashMap::from([
        (
            "bridge_toll",
            vec![
                "fn weigh(cargo: &str) -> usize {\n    cargo.chars().count()\n}\nfn main() {\n    let cargo = String::from(\"lantern oil\");\n    let toll = weigh(cargo.as_str());\n    println!(\"toll: {toll}\");\n    println!(\"cargo: {cargo}\");\n}\n",
            ],
        ),
        (
            "bridge_quill",
            vec![
                "fn inscribe(list: &mut Vec<String>, line: &str) {\n    list.push(String::from(line));\n}\nfn main() {\n    let mut scroll = vec![String::from(\"north road\")];\n    let first = scroll[0].clone();\n    let second = scroll[0].clone();\n    inscribe(&mut scroll, \"east road\");\n    println!(\"reader one: {first}\");\n    println!(\"reader two: {second}\");\n    println!(\"scroll: {}\", scroll.join(\", \"));\n}\n",
            ],
        ),
        (
            "bridge_lantern",
            vec![
                "fn before<'a, 'b>(line: &'a str, sep: &'b str) -> &'a str {\n    line.split(sep).next().unwrap_or(line)\n}\nfn main() {\n    let lantern = String::from(\"oil:wick:glass\");\n    let part;\n    {\n        let sep = String::from(\":\");\n        part = before(&lantern, &sep);\n    }\n    println!(\"first: {part}\");\n    println!(\"whole: {}\", before(\"glass\", \":\"));\n}\n",
            ],
        ),
    ])
}

#[derive(Debug)]
enum Role {
    Canonical,
    Starter,
    Wrong(usize),
    /// A [`literal_prints`] program: must be `StaticCheckFailed`.
    Literal,
    /// An [`alternatives`] program: must be `Passed`.
    Alternative(usize),
}

impl Role {
    fn label(&self) -> String {
        match self {
            Self::Canonical => "canonical".to_string(),
            Self::Starter => "starter".to_string(),
            Self::Wrong(i) => format!("wrong#{i}"),
            Self::Literal => "literal".to_string(),
            Self::Alternative(i) => format!("alt#{i}"),
        }
    }
}

struct Case {
    id: &'static str,
    role: Role,
    source: String,
}

fn run_parallel(cases: Vec<Case>) -> Vec<(Case, expectations::GradedRun)> {
    let queue = Mutex::new(cases);
    let results = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..workers() {
            s.spawn(|| {
                loop {
                    let Some(case) = queue.lock().expect("queue").pop() else {
                        break;
                    };
                    let graded = expectations::grade(case.id, &case.source, &RunLimits::default())
                        .unwrap_or_else(|e| {
                            panic!("{} {}: setup failure {e:#}", case.id, case.role.label())
                        });
                    results.lock().expect("results").push((case, graded));
                }
            });
        }
    });
    results.into_inner().expect("results")
}

#[test]
fn game_sources_parse() {
    let sols = canonical_solutions();
    let starters = starters();
    assert_eq!(sols.len(), 60, "solutions parsed");
    assert_eq!(starters.len(), 60, "starters parsed");
    for m in expectations::all() {
        assert!(
            sols.contains_key(m.id),
            "no canonical solution for {}",
            m.id
        );
        assert!(starters.contains_key(m.id), "no starter for {}", m.id);
    }
    let wrong = wrong_variants();
    for m in expectations::all() {
        let needs = !matches!(m.check, Check::RunOnly { .. });
        assert_eq!(
            needs,
            wrong.contains_key(m.id),
            "{}: wrong variants must exist exactly for checked missions",
            m.id
        );
    }
    let literal = literal_prints();
    for m in expectations::all() {
        assert_eq!(
            !m.uses.is_empty(),
            literal.contains_key(m.id),
            "{}: a literal-print variant must exist exactly for missions with `uses`",
            m.id
        );
    }
}

#[test]
#[ignore = "slow: one real wasm build per mission"]
fn print_canonical_transcripts() {
    // Regenerates the recorded data in src/expectations/data.rs: harness
    // results and the stdout each canonical solution prints.
    let sols = canonical_solutions();
    let settings = BuildSettings::default();
    for m in expectations::all() {
        let src = &sols[m.id];
        match m.check {
            Check::Harness(h) => {
                let checker = expectations::harness_module_source(&h, "NONCE");
                let layout = Layout::Harness {
                    entry: expectations::HARNESS_ENTRY_SOURCE,
                    checker: &checker,
                };
                let o = wasm_builder::compile_and_run_layout(
                    src,
                    &layout,
                    true,
                    &RunLimits::default(),
                    &settings,
                )
                .expect("build");
                let t = expectations::parse_transcript(&o.stdout, "NONCE");
                println!(
                    "{} [harness] compiled={} ran={} finished={}",
                    m.id, o.compiled, o.ran_ok, t.finished
                );
                for r in &t.results {
                    println!("    {r:?}");
                }
                if let Some(mo) = &t.main_output {
                    println!("    main: {:?}", expectations::normalize(mo));
                }
                if !o.compiled || !o.ran_ok {
                    println!("    stderr: {}", o.stderr);
                }
            }
            Check::CompileOnly { .. } => println!("{} [compile-only]", m.id),
            _ => {
                let o = wasm_builder::compile_and_run(src).expect("build");
                println!(
                    "{} [stdout] ok={} {:?}",
                    m.id,
                    o.ok(),
                    expectations::normalize(&o.stdout)
                );
            }
        }
    }
}

#[test]
#[ignore = "slow: ~200 real wasm builds"]
fn every_expectation_discriminates() {
    let sols = canonical_solutions();
    let starters = starters();
    let wrong = wrong_variants();
    let literal = literal_prints();
    let mut cases = Vec::new();
    for m in expectations::all() {
        if let Some(src) = literal.get(m.id) {
            cases.push(Case {
                id: m.id,
                role: Role::Literal,
                source: (*src).to_string(),
            });
        }
        cases.push(Case {
            id: m.id,
            role: Role::Canonical,
            source: sols[m.id].clone(),
        });
        cases.push(Case {
            id: m.id,
            role: Role::Starter,
            source: starters[m.id].clone(),
        });
        for (i, w) in wrong.get(m.id).into_iter().flatten().enumerate() {
            cases.push(Case {
                id: m.id,
                role: Role::Wrong(i),
                source: (*w).to_string(),
            });
        }
        for (i, a) in alternatives().get(m.id).into_iter().flatten().enumerate() {
            cases.push(Case {
                id: m.id,
                role: Role::Alternative(i),
                source: (*a).to_string(),
            });
        }
    }
    let total = cases.len();
    let results = run_parallel(cases);
    assert_eq!(results.len(), total);

    let mut failures = Vec::new();
    let mut proven: HashMap<&str, (bool, bool, usize, usize)> = HashMap::new();
    for (case, g) in &results {
        let check = expectations::lookup(case.id).expect("known").check;
        let entry = proven.entry(case.id).or_default();
        let ok = match (&case.role, check) {
            (Role::Canonical, Check::Stdout { .. } | Check::Harness(_)) => {
                g.state == GradeState::Passed
            }
            (Role::Canonical, Check::RunOnly { .. }) => g.state == GradeState::RanNotChecked,
            (Role::Canonical, Check::CompileOnly { .. }) => {
                g.state == GradeState::CompiledNotChecked && g.outcome.run_time.is_none()
            }
            // A run-only mission cannot reject its starter; record only.
            (Role::Starter, Check::RunOnly { .. }) => true,
            (Role::Starter | Role::Wrong(_), _) => !g.ok(),
            (Role::Literal, _) => g.state == GradeState::StaticCheckFailed,
            (Role::Alternative(_), _) => g.state == GradeState::Passed,
        };
        match case.role {
            Role::Canonical => entry.0 = ok,
            Role::Starter => entry.1 = ok,
            Role::Wrong(_) => {
                entry.2 += 1;
                if ok {
                    entry.3 += 1;
                }
            }
            // Counted through `failures` only.
            Role::Literal | Role::Alternative(_) => {}
        }
        println!(
            "{:<22} {:<10} -> {:?}{}",
            case.id,
            case.role.label(),
            g.state,
            if ok { "" } else { "   <-- UNEXPECTED" }
        );
        if !ok {
            failures.push(format!(
                "{} {:?}: got {:?}\nstdout: {}\nstderr: {}",
                case.id,
                case.role.label(),
                g.state,
                g.outcome.stdout,
                g.outcome.stderr
            ));
        }
    }
    let fully: Vec<&str> = expectations::all()
        .iter()
        .filter(|m| matches!(m.check, Check::Stdout { .. } | Check::Harness(_)))
        .filter(|m| {
            proven
                .get(m.id)
                .is_some_and(|p| p.0 && p.1 && p.2 > 0 && p.2 == p.3)
        })
        .map(|m| m.id)
        .collect();
    println!(
        "behaviour-checked missions proven (canonical passes, starter fails, all wrong variants fail): {}",
        fully.len()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
