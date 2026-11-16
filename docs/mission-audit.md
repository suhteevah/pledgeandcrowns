# Mission audit: solutions, graders, tutorial text

Covers all 57 missions in `MissionRegistry::default()` (Acts 1-8) as of
`finish/integration` 83f5daf. Every claim here comes from a command that
was run, listed under **Evidence**. Since then the Borrow Checker's trial
added three missions (60 missions, 57 NPCs), the Act 3-5 boss trials
nine more (69 missions, 57 NPCs), the Act 6-8 boss trials nine more
(78 missions, 57 NPCs), and Act 9 nine more (87 missions, 64 NPCs); see
**The Borrow Checker's trial**, **The Act 3-5 trials**, **The Act 6-8
trials** and **Act 9: The Forbidden Library** at the end.

Player-facing solutions live in
`game/src/plugins/editor/solutions/data.rs`. The "Show solution" button
(it unlocks after 3 failed compiles) now returns `Some` for all 57.

## Evidence

| Check | Command | Result |
|---|---|---|
| Coverage, stub grader, server pattern grader, `fn main` present, differs from starter | `cargo test -p pledge_and_crown --test solutions` | 57/57 |
| Native build, warnings denied | `cargo test -p pledge_and_crown --test solutions -- --ignored every_solution_compiles_natively_without_warnings` (runs `rustc --edition 2024 --crate-type bin -D warnings` on each solution; the edition is `PLAYER_EDITION`, see **Edition**) 57/57 |
| `wasm32-wasip1` build as a binary | `... -- --ignored every_solution_builds_for_wasm32_wasip1` (same `rustc` call plus `--target wasm32-wasip1`) 57/57 |
| Build and run in the real server sandbox | `... -- --ignored solutions_build_and_run_in_the_real_server_sandbox` (`pledge_compile_api::wasm_builder::compile_and_run`: a cargo build for `wasm32-wasip1`, then the wasmtime sandbox) 55/57 built and ran cleanly; 57/57 built. `thread_spawn` and `thread_scope` trap at runtime (expected, asserted) |
| Alternative-solution probes | a throwaway test, not committed, that runs `stub_verdict` and `grader::grade` on the snippets quoted in **Grader findings** | quoted below |
| Tutorial claims | standalone `rustc` runs on the snippets quoted in **Tutorial text** | quoted below |

## Table

Legend: Y = yes, verified by the commands above. "wasip1" means the program
builds as a `wasm32-wasip1` binary. "runs" means it also ran to a clean exit
in the server sandbox. Grader notes (G*, wrong arms, `break;`, `fold`)
describe the substring graders at 83f5daf; all of them, including G0b
and H1, are resolved (what remains open is semantics a pattern grader
cannot see), see **Grader findings** for the status of each. "native"
builds use edition 2024, the edition the server builds player code with
(see **Edition**).

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
| borrow_mut | `&mut`, deref write | Y | Y | Y | Y | G3: parameter must be named `x`. T2 (resolved) |
| string_vs_str | `String` vs `&str` | Y | Y | Y | Y | G4: `"you".to_string()` rejected |
| option_unwrap_or | `.unwrap_or` | Y | Y | Y | Y | |
| for_in_range | `for` over a range | Y | Y | Y | Y | `0..=9` rejected (fair: the task names `0..10`) |
| closure_basic | closures | Y | Y | Y | Y | G3: parameters must be named `a`/`b` |
| slice_basic | slices `&[T]` | Y | Y | Y | Y | |
| result_question_mark | `Result` + `?` | Y | Y | Y | Y | G5: `Ok(expr?)` rejected |
| derive_debug | `#[derive(Debug)]` | Y | Y | Y | Y | G6: `derive(Debug, Clone)` and `{:#?}` rejected, though the tutorial shows both |
| iter_map_collect | `map` / `collect` | Y | Y | Y | Y | G3: closure parameter must be named `x`. T1 (resolved) |
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
| dyn_trait | `Box<dyn Trait>` | Y | Y | Y | Y | H1, T3 (both resolved) |
| lifetimes | lifetime parameters | Y | Y | Y | Y | G10: a lifetime named `'s` rejected |
| assoc_type | associated types | Y | Y | Y | Y | |
| result_match | `match` on `Result` | Y | Y | Y | Y | |
| custom_error | custom error enum | Y | Y | Y | Y | |
| from_error | `From` + `?` conversion | Y | Y | Y | Y | |
| option_map | `Option::map` | Y | Y | Y | Y | |
| and_then | `and_then` chaining | Y | Y | Y | Y | T4 (resolved) |
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
| cell | `Cell` | Y | Y | Y | Y | `.replace(5)` rejected (fair: the task says `.set`). T5 (resolved) |
| rc_refcell | `Rc<RefCell<T>>` | Y | Y | Y | Y | |
| weak_ref | `Weak`, `downgrade` / `upgrade` | Y | Y | Y | Y | |

Known gaps (missions with no solution): **none**.

## Grader findings

**Status (grader pass, 2026-11-14).** Both graders now lex the source
(comments, strings, raw strings, byte strings and char literals are not
code; lifetimes are told apart from char literals) and match whitespace-
insensitive token patterns with wildcards (`$_`, `$X`, `$'a`, `$..`). The
engine and the table are test-enforced textually identical in both crates
(`server_and_stub_rule_tables_are_textually_identical`). Each finding below
carries a **Resolved** / **Open** line. Evidence: `grader_corpus` in
`game/tests/contract.rs` (counts under **Grader corpus** below), plus
`corpus_correct_programs_compile` (`--ignored`), which builds every
"correct" corpus program with `rustc`.

The original findings follow. They describe the substring graders as they
were at 83f5daf; both graders agreed byte for byte, and every finding was
run against **both**, with the same result from each.

- **G0 (all 57 missions): the grader ignores comments and dead code.**
  `fn main() {}` followed by the real solution inside a `/* ... */` block
  passes all **57 of 57** missions. So does a program that does nothing. The
  pattern grader cannot replace `/compile-real` plus an output check (plan
  item D2).
  **Resolved** for comments and strings: code inside `//`, `/* */` (nested)
  or any string literal no longer counts, and the player solution hidden
  that way fails all 57 missions (corpus (c)). **Open, not closable by a
  pattern grader:** dead code that is real code (an uncalled `fn`) still
  counts, and a program that has the right shape but prints the wrong
  thing still passes. Only `/compile-real` with an output check closes
  that.
- **G0b: no `fn main` required.** The `contract.rs` fixtures for
  `if_else_sign` and `match_option` pass both graders, yet `rustc` rejects
  them with `error[E0601]: main function not found`. They would fail
  `/compile-real`. `cargo_grader::compile_check` builds an rlib, so it does
  not catch this either.
  **Resolved.** One shared rule in the grader engine (not 57 table
  entries), `has_entry_point`, runs after a mission's own checks: the
  code must contain a top-level `fn main(` item (`pub`, `pub(..)` and
  `const` allowed). A `main` inside a `mod`, `impl`, trait or another fn
  does not count, nor does `async` / `unsafe` / `extern "C"` main or
  `fn main<T>` (rustc rejects all four: E0752, E0580, E0580, E0131), nor
  one in a comment or string. Failing it gives
  "`{npc}` — every Rust program starts at `fn main()`: add one at the top
  level, outside any `impl` or `mod`". It is checked last so a missing
  concept is always reported first, and no starter's first message is the
  `fn main` hint (`grader_corpus` asserts it). Known limit: a `main`
  re-exported with `use m::main;` is valid Rust but is rejected. Both
  fixtures now have a `fn main`, and `corpus_correct_programs_compile`
  also builds all 57 fixtures as binaries, which is what would have
  caught G0b.
- **G1 `double_function`:** `fn double(n: i32) -> i32 { 2 * n }` and
  `n + n` are both correct and both REJECTED. The needle is `* 2` / `*2`.
  **Resolved:** the check accepts `* 2`, `2 *` or `$X + $X`.
- **G2 `if_else_sign`:** `if n < 0 { "pos" } else { "neg" }` returns wrong
  answers and is ACCEPTED. Similarly, `match_option` accepts
  `Some(_) => 0, None => 0`, and `loop_break` accepts `break;` with no value.
  **Resolved** for all three examples. `if_else_sign` now needs the `< 0`
  branch to give `"neg"`, and `"zero"` / `"pos"` each to sit on a branch
  whose guard fits (`== 0`, `> 0`, or the final `else` after the other two),
  so neg/pos and zero/pos swaps fail. `match_option` needs
  `Some($X) => $X` and `None => 0` (bare, block or `return` arms).
  `loop_break` needs `break` followed by a value. **Open, not closable:**
  `if n < 0 { "neg" } else if n > 5 { "pos" } else { "zero" }` (wrong for
  1..=5) still passes; whether a guard is the *right* comparison is
  semantics.
- **G3: parameter names are hard-coded.** `borrow_mut` rejects
  `fn bump(n: &mut i32) { *n += 1; }` because it needs `*x`. `closure_basic`
  rejects `|x, y| x + y` because it needs `+ b`. `iter_map_collect` rejects
  `.map(|n| n * 2)` because it needs `|x|`. All three are correct.
  **Resolved:** `borrow_mut` captures the parameter name
  (`fn bump($X: &mut i32) $.. *$X`), `closure_basic` captures both closure
  parameters and accepts either order of the sum, `iter_map_collect`
  accepts any single binder (`|$_|`, `|&$_|`, `|$_: T|`).
- **G4 `string_vs_str`:** `let s = "you".to_string(); greet(&s);` is
  REJECTED. `String::from` is required.
  **Resolved:** `.to_string()`, `.to_owned()`, `String::new()` and
  `format!` are accepted too.
- **G5 `result_question_mark`:**
  `Ok(s.parse::<i32>().map_err(|e| e.to_string())?)` is REJECTED. The `?`
  must be followed by `;` or a newline.
  **Resolved:** any `?` token in code counts (a `?` in a format string is
  inside a string literal and does not).
- **G6 `derive_debug`:** `#[derive(Debug, Clone)]` is REJECTED (the needle
  is the literal `#[derive(Debug)]`). `println!("{item:#?}")` is REJECTED
  (`:#?` does not contain `:?`). The mission's own tutorial shows both
  forms.
  **Resolved:** `#[derive( $.. Debug` accepts `Debug` anywhere in the
  derive list; the formatter check looks inside string literals (the only
  check that does) for `:?` or `:#?`.
- **G7 `assoc_new`:** `fn new(level: i32) -> Hero { Hero { level } }` is
  REJECTED (needs `Self`).
  **Resolved:** `Self` or `-> Hero`.
- **G8 `tuple_struct`:** `struct Meters(f64); fn main() { let _d = Meters(3.0); }`
  is ACCEPTED: the float literal `3.0` satisfies the `.0` needle, and the
  field is never read.
  **Resolved:** `3.0` lexes as one number token, so only a real field
  access `.0` matches. **Open, not closable:** `.0` on some other tuple
  next to an unused `Meters` still passes (needs data flow).
- **G9 `trait_def`:** an inherent `impl Fire { fn name... }` plus an unused
  `trait Element` and any `for` loop is ACCEPTED. No trait is implemented.
  **Resolved:** the check captures the trait's name and needs
  `impl <that name> for` (before or after the trait definition, generic
  impls allowed), so an inherent impl, a `for` loop, or implementing a
  different (std) trait all fail.
- **G10: generic and lifetime names are hard-coded.** `generic_struct`
  rejects `struct Pair<T>(T, T);` (no `: T`) and
  `struct Pair<U> { a: U, b: U }`. `lifetimes` rejects
  `fn longest<'s>(x: &'s str, y: &'s str) -> &'s str`.
  **Resolved:** `generic_struct` captures the type parameter and needs a
  field of that type (named or tuple struct, bounds allowed); `lifetimes`
  accepts any lifetime name used consistently (`<$'a $.. &$'a`);
  `generic_fn` accepts any type-parameter name.
- **G11 `iter_zip`:** `for (x, y) in std::iter::zip(&a, &b)` (stable since
  1.59) is REJECTED.
  **Resolved:** any call to `zip(`.
- Also REJECTED, but defensible because the task names the construct:
  `mut_binding` `x = x + 1`, `while_loop` `while n != 0`, `for_in_range`
  `0..=9`, `cell` `.replace(5)`, `closure_move` `move||` (no space; rustfmt
  would add one), `unwrap_or_else(Default::default)`.
  **Unchanged on purpose**, except `move||`, which now passes because
  whitespace no longer matters.
- **`iter_fold`:** `fold(0, |acc, _| acc)` accepted (only needle is
  `.fold(`). **Resolved:** the fold closure's element parameter must be
  used in its body (`.fold( $.. |$_ $.., $X $..| $.. $X`, `&x` allowed).
  **Open, not closable:** `|acc, x| x` (keeps the last element) or
  `|acc, x| acc * x` still pass; only an output check tells a sum from
  another reduction.
  **Named functions accepted (follow-up).** The tightened check had
  rejected `fold(0, add)`, which is valid, idiomatic Rust. It now also
  accepts a named fold function when the grader can see it use its
  element parameter: a local `fn add(acc: i32, x: i32) -> i32 { .. x .. }`
  defined before the fold, or after it when the fold call is not nested
  inside another bracket group, and a closure bound to a name first
  (`let add = |acc, x| acc + x; .fold(0, add)`). A named fn that ignores
  its element (`fn keep(acc: i32, _x: &i32) -> i32 { acc }`) still fails.
  **Limits:** a path function (`fold(0, i32::wrapping_add)`), a fn with a
  destructured `&x` parameter or a non-identifier return type, and a fn
  defined after a fold nested in e.g. `println!(..)` are rejected.
- **H1 `dyn_trait` hint drift.** The tutorial's Hint section in
  `mission.rs` said "The grader needs `Box<dyn` and `dyn `"; the grader
  checks `Box<dyn` and `Box::new`.
  **Resolved:** the hint in `mission.rs` now names `Box<dyn` and
  `Box::new`, matching the grader table.
- **Token matching side effects** (new behaviour, all intended):
  identifiers match whole, so `RefCell::new` no longer satisfies `cell`'s
  `Cell::new`, and `atomic` lists the std `Atomic*` types instead of
  matching the substring `Atomic`.

### Grader corpus

`grader_corpus` (both graders, which must agree on every input):

| set | result |
|---|---|
| (a) player solutions (`editor/solutions/data.rs`) pass | 57/57 |
| (b) `contract.rs` canonical fixtures pass | 57/57 |
| (c) player solution hidden in a block comment, in line comments AND in a raw string, next to an empty `fn main`: all three fail | 57/57 |
| (d) correct programs pass: 15 audit "correct but rejected" examples + 23 differently-written solutions for the tightened missions + 3 named-fn / named-closure folds + 1 `pub fn main` | 42/42 |
| (e) wrong programs fail: those the old grader accepted (G2 x4, G8, G9 x2, `iter_fold` x2) + G0b x4 (no `main`, `main` only in a `mod`, only as a method, `async fn main`) | 13/13 |
| (f) unmodified starter code fails, and never with the `fn main` hint first | 57/57 |

Every (d) program and every canonical fixture builds as a binary with
`rustc --edition 2024` (`corpus_correct_programs_compile`, `--ignored`).
Lexer: 16 lexer tests and 5 matcher tests in `game/tests/stub_grader.rs`,
plus 3 entry-point tests; engine smoke tests in the server's `grader.rs`.

### Snapshot diff

**`fn main` pass (G0b, 2026-11-14).** Before regenerating, all 293
inputs of the previous snapshots were replayed through the new graders
(stub and server, identical results): **2 verdict changes, both
pass->fail, 0 fail->pass, 0 message-only changes, 291 byte-identical.**
The 2 are the `if_else_sign` and `match_option` canonical fixtures,
which had no `fn main`; both now fail with the `fn main` hint, so every
change is explained by the new rule. (The `iter_fold` named-fn change
was applied afterwards and moved no snapshot verdict.) The regenerated
snapshots hold 349 cases: the 282 inputs that still exist are all
unchanged; the 11 old cases of the two fixed fixtures were replaced by 12
cases of the new fixture text (one of them its `fn main` near-miss); and
the other 55 missions gained one near-miss each (removing the `main()`
token now yields the new message). 282 + 12 + 55 = 349.

Previous pass (comment-aware lexer): 285 replayed, 0 verdict changes, 5
starter messages changed because their `// ...` comments no longer
satisfied early checks.

## Edition

The server builds player code as **Rust 2024** (the frozen manifests in
`compile-api/src/wasm_builder.rs` and `cargo_grader.rs`). The audit
suites used to call `rustc --edition 2021`, so a program could pass the
tests and fail on the server. They now all use
`game/tests/common/mod.rs::PLAYER_EDITION`; the server exports no
constant, so `solutions.rs::player_edition_matches_the_server_manifests`
greps both manifests and fails if either disagrees.

Re-run under 2024: 57/57 solutions build natively with `-D warnings`
and for `wasm32-wasip1`; all 42 corpus programs and 57 canonical
fixtures build; every starter compiles except `lifetimes` (still exactly
E0106). A throwaway differential check also built all 57 tutorial code
blocks under both editions (as written, wrapped as items and wrapped in
`fn main`): every block's errors and warnings were identical in 2021 and
2024. **No 2021-vs-2024 difference was found anywhere in Acts 1-8**, so no
solution, fixture or tutorial text changed for the edition. The risk is
in future content: 2024 requires `unsafe extern "C" { .. }` blocks and
`#[unsafe(no_mangle)]` / `#[unsafe(export_name)]`, rejects references to
`static mut` (deny-by-default `static_mut_refs`), reserves `gen`, and
changes `impl Trait` capture and `if let` / tail-expression temporary
scopes. Act 9 (FFI) text and solutions must be written for 2024.

## Tutorial text (hard rule 3)

Each item was checked by compiling the snippet.

**Status: T1-T8 are all RESOLVED in `mission.rs`** (second pass, rustc
1.99.0, edition 2021; the code blocks were re-checked under 2024 with
identical results, see **Edition**). That pass compiled or ran 104 snippets covering
every tutorial code block and every claimed output, error or rule; only
the T4 snippet disagreed with its text. Every starter compiles except
`lifetimes`, which fails with exactly `E0106` as intended; the ignored
test `every_starter_compiles_or_fails_for_its_intended_reason`
(`game/tests/mission_text.rs`) keeps it that way. The fixes:
T4 now chains four `half` calls (`Some(1)`, then `None (1 is odd)`), and
its Task says four. T1, T2, T3 and T5 use the corrections quoted below,
with T2 naming E0308.

- **T6 `struct_basic` (invalid syntax shown as code).** "Construction uses
  the `TypeName { field: value, .. }` literal". A bare `..` is not legal in
  a struct expression: `error[E0797]: base expression required after ..`
  (confirmed). Now reads `TypeName { field: value }`, one `field: value`
  per field.
- **T7 `generic_struct` (same).** "annotate `Pair::<i32> { .. }`" gives
  the same E0797. Now reads `Pair::<i32> { a: 1, b: 2 }` (compiles).
- **T8 `assoc_new` (conflated two shorthands).** "`Self { level }` is
  shorthand for `Hero { level }`" mixed `Self` with field-init shorthand.
  Now reads "shorthand for `Hero { level: level }`: `Self` names the type,
  and the field name can stand alone when the local shares it."

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

Nothing else in the 57 tutorials, goals, hints or starters was found to be
technically wrong. Every "The grader needs ..." hint was compared with the
`stub_grader.rs` rule table as of `ffd62c1`; only H1 differed. The Borrow
Checker NPC's lines are in `npc.rs` and were not part of either pass.

## Behaviour checks on `/compile-real` (pinned outputs, 2026-11-14)

The compile server grades each mission with one check from
`compile-api/src/expectations/data.rs`. Before this pass 41 of 57 were
behaviour-checked; 13 were `RunOnly` ("PASS - NOT VERIFIED") because the
Task let the player choose the values, and 3 are compile-only. The 13
Tasks now name the values and show every expected line in a code span,
so one behaviour is specified.

**Totals now: 83 of 87 behaviour-checked (34 output checks, 49 harness
checks), 4 static-only, 0 unverified** (the Borrow Checker's trial,
2026-11-14, added three harness-checked stages, the Act 3-5 trials,
2026-11-15, nine more, the Act 6-8 trials, 2026-11-15, eight harness
checks and one static-only stage, and Act 9, 2026-11-15, five output
checks and four harness checks, those four in `UnsafeLateNonce` mode;
see the sections below). Before the
trials: 54 of 57 (25 harness checks). The 4 static-only missions cannot
run on `wasm32-wasip1`: `thread_spawn`, `thread_scope` and
`hydra_threads` trap (asserted by
`solutions_build_and_run_in_the_real_server_sandbox`), and `async_fn` has
no executor. `hydra_body` is counted as behaviour-checked: its harness
checks `await_heads` and does not run its threaded `main` (which also
traps in the sandbox). `expectations::tests::classification_totals` pins
these numbers.

**The three Act 7 "concurrency" missions do not need threads.** Classified
by running them, not by name: the canonical solutions of `arc_mutex`,
`mpsc_channel` and `atomic` use `Arc<Mutex>`, a channel and an atomic on
the main thread only (no `spawn`), and all three built and ran cleanly in
the real sandbox (`print_canonical_transcripts`: `ok=true`, before and
after this change). The `arc_mutex` Task says so: the sandbox runs one
thread and the cloned `Arc` is what you would move into a spawned thread.

**Construct tokens (`uses`).** A short pinned output can be printed as
literals. Each of the 13 entries therefore also carries `uses`, token
groups checked after the behaviour matched (same rule as the compile-only
static check: comments and strings do not count); missing tokens give
`static_check_failed`. What this proves and does not: output + tokens
shows the program printed the right lines AND contains the construct;
it does NOT prove the construct produced those lines (a program could
call `Rc::clone` once and still print literals). The harness mission
(`enum_match`) is stronger: the checker calls `degrees` itself.

| id | old task -> new task | check |
|---|---|---|
| vec_iter | sum any `Vec<i32>` -> sum `vec![10, 20, 30]`, print `60` | output `60` + `.iter()`, `.sum` |
| closure_basic | call `add` with anything -> print `add(19, 23)`: `42` | output `42` + `let add`, `\|` |
| enum_match | any enum with 2+ variants mapped to anything -> `North/East/South/West`, `fn degrees(d: Direction) -> i32` giving 0/90/180/270, `main` prints all four | harness (4 calls) + main prints `0`,`90`,`180`,`270` + `match` |
| and_then | chain four times (nothing printed) -> print 2-step `Some(2)` then 4-step `None` | output lines in order + `.and_then(` |
| hashmap_basic | insert/get any key -> insert `7` at `"vault"`, print `get("vault")`, `get("cellar")`: `Some(7)`, `None` | output in order + `.insert(`, `.get(` |
| arc_mutex | lock and mutate -> `0`, `Arc::clone`, add `5` via clone's guard, print via first: `5` | output `5` + `Arc::clone`/`.clone()`, `.lock(` |
| mpsc_channel | send/recv one value -> send `7`, `11`, recv twice: `7`, `11` | output in order + `.send(`, `.recv(` |
| atomic | `fetch_add` once -> start `10`, print `fetch_add`'s return then `load`: `10`, `11` | output in order + `.fetch_add(`, `.load(` |
| rc_basic | `Rc::new` + `Rc::clone` -> print `strong_count` before clone, after clone, after `drop`: `1`, `2`, `1` | output in order + `Rc::clone`, `strong_count` |
| refcell | mutate via `borrow_mut` -> `10`, print, add `5` via a shared `&`, print: `10`, `15` | output in order + `.borrow_mut(` |
| cell | `set` then `get` -> `1`, print, `set(5)` via a shared `&`, print: `1`, `5` | output in order + `.set(`, `.get(` |
| rc_refcell | mutate through the clone -> `0`, add `10` via clone and `5` via original, print: `15` | output `15` + `Rc::clone`/`.clone()`, `.borrow_mut(` |
| weak_ref | `downgrade` + `upgrade` -> print `upgrade()` before and after `drop`: `Some(5)`, `None` | output in order + `downgrade`, `.upgrade(` |

Wrong-but-compiling variants that now fail (all in
`compile-api/tests/expectations_proof.rs`, alongside a literal-print
program per mission that must be `static_check_failed`): `vec_iter` one
element short (`30`) and `.count()` (`3`); `closure_basic` `a - b`, `a + a`;
`enum_match` East/West swapped, and a `_ =>` catch-all with a `main` that
prints the right lines (the harness catches it); `and_then` three steps
(`Some(1)`); `hashmap_basic` unwrapped value, wrong key; `arc_mutex` value
copied out of the guard, and a second `Arc` around a copy (`0`);
`mpsc_channel` one `recv`; `atomic` `load` printed twice, and "fetch_add
returns the new value" (`11`, `11`); `rc_basic` `Rc::new(clone of the
String)` (`1`,`1`,`1`) and no `drop` (`1`,`2`,`2`); `refcell` value copied
out (`10`,`10`); `cell` changed the `get()` copy (`1`,`1`); `rc_refcell` two
boxes (`5`) and printing via the clone before the second change (`10`);
`weak_ref` upgraded `Rc` kept alive across the `drop` (`Some(5)`,`Some(5)`).

`every_expectation_discriminates` (`--ignored`, 195 real wasm builds +
sandbox runs): 0 unexpected results; **54 of 54 behaviour-checked missions
proven** (canonical passes, starter fails, every wrong variant fails), and
all 13 literal-print programs are `static_check_failed`.

Every starter still compiles and fails its check (prints nothing, `todo`
or `0`; `enum_match`'s starter has no `degrees`, so the checker cannot
build). The expected lines in `data.rs` were recorded by running the new
solutions through the real build + sandbox path
(`print_canonical_transcripts`). `game/tests/mission_text.rs` now asserts
each pinned line is shown in the Task text, and (`--ignored`) that each
solution prints exactly that natively, and that the outputs the new Hints
claim for common mistakes (`rc_basic`, `refcell`, `cell`, `rc_refcell`,
`weak_ref`) are what those programs print.

Grader tables (`stub_grader.rs`, `grader.rs`): no token changed. The
`enum_match` contract fixture now uses the new `degrees` shape; the other
12 fixtures still satisfy their unchanged token rules. Snapshots
regenerated (`PNC_UPDATE_GRADER_SNAPSHOTS=1`): 349 cases before and after
in each file; 332 byte-identical; 17 inputs changed (13 starters whose
comments or text changed, and 4 `enum_match` fixture cases: canonical and
3 near-misses); for every `(id, case)` the verdict (ok, stdout, stderr) is
identical before and after: **0 verdict or message changes.**

## Progression

Gating is real and strict-linear. The `prereq: None` in every literal is
a placeholder: `MissionRegistry::default` then sets each prereq to the
previous mission. `game/tests/mission_text.rs::progression_is_strict_linear`
proves mission N+1 stays locked until N is cleared, for every N.

## The Borrow Checker's trial (2026-11-14)

Three stage missions after `while_loop`, owned by the Borrow Checker
(`game/src/plugins/trial.rs`): `bridge_toll` (move vs borrow),
`bridge_quill` (many `&` or one `&mut`), `bridge_lantern` (a lifetime on a
returned slice). 60 missions, 57 NPCs. The free demo ends on the trial: 24
free missions, 36 after.

Evidence (rustc 1.99.0, edition 2024):

- `mission_text -- --include-ignored`: every starter fails with its stage's
  one error (E0382, E0502, E0106); `borrow_checker_claims_are_what_rustc_emits`
  compiles every error the Borrow Checker names and the wording he quotes
  (E0382 "borrow of moved value: `cargo`", E0502 "cannot borrow `scroll` as
  mutable because it is also borrowed as immutable", E0106 "missing lifetime
  specifier" and its help tying `sep` to `'a`, E0597 "`sep` does not live
  long enough" after following that help), plus "`weigh(cargo.clone())`
  also compiles". A fast test fails if he names an uncompiled code.
- `solutions -- --include-ignored`: 60/60 solutions, native `-D warnings`,
  `wasm32-wasip1`, sandbox run.
- `contract -- --include-ignored`: grader corpus 60/60 (a, b, c, f), 45/45
  correct (3 trial alternatives), 16/16 wrong (3 trial shortcuts); 105/105
  corpus programs build.
- `expectations_proof -- --ignored`: each stage is a harness check pinning
  the signature; canonical and one alternative pass, starter and three
  wrong-but-compiling variants per stage fail (57 behaviour-checked
  missions proven).
- Grader snapshots: 18 new cases per grader, 0 changed rows.

## The Act 3-5 trials (2026-11-15)

Nine stage missions on the same mechanism, each trial owned by its act's
lead NPC, who keeps the act's first mission as a prologue:

- **The Variant Wraith** (the Guildmaster, after `enum_data_match`):
  `wraith_forms` (a `_` arm answers the Shadow wrongly; name every
  variant), `wraith_essence` (bind struct-variant fields in the pattern),
  `wraith_possessed` (a fourth variant makes the match non-exhaustive).
- **The Chimera of Vexis** (Vexis, after `assoc_type`): `chimera_heads`
  (`impl Enemy for` each head), `chimera_bound` (`volley<T: Enemy>`),
  `chimera_plasma` (`battle(&[Box<dyn Enemy>])` instead of a `match` on a
  `Head` enum).
- **The Tavernkeeper's Wager** (the Barkeep, after `unwrap_or_else`):
  `wager_stake` (`?` needs a `Result` return), `wager_from` (`From` impls
  so `?` converts), `wager_quest` (three steps chained with `?`).

69 missions, 57 NPCs. The demo is unchanged: 24 free missions, ending on
`bridge_lantern`; 45 after.

Server checks: all nine are harness checks pinning each signature. Stages
2 and 3 of the Chimera implement the player's trait for a checker-owned
head and pass it in, so only a bounded generic or trait-object code
passes. `wraith_possessed`'s result depends on the variant's `will`, so a
blanket `_` arm cannot pass. Every Wager stage calls the player's code on
failing inputs (an `unwrap` on a failing step crashes the checker), and
`wager_quest` checks the step order, needs a `?`, and rejects `match`,
`unwrap`, `expect` and `panic!` through a new `MissionCheck::forbids` list
(whole-word tokens in code, comments and strings removed).

Evidence (rustc 1.99.0, edition 2024):

- `mission_text -- --include-ignored`: 11/11. Seven stage starters fail
  with exactly their one error (E0533, E0004, E0599 x2, E0308, E0277 x2);
  `wraith_forms` and `wager_quest` starters compile and misbehave.
  `trial_claims_are_what_rustc_emits`: 19 claims across the four trials,
  including the quoted wording, rustc's help text where a hint cites it,
  "also compiles" claims, and two runtime claims (`unwrap` panics after
  the stated output). The fast test fails if any trial owner names an
  uncompiled code; `trial_code_lines_fit_the_code_pane` holds every new
  starter, solution and tutorial code line to 72 columns.
- `solutions -- --include-ignored`: 69/69 native `-D warnings`, 69/69
  `wasm32-wasip1`, 67/69 run cleanly in the sandbox (the two thread
  missions trap, as before).
- `contract -- --include-ignored`: grader corpus 69/69 (a, b, c, f), 55/55
  correct (10 new trial alternatives), 21/21 wrong (5 new trial
  shortcuts).
- `expectations_proof -- --ignored`: 274 cases; for each new stage the
  canonical and every alternative pass, the starter and at least three
  wrong variants fail; `wager_quest` also has four forbidden-token variants
  and one `?`-less combinator chain (all `StaticCheckFailed`). 66
  behaviour-checked missions proven.
- Grader snapshots: 51 new cases per grader, 0 changed rows.

## The Act 6-8 trials (2026-11-15)

Nine more stage missions on the same mechanism, owned by each act's lead
NPC, who keeps the act's first mission as a prologue:

- **The Forge-Golem** (the Keymaster, after `closure_move`): `golem_parts`
  (filter before map; mapping first leaves only labels, E0609),
  `golem_sum` (`sum` needs a type, E0283), `golem_core` (total damage as
  one filter, map, fold chain; no `for`, `while`, `loop` or `mut`).
- **The Tide Hydra** (the Dockmaster, after `async_fn`): `hydra_senders`
  (a sender clone per head, E0382; one thread), `hydra_threads` (one
  `move` thread per head, E0373; sorted reports), `hydra_body` (count
  every `Down` before striking).
- **The Memory Warden** (the Vaultwright, after `weak_ref`, ending the
  registry): `warden_box` (a recursive ledger needs a `Box`, E0072),
  `warden_shared` (a shared vault as `Rc<RefCell<u32>>`, E0594),
  `warden_weak` (a ring of debts linked with `Weak` and walked once).

78 missions, 57 NPCs. The demo is unchanged: 24 free missions, ending on
`bridge_lantern`; 54 after.

Server checks: eight harness checks pin each signature and call the
player's code on inputs it never saw (parts, heads, ledgers, debtors and
rings of the checker's own). `golem_core` sends an armoured part first,
needs a `.fold(`/`.sum` and forbids `for`, `while`, `loop` and `mut`
(`MissionCheck::forbids`). `wasm32-wasip1` has no threads, so:

- `hydra_threads` is `CompileOnly` (static checks for `spawn(`, a `move`
  closure and `.sort`), the fourth static-only mission;
- `hydra_body` is a harness check with `run_main: false`: the checker
  fills a channel from one thread, drops the sender and calls
  `await_heads`; the player's threaded `main` is never run;
- both programs' outputs are pinned in `mission_text.rs` and proven
  natively, rerun 20 times after the first run (sorted reports; a head's
  hits reach the channel before its `Down`).

Evidence (rustc 1.99.0, edition 2024):

- `mission_text -- --include-ignored`: 11/11. 16 stage starters (and
  `lifetimes`) fail with exactly their one error (six new: E0609, E0283, E0382, E0373, E0072,
  E0594); `golem_core`, `hydra_body` and `warden_weak` starters compile
  and misbehave. `trial_claims_are_what_rustc_emits`: 28 claims across
  the seven trials (9 new, including rustc's `tx.clone()` and `move`
  help, the E0072 help text, two "also compiles" claims and the
  RefCell already borrowed panic). Hint claims also run the `golem_core`
  and `warden_weak` starters, a `take_while` variant, a
  `recv_timeout` Timeout/Disconnected program and two edited starters.
- `solutions -- --include-ignored`: 78/78 native `-D warnings`, 78/78
  `wasm32-wasip1`, 74/78 run cleanly in the sandbox (`thread_spawn`,
  `thread_scope`, `hydra_threads` and `hydra_body` trap, as listed).
- `contract -- --include-ignored`: grader corpus 78/78 (a, b, c, f),
  71/71 correct (16 new trial alternatives), 24/24 wrong (3 new trial
  shortcuts), 149/149 corpus programs compile.
- `expectations_proof -- --include-ignored`: 341 cases, 0 unexpected;
  for each new stage the canonical and every alternative pass (the two
  `hydra_threads` alternatives as `CompiledNotChecked`), the starter and
  at least three wrong variants fail; `golem_core` also has four
  forbidden-token variants and one fold-less recursion (all
  `StaticCheckFailed`). 74 behaviour-checked missions proven.
- Grader snapshots: 51 new cases per grader, 0 changed rows.

## Act 9: The Forbidden Library (2026-11-15)

Six missions and the Librarian's trial, after the Memory Warden:
`raw_pointer` (the Shelfmark), `extern_c` (the Envoy), `macro_rules` (the
Macro Monk), `drop_trait` (the Sexton), `send_sync` (the Courier),
`maybe_uninit` (the Bookbinder), then `librarian_index`,
`librarian_demand`, `librarian_seal` (the Librarian, no prologue: his
roster mission is stage 1). 87 missions, 64 NPCs. The demo is unchanged:
24 free missions, ending on `bridge_lantern`; 63 after.

Server checks: `raw_pointer`, `macro_rules`, `drop_trait`, `maybe_uninit`
and `librarian_demand` pin their stdout and need the construct's tokens
(`uses`). `extern_c`, `send_sync`, `librarian_index` and `librarian_seal`
are harness checks in `HarnessMode::UnsafeLateNonce` (unsafe allowed;
result key delivered after the player's code finishes; forgery limit in
`compile-api/HARNESS-NOTES.md`) that call the player's items with the
checker's own inputs; all but `extern_c` forbid `no_mangle`. The game
labels those four passes NOT VERIFIED. No Miri run backs the `unsafe`
solutions.

Evidence (rustc 1.99.0, edition 2024; editions 2021 and 2024 and
`wasm32-wasip1` checked separately for every Act 9 program, see the
spec's Implementation record):

- `mission_text -- --include-ignored`: 13/13. Six new starters fail with
  exactly their error (E0133 x3, E0040, E0277, E0381), two with exactly a
  code-less macro error, `librarian_seal`'s compiles and misbehaves.
  `trial_claims_are_what_rustc_emits`: 33 claims (5 new, the Librarian's);
  `act9_villager_claims_are_what_rustc_emits`: 15 claims (compiler errors,
  edition-2024 `unsafe extern` / `unsafe(no_mangle)`, macro behaviour,
  `process::exit`, `Rc`'s missing `Send`/`Sync`, `MaybeUninit` not
  dropping). Pinned outputs for all nine and two new hint claims.
- `solutions -- --include-ignored`: 87/87 native `-D warnings`, 87/87
  `wasm32-wasip1`, sandbox run as listed (the four thread programs trap;
  every Act 9 solution runs cleanly).
- `contract -- --include-ignored`: grader corpus 87/87 (a, b, c, f), 76/76
  correct (5 new), 28/28 wrong (4 new), 163/163 corpus programs compile.
- `expectations_proof -- --include-ignored`: 396 cases, 0 unexpected (55
  Act 9); 83 behaviour-checked missions proven.
- `act9_unsafe_harness -- --include-ignored`: 12/12 fast and slow, 47
  attack/fixture cases graded as expected against the REGISTERED checks;
  the two documented known-limit tests still behave as documented.
- Grader snapshots: new rows only, 0 changed rows.
