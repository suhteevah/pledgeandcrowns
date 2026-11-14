# Mission audit: solutions, graders, tutorial text

Covers all 57 missions in `MissionRegistry::default()` (Acts 1-8) as of
`finish/integration` 83f5daf. Every claim here comes from a command that
was run, listed under **Evidence**.

Player-facing solutions live in
`game/src/plugins/editor/solutions/data.rs`. The "Show solution" button
(it unlocks after 3 failed compiles) now returns `Some` for all 57.

## Evidence

| Check | Command | Result |
|---|---|---|
| Coverage, stub grader, server pattern grader, `fn main` present, differs from starter | `cargo test -p pledge_and_crown --test solutions` | 57/57 |
| Native build, warnings denied | `cargo test -p pledge_and_crown --test solutions -- --ignored every_solution_compiles_natively_without_warnings` (runs `rustc --edition 2021 --crate-type bin -D warnings` on each solution) 57/57 |
| `wasm32-wasip1` build as a binary | `... -- --ignored every_solution_builds_for_wasm32_wasip1` (same `rustc` call plus `--target wasm32-wasip1`) 57/57 |
| Build and run in the real server sandbox | `... -- --ignored solutions_build_and_run_in_the_real_server_sandbox` (`pledge_compile_api::wasm_builder::compile_and_run`: a cargo build for `wasm32-wasip1`, then the wasmtime sandbox) 55/57 built and ran cleanly; 57/57 built. `thread_spawn` and `thread_scope` trap at runtime (expected, asserted) |
| Alternative-solution probes | a throwaway test, not committed, that runs `stub_verdict` and `grader::grade` on the snippets quoted in **Grader findings** | quoted below |
| Tutorial claims | standalone `rustc` runs on the snippets quoted in **Tutorial text** | quoted below |

## Table

Legend: Y = yes, verified by the commands above. "wasip1" means the program
builds as a `wasm32-wasip1` binary. "runs" means it also ran to a clean exit
in the server sandbox.

| id | concept | solution | native | stub grader | wasip1 | notes |
|---|---|---|---|---|---|---|
| intro_let_binding | `let` bindings | Y | Y | Y | Y | |
| double_function | functions, implicit return | Y | Y | Y | Y | G1: `2 * n` rejected |
| borrow_preview | shared borrow `&` | Y | Y | Y | Y | |
| mut_binding | `let mut`, `+=` | Y | Y | Y | Y | `x = x + 1` rejected, though the tutorial calls it "also fine" |
| if_else_sign | `if` as an expression | Y | Y | Y | Y | contract fixture has no `fn main` (E0601, confirmed). G2: wrong labels accepted |
| loop_break | `loop` + `break value` | Y | Y | Y | Y | `break;` with no value accepted |
| match_option | `match` on `Option` | Y | Y | Y | Y | contract fixture has no `fn main` (E0601, confirmed). Wrong arms accepted |
| struct_basic | structs, field access | Y | Y | Y | Y | |
| vec_iter | `Vec`, `.iter().sum()` | Y | Y | Y | Y | |
| tuple_destructure | tuple patterns in `let` | Y | Y | Y | Y | |
| borrow_mut | `&mut`, deref write | Y | Y | Y | Y | G3: parameter must be named `x`. T2 |
| string_vs_str | `String` vs `&str` | Y | Y | Y | Y | G4: `"you".to_string()` rejected |
| option_unwrap_or | `.unwrap_or` | Y | Y | Y | Y | |
| for_in_range | `for` over a range | Y | Y | Y | Y | `0..=9` rejected (fair: the task names `0..10`) |
| closure_basic | closures | Y | Y | Y | Y | G3: parameters must be named `a`/`b` |
| slice_basic | slices `&[T]` | Y | Y | Y | Y | |
| result_question_mark | `Result` + `?` | Y | Y | Y | Y | G5: `Ok(expr?)` rejected |
| derive_debug | `#[derive(Debug)]` | Y | Y | Y | Y | G6: `derive(Debug, Clone)` and `{:#?}` rejected, though the tutorial shows both |
| iter_map_collect | `map` / `collect` | Y | Y | Y | Y | G3: closure parameter must be named `x`. T1 |
| enum_match | enums, `match` | Y | Y | Y | Y | |
| while_loop | `while` | Y | Y | Y | Y | `while n != 0` rejected |
| impl_method | methods, `&self` | Y | Y | Y | Y | |
| assoc_new | associated fn `new` | Y | Y | Y | Y | G7: `-> Hero` rejected; `Self` required |
| if_let | `if let` | Y | Y | Y | Y | |
| while_let | `while let` | Y | Y | Y | Y | |
| tuple_struct | tuple struct / newtype | Y | Y | Y | Y | G8: `Meters(3.0)` passes via the float literal, field never read |
| enum_data_match | enum variants carrying data | Y | Y | Y | Y | |
| trait_def | traits, `impl Trait for` | Y | Y | Y | Y | G9: inherent `impl` plus a `for` loop passes |
| generic_fn | generic fn + bound | Y | Y | Y | Y | |
| generic_struct | generic struct | Y | Y | Y | Y | G10: `struct Pair<T>(T, T)` and `Pair<U>` rejected |
| dyn_trait | `Box<dyn Trait>` | Y | Y | Y | Y | H1: hint names the wrong needles. T3 |
| lifetimes | lifetime parameters | Y | Y | Y | Y | G10: a lifetime named `'s` rejected |
| assoc_type | associated types | Y | Y | Y | Y | |
| result_match | `match` on `Result` | Y | Y | Y | Y | |
| custom_error | custom error enum | Y | Y | Y | Y | |
| from_error | `From` + `?` conversion | Y | Y | Y | Y | |
| option_map | `Option::map` | Y | Y | Y | Y | |
| and_then | `and_then` chaining | Y | Y | Y | Y | **T4 (wrong output in tutorial)** |
| unwrap_or_else | lazy default | Y | Y | Y | Y | contract fixture `\|\| 0` trips clippy `unnecessary_lazy_evaluations`; the player solution uses a side-effecting closure instead |
| hashmap_basic | `HashMap` | Y | Y | Y | Y | |
| iter_filter | `filter` | Y | Y | Y | Y | |
| iter_fold | `fold` | Y | Y | Y | Y | `fold(0, \|acc, _\| acc)` accepted (only needle is `.fold(`) |
| iter_enumerate | `enumerate` | Y | Y | Y | Y | |
| iter_zip | `zip` | Y | Y | Y | Y | G11: `std::iter::zip(&a, &b)` rejected |
| closure_move | `move` closures | Y | Y | Y | Y | |
| thread_spawn | `thread::spawn` / `join` | Y | Y | Y | Y **Confirmed:** builds for wasip1, then traps at runtime in the server sandbox (`wasm trap: unreachable instruction executed`, i.e. `spawn` panics; wasip1 std has no threads). `/compile-real` can never pass this mission. |
| arc_mutex | `Arc<Mutex<T>>` | Y | Y | Y | Y | single-threaded on purpose, so it can run on wasip1 |
| mpsc_channel | channels | Y | Y | Y | Y | single-threaded on purpose |
| atomic | atomics | Y | Y | Y | Y | |
| thread_scope | `thread::scope` | Y | Y | Y | Y **Confirmed:** same trap as `thread_spawn`. `/compile-real` can never pass this mission. |
| async_fn | `async fn` / `.await` | Y | Y | Y | Y | the future is never polled (no executor); the tutorial says so |
| box_basic | `Box`, recursive types | Y | Y | Y | Y | |
| rc_basic | `Rc` | Y | Y | Y | Y | |
| refcell | `RefCell` | Y | Y | Y | Y | |
| cell | `Cell` | Y | Y | Y | Y | `.replace(5)` rejected (fair: the task says `.set`). T5 |
| rc_refcell | `Rc<RefCell<T>>` | Y | Y | Y | Y | |
| weak_ref | `Weak`, `downgrade` / `upgrade` | Y | Y | Y | Y | |

Known gaps (missions with no solution): **none**.

## Grader findings

Both graders are substring matchers, and the server and stub agree byte for
byte. Every finding below was run against **both** graders, with the same
result from each.

- **G0 (all 57 missions): the grader ignores comments and dead code.**
  `fn main() {}` followed by the real solution inside a `/* ... */` block
  passes all **57 of 57** missions. So does a program that does nothing. The
  pattern grader cannot replace `/compile-real` plus an output check (plan
  item D2).
- **G0b: no `fn main` required.** The `contract.rs` fixtures for
  `if_else_sign` and `match_option` pass both graders, yet `rustc` rejects
  them with `error[E0601]: main function not found`. They would fail
  `/compile-real`. `cargo_grader::compile_check` builds an rlib, so it does
  not catch this either.
- **G1 `double_function`:** `fn double(n: i32) -> i32 { 2 * n }` and
  `n + n` are both correct and both REJECTED. The needle is `* 2` / `*2`.
- **G2 `if_else_sign`:** `if n < 0 { "pos" } else { "neg" }` returns wrong
  answers and is ACCEPTED. Similarly, `match_option` accepts
  `Some(_) => 0, None => 0`, and `loop_break` accepts `break;` with no value.
- **G3: parameter names are hard-coded.** `borrow_mut` rejects
  `fn bump(n: &mut i32) { *n += 1; }` because it needs `*x`. `closure_basic`
  rejects `|x, y| x + y` because it needs `+ b`. `iter_map_collect` rejects
  `.map(|n| n * 2)` because it needs `|x|`. All three are correct.
- **G4 `string_vs_str`:** `let s = "you".to_string(); greet(&s);` is
  REJECTED. `String::from` is required.
- **G5 `result_question_mark`:**
  `Ok(s.parse::<i32>().map_err(|e| e.to_string())?)` is REJECTED. The `?`
  must be followed by `;` or a newline.
- **G6 `derive_debug`:** `#[derive(Debug, Clone)]` is REJECTED (the needle
  is the literal `#[derive(Debug)]`). `println!("{item:#?}")` is REJECTED
  (`:#?` does not contain `:?`). The mission's own tutorial shows both
  forms.
- **G7 `assoc_new`:** `fn new(level: i32) -> Hero { Hero { level } }` is
  REJECTED (needs `Self`).
- **G8 `tuple_struct`:** `struct Meters(f64); fn main() { let _d = Meters(3.0); }`
  is ACCEPTED: the float literal `3.0` satisfies the `.0` needle, and the
  field is never read.
- **G9 `trait_def`:** an inherent `impl Fire { fn name... }` plus an unused
  `trait Element` and any `for` loop is ACCEPTED. No trait is implemented.
- **G10: generic and lifetime names are hard-coded.** `generic_struct`
  rejects `struct Pair<T>(T, T);` (no `: T`) and
  `struct Pair<U> { a: U, b: U }`. `lifetimes` rejects
  `fn longest<'s>(x: &'s str, y: &'s str) -> &'s str`.
- **G11 `iter_zip`:** `for (x, y) in std::iter::zip(&a, &b)` (stable since
  1.59) is REJECTED.
- Also REJECTED, but defensible because the task names the construct:
  `mut_binding` `x = x + 1`, `while_loop` `while n != 0`, `for_in_range`
  `0..=9`, `cell` `.replace(5)`, `closure_move` `move||` (no space; rustfmt
  would add one), `unwrap_or_else(Default::default)`.
- **H1 `dyn_trait` hint drift:** the hint says "The grader needs `Box<dyn`
  and `dyn `". The grader actually checks `Box<dyn` and `Box::new`.

## Tutorial text (hard rule 3)

Each item was checked by compiling the snippet.

- **T4 `and_then` (incorrect output, P0 under hard rule 3).** The Syntax
  block reads
  `Some(8).and_then(half).and_then(half).and_then(half);  // None (3 is odd)`.
  The real result is `Some(1)` (8 -> 4 -> 2 -> 1). The program printed
  `Some(2)` / `Some(1)`. 3 never appears. **Correction:** a fourth
  `.and_then(half)` gives `None`, or start from `Some(12)` (12 -> 6 -> 3 ->
  None). The Task line ("Chain it on `Some(8)` ... so an odd intermediate
  short-circuits") needs four calls to show the short-circuit. The player
  solution does that and comments it.
- **T1 `iter_map_collect` (incorrect mechanism).** "The closure receives
  `&i32` here; arithmetic auto-derefs so `x * 2` works without an explicit
  `*x`." Operators do not auto-deref. `x * 2` compiles because std
  implements `Mul<i32> for &i32`. The same closure with `x == 2` fails with
  `error[E0277]: can't compare &i32 with {integer}` (confirmed).
  **Correction:** "std implements arithmetic operators for `&i32`, so
  `x * 2` works; comparisons like `x == 2` still need `*x`."
- **T2 `borrow_mut` (imprecise).** "without [`*x`] you'd be reassigning the
  local `x` binding (the reference itself), not the value it points at."
  In fact `x = v;` does not compile at all:
  `error[E0308]: mismatched types: expected &mut i32, found i32`
  (confirmed). `x` is also not `mut`. **Correction:** "without it the
  compiler rejects the assignment: `x` is a `&mut i32`, not an `i32`."
- **T3 `dyn_trait` (imprecise).** "`Box::new` moves each value to the heap
  and erases its concrete type down to `dyn Element`." `Box::new` produces
  a `Box<Fire>`. The erasure is the unsizing coercion to `Box<dyn Element>`,
  driven by the `Vec<Box<dyn Element>>` annotation. **Correction:**
  "`Box::new` moves each value to the heap; the `Vec<Box<dyn Element>>`
  annotation then coerces each `Box<Fire>` / `Box<Water>` to
  `Box<dyn Element>`."
- **T5 `cell` (imprecise).** "`Cell<T>` is the simpler sibling of
  `RefCell`, for `Copy` types ... you can't get a reference *into* a
  `Cell`." `Cell<T>` works for any `T`; only `.get()` needs `T: Copy`
  (`.set`, `.replace`, `.take` work for any `T`). `Cell::get_mut(&mut self)`
  does return `&mut T`. **Correction:** "most useful for small `Copy`
  types; through a shared `&Cell` you can only move whole values in and
  out."

Nothing else in the 57 tutorials was found to be technically wrong. The
Borrow Checker NPC's lines are in `npc.rs` and were not part of this pass.
