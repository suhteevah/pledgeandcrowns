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

## Progression

Gating is real and strict-linear. The `prereq: None` in every literal is
a placeholder: `MissionRegistry::default` then sets each prereq to the
previous mission. `game/tests/mission_text.rs::progression_is_strict_linear`
proves mission N+1 stays locked until N is cleared, for every N.
