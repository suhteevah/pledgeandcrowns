// SPDX-License-Identifier: MIT
//! Editor colour roles: the single source of truth for every colour the
//! code editor paints. Values come from `design/03-art-style-bible.md`
//! §UI Style ("Code editor frame") and the Heraldic Code v2.0 palette.
//! Constants are named by ROLE, with the palette swatch name in the doc
//! comment, so a palette revision (or a move to a shared UI theme) is a
//! one-file change.
//!
//! Syntax colours are bible-locked. Chrome colours (everything that is
//! not a highlighted code token) are chosen so every text/background
//! pair the editor draws meets WCAG AA (4.5:1); see the tests.

use bevy_egui::egui::Color32;

// --- Syntax (bible §UI style, locked) ----------------------------------------

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
/// Errors and underline squiggles in the code page: Alarm scarlet
/// `#E63946`.
pub const ERROR: Color32 = Color32::from_rgb(0xE6, 0x39, 0x46);
/// Plain identifiers, punctuation and the text cursor: Abyssal `#062123`.
pub const PLAIN: Color32 = Color32::from_rgb(0x06, 0x21, 0x23);

// --- Surfaces and frame ------------------------------------------------------

/// Code page background: Parchment cream `#FCEFC8`.
pub const PAGE_BG: Color32 = Color32::from_rgb(0xFC, 0xEF, 0xC8);
/// Every editor surface (window, panes): Parchment cream `#FCEFC8`.
pub const SURFACE: Color32 = PAGE_BG;
/// Editor frame / bevel / pane borders: Basalt `#3E3833`.
pub const FRAME: Color32 = Color32::from_rgb(0x3E, 0x38, 0x33);
/// Inset chips (inline code, fenced code, expected/actual) and idle
/// buttons: Aged paper `#BFB2A0`.
pub const CHIP_BG: Color32 = Color32::from_rgb(0xBF, 0xB2, 0xA0);
/// Hovered button / text selection: Brass leaf `#F0D27D`.
pub const HOVER_BG: Color32 = Color32::from_rgb(0xF0, 0xD2, 0x7D);
/// Pressed button: Old gold `#D2A53F`.
pub const PRESSED_BG: Color32 = NUMBER;

// --- Text --------------------------------------------------------------------

/// Body text everywhere outside the code tokens: Coalblack `#161313`.
pub const TEXT: Color32 = Color32::from_rgb(0x16, 0x13, 0x13);
/// Secondary text (hints, captions, disabled buttons): Basalt `#3E3833`.
pub const TEXT_MUTED: Color32 = FRAME;
/// Headings and accents (section titles, rustc error headers): Oxblood
/// `#6B1F35`, the brand colour.
pub const ACCENT: Color32 = Color32::from_rgb(0x6B, 0x1F, 0x35);
/// Text on an Oxblood / Pine / Basalt fill: Parchment cream.
pub const TEXT_ON_DARK: Color32 = PAGE_BG;
/// Line-number gutter (non-error lines): Bronze `#5E4116`. Stone grey
/// (the comment colour) is only 4.2:1 on Parchment, below AA.
pub const GUTTER: Color32 = Color32::from_rgb(0x5E, 0x41, 0x16);
/// Gutter number of a line with an error: Oxblood, bold. Alarm scarlet
/// keeps the squiggle in the code itself (scarlet text is 3.6:1).
pub const GUTTER_ERROR: Color32 = ACCENT;

// --- Result banners ----------------------------------------------------------

/// Verified pass: Pine `#27502E` fill, Parchment text.
pub const BANNER_PASS: Color32 = Color32::from_rgb(0x27, 0x50, 0x2E);
/// Not-verified / pattern-only pass: Brass leaf fill, Coalblack text. Warm
/// and calm on purpose: not an alarm.
pub const BANNER_UNVERIFIED: Color32 = HOVER_BG;
/// Failure: Oxblood fill, Parchment text.
pub const BANNER_FAIL: Color32 = ACCENT;
/// Not checked (server busy, offline, refused): Basalt fill.
pub const BANNER_NOT_CHECKED: Color32 = FRAME;
/// Compiling: Aged paper fill, Coalblack text.
pub const BANNER_PENDING: Color32 = CHIP_BG;

/// Passing verdict colour for any legacy single-colour use: Pine.
pub const PASS: Color32 = BANNER_PASS;

/// WCAG 2.x contrast ratio between two opaque colours.
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    fn channel(c: u8) -> f32 {
        let s = f32::from(c) / 255.0;
        if s <= 0.040_45 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    fn luminance(c: Color32) -> f32 {
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Every (text, background) pair the editor chrome draws. Kept next to
/// the constants so a new pair is added where its colours are chosen.
pub const CHROME_TEXT_PAIRS: &[(&str, Color32, Color32)] = &[
    ("body text on surface", TEXT, SURFACE),
    ("muted text on surface", TEXT_MUTED, SURFACE),
    ("heading on surface", ACCENT, SURFACE),
    ("code token plain on page", PLAIN, PAGE_BG),
    ("keyword on page", KEYWORD, PAGE_BG),
    ("gutter on page", GUTTER, PAGE_BG),
    ("gutter error on page", GUTTER_ERROR, PAGE_BG),
    ("gutter error on its chip", GUTTER_ERROR, CHIP_BG),
    ("rustc error header on surface", ACCENT, SURFACE),
    ("rustc warning header on surface", GUTTER, SURFACE),
    ("chip text", TEXT, CHIP_BG),
    ("idle button", TEXT, CHIP_BG),
    ("disabled button", TEXT_MUTED, CHIP_BG),
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
            let r = contrast_ratio(*fg, *bg);
            assert!(r >= 4.5, "{name}: {r:.2}:1 is below 4.5:1");
        }
    }

    #[test]
    fn contrast_ratio_matches_reference_values() {
        assert!((contrast_ratio(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast_ratio(TEXT, SURFACE) - 16.1).abs() < 0.1);
        // The bible-locked comment colour is why the gutter moved off it.
        assert!(contrast_ratio(COMMENT, PAGE_BG) < 4.5);
    }
}
