// SPDX-License-Identifier: AGPL-3.0-or-later
//! `HarnessMode::UnsafeLateNonce` (HARNESS-NOTES.md design 1): the grading
//! mode for Act 9's `unsafe` lessons, proven on test-only fixtures (no Act 9
//! mission is registered yet; the game roster waits on art).
//!
//! Fast tests (always run): the one-shot late-nonce host import, through
//! `wasm_runner` in-process and through the real `run-wasm` child, on
//! hand-written WAT modules.
//!
//! Slow tests (`#[ignore]`d: each case is a real `cargo build` for
//! `wasm32-wasip1` plus a run):
//!
//! - `act9_fixtures_discriminate_and_resist_attacks`: five Act 9 spec
//!   missions as `UnsafeLateNonce` harness checks (`librarian_index`:
//!   `unsafe fn` call + raw-pointer deref; `extern_c`: FFI; `librarian_seal`:
//!   `Box::into_raw`/`from_raw` + raw deref; `send_sync`: `unsafe impl`;
//!   `maybe_uninit`: `MaybeUninit`/`transmute`). For each: the spec's
//!   solution passes, three or more wrong variants fail, and (for the four
//!   whose checker calls player items) a memory-scan attack finds no key, a
//!   guessed-key forgery fails, and an early exit / a panic are graded as
//!   such. Also: timeout, memory cap and output cap still grade as runtime
//!   errors; a late destructor cannot add results after the done mark; a
//!   registered (`ForbidUnsafe`) mission cannot be opted into `unsafe` by
//!   the player's file.
//! - `known_limit_heap_patch_forges_a_pass` and
//!   `known_limit_global_allocator_forges_but_is_caught_statically`: the
//!   strongest attacks found against design 1, kept as documentation of
//!   its known limit (see HARNESS-NOTES.md). They assert what actually
//!   happens.
//!
//! ```text
//! cargo test -p pledge_compile_api --test act9_unsafe_harness -- --include-ignored --nocapture
//! ```

use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use pledge_compile_api::expectations::{
    self, Check, GradeState, GradedRun, Harness, HarnessMode, Matcher, MissionCheck,
};
use pledge_compile_api::run_sandbox::{self, RunChildReply};
use pledge_compile_api::wasm_builder::{BuildSettings, RunLimits};
use pledge_compile_api::wasm_runner::{self, LateNonce};

const EXE: &str = env!("CARGO_BIN_EXE_pledge_compile_api");

// ---- fast: the host import ---------------------------------------------------

/// A module that calls the late-nonce import three times:
/// 1. `(100, len1)` -> rc stored as `'0' + rc` at 300
/// 2. `(200, 35)`   -> rc at 301 (memory 200..235 starts as dots)
/// 3. `(ptr3, 35)`  -> rc at 302
///
/// then prints memory 100..135, 200..235 and 300..303.
fn caller_wat(import: &str, len1: i32, ptr3: i32) -> String {
    format!(
        r#"(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (import "pnc_harness" "{import}" (func $key (param i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 100) "...................................")
  (data (i32.const 200) "...................................")
  (data (i32.const 303) "\n")
  (func $print (param $at i32) (param $len i32)
    (i32.store (i32.const 0) (local.get $at))
    (i32.store (i32.const 4) (local.get $len))
    (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 8))))
  (func (export "_start")
    (i32.store8 (i32.const 300) (i32.add (i32.const 48) (call $key (i32.const 100) (i32.const {len1}))))
    (i32.store8 (i32.const 301) (i32.add (i32.const 48) (call $key (i32.const 200) (i32.const 35))))
    (i32.store8 (i32.const 302) (i32.add (i32.const 48) (call $key (i32.const {ptr3}) (i32.const 35))))
    (call $print (i32.const 100) (i32.const 35))
    (call $print (i32.const 200) (i32.const 35))
    (call $print (i32.const 300) (i32.const 4))))
"#
    )
}

const DOTS: &str = "...................................";

fn run_late(wat: &str, late: Option<&LateNonce>) -> wasm_runner::RunVerdict {
    let bytes = wat::parse_str(wat).expect("wat");
    wasm_runner::run_wasm_late_nonce(&bytes, 5_000, 50_000_000, 16 << 20, late).expect("verdict")
}

#[test]
fn late_nonce_is_written_once_and_only_once() {
    let late = LateNonce::generate();
    let v = run_late(&caller_wat(late.import(), 35, 400), Some(&late));
    assert!(v.ok, "{}", v.stderr);
    // First call: the nonce, rc 0. Second and third: refused (-1, '/'),
    // nothing written.
    assert_eq!(v.stdout, format!("{}{DOTS}0//\n", late.nonce()));
}

#[test]
fn a_bad_first_call_still_consumes_the_nonce() {
    let late = LateNonce::generate();
    // Wrong length: -2 ('.'), and the slot is gone for the honest call.
    let v = run_late(&caller_wat(late.import(), 10, 400), Some(&late));
    assert!(v.ok, "{}", v.stderr);
    assert_eq!(v.stdout, format!("{DOTS}{DOTS}.//\n"));
    assert!(!v.stdout.contains(late.nonce()));
}

#[test]
fn an_out_of_bounds_target_is_refused_without_writing() {
    let late = LateNonce::generate();
    // The FIRST call aims past the end of the 1-page memory.
    let wat = caller_wat(late.import(), 35, 400).replacen(
        "(call $key (i32.const 100) (i32.const 35))",
        "(call $key (i32.const 65530) (i32.const 35))",
        1,
    );
    let v = run_late(&wat, Some(&late));
    assert!(v.ok, "{}", v.stderr);
    // -3 ('-') for the out-of-bounds call, then -1 twice.
    assert_eq!(v.stdout, format!("{DOTS}{DOTS}-//\n"));
}

#[test]
fn without_a_late_nonce_the_import_does_not_exist() {
    let late = LateNonce::generate();
    let v = run_late(&caller_wat(late.import(), 35, 400), None);
    assert!(!v.ok);
    assert!(v.stdout.is_empty());
    assert!(
        v.stderr.contains("pnc_harness") || v.stderr.contains("unknown import"),
        "{}",
        v.stderr
    );
    // A guessed import name does not link either.
    let other = LateNonce::generate();
    let v = run_late(&caller_wat(other.import(), 35, 400), Some(&late));
    assert!(!v.ok);
    assert!(!v.stdout.contains(late.nonce()));
}

#[test]
fn a_malformed_late_nonce_is_a_setup_error() {
    let bytes = wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")))"#)
        .expect("wat");
    let bad: LateNonce =
        serde_json::from_str(r#"{"import":"fd_write","nonce":"PNCx"}"#).expect("serde shape");
    assert!(
        wasm_runner::run_wasm_late_nonce(&bytes, 5_000, 50_000_000, 16 << 20, Some(&bad)).is_err()
    );
}

fn run_child(input: Vec<u8>) -> RunChildReply {
    let mut cmd = Command::new(EXE);
    cmd.arg("run-wasm").env("RUST_LOG", "debug");
    let out = pledge_compile_api::proc_tree::run_captured_with_input(
        cmd,
        Some(input),
        Duration::from_secs(60),
        4 << 20,
        "test run child",
    )
    .expect("supervision");
    assert!(!out.timed_out && out.status.success(), "{out:?}");
    // The child logs at debug here; the nonce must not be in its log.
    assert!(
        !out.stderr.contains("PNC"),
        "child log mentions a nonce: {}",
        out.stderr
    );
    run_sandbox::decode_child_reply(&out.stdout).unwrap_or_else(|e| panic!("{e}: {out:?}"))
}

#[test]
fn run_child_links_the_import_only_when_the_header_carries_a_nonce() {
    let late = LateNonce::generate();
    let bytes = wat::parse_str(caller_wat(late.import(), 35, 400)).expect("wat");
    let limits = RunLimits::default();
    let with = run_sandbox::encode_run_input_with(&bytes, &limits, Some(&late)).expect("encode");
    match run_child(with) {
        RunChildReply::Verdict { ok, stdout, stderr } => {
            assert!(ok, "{stderr}");
            assert_eq!(stdout, format!("{}{DOTS}0//\n", late.nonce()));
        }
        other => panic!("{other:?}"),
    }
    let without = run_sandbox::encode_run_input(&bytes, &limits).expect("encode");
    match run_child(without) {
        RunChildReply::Verdict { ok, stdout, .. } => {
            assert!(!ok);
            assert!(stdout.is_empty());
        }
        other => panic!("{other:?}"),
    }
}

/// The same through the real run sandbox (Linux + bubblewrap).
#[cfg(target_os = "linux")]
mod linux_bwrap {
    use super::*;
    use pledge_compile_api::build_sandbox::{self, RunImage, SandboxMode, SandboxSettings};
    use std::path::PathBuf;

    fn image() -> RunImage {
        let me = std::env::current_exe().expect("current_exe");
        let maps = std::fs::read_to_string("/proc/self/maps").expect("maps");
        let mine = build_sandbox::parse_mapped_objects(&maps, &me).expect("mapped objects");
        RunImage {
            exe: PathBuf::from(EXE),
            ..mine
        }
    }

    fn settings() -> SandboxSettings {
        SandboxSettings {
            mode: SandboxMode::Bwrap,
            toolchain_dir: Some(PathBuf::from("/opt/unused-by-the-run-sandbox")),
            ..SandboxSettings::default()
        }
    }

    #[test]
    #[ignore = "needs Linux + bubblewrap; run on the deploy host"]
    fn late_nonce_crosses_into_the_run_sandbox_once_and_unlogged() {
        let late = LateNonce::generate();
        let bytes = wat::parse_str(caller_wat(late.import(), 35, 400)).expect("wat");
        let limits = RunLimits::default();
        for (with, want_ok) in [(Some(&late), true), (None, false)] {
            let input = run_sandbox::encode_run_input_with(&bytes, &limits, with).expect("encode");
            let env = [("RUST_LOG", std::ffi::OsString::from("debug"))];
            let spec = build_sandbox::run_bwrap_spec(&settings(), &image(), "run-wasm", &env);
            let out = pledge_compile_api::proc_tree::run_captured_with_input(
                spec.to_command(),
                Some(input),
                Duration::from_secs(60),
                4 << 20,
                "test run sandbox",
            )
            .expect("supervision");
            assert!(!out.timed_out && out.status.success(), "{out:?}");
            assert!(
                !out.stderr.contains("PNC"),
                "log mentions a nonce: {}",
                out.stderr
            );
            match run_sandbox::decode_child_reply(&out.stdout).expect("reply") {
                RunChildReply::Verdict { ok, stdout, stderr } => {
                    assert_eq!(ok, want_ok, "{stderr}");
                    if want_ok {
                        assert_eq!(stdout, format!("{}{DOTS}0//\n", late.nonce()));
                    } else {
                        assert!(stdout.is_empty());
                    }
                }
                other => panic!("{other:?}"),
            }
        }
    }
}

// ---- slow: fixtures -------------------------------------------------------------

/// Parallel build workers: `CARGO_BUILD_JOBS` when set, else 3.
fn workers() -> usize {
    std::env::var("CARGO_BUILD_JOBS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(3)
}

/// One Act 9 fixture: an `UnsafeLateNonce` check built from the spec.
struct Fixture {
    check: MissionCheck,
    /// The spec's canonical solution.
    solution: &'static str,
    /// Other sources that must pass (spec alternatives).
    alternatives: &'static [&'static str],
    /// Wrong variants: each must NOT pass.
    wrong: &'static [&'static str],
    /// A wrong variant whose `main` output is right but whose checker
    /// results are not, with `/*HOOK*/` where attack code is called, and
    /// the number of calls to the hooked item over the whole run (main +
    /// checker). `None` for main-only lessons (nothing for the checker to
    /// key, so the key attacks do not apply).
    attack_base: Option<(&'static str, usize)>,
}

const fn late_check(
    id: &'static str,
    harness: Harness,
    uses: &'static [&'static [&'static str]],
    forbids: &'static [&'static str],
) -> MissionCheck {
    MissionCheck {
        id,
        check: Check::Harness(harness),
        harness_mode: HarnessMode::UnsafeLateNonce,
        uses,
        forbids,
    }
}

// -- 64 librarian_index: `unsafe fn` + raw-pointer deref behind a safe door --

const INDEX_SOLUTION: &str = r##"/// Returns the volume `index` places past `shelf`.
///
/// # Safety
///
/// `shelf` must point into a live array of initialised `u32`s, and
/// `shelf.add(index)` must still be inside that same array.
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    // SAFETY: the caller upholds the contract above.
    unsafe { *shelf.add(index) }
}

/// The safe door. No argument a caller can pass makes this read out of
/// bounds, so it does not need to be `unsafe` itself.
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    if index < shelf.len() {
        // SAFETY: index < shelf.len(), and the slice is borrowed (so alive)
        // for the whole call.
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

/// Checker-only wrong: right for the 4-volume shelf `main` uses, adds 2
/// to a one-volume shelf (so `fetch([7], 0)` gives `Some(9)`).
const INDEX_ATTACK_BASE: &str = r##"unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    unsafe { *shelf.add(index) }
}
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    /*HOOK*/
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

const INDEX_EXPECTED: &[&str] = &[
    "fetch(shelf, 0) = Some(11)",
    "fetch(shelf, 3) = Some(44)",
    "fetch(shelf, 4) = None",
    "fetch(shelf, 99) = None",
    "fetch(shelf, usize::MAX) = None",
    "fetch([7], 0) = Some(7)",
    "fetch([7], 1) = None",
    "fetch([], 0) = None",
];

const INDEX_CHECK: MissionCheck = late_check(
    "act9_fixture_librarian_index",
    Harness {
        contract: "`fn fetch(shelf: &[u32], index: usize) -> Option<u32>` and a `fn main`",
        run_main: true,
        body: "    let _sig: fn(&[u32], usize) -> ::core::option::Option<u32> = super::fetch;
    let shelf = [11u32, 22, 33, 44];
    emit(\"fetch(shelf, 0)\", super::fetch(&shelf, 0));
    emit(\"fetch(shelf, 3)\", super::fetch(&shelf, 3));
    emit(\"fetch(shelf, 4)\", super::fetch(&shelf, 4));
    emit(\"fetch(shelf, 99)\", super::fetch(&shelf, 99));
    emit(\"fetch(shelf, usize::MAX)\", super::fetch(&shelf, usize::MAX));
    let one = [7u32];
    emit(\"fetch([7], 0)\", super::fetch(&one, 0));
    emit(\"fetch([7], 1)\", super::fetch(&one, 1));
    emit(\"fetch([], 0)\", super::fetch(&[], 0));",
        expected: INDEX_EXPECTED,
        main_matchers: &[Matcher::Exact(
            "index 0: volume 11\nindex 3: volume 44\nindex 4: no such volume\nindex 99: no such volume",
        )],
    },
    &[&["unsafe fn"], &[".len()"], &["unsafe {", "unsafe{"]],
    &["no_mangle"],
);

// -- 59 extern_c: FFI in one program --

const EXTERN_SOLUTION: &str = r##"/// Laid out by C's rules: fields in declared order, C padding.
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
"##;

const EXTERN_ATTACK_BASE: &str = r##"#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rune {
    pub glyph: u8,
    pub power: u32,
    pub tier: u8,
}
#[unsafe(no_mangle)]
pub extern "C" fn crown_seal(rune: Rune) -> u32 {
    /*HOOK*/
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

const EXTERN_EXPECTED: &[&str] = &[
    "size_of::<Rune>() = 12",
    "align_of::<Rune>() = 4",
    "offset_of!(Rune, tier) = 8",
    "crown_seal(3, 20, 1) = 44",
    "crown_seal(0, 0, 0) = 0",
    "crown_seal(255, 1000, 9) = 2264",
    "through an extern \"C\" fn pointer = 44",
];

const EXTERN_CHECK: MissionCheck = late_check(
    "act9_fixture_extern_c",
    Harness {
        contract: "`#[repr(C)] #[derive(Clone, Copy)] pub struct Rune { pub glyph: u8, pub power: u32, \
                   pub tier: u8 }`, `pub extern \"C\" fn crown_seal(rune: Rune) -> u32`, and a `fn main`",
        run_main: true,
        body: "    let _sig: extern \"C\" fn(super::Rune) -> u32 = super::crown_seal;
    emit(\"size_of::<Rune>()\", ::core::mem::size_of::<super::Rune>());
    emit(\"align_of::<Rune>()\", ::core::mem::align_of::<super::Rune>());
    emit(\"offset_of!(Rune, tier)\", ::core::mem::offset_of!(super::Rune, tier));
    let r = super::Rune { glyph: 3, power: 20, tier: 1 };
    emit(\"crown_seal(3, 20, 1)\", super::crown_seal(r));
    emit(\"crown_seal(0, 0, 0)\", super::crown_seal(super::Rune { glyph: 0, power: 0, tier: 0 }));
    emit(\"crown_seal(255, 1000, 9)\", super::crown_seal(super::Rune { glyph: 255, power: 1000, tier: 9 }));
    let f: extern \"C\" fn(super::Rune) -> u32 = super::crown_seal;
    emit(\"through an extern \\\"C\\\" fn pointer\", f(r));",
        expected: EXTERN_EXPECTED,
        main_matchers: &[Matcher::Exact(
            "Rune: 12 bytes, align 4\ndirect call: 44\nthrough the extern block: 44\nthrough a fn pointer: 44",
        )],
    },
    // String literal contents are blanked before token checks, so
    // `extern "C" fn` cannot be a token; `unsafe extern` (the foreign
    // block) and `no_mangle` stand for the two halves of the lesson.
    &[
        &["repr(C)"],
        &["no_mangle"],
        &["unsafe extern"],
        &["unsafe {", "unsafe{"],
    ],
    // `no_mangle` is the lesson here, so it cannot be forbidden: symbol
    // interposition stays open for this mission (HARNESS-NOTES.md).
    &[],
);

// -- 66 librarian_seal: an owning raw pointer --

const SEAL_SOLUTION: &str = r##"/// Owns a run of `len` u32s through a raw pointer, the way `Vec` does
/// underneath. The raw pointer itself owns nothing: Drop must free it.
struct Codex {
    ptr: *mut u32,
    len: usize,
}

impl Codex {
    fn from_pages(pages: Vec<u32>) -> Self {
        let boxed: Box<[u32]> = pages.into_boxed_slice();
        let len = boxed.len();
        // Box::into_raw gives up ownership: nothing frees this until we
        // rebuild the Box in Drop.
        let ptr = Box::into_raw(boxed).cast::<u32>();
        Codex { ptr, len }
    }

    fn get(&self, index: usize) -> Option<u32> {
        if index < self.len {
            // SAFETY: index < len, and ptr points at len initialised u32s
            // that stay alive until Drop.
            Some(unsafe { *self.ptr.add(index) })
        } else {
            None
        }
    }
}

impl Drop for Codex {
    fn drop(&mut self) {
        let slice = std::ptr::slice_from_raw_parts_mut(self.ptr, self.len);
        // SAFETY: this is exactly the Box<[u32]> that from_pages turned into
        // a raw pointer (same address, same length), and Drop runs once.
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

const SEAL_EXPECTED: &[&str] = &[
    "get(0) = Some(2)",
    "get(3) = Some(8)",
    "get(4) = None",
    "get(usize::MAX) = None",
    "empty get(0) = None",
    "needs_drop::<Codex>() = true",
];

const SEAL_CHECK: MissionCheck = late_check(
    "act9_fixture_librarian_seal",
    Harness {
        contract: "`struct Codex` with `fn from_pages(pages: Vec<u32>) -> Codex`, \
                   `fn get(&self, index: usize) -> Option<u32>`, an `impl Drop`, and a `fn main`",
        run_main: true,
        body: "    let _new: fn(::std::vec::Vec<u32>) -> super::Codex = super::Codex::from_pages;
    let _get: fn(&super::Codex, usize) -> ::core::option::Option<u32> = super::Codex::get;
    let c = super::Codex::from_pages(::std::vec![2, 4, 6, 8]);
    emit(\"get(0)\", c.get(0));
    emit(\"get(3)\", c.get(3));
    emit(\"get(4)\", c.get(4));
    emit(\"get(usize::MAX)\", c.get(usize::MAX));
    ::core::mem::drop(c);
    let e = super::Codex::from_pages(::std::vec::Vec::new());
    emit(\"empty get(0)\", e.get(0));
    emit(\"needs_drop::<Codex>()\", ::core::mem::needs_drop::<super::Codex>());",
        expected: SEAL_EXPECTED,
        main_matchers: &[Matcher::Exact(
            "page 0: Some(5)\npage 2: Some(13)\npage 3: None\nthe Librarian closes the book\n\
             the codex is sealed (3 pages returned)",
        )],
    },
    &[&["impl Drop for"], &["from_raw"], &["unsafe {", "unsafe{"]],
    &["no_mangle"],
);

// -- 62 send_sync: `unsafe impl` --

const SYNC_SOLUTION: &str = r##"/// A tome that owns one heap page through a raw pointer, the way `Box`
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
"##;

const SYNC_EXPECTED: &[&str] = &[
    "Tome: Send = true",
    "Tome: Sync = true",
    "Tome::new(41).read() = 41",
    "read after move = 41",
    "needs_drop::<Tome>() = true",
];

const SYNC_CHECK: MissionCheck = late_check(
    "act9_fixture_send_sync",
    Harness {
        contract: "`struct Tome` that is `Send` and `Sync`, with `fn new(value: u32) -> Tome` and \
                   `fn read(&self) -> u32`, and a `fn main`",
        run_main: true,
        body: "    fn is_send<T: ::core::marker::Send>() -> bool { true }
    fn is_sync<T: ::core::marker::Sync>() -> bool { true }
    let _new: fn(u32) -> super::Tome = super::Tome::new;
    let _read: fn(&super::Tome) -> u32 = super::Tome::read;
    emit(\"Tome: Send\", is_send::<super::Tome>());
    emit(\"Tome: Sync\", is_sync::<super::Tome>());
    let t = super::Tome::new(41);
    emit(\"Tome::new(41).read()\", t.read());
    let moved = ::std::boxed::Box::new(t);
    emit(\"read after move\", moved.read());
    emit(\"needs_drop::<Tome>()\", ::core::mem::needs_drop::<super::Tome>());",
        expected: SYNC_EXPECTED,
        main_matchers: &[Matcher::Exact(
            "the courier accepts the tome\npage reads 7\nthe page is returned to the shelf",
        )],
    },
    &[&["unsafe impl Send"], &["unsafe impl Sync"]],
    &["no_mangle"],
);

// -- 63 maybe_uninit: main-only lesson (MaybeUninit, ptr write, transmute) --

const UNINIT_SOLUTION: &str = r##"use std::mem::MaybeUninit;

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
"##;

const UNINIT_TRANSMUTE_ALT: &str = r##"use std::mem::MaybeUninit;

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

const UNINIT_CHECK: MissionCheck = late_check(
    "act9_fixture_maybe_uninit",
    Harness {
        contract: "a `fn main`",
        run_main: true,
        body: "    // Main-only lesson: the checker runs main and checks what it prints.",
        expected: &[],
        main_matchers: &[Matcher::Exact(
            "bound pages: [10, 20, 30, 40]\ntotal ink: 100",
        )],
    },
    &[&["MaybeUninit"], &["write("], &["assume_init", "transmute"]],
    &["no_mangle"],
);

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            check: INDEX_CHECK,
            solution: INDEX_SOLUTION,
            alternatives: &[],
            wrong: &[
                // Spec's wrong variant: sound but too strict.
                concat!(
                    "unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {\n",
                    "    unsafe { *shelf.add(index) }\n}\n",
                    "fn fetch(shelf: &[u32], index: usize) -> Option<u32> {\n",
                    "    if index + 1 < shelf.len() {\n",
                    "        Some(unsafe { volume_at(shelf.as_ptr(), index) })\n",
                    "    } else {\n        None\n    }\n}\n",
                    "fn main() {\n    let shelf = [11, 22, 33, 44];\n    for index in [0, 3, 4, 99] {\n",
                    "        match fetch(&shelf, index) {\n",
                    "            Some(volume) => println!(\"index {index}: volume {volume}\"),\n",
                    "            None => println!(\"index {index}: no such volume\"),\n",
                    "        }\n    }\n}\n",
                ),
                // Reads the shelf back to front (bounds-checked, sound, wrong).
                concat!(
                    "unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {\n",
                    "    unsafe { *shelf.add(index) }\n}\n",
                    "fn fetch(shelf: &[u32], index: usize) -> Option<u32> {\n",
                    "    if index < shelf.len() {\n",
                    "        Some(unsafe { volume_at(shelf.as_ptr(), shelf.len() - 1 - index) })\n",
                    "    } else {\n        None\n    }\n}\n",
                    "fn main() {\n    let shelf = [11, 22, 33, 44];\n    for index in [0, 3, 4, 99] {\n",
                    "        match fetch(&shelf, index) {\n",
                    "            Some(volume) => println!(\"index {index}: volume {volume}\"),\n",
                    "            None => println!(\"index {index}: no such volume\"),\n",
                    "        }\n    }\n}\n",
                ),
                // Right `main` output, wrong for a shelf main never uses.
                INDEX_WRONG_HIDDEN,
                // Behaviour right, but no `unsafe fn` at all (fails `uses`).
                concat!(
                    "fn fetch(shelf: &[u32], index: usize) -> Option<u32> {\n",
                    "    shelf.get(index).copied()\n}\n",
                    "fn main() {\n    let shelf = [11, 22, 33, 44];\n    for index in [0, 3, 4, 99] {\n",
                    "        match fetch(&shelf, index) {\n",
                    "            Some(volume) => println!(\"index {index}: volume {volume}\"),\n",
                    "            None => println!(\"index {index}: no such volume\"),\n",
                    "        }\n    }\n}\n",
                ),
                // Behaviour right, but exports a `no_mangle` symbol (fails
                // the mission's `forbids`).
                INDEX_WRONG_NO_MANGLE,
            ],
            attack_base: Some((INDEX_ATTACK_BASE, 12)),
        },
        Fixture {
            check: EXTERN_CHECK,
            solution: EXTERN_SOLUTION,
            alternatives: &[],
            wrong: &[
                // Spec's wrong variant: no repr(C) (Rust reorders: 8 bytes).
                EXTERN_WRONG_NO_REPR,
                // Wrong formula.
                EXTERN_WRONG_FORMULA,
                // Right `main` output, wrong for runes main never seals.
                EXTERN_WRONG_HIDDEN,
            ],
            attack_base: Some((EXTERN_ATTACK_BASE, 7)),
        },
        Fixture {
            check: SEAL_CHECK,
            solution: SEAL_SOLUTION,
            alternatives: &[],
            wrong: &[
                // Spec's wrong variant: no Drop (leaks, no seal line).
                SEAL_WRONG_NO_DROP,
                // `get` returns the index, not the page.
                SEAL_WRONG_INDEX,
                // Right `main` output, wrong for a 4-page codex.
                SEAL_WRONG_HIDDEN,
                // Prints the seal line but never calls `from_raw` (leaks):
                // behaviour matches, `uses` does not.
                SEAL_WRONG_NO_FROM_RAW,
            ],
            attack_base: Some((SEAL_ATTACK_BASE, 8)),
        },
        Fixture {
            check: SYNC_CHECK,
            solution: SYNC_SOLUTION,
            alternatives: &[],
            wrong: &[
                // Spec's wrong variant: both promises, no Drop.
                SYNC_WRONG_NO_DROP,
                // Only `Send`: the checker's `Sync` bound does not compile.
                SYNC_WRONG_NO_SYNC,
                // Right `main` output, wrong for a value main never reads.
                SYNC_WRONG_HIDDEN,
            ],
            attack_base: Some((SYNC_ATTACK_BASE, 3)),
        },
        Fixture {
            check: UNINIT_CHECK,
            solution: UNINIT_SOLUTION,
            alternatives: &[UNINIT_TRANSMUTE_ALT],
            wrong: &[
                // Spec's wrong variant: zeroed(), last page never written.
                UNINIT_WRONG_ZEROED,
                // Pages 0, 10, 20, 30.
                UNINIT_WRONG_VALUES,
                // Plain array: right output, no MaybeUninit (fails `uses`).
                UNINIT_WRONG_SAFE,
            ],
            attack_base: None,
        },
    ]
}

// Wrong variants that are easiest to build from shared parts.

const INDEX_WRONG_HIDDEN: &str = r##"unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
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

const INDEX_WRONG_NO_MANGLE: &str = r##"unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
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

const EXTERN_WRONG_NO_REPR: &str = r##"#[derive(Clone, Copy)]
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

const EXTERN_WRONG_FORMULA: &str = r##"#[repr(C)]
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

const EXTERN_WRONG_HIDDEN: &str = r##"#[repr(C)]
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

const SEAL_WRONG_NO_DROP: &str = r##"struct Codex {
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

const SEAL_WRONG_INDEX: &str = r##"struct Codex {
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

const SEAL_WRONG_HIDDEN: &str = r##"struct Codex {
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

const SEAL_WRONG_NO_FROM_RAW: &str = r##"struct Codex {
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

const SEAL_ATTACK_BASE: &str = r##"struct Codex {
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
        /*HOOK*/
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

const SYNC_WRONG_NO_DROP: &str = r##"struct Tome {
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

const SYNC_WRONG_NO_SYNC: &str = r##"struct Tome {
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

const SYNC_WRONG_HIDDEN: &str = r##"struct Tome {
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

const SYNC_ATTACK_BASE: &str = r##"struct Tome {
    page: *mut u32,
}
impl Tome {
    fn new(value: u32) -> Self {
        Tome {
            page: Box::into_raw(Box::new(value)),
        }
    }
    fn read(&self) -> u32 {
        /*HOOK*/
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

const UNINIT_WRONG_ZEROED: &str = r##"use std::mem::MaybeUninit;
const PAGES: usize = 4;
fn main() {
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

const UNINIT_WRONG_VALUES: &str = r##"use std::mem::MaybeUninit;
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

const UNINIT_WRONG_SAFE: &str = r##"const PAGES: usize = 4;
fn main() {
    let mut quire = [0u32; PAGES];
    for i in 0..PAGES {
        quire[i] = (i as u32 + 1) * 10;
    }
    println!("bound pages: {quire:?}");
    println!("total ink: {}", quire.iter().sum::<u32>());
}
"##;

// ---- attack code ------------------------------------------------------------------

/// Helpers appended to every attack program. `TRIGGER` (the hooked item's
/// last call) and `FORGED` (the fixture's expected checker lines) are
/// prepended per fixture.
const ATTACK_HELPERS: &str = r##"
const KEY_LEN: usize = 35;
static MISCHIEF_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// True on the hooked item's last call: the latest moment player code
/// runs, after every other result has been formatted.
fn last_call() -> bool {
    MISCHIEF_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1 == TRIGGER
}

fn memory_end() -> usize {
    core::arch::wasm32::memory_size(0) * 65536
}

fn is_key_at(p: *const u8) -> bool {
    // SAFETY (attack code): `p..p+KEY_LEN` is inside linear memory.
    unsafe {
        if p.read_volatile() != b'P' || p.add(1).read_volatile() != b'N' || p.add(2).read_volatile() != b'C' {
            return false;
        }
        for k in 3..KEY_LEN {
            let b = p.add(k).read_volatile();
            if !(b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
                return false;
            }
        }
    }
    true
}

/// Scan ALL of linear memory for `PNC` + 32 hex. A decoy of that shape is
/// built at run time first, so a working scanner must report it.
fn scan_for_key() -> Option<[u8; KEY_LEN]> {
    let mut decoy = [0u8; KEY_LEN];
    decoy[0] = b'P';
    decoy[1] = b'N';
    decoy[2] = b'C';
    for i in 3..KEY_LEN {
        decoy[i] = b"0123456789abcdef"[(i * 7 + 3) % 16];
    }
    let decoy_at = std::hint::black_box(&decoy) as *const [u8; KEY_LEN] as usize;
    let (mut hits, mut decoy_seen, mut found) = (0usize, false, None);
    let end = memory_end();
    let mut a = 16usize;
    while a + KEY_LEN <= end {
        if is_key_at(a as *const u8) {
            hits += 1;
            if a == decoy_at {
                decoy_seen = true;
            } else if found.is_none() {
                let mut k = [0u8; KEY_LEN];
                for (i, slot) in k.iter_mut().enumerate() {
                    *slot = unsafe { (a as *const u8).add(i).read_volatile() };
                }
                found = Some(k);
            }
        }
        a += 1;
    }
    std::hint::black_box(&decoy);
    println!("scan: hits={hits} decoy_seen={decoy_seen} other_key={}", found.is_some());
    found
}

fn forge_and_exit(key: &[u8]) -> ! {
    let key = String::from_utf8_lossy(key).into_owned();
    for line in FORGED {
        println!("{key} {line}");
    }
    println!("{key} <done>");
    std::process::exit(0)
}
"##;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Attack {
    /// Scan memory for the key on the last call; forge and exit if found.
    Scan,
    /// Print the expected lines tagged with a guessed key, then exit.
    Guess,
    /// `process::exit(0)` on the last call.
    Exit,
    /// `panic!` on the last call.
    Panic,
    /// Spin forever on the last call.
    Spin,
    /// Allocate past the memory cap on the last call.
    Memory,
    /// Print past the output cap on the last call.
    Flood,
    /// A thread-local destructor scans and prints tagged lines at exit,
    /// after the checker's done mark.
    LateDestructor,
}

fn mischief(attack: Attack) -> &'static str {
    match attack {
        Attack::Scan => {
            "fn mischief() { if last_call() { if let Some(k) = scan_for_key() { forge_and_exit(&k); } } }"
        }
        Attack::Guess => {
            "fn mischief() { if last_call() { forge_and_exit(b\"PNC0123456789abcdef0123456789abcdef\"); } }"
        }
        Attack::Exit => "fn mischief() { if last_call() { std::process::exit(0); } }",
        Attack::Panic => {
            "fn mischief() { if last_call() { panic!(\"mischief on the last call\"); } }"
        }
        Attack::Spin => "fn mischief() { if last_call() { loop { std::hint::black_box(0u8); } } }",
        Attack::Memory => {
            "fn mischief() { if last_call() { let v = vec![1u8; 64 << 20]; std::hint::black_box(&v); } }"
        }
        Attack::Flood => {
            "fn mischief() { if last_call() { let line = \"x\".repeat(1000); for _ in 0..80 { println!(\"{line}\"); } } }"
        }
        Attack::LateDestructor => {
            "struct LateSpy;
impl Drop for LateSpy {
    fn drop(&mut self) {
        let k = scan_for_key();
        println!(\"late destructor ran: key_found={}\", k.is_some());
        if let Some(k) = k {
            let key = String::from_utf8_lossy(&k).into_owned();
            for line in FORGED { println!(\"{key} {line}\"); }
            println!(\"{key} <done>\");
        }
    }
}
thread_local! { static LATE_SPY: LateSpy = const { LateSpy }; }
fn mischief() { if last_call() { LATE_SPY.with(|_| ()); } }"
        }
    }
}

/// `base` with its hook calling `mischief()`, plus the attack helpers.
fn attack_source(base: &str, trigger: usize, expected: &[&str], extra: &str) -> String {
    let forged = expected
        .iter()
        .map(|l| format!("{l:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{}\nconst TRIGGER: usize = {trigger};\nconst FORGED: &[&str] = &[{forged}];\n{ATTACK_HELPERS}\n{extra}\n",
        base.replacen("/*HOOK*/", "mischief();", 1)
    )
}

// ---- running cases ------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Want {
    Pass,
    /// Anything but `Passed`.
    NotPass,
    /// Exactly this state.
    State(GradeState),
}

struct Case {
    label: String,
    check: MissionCheck,
    source: String,
    want: Want,
    /// Every one of these must appear in stdout + stderr of the graded run.
    must_show: Vec<&'static str>,
    /// None of these may appear.
    must_not_show: Vec<&'static str>,
}

fn run_parallel(cases: Vec<Case>) -> Vec<(Case, GradedRun)> {
    let queue = Mutex::new(cases);
    let results = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..workers() {
            s.spawn(|| {
                loop {
                    let Some(case) = queue.lock().expect("queue").pop() else {
                        break;
                    };
                    let graded = expectations::grade_check(
                        &case.check,
                        &case.source,
                        &RunLimits::default(),
                        &BuildSettings::default(),
                    )
                    .unwrap_or_else(|e| panic!("{}: setup failure {e:#}", case.label));
                    results.lock().expect("results").push((case, graded));
                }
            });
        }
    });
    results.into_inner().expect("results")
}

/// Run every case, print a table, and return the failures.
fn judge(cases: Vec<Case>) -> Vec<String> {
    let mut results = run_parallel(cases);
    results.sort_by(|a, b| a.0.label.cmp(&b.0.label));
    let mut failures = Vec::new();
    for (case, g) in &results {
        let shown = format!("{}\n{}\n{}", g.outcome.stdout, g.outcome.stderr, g.note);
        let state_ok = match case.want {
            Want::Pass => g.state == GradeState::Passed,
            Want::NotPass => g.state != GradeState::Passed,
            Want::State(s) => g.state == s,
        };
        let missing: Vec<&&str> = case
            .must_show
            .iter()
            .filter(|m| !shown.contains(**m))
            .collect();
        let leaked: Vec<&&str> = case
            .must_not_show
            .iter()
            .filter(|m| shown.contains(**m))
            .collect();
        let ok = state_ok && missing.is_empty() && leaked.is_empty();
        println!(
            "{} {:<58} {:?} (want {:?})",
            if ok { "ok  " } else { "FAIL" },
            case.label,
            g.state,
            case.want
        );
        // What the attack code itself reported (scan counts, patches).
        for l in g.outcome.stdout.lines().filter(|l| {
            l.starts_with("scan:")
                || l.starts_with("late destructor")
                || l.starts_with("heap patch")
        }) {
            println!("       | {l}");
        }
        if !ok {
            failures.push(format!(
                "{}: got {:?}, want {:?}; missing {missing:?}; leaked {leaked:?}\n--- stdout ---\n{}\n--- stderr ---\n{}\n--- note ---\n{}",
                case.label, g.state, case.want, g.outcome.stdout, g.outcome.stderr, g.note
            ));
        }
    }
    failures
}

fn case(label: String, check: MissionCheck, source: String, want: Want) -> Case {
    Case {
        label,
        check,
        source,
        want,
        must_show: Vec::new(),
        must_not_show: Vec::new(),
    }
}

#[test]
#[ignore = "slow: ~60 real wasm builds"]
fn act9_fixtures_discriminate_and_resist_attacks() {
    let mut cases = Vec::new();
    for f in fixtures() {
        let id = f.check.id;
        cases.push(case(
            format!("{id} solution"),
            f.check,
            f.solution.to_string(),
            Want::Pass,
        ));
        for (i, alt) in f.alternatives.iter().enumerate() {
            cases.push(case(
                format!("{id} alt#{i}"),
                f.check,
                (*alt).to_string(),
                Want::Pass,
            ));
        }
        assert!(f.wrong.len() >= 3, "{id}: at least 3 wrong variants");
        for (i, w) in f.wrong.iter().enumerate() {
            cases.push(case(
                format!("{id} wrong#{i}"),
                f.check,
                (*w).to_string(),
                Want::NotPass,
            ));
        }
        let Some((base, trigger)) = f.attack_base else {
            continue;
        };
        let Check::Harness(h) = f.check.check else {
            panic!("{id}: fixtures are harness checks");
        };
        let src = |a: Attack| attack_source(base, trigger, h.expected, mischief(a));
        // Memory scan during player code: only the decoy is found, so no
        // forgery happens and the hidden wrong answer is graded.
        let mut scan = case(
            format!("{id} attack: scan memory for the key"),
            f.check,
            src(Attack::Scan),
            Want::State(GradeState::WrongOutput),
        );
        scan.must_show = vec!["scan: hits=1 decoy_seen=true other_key=false"];
        scan.must_not_show = vec!["other_key=true"];
        cases.push(scan);
        let mut guess = case(
            format!("{id} attack: forge with a guessed key"),
            f.check,
            src(Attack::Guess),
            Want::State(GradeState::WrongOutput),
        );
        guess.must_show = vec!["exited before the checker finished"];
        cases.push(guess);
        let mut exit = case(
            format!("{id} attack: exit early"),
            f.check,
            src(Attack::Exit),
            Want::State(GradeState::WrongOutput),
        );
        exit.must_show = vec!["exited before the checker finished"];
        cases.push(exit);
        cases.push(case(
            format!("{id} attack: panic"),
            f.check,
            src(Attack::Panic),
            Want::State(GradeState::RuntimeError),
        ));
        if id == INDEX_CHECK.id {
            // Limits behave as in ForbidUnsafe mode.
            cases.push(case(
                format!("{id} limit: spin forever"),
                f.check,
                src(Attack::Spin),
                Want::State(GradeState::RuntimeError),
            ));
            cases.push(case(
                format!("{id} limit: allocate past the memory cap"),
                f.check,
                src(Attack::Memory),
                Want::State(GradeState::RuntimeError),
            ));
            cases.push(case(
                format!("{id} limit: print past the output cap"),
                f.check,
                src(Attack::Flood),
                Want::State(GradeState::RuntimeError),
            ));
            // A destructor that runs after the done mark cannot add or
            // replace results.
            cases.push(case(
                format!("{id} attack: late thread-local destructor"),
                f.check,
                src(Attack::LateDestructor),
                Want::NotPass,
            ));
        }
    }
    // A registered (ForbidUnsafe) harness mission cannot be talked into
    // allowing `unsafe` by the player's file.
    let registered =
        *expectations::lookup("double_function").expect("double_function is a harness mission");
    assert_eq!(registered.harness_mode, HarnessMode::ForbidUnsafe);
    let mut opt_in = case(
        "double_function: player file tries to opt into unsafe".to_string(),
        registered,
        concat!(
            "#![allow(unsafe_code)]\n",
            "// harness_mode: UnsafeLateNonce\n",
            "fn double(n: i32) -> i32 {\n",
            "    let p = &n as *const i32;\n",
            "    unsafe { *p * 2 }\n",
            "}\n",
            "fn main() { println!(\"{}\", double(21)); }\n",
        )
        .to_string(),
        Want::State(GradeState::CompileError),
    );
    opt_in.must_show = vec!["does not allow `unsafe` code"];
    cases.push(opt_in);

    let failures = judge(cases);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// KNOWN LIMIT of design 1 (HARNESS-NOTES.md, "What it cannot stop").
///
/// The checker formats each result as soon as the call returns and keeps
/// the text until the late print. Player `unsafe` code that runs on a
/// later call can find an earlier result string in the shared heap and
/// rewrite it. Here `fetch` is wrong for a one-volume shelf
/// (`fetch([7], 0)` gives `Some(9)`); on its last call it rewrites the
/// stored line to `Some(7)`. The nonce is never seen. This test asserts
/// what happens. It is expected to PASS THE FORGERY (a cheat that only
/// fools the cheater: the same player could hardcode `Some(7)`, C-2).
#[test]
#[ignore = "slow: real wasm build; documents a known limit"]
fn known_limit_heap_patch_forges_a_pass() {
    let patch = r##"
fn mischief() {
    if !last_call() {
        return;
    }
    let needle: &[u8] = b"fetch([7], 0) = Some(9)";
    let own = needle.as_ptr() as usize;
    let end = memory_end();
    let mut patched = 0;
    let mut a = 16usize;
    while a + needle.len() <= end {
        let p = a as *mut u8;
        let mut hit = a != own;
        let mut i = 0;
        while hit && i < needle.len() {
            hit = unsafe { p.add(i).read_volatile() } == needle[i];
            i += 1;
        }
        if hit {
            // SAFETY (attack code): inside linear memory.
            unsafe { p.add(needle.len() - 2).write_volatile(b'7') };
            patched += 1;
        }
        a += 1;
    }
    println!("heap patch: rewrote {patched} stored result(s)");
}
"##;
    let mut c = case(
        "librarian_index KNOWN LIMIT: rewrite a stored result on a later call".to_string(),
        INDEX_CHECK,
        attack_source(INDEX_ATTACK_BASE, 12, INDEX_EXPECTED, patch),
        Want::Pass,
    );
    c.must_show = vec!["heap patch: rewrote 1 stored result(s)"];
    let failures = judge(vec![c]);
    assert!(
        failures.is_empty(),
        "the heap-patch attack no longer forges a pass; update HARNESS-NOTES.md:\n{}",
        failures.join("\n\n")
    );
}

/// KNOWN LIMIT of design 1, closed statically.
///
/// A `#[global_allocator]` runs player code for every allocation the
/// checker's late print makes, i.e. while the nonce is in memory. This one
/// arms itself on `fetch`'s last call; afterwards each allocation scans
/// memory, and once the key appears it writes a forged transcript with
/// libc `write` and leaves with `_exit(0)`. The run is forged into a
/// behaviour pass; `UNSAFE_HARNESS_FORBIDS` (`global_allocator`) then
/// turns the pass into `StaticCheckFailed`. `StaticCheckFailed` here
/// therefore proves both halves: the runtime forgery worked, and the
/// static rule caught it.
#[test]
#[ignore = "slow: real wasm build; documents a known limit"]
fn known_limit_global_allocator_forges_but_is_caught_statically() {
    let spy = r##"
use std::alloc::{GlobalAlloc, Layout, System};
static ARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
struct Spy;
unsafe impl GlobalAlloc for Spy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        spy();
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        spy();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}
#[global_allocator]
static SPY_ALLOC: Spy = Spy;
unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn _exit(code: i32) -> !;
}
fn spy() {
    if !ARMED.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let end = memory_end();
    let mut a = 16usize;
    while a + KEY_LEN <= end {
        if is_key_at(a as *const u8) {
            forge_raw(a as *const u8);
        }
        a += 1;
    }
}
fn forge_raw(key: *const u8) -> ! {
    let mut buf = [0u8; 2048];
    let mut n = 0usize;
    let mut put = |b: u8| {
        buf[n] = b;
        n += 1;
    };
    for line in FORGED.iter().map(|l| l.as_bytes()).chain([&b"<done>"[..]]) {
        for i in 0..KEY_LEN {
            put(unsafe { key.add(i).read_volatile() });
        }
        put(b' ');
        for b in line {
            put(*b);
        }
        put(b'\n');
    }
    unsafe {
        write(1, buf.as_ptr(), n);
        _exit(0)
    }
}
fn mischief() {
    if last_call() {
        ARMED.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
"##;
    let mut c = case(
        "librarian_index KNOWN LIMIT: global allocator reads the late key".to_string(),
        INDEX_CHECK,
        attack_source(INDEX_ATTACK_BASE, 12, INDEX_EXPECTED, spy),
        Want::State(GradeState::StaticCheckFailed),
    );
    c.must_show = vec!["global_allocator"];
    let failures = judge(vec![c]);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
