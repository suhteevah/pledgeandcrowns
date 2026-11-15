// SPDX-License-Identifier: MIT
//! Editor colour roles, as ALIASES of the shared theme
//! ([`crate::plugins::ui_theme`], the single source of every colour).
//! Nothing here is a literal; this file only gives the editor's chrome
//! (buttons, banners, gutter) semantic names so the drawing code reads as
//! "what" rather than "which swatch".
//!
//! Syntax colours (bible v2.1) are the theme's `SYNTAX_*` roles; they are
//! re-exported here under the editor's historical names. The code SURFACES
//! (code pane, solution view, rustc output block) do not read these: they
//! draw from the live `ui_theme::code_theme()` (ink by default, the bible
//! v2.2 candidate; these parchment values under v2.1).

use crate::plugins::ui_theme as theme;
use bevy_egui::egui::Color32;

// --- Syntax (bible v2.1) -----------------------------------------------------

/// Keywords: Wineflesh.
pub const KEYWORD: Color32 = theme::SYNTAX_KEYWORD;
/// Error squiggle and error-line gutter bar (marks, not text): Alarm scarlet.
pub const ERROR_MARK: Color32 = theme::ERROR_MARK;
/// Plain code text: Coalblack.
pub const PLAIN: Color32 = theme::SYNTAX_PLAIN;
/// Text cursor in the code page: Abyssal.
pub const CURSOR: Color32 = theme::CODE_CURSOR;

// --- Surfaces and frame ------------------------------------------------------

/// Code page background: Parchment cream.
pub const PAGE_BG: Color32 = theme::CODE_PAGE;
/// Every editor surface (window, panes): Parchment cream.
pub const SURFACE: Color32 = theme::SURFACE;
/// Editor frame / bevel / pane borders: Basalt.
pub const FRAME: Color32 = theme::FRAME;
/// Inset chips (inline code, expected/actual) and idle buttons: Aged paper.
pub const CHIP_BG: Color32 = theme::CODE_CHIP;
/// Hovered button / text selection: Brass leaf.
pub const HOVER_BG: Color32 = theme::BRASS_LEAF;
/// Pressed button: Old gold.
pub const PRESSED_BG: Color32 = theme::OLD_GOLD;

// --- Text --------------------------------------------------------------------

/// Body text everywhere outside the code tokens: Coalblack.
pub const TEXT: Color32 = theme::TEXT;
/// Secondary text (hints, captions, disabled buttons): Basalt.
pub const TEXT_MUTED: Color32 = theme::TEXT_MUTED;
/// Headings and accents (section titles, rustc error headers): Oxblood.
pub const ACCENT: Color32 = theme::TEXT_HEADING;
/// rustc error headers in the result pane: Oxblood (error TEXT, v2.1).
pub const ERROR_TEXT: Color32 = theme::SYNTAX_ERROR_TEXT;
/// Text on an Oxblood / Pine / Basalt fill: Parchment cream.
pub const TEXT_ON_DARK: Color32 = theme::PARCHMENT_CREAM;
/// Line-number gutter (non-error lines): Bronze.
pub const GUTTER: Color32 = theme::GUTTER;
/// Gutter number of a line with an error: Oxblood on an Aged paper band.
pub const GUTTER_ERROR: Color32 = theme::GUTTER_ERROR;
/// rustc warning headers: Bronze.
pub const WARNING_TEXT: Color32 = theme::BRONZE;

// --- Result banners ----------------------------------------------------------

/// Verified pass: Pine fill, Parchment text.
pub const BANNER_PASS: Color32 = theme::PINE;
/// Not-verified / pattern-only pass: Brass leaf fill, Coalblack text. Warm
/// and calm on purpose: not an alarm.
pub const BANNER_UNVERIFIED: Color32 = HOVER_BG;
/// Failure: Oxblood fill, Parchment text.
pub const BANNER_FAIL: Color32 = theme::OXBLOOD;
/// Not checked (server busy, offline, refused): Basalt fill.
pub const BANNER_NOT_CHECKED: Color32 = FRAME;
/// Compiling: Aged paper fill, Coalblack text.
pub const BANNER_PENDING: Color32 = CHIP_BG;

/// Every (text, background) pair the editor chrome draws. Kept next to
/// the aliases so a new pair is added where its colours are chosen.
pub const CHROME_TEXT_PAIRS: &[(&str, Color32, Color32)] = &[
    ("body text on surface", TEXT, SURFACE),
    ("muted text on surface", TEXT_MUTED, SURFACE),
    ("heading on surface", ACCENT, SURFACE),
    ("code token plain on page", PLAIN, PAGE_BG),
    ("keyword on page", KEYWORD, PAGE_BG),
    ("gutter on page", GUTTER, PAGE_BG),
    ("gutter error on page", GUTTER_ERROR, PAGE_BG),
    ("gutter error on its chip", GUTTER_ERROR, CHIP_BG),
    ("rustc error header on surface", ERROR_TEXT, SURFACE),
    ("rustc warning header on surface", WARNING_TEXT, SURFACE),
    ("chip text", TEXT, CHIP_BG),
    ("idle button", TEXT, CHIP_BG),
    ("disabled button", TEXT_MUTED, SURFACE),
    ("hovered button", TEXT, HOVER_BG),
    ("pressed button", TEXT, PRESSED_BG),
    ("selected text", TEXT, HOVER_BG),
    ("title bar", TEXT_ON_DARK, ACCENT),
    ("banner pass", TEXT_ON_DARK, BANNER_PASS),
    ("banner unverified", TEXT, BANNER_UNVERIFIED),
    ("banner fail", TEXT_ON_DARK, BANNER_FAIL),
    ("banner not checked", TEXT_ON_DARK, BANNER_NOT_CHECKED),
    ("banner pending", TEXT, BANNER_PENDING),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_chrome_text_pair_meets_aa() {
        for (name, fg, bg) in CHROME_TEXT_PAIRS {
            let r = theme::contrast_ratio(*fg, *bg);
            assert!(r >= 4.5, "{name}: {r:.2}:1 is below 4.5:1");
        }
    }

    #[test]
    fn error_mark_is_scarlet_but_error_text_is_not() {
        assert_eq!(ERROR_MARK, theme::ALARM_SCARLET);
        assert_ne!(ERROR_TEXT, ERROR_MARK);
        assert_ne!(GUTTER_ERROR, ERROR_MARK);
    }
}
