# Act 7 — Concurrent Coast mission batch (design spec)

**Date:** 2026-06-18
**Status:** approved (Matt: "go on act 7")
**Topic:** 6 missions covering Act 7's concurrency tier, in the existing
mission-tutorial architecture (same shape as Acts 3–6).

## Goal

`design/01-curriculum.md` §"Act 7 — The Concurrent Coast" teaches threads,
`Arc<Mutex<T>>`, channels (`mpsc`), and `async`/`await` (tokio basics). This
batch ships 6 missions appended after `closure_move` (registry 45 → 51).

| # | id | NPC | Concept | Grader tokens |
|---|----|-----|---------|---------------|
| 46 | `thread_spawn` | The Dockmaster | `thread::spawn` + `.join()` | `thread::spawn`, `.join(` |
| 47 | `arc_mutex` | The Lighthouse Keeper | `Arc<Mutex<T>>` + `.lock()` | `Arc`, `Mutex`, `.lock(` |
| 48 | `mpsc_channel` | The Signaler | `mpsc::channel` send/recv | `mpsc::channel`, `.send(`, `.recv(` |
| 49 | `atomic` | The Tidewatch | `AtomicUsize` + `.fetch_add()` | `Atomic`, `.fetch_add(` |
| 50 | `thread_scope` | The Harbormaster | `thread::scope` (borrow across threads) | `thread::scope`, `.spawn(` |
| 51 | `async_fn` | The Tideforecaster | `async fn` + `.await` | `async fn`, `.await` |

## Sandbox note (important)

The grading sandbox is **dependency-free** and runs `cargo check` on the
host (`cargo_grader`). All std-concurrency canonicals (threads, Arc/Mutex,
mpsc, atomics, scope) compile and check on the host with no deps — solid.
**`async_fn` is compile-only:** `async fn` + `.await` *compile* fine (the
futures are created but never polled — no runtime needed to type-check),
but actually *running* async needs an executor like `tokio`, which the
sandbox can't pull in. The tutorial says so explicitly. This is a known
limitation of the pattern/compile grader for runtime-dependent features;
the concept (write an `async fn`, `.await` another) is still taught and
graded by token + compile.

## Per-mission canonical solutions (all `cargo check` clean on host)

```rust
// thread_spawn
use std::thread;
fn main() {
    let handle = thread::spawn(|| 21 * 2);
    let _ = handle.join();
}

// arc_mutex
use std::sync::{Arc, Mutex};
fn main() {
    let shared = Arc::new(Mutex::new(0));
    {
        let mut guard = shared.lock().unwrap();
        *guard += 1;
    }
    let _ = shared;
}

// mpsc_channel
use std::sync::mpsc;
fn main() {
    let (tx, rx) = mpsc::channel();
    tx.send(7).unwrap();
    let _ = rx.recv();
}

// atomic
use std::sync::atomic::{AtomicUsize, Ordering};
fn main() {
    let counter = AtomicUsize::new(0);
    counter.fetch_add(1, Ordering::SeqCst);
    let _ = counter;
}

// thread_scope
use std::thread;
fn main() {
    let data = vec![1, 2, 3];
    thread::scope(|s| {
        s.spawn(|| {
            let _ = &data;
        });
    });
}

// async_fn (compiles; futures never polled — no runtime here)
async fn double(x: i32) -> i32 {
    x * 2
}
async fn run() -> i32 {
    double(21).await
}
fn main() {
    let _future = run();
}
```

Each canonical passes its grader, compiles under host `cargo check`, and the
neutral starter fails (lacks ≥1 token). `thread_scope`'s `.spawn(` is the
scoped `s.spawn`, distinct from `thread_spawn`'s `thread::spawn` — graders
only run the requested arm regardless.

## Touch-points (mirror of Acts 3–6 batches)

`mission.rs`, `npc.rs`, `assets.rs`, `compile-api/src/grader.rs` (+ unit
tests), `game/src/plugins/stub_grader.rs` (byte-identical), `contract.rs` +
`tests/stub_grader.rs` (canonicals), art batch 9 (`ref-54..59` + specs →
sprites), `04b-art-deliverables.md`, `01-curriculum.md` Act 7 note.

## Acceptance criteria

- 51 missions; the 6 new reachable after the prior 45.
- `scripts/ci.ps1` green; `cargo test --workspace -- --ignored` green (all
  51 canonicals compile on host). 51 NPCs, all with distinct art.
