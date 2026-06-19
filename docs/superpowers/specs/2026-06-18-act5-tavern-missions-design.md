# Act 5 — Tavern of Tribulations mission batch (design spec)

**Date:** 2026-06-18
**Status:** approved (Matt: "go on act 5")
**Topic:** 6 missions covering Act 5's error-handling tier, in the existing
mission-tutorial architecture (same shape as the Act 3 / Act 4 batches).

## Goal

`design/01-curriculum.md` §"Act 5 — The Tavern of Tribulations" teaches
`Option`/`Result` methods, `?`, custom error types (via `thiserror`), the
`From` trait for error conversion, `panic!` vs recoverable, the `unwrap*`
family, and error-chain propagation. The prelude already has `match_option`,
`option_unwrap_or`, and `result_question_mark`. This batch fills the **6
gaps**, appended to the chain after `assoc_type` (registry 33 → 39).

| # | id | NPC | Concept | Grader tokens |
|---|----|-----|---------|---------------|
| 34 | `result_match` | The Barkeep | `match` on `Result` (Ok/Err) | `match `, `Ok(`, `Err(` |
| 35 | `custom_error` | The Bouncer | custom error `enum` returned in a `Result` | `enum `, `Result<`, `Err(` |
| 36 | `from_error` | The Interpreter | `impl From<X> for Err` (error conversion) | `impl From<`, ` for `, `fn from` |
| 37 | `option_map` | The Mixologist | `.map()` on an `Option` | `Option<`, `.map(` |
| 38 | `and_then` | The Tabkeeper | `.and_then()` fallible chaining | `Option`, `.and_then(` |
| 39 | `unwrap_or_else` | The Cellarer | `.unwrap_or_else(\|\| …)` lazy default | `.unwrap_or_else(`, `\|\|` |

**Dependency-free note:** the doc names `thiserror` for custom errors, but
the compile sandbox is dep-free. `custom_error` teaches a hand-written
`enum` error (std-only, and the right thing to understand first); the
tutorial mentions `thiserror` as the real-world shortcut. `.unwrap_or_else`
does NOT trip `option_unwrap_or`'s `.unwrap_or(` token (different next char).

## Per-mission canonical solutions

```rust
// result_match
fn describe(r: Result<i32, String>) -> i32 {
    match r {
        Ok(v) => v,
        Err(_e) => -1,
    }
}
fn main() { let _ = describe(Ok(5)); }

// custom_error
enum BrewError { TooHot, TooCold }
fn check(temp: i32) -> Result<i32, BrewError> {
    if temp > 100 { Err(BrewError::TooHot) }
    else if temp < 0 { Err(BrewError::TooCold) }
    else { Ok(temp) }
}
fn main() { let _ = check(50); }

// from_error
struct ParseFail;
enum AppError { Parse }
impl From<ParseFail> for AppError {
    fn from(_e: ParseFail) -> Self { AppError::Parse }
}
fn main() { let _e: AppError = ParseFail.into(); }

// option_map
fn add_one(o: Option<i32>) -> Option<i32> { o.map(|x| x + 1) }
fn main() { let _ = add_one(Some(3)); }

// and_then
fn half(n: i32) -> Option<i32> {
    if n % 2 == 0 { Some(n / 2) } else { None }
}
fn main() { let _ = Some(8).and_then(half).and_then(half); }

// unwrap_or_else
fn value(o: Option<i32>) -> i32 { o.unwrap_or_else(|| 0) }
fn main() { let _ = value(None); }
```

Each canonical passes its grader, compiles under `cargo check`, and the
neutral starter fails (lacks ≥1 token). Cross-grade overlaps (`custom_error`
contains `Err(`/`Ok(`; `option_map`/`and_then` share `.map`/Option families)
are harmless — graders only run the requested arm and cross-grade tests are
hand-picked spot-checks.

## Touch-points (mirror of Acts 3–4 batches)

`mission.rs`, `npc.rs`, `assets.rs`, `compile-api/src/grader.rs` (+ unit
tests), `game/src/plugins/stub_grader.rs` (byte-identical), `contract.rs` +
`tests/stub_grader.rs` (canonicals), art batch 7 (`ref-42..47` + specs →
sprites), `04b-art-deliverables.md`, `01-curriculum.md` Act 5 note.

## Acceptance criteria

- 39 missions; the 6 new reachable after the prior 33.
- `scripts/ci.ps1` green; `cargo test --workspace -- --ignored` green (all
  39 canonicals compile). 39 NPCs, all with distinct art.
