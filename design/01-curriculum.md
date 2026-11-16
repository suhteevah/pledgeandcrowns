# 01 — Curriculum

> "The audience for this game leaves it as a Rust developer. That's the contract." — design north star

## Design meta

- **Target campaign playtime:** 25–35 hours across Acts 1–10. (~3 hours/act average; some acts longer.)
- **Encounter density:** ~3–5 combat encounters per act, plus 1 boss. Side quests are optional and add 1–2 hours/act.
- **Save system:** Auto-save on zone transition + manual save anywhere out of combat. No save scumming during boss fights.
- **Difficulty curve:** Acts 1–3 are tutorial-firm. Acts 4–7 introduce real challenge. Acts 8–10 expect the player to be writing competent, idiomatic Rust unaided.
- **Code-on-screen rule:** every encounter has the player's code visible during execution. Watching their function be called by the engine is the core teaching loop.
- **Linting:** `clippy`-style hints surface as in-world advice from a sidekick NPC ("Cliphy, the helpful imp"). Optional, dismissible.

## Curriculum coverage matrix

This is the contract: every concept area in the table below is taught somewhere in Acts 1–10 (the Act 9 and Act 10 code-status notes record the substitutions). The Temple (Act 11) does not introduce new Rust content — it tests Rust mastery by contrast with HolyC.

**What is not taught (2026-11-15).** The game does not cover the whole Rust Book. Two Book topics have no mission and no tutorial section: **modules and visibility** (`mod`, `pub`, `use` paths, crates and packages; Book ch. 7; starters use `use std::...` lines but nothing teaches the module system) and **writing tests** (`#[test]`, `assert!`/`assert_eq!` in tests, `cargo test`; Book ch. 11). Every player program is a single file with `fn main`. They are cut from v1.0, not hidden: adding them (Act 3 and Act 5 fit) is a v1.1 option. Do not describe the game as covering "every concept in the Rust Book".

**How the compiler claims are proven.** Where the code-status notes below say an NPC's compiler claims "are compiled in `game/tests/mission_text.rs`": every error code named in any mission text or dialogue row is bound to a claim by a fast test that runs in `scripts/ci.ps1`. Compiling and running those claims is `#[ignore]`d (slow) and runs in `scripts/ci-slow.ps1`, which first requires rustc to be the server's pinned release; a release tag requires a recorded green `ci-slow` run.

| Concept area                     | Primary Act | Reinforced In |
|----------------------------------|-------------|---------------|
| Variables, mutability, types     | 1           | All           |
| Functions, control flow          | 1           | All           |
| Ownership                        | 2           | 3, 6, 8       |
| References & borrowing           | 2           | 4, 7, 8       |
| Lifetimes (basic)                | 2           | 4, 7          |
| Structs                          | 3           | All later     |
| Enums                            | 3           | 4, 5          |
| Pattern matching                 | 3           | 5, 8, 9       |
| Methods, `impl` blocks           | 3           | All later     |
| Traits, generics                 | 4           | 6, 7, 8       |
| Trait objects (`dyn`)            | 4           | 8             |
| Associated types                 | 4           | 8             |
| `Result`, `Option`, `?`          | 5           | 6, 7, 9, 10   |
| Custom error types               | 5           | 9             |
| Collections (`Vec`, `HashMap`)   | 6           | All later     |
| Iterators, closures              | 6           | 7, 10         |
| Threads, `Arc<Mutex<T>>`         | 7           | 10            |
| Channels (`mpsc`)                | 7           | —             |
| `async`/`await`, tokio basics    | 7           | 10            |
| `Box`, `Rc`, `Arc`, `Weak`       | 8           | 9             |
| `RefCell`, `Cell`                | 8           | —             |
| `unsafe`, raw pointers           | 9           | 10            |
| FFI (`extern "C"`)               | 9           | —             |
| Macros (`macro_rules!`)          | 9           | 10            |
| `Drop`, `Send`, `Sync`           | 9           | —             |
| Performance, profiling, SIMD     | 10          | —             |
| `no_std`, embedded               | 10          | —             |
| Ecosystem: cargo/clippy/docs.rs  | 10          | —             |

---

# Act 1 — The Hearthstone Village

**Concepts taught:** variables, mutability (`let` vs `let mut`), primitive types (i32, u64, bool, char, &str, String), arithmetic, `println!` and string formatting, simple functions with parameters and return values, `if`/`else`, `loop`/`while`/`for`, tuples, arrays.

**Story beat.** The player wakes in Hearthstone Village, a peaceful starter town. **Ferris** (small crab guide, recurring throughout the game) explains that the world's magic system has been corrupted — only those who can speak the Old Tongue (Rust) can wield it. The village elder asks the player to fetch firewood, count chickens, and ring the bell at sundown — each a coding exercise.

**Mechanic introduction: The Cast Editor.** First time the player attempts a magical action, the in-game code editor opens. They must complete a `fn cast()` stub. The editor highlights the line their code is currently executing as the spell visually fires. This is the core loop for the entire game.

**Code status (2026-11-15): the live line highlight is cut from v1.0.** The Cast Editor ships (syntax highlighting, compile, graded result with VERIFIED / NOT VERIFIED / PATTERN CHECK ONLY banners, compiler diagnostics), but it does not highlight the line being executed and no spell animation follows the code: player code runs on the server (or is pattern-checked offline), never step by step in the game. The core loop as built is write, compile, read the verdict.

**Encounters:**
1. **Firewood Quest:** complete a function `fn count_logs(pile: i32) -> i32` that returns logs + 1. Tutorial-level scaffolding.
2. **Bell Tower:** write a `for i in 0..3` loop that rings the bell three times.
3. **Chicken Coop:** use a `while` loop with mutable counter; introduces `let mut`.
4. **Bandit Sparring Dummy** (optional combat tutorial): write `fn attack() -> i32` returning damage. Pure throughput, no mechanics yet.

**Boss:** none. Act 1 ends when the player rings the bell at sundown, cueing Ferris's "you're ready to leave the village" speech.

**Acceptance criteria.** Player can read and write simple Rust functions with parameters and returns; uses `let mut` correctly; completes basic loops and conditionals.

**Estimated playtime:** 1.5–2 hours.

---

# Act 2 — The Borrow Checker's Bridge

**Concepts taught:** ownership (move semantics), references (`&T`, `&mut T`), the borrowing rules (one mutable XOR many immutable), basic lifetime annotations (`'a`), slices (`&[T]`, `&str`), the difference between `String` and `&str`.

**Story beat.** Outside Hearthstone, a single bridge crosses the Riftwater Gorge. At its center stands **The Borrow Checker** — a tall, robed figure made of glowing reference symbols. He does not let bad code pass. His dialogue is `rustc` error messages, rendered as in-world utterances ("This value does not live long enough — try again, traveller.") with a translation panel for newcomers.

**Mechanic introduction: The Checkpost.** The Borrow Checker rejects code that violates ownership rules. The player must rewrite their function to compile cleanly. Each rejected attempt has the Checker visibly point at the problem ("Here. Here you have moved what you intended to lend."). Players cannot brute-force past — the bridge is impassable until the code is correct.

**Encounters:**
1. **First Move:** the player must hand the Checker a `String` artifact, then immediately try to use the original. Checker explains move semantics. Player rewrites with `&` borrow.
2. **Two Travellers:** two NPCs both want to read a scroll. Player must give out two `&` borrows. Easy.
3. **The Mutable Quill:** an NPC wants to write on the scroll. Player must learn `&mut`, and that they can't have outstanding `&` references at the same time.
4. **The Lifetime Lantern:** the player crafts a function that returns a slice of an input. The Checker requires explicit `'a` annotations on the function signature. This is the hardest puzzle in Act 2.

**Boss: The Borrow Checker.** Not a damage fight. A four-stage code puzzle. Each stage gives the player a broken function and a desired behaviour. The player must produce code that compiles AND passes the test cases. Failed compiles deal 0 damage and waste a turn; if the player runs out of turns (a generous budget), the Checker says "Come back when you have studied," and resets the encounter — no permadeath.

When the player finally passes, the Checker bows: "You may cross. The Old Tongue speaks through you." He hands them the **Borrowstone**, a key item that unlocks `&mut` self-methods in subsequent gameplay.

**Acceptance criteria.** Player can confidently use `&` and `&mut`, understands one-mutable-XOR-many-immutable, can read a basic lifetime annotation and explain why it's there.

**Estimated playtime:** 2–3 hours. This is the hardest tutorial act, by design — Acts 1–2 together are the MVP vertical slice.

**Code status (2026-11-14).** The boss ships as a *trial* (finish plan D1: no combat, turns or key items): three staged missions owned by the Borrow Checker, at the end of Act 2, after his prologue mission `borrow_preview`. Mechanism: `game/src/plugins/trial.rs`. Stages: `bridge_toll` (First Move: a `String` passed by value is moved, E0382; borrow with `&str`), `bridge_quill` (Two Travellers + Mutable Quill: many `&` or one `&mut`, E0502), `bridge_lantern` (Lifetime Lantern: `'a` on the input the answer borrows from, E0106; tying both inputs fails with E0597). Every compiler error he names is compiled in `game/tests/mission_text.rs`. The trial is the last content of the free demo (24 missions).

---

# Act 3 — The Guildhall Quarter

**Concepts taught:** structs (named-field, tuple-struct, unit-struct), enums and their variants, exhaustive `match`, `if let`, `while let`, methods (`impl` blocks), `Self`/`self`, associated functions (`Self::new`), introduction to traits via `Display`/`Debug` derive.

**Story beat.** Beyond the bridge lies the city of **Guildhall**. The player can join one of four guilds — **Warriors, Mages, Rangers, Artificers** — each defined by a different `Class` enum variant. The guildmaster gives a tour: every NPC, every item, every shop is now a struct or enum.

**Mechanic introduction: The Party Tab.** The player can now build a party. Each `PartyMember` is a struct with stats, equipment (an enum), and a class. The Party Tab UI shows the actual `struct` the engine is using.

**Encounters:**
1. **Guild Initiation:** define your character's `Class` enum and `Stats` struct.
2. **The Equipment Smith:** build an `Item` enum with variants `Weapon { damage }`, `Armor { defense }`, `Potion { heal }`. Use exhaustive `match` to handle equip logic.
3. **The Mistmonster:** an enemy whose AI requires the player to write `match enemy.state { Hidden => ..., Approaching => ..., Attacking => ... }`. Non-exhaustive matches fail to compile.

**Boss: The Variant Wraith.** A shapeshifter that cycles between three forms (`Wraith::Solid`, `Wraith::Mist`, `Wraith::Shadow`). The player's combat function must handle all three exhaustively or the encounter loops forever. Adding a new form mid-fight (`Wraith::Possessed`) tests that the player understands non-exhaustive `match` warnings — they must ship the new arm.

**Acceptance criteria.** Player defines structs and enums fluently, writes exhaustive matches, derives `Debug`, calls methods on instances, distinguishes `Self::new(...)` (associated fn) from `self.method()` (instance method).

**Estimated playtime:** 3 hours.

**Code status (2026-06-18).** The Act 3 *concepts* the prelude skipped now ship as 6 missions in the mission-tutorial loop (the RPG layer — party tab, combat, the Variant Wraith boss — is still future work): `impl_method` (The Guildmaster), `assoc_new` (The Recruiter), `if_let` (The Locksmith), `while_let` (The Porter), `tuple_struct` (The Surveyor), `enum_data_match` (The Armorer). Basic struct/enum/match/derive were already covered by the prelude (Herald/Heraldic Sage/Oracle/Chronicler). Design: `docs/superpowers/specs/2026-06-18-act3-guildhall-missions-design.md`.

**Code status (2026-11-15).** The Variant Wraith ships as a *trial* (the Bridge's mechanism, `game/src/plugins/trial.rs`): three staged missions owned by the Guildmaster, after `enum_data_match`; `impl_method` stays his prologue. Stages: `wraith_forms` (a `_` arm answers the Shadow wrongly; name every variant, and dropping `_` without naming Shadow is E0004), `wraith_essence` (struct-like variants `Solid { armor }`, `Mist { density }` bound in the pattern; a bare `Wraith::Solid` pattern is E0533), `wraith_possessed` (a fourth form `Possessed { will }` makes the match non-exhaustive, E0004; its damage depends on `will`, so a blanket `_` arm cannot pass). Every compiler error he names is compiled in `game/tests/mission_text.rs`.

---

# Act 4 — The Trait Mage's Tower

**Concepts taught:** generic functions and structs, trait definitions, trait bounds (`<T: Foo>`, `where` clauses), default methods on traits, blanket impls, supertraits, trait objects (`dyn Trait`, `Box<dyn Trait>`), associated types (`type Output`), the `Iterator` trait teaser.

**Story beat.** **Vexis the Trait Mage** lives in a tower whose floors only respond to spells of specific element types. The tower is a generic data structure: each floor is `Floor<E: Element>`. The player learns to write spells that work over any `Element`, then learns to put differently-typed spells into the same `Vec<Box<dyn Spell>>`.

**Mechanic introduction: The Grimoire.** A code-editing UI specifically for trait definitions. The player adds new traits and impls; the world responds by gaining new options.

**Encounters:**
1. **The Gauntlet of Elements:** define `trait Element` with associated method `name() -> &'static str`. Implement for `Fire`, `Water`, `Earth`, `Air`.
2. **Generic Wandcraft:** write `fn cast<E: Element>(wand: &Wand<E>) -> Spell<E>`. Use it with all four elements.
3. **The Polyglot Familiar:** a pet familiar that can hold multiple element types. Player must use `Box<dyn Element>` in a `Vec`.
4. **Default Method Workshop:** add a default method to the `Element` trait. Override it for one element only. Test that overriding works.

**Boss: The Chimera of Vexis.** Three heads, each a different element. The player's combat function takes `enemies: &[Box<dyn Enemy>]` and must dispatch correctly using trait methods. Solving this with `match` on concrete types is the wrong approach — the boss reveals a fourth head mid-fight (`Plasma`), and only trait-object code adapts gracefully.

**Acceptance criteria.** Player writes generic functions with bounds, defines and implements traits, knows when to use generics vs. trait objects, understands the static-vs-dynamic dispatch tradeoff.

**Estimated playtime:** 3.5 hours.

**Code status (2026-06-18).** The Act 4 *concepts* ship as 6 missions in the mission-tutorial loop (the RPG layer — Vexis's tower, the Grimoire, the Chimera boss — is future work): `trait_def` (Vexis the Archmage), `generic_fn` (The Wandwright), `generic_struct` (The Conjurer), `dyn_trait` (The Familiar), `lifetimes` (The Lanternkeeper — the doc's Lifetime Lantern, deferred from Act 2), `assoc_type` (The Loremaster). Default methods / blanket impls / supertraits / the Iterator impl are taught in tutorials but not their own missions (hard to pattern-grade cleanly). Design: `docs/superpowers/specs/2026-06-18-act4-trait-tower-missions-design.md`.

**Code status (2026-11-15).** The Chimera of Vexis ships as a trial owned by Vexis, after `assoc_type`; `trait_def` stays his prologue. Stages: `chimera_heads` (implement `trait Enemy` for the Frost and Storm heads; a head without the impl has no such method, E0599), `chimera_bound` (a generic `volley<T: Enemy>`; without the bound `head.name()` is E0599; the server checker calls it with a head type of its own), `chimera_plasma` (`fn battle(heads: &[Box<dyn Enemy>]) -> u32` replaces a `match` on a `Head` enum; the starter's `battle(&heads)` is E0308 once the heads arrive as trait objects with the fourth head, `Plasma`; the checker sends a head it never saw). Every compiler error he names is compiled in `game/tests/mission_text.rs`.

---

# Act 5 — The Tavern of Tribulations

**Concepts taught:** `Option<T>` and its methods, `Result<T, E>` and its methods, the `?` operator, custom error types via `thiserror`, the `From` trait for error conversion, `panic!` vs recoverable errors, `unwrap`/`expect`/`unwrap_or`/`unwrap_or_else`, error-chain propagation across functions.

**Story beat.** **The Crooked Cask** is a tavern at a crossroads where every action has a chance to fail. The barkeep, **Old Maybe**, speaks in `Option`. Every quest-giver returns `Result`. The player must learn to handle failure gracefully — and learn that `unwrap()` in production code is how heroes die.

**Mechanic introduction: The Failure Log.** A new HUD element. Every time the player calls `.unwrap()` on something that turns out to be `None` or `Err`, the game records it. End-of-act tally; affects the act's final reward. The game doesn't lecture; it just keeps score.

**Encounters:**
1. **The Lost Coin:** find an NPC's coin. Function returns `Option<Coin>`. Player must handle `None` without panicking.
2. **The Cursed Order:** a tavern order that may fail at any of three steps (find ingredient, mix, deliver). Cascading `Result<_, OrderError>` with `?`. Custom error type required.
3. **The Stewing Pot:** parse user input from an NPC into an integer (`str::parse::<i32>()`). Handle `ParseIntError` via `From` impl into your custom error.
4. **The Ledger:** an account-keeping minigame where the player must aggregate `Vec<Result<Transaction, _>>` into `Result<Total, _>` — first exposure to `collect::<Result<_, _>>()`.

**Boss: The Tavernkeeper's Wager.** A multi-step quest in a single function. The player writes `fn quest() -> Result<Reward, QuestError>`. Each step uses `?`. Errors must convert via `From` into `QuestError`. The Tavernkeeper grades not just whether it works but whether the player used `?` (vs. nested `match`). Style points reduce `panic!` damage in later acts.

**Acceptance criteria.** Player handles `Option` and `Result` idiomatically, uses `?` correctly, defines a custom error type with `thiserror`, never reaches for `unwrap()` outside of tests.

**Estimated playtime:** 3 hours.

**Code status (2026-06-18).** The Act 5 *concepts* the prelude skipped ship as 6 missions (the prelude already had `match_option`/`option_unwrap_or`/`result_question_mark`): `result_match` (The Barkeep), `custom_error` (The Bouncer), `from_error` (The Interpreter), `option_map` (The Mixologist), `and_then` (The Tabkeeper), `unwrap_or_else` (The Cellarer). Custom errors are taught as a hand-written `enum` (the grading sandbox is dependency-free); `thiserror` is mentioned in the tutorial as the production shortcut. Design: `docs/superpowers/specs/2026-06-18-act5-tavern-missions-design.md`.

**Code status (2026-11-15).** The Tavernkeeper's Wager ships as a trial owned by the Barkeep (no separate Tavernkeeper NPC exists; the Barkeep keeps the Crooked Cask), after `unwrap_or_else`; `result_match` stays his prologue. Stages: `wager_stake` (`?` in a function returning `u32` is E0277; return `Result<u32, ParseIntError>`), `wager_from` (`impl From<ParseIntError>` and `impl From<Short>` for `WagerError`, so `?` converts; without them E0277, "`?` couldn't convert the error"), `wager_quest` (`fn quest(map, key) -> Result<Reward, QuestError>` chaining three steps with `?`; the starter's `.unwrap()` panics on the first failed step). The server rejects `match`, `unwrap`, `expect` and `panic!` in the last stage (`MissionCheck::forbids`) and checks the step order. The style-points / `panic!` damage idea is not implemented (no combat layer). Every compiler error he names is compiled in `game/tests/mission_text.rs`.

---

# Act 6 — The Iterator Forge

**Concepts taught:** `Vec<T>`, `HashMap<K, V>`, `BTreeMap`, `HashSet`, the `Iterator` trait in depth, `map`/`filter`/`fold`/`reduce`/`collect`/`flat_map`/`zip`/`chain`/`take`/`skip`/`enumerate`, closures (`Fn`, `FnMut`, `FnOnce`), `move` closures, lazy evaluation, when iteration beats indexing.

**Story beat.** The **Iron Vale** holds the **Iterator Forge** — a vast factory where weapons, potions, and spells are produced by streams of raw materials flowing through transformation pipelines. Every NPC speaks in iterator chains. Town-building begins here in earnest: the player can now set up automated production lines.

**Mechanic introduction: Town Pipelines.** A persistent, between-acts feature. The player designs supply chains as iterator pipelines. `inputs.iter().filter(quality_check).map(refine).collect()`. The town's economy ticks forward each in-game day based on what compiles and runs.

**Encounters:**
1. **The Sorting Belt:** sort `Vec<Ore>` by purity using `sort_by_key`. Filter junk with `.filter()`.
2. **The Dual-Furnace:** two streams of ingredients. `zip` them. `map` to a recipe. `collect()` into a `Vec<Potion>`.
3. **The Census Clerk:** count NPC types in a `Vec<Citizen>` by class using `HashMap<Class, u32>`. First exposure to `entry().or_insert()`.
4. **The Closure Workshop:** the player must write a `move` closure that captures local state by value. The Forge's apprentices explain the difference between `Fn`, `FnMut`, `FnOnce` through trial and error.

**Boss: The Forge-Golem.** A massive automaton built of iterator chains. To defeat it, the player must construct a single chain that takes the golem's `parts: Vec<Part>`, filters out armored parts, maps each to a weakness, folds into a total damage value, and returns it from a function. No intermediate variables allowed (style enforced by Cliphy the imp). Pure iterator-chain mastery.

**Acceptance criteria.** Player chooses the right collection, writes idiomatic iterator chains over indexed loops, uses closures fluently, understands move semantics in closures.

**Estimated playtime:** 3.5 hours.

**Code status (2026-06-18).** The Act 6 *concepts* the prelude skipped ship as 6 missions (the prelude already had `vec_iter`/`iter_map_collect`/`closure_basic`): `hashmap_basic` (The Keymaster), `iter_filter` (The Sifter), `iter_fold` (The Smelter), `iter_enumerate` (The Tallywright), `iter_zip` (The Riveter), `closure_move` (The Bondsmith). Design: `docs/superpowers/specs/2026-06-18-act6-iterator-forge-missions-design.md`.

**Code status (2026-11-15).** The Forge-Golem ships as a trial owned by the Keymaster, after `closure_move`; `hashmap_basic` stays his prologue. Stages over a `Vec<Part>` (`name`, `armoured`, `weakness`): `golem_parts` (`.map` before `.filter` leaves only labels, so `.filter(|p| !p.armoured)` is E0609; filter first), `golem_sum` (`let total = ...sum();` is E0283, "type annotations needed"; name the type or use `fold`), `golem_core` (`total_damage(parts, blow)` as one chain, filter, map, fold; the starter's loop `break`s at the first armoured part). The server rejects `for`, `while`, `loop` and `mut` in the last stage (`MissionCheck::forbids`) and needs a `.fold(`/`.sum`. Cliphy the imp and the "no intermediate variables" rule are not implemented. Every compiler error he names is compiled in `game/tests/mission_text.rs`.

---

# Act 7 — The Concurrent Coast

**Concepts taught:** `std::thread::spawn`, `JoinHandle`, `Arc<T>` for shared ownership, `Mutex<T>` and `RwLock<T>` for shared mutation, `mpsc` channels (sync, async, multi-producer), `Send` and `Sync` markers (introduced lightly), `async fn` and `.await`, `tokio::main`, `tokio::spawn`, `tokio::select!`, races and deadlocks (and how to avoid them).

**Story beat.** **Briney Cove**, a coastal town with a dozen harbors. The fish run on multiple piers simultaneously, and the player must control multiple party members in parallel. The town's lighthouse is failing because its keepers are deadlocking each other. The player must fix it.

**Mechanic introduction: Party Concurrency.** Up to four party members can act simultaneously, each driven by an `async` task or thread. The player writes their orchestration code.

**Encounters:**
1. **The Two-Net Cast:** spawn two threads to net fish on different piers. Join them. Sum results.
2. **The Crab Auctioneer:** a crab market with shared price data. `Arc<Mutex<Price>>`. Multiple bidders. First exposure to deadlock (and how to dodge it via lock scope).
3. **The Lighthouse Crisis:** the keepers hold locks in opposite orders. The player must refactor to avoid the deadlock. Failure makes the lighthouse remain dark and ships crash visibly. (The game lets the player observe the failure before fixing it.)
4. **The Async Tavern:** introduces `async fn`. NPCs return `Future`s. The player awaits them, learns `tokio::spawn`, and writes a small `select!` between two NPCs.

**Boss: The Tide Hydra.** Multiple heads regrowing in real-time. The player must spawn one task per head, attack them concurrently, and use a channel to coordinate when all heads are down so the body can be struck. Sequential code cannot win; the hydra regrows faster than one task can kill.

**Acceptance criteria.** Player spawns and joins threads, picks `Mutex` vs `RwLock` correctly, writes basic async functions and runs them under tokio, recognises and avoids common deadlock patterns.

**Estimated playtime:** 4 hours. This is one of the meatiest acts.

**Code status (2026-06-18).** The Act 7 *concepts* ship as 6 missions: `thread_spawn` (The Dockmaster), `arc_mutex` (The Lighthouse Keeper), `mpsc_channel` (The Signaler), `atomic` (The Tidewatch), `thread_scope` (The Harbormaster), `async_fn` (The Tideforecaster). *(Corrected 2026-11-15: the server's sandbox has no threads (wasm32-wasip1), so `thread_spawn` and `thread_scope` are static-only there (`CompileOnly`: compiled and token-checked, never run). `arc_mutex`, `mpsc_channel` and `atomic` are behaviour-checked. The original sentence follows.)* ~~The std-concurrency five compile + grade fully on the host sandbox~~; **`async_fn` is compile-only** — `async fn`/`.await` type-check and are token-graded, but actually running them needs a runtime (`tokio`) the dependency-free sandbox can't load (the tutorial says so). Design: `docs/superpowers/specs/2026-06-18-act7-concurrent-coast-missions-design.md`.

**Code status (2026-11-15).** The Tide Hydra ships as a trial owned by the Dockmaster, after `async_fn`; `thread_spawn` stays his prologue. Stages: `hydra_senders` (each head reports on its own `tx.clone()`; passing the one `Sender` into a loop is E0382; one thread, so the server calls `hunt` on heads of its own), `hydra_threads` (one `thread::spawn` per head; without `move` the closure borrows `hp`, E0373; the reports are sorted because nothing fixes the order threads finish in), `hydra_body` (count `Report::Down` until every head is down before striking; the starter strikes at the first `Down`). The server has no threads (wasm32-wasip1): `hydra_threads` is compile-only there (static checks for `spawn(`, a `move` closure and a sort), and `hydra_body` is checked by feeding `await_heads` reports from one thread without running the player's `main`. Both thread programs' outputs are proven natively (`game/tests/mission_text.rs`, run 21 times each). Regrowing heads are not implemented (no real-time layer). Every compiler error he names is compiled in `game/tests/mission_text.rs`.

---

# Act 8 — The Vault of Pointers

**Concepts taught:** `Box<T>` and heap allocation, `Rc<T>` for shared ownership in single-threaded code, `Arc<T>` (revisit, cross-thread), `RefCell<T>` for interior mutability, `Cell<T>` for `Copy` types, `Weak<T>` for breaking cycles, when each is appropriate, basic recursive data structures (singly- and doubly-linked lists, trees).

**Story beat.** Beneath Briney Cove lies **The Vault** — a treasury of ancient artifacts, each of which grants power but demands the right form of grasp. The Vault Warden tests applicants: only those who can wield each pointer type without leaking will ascend.

**Mechanic introduction: The Artifact Workshop.** Players forge magical items by writing constructors. The artifact's behaviour at runtime depends on the smart-pointer choice. A misused `Rc` produces a visible cycle; the artifact corrupts and is lost.

**Encounters:**
1. **The Box of Heaps:** a recursive `enum List` requires `Box<List>` to compile. The player learns the size-known-at-compile-time rule.
2. **The Shared Lantern:** an artifact lit by multiple party members simultaneously requires `Rc<Lantern>`. (Threaded version forces `Arc`.)
3. **The Mutating Mirror:** a struct field that must be mutable through a `&self` method requires `RefCell`. The player observes a runtime panic from violating borrow rules and learns RefCell is not a free lunch.
4. **The Family Tree:** a doubly-linked list of ancestors. Naive `Rc<Node>` cycles leak; the player must use `Weak<Node>` for parent links.

**Boss: The Memory Warden.** A four-phase fight where each phase demands a different pointer type. The final phase is a cycle test: the Warden constructs a circular debt graph (he owes you, you owe a third party, third party owes him); the player's resolution function must traverse it without infinite recursion using `Weak`.

**Acceptance criteria.** Player picks `Box`/`Rc`/`Arc`/`RefCell` correctly per use case; constructs recursive data without leaks; understands that `RefCell` borrow violations panic at runtime.

**Estimated playtime:** 4 hours.

**Code status (2026-06-19).** The Act 8 *concepts* ship as 6 missions (`Arc` was already covered by Act 7's `arc_mutex`): `box_basic` (The Vaultwright), `rc_basic` (The Sharekeeper), `refcell` (The Warden), `cell` (The Swapwarden), `rc_refcell` (The Strongbox), `weak_ref` (The Ghostkeeper). Design: `docs/superpowers/specs/2026-06-19-act8-vault-pointers-missions-design.md`.

**Code status (2026-11-15).** The Memory Warden ships as a trial owned by the Vaultwright, after `weak_ref`, and ends the registry; `box_basic` stays his prologue. Three phases, not four: `warden_box` (a ledger `enum` holding itself inline is E0072; `Entry(u32, Box<Ledger>)`), `warden_shared` (two debtors share a vault; assigning through an `Rc` is E0594; `Rc<RefCell<u32>>`, and the hint's held `borrow()` panics with "RefCell already borrowed"), `warden_weak` (the circular debt of this section, the Warden owes you, you owe the teller, the teller owes the Warden: with `Rc` links `strong_count` is 2 and the ring outlives `main`'s handles; with `Weak` links it is 1 and the probe finds nothing; `circle` walks the ring with `upgrade()` and stops at the start). All three are behaviour-checked by the server. Every compiler error he names is compiled in `game/tests/mission_text.rs`.

---

# Act 9 — The Forbidden Library

**Concepts taught:** `unsafe` blocks (when and why), raw pointers (`*const T`, `*mut T`), dereference rules, `extern "C"` and FFI, calling C from Rust and vice versa, declarative macros (`macro_rules!`), brief intro to procedural macros (just enough to read one), `Drop`, manual `Send`/`Sync` impls, pointer arithmetic, `MaybeUninit`.

**Story beat.** The **Forbidden Library** is sealed. Inside, the laws of memory safety can be temporarily lifted — but the cost is a Sanity meter. The Librarian, an ancient sage, warns that everything inside is real. Errors here corrupt the player's save file (purely cosmetic — the world map is replaced with `<corrupted>` glyphs until repaired). This is the act where the game stops holding hands.

**Mechanic introduction: The Sanity Meter.** Each `unsafe` block costs Sanity. Sanity regenerates over time. Spending it grants temporary access to powerful spells. Players who try to brute-force everything in `unsafe` end the act with corruption effects on their character.

**Encounters:**
1. **The Pointer's Edge:** convert a `&T` to `*const T`, dereference it inside `unsafe`. Explain why this is fine here and not always.
2. **The Foreign Tongue:** call a C function via `extern "C"`. The game ships a tiny C library (`libcrown.so`) with a single function for the player to invoke.
3. **The Macro Monk:** define a `macro_rules!` that abbreviates a common spell. Use it three times in subsequent encounters.
4. **The Drop Ritual:** implement `Drop` for a custom resource type that must release a "soul" on death. Observe order-of-drop semantics.

**Boss: The Librarian.** Unkillable by normal means. The only way to defeat him is to define a custom `macro_rules!` that, when invoked, expands to the exact incantation he requires. The macro definition is part of the boss puzzle — the player must read his "demand" (a deliberately verbose expression) and write a macro that produces it.

**Acceptance criteria.** Player writes minimal correct `unsafe` blocks, calls C from Rust, defines and invokes a `macro_rules!`, understands when each tool is justified.

**Estimated playtime:** 4 hours.

**Code status (2026-11-15).** Act 9 ships as six missions and the Librarian's trial, after the Memory Warden (registry 78 → 87; design: `docs/superpowers/specs/2026-11-14-act9-forbidden-library.md`). Missions: `raw_pointer` (The Shelfmark: `*mut`/`*const` from references, deref inside `unsafe`; the starter's bare deref is E0133), `extern_c` (The Envoy: `#[repr(C)]`, `#[unsafe(no_mangle)] pub extern "C" fn`, called back through an `unsafe extern "C"` block; E0133 at the call), `macro_rules` (The Macro Monk: a `chant!` macro over `$( ... ),+`), `drop_trait` (The Sexton: `impl Drop`, drop order, `drop(x)`, `mem::forget`; `x.drop()` is E0040), `send_sync` (The Courier: `unsafe impl Send`/`Sync` for a raw-pointer-owning `Tome`, checked through `T: Send`/`T: Sync` bounds; E0277), `maybe_uninit` (The Bookbinder: fill a `MaybeUninit<[u32; 4]>` through a raw pointer, then `assume_init`; E0381). Boss: the Librarian, a trial with no prologue (his roster mission is stage 1): `librarian_index` (a safe `fetch` over an `unsafe fn` with a `# Safety` contract, E0133), `librarian_demand` (the curriculum's boss puzzle: a recursive `macro_rules!` with internal `@acc` rules that expands to his exact demand), `librarian_seal` (a `Codex` owning a `Box<[u32]>` through a raw pointer, rebuilt with `Box::from_raw` in `Drop`). Every compiler error and run-time behaviour the act's text claims is compiled or run in `game/tests/mission_text.rs`.

Substitutions (the sandbox is one std-only file built for `wasm32-wasip1`): no C library ships (`#[link]` is refused, there is no C toolchain), so `extern_c` exports a C-ABI function and calls it back through a foreign declaration in the same program; procedural macros are named in the `macro_rules` tutorial, not built; `Send`/`Sync` are checked at compile time (no threads on wasip1); `static mut` and unions are named, not exercised. Not built (finish plan D1): the Sanity meter, save corruption, corruption effects, the Librarian's lectern and arena, his intro/defeat frames. The Sealed Stacks landmark (REF-95) stands behind the Librarian (approved and wired 2026-11-16).

Server grading: `raw_pointer`, `macro_rules`, `drop_trait`, `maybe_uninit` and `librarian_demand` are checked by their stdout plus the construct's tokens. `extern_c`, `send_sync`, `librarian_index` and `librarian_seal` are harness checks that call the player's items with inputs of the checker's own, in `HarnessMode::UnsafeLateNonce` (`unsafe` allowed; the result key reaches the checker only after the player's code has finished). That mode has a documented forgery limit (`compile-api/HARNESS-NOTES.md`), so the game labels those passes NOT VERIFIED (they still count). No Miri run backs the `unsafe` solutions: Miri is not installed; the spec's per-mission soundness arguments stand in for it.

---

# Act 10 — The Throne of the Compiler

**Concepts taught:** profiling with `cargo flamegraph`, criterion benchmarks, allocation reduction (preallocation, `SmallVec`, `Cow`), inlining hints, SIMD intro (via `std::simd`), `no_std` and `alloc`-only environments, embedded Rust teaser, the broader ecosystem (cargo, rustup, clippy, rustfmt, docs.rs, semver, crates.io).

**Story beat.** The world is collapsing under its own complexity. The Compiler — a silver-clad colossus at the throne — demands proof that the player's town and party are not just *correct* but *fast*. The Borrow Checker, the Trait Mage, the Vault Warden, the Tavernkeeper, every NPC the player has met — they all ascend to the Throne to watch the trial.

**Mechanic introduction: The Optimization Endgame.** Every system the player has built across Acts 1–9 is now scored. Town pipelines (Act 6), party concurrency (Act 7), artifact forging (Act 8), unsafe rituals (Act 9) all generate numbers: throughput, allocations per tick, p99 latency. The Compiler's HP scales to match the player's last known optimization score. The player can win only by improving.

**Encounters:**
1. **The Flame Reading:** the player runs a profiler on their town's economy loop. Reads the flamegraph. Identifies the hottest function. Optimizes it.
2. **The Allocation Audit:** a town pipeline allocates per-iteration. Refactor to preallocate. Observe the throughput jump in real-time.
3. **The SIMD Shrine:** a vector-math kernel. Rewrite with `std::simd`. Observe the speedup.
4. **The `no_std` Outpost:** a small embedded subzone that cannot use `std`. The player ports a function. Tests run on a simulated MCU.

**Boss: The Compiler.** A multi-stage optimization battle. Each stage:
- The Compiler casts a benchmark.
- The player has 2 minutes to optimize their solution.
- Whoever's score wins the round takes a stage.
- Best of five.

**Final reward:** the player ascends to the Throne and is told the truth — the entire world is a Rust program they have been writing all along. Credits roll over a `cargo doc` rendering of their final codebase. The player can now enter **The Temple** (Act 11).

**Acceptance criteria.** Player reads flamegraphs, eliminates allocations from hot paths, recognises when SIMD is appropriate, can navigate `cargo`, `clippy`, `rustfmt`, `docs.rs`, and the wider ecosystem unaided.

**Estimated playtime:** 4–5 hours.

**Code status (2026-11-15).** Act 10 ships as six missions and the Compiler's trial, after the Librarian (registry 87 → 96, the last act; design: `docs/superpowers/specs/2026-11-14-act10-throne.md`). Missions: `prealloc` (The Assayer: a counting `#[global_allocator]` measures a 1000-coin `Vec`; `Vec::with_capacity` makes it exactly one allocation), `cow_borrow` (The Copyist: return `Cow<'_, str>`, borrowing titles that need no change), `const_eval` (The Astronomer: a `const fn` table computed at compile time and checked with `const _: () = assert!(..)`; the starter is E0015, a false assert is E0080), `chunks_exact` (The Choirmaster: four lanes with `chunks_exact(4)` and the `remainder()`), `no_std` (The Hermit: a `#![no_std]` crate formats into a 32-byte stack buffer through `core::fmt::Write`; the starter is E0425), `semver_caret` (The Registrar: Cargo's caret rule for 1.x, 0.x and 0.0.x). Boss: the Compiler, a trial with no prologue (his roster mission is stage 1): `compiler_alloc` (a proclamation built in one allocation), `compiler_lifetime` (a lifetime on `name` only; E0106, and rustc's help followed literally is E0597 in `main`), `compiler_bound` (`PartialOrd`, not `Ord`; E0369, and `Ord` refuses floats with E0277). Clearing `compiler_bound` clears every mission and shows the epilogue (with a short credits block). Every compiler error and run-time behaviour the act's text claims is compiled or run in `game/tests/mission_text.rs`; the Compiler's dialogue quotes no compiler output (the editor's result pane shows the player's real stderr).

Substitutions (no grader can time code, and the sandbox is one std-only file built for `wasm32-wasip1`): no mission grades speed. Profiling (`cargo flamegraph`, `perf`, DHAT), criterion and `smallvec` are named in tutorials; the "allocation audit" and the Compiler's benchmark are deterministic allocation counts. `#[inline]` is named only. `std::simd` is unstable (E0658 on stable, asserted) and the server does not enable wasm SIMD, so the SIMD Shrine is `chunks_exact`, the lane-shaped loop, graded on its answer. A `no_std` binary does not link for `wasm32-wasip1`, so the crate root is `#![no_std]` and `main` alone reaches std, renamed `host`; the firmware module also builds as a real `#![no_std]` library for `thumbv7em-none-eabihf` and `riscv32imac-unknown-none-elf` (spec record, not a test). cargo, rustup, clippy, rustfmt, docs.rs and crates.io are tutorial text; the caret rule is the graded ecosystem lesson. Not built (finish plan D1): the Optimization Endgame scoring, the Compiler's HP, a timed best-of-five, the watching cast on the Throne plaza, the Compiler's intro/defeat frames, credits over a `cargo doc` rendering, a "your programs" view.

Server grading: all nine are checked by their stdout plus the construct's tokens, in the plain build (no harness). `prealloc` and `compiler_alloc` contain the player's own counting `GlobalAlloc`; the plain build has never carried `forbid(unsafe_code)`, so no harness mode is involved, and their passes are ordinary verified output checks. The pinned `allocations: 1` is the same natively and in the sandbox, and is std's documented guarantee for an exact `with_capacity`.

---

# Act 11 — The Temple

Specified in `08-temple-appendix.md`. Brief: HolyC roguelike post-game, no Rust safety rules, everything compiles, segfaults are part of the loop.

---

## Pacing summary

| Act    | Title                       | Hours | Key concept(s)                |
|--------|-----------------------------|-------|-------------------------------|
| 1      | Hearthstone Village         | 1.5–2 | Variables, functions, control |
| 2      | Borrow Checker's Bridge     | 2–3   | **Ownership, borrowing**      |
| 3      | Guildhall Quarter           | 3     | Structs, enums, match         |
| 4      | Trait Mage's Tower          | 3.5   | Traits, generics, dyn         |
| 5      | Tavern of Tribulations      | 3     | Result, Option, ?             |
| 6      | Iterator Forge              | 3.5   | Collections, iterators        |
| 7      | Concurrent Coast            | 4     | Threads, async                |
| 8      | Vault of Pointers           | 4     | Smart pointers                |
| 9      | Forbidden Library           | 4     | unsafe, FFI, macros           |
| 10     | Throne of the Compiler      | 4–5   | Optimization, ecosystem       |
| **Σ**  |                             | **32–37** |                            |
| 11     | The Temple (replayable)     | ∞     | HolyC contrast                |
