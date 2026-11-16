// SPDX-License-Identifier: AGPL-3.0-or-later
//! Act 9 programs for `expectations_proof.rs`: wrong variants, literal
//! (lesson-skipping) variants, forbidden-token variants and alternatives.
//! Each was built with rustc 1.99.0, editions 2021 and 2024, native and
//! `wasm32-wasip1`, before it was added. The game's `tests/contract.rs`
//! holds the same alternatives (the two crates may not share code).

#![allow(dead_code)]

pub const WRONG_DROP_TRAIT_0: &str = r##"use std::mem;

struct Soul {
    name: &'static str,
}

impl Drop for Soul {
    fn drop(&mut self) {
        println!("{} is released", self.name);
    }
}

fn main() {
    let _first = Soul { name: "first" };
    let _second = Soul { name: "second" };
    let early = Soul { name: "early" };
    let bound = Soul { name: "bound" };

    // Forgets the wrong soul and drops the one that should be kept.
    mem::forget(early);
    println!("the ritual begins");
    let _ = Soul { name: "unnamed" };
    {
        let _inner = Soul { name: "inner" };
        println!("inner scope ends");
    }
    drop(bound);
    println!("main ends");
}
"##;

pub const WRONG_EXTERN_C_0: &str = r##"#[derive(Clone, Copy)]
pub struct Rune {
    pub glyph: u8,
    pub power: u32,
    pub tier: u8,
}
#[unsafe(no_mangle)]
pub extern "C" fn crown_seal(rune: Rune) -> u32 {
    rune.power * 2 + rune.tier as u32 + rune.glyph as u32
}
mod foreign {
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
    println!("direct call: {}", crown_seal(rune));
    let sealed = unsafe { foreign::crown_seal(rune) };
    println!("through the extern block: {sealed}");
    let callback: extern "C" fn(Rune) -> u32 = crown_seal;
    println!("through a fn pointer: {}", callback(rune));
}
"##;

pub const WRONG_EXTERN_C_1: &str = r##"// The wrong formula: power counted three times.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rune {
    pub glyph: u8,
    pub power: u32,
    pub tier: u8,
}
#[unsafe(no_mangle)]
pub extern "C" fn crown_seal(rune: Rune) -> u32 {
    rune.power * 3 + rune.tier as u32 + rune.glyph as u32
}
mod foreign {
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
    println!("direct call: {}", crown_seal(rune));
    let sealed = unsafe { foreign::crown_seal(rune) };
    println!("through the extern block: {sealed}");
    let callback: extern "C" fn(Rune) -> u32 = crown_seal;
    println!("through a fn pointer: {}", callback(rune));
}
"##;

pub const WRONG_EXTERN_C_2: &str = r##"// Right for the rune `main` seals, wrong for any other.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rune {
    pub glyph: u8,
    pub power: u32,
    pub tier: u8,
}
#[unsafe(no_mangle)]
pub extern "C" fn crown_seal(rune: Rune) -> u32 {
    if rune.glyph == 3 {
        rune.power * 2 + rune.tier as u32 + rune.glyph as u32
    } else {
        rune.power * 2
    }
}
mod foreign {
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
    println!("direct call: {}", crown_seal(rune));
    let sealed = unsafe { foreign::crown_seal(rune) };
    println!("through the extern block: {sealed}");
    let callback: extern "C" fn(Rune) -> u32 = crown_seal;
    println!("through a fn pointer: {}", callback(rune));
}
"##;

pub const WRONG_LIBRARIAN_DEMAND_0: &str = r##"fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// Folds from the right: seal(page(a), seal(page(b), seal(page(c),
// page(d)))). Compiles, and is not what the Librarian demanded.
macro_rules! incant {
    ($last:expr) => { page($last) };
    ($first:expr, $($rest:expr),+) => {
        seal(page($first), incant!($($rest),+))
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
"##;

pub const WRONG_LIBRARIAN_DEMAND_1: &str = r##"fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// Swaps the arguments: seal(page(next), acc) instead of seal(acc, ..).
macro_rules! incant {
    (@acc $acc:expr) => { $acc };
    (@acc $acc:expr, $next:expr $(, $rest:expr)*) => {
        incant!(@acc seal(page($next), $acc) $(, $rest)*)
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
"##;

pub const WRONG_LIBRARIAN_DEMAND_2: &str = r##"fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// Forgets `page` on every verse after the first.
macro_rules! incant {
    (@acc $acc:expr) => { $acc };
    (@acc $acc:expr, $next:expr $(, $rest:expr)*) => {
        incant!(@acc seal($acc, $next) $(, $rest)*)
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
"##;

pub const WRONG_LIBRARIAN_INDEX_0: &str = r##"unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    unsafe { *shelf.add(index) }
}

// Sound (it never reads out of bounds) but too strict: it refuses the
// last real volume.
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    if index + 1 < shelf.len() {
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
"##;

pub const WRONG_LIBRARIAN_INDEX_1: &str = r##"// Reads the shelf back to front (bounds-checked, sound, wrong).
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    unsafe { *shelf.add(index) }
}
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    if index < shelf.len() {
        Some(unsafe { volume_at(shelf.as_ptr(), shelf.len() - 1 - index) })
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
"##;

pub const WRONG_LIBRARIAN_INDEX_2: &str = r##"// Right for the shelf `main` uses, wrong for a one-volume shelf.
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    unsafe { *shelf.add(index) }
}
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    if index < shelf.len() {
        let bump = if shelf.len() == 1 { 2 } else { 0 };
        Some(unsafe { volume_at(shelf.as_ptr(), index) } + bump)
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
"##;

pub const WRONG_LIBRARIAN_SEAL_0: &str = r##"// A correct `get`, but no Drop: the three pages leak and the codex is
// never sealed. Leaking is not undefined behaviour, so this compiles and
// runs cleanly. It is still wrong.
struct Codex {
    ptr: *mut u32,
    len: usize,
}

impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        let boxed: Box<[u32]> = pages.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast::<u32>();
        Codex { ptr, len }
    }

    fn get(&self, index: usize) -> Option<u32> {
        if index < self.len {
            Some(unsafe { *self.ptr.add(index) })
        } else {
            None
        }
    }
}

fn main() {
    let codex = Codex::from_pages(vec![5, 8, 13]);
    println!("page 0: {:?}", codex.get(0));
    println!("page 2: {:?}", codex.get(2));
    println!("page 3: {:?}", codex.get(3));
    println!("the Librarian closes the book");
}
"##;

pub const WRONG_LIBRARIAN_SEAL_1: &str = r##"// `get` hands back the index, not the page.
struct Codex {
    ptr: *mut u32,
    len: usize,
}
impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        let boxed: Box<[u32]> = pages.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast::<u32>();
        Codex { ptr, len }
    }
    fn get(&self, index: usize) -> Option<u32> {
        if index < self.len { Some(index as u32) } else { None }
    }
}
impl Drop for Codex {
    fn drop(&mut self) {
        let slice = std::ptr::slice_from_raw_parts_mut(self.ptr, self.len);
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
"##;

pub const WRONG_LIBRARIAN_SEAL_2: &str = r##"// Right for the three-page codex `main` builds, wrong for four pages.
struct Codex {
    ptr: *mut u32,
    len: usize,
}
impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        let boxed: Box<[u32]> = pages.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast::<u32>();
        Codex { ptr, len }
    }
    fn get(&self, index: usize) -> Option<u32> {
        if index < self.len {
            let twice = if self.len == 4 { 2 } else { 1 };
            Some(unsafe { *self.ptr.add(index) } * twice)
        } else {
            None
        }
    }
}
impl Drop for Codex {
    fn drop(&mut self) {
        let slice = std::ptr::slice_from_raw_parts_mut(self.ptr, self.len);
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
"##;

pub const WRONG_MACRO_RULES_0: &str = r##"// Matches the whole list but only ever uses the first verse.
macro_rules! chant {
    ($first:expr $(, $rest:expr)* $(,)?) => {{
        let verse = $first;
        println!("  verse: {verse}");
        verse
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
"##;

pub const WRONG_MAYBE_UNINIT_0: &str = r##"use std::mem::MaybeUninit;

const PAGES: usize = 4;

fn main() {
    // zeroed() makes every page a valid 0, so skipping the last write is
    // not undefined behaviour here. It is still the wrong book.
    let mut quire = MaybeUninit::<[u32; PAGES]>::zeroed();
    let first: *mut u32 = quire.as_mut_ptr().cast::<u32>();
    for i in 0..PAGES - 1 {
        unsafe { first.add(i).write((i as u32 + 1) * 10) };
    }
    let quire: [u32; PAGES] = unsafe { quire.assume_init() };
    println!("bound pages: {quire:?}");
    println!("total ink: {}", quire.iter().sum::<u32>());
}
"##;

pub const WRONG_MAYBE_UNINIT_1: &str = r##"// Pages 0, 10, 20, 30: the formula is off by one page.
use std::mem::MaybeUninit;
const PAGES: usize = 4;
fn main() {
    let mut quire = MaybeUninit::<[u32; PAGES]>::uninit();
    let first: *mut u32 = quire.as_mut_ptr().cast::<u32>();
    for i in 0..PAGES {
        unsafe { first.add(i).write(i as u32 * 10) };
    }
    let quire: [u32; PAGES] = unsafe { quire.assume_init() };
    println!("bound pages: {quire:?}");
    println!("total ink: {}", quire.iter().sum::<u32>());
}
"##;

pub const WRONG_RAW_POINTER_0: &str = r##"fn main() {
    let gold: i32 = 10;
    // Points at a copy, so the write never reaches `gold`.
    let mut copy = gold;
    let write_ptr: *mut i32 = &mut copy;
    unsafe {
        *write_ptr += 5;
    }
    let read_ptr: *const i32 = &gold;
    let seen = unsafe { *read_ptr };
    println!("pointer is null: {}", read_ptr.is_null());
    println!("gold through the pointer: {seen}");
    println!("gold by name: {gold}");
}
"##;

pub const WRONG_SEND_SYNC_0: &str = r##"// Makes both promises but forgets to free the page: it leaks, and the
// shelf never gets its page back.
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
        unsafe { *self.page }
    }
}

unsafe impl Send for Tome {}
unsafe impl Sync for Tome {}

fn courier_accepts<T: Send>(item: T) -> T {
    item
}

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
"##;

pub const WRONG_SEND_SYNC_1: &str = r##"// Only `Send`: the checker's `Sync` bound does not compile.
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
        unsafe { *self.page }
    }
}
impl Drop for Tome {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.page)) };
        println!("the page is returned to the shelf");
    }
}
unsafe impl Send for Tome {}
fn courier_accepts<T: Send>(item: T) -> T {
    item
}
fn main() {
    let tome = Tome::new(7);
    let carried = courier_accepts(tome);
    println!("the courier accepts the tome");
    println!("page reads {}", carried.read());
}
"##;

pub const WRONG_SEND_SYNC_2: &str = r##"// Right for the page `main` stores, wrong for any other.
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
        let v = unsafe { *self.page };
        if v == 7 { v } else { v + 1 }
    }
}
impl Drop for Tome {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.page)) };
        println!("the page is returned to the shelf");
    }
}
unsafe impl Send for Tome {}
unsafe impl Sync for Tome {}
fn courier_accepts<T: Send>(item: T) -> T {
    item
}
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
"##;

pub const LITERAL_DROP_TRAIT: &str = r##"fn main() {
    for line in [
        "early is released",
        "the ritual begins",
        "unnamed is released",
        "inner scope ends",
        "inner is released",
        "main ends",
        "second is released",
        "first is released",
    ] {
        println!("{line}");
    }
}
"##;

pub const LITERAL_EXTERN_C: &str = r##"// Right behaviour, but never calls through a foreign declaration.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rune {
    pub glyph: u8,
    pub power: u32,
    pub tier: u8,
}
#[unsafe(no_mangle)]
pub extern "C" fn crown_seal(rune: Rune) -> u32 {
    rune.power * 2 + rune.tier as u32 + rune.glyph as u32
}
fn main() {
    let rune = Rune { glyph: 3, power: 20, tier: 1 };
    println!(
        "Rune: {} bytes, align {}",
        std::mem::size_of::<Rune>(),
        std::mem::align_of::<Rune>()
    );
    println!("direct call: {}", crown_seal(rune));
    println!("through the extern block: {}", crown_seal(rune));
    let callback: extern "C" fn(Rune) -> u32 = crown_seal;
    println!("through a fn pointer: {}", callback(rune));
}
"##;

pub const LITERAL_LIBRARIAN_DEMAND: &str = r##"fn main() {
    println!("the demand: 78");
    println!("incant!(1, 2, 3, 4): 78");
    println!("the incantation matches: true");
    println!("incant!(7): 21");
    println!("incant!(2, 9): 39");
}
"##;

pub const LITERAL_LIBRARIAN_INDEX: &str = r##"// Right behaviour from safe `get`: no `unsafe fn`, no lesson.
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    shelf.get(index).copied()
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
"##;

pub const LITERAL_LIBRARIAN_SEAL: &str = r##"// Prints the seal line but never calls `from_raw`: the pages leak.
struct Codex {
    ptr: *mut u32,
    len: usize,
}
impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        let boxed: Box<[u32]> = pages.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast::<u32>();
        Codex { ptr, len }
    }
    fn get(&self, index: usize) -> Option<u32> {
        if index < self.len {
            Some(unsafe { *self.ptr.add(index) })
        } else {
            None
        }
    }
}
impl Drop for Codex {
    fn drop(&mut self) {
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
"##;

pub const LITERAL_MACRO_RULES: &str = r##"fn main() {
    for (verses, total) in [(&[1][..], 1), (&[2, 3, 4][..], 9), (&[10, 20][..], 30)] {
        for v in verses {
            println!("  verse: {v}");
        }
        let label = match total {
            1 => "first",
            9 => "second",
            _ => "third",
        };
        println!("{label} chant: {total}");
    }
}
"##;

pub const LITERAL_MAYBE_UNINIT: &str = r##"// Right output from a plain array: no MaybeUninit at all.
const PAGES: usize = 4;
fn main() {
    let mut quire = [0u32; PAGES];
    for i in 0..PAGES {
        quire[i] = (i as u32 + 1) * 10;
    }
    println!("bound pages: {quire:?}");
    println!("total ink: {}", quire.iter().sum::<u32>());
}
"##;

pub const LITERAL_RAW_POINTER: &str = r##"fn main() {
    println!("pointer is null: false");
    println!("gold through the pointer: 15");
    println!("gold by name: 15");
}
"##;

pub const LITERAL_SEND_SYNC: &str = r##"// Right behaviour with a Box, which is Send and Sync on its own: no
// promise is made, so the lesson is not used.
struct Tome {
    page: Box<u32>,
}
impl Tome {
    fn new(value: u32) -> Self {
        Tome {
            page: Box::new(value),
        }
    }
    fn read(&self) -> u32 {
        *self.page
    }
}
impl Drop for Tome {
    fn drop(&mut self) {
        println!("the page is returned to the shelf");
    }
}
fn courier_accepts<T: Send>(item: T) -> T {
    item
}
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
"##;

pub const FORBID_LIBRARIAN_INDEX: &str = r##"// Correct, but exports a `no_mangle` symbol, which this mission rules out.
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    unsafe { *shelf.add(index) }
}
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    if index < shelf.len() {
        Some(unsafe { volume_at(shelf.as_ptr(), index) })
    } else {
        None
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn shelf_probe() -> u32 {
    1
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
"##;

pub const FORBID_LIBRARIAN_SEAL: &str = r##"// Correct, but exports a `no_mangle` symbol, which this mission rules out.
struct Codex {
    ptr: *mut u32,
    len: usize,
}
impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        let boxed: Box<[u32]> = pages.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast::<u32>();
        Codex { ptr, len }
    }
    fn get(&self, index: usize) -> Option<u32> {
        if index < self.len {
            Some(unsafe { *self.ptr.add(index) })
        } else {
            None
        }
    }
}
impl Drop for Codex {
    fn drop(&mut self) {
        let slice = std::ptr::slice_from_raw_parts_mut(self.ptr, self.len);
        drop(unsafe { Box::from_raw(slice) });
        println!("the codex is sealed ({} pages returned)", self.len);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn codex_probe() -> u32 {
    1
}
fn main() {
    let codex = Codex::from_pages(vec![5, 8, 13]);
    println!("page 0: {:?}", codex.get(0));
    println!("page 2: {:?}", codex.get(2));
    println!("page 3: {:?}", codex.get(3));
    println!("the Librarian closes the book");
}
"##;

pub const FORBID_SEND_SYNC: &str = r##"// Correct, but exports a `no_mangle` symbol, which this mission rules out.
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
        unsafe { *self.page }
    }
}
impl Drop for Tome {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.page)) };
        println!("the page is returned to the shelf");
    }
}
unsafe impl Send for Tome {}
unsafe impl Sync for Tome {}
#[unsafe(no_mangle)]
pub extern "C" fn courier_probe() -> u32 {
    1
}
fn courier_accepts<T: Send>(item: T) -> T {
    item
}
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
"##;

pub const ALT_LIBRARIAN_DEMAND_0: &str = r##"fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// The same push-down fold, with a `;` between the running result and
// the verses still to come.
macro_rules! incant {
    (@fold $acc:expr ;) => {
        $acc
    };
    (@fold $acc:expr ; $next:expr $(, $rest:expr)*) => {
        incant!(@fold seal($acc, page($next)) ; $($rest),*)
    };
    ($first:expr $(, $rest:expr)* $(,)?) => {
        incant!(@fold page($first) ; $($rest),*)
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
"##;

pub const ALT_LIBRARIAN_INDEX_0: &str = r##"/// # Safety
///
/// `index` must be less than the length of the shelf `shelf` points at.
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    unsafe { *shelf.add(index) }
}

fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    if index >= shelf.len() {
        return None;
    }
    let volume = unsafe { volume_at(shelf.as_ptr(), index) };
    Some(volume)
}

fn main() {
    let shelf = [11, 22, 33, 44];
    for index in [0, 3, 4, 99] {
        if let Some(volume) = fetch(&shelf, index) {
            println!("index {index}: volume {volume}");
        } else {
            println!("index {index}: no such volume");
        }
    }
}
"##;

pub const ALT_LIBRARIAN_SEAL_0: &str = r##"// Keeps the whole slice pointer (address and length) in one field.
struct Codex {
    pages: *mut [u32],
}

impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        Codex {
            pages: Box::into_raw(pages.into_boxed_slice()),
        }
    }

    fn get(&self, index: usize) -> Option<u32> {
        if index >= self.pages.len() {
            return None;
        }
        Some(unsafe { *self.pages.cast::<u32>().add(index) })
    }
}

impl Drop for Codex {
    fn drop(&mut self) {
        let returned = self.pages.len();
        unsafe { drop(Box::from_raw(self.pages)) };
        println!("the codex is sealed ({returned} pages returned)");
    }
}

fn main() {
    let codex = Codex::from_pages(vec![5, 8, 13]);
    println!("page 0: {:?}", codex.get(0));
    println!("page 2: {:?}", codex.get(2));
    println!("page 3: {:?}", codex.get(3));
    println!("the Librarian closes the book");
}
"##;

pub const ALT_MAYBE_UNINIT_0: &str = r##"use std::mem::MaybeUninit;

const PAGES: usize = 4;

fn main() {
    // Alternative: an array of MaybeUninit slots, each written with the
    // safe MaybeUninit::write, then reinterpreted as [u32; 4].
    let mut slots = [const { MaybeUninit::<u32>::uninit() }; PAGES];
    for (i, slot) in slots.iter_mut().enumerate() {
        slot.write((i as u32 + 1) * 10);
    }
    // SAFETY: every slot was written above, and [MaybeUninit<u32>; 4] has
    // the same size and layout as [u32; 4].
    let quire: [u32; PAGES] = unsafe { std::mem::transmute(slots) };
    println!("bound pages: {quire:?}");
    println!("total ink: {}", quire.iter().sum::<u32>());
}
"##;

pub const ALT_RAW_POINTER_0: &str = r##"fn main() {
    let mut gold: i32 = 10;
    // Alternative: raw-borrow syntax creates the pointers without ever
    // making an intermediate reference.
    let write_ptr = &raw mut gold;
    unsafe {
        *write_ptr += 5;
    }
    let read_ptr = &raw const gold;
    let seen = unsafe { *read_ptr };
    println!("pointer is null: {}", read_ptr.is_null());
    println!("gold through the pointer: {seen}");
    println!("gold by name: {gold}");
}
"##;
