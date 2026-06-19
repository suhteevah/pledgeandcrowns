# Act 6 — Iterator Forge mission batch (design spec)

**Date:** 2026-06-18
**Status:** approved (Matt: "go on 6")
**Topic:** 6 missions covering Act 6's collections/iterators/closures tier,
in the existing mission-tutorial architecture (same shape as Acts 3–5).

## Goal

`design/01-curriculum.md` §"Act 6 — The Iterator Forge" teaches collections
(`Vec`, `HashMap`), iterators, and closures. The prelude already has
`vec_iter` (`.iter().sum`), `iter_map_collect` (`.map().collect`), and
`closure_basic`. This batch fills the **6 gaps**, appended after
`unwrap_or_else` (registry 39 → 45).

| # | id | NPC | Concept | Grader tokens |
|---|----|-----|---------|---------------|
| 40 | `hashmap_basic` | The Keymaster | `HashMap` insert + get | `HashMap`, `.insert(`, `.get(` |
| 41 | `iter_filter` | The Sifter | `.filter()` then collect | `.filter(`, `.collect` |
| 42 | `iter_fold` | The Smelter | `.fold(init, \|acc, x\| …)` reduce | `.fold(` |
| 43 | `iter_enumerate` | The Tallywright | `.enumerate()` (index + value) | `.enumerate(` |
| 44 | `iter_zip` | The Riveter | `.zip()` (pair two iterators) | `.zip(` |
| 45 | `closure_move` | The Bondsmith | `move \|\|` closure (capture by value) | `move \|` |

## Per-mission canonical solutions

```rust
// hashmap_basic
use std::collections::HashMap;
fn main() {
    let mut scores = HashMap::new();
    scores.insert("ace", 1);
    let _ = scores.get("ace");
}

// iter_filter
fn main() {
    let v = vec![1, 2, 3, 4];
    let evens: Vec<&i32> = v.iter().filter(|x| **x % 2 == 0).collect();
    let _ = evens;
}

// iter_fold
fn main() {
    let v = vec![1, 2, 3];
    let total = v.iter().fold(0, |acc, x| acc + x);
    let _ = total;
}

// iter_enumerate
fn main() {
    let v = vec![10, 20, 30];
    for (i, x) in v.iter().enumerate() {
        let _ = (i, x);
    }
}

// iter_zip
fn main() {
    let a = vec![1, 2];
    let b = vec![3, 4];
    for (x, y) in a.iter().zip(b.iter()) {
        let _ = (x, y);
    }
}

// closure_move
fn main() {
    let name = String::from("Garin");
    let greet = move || println!("hail, {name}");
    greet();
}
```

Each canonical passes its grader, compiles under `cargo check`, and the
neutral starter fails (lacks ≥1 token). Token notes: `.filter(`/`.fold(`/
`.enumerate(`/`.zip(`/`move |` are each distinctive to their mission;
`.collect` on `iter_filter` is shared with `iter_map_collect` but the two
differ on `.filter(` vs `.map(`, and graders only run the requested arm.

## Touch-points (mirror of Acts 3–5 batches)

`mission.rs`, `npc.rs`, `assets.rs`, `compile-api/src/grader.rs` (+ unit
tests), `game/src/plugins/stub_grader.rs` (byte-identical), `contract.rs` +
`tests/stub_grader.rs` (canonicals), art batch 8 (`ref-48..53` + specs →
sprites), `04b-art-deliverables.md`, `01-curriculum.md` Act 6 note.

## Acceptance criteria

- 45 missions; the 6 new reachable after the prior 39.
- `scripts/ci.ps1` green; `cargo test --workspace -- --ignored` green (all
  45 canonicals compile). 45 NPCs, all with distinct art.
