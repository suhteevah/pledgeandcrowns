// SPDX-License-Identifier: MIT
//! Editor colour roles: the single source of truth for every colour the
//! code editor paints. Values come from `design/03-art-style-bible.md`
//! §UI Style ("Code editor frame") and the Heraldic Code v2.0 palette.
//! Constants are named by ROLE, with the palette swatch name in the doc
//! comment, so a palette revision is a one-file change.

use bevy_egui::egui::Color32;

/// Keywords (`fn`, `let`, `mut`): Wineflesh `#982D52`.
pub const KEYWORD: Color32 = Color32::from_rgb(0x98, 0x2D, 0x52);
/// Types (capitalised identifiers): Main teal `#2A8482`.
pub const TYPE: Color32 = Color32::from_rgb(0x2A, 0x84, 0x82);
/// String literals: Forest `#487E40`.
pub const STRING: Color32 = Color32::from_rgb(0x48, 0x7E, 0x40);
/// Comments: Stone grey `#7A7064`.
pub const COMMENT: Color32 = Color32::from_rgb(0x7A, 0x70, 0x64);
/// Number literals: Old gold `#D2A53F`.
pub const NUMBER: Color32 = Color32::from_rgb(0xD2, 0xA5, 0x3F);
/// Errors, error-line gutter marks and underline squiggles: Alarm
/// scarlet `#E63946`.
pub const ERROR: Color32 = Color32::from_rgb(0xE6, 0x39, 0x46);
/// Plain identifiers, punctuation and the text cursor: Abyssal `#062123`.
pub const PLAIN: Color32 = Color32::from_rgb(0x06, 0x21, 0x23);
/// Code page background: Parchment cream `#FCEFC8`.
pub const PAGE_BG: Color32 = Color32::from_rgb(0xFC, 0xEF, 0xC8);
/// Editor frame / bevel: Basalt `#3E3833`.
pub const FRAME: Color32 = Color32::from_rgb(0x3E, 0x38, 0x33);
/// Line-number gutter text (non-error lines): Stone grey `#7A7064`,
/// the same swatch as comments, so the gutter recedes like a comment.
pub const GUTTER: Color32 = COMMENT;
/// Passing verdict heading in the result pane: Forest `#487E40`. The
/// bible names no "success" role; Forest is the palette's green.
pub const PASS: Color32 = STRING;
