// SPDX-License-Identifier: MIT
//! Single source of truth for every asset path the game loads.
//! Tests read this module to verify the files exist on disk; runtime
//! plugins read it so a renamed file fails compile, not at startup.

// Migrated 2026-05-02 from flat `sprites/X.png` to the
// `sprites/{category}/{name}_{frame}.png` convention from CLAUDE.md.
// All NPCs (including Ferris and the Borrow Checker) live under
// `sprites/npc/`; the player sprite gets its own category dir so
// future protagonist animation frames have a home.
pub const SPRITE_PLAYER: &str = "sprites/player/player_idle_0.png";
pub const SPRITE_FERRIS: &str = "sprites/npc/ferris_idle_0.png";
pub const SPRITE_BORROW_CHECKER: &str = "sprites/npc/borrow_checker_idle_0.png";
pub const SPRITE_TILES_VILLAGE: &str = "sprites/tiles_village.png";
pub const SPRITE_TITLE: &str = "sprites/title.png";

// NPC batch 1 (2026-05-02) — Smith, Cartographer, Trait Mage, Bellringer, Cooper.
// Sources: design/art/refs/ref-11..15-*.jsx, rendered via scripts/render-refs.py.
pub const SPRITE_SMITH: &str = "sprites/npc/smith_idle_0.png";
pub const SPRITE_CARTOGRAPHER: &str = "sprites/npc/cartographer_idle_0.png";
pub const SPRITE_TRAIT_MAGE: &str = "sprites/npc/trait_mage_idle_0.png";
pub const SPRITE_BELLRINGER: &str = "sprites/npc/bellringer_idle_0.png";
pub const SPRITE_COOPER: &str = "sprites/npc/cooper_idle_0.png";

// NPC batch 2 (2026-05-02) — Oracle, Herald, Twin, Tinker, Heraldic Sage.
// Sources: design/art/refs/ref-16..20-*.jsx, rendered via scripts/render-refs.py.
pub const SPRITE_ORACLE: &str = "sprites/npc/oracle_idle_0.png";
pub const SPRITE_HERALD: &str = "sprites/npc/herald_idle_0.png";
pub const SPRITE_TWIN: &str = "sprites/npc/twin_idle_0.png";
pub const SPRITE_TINKER: &str = "sprites/npc/tinker_idle_0.png";
pub const SPRITE_HERALDIC_SAGE: &str = "sprites/npc/heraldic_sage_idle_0.png";

// NPC batch 3 (2026-05-02) — Forgewright, Linguist, Pilgrim, Drillmaster, Reckoner.
// Sources: design/art/refs/ref-21..25-*.jsx; PNGs rendered direct from JSX
// grids by the art-lead's helper (the JSX harness is the canonical source).
pub const SPRITE_FORGEWRIGHT: &str = "sprites/npc/forgewright_idle_0.png";
pub const SPRITE_LINGUIST: &str = "sprites/npc/linguist_idle_0.png";
pub const SPRITE_PILGRIM: &str = "sprites/npc/pilgrim_idle_0.png";
pub const SPRITE_DRILLMASTER: &str = "sprites/npc/drillmaster_idle_0.png";
pub const SPRITE_RECKONER: &str = "sprites/npc/reckoner_idle_0.png";

// NPC batch 4 (2026-06-18) — Quartermaster, Auditor, Chronicler, Alchemist.
// Replaces the last four SPRITE_PLAYER placeholders. Sources:
// design/art/refs/ref-26..29-*.jsx, rendered via `cargo run -p render-refs`.
// First-pass sprites authored from design/art/specs/{slug}.md; pending
// Matt's art-review pass (the locked 3-revision approval flow).
pub const SPRITE_QUARTERMASTER: &str = "sprites/npc/quartermaster_idle_0.png";
pub const SPRITE_AUDITOR: &str = "sprites/npc/auditor_idle_0.png";
pub const SPRITE_CHRONICLER: &str = "sprites/npc/chronicler_idle_0.png";
pub const SPRITE_ALCHEMIST: &str = "sprites/npc/alchemist_idle_0.png";

// NPC batch 5 (2026-06-18) — Act 3 Guildhall Quarter: Guildmaster, Recruiter,
// Locksmith, Porter, Surveyor, Armorer. Sources: design/art/refs/ref-30..35-*.jsx,
// rendered via `cargo run -p render-refs`. First-pass, pending Matt's art review.
pub const SPRITE_GUILDMASTER: &str = "sprites/npc/guildmaster_idle_0.png";
pub const SPRITE_RECRUITER: &str = "sprites/npc/recruiter_idle_0.png";
pub const SPRITE_LOCKSMITH: &str = "sprites/npc/locksmith_idle_0.png";
pub const SPRITE_PORTER: &str = "sprites/npc/porter_idle_0.png";
pub const SPRITE_SURVEYOR: &str = "sprites/npc/surveyor_idle_0.png";
pub const SPRITE_ARMORER: &str = "sprites/npc/armorer_idle_0.png";

// NPC batch 6 (2026-06-18) — Act 4 Trait Mage's Tower: Vexis, Wandwright,
// Conjurer, Familiar, Lanternkeeper, Loremaster. Sources:
// design/art/refs/ref-36..41-*.jsx, rendered via `cargo run -p render-refs`.
// First-pass, pending Matt's art review.
pub const SPRITE_VEXIS: &str = "sprites/npc/vexis_idle_0.png";
pub const SPRITE_WANDWRIGHT: &str = "sprites/npc/wandwright_idle_0.png";
pub const SPRITE_CONJURER: &str = "sprites/npc/conjurer_idle_0.png";
pub const SPRITE_FAMILIAR: &str = "sprites/npc/familiar_idle_0.png";
pub const SPRITE_LANTERNKEEPER: &str = "sprites/npc/lanternkeeper_idle_0.png";
pub const SPRITE_LOREMASTER: &str = "sprites/npc/loremaster_idle_0.png";

// NPC batch 7 (2026-06-18) — Act 5 Tavern of Tribulations: Barkeep, Bouncer,
// Interpreter, Mixologist, Tabkeeper, Cellarer. Sources:
// design/art/refs/ref-42..47-*.jsx, rendered via `cargo run -p render-refs`.
// First-pass, pending Matt's art review.
pub const SPRITE_BARKEEP: &str = "sprites/npc/barkeep_idle_0.png";
pub const SPRITE_BOUNCER: &str = "sprites/npc/bouncer_idle_0.png";
pub const SPRITE_INTERPRETER: &str = "sprites/npc/interpreter_idle_0.png";
pub const SPRITE_MIXOLOGIST: &str = "sprites/npc/mixologist_idle_0.png";
pub const SPRITE_TABKEEPER: &str = "sprites/npc/tabkeeper_idle_0.png";
pub const SPRITE_CELLARER: &str = "sprites/npc/cellarer_idle_0.png";

// NPC batch 8 (2026-06-18) — Act 6 Iterator Forge: Keymaster, Sifter, Smelter,
// Tallywright, Riveter, Bondsmith. Sources: design/art/refs/ref-48..53-*.jsx,
// rendered via `cargo run -p render-refs`. First-pass, pending Matt's art review.
pub const SPRITE_KEYMASTER: &str = "sprites/npc/keymaster_idle_0.png";
pub const SPRITE_SIFTER: &str = "sprites/npc/sifter_idle_0.png";
pub const SPRITE_SMELTER: &str = "sprites/npc/smelter_idle_0.png";
pub const SPRITE_TALLYWRIGHT: &str = "sprites/npc/tallywright_idle_0.png";
pub const SPRITE_RIVETER: &str = "sprites/npc/riveter_idle_0.png";
pub const SPRITE_BONDSMITH: &str = "sprites/npc/bondsmith_idle_0.png";

// NPC batch 9 (2026-06-18) — Act 7 Concurrent Coast: Dockmaster, Lighthouse
// Keeper, Signaler, Tidewatch, Harbormaster, Tideforecaster. Sources:
// design/art/refs/ref-54..59-*.jsx, rendered via `cargo run -p render-refs`.
// First-pass, pending Matt's art review.
pub const SPRITE_DOCKMASTER: &str = "sprites/npc/dockmaster_idle_0.png";
pub const SPRITE_LIGHTHOUSEKEEPER: &str = "sprites/npc/lighthousekeeper_idle_0.png";
pub const SPRITE_SIGNALER: &str = "sprites/npc/signaler_idle_0.png";
pub const SPRITE_TIDEWATCH: &str = "sprites/npc/tidewatch_idle_0.png";
pub const SPRITE_HARBORMASTER: &str = "sprites/npc/harbormaster_idle_0.png";
pub const SPRITE_TIDEFORECASTER: &str = "sprites/npc/tideforecaster_idle_0.png";

// Audio (Stable Audio Open generated, baked via tools/synthwave-gen).
// Generation is a Matt-action: `powershell -ExecutionPolicy Bypass
// -File scripts/synthwave-gen.ps1`. Files may be absent during early
// development; AudioPlugin treats missing assets as silently inert.
pub const AUDIO_TITLE: &str = "audio/title.wav";
pub const AUDIO_VILLAGE: &str = "audio/village.wav";
pub const AUDIO_MISSION_CLEAR: &str = "audio/mission_clear.wav";
pub const AUDIO_MISSION_LOCKED: &str = "audio/mission_locked.wav";
pub const AUDIO_EDITOR_OPEN: &str = "audio/editor_open.wav";
pub const AUDIO_EPILOGUE: &str = "audio/epilogue.wav";

/// Every audio asset path. Used by AudioPlugin to bulk-load handles
/// at startup AND by the optional asset-existence test (skipped when
/// the synthwave-gen tool hasn't run yet).
pub const ALL_AUDIO_PATHS: &[&str] = &[
    AUDIO_TITLE,
    AUDIO_VILLAGE,
    AUDIO_MISSION_CLEAR,
    AUDIO_MISSION_LOCKED,
    AUDIO_EDITOR_OPEN,
    AUDIO_EPILOGUE,
];

/// Every asset path the game references. Used by the asset-existence
/// audit test in `tests/assets.rs`.
pub const ALL_SPRITE_PATHS: &[&str] = &[
    SPRITE_PLAYER,
    SPRITE_FERRIS,
    SPRITE_BORROW_CHECKER,
    SPRITE_TILES_VILLAGE,
    SPRITE_TITLE,
    SPRITE_SMITH,
    SPRITE_CARTOGRAPHER,
    SPRITE_TRAIT_MAGE,
    SPRITE_BELLRINGER,
    SPRITE_COOPER,
    SPRITE_ORACLE,
    SPRITE_HERALD,
    SPRITE_TWIN,
    SPRITE_TINKER,
    SPRITE_HERALDIC_SAGE,
    SPRITE_FORGEWRIGHT,
    SPRITE_LINGUIST,
    SPRITE_PILGRIM,
    SPRITE_DRILLMASTER,
    SPRITE_RECKONER,
    SPRITE_QUARTERMASTER,
    SPRITE_AUDITOR,
    SPRITE_CHRONICLER,
    SPRITE_ALCHEMIST,
    SPRITE_GUILDMASTER,
    SPRITE_RECRUITER,
    SPRITE_LOCKSMITH,
    SPRITE_PORTER,
    SPRITE_SURVEYOR,
    SPRITE_ARMORER,
    SPRITE_VEXIS,
    SPRITE_WANDWRIGHT,
    SPRITE_CONJURER,
    SPRITE_FAMILIAR,
    SPRITE_LANTERNKEEPER,
    SPRITE_LOREMASTER,
    SPRITE_BARKEEP,
    SPRITE_BOUNCER,
    SPRITE_INTERPRETER,
    SPRITE_MIXOLOGIST,
    SPRITE_TABKEEPER,
    SPRITE_CELLARER,
    SPRITE_KEYMASTER,
    SPRITE_SIFTER,
    SPRITE_SMELTER,
    SPRITE_TALLYWRIGHT,
    SPRITE_RIVETER,
    SPRITE_BONDSMITH,
    SPRITE_DOCKMASTER,
    SPRITE_LIGHTHOUSEKEEPER,
    SPRITE_SIGNALER,
    SPRITE_TIDEWATCH,
    SPRITE_HARBORMASTER,
    SPRITE_TIDEFORECASTER,
];
