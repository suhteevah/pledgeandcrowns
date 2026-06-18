# Act 3 — Guildhall Quarter mission batch (design spec)

**Date:** 2026-06-18
**Status:** approved (design shape confirmed by Matt)
**Topic:** Fill the doc's Act 3 curriculum gaps with 6 new missions, in the
existing mission-tutorial architecture (no new engine systems).

## Goal

`design/01-curriculum.md` §"Act 3 — The Guildhall Quarter" teaches structs,
enums, exhaustive `match`, **`if let`**, **`while let`**, **methods/`impl`**,
**`Self`/`self`**, **associated functions (`Self::new`)**, and tuple/unit
structs. The shipped 21-mission prelude already covers basic struct
(`struct_basic`), enum (`enum_match`), `match` on `Option` (`match_option`),
and `derive(Debug)` (`derive_debug`). This batch adds the **6 gaps**:

| # | id | NPC | Concept |
|---|----|-----|---------|
| 22 | `impl_method` | The Guildmaster | `impl` blocks, methods, `&self`, `self.field` |
| 23 | `assoc_new` | The Recruiter | associated function `Self::new`, `Self` |
| 24 | `if_let` | The Locksmith | `if let Some(x) =` (single-pattern match) |
| 25 | `while_let` | The Porter | `while let Some(x) = …pop()` (drain) |
| 26 | `tuple_struct` | The Surveyor | newtype `struct Meters(f64)` + `.0` |
| 27 | `enum_data_match` | The Armorer | enum with struct-variant **data** + exhaustive `match` (the doc's "Equipment Smith" encounter; renamed to **Armorer** to avoid colliding with the existing "The Smith") |

They append to the strict-linear chain after `while_loop`; `MissionRegistry::default`
sets each prereq to the prior mission automatically.

## Non-goals (out of scope, by the approved scope decision)

- No new zone — the 6 NPCs live in Hearthstone Village like the rest.
- No RPG layer (party tab, combat, the Variant Wraith boss). The doc
  describes these for Act 3; they require engine systems the MVP code
  doesn't have. Deferred.
- No `unit_struct` mission — folded as a mention into the `tuple_struct`
  tutorial (beginners rarely author marker types; low curriculum value).

## Per-mission detail

Each mission supplies: a `prompt`, a 4-section `tutorial`
(`## Concept`/`## Syntax`/`## Task`/`## Hint`, ~100-200 words, matching the
existing voice), a **neutral `starter_code`** (must NOT pass the grader),
a grader token set, a canonical solution, and byte-identical pass/fail
flavor in both graders.

### 22 — `impl_method` (The Guildmaster)
- **tokens:** `impl`, `&self`, `self.`
- **canonical:**
  ```rust
  struct Hero { level: i32 }
  impl Hero {
      fn power(&self) -> i32 { self.level * 10 }
  }
  fn main() {
      let h = Hero { level: 3 };
      println!("{}", h.power());
  }
  ```
- **starter:** `struct Hero { level: i32 }` + a `main` that builds one and a
  comment to add a `power(&self)` method — no `impl`/`&self`/`self.`.
- **pass:** `the Guildmaster nods. "duties bound to a name — that is a method."`
- **fail:** `the Guildmaster waits — {e}`

### 23 — `assoc_new` (The Recruiter)
- **tokens:** `fn new`, `Self`, `::new`
- **canonical:**
  ```rust
  struct Hero { level: i32 }
  impl Hero {
      fn new(level: i32) -> Self { Self { level } }
  }
  fn main() {
      let h = Hero::new(5);
      println!("{}", h.level);
  }
  ```
- **starter:** `struct Hero` + comment "add an associated function that
  returns `Self`, then construct a Hero with it" — no `fn new`/`Self`/`::new`.
- **pass:** `the Recruiter stamps the roll. "Self::new — a member forged from a name."`
- **fail:** `the Recruiter shakes her head — {e}`

### 24 — `if_let` (The Locksmith)
- **tokens:** `if let`, `Some(`
- **canonical:**
  ```rust
  fn main() {
      let maybe: Option<i32> = Some(7);
      if let Some(n) = maybe {
          println!("{n}");
      }
  }
  ```
- **starter:** has `Some(7)` (so it lacks only `if let`) → fails. Mirrors the
  `match_option` precedent (starter may contain a token as long as it does
  not pass on all tokens).
- **pass:** `the Locksmith turns the one key that fits. "Some — and only Some — opens."`
- **fail:** `the Locksmith frowns — {e}`

### 25 — `while_let` (The Porter)
- **tokens:** `while let`, `.pop(`
- **canonical:**
  ```rust
  fn main() {
      let mut stack = vec![1, 2, 3];
      while let Some(top) = stack.pop() {
          println!("{top}");
      }
  }
  ```
- **starter:** builds the stack, comment to drain with `while let` + `.pop()`
  — no `while let`/`.pop(`.
- **pass:** `the Porter empties the cart, crate by crate. "while there is a Some, keep unloading."`
- **fail:** `the Porter sets the crate down — {e}`

### 26 — `tuple_struct` (The Surveyor)
- **tokens:** `struct Meters(`, `.0`
- **canonical:**
  ```rust
  struct Meters(f64);
  fn main() {
      let d = Meters(3.5);
      println!("{}", d.0);
  }
  ```
- **starter:** a `main` with a raw `f64` and a comment to wrap it in a
  `Meters` tuple struct and read `.0` — no `struct Meters(`/`.0`.
- **pass:** `the Surveyor reads the rod. "a bare number, now named Meters — a newtype."`
- **fail:** `the Surveyor squints down the line — {e}`

### 27 — `enum_data_match` (The Armorer)
- **tokens:** `enum Item`, `Weapon`, `match`, `=>`
- **canonical:**
  ```rust
  enum Item {
      Weapon { damage: i32 },
      Potion { heal: i32 },
  }
  fn value(item: Item) -> i32 {
      match item {
          Item::Weapon { damage } => damage,
          Item::Potion { heal } => heal,
      }
  }
  fn main() {
      let _ = value(Item::Weapon { damage: 10 });
  }
  ```
- **starter:** a placeholder `value(item: i32)` and a comment to define
  `enum Item` with `Weapon { damage }` / `Potion { heal }` and match
  exhaustively — no `enum Item`/`Weapon`/`match`/`=>`.
- **pass:** `the Armorer lays out the rack. "weapon or potion — name every kind, miss none."`
- **fail:** `the Armorer taps the anvil — {e}`

## NPCs (placement in `npc.rs::NPC_ROSTER`)

Six `NpcSpec` entries, `native_px: 32.0`, spread across the village clear of
existing NPCs and >28px apart (INTERACT_RADIUS):

| NPC | mission_id | pos |
|-----|-----------|-----|
| The Guildmaster | impl_method | (120, 64) |
| The Recruiter | assoc_new | (-120, -64) |
| The Locksmith | if_let | (160, 96) |
| The Porter | while_let | (-160, -96) |
| The Surveyor | tuple_struct | (60, -100) |
| The Armorer | enum_data_match | (-60, 100) |

## Art (batch 5)

Six first-pass 32×32 sprites via the canonical `render-refs` pipeline
(`design/art/refs/ref-30..35-*.jsx` → PNG → `game/assets/sprites/npc/`),
palette-compliant (Heraldic Code), each with a Guildhall-themed prop
(guild seal, recruit-roll, ring of keys, hand-cart, surveyor's rod, weapon
rack). New `SPRITE_*` constants in `assets.rs` + `ALL_SPRITE_PATHS`. These
are first-pass authored sprites pending Matt's locked art-review (the
3-revision flow), consistent with batch 4. Specs land in
`design/art/specs/{slug}.md`; the deliverables manifest logs batch 5.

## Touch-points (files changed)

1. `game/src/plugins/mission.rs` — 6 `Mission` entries (linear prereq auto).
2. `game/src/plugins/npc.rs` — 6 `NpcSpec` + 6 sprite imports.
3. `game/src/assets.rs` — 6 `SPRITE_*` consts + `ALL_SPRITE_PATHS`.
4. `compile-api/src/grader.rs` — 6 `grade()` arms.
5. `game/src/plugins/stub_grader.rs` — 6 mirrored arms (flavor byte-identical).
6. `game/tests/contract.rs` — 6 `canonical_solution()` entries.
7. `design/art/refs/ref-30..35-*.jsx`, `design/art/specs/*.md`,
   `design/04b-art-deliverables.md` — art batch 5.
8. `design/01-curriculum.md` — note the 6 Act-3-gap missions landed in code.

## Correctness contract (must stay green)

`game/tests/contract.rs` enforces, for every registry mission:
- a canonical solution exists, **passes** the grader, and (slow) passes
  `cargo check`;
- the **starter does NOT pass** (no trivial win);
- server (`grader.rs`) and stub (`stub_grader.rs`) flavor agree
  **byte-for-byte** (modulo the `[stub] ` prefix);
- no mission falls through to the freeform stub.

Plus `registry.rs` (linear prereq, no cycles, full reachability) and the
asset-existence audit (every `ALL_SPRITE_PATHS` file exists). A
`voice-editor` pass aligns the 6 new NPCs' dialogue with the cast.

## Acceptance criteria

- 27 missions in the registry; the 6 new ones reachable by clearing the
  prior 21 in order.
- `scripts/ci.ps1` green (fmt + check + clippy `-D warnings` + test),
  including the contract + registry + asset suites.
- The slow suite green: `cargo test --workspace -- --ignored` (canonical
  solutions compile; cargo-check parity).
- All 27 NPCs carry distinct art.
