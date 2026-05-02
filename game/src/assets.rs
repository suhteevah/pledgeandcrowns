// SPDX-License-Identifier: MIT
//! Single source of truth for every asset path the game loads.
//! Tests read this module to verify the files exist on disk; runtime
//! plugins read it so a renamed file fails compile, not at startup.

pub const SPRITE_PLAYER: &str = "sprites/player.png";
pub const SPRITE_FERRIS: &str = "sprites/ferris.png";
pub const SPRITE_BORROW_CHECKER: &str = "sprites/borrow_checker.png";
pub const SPRITE_TILES_VILLAGE: &str = "sprites/tiles_village.png";
pub const SPRITE_TITLE: &str = "sprites/title.png";

/// Every asset path the game references. Used by the asset-existence
/// audit test in `tests/assets.rs`.
pub const ALL_SPRITE_PATHS: &[&str] = &[
    SPRITE_PLAYER,
    SPRITE_FERRIS,
    SPRITE_BORROW_CHECKER,
    SPRITE_TILES_VILLAGE,
    SPRITE_TITLE,
];
