// SPDX-License-Identifier: MIT
//! The game's one UI look: role-named palette constants, the global egui
//! `Style`, and small widgets every panel shares.
//!
//! Source of truth is `design/03-art-style-bible.md` ("Palette" and "UI
//! style"). Panels are Parchment cream with Coalblack text inside a
//! 2 px Basalt double-line frame with iron corner blocks; no drop shadows,
//! square corners. A highlighted menu row is an Oxblood bar with Parchment
//! text and an Old gold marker; nothing uses `> x <` text markers.
//!
//! How to adopt it from another plugin (the code editor will):
//! - [`UiThemePlugin`] installs [`apply_style`] on the primary egui
//!   context once, so plain `ui.label` / `ui.heading` / sliders / scroll
//!   bars already match. Nothing else is required.
//! - Build a panel with [`panel_area`] + [`framed_panel`] (or call
//!   [`framed_panel`] inside any `Ui`), then [`heading`], [`menu_item`],
//!   [`key_hint`] and [`status_mark`] inside it.
//! - Tutorial text goes through [`markdown::show`], which renders the
//!   small markdown subset the curriculum uses. It is the game's ONLY
//!   markdown renderer (mission prompt, recap, editor goal pane); fenced
//!   code and the editor's solution view share [`markdown::code_block`]
//!   and the syntax colouring in [`highlight`].
//! - Colours: every colour literal in `game/src/plugins` lives in this
//!   file (a test enforces it). Use a role (`TEXT`, `SURFACE`,
//!   `SYNTAX_KEYWORD`, ...) or, failing that, a swatch by bible name.
//! - Text sizes: never below [`MIN_TEXT_SIZE`]; use [`BODY_SIZE`] for
//!   reading text. Every text/surface pair this module uses is listed in
//!   [`TEXT_PAIRS`] and unit-tested for at least 4.5:1 contrast.
//!
//! The font is still egui's default (Ubuntu-Light / Hack): the bible's
//! pixel font is an owner decision; no licensed pixel font is in the repo.

pub mod highlight;
pub mod markdown;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

// --- Palette: the bible's 32 swatches, by bible name ------------------------
// This file is the ONLY place in `game/src/plugins` where a colour is written
// as a literal (enforced by `tests::no_colour_literals_outside_ui_theme`).
// Every other plugin imports a role below (or a swatch by name).
// Burgundy ramp (anchor).
/// `#1B0810` outline / deepest shadow on red surfaces.
pub const INKBLOOD: egui::Color32 = egui::Color32::from_rgb(0x1B, 0x08, 0x10);
/// `#3E1220` deep heraldic shadow.
pub const CRYPT: egui::Color32 = egui::Color32::from_rgb(0x3E, 0x12, 0x20);
/// `#6B1F35` primary brand: selection bars, name plates, headings.
pub const OXBLOOD: egui::Color32 = egui::Color32::from_rgb(0x6B, 0x1F, 0x35);
/// `#982D52` mid-tone; slider fill.
pub const WINEFLESH: egui::Color32 = egui::Color32::from_rgb(0x98, 0x2D, 0x52);
/// `#C56883` soft fabric highlight.
pub const DUSTY_ROSE: egui::Color32 = egui::Color32::from_rgb(0xC5, 0x68, 0x83);
/// `#EBC2CC` pale rim light; text-selection background.
pub const PINK_QUARTZ: egui::Color32 = egui::Color32::from_rgb(0xEB, 0xC2, 0xCC);
// Gold ramp.
/// `#2D1F0A` gold deepest shadow.
pub const BOG_UMBER: egui::Color32 = egui::Color32::from_rgb(0x2D, 0x1F, 0x0A);
/// `#5E4116` antique metal mid-shadow; code comments, line-number gutter.
pub const BRONZE: egui::Color32 = egui::Color32::from_rgb(0x5E, 0x41, 0x16);
/// `#9C7026` gold mid-tone.
pub const ANTIQUE_BRASS: egui::Color32 = egui::Color32::from_rgb(0x9C, 0x70, 0x26);
/// `#D2A53F` the crown: selection marker, corner rivets.
pub const OLD_GOLD: egui::Color32 = egui::Color32::from_rgb(0xD2, 0xA5, 0x3F);
/// `#F0D27D` bright metal highlight.
pub const BRASS_LEAF: egui::Color32 = egui::Color32::from_rgb(0xF0, 0xD2, 0x7D);
/// `#FCEFC8` primary UI surface.
pub const PARCHMENT_CREAM: egui::Color32 = egui::Color32::from_rgb(0xFC, 0xEF, 0xC8);
// Teal ramp.
/// `#062123` code-editor cursor.
pub const ABYSSAL: egui::Color32 = egui::Color32::from_rgb(0x06, 0x21, 0x23);
/// `#154548` borrow-checker robe shadow; types in code (bible v2.1).
pub const DEEP_TEAL: egui::Color32 = egui::Color32::from_rgb(0x15, 0x45, 0x48);
/// `#2A8482` Borrow Checker signature (not text-safe on parchment: 3.9:1).
pub const MAIN_TEAL: egui::Color32 = egui::Color32::from_rgb(0x2A, 0x84, 0x82);
/// `#5BB8AF` magic emission, ice highlights.
pub const BRIGHT_TEAL: egui::Color32 = egui::Color32::from_rgb(0x5B, 0xB8, 0xAF);
/// `#A4DED4` magic glow rim, fog.
pub const MIST_TEAL: egui::Color32 = egui::Color32::from_rgb(0xA4, 0xDE, 0xD4);
// Greens.
/// `#142A19` foliage shadow.
pub const MOSSBED: egui::Color32 = egui::Color32::from_rgb(0x14, 0x2A, 0x19);
/// `#27502E` dark grass; strings in code (bible v2.1); pass banner.
pub const PINE: egui::Color32 = egui::Color32::from_rgb(0x27, 0x50, 0x2E);
/// `#487E40` primary grass; the "cleared" tick (a mark, not text).
pub const FOREST: egui::Color32 = egui::Color32::from_rgb(0x48, 0x7E, 0x40);
/// `#82B450` fresh growth, lit grass.
pub const SPRING_MEADOW: egui::Color32 = egui::Color32::from_rgb(0x82, 0xB4, 0x50);
/// `#C9DC97` pale grass, wheat.
pub const HAYFIELD: egui::Color32 = egui::Color32::from_rgb(0xC9, 0xDC, 0x97);
// Neutrals.
/// `#161313` universal outline and body text (never pure black).
pub const COALBLACK: egui::Color32 = egui::Color32::from_rgb(0x16, 0x13, 0x13);
/// `#3E3833` frames; secondary text.
pub const BASALT: egui::Color32 = egui::Color32::from_rgb(0x3E, 0x38, 0x33);
/// `#7A7064` separators only (never text: 4.2:1 on parchment).
pub const STONE_GREY: egui::Color32 = egui::Color32::from_rgb(0x7A, 0x70, 0x64);
/// `#BFB2A0` old documents: inline code chips, slider rails, idle bevel shadow.
pub const AGED_PAPER: egui::Color32 = egui::Color32::from_rgb(0xBF, 0xB2, 0xA0);
// Tetradic violet (magic only).
/// `#3A1559` magic shadow, corruption.
pub const ROYAL_ARCANE: egui::Color32 = egui::Color32::from_rgb(0x3A, 0x15, 0x59);
/// `#9D6FE0` spell emission, runes.
pub const MAGE_GLOW: egui::Color32 = egui::Color32::from_rgb(0x9D, 0x6F, 0xE0);
// Cool accents.
/// `#0E2E54` night sky; number literals in code (bible v2.1).
pub const COBALT_DEEP: egui::Color32 = egui::Color32::from_rgb(0x0E, 0x2E, 0x54);
/// `#377AB8` daytime water.
pub const COBALT: egui::Color32 = egui::Color32::from_rgb(0x37, 0x7A, 0xB8);
/// `#E63946` error MARKS only (squiggles, gutter bar): 3.6:1 is not text-safe.
pub const ALARM_SCARLET: egui::Color32 = egui::Color32::from_rgb(0xE6, 0x39, 0x46);
/// `#FFFFFF` pure highlights only.
pub const SPECULAR_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);

// --- Roles -----------------------------------------------------------------
/// Panel background.
pub const SURFACE: egui::Color32 = PARCHMENT_CREAM;
/// Body text on [`SURFACE`].
pub const TEXT: egui::Color32 = COALBLACK;
/// Secondary text (hints, captions) on [`SURFACE`].
pub const TEXT_MUTED: egui::Color32 = BASALT;
/// Headings on [`SURFACE`].
pub const TEXT_HEADING: egui::Color32 = OXBLOOD;
/// Panel frame lines.
pub const FRAME: egui::Color32 = BASALT;
/// Highlighted row fill.
pub const SELECT_FILL: egui::Color32 = OXBLOOD;
/// Text on [`SELECT_FILL`].
pub const SELECT_TEXT: egui::Color32 = PARCHMENT_CREAM;
/// The selection marker and corner rivets.
pub const MARKER: egui::Color32 = OLD_GOLD;
/// Inline code chip background (plain Coalblack code text only: the
/// syntax colours are not all 4.5:1 on Aged paper, see [`SYNTAX_TEXT`]).
pub const CODE_CHIP: egui::Color32 = AGED_PAPER;
/// Text on [`CODE_CHIP`].
pub const CODE_CHIP_TEXT: egui::Color32 = COALBLACK;
/// Thin rules between panel sections.
pub const RULE: egui::Color32 = STONE_GREY;

// --- Code roles (bible v2.1, "Code editor frame") ---------------------------
/// Every surface syntax-coloured code is drawn on: the editor's code page,
/// the revealed solution and fenced code blocks in tutorials. All three are
/// Parchment cream; fenced blocks get a 1 px [`CODE_BLOCK_FRAME`] instead
/// of an Aged paper chip (bible v2.1).
pub const CODE_PAGE: egui::Color32 = PARCHMENT_CREAM;
/// 1 px frame around a fenced code block / the solution view.
pub const CODE_BLOCK_FRAME: egui::Color32 = BASALT;
/// Keywords (`fn`, `let`, `mut`).
pub const SYNTAX_KEYWORD: egui::Color32 = WINEFLESH;
/// Types (capitalised identifiers).
pub const SYNTAX_TYPE: egui::Color32 = DEEP_TEAL;
/// String literals.
pub const SYNTAX_STRING: egui::Color32 = PINE;
/// Comments: quieter than code by hue, not by low contrast.
pub const SYNTAX_COMMENT: egui::Color32 = BRONZE;
/// Number literals (gold cannot reach 4.5:1 on cream and still read gold).
pub const SYNTAX_NUMBER: egui::Color32 = COBALT_DEEP;
/// Identifiers and punctuation.
pub const SYNTAX_PLAIN: egui::Color32 = COALBLACK;
/// Error text (rustc error headers, an error line's gutter number).
pub const SYNTAX_ERROR_TEXT: egui::Color32 = OXBLOOD;
/// Error marks that are not text: the underline squiggle and the gutter
/// bar on an error line (non-text needs 3:1; scarlet is 3.6:1).
pub const ERROR_MARK: egui::Color32 = ALARM_SCARLET;
/// Text cursor in code.
pub const CODE_CURSOR: egui::Color32 = ABYSSAL;
/// Line numbers in the code page's gutter.
pub const GUTTER: egui::Color32 = BRONZE;
/// Line number of a line rustc flagged (drawn on a [`CODE_CHIP`] band).
pub const GUTTER_ERROR: egui::Color32 = SYNTAX_ERROR_TEXT;

/// Every syntax text colour, for the contrast test.
pub const SYNTAX_TEXT: &[(&str, egui::Color32)] = &[
    ("keyword", SYNTAX_KEYWORD),
    ("type", SYNTAX_TYPE),
    ("string", SYNTAX_STRING),
    ("comment", SYNTAX_COMMENT),
    ("number", SYNTAX_NUMBER),
    ("plain", SYNTAX_PLAIN),
    ("error text", SYNTAX_ERROR_TEXT),
    ("gutter", GUTTER),
];
/// Every surface syntax text is drawn on.
pub const SYNTAX_SURFACES: &[(&str, egui::Color32)] = &[("code page / code block", CODE_PAGE)];

/// The one inline-code look (markdown and dialogue): monospace
/// [`CODE_CHIP_TEXT`] on a [`CODE_CHIP`] band.
pub fn inline_code_format(size: f32) -> egui::TextFormat {
    egui::TextFormat {
        font_id: egui::FontId::monospace(size),
        color: CODE_CHIP_TEXT,
        background: CODE_CHIP,
        expand_bg: 1.0,
        ..Default::default()
    }
}

// --- Sizes (points at 1x) --------------------------------------------------
/// No text anywhere is smaller than this.
pub const MIN_TEXT_SIZE: f32 = 13.0;
/// Captions, key hints, the HUD pin's sub-line.
pub const SMALL_SIZE: f32 = 14.0;
/// Reading text (tutorial, prompts, dialogue body).
pub const BODY_SIZE: f32 = 16.0;
/// Monospace (code chips sit a hair smaller so they align with body text).
pub const MONO_SIZE: f32 = 15.0;
/// Menu rows.
pub const MENU_ITEM_SIZE: f32 = 18.0;
/// Panel headings.
pub const HEADING_SIZE: f32 = 22.0;
/// Section headings inside tutorial text.
pub const SUBHEADING_SIZE: f32 = 18.0;
/// Menu row size.
pub const MENU_ITEM_WIDTH: f32 = 240.0;
pub const MENU_ITEM_HEIGHT: f32 = 32.0;

/// Every (text, surface) pair the themed UI draws, for the contrast test.
pub const TEXT_PAIRS: &[(&str, egui::Color32, egui::Color32)] = &[
    ("body on parchment", TEXT, SURFACE),
    ("muted on parchment", TEXT_MUTED, SURFACE),
    ("heading on parchment", TEXT_HEADING, SURFACE),
    ("strong (wineflesh) on parchment", WINEFLESH, SURFACE),
    ("selected row text", SELECT_TEXT, SELECT_FILL),
    ("code on aged paper", TEXT, CODE_CHIP),
    ("inline code chip", CODE_CHIP_TEXT, CODE_CHIP),
    ("muted on aged paper", TEXT_MUTED, CODE_CHIP),
    (
        "error-line gutter number on its band",
        GUTTER_ERROR,
        CODE_CHIP,
    ),
    ("body on text selection", TEXT, PINK_QUARTZ),
    (
        "error on parchment (scarlet is not text-safe; use oxblood)",
        OXBLOOD,
        SURFACE,
    ),
];

// --- Contrast ---------------------------------------------------------------

fn channel_linear(c: u8) -> f64 {
    let s = f64::from(c) / 255.0;
    if s <= 0.040_45 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.x relative luminance.
pub fn relative_luminance(c: egui::Color32) -> f64 {
    0.2126 * channel_linear(c.r()) + 0.7152 * channel_linear(c.g()) + 0.0722 * channel_linear(c.b())
}

/// WCAG 2.x contrast ratio (1.0 ..= 21.0).
pub fn contrast_ratio(a: egui::Color32, b: egui::Color32) -> f64 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

// --- Global style -----------------------------------------------------------

fn widget(
    fill: egui::Color32,
    stroke: egui::Color32,
    text: egui::Color32,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: egui::Stroke::new(1.0_f32, stroke),
        corner_radius: egui::CornerRadius::ZERO,
        fg_stroke: egui::Stroke::new(1.5_f32, text),
        expansion: 0.0,
    }
}

/// Mutate `style` into the Pledge & Crown look. Pure; safe to call on any
/// `Style` (the plugin applies it to both egui themes).
pub fn apply_style(style: &mut egui::Style) {
    use egui::{FontFamily, FontId, TextStyle};
    style.text_styles = [
        (
            TextStyle::Small,
            FontId::new(SMALL_SIZE, FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(BODY_SIZE, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(BODY_SIZE, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(HEADING_SIZE, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(MONO_SIZE, FontFamily::Monospace),
        ),
    ]
    .into();

    let v = &mut style.visuals;
    v.dark_mode = false;
    v.override_text_color = None;
    v.weak_text_color = Some(TEXT_MUTED);
    v.hyperlink_color = OXBLOOD;
    v.faint_bg_color = SURFACE;
    v.extreme_bg_color = SURFACE;
    v.text_edit_bg_color = Some(SURFACE);
    v.code_bg_color = CODE_CHIP;
    v.warn_fg_color = BRONZE;
    v.error_fg_color = OXBLOOD;
    v.window_corner_radius = egui::CornerRadius::ZERO;
    v.menu_corner_radius = egui::CornerRadius::ZERO;
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.window_fill = SURFACE;
    v.panel_fill = SURFACE;
    v.window_stroke = egui::Stroke::new(2.0_f32, FRAME);
    v.window_highlight_topmost = false;
    v.selection.bg_fill = PINK_QUARTZ;
    v.selection.stroke = egui::Stroke::new(1.0_f32, OXBLOOD);
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.6 };
    v.striped = false;
    v.text_cursor.stroke = egui::Stroke::new(2.0_f32, CODE_CURSOR);

    // Idle widgets: parchment with a Basalt edge (the bevel is drawn by
    // `menu_item`; plain egui buttons get the edge only). Hover: Oxblood
    // with Parchment text and an Old gold edge.
    v.widgets.noninteractive = widget(SURFACE, RULE, TEXT);
    v.widgets.inactive = widget(AGED_PAPER, BASALT, TEXT);
    v.widgets.hovered = widget(SELECT_FILL, MARKER, SELECT_TEXT);
    // Pressed is "bevel down" (Aged paper), not Oxblood: egui also uses
    // `widgets.active` text colour for `RichText::strong()`, which must
    // stay dark on parchment.
    v.widgets.active = widget(AGED_PAPER, BASALT, TEXT);
    v.widgets.open = widget(AGED_PAPER, BASALT, TEXT);

    let s = &mut style.spacing;
    s.item_spacing = egui::vec2(8.0, 6.0);
    s.button_padding = egui::vec2(10.0, 4.0);
    s.window_margin = egui::Margin::same(12);
    s.slider_width = 180.0;
    s.slider_rail_height = 6.0;
    s.interact_size.y = 24.0;
    s.scroll = egui::style::ScrollStyle::solid();
    s.scroll.bar_width = 10.0;
}

/// Install the theme on a context: light theme, both theme slots styled.
pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    ctx.all_styles_mut(apply_style);
    tracing::debug!("ui_theme: style installed");
}

pub struct UiThemePlugin;

impl Plugin for UiThemePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("UiThemePlugin::build");
        app.add_systems(EguiPrimaryContextPass, install_once);
    }
}

fn install_once(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    install(ctx);
    *done = true;
    tracing::info!("ui_theme: Pledge & Crown egui theme active");
}

// --- Panels -----------------------------------------------------------------

/// Outer edge of a framed panel (the first of the two Basalt lines).
fn frame_line(margin: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(egui::Stroke::new(2.0_f32, FRAME))
        .inner_margin(egui::Margin::same(margin))
}

/// Draw `add` inside the bible's window frame: two nested 2 px Basalt
/// lines with a parchment gap (the same construction as the dialogue
/// box) and a small iron block with a gold rivet on each corner.
pub fn framed_panel<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let out = frame_line(2).show(ui, |ui| frame_line(12).show(ui, add).inner);
    paint_corner_blocks(ui.painter(), out.response.rect);
    out.inner
}

fn paint_corner_blocks(painter: &egui::Painter, rect: egui::Rect) {
    const BLOCK: f32 = 10.0;
    const RIVET: f32 = 4.0;
    for corner in [
        rect.left_top(),
        rect.right_top() - egui::vec2(BLOCK, 0.0),
        rect.left_bottom() - egui::vec2(0.0, BLOCK),
        rect.right_bottom() - egui::vec2(BLOCK, BLOCK),
    ] {
        let block = egui::Rect::from_min_size(corner, egui::Vec2::splat(BLOCK));
        painter.rect_filled(block, egui::CornerRadius::ZERO, FRAME);
        let rivet = egui::Rect::from_center_size(block.center(), egui::Vec2::splat(RIVET));
        painter.rect_filled(rivet, egui::CornerRadius::ZERO, MARKER);
    }
}

/// A frameless, non-interactive-by-itself area anchored on screen. Put a
/// [`framed_panel`] inside. Modal panels use `Order::Foreground`.
pub fn panel_area(
    id: &str,
    align: egui::Align2,
    offset: egui::Vec2,
    order: egui::Order,
) -> egui::Area {
    egui::Area::new(egui::Id::new(id))
        .order(order)
        .anchor(align, offset)
        .interactable(true)
}

/// A panel heading in Oxblood.
pub fn heading(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.label(
        egui::RichText::new(text.into())
            .size(HEADING_SIZE)
            .color(TEXT_HEADING)
            .strong(),
    )
}

/// Secondary caption text (never under [`SMALL_SIZE`]).
pub fn caption(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.label(
        egui::RichText::new(text.into())
            .size(SMALL_SIZE)
            .color(TEXT_MUTED),
    )
}

/// A 1 px rule spanning the panel.
pub fn rule(ui: &mut egui::Ui) {
    ui.add_space(2.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, RULE);
    ui.add_space(2.0);
}

/// One `[key] action` pair per entry, on one line, in caption size. The
/// key is Oxblood, the action Basalt.
pub fn key_hint(ui: &mut egui::Ui, pairs: &[(&str, &str)]) {
    let mut job = egui::text::LayoutJob::default();
    let font = egui::FontId::proportional(SMALL_SIZE);
    for (i, (key, action)) in pairs.iter().enumerate() {
        let lead = if i == 0 { 0.0 } else { 14.0 };
        job.append(
            &format!("[{key}]"),
            lead,
            egui::TextFormat::simple(font.clone(), OXBLOOD),
        );
        job.append(
            action,
            5.0,
            egui::TextFormat::simple(font.clone(), TEXT_MUTED),
        );
    }
    ui.label(job);
}

/// Paint the gold selection marker (a right-pointing pennant) with its
/// left edge at `x`, vertically centred on `cy`.
fn paint_marker(painter: &egui::Painter, x: f32, cy: f32) {
    let pts = vec![
        egui::pos2(x, cy - 5.0),
        egui::pos2(x + 6.0, cy),
        egui::pos2(x, cy + 5.0),
    ];
    painter.add(egui::Shape::convex_polygon(pts, MARKER, egui::Stroke::NONE));
}

/// Paint one menu row into `rect`. Idle: parchment with a slight raised
/// bevel. Selected: Oxblood with Parchment text and gold markers either
/// side. Pressed: bevel inverted and the content shifted 1 px.
pub fn paint_menu_row(
    ui: &egui::Ui,
    rect: egui::Rect,
    label: &str,
    selected: bool,
    pressed: bool,
    size: f32,
) {
    let painter = ui.painter();
    let offset = if pressed {
        egui::vec2(1.0, 1.0)
    } else {
        egui::Vec2::ZERO
    };
    let (fill, text) = if selected {
        (SELECT_FILL, SELECT_TEXT)
    } else {
        (SURFACE, TEXT)
    };
    painter.rect_filled(rect.translate(offset), egui::CornerRadius::ZERO, fill);
    // Bevel: light top/left, dark bottom/right (swapped while pressed).
    let (light, dark) = if selected {
        (WINEFLESH, INKBLOOD)
    } else {
        (SPECULAR_WHITE, AGED_PAPER)
    };
    let (tl, br) = if pressed {
        (dark, light)
    } else {
        (light, dark)
    };
    let r = rect.shrink(0.5);
    painter.line_segment(
        [r.left_top(), r.right_top()],
        egui::Stroke::new(1.0_f32, tl),
    );
    painter.line_segment(
        [r.left_top(), r.left_bottom()],
        egui::Stroke::new(1.0_f32, tl),
    );
    painter.line_segment(
        [r.left_bottom(), r.right_bottom()],
        egui::Stroke::new(1.0_f32, br),
    );
    painter.line_segment(
        [r.right_top(), r.right_bottom()],
        egui::Stroke::new(1.0_f32, br),
    );
    if selected {
        let cy = rect.center().y + offset.y;
        paint_marker(painter, rect.left() + 10.0 + offset.x, cy);
        // Mirror on the right.
        let x = rect.right() - 10.0 + offset.x;
        let pts = vec![
            egui::pos2(x, cy - 5.0),
            egui::pos2(x - 6.0, cy),
            egui::pos2(x, cy + 5.0),
        ];
        painter.add(egui::Shape::convex_polygon(pts, MARKER, egui::Stroke::NONE));
    }
    painter.text(
        rect.center() + offset,
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(size),
        text,
    );
}

/// A menu row of `width` points. Returns the click response; the caller
/// owns selection (hover → select, click → activate), exactly as with the
/// old `Button::selectable`.
pub fn menu_item(ui: &mut egui::Ui, label: &str, selected: bool, width: f32) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(width, MENU_ITEM_HEIGHT), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        paint_menu_row(
            ui,
            rect,
            label,
            selected,
            resp.is_pointer_button_down_on(),
            MENU_ITEM_SIZE,
        );
    }
    resp
}

/// Small status glyphs drawn as shapes (the font has no tick glyph).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// A Forest tick.
    Cleared,
    /// A Basalt padlock.
    Locked,
    /// The gold pennant (the next thing to do).
    Next,
}

/// Draw a status mark in a square the height of `size`-point text.
pub fn status_mark(ui: &mut egui::Ui, status: Status, size: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());
    let p = ui.painter();
    let r = rect.shrink(size * 0.12);
    match status {
        Status::Cleared => {
            let stroke = egui::Stroke::new((size / 7.0).max(2.0), FOREST);
            let a = egui::pos2(r.left(), r.center().y);
            let b = egui::pos2(r.left() + r.width() * 0.38, r.bottom() - r.height() * 0.1);
            let c = egui::pos2(r.right(), r.top() + r.height() * 0.1);
            p.line_segment([a, b], stroke);
            p.line_segment([b, c], stroke);
        }
        Status::Locked => {
            let body = egui::Rect::from_min_max(
                egui::pos2(r.left(), r.center().y - 1.0),
                r.right_bottom(),
            );
            p.rect_filled(body, egui::CornerRadius::ZERO, BASALT);
            let shackle = egui::Rect::from_min_max(
                egui::pos2(r.left() + r.width() * 0.22, r.top()),
                egui::pos2(r.right() - r.width() * 0.22, r.center().y),
            );
            p.rect_stroke(
                shackle,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(2.0_f32, BASALT),
                egui::StrokeKind::Inside,
            );
        }
        Status::Next => paint_marker(p, r.left() + r.width() * 0.25, r.center().y),
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_pair_meets_4_5_to_1() {
        for (name, fg, bg) in TEXT_PAIRS {
            let ratio = contrast_ratio(*fg, *bg);
            println!("{name}: {ratio:.2}:1");
            assert!(ratio >= 4.5, "{name} is only {ratio:.2}:1");
        }
    }

    #[test]
    fn contrast_ratio_matches_wcag_reference_points() {
        let black = egui::Color32::from_rgb(0, 0, 0);
        assert!((contrast_ratio(SPECULAR_WHITE, black) - 21.0).abs() < 0.01);
        assert!((contrast_ratio(black, black) - 1.0).abs() < 1e-9);
        // Order does not matter.
        assert_eq!(contrast_ratio(TEXT, SURFACE), contrast_ratio(SURFACE, TEXT));
    }

    #[test]
    fn stone_grey_is_not_safe_for_body_text() {
        // Documents why RULE never carries reading text and why comments
        // moved to Bronze in bible v2.1.
        assert!(contrast_ratio(STONE_GREY, SURFACE) < 4.5);
    }

    /// Bible v2.1: every syntax text colour is at least 4.5:1 on every
    /// surface it is drawn on.
    #[test]
    fn syntax_text_meets_4_5_on_every_code_surface() {
        for (surface_name, surface) in SYNTAX_SURFACES {
            for (name, fg) in SYNTAX_TEXT {
                let ratio = contrast_ratio(*fg, *surface);
                println!("{name} on {surface_name}: {ratio:.2}:1");
                assert!(
                    ratio >= 4.5,
                    "{name} on {surface_name} is only {ratio:.2}:1"
                );
            }
        }
    }

    /// Why fenced code blocks are Parchment with a Basalt frame rather than
    /// an Aged paper chip (bible v2.1): three of the syntax colours fall
    /// under 4.5:1 on Aged paper. If this ever stops failing, chips could
    /// carry syntax colour again; until then only plain code text may.
    #[test]
    fn syntax_colours_are_not_chip_safe_so_chips_stay_plain() {
        let failing: Vec<&str> = SYNTAX_TEXT
            .iter()
            .filter(|(_, fg)| contrast_ratio(*fg, CODE_CHIP) < 4.5)
            .map(|(n, _)| *n)
            .collect();
        assert_eq!(failing, vec!["keyword", "string", "comment", "gutter"]);
        assert_ne!(CODE_PAGE, CODE_CHIP);
        assert!(contrast_ratio(CODE_CHIP_TEXT, CODE_CHIP) >= 4.5);
    }

    /// Error marks are non-text: 3:1 (WCAG 1.4.11) on the code page.
    #[test]
    fn error_mark_meets_non_text_contrast() {
        assert!(contrast_ratio(ERROR_MARK, CODE_PAGE) >= 3.0);
        // ...and is not text-safe, which is why error TEXT is Oxblood.
        assert!(contrast_ratio(ERROR_MARK, CODE_PAGE) < 4.5);
    }

    /// Syntax roles must stay distinguishable from each other: no two
    /// token classes share a colour.
    #[test]
    fn syntax_roles_are_distinct() {
        let roles = [
            SYNTAX_KEYWORD,
            SYNTAX_TYPE,
            SYNTAX_STRING,
            SYNTAX_COMMENT,
            SYNTAX_NUMBER,
            SYNTAX_PLAIN,
            SYNTAX_ERROR_TEXT,
        ];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    /// Files under `game/src/plugins` that may contain a colour literal,
    /// and why. Everything else must import from this module.
    const COLOUR_LITERAL_ALLOW: &[(&str, &str)] = &[(
        "quest_marker.rs",
        "deliberately off-palette neon gold so the marker pops against grass \
         and cobble alike; baked into a generated RGBA image, not egui",
    )];

    /// Does `line` contain a colour literal? Catches egui/Bevy colour
    /// constructors called with literal arguments, named colour constants,
    /// `#RRGGBB` hex strings and runs of three hex bytes.
    fn colour_literal(line: &str) -> Option<&'static str> {
        let code = line.trim_start();
        if code.starts_with("//") {
            return None;
        }
        const CTORS: &[&str] = &[
            "from_rgb(",
            "from_rgba_unmultiplied(",
            "from_rgba_premultiplied(",
            "from_gray(",
            "srgb(",
            "srgba(",
            "srgb_u8(",
            "srgba_u8(",
            "srgb_from_array(",
            "linear_rgb(",
            "hex(",
        ];
        for ctor in CTORS {
            let mut rest = code;
            while let Some(i) = rest.find(ctor) {
                let after = rest[i + ctor.len()..].trim_start();
                if after.starts_with(|c: char| c.is_ascii_digit() || c == '"' || c == '[') {
                    return Some(ctor);
                }
                rest = &rest[i + ctor.len()..];
            }
        }
        for named in [
            "Color32::BLACK",
            "Color32::WHITE",
            "Color32::RED",
            "Color32::GREEN",
            "Color32::BLUE",
            "Color32::YELLOW",
            "Color32::GOLD",
            "Color32::GRAY",
            "Color32::DARK_",
            "Color32::LIGHT_",
            "Color32::KHAKI",
            "Color32::BROWN",
            "Color32::ORANGE",
            "Color::BLACK",
            "Color::WHITE",
            "palettes::",
        ] {
            if code.contains(named) {
                return Some("named colour constant");
            }
        }
        let b = code.as_bytes();
        let is_hex = |c: u8| c.is_ascii_hexdigit();
        for (i, &c) in b.iter().enumerate() {
            // "#RRGGBB" (or #RRGGBBAA) not followed by more hex.
            if c == b'#' && i + 7 <= b.len() && b[i + 1..i + 7].iter().all(|c| is_hex(*c)) {
                let end = b[i + 7..].iter().take_while(|c| is_hex(**c)).count();
                if end == 0 || end == 2 {
                    return Some("#hex");
                }
            }
        }
        // Three `0xNN` bytes in a row: an RGB(A) array.
        let mut run = 0;
        let mut rest = code;
        while let Some(i) = rest.find("0x") {
            let digits = rest[i + 2..].bytes().take_while(|c| is_hex(*c)).count();
            let tail = rest[i + 2 + digits..].trim_start();
            if digits == 2 && (tail.starts_with(',') || tail.starts_with(']')) {
                run += 1;
                if run >= 3 {
                    return Some("hex byte triple");
                }
            } else {
                run = 0;
            }
            rest = &rest[i + 2 + digits..];
        }
        None
    }

    #[test]
    fn colour_literal_detector_catches_the_usual_shapes() {
        assert!(colour_literal("let c = egui::Color32::from_rgb(0x98, 0x2D, 0x52);").is_some());
        assert!(colour_literal("Color::srgb(0.5, 0.2, 0.1)").is_some());
        assert!(colour_literal("const G: [u8; 4] = [0xFF, 0xDE, 0x2A, 0xFF];").is_some());
        assert!(colour_literal("let s = \"#982D52\";").is_some());
        assert!(colour_literal("x.color(egui::Color32::WHITE)").is_some());
        // Not literals: conversions, transparency, comments, ids.
        assert!(colour_literal("Color::srgb_u8(c.r(), c.g(), c.b())").is_none());
        assert!(colour_literal("background: egui::Color32::TRANSPARENT,").is_none());
        assert!(colour_literal("/// Wineflesh #982D52").is_none());
        assert!(colour_literal("let id = 0x1234;").is_none());
        assert!(colour_literal("format!(\"#{n}\")").is_none());
    }

    fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            panic!("cannot read {}", dir.display());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// One theme: no colour literal anywhere in `game/src/plugins` outside
    /// this file (and its allow-list).
    #[test]
    fn no_colour_literals_outside_ui_theme() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugins");
        let mut files = Vec::new();
        rust_files(&root, &mut files);
        assert!(files.len() > 10, "found only {} files", files.len());
        let mut hits = Vec::new();
        for path in &files {
            let rel = path
                .strip_prefix(&root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if rel == "ui_theme.rs" || COLOUR_LITERAL_ALLOW.iter().any(|(f, _)| *f == rel) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(path) else {
                panic!("cannot read {}", path.display());
            };
            for (n, line) in text.lines().enumerate() {
                if let Some(kind) = colour_literal(line) {
                    hits.push(format!("{rel}:{}: {kind}: {}", n + 1, line.trim()));
                }
            }
        }
        assert!(
            hits.is_empty(),
            "colour literals outside ui_theme.rs (import a role instead):\n{}",
            hits.join("\n")
        );
    }

    #[test]
    fn no_text_style_is_below_the_minimum() {
        let mut style = egui::Style::default();
        apply_style(&mut style);
        for (ts, font) in &style.text_styles {
            assert!(font.size >= MIN_TEXT_SIZE, "{ts:?} is {}", font.size);
        }
        for s in [
            SMALL_SIZE,
            BODY_SIZE,
            MONO_SIZE,
            MENU_ITEM_SIZE,
            HEADING_SIZE,
            SUBHEADING_SIZE,
        ] {
            assert!(s >= MIN_TEXT_SIZE);
        }
    }

    #[test]
    fn style_has_no_shadows_or_rounding() {
        let mut style = egui::Style::default();
        apply_style(&mut style);
        let v = &style.visuals;
        assert_eq!(v.window_shadow, egui::Shadow::NONE);
        assert_eq!(v.popup_shadow, egui::Shadow::NONE);
        assert_eq!(v.window_corner_radius, egui::CornerRadius::ZERO);
        assert_eq!(v.window_fill, PARCHMENT_CREAM);
        assert_eq!(v.widgets.hovered.weak_bg_fill, OXBLOOD);
        assert_eq!(v.widgets.inactive.corner_radius, egui::CornerRadius::ZERO);
        assert!(!v.dark_mode);
    }

    /// egui derives default, strong and weak label colours from the
    /// widget visuals; all three must stay readable on the panel surface.
    #[test]
    fn derived_label_colours_are_readable_on_parchment() {
        let mut style = egui::Style::default();
        apply_style(&mut style);
        let v = &style.visuals;
        for (name, c) in [
            ("text", v.text_color()),
            ("strong", v.strong_text_color()),
            ("weak", v.weak_text_color()),
        ] {
            let ratio = contrast_ratio(c, SURFACE);
            assert!(ratio >= 4.5, "{name} label colour is {ratio:.2}:1");
        }
        // Hovered rows: Parchment text on Oxblood.
        let h = &v.widgets.hovered;
        assert!(contrast_ratio(h.fg_stroke.color, h.weak_bg_fill) >= 4.5);
    }

    #[test]
    fn constants_match_the_bible_hex_values() {
        let hex = |c: egui::Color32| format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b());
        assert_eq!(hex(PARCHMENT_CREAM), "#FCEFC8");
        assert_eq!(hex(AGED_PAPER), "#BFB2A0");
        assert_eq!(hex(BASALT), "#3E3833");
        assert_eq!(hex(COALBLACK), "#161313");
        assert_eq!(hex(OXBLOOD), "#6B1F35");
        assert_eq!(hex(WINEFLESH), "#982D52");
        assert_eq!(hex(OLD_GOLD), "#D2A53F");
        assert_eq!(hex(BRASS_LEAF), "#F0D27D");
        assert_eq!(hex(MAIN_TEAL), "#2A8482");
        assert_eq!(hex(ALARM_SCARLET), "#E63946");
        assert_eq!(hex(STONE_GREY), "#7A7064");
        assert_eq!(hex(INKBLOOD), "#1B0810");
        assert_eq!(hex(COBALT_DEEP), "#0E2E54");
        assert_eq!(hex(DEEP_TEAL), "#154548");
        assert_eq!(hex(PINE), "#27502E");
        assert_eq!(hex(BRONZE), "#5E4116");
        assert_eq!(hex(ABYSSAL), "#062123");
        assert_eq!(hex(BOG_UMBER), "#2D1F0A");
        assert_eq!(hex(MOSSBED), "#142A19");
        assert_eq!(hex(ROYAL_ARCANE), "#3A1559");
        assert_eq!(hex(MAGE_GLOW), "#9D6FE0");
    }

    /// The v2.1 syntax table, by role, exactly as the bible lists it.
    #[test]
    fn syntax_roles_match_bible_v2_1() {
        assert_eq!(SYNTAX_KEYWORD, WINEFLESH);
        assert_eq!(SYNTAX_TYPE, DEEP_TEAL);
        assert_eq!(SYNTAX_STRING, PINE);
        assert_eq!(SYNTAX_COMMENT, BRONZE);
        assert_eq!(SYNTAX_NUMBER, COBALT_DEEP);
        assert_eq!(SYNTAX_ERROR_TEXT, OXBLOOD);
        assert_eq!(ERROR_MARK, ALARM_SCARLET);
        assert_eq!(SYNTAX_PLAIN, COALBLACK);
        assert_eq!(CODE_CURSOR, ABYSSAL);
        assert_eq!(GUTTER, BRONZE);
        assert_eq!(CODE_PAGE, PARCHMENT_CREAM);
        assert_eq!(CODE_BLOCK_FRAME, BASALT);
    }
}
