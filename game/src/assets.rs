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
];
