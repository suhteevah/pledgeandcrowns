# Act 4 — Trait Mage's Tower mission batch (design spec)

**Date:** 2026-06-18
**Status:** approved (Matt: "go into act 4")
**Topic:** 6 missions covering Act 4's trait/generic tier, in the existing
mission-tutorial architecture (same shape as the Act 3 batch).

## Goal

`design/01-curriculum.md` §"Act 4 — The Trait Mage's Tower" teaches
generics (fns + structs), trait definitions + bounds, default methods,
blanket impls, supertraits, trait objects (`dyn`/`Box<dyn>`), associated
types, and an `Iterator` teaser. This batch ships the **6 cleanly
pattern-gradeable core concepts**, appended to the strict-linear chain
after `enum_data_match` (mission 27 → 33).

| # | id | NPC | Concept | Grader tokens |
|---|----|-----|---------|---------------|
| 28 | `trait_def` | Vexis the Archmage | define a `trait`, `impl … for` a type, call the method | `trait `, `impl `, ` for ` |
| 29 | `generic_fn` | The Wandwright | generic fn with a bound `<T: PartialOrd>` | `<T`, `PartialOrd` |
| 30 | `generic_struct` | The Conjurer | generic `struct Pair<T>` | `struct `, `<T>`, `: T` |
| 31 | `dyn_trait` | The Familiar | `Box<dyn Trait>` in a `Vec` (dynamic dispatch) | `Box<dyn`, `dyn ` |
| 32 | `lifetimes` | The Lanternkeeper | explicit lifetime `<'a>` (the "Lifetime Lantern") | `<'a>`, `&'a` |
| 33 | `assoc_type` | The Loremaster | associated type `type Output` | `type Output`, `Self::Output` |

(Mission numbers are registry order; prereqs auto-link linearly.)

## Non-goals

- Default methods, blanket impls, supertraits, the `Iterator` impl teaser:
  hard to pattern-grade cleanly (they're about absence/structure rather
  than token presence). Mentioned in tutorials; not their own missions.
- The RPG layer (Vexis's tower floors, the Grimoire UI, the Chimera boss):
  deferred like Acts 3, same as the rest of the MVP.

## Cross-grade note

`dyn_trait` and `assoc_type` canonicals necessarily contain a trait
definition, so they would also satisfy `trait_def`'s token set. This is
harmless: `grade()` only ever runs the arm for the requested encounter id,
and the cross-grade tests are hand-picked spot-checks (not all-pairs) — no
checked pair conflicts. The build-on-prior-concept overlap is curriculum-
correct (you can't have `Box<dyn Element>` without `trait Element`).

## Per-mission detail

### 28 — `trait_def` (Vexis the Archmage)
- **canonical:**
  ```rust
  trait Element {
      fn name(&self) -> &str;
  }
  struct Fire;
  impl Element for Fire {
      fn name(&self) -> &str { "fire" }
  }
  fn main() {
      let f = Fire;
      println!("{}", f.name());
  }
  ```
- **starter:** `struct Fire;` + a `main` whose comment avoids `trait `/
  `impl `/` for `.
- **pass:** `Vexis lowers his staff. "a capability named once, granted to a type — that is a trait."`

### 29 — `generic_fn` (The Wandwright)
- **canonical:**
  ```rust
  fn larger<T: PartialOrd>(a: T, b: T) -> T {
      if a > b { a } else { b }
  }
  fn main() {
      println!("{}", larger(3, 7));
  }
  ```
- **starter:** empty-ish `main` with a comment (no `<T`/`PartialOrd`).
- **pass:** `the Wandwright sights down the blank. "one wand, any element — bounded by what it can compare."`

### 30 — `generic_struct` (The Conjurer)
- **canonical:**
  ```rust
  struct Pair<T> {
      a: T,
      b: T,
  }
  fn main() {
      let p = Pair { a: 1, b: 2 };
      let _ = (p.a, p.b);
  }
  ```
- **starter:** `main` with a comment (no `struct `/`<T>`/`: T`).
- **pass:** `the Conjurer cups two matching lights. "a vessel for any type — so long as both halves agree."`

### 31 — `dyn_trait` (The Familiar)
- **canonical:**
  ```rust
  trait Element {
      fn name(&self) -> &str;
  }
  struct Fire;
  impl Element for Fire {
      fn name(&self) -> &str { "fire" }
  }
  struct Water;
  impl Element for Water {
      fn name(&self) -> &str { "water" }
  }
  fn main() {
      let zoo: Vec<Box<dyn Element>> = vec![Box::new(Fire), Box::new(Water)];
      for e in &zoo {
          println!("{}", e.name());
      }
  }
  ```
- **starter:** has a `trait Element` + two impls provided; the player adds
  the `Vec<Box<dyn Element>>`. Lacks `Box<dyn`/`dyn ` so it fails until they
  do. (Provides the boilerplate so the mission is about the trait object,
  not re-typing two impls.)
- **pass:** `the Familiar shifts through its forms. "many shapes, one cage — dispatched at a touch."`

### 32 — `lifetimes` (The Lanternkeeper)
- **canonical:**
  ```rust
  fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
      if x.len() > y.len() { x } else { y }
  }
  fn main() {
      let _ = longest("aa", "b");
  }
  ```
- **starter:** a `longest(x: &str, y: &str)` stub that won't compile/lacks
  `<'a>`/`&'a`; comment to add the lifetime.
- **pass:** `the Lanternkeeper keeps the flame. "the borrow lives exactly as long as 'a — no shorter."`

### 33 — `assoc_type` (The Loremaster)
- **canonical:**
  ```rust
  trait Producer {
      type Output;
      fn make(&self) -> Self::Output;
  }
  struct Coiner;
  impl Producer for Coiner {
      type Output = i32;
      fn make(&self) -> Self::Output { 7 }
  }
  fn main() {
      let c = Coiner;
      let _ = c.make();
  }
  ```
- **starter:** a `trait Producer` with a `make` returning a fixed type;
  comment to make the output an associated type. Lacks `type Output`/
  `Self::Output`.
- **pass:** `the Loremaster turns the page. "each producer names its own yield — the type follows the trait."`

## Touch-points (mirror of the Act 3 batch)

`mission.rs` (6 entries), `npc.rs` (6 `NpcSpec` + imports), `assets.rs`
(6 consts + `ALL_SPRITE_PATHS`), `compile-api/src/grader.rs` (6 arms +
unit tests), `game/src/plugins/stub_grader.rs` (6 byte-identical arms),
`game/tests/contract.rs` + `game/tests/stub_grader.rs` (6 canonicals each),
art batch 6 (`ref-36..41` + specs → sprites), `04b-art-deliverables.md`,
`01-curriculum.md` Act 4 code-status note.

## Acceptance criteria

- 33 missions in the registry; the 6 new reachable after the prior 27.
- `scripts/ci.ps1` green (fmt/check/clippy/test incl. contract + registry +
  asset suites + grader unit tests).
- `cargo test --workspace -- --ignored` green (all 33 canonicals compile).
- 33 NPCs, all with distinct art.
