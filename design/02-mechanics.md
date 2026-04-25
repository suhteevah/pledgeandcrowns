# 02 — Mechanics

## The core loop in one sentence

The player encounters a problem in the world, the code editor opens, they write Rust to solve it, the engine compiles and runs that code in a sandbox, the world responds.

## The Cast Editor

The Cast Editor is the central UI. It is `bevy_egui` + `egui_code_editor` + a results panel. It opens whenever the player initiates an action that requires code (combat, crafting, dialogue puzzles, town construction).

```
┌─────────────────────────────────────────────────────────────┐
│  CAST  │  TEST  │  COMMIT                              ESC  │
├─────────────────────────┬───────────────────────────────────┤
│  fn cast() -> Spell {   │   GOAL                            │
│      // your code here  │   Defeat the goblin.              │
│  }                      │                                   │
│                         │   The function should return a    │
│                         │   Spell whose damage exceeds the  │
│                         │   goblin's HP (12).               │
│                         │                                   │
│                         │   AVAILABLE                       │
│                         │   - struct Spell { damage: i32 }  │
│                         │                                   │
│                         ├───────────────────────────────────┤
│                         │   COMPILER OUTPUT                 │
│                         │   (empty)                         │
└─────────────────────────┴───────────────────────────────────┘
```

**CAST** runs the code in the live encounter. **TEST** runs hidden test cases without committing to the encounter (no risk). **COMMIT** runs once and saves the result; only available after a successful CAST in some encounters.

## Compilation pipeline

Every CAST/TEST goes through the compile pipeline:

1. Player code is wrapped in a harness (the engine adds the `main`, the test scaffolding, the assertion machinery).
2. The wrapped source is sent to the compile API (Axum service).
3. The service runs `cargo build --target wasm32-unknown-unknown --release` in a sandboxed working dir.
4. The resulting `.wasm` is returned to the client.
5. The client loads the `.wasm` in `wasmtime` with strict resource limits (memory cap, instruction-count fuel, no system access).
6. The wasm is invoked, results are serialised back, the world reacts.

Errors at every stage surface in the Compiler Output panel with the player's source highlighted at the offending line. **The compiler errors the player sees are the actual rustc errors** — never paraphrased — because reading rustc output is itself a skill we want to teach.

## Combat

**Turn structure.** Initiative is a stat on `PartyMember` and `Enemy`. On a unit's turn, if it's the player, the Cast Editor opens with that turn's encounter context (enemy stats, available spells, terrain modifiers). On enemy turns, AI runs.

**Damage formula.** Each successful CAST returns a `Spell` (or higher-typed equivalent for later acts). Damage = `spell.base_damage * elegance_multiplier - target.defense`. Elegance multiplier is 1.0 by default; rises with clippy-clean code, falls with `unwrap()`s in non-test code. Clean code is harder-hitting code. Yes, this is opinionated.

**Failure modes.**
- Code does not compile → 0 damage, turn lost. Compiler errors shown.
- Code compiles but panics → unit takes self-damage equal to the panic damage table (configurable per encounter). `unwrap()`-on-`None` always panics.
- Code compiles, runs, returns wrong type → caught at the harness level, treated as a logic error, half damage.
- Code compiles, runs, returns right type but wrong value → some encounters check value-correctness via test cases; others let the value pass and let the in-world result speak.

**Persistent spells.** The player's previously-written, validated spell functions live in the **Spellbook** (a real Rust source file under the hood, stored per save). Subsequent encounters can reuse them by name. Rewriting a spell mid-encounter is allowed.

## Crafting and item system

Items are enums (introduced Act 3). `Weapon`, `Armor`, `Potion`, `Artifact`, `Scroll`. Each variant has a struct payload.

Crafting unlocks at Act 6 (the Forge). Recipes are iterator chains the player writes:

```rust
fn craft_potion(herbs: Vec<Herb>) -> Result<Potion, CraftError> {
    herbs.into_iter()
        .filter(|h| h.purity > 0.5)
        .map(|h| h.distill())
        .reduce(Potion::combine)
        .ok_or(CraftError::NotEnoughHerbs)
}
```

Items have a `quality` field that reflects code style. Cleaner code → higher quality. Quality drives sale prices in town economy.

## Town building

Unlocks at Act 6, deepens through Act 10.

The town is a real `World` struct with `Vec<Building>`, `HashMap<ResourceType, u64>`, etc. Players construct buildings by writing constructor functions. Each building has:

- A `tick(dt: Duration)` method the player implements.
- Inputs (resources consumed per tick) and outputs (resources produced).
- A visual representation that updates each in-game day.

The economy loop runs the `tick` of every building once per simulated day. The player can speed up time, watch their town hum, and observe via in-game graphs (Grafana-style dashboards displayed as illuminated parchment) where bottlenecks are.

This is the pre-Factorio-endgame mode. The player's town is their codebase, made visible.

## Progression and stats

`PartyMember` has classic RPG stats: HP, Mana (used for spell base costs), Initiative, three element resistances. Stats grow on level-up. Level-up gates are tied to encountering and resolving curriculum concepts, not to grinding XP from random battles.

Equipment slots: weapon, armor, two accessories. Each accessory may grant a passive code-style bonus (e.g., "Pendant of Iteration: +10% elegance multiplier on functions using `iter()` chains").

## Save system

Save files are real serialised Rust structs (using `serde` + `bincode`). The Spellbook is a real `.rs` file. The player's town state is a serialised `World`. This is intentional: the save file format is itself a Rust learning artifact. Advanced players can inspect their saves in a hex editor or text editor.

**Auto-save** triggers on:
- Zone transition.
- Boss encounter start.
- Every 10 minutes during freeplay.

**Manual save** is allowed anywhere out of combat.

**Cloud save** via Steam Cloud / iCloud is post-MVP.

## Accessibility

- All puzzles have a "show me the solution" option after three failed attempts. Using it is logged but not penalised. Some players learn by reading.
- Colorblind palettes for the four element classes (Act 4+).
- Keyboard-only and gamepad-only control schemes both fully supported.
- A "story mode" toggle that increases the elegance-multiplier floor and lowers panic damage. Lets players who care more about the world than the code progress without grinding code style.

## The Cliphy hint system

**Cliphy the Imp** (an affectionate riff on Clippy the paperclip and `clippy` the linter) is an optional in-world advisor. When the player's code compiles but is non-idiomatic, Cliphy may pop up with a hint: "*Pssst — your `for i in 0..vec.len() { vec[i]... }` would be cleaner as `for x in vec.iter()`. Just saying.*" Players can mute Cliphy permanently in settings. Cliphy's hints are exactly the same as `cargo clippy` warnings, translated into in-character voice.

## Difficulty and accessibility settings

- **Strict Mode** (default): full code-style enforcement, panic damage, elegance scoring.
- **Story Mode**: as described above. Recommended for first-time programmers.
- **Veteran Mode** (post-game): increased enemy stats, no Cliphy, no test-case revelation between attempts. For players replaying after Act 10.

## What this game is *not*

- Not a code-along tutorial. The player is doing, not following.
- Not a sandbox. There is a story. The player is on a journey.
- Not a multiplayer game. Single-player + leaderboards is the ceiling.
- Not a procedural-generation roguelike (except the Temple, which is opt-in post-game).
- Not a typing test. Code length is irrelevant to scoring; correctness and idiom are everything.
