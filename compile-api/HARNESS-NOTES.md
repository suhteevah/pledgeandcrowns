# Harness grading: what it guarantees, and Act 9

*Third security review (2026-11-15), findings C-1 and C-2. Design note
only: Act 9 grading is NOT implemented.*

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
