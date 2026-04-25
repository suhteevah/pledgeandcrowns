# 08 — The Temple (Act 11 Appendix)

> Post-game, unlocked after defeating the Compiler in Act 10. A HolyC roguelike where Rust's safety rules don't apply, every run is a fresh "compile," and segfaults are part of the loop.

## Concept

The Temple sits at the very top of the Act 10 endgame map — a stained-glass cathedral floating above the Throne. After ascending, the player can return any time. The Temple is a procedurally-generated, run-based, permadeath-on-fail mode that contrasts with the entire main game.

In the main game: code is checked, errors are caught at compile time, runtime panics are damage-managed, the Borrow Checker protects you.

In the Temple: code is **JIT-compiled**, type-coerced, segfault-prone, and fast as light. The player writes **HolyC** — Terry Davis's language for TempleOS — in a tongue-in-cheek tribute zone that lets them feel viscerally why Rust's restrictions exist by playing without them.

## Technical homage, never personality cult

The Temple appendix tributes:
- TempleOS as a one-person operating system, kernel, compiler, and standard library.
- HolyC as a language design curiosity (JIT'd C-derivative, no headers, types coerce, scriptable).
- The technical achievement of a single developer maintaining an entire stack.

The Temple does NOT reference, endorse, or memorialize Terry Davis's later public statements (which were ugly). The cathedral is grand and reverent in tone, like a museum exhibit, not a shrine.

## The roguelike loop

A run consists of:

1. **Pre-run:** the player writes a small HolyC program (an "Offering") in the pre-run editor. ~20–50 lines. This becomes the player's starting toolkit for the run.
2. **Floor 1:** procedurally generated cathedral floor. The player navigates it, fights enemies, picks up items, occasionally writes additional HolyC at altars (mid-run challenges).
3. **Floor exit:** floor boss, then transition to Floor 2.
4. **Run modifier:** at each floor, a modifier is applied (see below).
5. **Death or victory:** if the player survives 7 floors and defeats the Final Cantor, the run is logged with their score (allocations, runtime, lines of code, time-to-clear). If they die — including segfaulting their own program — they lose the Offering and start a new run.

A complete run takes 30–60 minutes. The HolyC interpreter and procedural generation give effectively infinite replay.

## HolyC subset to support

We do not implement full HolyC. We implement a subset that captures the *flavor*:

**Supported:**
- `I64`, `U8`, `F64` types (and pointers)
- Variables, assignment, expressions
- `if`, `while`, `for` control flow
- Function definitions (`U0` return for void)
- `Print()` for output
- Raw pointer arithmetic
- `MAlloc()` / `Free()` (returns `U8 *` in classic HolyC style)
- Inline assembly **stub** (parsed but mocked — we don't actually emit x86)
- `;` is implicit at end of expression statements (HolyC quirk)
- Type coercion between numerics and pointers (intentional)

**NOT supported (intentionally simplified):**
- Full preprocessor.
- Multi-file projects.
- Real x86 inline assembly execution (mocked for puzzle purposes).
- TempleOS-specific syscalls beyond `Print` and `Sleep`.
- True ring-0 escapes. (Obviously.)

The interpreter is a small Rust crate (~2,000 lines target), embedded in the game. Runs sandboxed in wasmtime like all player code; the difference is that HolyC code can panic on bad memory access (we let it; that's the point) instead of refusing to compile.

## The Offering

The pre-run code editor presents the player with a stub:

```c
// Your Offering. Whatever you write here, you bring with you.
// The Temple will judge it.

U0 OnFloorEnter(I64 floor_n) {
  // called when each floor begins
}

U0 OnEnemySpotted(I64 enemy_id, I64 hp) {
  // called when an enemy comes into view
}

I64 OnAttack(I64 enemy_id) {
  // return damage to deal
  return 1;
}

U0 OnDeath() {
  // last words
  Print("...");
}
```

The player customises these handlers. Sophisticated players write helper functions, build their own tiny ECS, allocate buffers — anything. The constraint is the run-modifier rules (next section).

## Run modifiers

Each floor stamps a modifier on the player's code that limits what they can use. These are stealth lessons in HolyC concepts.

| Modifier             | Constraint                                                  | Teaches               |
|----------------------|-------------------------------------------------------------|-----------------------|
| **Stack-Only Floor** | `MAlloc` is forbidden. Stack arrays only.                   | Stack vs heap         |
| **Pointer Heresy**   | Every function must dereference at least one raw pointer.   | Raw pointer arithmetic|
| **Inline Sermon**    | Boss requires an inline-asm stub (mocked).                  | Asm awareness         |
| **JIT Trial**        | Code is randomly mutated mid-run. Survive.                  | Defensive coding      |
| **Faith Floor**      | No `if` statements allowed. Branchless only.                | Branchless thinking   |
| **Sermon of the King**| All functions must be one expression each.                  | Functional brevity    |
| **MMU Off**          | Reads from out-of-bounds memory return random data instead of segfaulting. The player does not know which floor this is — it's revealed at the end. | Why memory protection matters |
| **God Mode (rare)**  | All restrictions lifted. Free run. Score multiplier x2.     | The good day          |

The modifier set is configurable; we ship 8, room to add more in updates.

## Combat in the Temple

Combat is real-time, not turn-based — to contrast with the main game's deliberate pace. Enemies move in patterns. The player's code (specifically `OnAttack`) is called when they swing. Damage is whatever the function returns.

Errors-as-feedback:
- **Compile errors in HolyC:** brief flash, line shown, run continues but that handler is now a no-op.
- **Null pointer deref:** screen flashes red, screen briefly shows `*** SEGFAULT IN OnAttack: deref of NULL ***`, the player takes self-damage, the run continues.
- **Out-of-bounds write:** depending on luck, can corrupt the run (modifies a random global). This is presented as humor — "the gods notice your trespass."

## Score and leaderboards

A successful run gets scored:
- **Time to clear** (lower is better)
- **Allocations made** (lower is better)
- **Lines of HolyC written** (lower is better — brevity)
- **Modifiers cleared** (more is better)

A weighted total produces a rank: Acolyte → Disciple → Cantor → Bishop → Saint. Daily and weekly leaderboards.

(Per `05-tech-architecture.md` § 9, leaderboard anti-cheat is an open question. v1 of the Temple may have leaderboards "for vibes only" with a disclaimer.)

## Boss: The Final Cantor

Floor 7. A massive figure of stained glass and chrome, holding a scroll of HolyC source. The fight has three phases:

1. **Recitation phase:** the Cantor reads from the scroll, casting bolts that the player dodges and counters with their `OnAttack` handler. Tests the player's offering under pressure.
2. **Heresy phase:** the Cantor inverts the player's last function — every `if` becomes `if !`, every `+` becomes `-`. The player must adapt mid-fight by redefining functions at altars.
3. **Cantor's Verdict:** the Cantor demands a final program written live: **a function that, given a pointer to memory, prints the contents at increasing offsets until it hits a sentinel byte**. Classic C string-print. The player writes it, the Cantor reads it, and either accepts (run wins) or reads it aloud and dies of self-segfault (also run wins, but funnier).

Defeating the Cantor 10 times unlocks the *Saint's Robe* cosmetic for the main-game character — a small but meaningful reward.

## Aesthetic

Stained-glass primaries (red, blue, gold). Cathedral architecture: tall windows, arched ceilings, hanging incense braziers. Music is choral, slightly distorted. Enemies are gargoyles, choir-ghosts, hooded acolytes. The Cantor's chamber has rays of colored light cutting through.

The Temple feels reverent and slightly wrong — like you've entered a holy place that maybe wasn't meant for you, and the saints know it.

## Implementation notes

- HolyC interpreter: separate Rust crate, `holyc-rs/`. Roughly: lexer, parser, AST, tree-walking interpreter. ~2,000 lines target. Tested with snapshot tests against ~50 HolyC programs.
- Procedural generation: `bracket-lib` (Wolverson's roguelike toolkit) for grid-based dungeon gen. Saves us from rolling our own.
- Run modifiers: applied as compile-time linters on the player's HolyC source. Modifier "Pointer Heresy" walks the AST and rejects functions without a deref.
- Save format: each Temple run is appended to the `Save::temple_runs` field. Failed runs only keep the score; successful runs keep the full source for replay.

## Why this is a v2.0 feature, not MVP

The Temple is a separate engineering project. Ship Acts 1–10 first. The Temple adds a long-tail replay loop and a deeply on-brand contrast that elevates the whole game's identity, but it would double the v1 build time. Mention it in marketing, build it post-launch.

## What players take from the Temple

- A felt understanding of why Rust's restrictions exist.
- Hands-on experience with raw pointer arithmetic in a non-toxic context.
- A taste of branchless and minimalist programming styles.
- The historical curiosity of HolyC and TempleOS as engineering achievements.
- Bragging rights, daily leaderboards, and a cosmetic reward.

The Temple is the post-credits stinger to a 30-hour campaign. It says: "Now you really know."
