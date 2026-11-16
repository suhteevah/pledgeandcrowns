// SPDX-License-Identifier: AGPL-3.0-or-later
//! Act 10 programs for `expectations_proof.rs`: wrong variants, literal
//! (lesson-skipping) variants and alternatives. Each was built with rustc
//! 1.99.0, editions 2021 and 2024, native and `wasm32-wasip1`, before it
//! was added. The game's `tests/contract.rs` holds the same alternatives
//! (the two crates may not share code). The allocator programs' `unsafe`
//! and `#[global_allocator]` are player code inside these strings.

#![allow(dead_code)]

pub const ALT_PREALLOC_0: &str = r##"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

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
    // Alternative: collect a range, whose length is known up front.
    let ledger: Vec<u32> = (0..1000u32).collect();
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    println!("coins: {}", ledger.len());
    println!("allocations: {}", after - before);
}
"##;

pub const ALT_CHUNKS_EXACT_0: &str = r##"// Alternative: chunks(4) yields a short last chunk, so there is no
// remainder step.
fn choir_sum(voices: &[u32]) -> u32 {
    let mut lanes = [0u32; 4];
    for chunk in voices.chunks(4) {
        for (lane, voice) in lanes.iter_mut().zip(chunk) {
            *lane += voice;
        }
    }
    lanes.iter().sum::<u32>()
}

fn main() {
    let voices: Vec<u32> = (1..=10).collect();
    println!("choir sum: {}", choir_sum(&voices));
    println!("plain sum: {}", voices.iter().sum::<u32>());
    println!("empty choir: {}", choir_sum(&[]));
}
"##;

pub const ALT_COMPILER_ALLOC_0: &str = r##"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

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

// Alternative: the standard library's join.
fn proclaim(names: &[&str]) -> String {
    names.join(", ")
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
"##;

pub const ALT_COMPILER_LIFETIME_0: &str = r##"use std::borrow::Cow;

// Alternative: any lifetime name works; a `match` on the title.
fn herald<'n>(name: &'n str, title: &str) -> Cow<'n, str> {
    match title {
        "" => Cow::Borrowed(name),
        t => Cow::Owned(format!("{t} {name}")),
    }
}

fn main() {
    let name = String::from("Ferris");
    let plain = herald(&name, "");
    let titled = {
        let title = String::from("Sir");
        herald(&name, &title)
    };
    for line in [&plain, &titled] {
        let how = match line {
            Cow::Borrowed(_) => "borrowed",
            Cow::Owned(_) => "owned",
        };
        println!("{line} ({how})");
    }
}
"##;

pub const ALT_COMPILER_BOUND_0: &str = r##"// Alternative: the bound in a `where` clause, and a fold for the loop.
fn strongest<T>(champions: &[T]) -> Option<&T>
where
    T: PartialOrd,
{
    champions.iter().fold(None, |best, champion| match best {
        Some(b) if b >= champion => Some(b),
        _ => Some(champion),
    })
}

fn main() {
    println!("{:?}", strongest(&[3, 9, 4]));
    println!("{:?}", strongest(&[0.5, 2.5, 1.0]));
    println!("{:?}", strongest(&["ferris", "vexis", "borrow"]));
    println!("{:?}", strongest::<i32>(&[]));
}
"##;

pub const WRONG_PREALLOC_0: &str = r##"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

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
    // Preallocates, but far too little: the Vec still has to regrow.
    let mut ledger = Vec::with_capacity(10);
    for coin in 0..1000u32 {
        ledger.push(coin);
    }
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    println!("coins: {}", ledger.len());
    println!("allocations: {}", after - before);
}
"##;

pub const WRONG_COW_BORROW_0: &str = r##"use std::borrow::Cow;

// Has the right type but always copies, so it saves nothing.
fn shelf_name(title: &str) -> Cow<'_, str> {
    Cow::Owned(title.replace(' ', "_"))
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
"##;

pub const WRONG_CONST_EVAL_0: &str = r##"// Off by one in the loop bound, and no compile-time check to catch it.
const fn star_table() -> [u32; 8] {
    let mut table = [0u32; 8];
    let mut i = 0;
    while i < 7 {
        table[i] = (i as u32) * (i as u32) + 1;
        i += 1;
    }
    table
}

const STARS: [u32; 8] = star_table();

fn main() {
    println!("star chart: {STARS:?}");
    println!("brightest: {}", STARS[7]);
}
"##;

pub const WRONG_CHUNKS_EXACT_0: &str = r##"fn choir_sum(voices: &[u32]) -> u32 {
    let mut lanes = [0u32; 4];
    let mut chunks = voices.chunks_exact(4);
    for chunk in &mut chunks {
        for (lane, voice) in lanes.iter_mut().zip(chunk) {
            *lane += voice;
        }
    }
    // Counts the soloists instead of adding their voices.
    let soloists = chunks.remainder().len() as u32;
    lanes.iter().sum::<u32>() + soloists
}

fn main() {
    let voices: Vec<u32> = (1..=10).collect();
    println!("choir sum: {}", choir_sum(&voices));
    println!("plain sum: {}", voices.iter().sum::<u32>());
    println!("empty choir: {}", choir_sum(&[]));
}
"##;

pub const WRONG_NO_STD_0: &str = r##"#![no_std]

extern crate std as host;

mod firmware {
    use core::fmt::{self, Write};

    pub struct Beacon {
        bytes: [u8; 32],
        len: usize,
    }

    impl Beacon {
        pub const fn new() -> Self {
            Beacon { bytes: [0; 32], len: 0 }
        }

        pub fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("<garbled>")
        }
    }

    impl Write for Beacon {
        // Silently drops whatever does not fit and reports success.
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for &b in s.as_bytes() {
                if self.len < self.bytes.len() {
                    self.bytes[self.len] = b;
                    self.len += 1;
                }
            }
            Ok(())
        }
    }

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
"##;

pub const WRONG_SEMVER_CARET_0: &str = r##"#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

// Handles 0.x but forgets that ^0.0.z pins the patch as well.
fn caret_allows(required: Version, candidate: Version) -> bool {
    if candidate < required {
        return false;
    }
    if required.major > 0 {
        candidate.major == required.major
    } else {
        candidate.major == 0 && candidate.minor == required.minor
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
"##;

pub const WRONG_COMPILER_ALLOC_0: &str = r##"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

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

// Reserves room for the letters but forgets the separators, so the
// String has to grow once it runs out.
fn proclaim(names: &[&str]) -> String {
    let letters: usize = names.iter().map(|name| name.len()).sum();
    let mut line = String::with_capacity(letters);
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            line.push_str(", ");
        }
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
"##;

pub const WRONG_COMPILER_LIFETIME_0: &str = r##"use std::borrow::Cow;

// Compiles, but copies even when nothing changes.
fn herald(name: &str, title: &str) -> Cow<'static, str> {
    if title.is_empty() {
        Cow::Owned(name.to_string())
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
    };
    for line in [&plain, &titled] {
        let how = match line {
            Cow::Borrowed(_) => "borrowed",
            Cow::Owned(_) => "owned",
        };
        println!("{line} ({how})");
    }
}
"##;

pub const WRONG_COMPILER_BOUND_0: &str = r##"// Right bound, wrong comparison: this crowns the weakest.
fn strongest<T: PartialOrd>(champions: &[T]) -> Option<&T> {
    let mut best = champions.first()?;
    for champion in champions {
        if champion < best {
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
"##;

pub const WRONG_COMPILER_ALLOC_1: &str = r##"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

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

// Sizes the String exactly, then throws it away: every `format!` still
// builds a new one.
fn proclaim(names: &[&str]) -> String {
    let letters: usize = names.iter().map(|name| name.len()).sum();
    let separators = names.len().saturating_sub(1) * ", ".len();
    let mut line = String::with_capacity(letters + separators);
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            line = format!("{line}, ");
        }
        line = format!("{line}{name}");
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
"##;

pub const WRONG_COMPILER_ALLOC_2: &str = r##"use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The Assayer's scale: wraps the system allocator and counts every
/// allocation and reallocation.
struct CountingAlloc;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

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

// One allocation, but the wrong separator: the proclamation changed.
fn proclaim(names: &[&str]) -> String {
    names.join(" ")
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
"##;

pub const WRONG_COMPILER_LIFETIME_1: &str = r##"use std::borrow::Cow;

// Follows rustc's E0106 suggestion literally and ties BOTH inputs to the
// result. The signature now claims the result may borrow from `title`.
fn herald<'a>(name: &'a str, title: &'a str) -> Cow<'a, str> {
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
    };
    for line in [&plain, &titled] {
        let how = match line {
            Cow::Borrowed(_) => "borrowed",
            Cow::Owned(_) => "owned",
        };
        println!("{line} ({how})");
    }
}
"##;

pub const WRONG_COMPILER_LIFETIME_2: &str = r##"use std::borrow::Cow;

// The honest signature, but it copies the name anyway.
fn herald<'a>(name: &'a str, title: &str) -> Cow<'a, str> {
    if title.is_empty() {
        Cow::Owned(name.to_string())
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
    };
    for line in [&plain, &titled] {
        let how = match line {
            Cow::Borrowed(_) => "borrowed",
            Cow::Owned(_) => "owned",
        };
        println!("{line} ({how})");
    }
}
"##;

pub const WRONG_COMPILER_BOUND_1: &str = r##"// Reaches for Ord: fine for integers and strings, refused for floats.
fn strongest<T: Ord>(champions: &[T]) -> Option<&T> {
    champions.iter().max()
}

fn main() {
    println!("{:?}", strongest(&[3, 9, 4]));
    println!("{:?}", strongest(&[0.5, 2.5, 1.0]));
    println!("{:?}", strongest(&["ferris", "vexis", "borrow"]));
    println!("{:?}", strongest::<i32>(&[]));
}
"##;

pub const WRONG_COMPILER_BOUND_2: &str = r##"// Right bound, but it hands back the last champion, not the best.
fn strongest<T: PartialOrd>(champions: &[T]) -> Option<&T> {
    let mut best = champions.first()?;
    for champion in champions {
        if champion > best {
            best = champion;
        }
    }
    let _ = best;
    champions.last()
}

fn main() {
    println!("{:?}", strongest(&[3, 9, 4]));
    println!("{:?}", strongest(&[0.5, 2.5, 1.0]));
    println!("{:?}", strongest(&["ferris", "vexis", "borrow"]));
    println!("{:?}", strongest::<i32>(&[]));
}
"##;

pub const LITERAL_PREALLOC: &str = r##"// Prints the scale's reading without building or measuring anything.
fn main() {
    println!("coins: {}", 1000);
    println!("allocations: {}", 1);
}
"##;

pub const LITERAL_COW_BORROW: &str = r##"// Prints the labels by hand: every name is still a fresh String.
fn shelf_name(title: &str) -> String {
    title.replace(' ', "_")
}

fn main() {
    for title in ["Grimoire", "Book of Crowns", "Ledger"] {
        let how = if title.contains(' ') { "copied" } else { "borrowed" };
        println!("{} ({how})", shelf_name(title));
    }
}
"##;

pub const LITERAL_CONST_EVAL: &str = r##"// The chart typed in by hand: nothing is computed by the compiler and
// nothing checks it.
const STARS: [u32; 8] = [1, 2, 5, 10, 17, 26, 37, 50];

fn main() {
    println!("star chart: {STARS:?}");
    println!("brightest: {}", STARS[7]);
}
"##;

pub const LITERAL_CHUNKS_EXACT: &str = r##"// A plain sum: right answer, no lanes at all.
fn choir_sum(voices: &[u32]) -> u32 {
    voices.iter().sum()
}

fn main() {
    let voices: Vec<u32> = (1..=10).collect();
    println!("choir sum: {}", choir_sum(&voices));
    println!("plain sum: {}", voices.iter().sum::<u32>());
    println!("empty choir: {}", choir_sum(&[]));
}
"##;

pub const LITERAL_NO_STD: &str = r##"// Plain std: a String and format!, and the overflow line typed in.
fn signal(tower: u8, lit: bool) -> String {
    format!("tower {tower}: {}", if lit { "lit" } else { "dark" })
}

fn main() {
    println!("{} (sent: true)", signal(7, true));
    println!("overflow refused: true");
}
"##;

pub const LITERAL_SEMVER_CARET: &str = r##"// The register's answers copied out, no rule at all.
fn main() {
    for line in [
        "^1.2.3 accepts 1.9.0: true",
        "^1.2.3 accepts 2.0.0: false",
        "^1.2.3 accepts 1.2.2: false",
        "^0.2.3 accepts 0.2.9: true",
        "^0.2.3 accepts 0.3.0: false",
        "^0.0.3 accepts 0.0.3: true",
        "^0.0.3 accepts 0.0.4: false",
        "^1.2.3 vs 1.x.0: not a version",
    ] {
        println!("{line}");
    }
}
"##;

pub const LITERAL_COMPILER_ALLOC: &str = r##"// Prints the proclamation and the count the Compiler wants, measuring
// nothing.
fn main() {
    println!("Ferris, the Borrow Checker, Vexis, the Librarian");
    println!("allocations: {}", 1);
}
"##;

pub const LITERAL_COMPILER_LIFETIME: &str = r##"// No `herald` at all: the two lines typed in.
fn main() {
    println!("Ferris (borrowed)");
    println!("Sir Ferris (owned)");
}
"##;

pub const LITERAL_COMPILER_BOUND: &str = r##"// One function per type instead of one generic function with a bound.
fn strongest_int(xs: &[i32]) -> Option<&i32> {
    xs.iter().max()
}

fn main() {
    println!("{:?}", strongest_int(&[3, 9, 4]));
    println!("{:?}", Some(2.5));
    println!("{:?}", ["ferris", "vexis", "borrow"].iter().max());
    println!("{:?}", strongest_int(&[]));
}
"##;
