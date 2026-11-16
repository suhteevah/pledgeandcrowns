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

mod act10_programs;
mod act9_programs;
use act9_programs as a9;
use act10_programs as a10;

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

// Shared items for the boss-trial variants below (string literals, so they
// can be `concat!`ed into `&'static str` programs).

/// The Wraith enum of the Variant Wraith's stages 2 and 3.
macro_rules! wraith_enum {
    () => {
        "#[derive(Debug, Clone, Copy)]\nenum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow }\n"
    };
}
/// Stage 2's `main`.
macro_rules! wraith_main3 {
    () => {
        "fn main() { for form in [Wraith::Solid { armor: 3 }, Wraith::Mist { density: 2 }, Wraith::Shadow] { println!(\"{form:?} takes {}\", damage(form, 10)); } }\n"
    };
}
/// Stage 3's enum and `main`.
macro_rules! wraith_enum4 {
    () => {
        "#[derive(Debug, Clone, Copy)]\nenum Wraith { Solid { armor: u32 }, Mist { density: u32 }, Shadow, Possessed { will: u32 } }\n"
    };
}
macro_rules! wraith_main4 {
    () => {
        "fn main() { for form in [Wraith::Solid { armor: 3 }, Wraith::Mist { density: 2 }, Wraith::Shadow, Wraith::Possessed { will: 4 }] { println!(\"{form:?} takes {}\", damage(form, 10)); } }\n"
    };
}
/// The Chimera's trait and its Fire head.
macro_rules! chimera_fire {
    () => {
        "trait Enemy { fn name(&self) -> &'static str; fn breath(&self) -> u32; }\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire { fn name(&self) -> &'static str { \"fire\" } fn breath(&self) -> u32 { 7 } }\n"
    };
}
/// The trait and all three original heads.
macro_rules! chimera_three {
    () => {
        concat!(
            chimera_fire!(),
            "impl Enemy for Frost { fn name(&self) -> &'static str { \"frost\" } fn breath(&self) -> u32 { 5 } }\n",
            "impl Enemy for Storm { fn name(&self) -> &'static str { \"storm\" } fn breath(&self) -> u32 { 9 } }\n"
        )
    };
}
/// Plus the Plasma head and stage 3's `main`.
macro_rules! chimera_plasma_items {
    () => {
        concat!(
            chimera_three!(),
            "struct Plasma;\nimpl Enemy for Plasma { fn name(&self) -> &'static str { \"plasma\" } fn breath(&self) -> u32 { 12 } }\n",
            "fn main() { let heads: Vec<Box<dyn Enemy>> = vec![Box::new(Fire), Box::new(Frost), Box::new(Storm), Box::new(Plasma)]; println!(\"total: {}\", battle(&heads)); }\n"
        )
    };
}
/// Everything of the Wager's stage 3 except `quest`.
macro_rules! quest_items {
    () => {
        "use std::num::ParseIntError;\n#[derive(Debug)]\nstruct WrongKey;\n#[derive(Debug)]\nstruct Short;\n#[derive(Debug)]\nstruct Reward { gold: u32 }\n#[derive(Debug)]\nenum QuestError { BadMap, Locked, Broke }\nimpl From<ParseIntError> for QuestError { fn from(_: ParseIntError) -> Self { QuestError::BadMap } }\nimpl From<WrongKey> for QuestError { fn from(_: WrongKey) -> Self { QuestError::Locked } }\nimpl From<Short> for QuestError { fn from(_: Short) -> Self { QuestError::Broke } }\nfn read_map(map: &str) -> Result<u32, ParseIntError> { map.trim().parse::<u32>() }\nfn open_vault(key: &str) -> Result<u32, WrongKey> { if key == \"brass\" { Ok(30) } else { Err(WrongKey) } }\nfn pay_guide(gold: u32, leagues: u32) -> Result<u32, Short> { let fee = leagues * 2; if fee > gold { Err(Short) } else { Ok(gold - fee) } }\nfn main() { for (m, k) in [(\"4\", \"brass\"), (\"far\", \"brass\"), (\"4\", \"bone\"), (\"20\", \"brass\")] { println!(\"{:?}\", quest(m, k).map(|r| r.gold)); } }\n"
    };
}
/// Everything of the Wager's stage 2 except the `From` impls.
macro_rules! wager_items {
    () => {
        "use std::num::ParseIntError;\n#[derive(Debug)]\nstruct Short;\nfn pay(purse: u32, stake: u32) -> Result<u32, Short> { if stake > purse { Err(Short) } else { Ok(purse - stake) } }\n#[derive(Debug)]\nenum WagerError { BadStake, TooPoor }\n"
    };
}
macro_rules! wager_main {
    () => {
        "fn main() { println!(\"{:?}\", place_bet(\"15\", 50)); println!(\"{:?}\", place_bet(\"lots\", 50)); println!(\"{:?}\", place_bet(\"80\", 50)); }\n"
    };
}
/// The Forge-Golem's `Part` and its five parts (every stage).
macro_rules! golem_items {
    () => {
        "struct Part { name: &'static str, armoured: bool, weakness: u32 }\nfn golem() -> Vec<Part> { vec![Part { name: \"arm\", armoured: false, weakness: 3 }, Part { name: \"chest\", armoured: true, weakness: 8 }, Part { name: \"knee\", armoured: false, weakness: 4 }, Part { name: \"head\", armoured: true, weakness: 5 }, Part { name: \"eye\", armoured: false, weakness: 6 }] }\n"
    };
}
/// Stage 1's `main`.
macro_rules! golem_main1 {
    () => {
        "fn main() { let parts = golem(); println!(\"weak points: {:?}\", weak_points(&parts)); }\n"
    };
}
/// Stage 2's `main`.
macro_rules! golem_main2 {
    () => {
        "fn main() { let parts = golem(); let names: Vec<&str> = parts.iter().map(|p| p.name).collect(); println!(\"golem: {}\", names.join(\" \")); println!(\"core damage: {}\", core_damage(&parts)); }\n"
    };
}
/// Stage 3's `main` (no loop, no `mut`).
macro_rules! golem_main3 {
    () => {
        "fn main() { let parts = golem(); let names: Vec<&str> = parts.iter().map(|p| p.name).collect(); println!(\"golem: {}\", names.join(\" \")); println!(\"blow 1: {}\", total_damage(&parts, 1)); println!(\"blow 3: {}\", total_damage(&parts, 3)); }\n"
    };
}
/// The Tide Hydra's `fight` (stages 1 and 2).
macro_rules! hydra_fight {
    () => {
        "use std::sync::mpsc::{self, Sender};\nfn fight(head: &'static str, hp: u32, tx: Sender<String>) { let blows = hp.div_ceil(4); tx.send(format!(\"{head} down after {blows} blows\")).expect(\"receiver dropped\"); }\n"
    };
}
/// The `main` of stages 1 and 2.
macro_rules! hydra_main {
    () => {
        "fn main() { let reports = hunt(&[(\"fang\", 9), (\"venom\", 6), (\"brine\", 13)]); for report in &reports { println!(\"{report}\"); } println!(\"{} heads down\", reports.len()); }\n"
    };
}
/// Stage 3: everything except `await_heads`. Its `main` spawns threads;
/// the checker does not run it.
macro_rules! hydra_body_items {
    () => {
        "use std::sync::mpsc::{self, Receiver};\nuse std::thread;\nenum Report { Hit, Down }\nfn main() { let blows = [3, 2, 4]; let (tx, rx) = mpsc::channel(); for n in blows { let tx = tx.clone(); thread::spawn(move || { for _ in 0..n { tx.send(Report::Hit).expect(\"receiver dropped\"); } tx.send(Report::Down).expect(\"receiver dropped\"); }); } drop(tx); let hits = await_heads(&rx, blows.len()); println!(\"{} heads down after {hits} hits\", blows.len()); println!(\"strike the body\"); }\n"
    };
}
/// The Memory Warden's ledger: everything except `total`.
macro_rules! ledger_items {
    () => {
        "#[derive(Debug)]\nenum Ledger { Entry(u32, Box<Ledger>), Closed }\nfn entry(amount: u32, rest: Ledger) -> Ledger { Ledger::Entry(amount, Box::new(rest)) }\nfn main() { let ledger = entry(5, entry(7, entry(3, Ledger::Closed))); println!(\"{ledger:?}\"); println!(\"owed: {}\", total(&ledger)); }\n"
    };
}
/// Stage 2's `Debtor` (its `main` is separate).
macro_rules! vault_debtor {
    () => {
        "use std::cell::RefCell;\nuse std::rc::Rc;\nstruct Debtor { name: &'static str, vault: Rc<RefCell<u32>> }\n"
    };
}
macro_rules! vault_main {
    () => {
        "fn main() { let vault = Rc::new(RefCell::new(0)); let miller = Debtor { name: \"miller\", vault: Rc::clone(&vault) }; let smith = Debtor { name: \"smith\", vault: Rc::clone(&vault) }; pay(&miller, 5); pay(&smith, 7); println!(\"{} and {} paid\", miller.name, smith.name); println!(\"vault holds {}\", vault.borrow()); println!(\"handles: {}\", Rc::strong_count(&vault)); }\n"
    };
}
/// Stage 3: everything except `circle`.
macro_rules! ring_items {
    () => {
        "use std::cell::RefCell;\nuse std::rc::{Rc, Weak};\nstruct Debtor { name: &'static str, owes: RefCell<Weak<Debtor>> }\nfn debtor(name: &'static str) -> Rc<Debtor> { Rc::new(Debtor { name, owes: RefCell::new(Weak::new()) }) }\nfn main() { let warden = debtor(\"warden\"); let you = debtor(\"you\"); let teller = debtor(\"teller\"); *warden.owes.borrow_mut() = Rc::downgrade(&you); *you.owes.borrow_mut() = Rc::downgrade(&teller); *teller.owes.borrow_mut() = Rc::downgrade(&warden); println!(\"circle: {:?}\", circle(&warden)); let strong = Rc::strong_count(&warden); let weak = Rc::weak_count(&warden); println!(\"warden: strong {strong}, weak {weak}\"); let probe = Rc::downgrade(&warden); drop((warden, you, teller)); let left = probe.upgrade().map(|d| d.name); println!(\"after the ledger closes: {left:?}\"); }\n"
    };
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
        // The Guildmaster's trial, The Variant Wraith.
        (
            "wraith_forms",
            vec![
                // Names the Shadow, but answers it with steel.
                "#[derive(Debug, Clone, Copy)]\nenum Wraith { Solid, Mist, Shadow }\nfn counter(form: Wraith) -> &'static str { match form { Wraith::Solid => \"steel\", Wraith::Mist => \"wind\", Wraith::Shadow => \"steel\" } }\nfn main() { for form in [Wraith::Solid, Wraith::Mist, Wraith::Shadow] { println!(\"{form:?}: {}\", counter(form)); } }",
                // Solid and Mist answers swapped.
                "#[derive(Debug, Clone, Copy)]\nenum Wraith { Solid, Mist, Shadow }\nfn counter(form: Wraith) -> &'static str { match form { Wraith::Solid => \"wind\", Wraith::Mist => \"steel\", Wraith::Shadow => \"light\" } }\nfn main() { for form in [Wraith::Solid, Wraith::Mist, Wraith::Shadow] { println!(\"{form:?}: {}\", counter(form)); } }",
                // Right `counter`, but `main` never shows the Shadow.
                "#[derive(Debug, Clone, Copy)]\nenum Wraith { Solid, Mist, Shadow }\nfn counter(form: Wraith) -> &'static str { match form { Wraith::Solid => \"steel\", Wraith::Mist => \"wind\", Wraith::Shadow => \"light\" } }\nfn main() { for form in [Wraith::Solid, Wraith::Mist] { println!(\"{form:?}: {}\", counter(form)); } }",
            ],
        ),
        (
            "wraith_essence",
            vec![
                // rustc's help followed: the fields are bound to `_` and ignored.
                concat!(
                    wraith_enum!(),
                    "fn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor: _ } => blow, Wraith::Mist { density: _ } => blow, Wraith::Shadow => 0 } }\n",
                    wraith_main3!()
                ),
                // Mist multiplies instead of spreading the blow.
                concat!(
                    wraith_enum!(),
                    "fn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor } => blow - armor, Wraith::Mist { density } => blow * density, Wraith::Shadow => 0 } }\n",
                    wraith_main3!()
                ),
                // Armor adds to the blow.
                concat!(
                    wraith_enum!(),
                    "fn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor } => blow + armor, Wraith::Mist { density } => blow / density, Wraith::Shadow => 0 } }\n",
                    wraith_main3!()
                ),
            ],
        ),
        (
            "wraith_possessed",
            vec![
                // A blanket `_ => 0` arm: compiles, strikes the host for nothing.
                concat!(
                    wraith_enum4!(),
                    "fn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor } => blow - armor, Wraith::Mist { density } => blow / density, Wraith::Shadow => 0, _ => 0 } }\n",
                    wraith_main4!()
                ),
                // Named, but the will is ignored.
                concat!(
                    wraith_enum4!(),
                    "fn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor } => blow - armor, Wraith::Mist { density } => blow / density, Wraith::Shadow => 0, Wraith::Possessed { .. } => blow } }\n",
                    wraith_main4!()
                ),
                // The will subtracts instead of adding.
                concat!(
                    wraith_enum4!(),
                    "fn damage(form: Wraith, blow: u32) -> u32 { match form { Wraith::Solid { armor } => blow - armor, Wraith::Mist { density } => blow / density, Wraith::Shadow => 0, Wraith::Possessed { will } => blow - will } }\n",
                    wraith_main4!()
                ),
            ],
        ),
        // Vexis's trial, The Chimera of Vexis.
        (
            "chimera_heads",
            vec![
                // Plain `impl` blocks: right output, but Frost and Storm are no `Enemy`.
                concat!(
                    chimera_fire!(),
                    "impl Frost { fn name(&self) -> &'static str { \"frost\" } fn breath(&self) -> u32 { 5 } }\nimpl Storm { fn name(&self) -> &'static str { \"storm\" } fn breath(&self) -> u32 { 9 } }\n",
                    "fn main() { println!(\"{} breathes {}\", Fire.name(), Fire.breath()); println!(\"{} breathes {}\", Frost.name(), Frost.breath()); println!(\"{} breathes {}\", Storm.name(), Storm.breath()); }"
                ),
                // Frost and Storm breaths swapped.
                concat!(
                    chimera_fire!(),
                    "impl Enemy for Frost { fn name(&self) -> &'static str { \"frost\" } fn breath(&self) -> u32 { 9 } }\nimpl Enemy for Storm { fn name(&self) -> &'static str { \"storm\" } fn breath(&self) -> u32 { 5 } }\n",
                    "fn main() { println!(\"{} breathes {}\", Fire.name(), Fire.breath()); println!(\"{} breathes {}\", Frost.name(), Frost.breath()); println!(\"{} breathes {}\", Storm.name(), Storm.breath()); }"
                ),
                // Storm's name capitalised.
                concat!(
                    chimera_fire!(),
                    "impl Enemy for Frost { fn name(&self) -> &'static str { \"frost\" } fn breath(&self) -> u32 { 5 } }\nimpl Enemy for Storm { fn name(&self) -> &'static str { \"Storm\" } fn breath(&self) -> u32 { 9 } }\n",
                    "fn main() { println!(\"{} breathes {}\", Fire.name(), Fire.breath()); println!(\"{} breathes {}\", Frost.name(), Frost.breath()); println!(\"{} breathes {}\", Storm.name(), Storm.breath()); }"
                ),
            ],
        ),
        (
            "chimera_bound",
            vec![
                // A trait object, not a bounded generic: right output, wrong signature.
                concat!(
                    chimera_three!(),
                    "fn volley(head: &dyn Enemy, rounds: u32) -> String { format!(\"{} x{rounds} = {}\", head.name(), head.breath() * rounds) }\n",
                    "fn main() { println!(\"{}\", volley(&Fire, 3)); println!(\"{}\", volley(&Frost, 3)); println!(\"{}\", volley(&Storm, 3)); }"
                ),
                // Adds the rounds instead of multiplying.
                concat!(
                    chimera_three!(),
                    "fn volley<T: Enemy>(head: &T, rounds: u32) -> String { format!(\"{} x{rounds} = {}\", head.name(), head.breath() + rounds) }\n",
                    "fn main() { println!(\"{}\", volley(&Fire, 3)); println!(\"{}\", volley(&Frost, 3)); println!(\"{}\", volley(&Storm, 3)); }"
                ),
                // Always three rounds, whatever it is asked: right `main` output only.
                concat!(
                    chimera_three!(),
                    "fn volley<T: Enemy>(head: &T, _rounds: u32) -> String { format!(\"{} x3 = {}\", head.name(), head.breath() * 3) }\n",
                    "fn main() { println!(\"{}\", volley(&Fire, 3)); println!(\"{}\", volley(&Frost, 3)); println!(\"{}\", volley(&Storm, 3)); }"
                ),
            ],
        ),
        (
            "chimera_plasma",
            vec![
                // Prints the heads, but the total is hard-coded.
                concat!(
                    chimera_plasma_items!(),
                    "fn battle(heads: &[Box<dyn Enemy>]) -> u32 { for head in heads { println!(\"{} breathes {}\", head.name(), head.breath()); } 33 }\n"
                ),
                // Only the first three heads fight.
                concat!(
                    chimera_plasma_items!(),
                    "fn battle(heads: &[Box<dyn Enemy>]) -> u32 { let mut total = 0; for head in heads.iter().take(3) { println!(\"{} breathes {}\", head.name(), head.breath()); total += head.breath(); } total }\n"
                ),
                // Counts the heads, not their breath.
                concat!(
                    chimera_plasma_items!(),
                    "fn battle(heads: &[Box<dyn Enemy>]) -> u32 { let mut total = 0; for head in heads { println!(\"{} breathes {}\", head.name(), head.breath()); total += 1; } total }\n"
                ),
            ],
        ),
        // The Barkeep's trial, The Tavernkeeper's Wager.
        (
            "wager_stake",
            vec![
                // Forgets to double.
                "use std::num::ParseIntError;\nfn read_stake(text: &str) -> Result<u32, ParseIntError> { text.trim().parse::<u32>() }\nfn double_stake(text: &str) -> Result<u32, ParseIntError> { let stake = read_stake(text)?; Ok(stake) }\nfn main() { println!(\"{:?}\", double_stake(\"20\")); println!(\"{:?}\", double_stake(\"twenty\")); }",
                // Swallows the error: a bad stake becomes Ok(0).
                "use std::num::ParseIntError;\nfn read_stake(text: &str) -> Result<u32, ParseIntError> { text.trim().parse::<u32>() }\nfn double_stake(text: &str) -> Result<u32, ParseIntError> { Ok(read_stake(text).unwrap_or(0) * 2) }\nfn main() { println!(\"{:?}\", double_stake(\"20\")); println!(\"{:?}\", double_stake(\"twenty\")); }",
                // `.unwrap()` inside: panics on `twenty`.
                "use std::num::ParseIntError;\nfn read_stake(text: &str) -> Result<u32, ParseIntError> { text.trim().parse::<u32>() }\nfn double_stake(text: &str) -> Result<u32, ParseIntError> { Ok(read_stake(text).unwrap() * 2) }\nfn main() { println!(\"{:?}\", double_stake(\"20\")); println!(\"{:?}\", double_stake(\"twenty\")); }",
                // A String error: right-looking output, wrong signature.
                "use std::num::ParseIntError;\nfn read_stake(text: &str) -> Result<u32, ParseIntError> { text.trim().parse::<u32>() }\nfn double_stake(text: &str) -> Result<u32, String> { let stake = read_stake(text).map_err(|e| format!(\"{e:?}\"))?; Ok(stake * 2) }\nfn main() { println!(\"{:?}\", double_stake(\"20\")); println!(\"{:?}\", double_stake(\"twenty\")); }",
            ],
        ),
        (
            "wager_from",
            vec![
                // The two conversions swapped.
                concat!(
                    wager_items!(),
                    "impl From<ParseIntError> for WagerError { fn from(_: ParseIntError) -> Self { WagerError::TooPoor } }\nimpl From<Short> for WagerError { fn from(_: Short) -> Self { WagerError::BadStake } }\n",
                    "fn place_bet(text: &str, purse: u32) -> Result<u32, WagerError> { let stake = text.trim().parse::<u32>()?; let left = pay(purse, stake)?; Ok(left) }\n",
                    wager_main!()
                ),
                // Never pays: a stake over the purse leaves Ok(0).
                concat!(
                    wager_items!(),
                    "impl From<ParseIntError> for WagerError { fn from(_: ParseIntError) -> Self { WagerError::BadStake } }\nimpl From<Short> for WagerError { fn from(_: Short) -> Self { WagerError::TooPoor } }\n",
                    "fn place_bet(text: &str, purse: u32) -> Result<u32, WagerError> { let _ = pay; let stake = text.trim().parse::<u32>()?; Ok(purse.saturating_sub(stake)) }\n",
                    wager_main!()
                ),
                // No `From` at all: a String error instead of WagerError.
                concat!(
                    wager_items!(),
                    "fn place_bet(text: &str, purse: u32) -> Result<u32, String> { let stake = text.trim().parse::<u32>().map_err(|_| String::from(\"BadStake\"))?; pay(purse, stake).map_err(|_| String::from(\"TooPoor\")) }\n",
                    wager_main!()
                ),
            ],
        ),
        (
            "wager_quest",
            vec![
                // Swallows a bad map: unwrap_or(0) walks zero leagues.
                concat!(
                    quest_items!(),
                    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { let leagues = read_map(map).unwrap_or(0); let gold = open_vault(key)?; let left = pay_guide(gold, leagues)?; Ok(Reward { gold: left }) }\n"
                ),
                // Opens the vault before reading the map: a bad map and a
                // wrong key report Locked, not BadMap.
                concat!(
                    quest_items!(),
                    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { let gold = open_vault(key)?; let leagues = read_map(map)?; let left = pay_guide(gold, leagues)?; Ok(Reward { gold: left }) }\n"
                ),
                // Every failure reported as BadMap.
                concat!(
                    quest_items!(),
                    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { let leagues = read_map(map)?; let gold = open_vault(key).map_err(|_| QuestError::BadMap)?; let left = pay_guide(gold, leagues).map_err(|_| QuestError::BadMap)?; Ok(Reward { gold: left }) }\n"
                ),
            ],
        ),
        // The Keymaster's trial, The Forge-Golem.
        (
            "golem_parts",
            vec![
                // Keeps the armoured parts instead.
                concat!(
                    golem_items!(),
                    "fn weak_points(parts: &[Part]) -> Vec<String> { parts.iter().filter(|p| p.armoured).map(|p| format!(\"{} ({})\", p.name, p.weakness)).collect() }\n",
                    golem_main1!()
                ),
                // No filter at all.
                concat!(
                    golem_items!(),
                    "fn weak_points(parts: &[Part]) -> Vec<String> { parts.iter().map(|p| format!(\"{} ({})\", p.name, p.weakness)).collect() }\n",
                    golem_main1!()
                ),
                // Labels without the weakness.
                concat!(
                    golem_items!(),
                    "fn weak_points(parts: &[Part]) -> Vec<String> { parts.iter().filter(|p| !p.armoured).map(|p| p.name.to_string()).collect() }\n",
                    golem_main1!()
                ),
                // Drops the chest and the head by name: right `main` output only.
                concat!(
                    golem_items!(),
                    "fn weak_points(parts: &[Part]) -> Vec<String> { parts.iter().map(|p| format!(\"{} ({})\", p.name, p.weakness)).filter(|l| !l.starts_with(\"chest\") && !l.starts_with(\"head\")).collect() }\n",
                    golem_main1!()
                ),
            ],
        ),
        (
            "golem_sum",
            vec![
                // Sums every part, armour and all.
                concat!(
                    golem_items!(),
                    "fn core_damage(parts: &[Part]) -> u32 { let total: u32 = parts.iter().map(|p| p.weakness).sum(); total * 2 }\n",
                    golem_main2!()
                ),
                // Forgets to double.
                concat!(
                    golem_items!(),
                    "fn core_damage(parts: &[Part]) -> u32 { let total: u32 = parts.iter().filter(|p| !p.armoured).map(|p| p.weakness).sum(); total }\n",
                    golem_main2!()
                ),
                // Counts the weak points instead of adding their weakness.
                concat!(
                    golem_items!(),
                    "fn core_damage(parts: &[Part]) -> u32 { let total = parts.iter().filter(|p| !p.armoured).count() as u32; total * 2 }\n",
                    golem_main2!()
                ),
            ],
        ),
        (
            "golem_core",
            vec![
                // No filter: armour takes the blow too.
                concat!(
                    golem_items!(),
                    "fn total_damage(parts: &[Part], blow: u32) -> u32 { parts.iter().map(|p| blow * p.weakness).sum() }\n",
                    golem_main3!()
                ),
                // Adds the blow instead of multiplying.
                concat!(
                    golem_items!(),
                    "fn total_damage(parts: &[Part], blow: u32) -> u32 { parts.iter().filter(|p| !p.armoured).map(|p| blow + p.weakness).sum() }\n",
                    golem_main3!()
                ),
                // `take_while` stops at the first armoured part, like the
                // starter's `break`.
                concat!(
                    golem_items!(),
                    "fn total_damage(parts: &[Part], blow: u32) -> u32 { parts.iter().take_while(|p| !p.armoured).map(|p| blow * p.weakness).sum() }\n",
                    golem_main3!()
                ),
            ],
        ),
        // The Dockmaster's trial, The Tide Hydra.
        (
            "hydra_senders",
            vec![
                // A new channel per head, keeping only the last receiver.
                concat!(
                    hydra_fight!(),
                    "fn hunt(heads: &[(&'static str, u32)]) -> Vec<String> { let mut last = Vec::new(); for &(head, hp) in heads { let (tx, rx) = mpsc::channel(); fight(head, hp, tx); last = rx.iter().collect(); } last }\n",
                    hydra_main!()
                ),
                // Collects only the first report.
                concat!(
                    hydra_fight!(),
                    "fn hunt(heads: &[(&'static str, u32)]) -> Vec<String> { let (tx, rx) = mpsc::channel(); for &(head, hp) in heads { fight(head, hp, tx.clone()); } drop(tx); rx.iter().take(1).collect() }\n",
                    hydra_main!()
                ),
                // Skips the first report.
                concat!(
                    hydra_fight!(),
                    "fn hunt(heads: &[(&'static str, u32)]) -> Vec<String> { let (tx, rx) = mpsc::channel(); for &(head, hp) in heads { fight(head, hp, tx.clone()); } drop(tx); rx.iter().skip(1).collect() }\n",
                    hydra_main!()
                ),
            ],
        ),
        (
            // Compile-only on this server: each fails the static check.
            "hydra_threads",
            vec![
                // A `move` closure, but called in place: no thread.
                concat!(
                    hydra_fight!(),
                    "use std::thread;\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> { let (tx, rx) = mpsc::channel(); for &(head, hp) in heads { let tx = tx.clone(); let job = move || fight(head, hp, tx); job(); } drop(tx); let _ = thread::current(); let mut reports: Vec<String> = rx.iter().collect(); reports.sort(); reports }\n",
                    hydra_main!()
                ),
                // Threads, but the reports are not sorted.
                concat!(
                    hydra_fight!(),
                    "use std::thread;\nfn hunt(heads: &[(&'static str, u32)]) -> Vec<String> { let (tx, rx) = mpsc::channel(); for &(head, hp) in heads { let tx = tx.clone(); thread::spawn(move || fight(head, hp, tx)); } drop(tx); rx.iter().collect() }\n",
                    hydra_main!()
                ),
                // Stage 1's answer again: no thread, no `move`.
                concat!(
                    hydra_fight!(),
                    "fn hunt(heads: &[(&'static str, u32)]) -> Vec<String> { let (tx, rx) = mpsc::channel(); for &(head, hp) in heads { fight(head, hp, tx.clone()); } drop(tx); let mut reports: Vec<String> = rx.iter().collect(); reports.sort(); reports }\n",
                    hydra_main!()
                ),
            ],
        ),
        (
            "hydra_body",
            vec![
                // Counts each `Down` as a hit too.
                concat!(
                    hydra_body_items!(),
                    "fn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 { let mut hits = 0; let mut down = 0; while down < heads { match rx.recv() { Ok(Report::Hit) => hits += 1, Ok(Report::Down) => { hits += 1; down += 1 } Err(_) => break } } hits }\n"
                ),
                // Stops one head early.
                concat!(
                    hydra_body_items!(),
                    "fn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 { let mut hits = 0; let mut down = 0; while down + 1 < heads { match rx.recv() { Ok(Report::Hit) => hits += 1, Ok(Report::Down) => down += 1, Err(_) => break } } hits }\n"
                ),
                // Returns the heads that fell, not the hits.
                concat!(
                    hydra_body_items!(),
                    "fn await_heads(rx: &Receiver<Report>, heads: usize) -> u32 { let mut down = 0; while down < heads { match rx.recv() { Ok(Report::Hit) => {} Ok(Report::Down) => down += 1, Err(_) => break } } down as u32 }\n"
                ),
            ],
        ),
        // The Vaultwright's trial, The Memory Warden.
        (
            "warden_box",
            vec![
                // Only the first entry counts.
                concat!(
                    ledger_items!(),
                    "fn total(ledger: &Ledger) -> u32 { match ledger { Ledger::Entry(amount, _) => *amount, Ledger::Closed => 0 } }\n"
                ),
                // Counts the entries instead of adding them.
                concat!(
                    ledger_items!(),
                    "fn total(ledger: &Ledger) -> u32 { match ledger { Ledger::Entry(_, rest) => 1 + total(rest), Ledger::Closed => 0 } }\n"
                ),
                // A closed ledger owes one.
                concat!(
                    ledger_items!(),
                    "fn total(ledger: &Ledger) -> u32 { match ledger { Ledger::Entry(amount, rest) => amount + total(rest), Ledger::Closed => 1 } }\n"
                ),
            ],
        ),
        (
            "warden_shared",
            vec![
                // Replaces the vault's coins instead of adding.
                concat!(
                    vault_debtor!(),
                    "fn pay(debtor: &Debtor, coins: u32) { *debtor.vault.borrow_mut() = coins; }\n",
                    vault_main!()
                ),
                // Adds to a copy: the vault never changes.
                concat!(
                    vault_debtor!(),
                    "fn pay(debtor: &Debtor, coins: u32) { let mut copy = *debtor.vault.borrow(); copy += coins; let _ = copy; }\n",
                    vault_main!()
                ),
                // Right `pay`, but each debtor gets a vault of their own.
                concat!(
                    vault_debtor!(),
                    "fn pay(debtor: &Debtor, coins: u32) { *debtor.vault.borrow_mut() += coins; }\n",
                    "fn main() { let vault = Rc::new(RefCell::new(0)); let miller = Debtor { name: \"miller\", vault: Rc::new(RefCell::new(0)) }; let smith = Debtor { name: \"smith\", vault: Rc::clone(&vault) }; pay(&miller, 5); pay(&smith, 7); println!(\"{} and {} paid\", miller.name, smith.name); println!(\"vault holds {}\", vault.borrow()); println!(\"handles: {}\", Rc::strong_count(&vault)); }\n"
                ),
            ],
        ),
        (
            "warden_weak",
            vec![
                // Follows one link and stops.
                concat!(
                    ring_items!(),
                    "fn circle(start: &Rc<Debtor>) -> Vec<&'static str> { let mut names = vec![start.name]; if let Some(d) = start.owes.borrow().upgrade() { if !Rc::ptr_eq(&d, start) { names.push(d.name); } } names }\n"
                ),
                // Names the start again when the ring closes.
                concat!(
                    ring_items!(),
                    "fn circle(start: &Rc<Debtor>) -> Vec<&'static str> { let mut names = vec![start.name]; let mut at = Rc::clone(start); loop { let next = at.owes.borrow().upgrade(); match next { Some(d) => { names.push(d.name); if Rc::ptr_eq(&d, start) { return names; } at = d; } None => return names } } }\n"
                ),
                // Leaves the start out.
                concat!(
                    ring_items!(),
                    "fn circle(start: &Rc<Debtor>) -> Vec<&'static str> { let mut names = Vec::new(); let mut at = Rc::clone(start); loop { let next = at.owes.borrow().upgrade(); match next { Some(d) if !Rc::ptr_eq(&d, start) => { names.push(d.name); at = d; } _ => return names } } }\n"
                ),
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
        // ── Act 9 (tests/act9_programs). The `late` harness missions also
        // have the spec's wrong variant plus checker-only wrongs: right
        // `main` output, wrong for inputs only the checker sends.
        ("raw_pointer", vec![a9::WRONG_RAW_POINTER_0]),
        (
            "extern_c",
            vec![
                // No repr(C): Rust's own layout, 8 bytes on this target.
                a9::WRONG_EXTERN_C_0,
                a9::WRONG_EXTERN_C_1,
                a9::WRONG_EXTERN_C_2,
            ],
        ),
        ("macro_rules", vec![a9::WRONG_MACRO_RULES_0]),
        ("drop_trait", vec![a9::WRONG_DROP_TRAIT_0]),
        (
            "send_sync",
            vec![
                a9::WRONG_SEND_SYNC_0,
                a9::WRONG_SEND_SYNC_1,
                a9::WRONG_SEND_SYNC_2,
            ],
        ),
        (
            "maybe_uninit",
            vec![a9::WRONG_MAYBE_UNINIT_0, a9::WRONG_MAYBE_UNINIT_1],
        ),
        (
            "librarian_index",
            vec![
                a9::WRONG_LIBRARIAN_INDEX_0,
                a9::WRONG_LIBRARIAN_INDEX_1,
                a9::WRONG_LIBRARIAN_INDEX_2,
            ],
        ),
        (
            "librarian_demand",
            vec![
                a9::WRONG_LIBRARIAN_DEMAND_0,
                a9::WRONG_LIBRARIAN_DEMAND_1,
                a9::WRONG_LIBRARIAN_DEMAND_2,
            ],
        ),
        (
            "librarian_seal",
            vec![
                a9::WRONG_LIBRARIAN_SEAL_0,
                a9::WRONG_LIBRARIAN_SEAL_1,
                a9::WRONG_LIBRARIAN_SEAL_2,
            ],
        ),
        // ── Act 10 (tests/act10_programs): the spec's wrong variants, all
        // pass the `uses` tokens and fail the pinned stdout (the two
        // allocator ones print a count above 1 in the sandbox), plus two
        // more per Compiler stage.
        ("prealloc", vec![a10::WRONG_PREALLOC_0]),
        ("cow_borrow", vec![a10::WRONG_COW_BORROW_0]),
        ("const_eval", vec![a10::WRONG_CONST_EVAL_0]),
        ("chunks_exact", vec![a10::WRONG_CHUNKS_EXACT_0]),
        ("no_std", vec![a10::WRONG_NO_STD_0]),
        ("semver_caret", vec![a10::WRONG_SEMVER_CARET_0]),
        (
            "compiler_alloc",
            vec![
                a10::WRONG_COMPILER_ALLOC_0,
                a10::WRONG_COMPILER_ALLOC_1,
                a10::WRONG_COMPILER_ALLOC_2,
            ],
        ),
        (
            "compiler_lifetime",
            vec![
                a10::WRONG_COMPILER_LIFETIME_0,
                // rustc's E0106 help followed literally: E0597 in `main`.
                a10::WRONG_COMPILER_LIFETIME_1,
                a10::WRONG_COMPILER_LIFETIME_2,
            ],
        ),
        (
            "compiler_bound",
            vec![
                a10::WRONG_COMPILER_BOUND_0,
                // `T: Ord` with `.iter().max()`: E0277 on the float call.
                a10::WRONG_COMPILER_BOUND_1,
                a10::WRONG_COMPILER_BOUND_2,
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
        // Not a literal print: the Wager's last stage needs `?`, and this
        // correct combinator chain has none (it is behaviourally right, so
        // only the `uses` token rejects it).
        (
            "wager_quest",
            concat!(
                quest_items!(),
                "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { read_map(map).map_err(QuestError::from).and_then(|leagues| open_vault(key).map_err(QuestError::from).and_then(|gold| pay_guide(gold, leagues).map_err(QuestError::from))).map(|left| Reward { gold: left }) }\n"
            ),
        ),
        // Not a literal print either: the Golem's last stage needs a
        // `.fold(`/`.sum`, and this correct recursion (no loop, no `mut`)
        // has neither, so only the `uses` token rejects it.
        (
            "golem_core",
            concat!(
                golem_items!(),
                "fn total_damage(parts: &[Part], blow: u32) -> u32 { match parts.split_first() { Some((p, rest)) => (if p.armoured { 0 } else { blow * p.weakness }) + total_damage(rest, blow), None => 0 } }\n",
                golem_main3!()
            ),
        ),
        // ── Act 9. Stdout missions: the lines as literals. Harness
        // missions: behaviour right without the lesson (a direct call for
        // FFI, a `Box` that needs no promise, safe `get`, a seal line with
        // no `from_raw`), so only the `uses` tokens reject them.
        ("raw_pointer", a9::LITERAL_RAW_POINTER),
        ("extern_c", a9::LITERAL_EXTERN_C),
        ("macro_rules", a9::LITERAL_MACRO_RULES),
        ("drop_trait", a9::LITERAL_DROP_TRAIT),
        ("send_sync", a9::LITERAL_SEND_SYNC),
        ("maybe_uninit", a9::LITERAL_MAYBE_UNINIT),
        ("librarian_index", a9::LITERAL_LIBRARIAN_INDEX),
        ("librarian_demand", a9::LITERAL_LIBRARIAN_DEMAND),
        ("librarian_seal", a9::LITERAL_LIBRARIAN_SEAL),
        // ── Act 10: the right lines without the construct; only the `uses`
        // tokens reject them.
        ("prealloc", a10::LITERAL_PREALLOC),
        ("cow_borrow", a10::LITERAL_COW_BORROW),
        ("const_eval", a10::LITERAL_CONST_EVAL),
        ("chunks_exact", a10::LITERAL_CHUNKS_EXACT),
        ("no_std", a10::LITERAL_NO_STD),
        ("semver_caret", a10::LITERAL_SEMVER_CARET),
        ("compiler_alloc", a10::LITERAL_COMPILER_ALLOC),
        ("compiler_lifetime", a10::LITERAL_COMPILER_LIFETIME),
        ("compiler_bound", a10::LITERAL_COMPILER_BOUND),
    ])
}

/// Behaviourally correct programs that use a token their mission rules out
/// ([`expectations::MissionCheck::forbids`]). Each must be
/// `StaticCheckFailed`. Exactly the missions with `forbids` have entries.
fn forbidden_variants() -> HashMap<&'static str, Vec<&'static str>> {
    HashMap::from([
        (
            "wager_quest",
            vec![
                // The nested `match` pyramid the Tavernkeeper grades against.
                concat!(
                    quest_items!(),
                    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { match read_map(map) { Ok(leagues) => match open_vault(key) { Ok(gold) => match pay_guide(gold, leagues) { Ok(left) => Ok(Reward { gold: left }), Err(e) => Err(e.into()) }, Err(e) => Err(e.into()) }, Err(e) => Err(e.into()) } }\n"
                ),
                // A guarded `.unwrap()`: never panics here, still ruled out.
                concat!(
                    quest_items!(),
                    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { if let Err(e) = read_map(map) { return Err(e.into()); } let leagues = read_map(map).unwrap(); let gold = open_vault(key)?; let left = pay_guide(gold, leagues)?; Ok(Reward { gold: left }) }\n"
                ),
                // `.expect(..)` on a step already known to be Ok.
                concat!(
                    quest_items!(),
                    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { let leagues = read_map(map)?; let gold = open_vault(key)?; let left = pay_guide(gold, leagues)?; let _ = read_map(map).expect(\"read twice\"); Ok(Reward { gold: left }) }\n"
                ),
                // A `panic!` on a branch that never runs.
                concat!(
                    quest_items!(),
                    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> { let leagues = read_map(map)?; let gold = open_vault(key)?; let left = pay_guide(gold, leagues)?; if left > 1000 { panic!(\"too rich\"); } Ok(Reward { gold: left }) }\n"
                ),
            ],
        ),
        (
            "golem_core",
            vec![
                // A `for` loop gathers the hits; `.sum()` adds them.
                concat!(
                    golem_items!(),
                    "fn total_damage(parts: &[Part], blow: u32) -> u32 { let mut hits = Vec::new(); for p in parts { if !p.armoured { hits.push(blow * p.weakness); } } hits.iter().sum() }\n",
                    golem_main3!()
                ),
                // The same with `while` and an index.
                concat!(
                    golem_items!(),
                    "fn total_damage(parts: &[Part], blow: u32) -> u32 { let mut i = 0; let mut hits = Vec::new(); while i < parts.len() { if !parts[i].armoured { hits.push(blow * parts[i].weakness); } i += 1; } hits.iter().sum() }\n",
                    golem_main3!()
                ),
                // The same with `loop` and `break`.
                concat!(
                    golem_items!(),
                    "fn total_damage(parts: &[Part], blow: u32) -> u32 { let mut it = parts.iter(); let mut hits = Vec::new(); loop { match it.next() { Some(p) if !p.armoured => hits.push(blow * p.weakness), Some(_) => {} None => break } } hits.iter().sum() }\n",
                    golem_main3!()
                ),
                // A chain whose fold mutates its accumulator: `mut`, nothing else.
                concat!(
                    golem_items!(),
                    "fn total_damage(parts: &[Part], blow: u32) -> u32 { parts.iter().filter(|p| !p.armoured).map(|p| blow * p.weakness).fold(0, |mut t, h| { t += h; t }) }\n",
                    golem_main3!()
                ),
            ],
        ),
        // ── Act 9 `late` missions that rule out `no_mangle` (a way to run
        // code during the checker's late print): correct, plus an export.
        ("send_sync", vec![a9::FORBID_SEND_SYNC]),
        ("librarian_index", vec![a9::FORBID_LIBRARIAN_INDEX]),
        ("librarian_seal", vec![a9::FORBID_LIBRARIAN_SEAL]),
    ])
}

/// Realistic correct solutions that differ from the canonical one. Each
/// must pass (`Passed`; `CompiledNotChecked` for a compile-only mission,
/// which is never run). The game's `tests/contract.rs` `CORRECT` corpus
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
        // The Act 3-5 trials. Each program is also in the game's
        // `tests/contract.rs` `CORRECT` corpus.
        ("wraith_forms", vec![WRAITH_FORMS_ALT]),
        ("wraith_essence", vec![WRAITH_ESSENCE_ALT]),
        ("wraith_possessed", vec![WRAITH_POSSESSED_ALT]),
        ("chimera_heads", vec![CHIMERA_HEADS_ALT]),
        (
            "chimera_bound",
            vec![CHIMERA_BOUND_ALT, CHIMERA_BOUND_WHERE_ALT],
        ),
        ("chimera_plasma", vec![CHIMERA_PLASMA_ALT]),
        ("wager_stake", vec![WAGER_STAKE_ALT]),
        ("wager_from", vec![WAGER_FROM_ALT]),
        ("wager_quest", vec![WAGER_QUEST_ALT]),
        // The Act 6-8 trials, the same way. `hydra_threads` is compile-only
        // here, so its alternatives must be `CompiledNotChecked`.
        ("golem_parts", vec![GOLEM_PARTS_ALT]),
        (
            "golem_sum",
            vec![GOLEM_SUM_TURBOFISH_ALT, GOLEM_SUM_FOLD_ALT],
        ),
        ("golem_core", vec![GOLEM_CORE_SUM_ALT, GOLEM_CORE_FOLD_ALT]),
        (
            "hydra_senders",
            vec![HYDRA_SENDERS_CLONE_ALT, HYDRA_SENDERS_BORROW_ALT],
        ),
        (
            "hydra_threads",
            vec![HYDRA_THREADS_JOIN_ALT, HYDRA_THREADS_BLOCK_ALT],
        ),
        (
            "hydra_body",
            vec![HYDRA_BODY_FOR_ALT, HYDRA_BODY_LET_ELSE_ALT],
        ),
        (
            "warden_box",
            vec![WARDEN_BOX_WHILE_ALT, WARDEN_BOX_IF_LET_ALT],
        ),
        ("warden_shared", vec![WARDEN_SHARED_ALT]),
        (
            "warden_weak",
            vec![WARDEN_WEAK_NAMES_ALT, WARDEN_WEAK_WHILE_ALT],
        ),
        // Act 9: the spec's two alternatives, and one per Librarian stage.
        // The game's `tests/contract.rs` `CORRECT` corpus holds the same.
        ("raw_pointer", vec![a9::ALT_RAW_POINTER_0]),
        ("maybe_uninit", vec![a9::ALT_MAYBE_UNINIT_0]),
        ("librarian_index", vec![a9::ALT_LIBRARIAN_INDEX_0]),
        ("librarian_demand", vec![a9::ALT_LIBRARIAN_DEMAND_0]),
        ("librarian_seal", vec![a9::ALT_LIBRARIAN_SEAL_0]),
        // Act 10: the spec's three alternatives. `collect` and `join` are
        // not promised to allocate once; they measure 1 in the sandbox,
        // which is what this pins.
        ("prealloc", vec![a10::ALT_PREALLOC_0]),
        ("chunks_exact", vec![a10::ALT_CHUNKS_EXACT_0]),
        ("compiler_alloc", vec![a10::ALT_COMPILER_ALLOC_0]),
        ("compiler_lifetime", vec![a10::ALT_COMPILER_LIFETIME_0]),
        ("compiler_bound", vec![a10::ALT_COMPILER_BOUND_0]),
    ])
}

// Alternatives for the Act 6-8 trials, written the way a player might.
// `tests/contract.rs` holds the same programs.

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

// Alternatives for the Act 3-5 trials, written the way a player might.

const WRAITH_FORMS_ALT: &str = "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid,\n    Mist,\n    Shadow,\n}\nfn counter(form: Wraith) -> &'static str {\n    match form {\n        Wraith::Shadow => \"light\",\n        Wraith::Mist => \"wind\",\n        Wraith::Solid => \"steel\",\n    }\n}\nfn main() {\n    for form in [Wraith::Solid, Wraith::Mist, Wraith::Shadow] {\n        println!(\"{:?}: {}\", form, counter(form));\n    }\n}\n";

const WRAITH_ESSENCE_ALT: &str = "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid { armor: u32 },\n    Mist { density: u32 },\n    Shadow,\n}\nfn damage(form: Wraith, blow: u32) -> u32 {\n    match form {\n        Wraith::Solid { armor: a } => blow - a,\n        Wraith::Mist { density: d } => blow / d,\n        Wraith::Shadow => 0,\n    }\n}\nfn main() {\n    let forms = [Wraith::Solid { armor: 3 }, Wraith::Mist { density: 2 }, Wraith::Shadow];\n    for f in forms.iter() {\n        println!(\"{:?} takes {}\", f, damage(*f, 10));\n    }\n}\n";

const WRAITH_POSSESSED_ALT: &str = "#[derive(Debug, Clone, Copy)]\nenum Wraith {\n    Solid { armor: u32 },\n    Mist { density: u32 },\n    Shadow,\n    Possessed { will: u32 },\n}\nfn damage(form: Wraith, blow: u32) -> u32 {\n    match form {\n        Wraith::Possessed { will: w } => w + blow,\n        Wraith::Solid { armor } => blow - armor,\n        Wraith::Mist { density } => blow / density,\n        Wraith::Shadow => 0,\n    }\n}\nfn main() {\n    let forms = [\n        Wraith::Solid { armor: 3 },\n        Wraith::Mist { density: 2 },\n        Wraith::Shadow,\n        Wraith::Possessed { will: 4 },\n    ];\n    for form in forms {\n        println!(\"{form:?} takes {}\", damage(form, 10));\n    }\n}\n";

const CHIMERA_HEADS_ALT: &str = "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nfn main() {\n    let heads: [&dyn Enemy; 3] = [&Fire, &Frost, &Storm];\n    for h in heads {\n        println!(\"{} breathes {}\", h.name(), h.breath());\n    }\n}\n";

const CHIMERA_BOUND_ALT: &str = "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nfn volley(head: &impl Enemy, rounds: u32) -> String {\n    let total = head.breath() * rounds;\n    format!(\"{} x{} = {}\", head.name(), rounds, total)\n}\nfn main() {\n    println!(\"{}\", volley(&Fire, 3));\n    println!(\"{}\", volley(&Frost, 3));\n    println!(\"{}\", volley(&Storm, 3));\n}\n";

const CHIMERA_BOUND_WHERE_ALT: &str = "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nfn volley<H>(head: &H, rounds: u32) -> String\nwhere\n    H: Enemy,\n{\n    format!(\"{} x{rounds} = {}\", head.name(), head.breath() * rounds)\n}\nfn main() {\n    println!(\"{}\", volley(&Fire, 3));\n    println!(\"{}\", volley(&Frost, 3));\n    println!(\"{}\", volley(&Storm, 3));\n}\n";

const CHIMERA_PLASMA_ALT: &str = "trait Enemy {\n    fn name(&self) -> &'static str;\n    fn breath(&self) -> u32;\n}\nstruct Fire;\nstruct Frost;\nstruct Storm;\nstruct Plasma;\nimpl Enemy for Fire {\n    fn name(&self) -> &'static str { \"fire\" }\n    fn breath(&self) -> u32 { 7 }\n}\nimpl Enemy for Frost {\n    fn name(&self) -> &'static str { \"frost\" }\n    fn breath(&self) -> u32 { 5 }\n}\nimpl Enemy for Storm {\n    fn name(&self) -> &'static str { \"storm\" }\n    fn breath(&self) -> u32 { 9 }\n}\nimpl Enemy for Plasma {\n    fn name(&self) -> &'static str { \"plasma\" }\n    fn breath(&self) -> u32 { 12 }\n}\nfn battle(enemies: &[Box<dyn Enemy>]) -> u32 {\n    enemies\n        .iter()\n        .map(|e| {\n            println!(\"{} breathes {}\", e.name(), e.breath());\n            e.breath()\n        })\n        .sum()\n}\nfn main() {\n    let heads: Vec<Box<dyn Enemy>> =\n        vec![Box::new(Fire), Box::new(Frost), Box::new(Storm), Box::new(Plasma)];\n    let total = battle(&heads);\n    println!(\"total: {total}\");\n}\n";

const WAGER_STAKE_ALT: &str = "fn read_stake(text: &str) -> Result<u32, std::num::ParseIntError> {\n    text.trim().parse::<u32>()\n}\nfn double_stake(text: &str) -> Result<u32, std::num::ParseIntError> {\n    Ok(read_stake(text)? * 2)\n}\nfn main() {\n    for stake in [\"20\", \"twenty\"] {\n        println!(\"{:?}\", double_stake(stake));\n    }\n}\n";

const WAGER_FROM_ALT: &str = "#[derive(Debug)]\nstruct Short;\nfn pay(purse: u32, stake: u32) -> Result<u32, Short> {\n    if stake > purse { Err(Short) } else { Ok(purse - stake) }\n}\n#[derive(Debug)]\nenum WagerError {\n    BadStake,\n    TooPoor,\n}\nimpl From<std::num::ParseIntError> for WagerError {\n    fn from(_e: std::num::ParseIntError) -> WagerError {\n        Self::BadStake\n    }\n}\nimpl From<Short> for WagerError {\n    fn from(_e: Short) -> WagerError {\n        Self::TooPoor\n    }\n}\nfn place_bet(text: &str, purse: u32) -> Result<u32, WagerError> {\n    let stake: u32 = text.trim().parse()?;\n    Ok(pay(purse, stake)?)\n}\nfn main() {\n    println!(\"{:?}\", place_bet(\"15\", 50));\n    println!(\"{:?}\", place_bet(\"lots\", 50));\n    println!(\"{:?}\", place_bet(\"80\", 50));\n}\n";

const WAGER_QUEST_ALT: &str = concat!(
    quest_items!(),
    "fn quest(map: &str, key: &str) -> Result<Reward, QuestError> {\n    let leagues = read_map(map)?;\n    Ok(Reward { gold: pay_guide(open_vault(key)?, leagues)? })\n}\n"
);

#[derive(Debug)]
enum Role {
    Canonical,
    Starter,
    Wrong(usize),
    /// A [`literal_prints`] program: must be `StaticCheckFailed`.
    Literal,
    /// An [`alternatives`] program: must be `Passed`.
    Alternative(usize),
    /// A [`forbidden_variants`] program: must be `StaticCheckFailed`.
    /// Counted as a wrong variant.
    Forbidden(usize),
}

impl Role {
    fn label(&self) -> String {
        match self {
            Self::Canonical => "canonical".to_string(),
            Self::Starter => "starter".to_string(),
            Self::Wrong(i) => format!("wrong#{i}"),
            Self::Literal => "literal".to_string(),
            Self::Alternative(i) => format!("alt#{i}"),
            Self::Forbidden(i) => format!("forbid#{i}"),
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
    assert_eq!(sols.len(), 96, "solutions parsed");
    assert_eq!(starters.len(), 96, "starters parsed");
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
    let forbidden = forbidden_variants();
    for m in expectations::all() {
        assert_eq!(
            !m.forbids.is_empty(),
            forbidden.contains_key(m.id),
            "{}: forbidden-token variants must exist exactly for missions with `forbids`",
            m.id
        );
    }
    // The boss-trial stages: at least three wrong variants (forbidden-token
    // variants count) and at least one alternative each. `hydra_threads`
    // is the one compile-only stage (its wrong variants fail the static
    // check, its alternatives are `CompiledNotChecked`).
    let alts = alternatives();
    for t in [
        "bridge_toll",
        "bridge_quill",
        "bridge_lantern",
        "wraith_forms",
        "wraith_essence",
        "wraith_possessed",
        "chimera_heads",
        "chimera_bound",
        "chimera_plasma",
        "wager_stake",
        "wager_from",
        "wager_quest",
        "golem_parts",
        "golem_sum",
        "golem_core",
        "hydra_senders",
        "hydra_threads",
        "hydra_body",
        "warden_box",
        "warden_shared",
        "warden_weak",
        "librarian_index",
        "librarian_demand",
        "librarian_seal",
        "compiler_alloc",
        "compiler_lifetime",
        "compiler_bound",
    ] {
        let n = wrong.get(t).map_or(0, Vec::len) + forbidden.get(t).map_or(0, Vec::len);
        assert!(n >= 3, "{t}: {n} wrong variants");
        assert!(
            alts.get(t).is_some_and(|a| !a.is_empty()),
            "{t}: no alternative"
        );
    }
}

#[test]
#[ignore = "slow: one real wasm build per mission"]
fn print_canonical_transcripts() {
    // Regenerates the recorded data in src/expectations/data.rs: harness
    // results and the stdout each canonical solution prints.
    assert_pinned_toolchain();
    let sols = canonical_solutions();
    let settings = BuildSettings::default();
    for m in expectations::all() {
        let src = &sols[m.id];
        match m.check {
            Check::Harness(h) => {
                // The nonce-in-checker source keeps this dev printout
                // simple; the entry is the mission's own (an Act 9
                // `UnsafeLateNonce` mission's `unsafe` must still build).
                let checker = expectations::harness_module_source(&h, "NONCE");
                let layout = Layout::Harness {
                    entry: expectations::harness_entry_source(m.harness_mode),
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

/// The proof is only a proof for the compiler players get: fail fast,
/// with the fix in the message, if the builds would use any rustc but
/// `toolchain::PINNED_RUSTC_VERSION`.
fn assert_pinned_toolchain() {
    use pledge_compile_api::toolchain::{check_version, probe_rustc};
    let found = probe_rustc(&BuildSettings::default().sandbox)
        .unwrap_or_else(|e| panic!("cannot tell which rustc the builds use: {e}"));
    if let Err(e) = check_version(&found) {
        panic!(
            "{e}\n(run this suite on the pinned toolchain, e.g. `rustup run <pinned> cargo test ...`)"
        );
    }
}

#[test]
#[ignore = "slow: ~200 real wasm builds"]
fn every_expectation_discriminates() {
    assert_pinned_toolchain();
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
        for (i, f) in forbidden_variants()
            .get(m.id)
            .into_iter()
            .flatten()
            .enumerate()
        {
            cases.push(Case {
                id: m.id,
                role: Role::Forbidden(i),
                source: (*f).to_string(),
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
            (Role::Literal | Role::Forbidden(_), _) => g.state == GradeState::StaticCheckFailed,
            // A compile-only mission's correct program builds and meets the
            // static checks; it is never run.
            (Role::Alternative(_), Check::CompileOnly { .. }) => {
                g.state == GradeState::CompiledNotChecked
            }
            (Role::Alternative(_), _) => g.state == GradeState::Passed,
        };
        match case.role {
            Role::Canonical => entry.0 = ok,
            Role::Starter => entry.1 = ok,
            Role::Wrong(_) | Role::Forbidden(_) => {
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
