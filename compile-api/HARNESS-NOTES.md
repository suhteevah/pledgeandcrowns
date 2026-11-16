# Harness grading: what it guarantees, and Act 9

*Third security review (2026-11-15), findings C-1 and C-2. Act 9 grading:
design 1 below is BUILT as the opt-in harness mode
`HarnessMode::UnsafeLateNonce` (2026-11-15). Four registered Act 9
missions use it (`extern_c`, `send_sync`, `librarian_index`,
`librarian_seal`, 2026-11-15); the other five Act 9 missions are checked
by their stdout. It raises the bar for forged results; it does not
guarantee them (see "Design 1 as built"), so the game labels these passes
NOT VERIFIED (they still count as clears).*

## How a harness mission is graded today

`src/expectations.rs` builds the player's file as module `main` of a
server-owned crate (`src/pnc_entry.rs`) with a server checker as a child
module. The checker calls the player's items with fixed inputs and prints
`<nonce> label = value` lines; only lines carrying this request's random
nonce count. Player and checker are compiled into ONE wasm module and run
in ONE instance: same linear memory, same stdout.

## What it can and cannot guarantee

Holds:

- A player cannot pass by printing the expected lines: without the nonce
  they are just program output.
- Since C-1, harness builds have `#![forbid(unsafe_code)]` in the server
  entry. It covers every module (the player's included) and cannot be
  relaxed by `#![allow]` in the player's file, so player code cannot read
  raw memory, call `extern` functions, use `#[no_mangle]`/`#[export_name]`
  or implement `unsafe` traits. It lives in the server file, so the
  player's `src/main.rs` keeps its one-line preamble
  (`SERVER_PREAMBLE_LINES = 1`). The refusal is shown as a plain
  sentence plus rustc's own error at the player's line.

Does not hold:

- **The nonce is a constant in the program's own memory.** Any code in the
  module that can read arbitrary memory can find it (`PNC` + 32 hex) and
  forge results. `forbid(unsafe_code)` removes the easy way; safe Rust is
  not a hard boundary: rustc and std have known soundness holes reachable
  from safe code (e.g. the long-standing implied-bounds lifetime hole that
  allows a safe `transmute`), so a determined player could still reach
  raw memory. The wasm sandbox protects the server, not the checker.
- The checker shares process state with the player: a player can change
  globals, install a panic hook, or exit before the checker finishes (the
  last is detected: no `<done>` mark means "exited early").
- **C-2 (accepted).** Failure messages reveal the fixed inputs and the
  expected values, unpinned return types accept a player-written `Debug`,
  and `MissionCheck::uses` is a substring check that dead code satisfies,
  so answers can be hardcoded. This is a known limit of same-process
  grading in a learning game, where a cheat only fools the cheater; the
  cure would be hidden per-request inputs and type-pinned signatures.

## Act 9: lessons that require `unsafe`

`forbid(unsafe_code)` and a lesson about `unsafe` cannot coexist, and with
`unsafe` allowed the same-instance nonce is readable by construction. Three
honest designs, cheapest first:

### 1. Nonce delivered at the END of the run, never while player code runs

The nonce is not compiled in. After the checker has made its LAST call
into player code, formatted every result to text and dropped every player
value, it asks the host for the nonce (an imported host function
`pnc_nonce(ptr, len)` added to the linker for harness runs only, or the
first line of stdin, which today is closed) and prints the
pre-formatted lines with it.

- For: small change (one host import or stdin, checker reorder); the nonce
  is not in memory at any moment player code executes, so scanning finds
  nothing.
- Against: player `unsafe` code that ran earlier can still corrupt
  checker state in the shared memory (the stored result strings, the
  allocator, stdout's buffer) so that the late, honest-looking print emits
  forged values. It raises the bar from "grep memory" to "write an exploit
  against the checker's heap"; it is not a guarantee. Player `Drop`
  impls, panic hooks and destructors must be provably finished first.

### 2. Host-side checker calling the player's exports

The player's code is built as a module that exports the mission's
functions through server-generated `extern "C"` wrappers; the checker is
Rust in the server process (the job/run child), which instantiates the
module and calls those exports directly with fixed inputs, reading results
from return values and copied-out memory.

- For: the checker and its expectations never enter the guest at all;
  nothing to scan, nothing to forge. A lying export is just a wrong
  answer, graded as such.
- Against: only simple types cross cleanly (integers, floats, byte
  strings via ptr/len); `String`, `Vec`, structs and traits need a
  hand-written encoding per mission (the generated wrappers run in the
  guest, but the host validates what they return). The player can still
  keep state between calls or special-case the inputs (C-2 class). The
  host-side checker becomes new trusted code in the run sandbox.

### 3. Checker in a separate instance, linked through the host

Two modules: the player's (as in design 2) and a server checker module
with its own linear memory, whose imports are satisfied by the player
instance's exports via the wasmtime `Linker`. Each instance has its own
memory (multi-memory stays disabled); the checker's nonce lives only in
the checker's memory, out of reach of the player's code, and the checker
prints the tagged lines itself.

- For: keeps the checker in Rust and in wasm (same sandbox, same limits)
  while giving it a memory the player cannot address; the existing
  nonce/transcript format and host comparison stay as they are.
- Against: cross-instance calls pass only scalars, so non-scalar values
  must be copied between memories by host functions (or the component
  model, which this server compiles in only for WASI p1 and does not
  expose to guests: enabling it for guests widens the attack surface the
  advisories in `.claude/reviews/linux-verification-2026-11-15.md`
  were about). Two instances per run (`MAX_INSTANCES` would go to 2, a
  reviewed change) and a little more compile time.

Recommendation when Act 9 is built: design 2 for missions whose
signatures are scalar or byte-oriented, design 3 where the lesson needs
richer types; design 1 only as a stopgap, labelled as such.

**Decision (2026-11-15, orchestrator):** design 1, as an explicit opt-in
mode, for v1.0. A forged grade only fools the cheater (C-2 is accepted
on the same grounds), the wasm and bwrap sandboxes protect the server
whatever the grade, and designs 2/3 cost more than v1.0 can afford.

## Design 1 as built: `HarnessMode::UnsafeLateNonce`

Code: `src/expectations.rs` (`HarnessMode`, `harness_module_source_late`,
`HARNESS_ENTRY_SOURCE_UNSAFE_ALLOWED`, `UNSAFE_HARNESS_FORBIDS`,
`parse_transcript_tagged`), `src/wasm_runner.rs` (`LateNonce`, the host
function), `src/run_sandbox.rs` (`RunHeader::late_nonce`). Proof:
`tests/act9_unsafe_harness.rs`.

### The switch

`MissionCheck::harness_mode` is a field of the server's own table, chosen
by mission id. Exactly four registered missions are `UnsafeLateNonce`
(`extern_c`, `send_sync`, `librarian_index`, `librarian_seal`); every
other is `ForbidUnsafe`, and the unit test
`only_the_act9_allow_list_uses_unsafe_late_nonce` (an explicit allow-list,
review finding P2-3) fails if any other mission gains the mode. A player
file that writes `#![allow(unsafe_code)]` on a `ForbidUnsafe` harness
mission is still a compile error with the plain "does not allow `unsafe`"
message (slow test case `double_function: player file tries to opt into
unsafe`). Only a `MissionCheck` with `harness_mode: UnsafeLateNonce` gets
the entry file without `#![forbid(unsafe_code)]` and the late checker.

Act 9 missions whose lesson lives in `main` (`raw_pointer`,
`macro_rules`, `drop_trait`, `maybe_uninit`, `librarian_demand`) are
`Stdout` checks plus construct tokens. Their build is the plain layout,
which has never carried the `forbid` and has no nonce, so their `unsafe`
needs no mode and gains nothing from one.

### Delivery channel: a host import, not stdin

stdin was rejected: it is readable from **safe** code (`std::io::stdin()`)
at any moment, so a player could read the nonce before the checker does.
A host import cannot be reached from the player's file:

- it is imported from module `pnc_harness`; Rust imports from any module
  but `env` only with `#[link(wasm_import_module = ...)]`, and the source
  deny-list refuses `link` inside any attribute (`source_policy.rs`);
- its name is `nonce_` + 32 random hex digits, fresh per request, compiled
  into the checker only (the player cannot read the checker's file:
  `include*!` is refused), so even a deny-list bypass would have to guess
  128 bits;
- it is one-shot: the first call consumes the nonce whatever its
  arguments; later calls get `-1` and nothing. A checker that does not get
  the nonce prints no tagged line, so the run grades as "exited before the
  checker finished".

It writes exactly 35 bytes through `wasmtime::Memory::write` (bounds
checked; out of range gives `-3`, including an all-ones `0xFFFFFFFF`
target and a module with no `memory` export, both tested since review
finding P2-2; wrong length `-2`), never blocks, and is
added to the `Linker` **only** for a run given a `LateNonce`. Every other
run links WASIp1 alone, so a module importing it fails to instantiate.
The engine `Config`, feature allow-list, fuel, epoch deadline, async
timeout, backstop, store limits (`MAX_INSTANCES` stays 1), WASI context
and the run sandbox's bwrap spec are unchanged. The nonce reaches the
`run-wasm` child in its stdin header (`late_nonce`, omitted for every
other run so their bytes are unchanged), is validated there (shape, no
unknown fields), and is never logged: `LateNonce`'s `Debug` redacts it,
and the fast test `run_child_links_the_import_only_when_the_header_carries_a_nonce`
runs the real child at `RUST_LOG=debug` and checks its log has no `PNC`.

### When the nonce enters guest memory

The checker (`harness_module_source_late`) contains no nonce. `run()` is
`calls(); finish();`:

1. `calls()` runs the player's `main` (if checked) and the mission body.
   `emit` formats each result to an owned `String`, drops the player's
   value at once, and stores the text in a checker-private
   `Mutex<Vec<String>>`. Every other player value is a local of `calls`
   and is dropped when it returns. The `<main>` marks are printed during
   this phase with a separate, non-secret per-request mark (`PNM...`):
   forging one only moves the boundary of the player's own `main` output.
2. `finish()` resets the panic hook (dropping any the player installed),
   takes the stored lines, and only THEN calls the import. That call is
   the first moment the nonce is in guest memory. At that moment the
   checker holds: the `Vec<String>` of formatted `label = value` lines and
   a 35-byte stack buffer. It builds one block (every result line, then
   `<done>`) and prints it with one `print!`, then flushes. `finish` makes
   no call into player code.

The host side (`parse_transcript_tagged`) counts results and `<done>` only
with the nonce, and flags any nonce-tagged line after `<done>` (or a
second `<done>`): such a run never passes.

Failure modes are those of `ForbidUnsafe` (slow test, `librarian_index`
fixture): early exit is "exited before the checker finished" (wrong
output), a panic, a spin, the memory cap and the output cap are runtime
errors.

### Static rules in this mode

`UNSAFE_HARNESS_FORBIDS` (`global_allocator`, `export_name`) is applied
to every `UnsafeLateNonce` pass, on top of the mission's own `forbids`.
Each names a way to run player code DURING the late print. `no_mangle` is
the same kind of hook but is the `extern_c` lesson, so missions forbid it
individually where they can (the fixtures do, except `extern_c`).

`uses` / `forbids` / pinned signatures work unchanged. What is checkable:
tokens in code (`unsafe {`, `unsafe fn`, `unsafe impl Send`, `from_raw`,
`MaybeUninit`, `repr(C)`, `no_mangle`). What is not: anything in a
comment or string, because the matcher blanks both first; so "has a
`// SAFETY:` comment" cannot be required (and a comment check would be
satisfied by dead text anyway). `extern "C" fn` is not a usable token
either: the `"C"` is blanked to `""`; use `unsafe extern` / `no_mangle`.

### What the proof shows (`tests/act9_unsafe_harness.rs`)

Since the missions were registered, the fixtures grade the REGISTERED
checks (looked up by id), so the proof covers what the server runs.
Five Act 9 missions, four of them `UnsafeLateNonce`: `librarian_index`
(`unsafe fn` call + raw-pointer deref), `extern_c` (FFI: `repr(C)`,
`no_mangle`, `unsafe extern`), `librarian_seal` (`Box::into_raw`/
`from_raw`, raw deref, `Drop`), `send_sync` (`unsafe impl`), `maybe_uninit`
(`MaybeUninit`, pointer `write`, the `transmute` alternative). The spec
has no `static mut` or union lesson (it cuts both for scope);
`send_sync`'s `unsafe impl` stands in as the other unsafe superpower.
For each, the spec's solution passes and three to five wrong variants do
not. For the four whose checker calls player items:

| Attack (on the hooked item's last call) | Result |
|---|---|
| scan all linear memory for `PNC` + 32 hex, forge and exit if found | finds only its own decoy (`hits=1 decoy_seen=true other_key=false`); graded on its real (wrong) answers |
| print expected lines tagged with a guessed key, exit | "exited before the checker finished" |
| `process::exit(0)` | same |
| `panic!` | runtime error |
| thread-local destructor that scans and prints after `<done>` | did not pass; on wasip1 the destructor never ran (no output), so the after-`<done>` rule is proven by the unit test only |

`maybe_uninit` checks only `main`'s output (the spec's solution has no
items to call), so it has nothing for the checker to key and the key
attacks do not apply; it is the same trust level as a `Stdout` check
plus its `uses` tokens, which is what it is now registered as.

### What it cannot stop (the known limit, tested)

Player `unsafe` code shares the checker's memory, and design 1 only keeps
the NONCE out of it. Two attacks, kept as documented `#[ignore]`d tests
that assert what actually happens:

- **Rewrite a stored result: SUCCEEDS.**
  `known_limit_heap_patch_forges_a_pass`: `fetch` is wrong for a
  one-volume shelf (`Some(9)` for `Some(7)`); on its last call it scans
  the heap for the checker's stored line `fetch([7], 0) = Some(9)` and
  rewrites the `9`. The checker then prints the forged line, honestly
  tagged, and the run is graded **Passed**. The nonce is never seen. Any
  result formatted before player code's last call can be rewritten this
  way, and the same holds for the allocator's state and std's stdout
  buffer. Not fixed, and not to be fixed by obscurity: it is the price of
  same-memory grading. The same player could simply hardcode `Some(7)`
  (C-2), so it changes nothing for an honest learner.
- **Run code during the late print: SUCCEEDS at run time, CAUGHT
  statically.** `known_limit_global_allocator_forges_but_is_caught_statically`:
  a `#[global_allocator]` armed on the last call scans memory on every
  allocation; the checker's late `String` growth hands it the nonce; it
  writes a forged transcript with libc `write` and `_exit(0)`. The
  behaviour check passes (forged), and `UNSAFE_HARNESS_FORBIDS` turns it
  into `StaticCheckFailed`. Other hooks of the same kind are NOT covered
  statically and are assumed to work: `#[no_mangle]`/`#[export_name]`
  interposition of a symbol std calls during the print (only
  `export_name` is forbidden mode-wide; `no_mangle` is allowed for
  `extern_c`), overwriting a vtable or function-pointer slot in the data
  segment so that `print!` calls player code, or corrupting the stdout
  buffer so the flushed bytes differ.

So, in this mode: a player cannot pass by scanning for or guessing the
nonce, by exiting early, or by printing after the checker; a player who
writes an exploit against the checker's own heap can. That is "raising
the bar", not a guarantee. If Act 9 ever needs real assurance, the
upgrade path is design 2 or 3 above; a cheap intermediate step would be a
second one-shot import that takes each result line the moment it is
formatted, so no stored result survives a later call into player code
(it would close the rewrite attack, not the late-print hooks).

## Act 10: a player `#[global_allocator]` in the plain build (2026-11-15)

`prealloc` and `compiler_alloc` teach measuring allocations with a
counting `GlobalAlloc`, so the player's program declares
`unsafe impl GlobalAlloc` and `#[global_allocator]`. Both missions are
`Check::Stdout` entries (`mu` in `expectations/data.rs`): the plain
layout, no harness, no checker, no nonce; their `harness_mode` is the
default `ForbidUnsafe`, which a stdout check never reads. The plain build
has never carried `#![forbid(unsafe_code)]`, and `UNSAFE_HARNESS_FORBIDS`
applies only to `UnsafeLateNonce` harness checks, so neither rule changes.
An allocator here can only change the program's own stdout, which the
player controls anyway; there is no server secret in that process to read
or forge. The late-nonce allow-list is unchanged (the four Act 9
missions). The sandbox's run limits apply to the allocator's code like any
other player code.
