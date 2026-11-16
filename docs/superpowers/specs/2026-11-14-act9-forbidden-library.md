# Act 9 — Forbidden Library mission batch (design spec)

**Date:** 2026-11-14
**Status:** IMPLEMENTED 2026-11-15 (wave 14). The registry grew 78 → 87,
not 57 → 66: the Acts 2-8 trials landed first, so these are missions
79-87. What changed from this draft is recorded in **Implementation
record (2026-11-15)** at the end; the game text and the tests are the
source of truth where they differ from the drafts below.
**Topic:** 6 missions plus a 3-stage boss trial covering Act 9's unsafe /
FFI / macro / drop / marker-trait tier, in the existing mission-tutorial
architecture (same shape as Acts 3–8). Registry 57 → 66.

## Goal

`design/01-curriculum.md` §"Act 9 — The Forbidden Library" teaches `unsafe`
blocks and raw pointers, `extern "C"` / FFI, `macro_rules!`, `Drop`, manual
`Send`/`Sync` impls, pointer arithmetic and `MaybeUninit`. This batch ships
one mission per concept, then the Librarian's trial, appended after
`weak_ref`.

| # | id | NPC | Concept | Grader tokens (any-of groups) |
|---|----|-----|---------|-------------------------------|
| 58 | `raw_pointer` | The Shelfmark | `*const`/`*mut`, deref in `unsafe` | `unsafe {`; `*mut`/`&raw mut`/`addr_of_mut!`; `*const`/`&raw const`/`addr_of!` |
| 59 | `extern_c` | The Envoy | `extern "C"`, `#[unsafe(no_mangle)]`, `#[repr(C)]`, foreign declarations | `repr(C)`; `no_mangle`; `extern "C" fn`; `unsafe {` |
| 60 | `macro_rules` | The Macro Monk | `macro_rules!` with repetition | `macro_rules!`; `$(` |
| 61 | `drop_trait` | The Sexton | `impl Drop`, drop order, `mem::drop`, `mem::forget` | `impl Drop for`; `forget(` |
| 62 | `send_sync` | The Courier | `unsafe impl Send` / `Sync`, auto traits | `unsafe impl Send`; `unsafe impl Sync` |
| 63 | `maybe_uninit` | The Bookbinder | `MaybeUninit`, pointer arithmetic, `ptr::write`, `assume_init` | `MaybeUninit`; `write(`; `assume_init`/`transmute` |
| 64 | `librarian_index` | The Librarian (trial 1/3) | `unsafe fn` + `# Safety` contract + safe wrapper | `unsafe fn`; `.len()` |
| 65 | `librarian_demand` | The Librarian (trial 2/3) | recursive `macro_rules!` with internal rules | `macro_rules!`; `$(` |
| 66 | `librarian_seal` | The Librarian (trial 3/3) | owning raw pointer: `Box::into_raw` / `from_raw` in `Drop` | `impl Drop for`; `from_raw` |

Every mission is graded first by its **expected stdout** (finish plan D2,
exact match) and only second by the token table. The tokens exist so the
offline pattern grader (web, mobile) has something to check and so a
program that prints the right text without using the concept still fails.
None of the tokens is a variable or parameter name (mission-audit G3).

## Hard rule 1: where `unsafe` may appear

Hard rule 1 ("no `unsafe` in shipped game code") is unchanged by this act.

- **Allowed:** `unsafe` inside the player's submitted program. It is the
  lesson. That program is compiled for `wasm32-wasip1` and run in the
  wasmtime sandbox, never in the game process (hard rule 2).
- **Allowed:** `unsafe` inside *string literals* in the game crate: starter
  code in `mission.rs`, canonical solutions in
  `editor/solutions/data.rs`, `contract.rs` and `tests/stub_grader.rs`, and
  grader tokens such as `"unsafe impl Send"`. These are `&'static str` data.
- **Not allowed:** any `unsafe` block, `unsafe fn`, `unsafe impl` or
  `#[unsafe(...)]` attribute in game or compile-api *Rust code* as a result
  of this batch. Nothing in this spec needs one. If an implementer finds
  they want one, the design is wrong; stop and ask.

## Server grading mode for this act (added 2026-11-15)

Every mission before Act 9 that the server grades by calling the
player's code builds with `#![forbid(unsafe_code)]`, which these lessons
cannot pass. The server now has an opt-in harness mode for them,
`HarnessMode::UnsafeLateNonce` (`compile-api/HARNESS-NOTES.md`, "Design 1
as built"): `unsafe` is allowed, and the checker's result key comes from
the host only after the player's code has finished. Any Act 9 mission
graded through a server checker (rather than by its stdout alone) must
be registered with that mode; all other missions keep `ForbidUnsafe`. It
raises the bar for forged grades; it does not guarantee them (a player
who rewrites the checker's stored results in memory can forge a pass,
which only fools the cheater). It was proven on five of this spec's
missions as test fixtures (`compile-api/tests/act9_unsafe_harness.rs`).
The server-written checker contains one `unsafe` block, the FFI call to
the host import; it is guest source in a string, compiled into the
player's wasm module, not compile-api Rust code, so the hard-rule-1 list
above still holds for the server process.

## Sandbox constraints and what they cost this act

The player's program is one `src/main.rs`, std only, no dependencies,
stable toolchain, built with `cargo build --target wasm32-wasip1` and run in
wasmtime with stdio only (no threads, files, network or env). The server
refuses `include*!`, `#[path]`, out-of-line `mod x;`, `#[link]`,
`#![feature]` and `$` inside attributes before building
(`compile-api/src/source_policy.rs`).

**Edition finding (affects both acts).** The server's frozen manifest is
**edition 2024** (`compile-api/src/wasm_builder.rs:339`), while
`game/tests/solutions.rs` builds solutions with `--edition 2021`. Every
program in this spec was therefore built in **both** editions. Two
consequences for Act 9:

1. Foreign declarations must be written `unsafe extern "C" { ... }` and
   the export attribute `#[unsafe(no_mangle)]`. Edition 2024 rejects the
   plain forms ("extern blocks must be unsafe"); edition 2021 accepts the
   `unsafe` forms (stable since Rust 1.82). A first draft using the plain
   forms compiled under 2021 and failed in the real server sandbox; that is
   how this was found.
2. The E0133 message differs by edition: 2024 says "requires unsafe
   block", 2021 says "requires unsafe function or block". The server shows
   the 2024 wording. Starter errors below are quoted from edition 2024;
   where 2021 differs it is noted.

The server also prepends `#![allow(dead_code, unused)]` to the source, so
on the server every diagnostic's line number is **one higher** than in the
local `rustc` output quoted here, and the path reads `src/main.rs`.

What the curriculum asks for that cannot be done here, and the substitute:

| Curriculum item | Why not | Substitute |
|---|---|---|
| Call a shipped C library (`libcrown.so`) | No C toolchain or `.so` in the sandbox; `#[link]` is refused by the server; wasm has no dynamic linking | `extern_c`: the player exports a C-ABI function (`#[unsafe(no_mangle)] pub extern "C" fn`) and calls it back through an `unsafe extern "C"` declaration in the same program. Both halves of FFI (Rust called *as* C, Rust calling *through* a C declaration) are real; the linker resolves the declaration to the exported symbol. |
| "Calling C from Rust and vice versa" with real C code | Same | Tutorial text says plainly that real C needs a C compiler, a `build.rs` (or the `cc` crate) and `#[link]`, none of which exist in the sandbox, and names `bindgen` / `cbindgen` |
| Brief intro to procedural macros | A proc macro must live in its own `proc-macro` crate; the sandbox is one file, no dependencies | Mentioned in the `macro_rules` tutorial only (what `#[derive]` is, that it runs Rust code at compile time); not graded |
| Manual `Send`/`Sync` demonstrated with threads | `wasm32-wasip1` std has no threads: `thread::spawn` panics (see `EXPECTED_SANDBOX_RUN_FAILURES` in `game/tests/solutions.rs`) | `send_sync` checks the promise at compile time through `T: Send` / `T: Sync` bounds, which is where Rust enforces it anyway |
| Sanity meter, save-file corruption, character corruption | RPG layer; out of scope for v1.0 (finish plan D1) | None. The Alarm-scarlet "corruption glitch" survives only as an art motif on the Librarian's arena prop |
| `static mut` and unions (two of the five unsafe superpowers) | Not cut by the sandbox; cut for scope | Named in the `raw_pointer` tutorial; not exercised |

## Verification record

Toolchain: `rustc 1.99.0 (b940084d7 2026-09-28)`, host
`x86_64-pc-windows-gnu`, LLVM 23.1.1. `wasm32-wasip1` installed.

For every program below (9 canonical solutions, 9 starters, 9 wrong
variants, plus alternatives):

| Check | Command | Result |
|---|---|---|
| Native, warnings denied, both editions | `rustc --edition {2021,2024} -D warnings <file>` | 9/9 solutions |
| `wasm32-wasip1`, warnings denied, both editions | same plus `--target wasm32-wasip1` | 9/9 solutions |
| Run native, both editions, stdout identical across editions | run the binaries, `cmp` | 9/9 solutions |
| Real server sandbox (cargo build for wasip1 with the server's deny-list, then wasmtime) | a throwaway `compile-api` test calling `wasm_builder::compile_and_run` on each file (not committed) | 9/9 solutions compiled, ran cleanly, stdout byte-identical to native; every starter and wrong variant behaved there exactly as recorded below |
| Starters | both editions, native and sandbox | 8 fail to compile for the intended reason; 1 (`librarian_seal`) compiles on the server and prints the wrong output (see each mission) |
| Alternatives | `alt/raw_pointer` (`&raw`), `alt/maybe_uninit` (array of `MaybeUninit` + `transmute`) | both editions, both targets, sandbox: same stdout as the canonical; both pass the token table |
| Wrong variants | both editions, native and sandbox | all compile and print output that differs from the expected stdout |
| Miri | `cargo +nightly miri --version` | **Not run.** Miri is not installed for the local nightly (`cargo-miri.exe is not installed for the toolchain nightly-x86_64-pc-windows-gnu`). The unsafe code below has NOT been checked by Miri. Each unsafe block carries a "why this is sound" note instead; installing Miri and running it on the nine solutions is an open item. |

## Per-mission detail

Each mission supplies a `prompt`, a 4-section `tutorial`
(`## Concept`/`## Syntax`/`## Task`/`## Hint`), a starter that must not
pass the grader, a `MissionRule`, a canonical solution, and expected stdout.
The tutorial drafts are in the NPC's voice but short; voice-editor owns the
final pass (finish plan C4). The three hints are escalating: the
`## Hint` section carries hint 1; hints 2 and 3 are for the B2 "hints after
failures" feature if it lands, otherwise append them to `## Hint`.

---

### 58 — `raw_pointer` (The Shelfmark)

**NPC.** The Shelfmark: keeper of the card catalogue, who deals only in
call numbers, never in books. "An address is a promise. I do not check
promises. You do."

**Framing.** The Library's first door opens only for someone who can
fetch a book by its shelfmark alone.

**Prompt.** "Make a raw pointer from a reference and dereference it inside `unsafe`."

**Tutorial draft.**

```text
## Concept
A raw pointer (`*const T`, `*mut T`) is an address with no promises: it
may be null, dangling, unaligned, or alias a `&mut`. Making one is safe.
Reading or writing through one is not, so it goes inside an `unsafe`
block. `unsafe` turns nothing off: types and the borrow checker still
apply to every reference in the block. It unlocks five extra operations
(dereference a raw pointer, call an `unsafe` fn, access a `static mut`,
implement an `unsafe` trait, read a union field) and makes you
responsible for the rules the compiler can no longer check. Break them
and the program has undefined behaviour: no defined meaning at all, not
merely a wrong answer.

## Syntax
let p: *mut i32 = &mut x;   // safe: just an address
unsafe { *p += 1; }          // you promise p is valid
let r = &raw const x;        // a *const i32, no reference made

## Task
Add 5 to `gold` through `write_ptr`, then read it back through a
`*const i32` and print the three lines.

## Hint
The compiler names the line: dereferencing a `*mut i32` must sit inside
`unsafe { ... }`.
```

**Hints.**
1. The compiler names the line: dereferencing a `*mut i32` must sit inside `unsafe { ... }`.
2. `let read_ptr: *const i32 = &gold;` coerces a shared reference to a raw pointer. Make it after the write.
3. `unsafe { *write_ptr += 5; }`, then `let seen = unsafe { *read_ptr };`. Put a `// SAFETY:` comment above each saying why the pointer is valid.

**Starter** (fails: E0133, the intended reason; has `*mut` but no `unsafe {`, so it also fails the grader):

```rust
fn main() {
    let mut gold: i32 = 10;
    let write_ptr: *mut i32 = &mut gold;
    // Add 5 to gold through write_ptr, then read gold back through a
    // read-only raw pointer and print both lines.
    *write_ptr += 5;
    println!("gold by name: {gold}");
}
```

Starter result, edition 2024 (server wording):

```text
error[E0133]: dereference of raw pointer is unsafe and requires unsafe block
 --> start/raw_pointer.rs:6:5
  |
6 |     *write_ptr += 5;
  |     ^^^^^^^^^^ dereference of raw pointer
  |
  = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0133`.
```

Edition 2021 differs only in the headline: "...requires unsafe function or block".

**Canonical solution:**

```rust
fn main() {
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
```

**Expected stdout:**

```text
pointer is null: false
gold through the pointer: 15
gold by name: 15
```

**Why this is sound.** `write_ptr` is derived from `&mut gold` and used
once, immediately, while no other reference to `gold` is used; `i32` is
aligned and `gold` is alive. `read_ptr` is derived from `&gold` after the
last use of `write_ptr`, so the shared borrow does not conflict with the
earlier unique one; the read happens while `gold` is alive and unmodified.
The address itself is never printed (it is not deterministic).

**Grader rule:**

```rust
MissionRule {
    id: "raw_pointer",
    npc: "the Shelfmark taps the card",
    checks: &[
        check(&["unsafe {", "unsafe{"], "dereference the pointer inside an `unsafe` block"),
        check(&["*mut", "&raw mut", "addr_of_mut!"], "write through a `*mut` raw pointer"),
        check(&["*const", "&raw const", "addr_of!"], "read back through a `*const` raw pointer"),
    ],
    pass: "the Shelfmark files the card. \"an address is not a book — but you fetched the book.\"",
},
```

**Alternative that must pass** (raw-borrow syntax, no `*mut` text; verified,
same stdout, passes the `&raw mut` / `&raw const` alternatives):

```rust
fn main() {
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
```

**Wrong variant that must fail** (points at a copy; compiles, passes the
tokens, fails the stdout check):

```rust
fn main() {
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
```

```text
pointer is null: false
gold through the pointer: 10
gold by name: 10
```

**References.** Rust Book ch. 20.1 "Unsafe Rust" → "Dereferencing a Raw
Pointer" and "Unsafe Superpowers"; Rustonomicon "Meet Safe and Unsafe",
"What Unsafe Rust Can Do"; Reference "Behavior considered undefined",
"Raw borrow operators".

---

### 59 — `extern_c` (The Envoy)

**NPC.** The Envoy: a diplomat who carries treaties between kingdoms that
share no language, only an agreed protocol. "We need not speak alike. We
must agree on where the arguments go."

**Framing.** The Library holds books written in a foreign tongue. The
Envoy lets you through only if you can both offer and accept a treaty.

**Prompt.** "Export a function with the C ABI and call it back through an `unsafe extern \"C\"` declaration."

**Tutorial draft.**

```text
## Concept
FFI is how Rust calls, and is called by, code from other languages.
`extern "C"` picks the C calling convention: where arguments and the
return value travel. `#[unsafe(no_mangle)]` exports a function under its
plain name so a C linker can find it. `#[repr(C)]` lays a struct out by
C's rules (declared field order, C padding); Rust's default layout is
unspecified. An `unsafe extern "C" { ... }` block declares a foreign
function. The compiler cannot check that declaration against the real
code, so calling it is unsafe. This sandbox cannot link a C library, so
you play both sides in one program: export `crown_seal` as a C library
would, then call it back through a foreign declaration.

## Syntax
#[repr(C)] struct Rune { glyph: u8, power: u32, tier: u8 }
#[unsafe(no_mangle)]
pub extern "C" fn crown_seal(r: Rune) -> u32 { /* ... */ }
unsafe extern "C" { fn crown_seal(r: Rune) -> u32; }
let v = unsafe { crown_seal(r) };
(Edition 2024 requires the `unsafe` in both places; 2021 accepts it.)

## Task
Give `Rune` a C layout, export `crown_seal` with the C ABI under its plain
name, call it through `foreign::crown_seal` inside `unsafe`, and print the
size line and the three results.

## Hint
The error points at the call: anything declared in an extern block is
unsafe to call, because only you know the declaration is true.
```

**Hints.**
1. The error points at the call: anything declared in an extern block is unsafe to call.
2. Add `#[repr(C)]` and `#[derive(Clone, Copy)]` to `Rune`, and `#[unsafe(no_mangle)] pub extern "C"` to `crown_seal`.
3. With `repr(C)`, `u8, u32, u8` is 12 bytes: 1, 3 padding, 4, 1, 3 tail padding; align 4. If you see 8, `repr(C)` is missing and Rust reordered the fields.

**Starter** (fails: E0133 at the foreign call; lacks `repr(C)`, `no_mangle`, `extern "C" fn`):

```rust
pub struct Rune {
    pub glyph: u8,
    pub power: u32,
    pub tier: u8,
}

// Give Rune a C layout, and export this with the C calling convention
// under its plain name so the foreign declaration below can find it.
pub fn crown_seal(rune: Rune) -> u32 {
    rune.power * 2 + rune.tier as u32 + rune.glyph as u32
}

mod foreign {
    unsafe extern "C" {
        pub fn crown_seal(rune: super::Rune) -> u32;
    }
}

fn main() {
    let rune = Rune { glyph: 3, power: 20, tier: 1 };
    let sealed = foreign::crown_seal(rune);
    println!("through the extern block: {sealed}");
}
```

```text
error[E0133]: call to unsafe function `foreign::crown_seal` is unsafe and requires unsafe block
  --> start/extern_c.rs:21:18
   |
21 |     let sealed = foreign::crown_seal(rune);
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0133`.
```

**Canonical solution:**

```rust
/// Laid out by C's rules: fields in declared order, C padding.
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
```

**Expected stdout:**

```text
Rune: 12 bytes, align 4
direct call: 44
through the extern block: 44
through a fn pointer: 44
```

**Why this is sound.** The foreign declaration in `foreign` has exactly the
signature of the exported definition (same symbol, a `repr(C)` `Copy`
argument, `u32` return, C ABI), and exactly one item exports the symbol
`crown_seal`, which is what `#[unsafe(no_mangle)]` asks us to promise. The
declaration resolves at link time to that definition; this was confirmed on
both targets by running it (native and in wasmtime). 12 bytes / align 4 is
fixed by C layout rules for `u8, u32, u8` when `u32` has alignment 4, which
holds on `wasm32` and `x86_64`.

**Grader rule:**

```rust
MissionRule {
    id: "extern_c",
    npc: "the Envoy waits at the border",
    checks: &[
        check(&["repr(C)"], "lay `Rune` out by C's rules with `#[repr(C)]`"),
        check(&["no_mangle"], "export `crown_seal` under its plain name with `#[unsafe(no_mangle)]`"),
        check(&["extern \"C\" fn"], "give `crown_seal` the C calling convention: `pub extern \"C\" fn`"),
        check(&["unsafe {", "unsafe{"], "call through the foreign declaration inside `unsafe`"),
    ],
    pass: "the Envoy seals the treaty. \"two tongues, one calling convention.\"",
},
```

**Wrong variant that must fail** (no `#[repr(C)]`): it fails under
`-D warnings` (`improper_ctypes`, `improper_ctypes_definitions`), but the
server does not deny warnings, so there it compiles and runs. It also fails
the `repr(C)` token. Observed stdout on 1.99.0:

```text
Rune: 8 bytes, align 4
direct call: 44
through the extern block: 44
through a fn pointer: 44
```

The `8 bytes` line is the observed default layout, which Rust does not
specify; it shows why the expected stdout includes the size.

**Not taught, stated in the tutorial:** linking real C (needs a C compiler,
`build.rs` or the `cc` crate, `#[link]`), `bindgen`/`cbindgen`, C strings
(`CStr`/`CString`), variadic functions, unwinding across FFI.

**References.** Rust Book ch. 20.1 → "Using `extern` Functions to Call
External Code" and "Calling Rust Functions from Other Languages";
Rustonomicon "FFI"; Reference "External blocks", "Type layout → The `C`
representation", "Application binary interface → `no_mangle`".

---

### 60 — `macro_rules` (The Macro Monk)

**NPC.** The Macro Monk: a silent copyist who carves a woodblock once and
prints the same page a thousand times. Speaks only in patterns.

**Framing.** The curriculum's Macro Monk: abbreviate a chant you will need
again and again.

**Prompt.** "Define a `macro_rules!` macro that takes one or more expressions."

**Tutorial draft.**

```text
## Concept
`macro_rules!` defines a macro by example: each rule is a pattern over
tokens and a template to paste in its place. Macros expand before type
checking, so unlike functions they can take any number of arguments.
`$x:expr` captures an expression; `$( ... ),+` repeats a sub-pattern one
or more times, separated by commas (`*` for zero or more). Pasting is
literal: write `$x` twice and the expression runs twice, so bind it to a
local once. Locals made inside the template cannot clash with the
caller's names (macro hygiene). A macro must be defined above its first
use in the file. (Procedural macros, such as `#[derive]`, are Rust
programs that run at compile time; they need their own crate, so they
cannot be written here.)

## Syntax
macro_rules! shout {
    ($($x:expr),+ $(,)?) => {{
        $( println!("{}!", $x); )+
    }};
}

## Task
Define `chant!` so the three calls in `main` compile. Print each verse as
`  verse: N` and make the whole call evaluate to the sum.

## Hint
"cannot find macro `chant`": define it with `macro_rules!` above `main`.
```

**Hints.**
1. "cannot find macro `chant`": define it with `macro_rules!` above `main`.
2. Match a list with `($($verse:expr),+ $(,)?)`; the `$(,)?` accepts the trailing comma in `chant!(10, 20,)`.
3. Use `{{ ... }}` so the expansion is one block expression: `let mut total = 0; $( let verse = $verse; println!("  verse: {verse}"); total += verse; )+ total`.

**Starter** (fails: the macro does not exist):

```rust
// Define a declarative macro named `chant` above main. It should accept
// one or more comma-separated expressions, print each as "  verse: N",
// and evaluate to their sum.

fn main() {
    let first = chant!(1);
    println!("first chant: {first}");
    let second = chant!(2, 3, 4);
    println!("second chant: {second}");
    let third = chant!(10, 20,);
    println!("third chant: {third}");
}
```

```text
error: cannot find macro `chant` in this scope
  --> start/macro_rules.rs:10:17
   |
10 |     let third = chant!(10, 20,);
   |                 ^^^^^

error: cannot find macro `chant` in this scope
 --> start/macro_rules.rs:8:18
  |
8 |     let second = chant!(2, 3, 4);
  |                  ^^^^^

error: cannot find macro `chant` in this scope
 --> start/macro_rules.rs:6:17
  |
6 |     let first = chant!(1);
  |                 ^^^^^

error: aborting due to 3 previous errors

```

**Canonical solution:**

```rust
// `chant!` takes one or more expressions separated by commas, prints each
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
```

**Expected stdout:**

```text
  verse: 1
first chant: 1
  verse: 2
  verse: 3
  verse: 4
second chant: 9
  verse: 10
  verse: 20
third chant: 30
```

**Grader rule:**

```rust
MissionRule {
    id: "macro_rules",
    npc: "the Macro Monk keeps his silence",
    checks: &[
        check(&["macro_rules!"], "define the macro with `macro_rules!`"),
        check(&["$("], "accept a list of verses with a repetition: `$( ... ),+`"),
    ],
    pass: "the Macro Monk lifts the woodblock. \"carve once, print as often as you like.\"",
},
```

**Wrong variant that must fail** (matches the list, uses only the first
verse; compiles, passes the tokens):

```rust
// Matches the whole list but only ever uses the first verse.
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
```

```text
  verse: 1
first chant: 1
  verse: 2
second chant: 2
  verse: 10
third chant: 10
```

**References.** Rust Book ch. 20.5 "Macros" → "Declarative Macros with
`macro_rules!`"; Reference "Macros By Example" (repetitions, hygiene,
textual scope); The Little Book of Rust Macros.

---

### 61 — `drop_trait` (The Sexton)

**NPC.** The Sexton: keeps the candles of the reading room and snuffs each
one when its reader leaves, last lit first out. "Every soul is released.
Unless you forget it."

**Framing.** The curriculum's Drop Ritual: a resource that must release
its soul when its owner is gone, and the order in which that happens.

**Prompt.** "Implement `Drop`, drop one value early, and keep another from ever dropping."

**Tutorial draft.**

```text
## Concept
`Drop::drop` runs automatically when a value's owner goes out of scope.
Locals drop in reverse order of declaration; a struct's fields drop in
declaration order, after the struct's own `drop`. You may not call
`x.drop()` yourself (E0040). To end a value early, move it into
`drop(x)`: `std::mem::drop` is an ordinary function that takes
ownership. `let _ = Soul { .. };` binds nothing, so the new value is
dropped at the end of that statement. Drop is not guaranteed to run:
`std::mem::forget` skips it (this is safe; leaking is not undefined
behaviour), `Rc` cycles leak, `std::process::exit` skips destructors,
and a panic that aborts instead of unwinding drops nothing. This sandbox
builds with `panic = "abort"`, so a panic here runs no destructors.

## Syntax
impl Drop for Soul {
    fn drop(&mut self) { println!("{} is released", self.name); }
}
drop(early);            // ends `early` now
std::mem::forget(kept); // its drop never runs

## Task
Release `early` before "the ritual begins", drop an unnamed soul, let
`_inner` end with its own scope, keep `bound` from ever being released,
and print "main ends".

## Hint
E0040: destructors are called for you. `drop(early);` moves the value
into `std::mem::drop`, which ends it there.
```

**Hints.**
1. E0040: destructors are called for you. `drop(early);` moves the value into `std::mem::drop`, which ends it there.
2. `let _ = Soul { name: "unnamed" };` is dropped at once; `let _inner = ...` lives to the end of its block. A leading underscore on a name only silences the unused-variable warning; it still binds.
3. `std::mem::forget(bound);` takes ownership and never runs `drop`. The last two lines come from `_second` then `_first`: reverse declaration order.

**Starter** (fails: E0040, the intended reason; lacks `forget(`):

```rust
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
    let _bound = Soul { name: "bound" };

    // Release `early` now, before the ritual begins.
    early.drop();
    println!("the ritual begins");

    // Then: an unnamed soul, an inner scope, keep `bound` from ever being
    // released, and print "main ends".
}
```

```text
error[E0040]: explicit use of destructor method
  --> start/drop_trait.rs:18:11
   |
18 |     early.drop();
   |           ^^^^ explicit destructor calls not allowed
   |
help: consider using `drop` function
   |
18 -     early.drop();
18 +     drop(early);
   |

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0040`.
```

**Canonical solution:**

```rust
use std::mem;

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
```

**Expected stdout:**

```text
early is released
the ritual begins
unnamed is released
inner scope ends
inner is released
main ends
second is released
first is released
```

**Grader rule:**

```rust
MissionRule {
    id: "drop_trait",
    npc: "the Sexton holds the snuffer",
    checks: &[
        check(&["impl Drop for"], "give `Soul` a destructor: `impl Drop for Soul`"),
        check(&["forget("], "keep `bound` from ever being released with `std::mem::forget`"),
    ],
    pass: "the Sexton snuffs the candles in order. \"last lit, first out — and one left burning.\"",
},
```

**Wrong variant that must fail** (forgets the wrong soul, drops the one
that should be kept; compiles, passes the tokens):

```rust
use std::mem;

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
```

```text
the ritual begins
unnamed is released
inner scope ends
inner is released
bound is released
main ends
second is released
first is released
```

**Hard rule 3 notes for voice-editor.** Do not write "Drop always runs" or
"Rust never leaks". `let _ = existing_variable;` does *not* move or drop
the variable (the `_` pattern binds nothing and the place is not moved);
only a fresh value expression, as in this mission, is dropped at once.
Edition 2024 changed when some temporaries in tail expressions and
`if let` scrutinees drop; nothing in this mission depends on that, and the
stdout is identical under 2021 and 2024 (verified).

**References.** Rust Book ch. 15.3 "Running Code on Cleanup with the
`Drop` Trait"; Reference "Destructors" (drop scopes, `_` patterns);
Rustonomicon "Leaking"; `std::mem::forget` docs.

---

### 62 — `send_sync` (The Courier)

**NPC.** The Courier: carries sealed tomes between the Library's towers and
refuses any parcel nobody will vouch for. "I do not open parcels. I take
your word for what is inside."

**Framing.** A tome that owns its page through a raw pointer cannot leave
the room until someone makes, and justifies, the promise the compiler
will not make on its own.

**Prompt.** "Promise, with `unsafe impl`, that `Tome` is `Send` and `Sync`, and justify both."

**Tutorial draft.**

```text
## Concept
`Send` means a value can be moved to another thread. `Sync` means a `&T`
can be shared between threads; exactly, `T: Sync` when `&T: Send`. Both
are auto traits: a struct has them when all its fields do. Raw pointers
have neither, so `Tome` has neither. `Rc` has neither (its count is not
atomic); `Cell` and `RefCell` are `Send` but not `Sync`. When you know
more than the compiler, `unsafe impl Send for T {}` is a promise. If it
is false, safe code can cause a data race, which is undefined behaviour.
This sandbox has no threads (`thread::spawn` panics on wasm32-wasip1),
so the Courier checks the promise where Rust always checks it: at
compile time, through `T: Send` and `T: Sync` bounds.

## Syntax
// SAFETY: why this is true
unsafe impl Send for Tome {}
unsafe impl Sync for Tome {}
fn needs_send<T: Send>(x: T) -> T { x }

## Task
Make `Tome` acceptable to `courier_accepts` and `readers_accept`, with a
`// SAFETY:` comment for each promise.

## Hint
E0277 names the culprit: `*mut u32` is neither `Send` nor `Sync`, and it
is inside `Tome`.
```

**Hints.**
1. E0277 names the culprit: `*mut u32` is neither `Send` nor `Sync`, and it is inside `Tome`.
2. `Tome` owns its page alone (made by `Box::into_raw`, freed once in `Drop`), like a `Box<u32>`. Handing all of that to another thread is fine.
3. `unsafe impl Send for Tome {}` and `unsafe impl Sync for Tome {}`. `Sync` holds because nothing can change the page through `&Tome`.

**Starter** (fails: two E0277, the intended reason):

```rust
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

// The courier and the readers refuse the tome. Decide whether it is sound
// to promise that a Tome may move to, and be shared between, threads.

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
```

```text
error[E0277]: `*mut u32` cannot be sent between threads safely
  --> start/send_sync.rs:37:35
   |
37 |     let carried = courier_accepts(tome);
   |                   --------------- ^^^^ `*mut u32` cannot be sent between threads safely
   |                   |
   |                   required by a bound introduced by this call
   |
   = help: within `Tome`, the trait `Send` is not implemented for `*mut u32`
note: required because it appears within the type `Tome`
  --> start/send_sync.rs:1:8
   |
 1 | struct Tome {
   |        ^^^^
note: required by a bound in `courier_accepts`
  --> start/send_sync.rs:27:23
   |
27 | fn courier_accepts<T: Send>(item: T) -> T {
   |                       ^^^^ required by this bound in `courier_accepts`

error[E0277]: `*mut u32` cannot be shared between threads safely
  --> start/send_sync.rs:39:33
   |
39 |     let shared = readers_accept(&carried);
   |                  -------------- ^^^^^^^^ `*mut u32` cannot be shared between threads safely
   |                  |
   |                  required by a bound introduced by this call
   |
   = help: within `Tome`, the trait `Sync` is not implemented for `*mut u32`
note: required because it appears within the type `Tome`
  --> start/send_sync.rs:1:8
   |
 1 | struct Tome {
   |        ^^^^
note: required by a bound in `readers_accept`
  --> start/send_sync.rs:31:22
   |
31 | fn readers_accept<T: Sync>(item: &T) -> &T {
   |                      ^^^^ required by this bound in `readers_accept`

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0277`.
```

**Canonical solution:**

```rust
/// A tome that owns one heap page through a raw pointer, the way `Box`
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
```

**Expected stdout:**

```text
the courier accepts the tome
page reads 7
the page is returned to the shelf
```

**Why this is sound.** `Send`: a `Tome` uniquely owns one heap `u32`
(it is not `Clone`, and the pointer never escapes), so moving a `Tome`
moves sole ownership, exactly as for `Box<u32>`; `u32: Send`; the system
allocator may free memory on a different thread from the one that
allocated it. `Sync`: the only operation through `&Tome` is `read`, a
plain read; concurrent reads of an unchanging `u32` are not a data race.
`read` is valid because `page` comes from `Box::into_raw` and is freed only
in `Drop`, once.

**Grader rule:**

```rust
MissionRule {
    id: "send_sync",
    npc: "the Courier will not carry it",
    checks: &[
        check(&["unsafe impl Send"], "promise the tome may move between threads: `unsafe impl Send for Tome {}`"),
        check(&["unsafe impl Sync"], "promise it may be shared by reference: `unsafe impl Sync for Tome {}`"),
    ],
    pass: "the Courier shoulders the satchel. \"you vouched for it, so I carry it.\"",
},
```

**Wrong variant that must fail** (both promises, but no `Drop`: the page
leaks; compiles, passes the tokens, misses the last line):

```rust
// Makes both promises but forgets to free the page: it leaks, and the
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
```

```text
the courier accepts the tome
page reads 7
```

**References.** Rust Book ch. 16.4 "Extensible Concurrency with `Send` and
`Sync`"; Rustonomicon "Send and Sync"; `std::marker::Send` / `Sync` docs
(the `&T: Send` equivalence).

---

### 63 — `maybe_uninit` (The Bookbinder)

**NPC.** The Bookbinder: sews blank quires and lets no one read a page
before it is written. "A blank page is not an empty page. It is a page
that does not exist yet."

**Framing.** A quire of four pages must be bound before it is filled,
and filled before anyone reads it.

**Prompt.** "Fill a `MaybeUninit<[u32; 4]>` through a raw pointer, then `assume_init`."

**Tutorial draft.**

```text
## Concept
Safe Rust never lets you read memory that has not been initialised
(E0381). `MaybeUninit<T>` is the opt-out: storage for a `T` that may not
hold a valid `T` yet, about which the compiler assumes nothing. You fill
it through a raw pointer (or the safe `MaybeUninit::write`), then call
`assume_init()`, which is `unsafe`: you promise every byte is
initialised and forms a valid `T`. Calling it early is undefined
behaviour even for plain integers; an uninitialised `u32` is not "some
random number". Fill with `ptr.write(v)`, not `*ptr = v`: assignment
first drops the old value, and dropping uninitialised memory is undefined
behaviour for any type with a destructor. `MaybeUninit` never drops what
it holds.

## Syntax
let mut slot = MaybeUninit::<[u32; 4]>::uninit();
let p: *mut u32 = slot.as_mut_ptr().cast::<u32>();
unsafe { p.add(i).write(v) };            // i < 4
let arr = unsafe { slot.assume_init() };  // after every write

## Task
Fill all four pages of a `MaybeUninit<[u32; 4]>` through a raw pointer,
then `assume_init` and print the pages and their total.

## Hint
E0381: you read the array before every element was set, and safe Rust
cannot prove a loop filled it.
```

**Hints.**
1. E0381: the array is read before safe Rust can prove every element was set.
2. `let mut quire = MaybeUninit::<[u32; PAGES]>::uninit(); let first = quire.as_mut_ptr().cast::<u32>();`
3. `unsafe { first.add(i).write((i as u32 + 1) * 10) }` for each `i` in `0..PAGES`, then `let quire = unsafe { quire.assume_init() };` after the loop, never before.

**Starter** (fails: E0381, the intended reason):

```rust
const PAGES: usize = 4;

fn main() {
    // Declare the quire now, fill each page in the loop, read it after.
    let quire: [u32; PAGES];
    for i in 0..PAGES {
        quire[i] = (i as u32 + 1) * 10;
    }
    println!("bound pages: {quire:?}");
    println!("total ink: {}", quire.iter().sum::<u32>());
}
```

```text
error[E0381]: used binding `quire` isn't initialized
 --> start/maybe_uninit.rs:9:29
  |
5 |     let quire: [u32; PAGES];
  |         ----- binding declared here but left uninitialized
...
9 |     println!("bound pages: {quire:?}");
  |                             ^^^^^ `quire` used here but it isn't initialized
  |
help: consider assigning a value
  |
5 |     let quire: [u32; PAGES] = [42; 4];
  |                             +++++++++

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0381`.
```

rustc reports the read on the `println!` line, not the element writes in
the loop. The hint is worded to match.

**Canonical solution:**

```rust
use std::mem::MaybeUninit;

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
```

**Expected stdout:**

```text
bound pages: [10, 20, 30, 40]
total ink: 100
```

**Why this is sound.** `first` points at element 0 of a `[u32; 4]`
allocation; for `i < 4`, `first.add(i)` stays inside it, and `u32` needs no
extra alignment beyond the array's. `write` stores without reading or
dropping the old bytes. After the loop all 16 bytes are initialised and
every bit pattern is a valid `u32`, so `assume_init` is justified.

**Grader rule:**

```rust
MissionRule {
    id: "maybe_uninit",
    npc: "the Bookbinder holds up a blank page",
    checks: &[
        check(&["MaybeUninit"], "bind the quire blank with `MaybeUninit`"),
        check(&["write("], "fill each page with `write`, which never reads or drops the old bytes"),
        check(&["assume_init", "transmute"], "declare it filled with `assume_init` once every page is written"),
    ],
    pass: "the Bookbinder closes the quire. \"every page written before anyone read one.\"",
},
```

**Alternative that must pass** (array of `MaybeUninit<u32>`, safe
`MaybeUninit::write`, then `transmute`; verified, same stdout):

```rust
use std::mem::MaybeUninit;

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
```

**Wrong variant that must fail** (`zeroed()`, last page never written:
sound because all-zero is a valid `u32`, and wrong):

```rust
use std::mem::MaybeUninit;

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
```

```text
bound pages: [10, 20, 30, 0]
total ink: 60
```

The tempting *unsound* variant (`uninit()` and a loop to `PAGES - 1`) is
undefined behaviour; it is not used as a grading example because its
output is meaningless. The tutorial says why.

**References.** `std::mem::MaybeUninit` docs (initialization invariant,
"Initializing an array element-by-element"); Rustonomicon "Unchecked
Uninitialized Memory"; Reference "Behavior considered undefined" (invalid
values, uninitialized memory).

---

## Boss trial — The Librarian (missions 64–66)

A boss is a staged chained mission (finish plan D1): one NPC, one arena
prop, three missions that must be cleared in order. No combat, no turns,
no Sanity meter. The Librarian is the curriculum's boss, kept whole in
stage 2 (the macro that expands to his exact demand); stages 1 and 3 test
the act's other two pillars.

**NPC.** The Librarian: ancient, patient, and literal. Guards a lectern
with a chained codex whose pages flicker with corruption until the trial
is won. "Everything in here is real. The compiler stopped protecting you
at the door. Show me you can protect yourself."

**How the stages chain.** Three registry entries with linear prereqs, all
owned by the one Librarian NPC. Talking to him opens the first uncleared
stage. That NPC-to-several-missions lookup is new engine behaviour; it is
track C3's job (the Borrow Checker trial needs it too) and is not designed
here.

### 64 — `librarian_index` (trial 1/3): the safe door

**Concept.** `unsafe fn` with a `# Safety` contract, and a safe wrapper
whose checks make the contract impossible to break. Soundness: a safe
function is sound only if *no* argument can cause undefined behaviour.

**Prompt.** "Make `fetch` safe for every index: check bounds, then call the `unsafe fn`."

**Tutorial draft.**

```text
## Concept
An `unsafe fn` has a contract the compiler cannot check, written in a
`# Safety` doc section. Calling it is `unsafe`: the caller promises to
keep the contract. The art of unsafe Rust is to wrap such a function in
a safe one whose own checks keep the contract for every possible input.
That wrapper is sound if no caller, using only safe code, can cause
undefined behaviour through it. Pointer arithmetic is where this
matters: `ptr.add(i)` is only defined while the result stays inside the
same allocation.

## Task
Make `fetch` safe to call with any index: check it against the shelf's
length, and only then call `volume_at` inside `unsafe`.

## Hint
The compiler stops you at the call: `volume_at` is an `unsafe fn`. But
wrapping the call in `unsafe` alone is not enough; index 99 must not read.
```

**Hints.**
1. The call to `volume_at` needs `unsafe`, and `unsafe` alone is not enough: index 99 must never be read.
2. `if index < shelf.len() { ... } else { None }`.
3. Inside the `if`: `Some(unsafe { volume_at(shelf.as_ptr(), index) })`, with a `// SAFETY:` comment that names the check.

**Starter** (fails: E0133; no bounds check):

```rust
/// Returns the volume `index` places past `shelf`.
///
/// # Safety
///
/// `shelf` must point into a live array of initialised `u32`s, and
/// `shelf.add(index)` must still be inside that same array.
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
    unsafe { *shelf.add(index) }
}

// The Librarian wants a door that is safe to open with ANY index.
fn fetch(shelf: &[u32], index: usize) -> Option<u32> {
    Some(volume_at(shelf.as_ptr(), index))
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
```

```text
error[E0133]: call to unsafe function `volume_at` is unsafe and requires unsafe block
  --> start/librarian_index.rs:13:10
   |
13 |     Some(volume_at(shelf.as_ptr(), index))
   |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0133`.
```

**Canonical solution:**

```rust
/// Returns the volume `index` places past `shelf`.
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
```

**Expected stdout:**

```text
index 0: volume 11
index 3: volume 44
index 4: no such volume
index 99: no such volume
```

**Why this is sound.** `volume_at` is only called when `index <
shelf.len()`, so `shelf.as_ptr().add(index)` is inside the borrowed slice,
which is alive for the call. Out-of-range indexes return `None` without
touching memory. The inner `unsafe { }` inside the `unsafe fn` compiles
cleanly in both editions (it is required style in 2024, where
`unsafe_op_in_unsafe_fn` warns).

**Grader rule:**

```rust
MissionRule {
    id: "librarian_index",
    npc: "the Librarian blocks the stacks",
    checks: &[
        check(&["unsafe fn"], "keep `volume_at` an `unsafe fn` with its `# Safety` contract"),
        check(&[".len()"], "check the index against the shelf's `.len()` before the unsafe call"),
    ],
    pass: "the Librarian steps aside. \"a door no index can break.\"",
},
```

**Wrong variant that must fail** (sound but too strict, refuses the last
volume):

```rust
unsafe fn volume_at(shelf: *const u32, index: usize) -> u32 {
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
```

```text
index 0: volume 11
index 3: no such volume
index 4: no such volume
index 99: no such volume
```

The tempting *unsound* variant `index <= shelf.len()` reads one past the
end for index 4: undefined behaviour, never used as a grading example.

**References.** Rust Book ch. 20.1 → "Creating a Safe Abstraction over
Unsafe Code"; Rustonomicon "Working with Unsafe"; `pointer::add` docs
(safety conditions); Rust API Guidelines, `# Safety` sections.

### 65 — `librarian_demand` (trial 2/3): the incantation

**Concept.** The curriculum's boss puzzle: read a verbose demand and write
a recursive `macro_rules!` that expands to exactly that expression, using
internal `@acc` rules to carry the partial result.

**Prompt.** "Grow `incant!` so `incant!(1, 2, 3, 4)` expands to the Librarian's exact demand."

**Tutorial draft.**

```text
## Concept
A macro can call itself. To build a left-nested expression such as
seal(seal(seal(a, b), c), d), carry the part built so far in an
accumulator: an internal rule marked with a literal token like `@acc`
that callers never write. Order matters: macro_rules tries rules top to
bottom, and a rule that starts parsing `$x:expr` and fails is an error,
not a fallthrough, so put the `@acc` rules first.

## Task
Make `incant!(a, b, c, d)` expand to
seal(seal(seal(page(a), page(b)), page(c)), page(d)), for any number of
arguments, one or more.

## Hint
"no rules expected `,`": your only rule takes exactly one expression.
```

**Hints.**
1. "no rules expected `,`": the only rule takes exactly one expression.
2. Entry rule: `($first:expr $(, $rest:expr)*) => { incant!(@acc page($first) $(, $rest)*) }`.
3. Two internal rules, listed first: `(@acc $acc:expr) => { $acc }` and `(@acc $acc:expr, $next:expr $(, $rest:expr)*) => { incant!(@acc seal($acc, page($next)) $(, $rest)*) }`.

**Starter** (fails: macro arity, the intended reason):

```rust
fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// Grow this macro so incant!(a, b, c, d) expands to
//     seal(seal(seal(page(a), page(b)), page(c)), page(d))
// and works for any number of arguments, one or more.
macro_rules! incant {
    ($first:expr) => {
        page($first)
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
```

```text
error: no rules expected `,`
  --> start/librarian_demand.rs:20:27
   |
12 | macro_rules! incant {
   | ------------------- when calling this macro
...
20 |     let answer = incant!(1, 2, 3, 4);
   |                           ^ no rules expected this token in macro call
   |
note: while trying to match meta-variable `$first:expr`
  --> start/librarian_demand.rs:13:6
   |
13 |     ($first:expr) => {
   |      ^^^^^^^^^^^

error: no rules expected `,`
  --> start/librarian_demand.rs:25:44
   |
12 | macro_rules! incant {
   | ------------------- when calling this macro
...
25 |     println!("incant!(2, 9): {}", incant!(2, 9));
   |                                            ^ no rules expected this token in macro call
   |
note: while trying to match meta-variable `$first:expr`
  --> start/librarian_demand.rs:13:6
   |
13 |     ($first:expr) => {
   |      ^^^^^^^^^^^

error: aborting due to 2 previous errors

```

**Canonical solution:**

```rust
fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// incant!(a, b, c, d) must expand to the Librarian's exact demand:
//     seal(seal(seal(page(a), page(b)), page(c)), page(d))
// The internal `@acc` rules carry the expression built so far. They come
// first because macro_rules tries rules top to bottom, and a rule that
// starts parsing `$x:expr` and fails is an error, not a fallthrough.
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
```

**Expected stdout:**

```text
the demand: 78
incant!(1, 2, 3, 4): 78
the incantation matches: true
incant!(7): 21
incant!(2, 9): 39
```

`seal` is deliberately not commutative or associative, so a right fold
gives a different number.

**Grader rule:**

```rust
MissionRule {
    id: "librarian_demand",
    npc: "the Librarian repeats his demand",
    checks: &[
        check(&["macro_rules!"], "answer with a `macro_rules!` macro"),
        check(&["$("], "take any number of arguments with a repetition: `$( ... )*`"),
    ],
    pass: "the Librarian hears his words returned. \"the incantation, exactly.\"",
},
```

**Wrong variant that must fail** (folds from the right; compiles, passes
the tokens):

```rust
fn page(n: u32) -> u32 {
    n * 3
}

fn seal(left: u32, right: u32) -> u32 {
    left * 2 + right
}

// Folds from the right: seal(page(a), seal(page(b), seal(page(c), page(d)))).
// Compiles, and is not what the Librarian demanded.
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
```

```text
the demand: 78
incant!(1, 2, 3, 4): 48
the incantation matches: false
incant!(7): 21
incant!(2, 9): 39
```

**References.** Reference "Macros By Example"; The Little Book of Rust
Macros, "Internal rules" and "Push-down accumulation".

### 66 — `librarian_seal` (trial 3/3): the codex

**Concept.** A type that owns heap memory through a raw pointer, the way
`Vec` does underneath: `Box::into_raw` gives up ownership, so the type's
`Drop` must rebuild the exact `Box` with `Box::from_raw`, exactly once.

**Prompt.** "Give `Codex` a bounds-checked `get` and a `Drop` that returns its pages."

**Tutorial draft.**

```text
## Concept
`Box::into_raw` turns a box into a raw pointer and gives up ownership:
nothing will free that memory unless you rebuild the same box with
`Box::from_raw` (same address, same type, same length for a slice) and
let it drop. Do it twice and you free memory twice; never and it leaks.
A raw pointer owns nothing; the struct that holds it must. That is what
`Vec` and `Box` do inside the standard library.

## Task
Write `get` so it returns `Some(page)` for valid indexes and `None`
otherwise without reading outside the codex, and implement `Drop` so the
pages go back to the heap and the codex prints its seal line.

## Hint
Nothing frees the pages: `Box::into_raw` handed them to you.
```

**Hints.**
1. Nothing frees the pages: `Box::into_raw` handed them to you, and `get` still returns `None`.
2. `get`: `if index < self.len { Some(unsafe { *self.ptr.add(index) }) } else { None }`.
3. `impl Drop for Codex`: rebuild the slice pointer with `std::ptr::slice_from_raw_parts_mut(self.ptr, self.len)`, then `drop(unsafe { Box::from_raw(slice) })`, then print.

**Starter** (compiles; wrong output, the intended reason: nothing is read
and nothing is freed). Under `-D warnings` it fails with
`dead_code` ("fields `ptr` and `len` are never read"); the server
prepends `#![allow(dead_code, unused)]`, so there it compiles and prints:

```rust
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

    // Return the page at `index`, or None past the end, without ever
    // reading outside the codex.
    fn get(&self, _index: usize) -> Option<u32> {
        None
    }
}

// The pages were handed out with Box::into_raw. Nothing gives them back.

fn main() {
    let codex = Codex::from_pages(vec![5, 8, 13]);
    println!("page 0: {:?}", codex.get(0));
    println!("page 2: {:?}", codex.get(2));
    println!("page 3: {:?}", codex.get(3));
    println!("the Librarian closes the book");
}
```

```text
page 0: None
page 2: None
page 3: None
the Librarian closes the book
```

**Canonical solution:**

```rust
/// Owns a run of `len` u32s through a raw pointer, the way `Vec` does
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
```

**Expected stdout:**

```text
page 0: Some(5)
page 2: Some(13)
page 3: None
the Librarian closes the book
the codex is sealed (3 pages returned)
```

**Why this is sound.** `from_pages` leaks a `Box<[u32]>` of `len`
elements and keeps its data pointer and length. `get` dereferences only
for `index < len`, inside that allocation, which lives until `Drop`.
`Drop` rebuilds a `*mut [u32]` with the same address and length, so
`Box::from_raw` reconstructs exactly the box that was leaked, with the
same layout; `Drop` runs at most once and `Codex` is not `Clone`, so the
memory is freed exactly once. An empty `Vec` gives a dangling, non-null
pointer with `len == 0`; `get` never dereferences it, and `Box::from_raw`
of a zero-length slice at that address is the documented round trip.

**Grader rule:**

```rust
MissionRule {
    id: "librarian_seal",
    npc: "the Librarian holds the codex open",
    checks: &[
        check(&["impl Drop for"], "seal the codex: `impl Drop for Codex`"),
        check(&["from_raw"], "give the pages back to the heap with `Box::from_raw`"),
    ],
    pass: "the Librarian closes the codex. \"borrowed from the heap, returned to it. the Library is yours.\"",
},
```

**Wrong variant that must fail** (correct `get`, no `Drop`: leaks, misses
the seal line):

```rust
// A correct `get`, but no Drop: the three pages leak and the codex is
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
```

```text
page 0: Some(5)
page 2: Some(13)
page 3: None
the Librarian closes the book
```

A `Drop` that prints the line without calling `from_raw` would pass the
stdout check and is caught only by the `from_raw` token. A `Drop` that
calls `Box::from_raw(self.ptr)` on the thin pointer frees with the wrong
layout: undefined behaviour, not used as an example.

**References.** `Box::into_raw` / `Box::from_raw` docs; Rustonomicon
"Implementing Vec" (allocating, deallocating); `std::ptr::slice_from_raw_parts_mut`.

---

## NPCs and placement

Seven `NpcSpec` entries, appended to `NPC_ROSTER` after the Ghostkeeper in
this order: Shelfmark, Envoy, Macro Monk, Sexton, Courier, Bookbinder,
Librarian. All names are unique against the current 57-entry roster
(checked).

**The village is full.** `world.rs` `DISTRICTS` is a hub (`count: 9`) and
eight quarters (`count: 6` each), summing to the 57-NPC roster.
`district_slot` assigns positions by roster index, so a 58th NPC today
falls through to the "safety net" row along the south edge. Two more
groups need two more districts, and there is no free room on the current
map: the map is 180×140 tiles (world half-extents 1440×1120), the ring
quarters already sit at x = ±960 and y = ±800 with 260-unit plazas
(reaching y = ±1060), and the corner quarters at (±720, ±700) fill the
diagonals.

**Proposal.** Grow `MAP_H` from 140 to 220 tiles (half-height 1760) and
extend the north–south main street to y = ±1400. Add two districts at the
ends of that street:

- **The Forbidden Library** at (0, −1400), south beyond The Old Gate (the
  Library is sealed, so it lies past the gate). `count: 7` (six plus the
  Librarian).
- **The Throne** at (0, +1400), north beyond Tavern Row, for Act 10.

With a 260-unit plaza, each spans 1140..1660, leaving 100 units (6 tiles)
of margin for the treeline. A 3-column grid of 7 at the existing 240
spacing needs 3 rows (−240, 0, +240), which fits the plaza. Appending the
two districts last in `DISTRICTS` makes `district_slot` put the Act 9
roster block in the Library and the Act 10 block in the Throne with no
change to the slot function. Each new district also needs 2–4
`COTTAGE_SPOTS` entries (`prop.rs` test `district_gets_two_to_four_cottages`)
— for the Library, the landmark below should stand in for one cottage
side. The roster, the districts and the map size must land in the same
commit, or NPCs fall to the safety-net row.

Alternative if a taller map is unwanted: widen `MAP_W` to 240 and put the
two districts at (±1560, 0) on the east–west street. This keeps the map
square-ish but puts the climax beside Market Row instead of at the end of
the main road.

## Art (tranche T5)

For the designer (Fable). Sprites are 32×32 per the bible, except the
Librarian at 64×64 (boss resolution, with intro and defeat frames per the
bible's boss rule). Zone roles from the bible's zone table: **Royal arcane,
Crypt, Aged paper, Alarm scarlet (corruption glitches only)**. Outline
Coalblack as everywhere.

**Palette conflict to resolve (open question 3).** The zone table makes
Royal arcane dominant for Act 9, but the bible's scarcity rule caps arcane
violet at under 5% of any frame (51 px of a 32×32 frame). The Act 4 kit
solved the same conflict by keeping violet to 10 px. Proposed: robe mass
in **Crypt** (the deep heraldic shadow, which reads violet-black next to
Aged paper), **Royal arcane** only on hood lining, trim and sigils within
the 51 px cap, **Aged paper** for scapulars, pages and faces.

**Costume-kit rule for the Library cast** (header text for a REF-75 kit
exemplar): deep-hooded Crypt robe falling straight to the floor, Aged-paper
scapular down the front, Royal-arcane hood lining and one sigil, face half
shadowed by the hood (two Aged-paper eye pixels), a chain at the belt.
**Silhouette = a hooded column:** rounded hood peak, no flare at the hem.
Distinct from the Tower kit (pointed hat over a flaring triangle robe), the
Vault kit (armoured, faceless), and the Acts 3–8 casts. No Alarm scarlet on
cast sprites.

| NPC | Role | Silhouette idea | Lesson prop |
|---|---|---|---|
| The Shelfmark | catalogue keeper | kit column, a little stooped | a long pointer rod ending in a blank call-number card: the address, not the book |
| The Envoy | treaty-bearer | kit column with a broader, squared Aged-paper tabard | one scroll with two different seals on one cord: two languages, one agreement |
| The Macro Monk | silent copyist | kit column, hood fully up, hands in sleeves | a carved woodblock stamp with a repeated pattern on its face |
| The Sexton | candle-keeper | kit column, shorter and broad | a long candle snuffer, one candle still lit at the hip (the forgotten soul) |
| The Courier | tome-carrier | kit column, robe kilted up to show boots | a strapped book-satchel with a wax seal |
| The Bookbinder | binder | kit column, sleeves rolled, Aged-paper apron over the scapular | an open blank quire with a needle and thread |
| The Librarian (64×64) | boss | very tall hooded column, hood brushing the frame top, a catalogue-drawer chest | arena prop below |

**Arena prop (Librarian):** a lectern with a chained codex, about 48×48.
The only Alarm-scarlet pixels in the act: a few glitch scanlines across the
codex's pages, removed in a "sealed" variant after stage 3 (two frames).

**District landmark (about 64×64):** "The Sealed Stacks" — a squat stone
book-tower door, shelves visible through a barred arch, a heavy chain and
lock across it, a Royal-arcane glow in the keyhole. Bottom-centre anchor,
like the REF-68 cottage.

## Touch-points (the batch recipe)

1. `game/src/plugins/mission.rs` — 9 `Mission` entries after `weak_ref`
   (linear prereq is automatic). Starters exactly as above.
2. `compile-api/src/grader.rs` and `game/src/plugins/stub_grader.rs` — the
   9 `MissionRule` entries above, textually identical in both
   `BEGIN/END MISSION TABLE` regions; per-mission unit tests (canonical
   passes, starter fails, one missing-token case each).
3. `game/src/plugins/npc.rs` — 7 `NpcSpec` (the Librarian maps to stage 1
   until C3 lands multi-stage NPCs; see open question 1).
4. Art: `design/art/refs/ref-75-librarykit.jsx` (kit exemplar, DRAFT
   header) then ref-76..82 for the cast, the arena prop and the landmark;
   render, stage on the canvas, wire only after Matt approves. Then
   `game/src/assets.rs` `SPRITE_*` + `ALL_SPRITE_PATHS`.
5. `game/src/plugins/editor/solutions/data.rs` — the 9 canonical solutions
   verbatim (they contain `unsafe`; they are string data, see hard rule 1
   above). `game/tests/contract.rs` and `game/tests/stub_grader.rs`
   canonical fixtures.
6. Server expectations — the D2 per-mission expected-stdout table gets the
   9 expected outputs above, exact match. Add none of the 9 to
   `EXPECTED_SANDBOX_RUN_FAILURES`: all run cleanly in the sandbox.
7. Docs — `design/01-curriculum.md` Act 9 "Code status" note (including
   the substitutions table), `design/04b-art-deliverables.md`, this spec's
   status line.

Plus `world.rs` / `prop.rs` for the district (above) and a `voice-editor`
pass.

## Correctness contract (must stay green)

Everything in the Act 3 spec's contract, plus:

- `game/tests/solutions.rs --ignored` builds every solution with
  `--edition 2021`. The server builds with 2024. Until the test also builds
  with `--edition 2024` (recommended; see open question 2), a solution can
  pass the test and fail on the server, which is exactly what the first
  draft of `extern_c` did.
- The 9 starters each fail as recorded (compile error or wrong stdout).

## Acceptance criteria

- 66 missions; the 9 new ones reachable after the prior 57, the three
  Librarian stages in order.
- `scripts/ci.ps1` green; `cargo test -p pledge_and_crown --test solutions
  -- --ignored` green for all 66.
- No `unsafe` outside string literals in `game/` or `compile-api/` (a
  `grep -n "unsafe" game/src compile-api/src` diff against the batch shows
  only string-literal hits).
- 66 missions, 64 NPCs, all with approved art (or explicitly listed as
  awaiting approval).

## Implementer checklist

- [ ] Read hard rules 1–4 and this spec's "Hard rule 1" section.
- [ ] `mission.rs`: 9 entries, ids/NPC names/prompts/tutorials/starters as above.
- [ ] `grader.rs` + `stub_grader.rs`: 9 identical `MissionRule`s; unit tests.
- [ ] `npc.rs`: 7 `NpcSpec`s in the order above.
- [ ] `world.rs`: `MAP_H` 220, N–S road to ±1400, two districts appended (Library `count: 7`, Throne `count: 7`); `prop.rs` cottage spots. Same commit as the roster.
- [ ] `solutions/data.rs`, `contract.rs`, `tests/stub_grader.rs`: canonicals verbatim.
- [ ] D2 expected-stdout table: 9 entries.
- [ ] Re-run every solution through `rustc --edition 2021` and `--edition 2024`, native and `wasm32-wasip1`, and through `compile_and_run`; outputs must match this spec byte for byte. If the toolchain has moved past 1.99.0, re-record the starter diagnostics.
- [ ] If Miri is available: `cargo +nightly miri run` on each of the 9 solutions; record the result here.
- [ ] Art: REF-75 kit first; nothing wired before approval.
- [ ] `voice-editor` pass; `rust-lead` review after merge.
- [ ] Update `01-curriculum.md`, `04b`, this status line.

## Open questions for Matt

1. **Trial wiring.** Three registry entries owned by one NPC need the C3
   multi-stage NPC lookup. Until then, should the Librarian ship as three
   separate NPCs (Librarian, Librarian's Index, Librarian's Seal), or should
   Act 9 wait for C3?
2. **Edition.** The server compiles player code as edition 2024; the game's
   solution test, the brief for this spec, and probably players' mental
   model say 2021. Keep 2024 on the server and make `solutions.rs` build
   both? Or pin the server to 2021? Act 9 is the first act where the two
   differ in what compiles (`unsafe extern`, `#[unsafe(no_mangle)]`).
3. **Act 9 palette.** Royal arcane is the zone's dominant role but the
   bible caps arcane violet at 5% of a frame. Approve "Crypt robes, Royal
   arcane trim under the cap"?
4. **Map growth.** Approve growing the map north–south to 220 tiles with the
   Library south of The Old Gate and the Throne north of Tavern Row, or
   prefer the east–west alternative?
5. **Miri.** Install Miri on the nightly toolchain so the unsafe solutions
   are checked mechanically before they ship as "correct" answers? Until
   then the soundness notes above are reasoning, not proof.
6. **`static mut` and unions.** Two of the five unsafe superpowers are named
   but not exercised. Acceptable for v1.0?

## Implementation record (2026-11-15)

The open questions were decided by the orchestrator: the Librarian is a
trial on the existing mechanism (no prologue, his roster mission is stage
1); edition 2024 everywhere (`game/tests/common` pins it); the map grew
north-south as proposed, with the Throne built empty (`count: 0`); the
palette question was settled by the approved art (REF-79..85, REF-93, not
the REF-75..82 numbers above); Miri is still not installed, so **no Miri
run backs any `unsafe` solution here**: the "why this is sound" notes
stand in for it; `static mut` and unions stay named, not exercised.

### Re-verification

Every program (9 solutions, 9 starters, every wrong variant,
alternative and claim program the game and server tests use) was rebuilt
with `rustc 1.99.0 (b940084d7 2026-09-28)`, editions 2021 and 2024,
native and `--target wasm32-wasip1` (solutions, alternatives and wrong
variants with `-D warnings`), and run natively; stdout identical across
editions. Results matched this draft except as listed below. The server
path is `compile-api/tests/expectations_proof.rs` (real cargo build for
wasip1 + wasmtime) and `act9_unsafe_harness.rs`.

### Corrections to this draft

1. **Macro rule order (librarian_demand).** The draft's tutorial and
   solution comment said the `@acc` rules must come first because "a rule
   that starts parsing `$x:expr` and fails is an error, not a
   fallthrough". False for this macro: a literal `@` can never start an
   expression, so `macro_rules!` does not try to parse `@acc ...` as
   `$first:expr`; it moves on to the next rule. With the entry rule
   written FIRST, `incant!(1, 2)` compiles and prints `12`
   (`game/tests/act9_programs`, `CLAIM_LIBRARIAN_DEMAND_ORDER`, asserted
   in `mission_text.rs`). The tutorial now says the order does not matter
   here and why; the solution comment says the same.
2. **Tutorial sections.** The game requires `## Syntax` in every tutorial
   (the recap shows Concept + Syntax); the three trial drafts had none.
   Added: an `unsafe fn` with a `# Safety` doc, a `count!` macro with an
   `@acc` rule (`count!(5, 6, 7)` is `3`, asserted), and the
   `into_raw` / `slice_from_raw_parts_mut` / `from_raw` round trip.
3. **Claims cut rather than left unproven (hard rule 3).** `drop_trait`
   no longer says a struct's fields drop after its own `drop`, that `Rc`
   cycles leak, or that the sandbox's `panic = "abort"` runs no
   destructors; `send_sync` no longer says `Cell`/`RefCell` are `Send`
   but not `Sync`; `extern_c`'s hint no longer says "if you see 8,
   `repr(C)` is missing" (Rust's default layout is unspecified; it now
   says the size line can come out different). Every claim that remains
   is compiled or run in `game/tests/mission_text.rs`
   (`act9_villager_claims`, `trial_claims("librarian")`, pinned outputs,
   trial hint claims), including ones the draft did not have: two `&mut`
   inside `unsafe` is still E0499; a plain `extern "C" {` block is
   "extern blocks must be unsafe" and a plain `#[no_mangle]` is "unsafe
   attribute used without unsafe" in 2024; a macro used above its
   definition is "cannot find macro"; `$x` written twice runs twice;
   macro hygiene; `process::exit` runs no destructor; `Rc` is neither
   `Send` nor `Sync`; a `MaybeUninit` never drops its value; the
   `unsafe`-only `fetch` compiles; the right fold gives `48`. Statements
   that something is undefined behaviour cannot be run; they follow the
   std/Reference docs.
4. **Server grading.** Not "expected stdout first" for all nine.
   `raw_pointer`, `macro_rules`, `drop_trait`, `maybe_uninit` and
   `librarian_demand` are `Stdout` checks plus construct tokens (`uses`).
   `extern_c`, `send_sync`, `librarian_index` and `librarian_seal` are
   harness checks in `HarnessMode::UnsafeLateNonce` that call the
   player's items with inputs the player never saw (the fixtures of
   `act9_unsafe_harness.rs`, promoted to real entries); all but
   `extern_c` forbid `no_mangle`. `librarian_demand` cannot be a harness
   check: `page`/`seal` inside the player's macro resolve at the call
   site, which would be the checker's module. The server's `uses` cannot
   name `extern "C" fn` (string contents are blanked), so it uses
   `unsafe extern`; the pattern grader's `extern "C" fn` works (string
   literals compare by contents). The game labels late-nonce passes
   NOT VERIFIED.
5. **Hints.** Folded into each `## Hint` (hint 1 and the gist of 2-3;
   there is no hints-after-failures feature). Every quoted error is the
   edition-2024 wording, asserted.
6. **Formatting only.** Solution comments rewrapped to fit the 72-column
   code pane (`librarian_index`, `librarian_demand`, `librarian_seal`'s
   `let slice =` split over two lines). Behaviour unchanged.
7. **Map.** As proposed, plus: the NPC grid compresses to keep a sprite
   one clear tile inside the cobble ring, and the clearance now comes from
   the largest sprite in the district (`native_px`): the Library's rows
   are 188 px apart (not 240), and the 64 px Librarian stands alone,
   centred, in the back (north) row. The hub, with the 64 px Borrow
   Checker, compressed from 240 to 228 px by the same rule. Cottages
   stand west and east of both new plazas (north is the street, south the
   treeline); the landmark spots are reserved, not built
   (`prop::PENDING_LANDMARKS`).
