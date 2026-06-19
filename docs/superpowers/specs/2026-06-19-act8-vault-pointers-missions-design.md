# Act 8 — Vault of Pointers mission batch (design spec)

**Date:** 2026-06-19
**Status:** approved (Matt: "go on act 8")
**Topic:** 6 missions covering Act 8's smart-pointer tier, same recipe as
Acts 3–7.

## Goal

`design/01-curriculum.md` §"Act 8 — The Vault of Pointers" teaches `Box`,
`Rc`, `Arc`, `Weak`, `RefCell`, `Cell`. The prelude/Act 7 already has `Arc`
(`arc_mutex`). This batch ships 6 missions appended after `async_fn`
(registry 51 → 57).

| # | id | NPC | Concept | Grader tokens |
|---|----|-----|---------|---------------|
| 52 | `box_basic` | The Vaultwright | `Box<T>` heap (recursive type) | `Box<`, `Box::new` |
| 53 | `rc_basic` | The Sharekeeper | `Rc::new` + `Rc::clone` | `Rc::new`, `Rc::clone` |
| 54 | `refcell` | The Warden | `RefCell` + `.borrow_mut()` | `RefCell`, `.borrow_mut(` |
| 55 | `cell` | The Swapwarden | `Cell` + `.set()` | `Cell::new`, `.set(` |
| 56 | `rc_refcell` | The Strongbox | `Rc<RefCell<T>>` shared-mutable | `Rc::new`, `RefCell`, `.borrow_mut(` |
| 57 | `weak_ref` | The Ghostkeeper | `Weak` + `Rc::downgrade` + `.upgrade()` | `Weak`, `downgrade`, `.upgrade(` |

## Per-mission canonical solutions (all host `cargo check` clean)

```rust
// box_basic
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}
fn main() {
    let list = Node {
        value: 1,
        next: Some(Box::new(Node { value: 2, next: None })),
    };
    let _ = list;
}

// rc_basic
use std::rc::Rc;
fn main() {
    let original = Rc::new(String::from("hoard"));
    let shared = Rc::clone(&original);
    let _ = (original, shared);
}

// refcell
use std::cell::RefCell;
fn main() {
    let guarded = RefCell::new(0);
    *guarded.borrow_mut() += 1;
    let _ = guarded.borrow();
}

// cell
use std::cell::Cell;
fn main() {
    let slot = Cell::new(1);
    slot.set(5);
    let _ = slot.get();
}

// rc_refcell
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let shared = Rc::new(RefCell::new(0));
    let clone = Rc::clone(&shared);
    *clone.borrow_mut() += 1;
    let _ = shared.borrow();
}

// weak_ref
use std::rc::{Rc, Weak};
fn main() {
    let strong = Rc::new(5);
    let weak: Weak<i32> = Rc::downgrade(&strong);
    let _ = weak.upgrade();
}
```

Each canonical passes its grader, compiles on host, and the neutral starter
fails. Cross-grade overlaps are deliberate and harmless (`rc_refcell`
contains `Rc`+`RefCell`; `cell`'s `Cell::new` is a substring of
`RefCell::new`) — graders only run the requested arm and the cross-grade
tests are hand-picked spot-checks.

## Touch-points (the recipe)

`mission.rs`, `npc.rs`, `assets.rs`, `compile-api/src/grader.rs` (+ unit
tests), `game/src/plugins/stub_grader.rs` (byte-identical), `contract.rs` +
`tests/stub_grader.rs` (canonicals), art batch 10 (`ref-60..65` + specs),
`04b-art-deliverables.md`, `01-curriculum.md` Act 8 note.

## Acceptance criteria

- 57 missions; the 6 new reachable after the prior 51.
- `scripts/ci.ps1` green; `cargo test --workspace -- --ignored` green (all
  57 canonicals compile on host). 57 NPCs, all with distinct art.
