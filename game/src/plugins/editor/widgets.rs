// SPDX-License-Identifier: MIT
//! egui drawing helpers for the Cast Editor, all in the bible's UI
//! language: Parchment surfaces, Basalt frames, Coalblack text, Oxblood
//! accents, Aged-paper chips. Colours come only from [`palette`] (aliases
//! of the shared `ui_theme`); tutorial text is drawn by the shared
//! renderer (`ui_theme::markdown`), not here. The decisions about WHAT to
//! show live in the pure modules ([`diagnostics`],
//! [`layout`](super::layout)).

use bevy_egui::egui::{self, Color32, FontFamily, FontId, Rect, RichText, Stroke, text::LayoutJob};

use super::diagnostics::{self, Banner, BannerStyle, LineRole};
use super::palette;
use crate::plugins::ui_theme::{self, CodeTheme};

/// Body text size inside the editor (points): the theme's small size, so
/// the dense editor still never drops under the theme minimum.
pub const BODY_SIZE: f32 = ui_theme::SMALL_SIZE;
/// Monospace size for code and compiler output.
pub const MONO_SIZE: f32 = ui_theme::MIN_TEXT_SIZE;
/// Captions and key hints.
pub const CAPTION_SIZE: f32 = ui_theme::MIN_TEXT_SIZE;

pub fn mono_font() -> FontId {
    FontId::new(MONO_SIZE, FontFamily::Monospace)
}

/// Restyle a `Ui` (and its children) to the parchment look. Scoped to the
/// editor's own `Ui`, so other egui windows keep their style.
pub fn apply_parchment_style(style: &mut egui::Style) {
    let v = &mut style.visuals;
    v.dark_mode = false;
    v.override_text_color = Some(palette::TEXT);
    v.weak_text_color = Some(palette::TEXT_MUTED);
    v.panel_fill = palette::SURFACE;
    v.window_fill = palette::SURFACE;
    v.faint_bg_color = palette::SURFACE;
    v.extreme_bg_color = palette::PAGE_BG;
    v.text_edit_bg_color = Some(palette::PAGE_BG);
    v.code_bg_color = palette::CHIP_BG;
    v.window_stroke = Stroke::new(2.0_f32, palette::FRAME);
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.window_corner_radius = egui::CornerRadius::ZERO;
    v.selection.bg_fill = palette::HOVER_BG;
    v.selection.stroke = Stroke::new(1.0_f32, palette::TEXT);
    v.hyperlink_color = palette::ACCENT;
    v.text_cursor.stroke = Stroke::new(2.0_f32, palette::CURSOR);
    // Disabled widgets keep full opacity; they are styled explicitly.
    v.disabled_alpha = 1.0;
    let w = &mut v.widgets;
    for (wv, bg) in [
        (&mut w.noninteractive, palette::SURFACE),
        (&mut w.inactive, palette::CHIP_BG),
        (&mut w.hovered, palette::HOVER_BG),
        (&mut w.active, palette::PRESSED_BG),
        (&mut w.open, palette::CHIP_BG),
    ] {
        wv.bg_fill = bg;
        wv.weak_bg_fill = bg;
        wv.bg_stroke = Stroke::new(1.0_f32, palette::FRAME);
        wv.fg_stroke = Stroke::new(1.0_f32, palette::TEXT);
        wv.corner_radius = egui::CornerRadius::ZERO;
        wv.expansion = 0.0;
    }
    // The global theme's text styles are sized for panels; the editor is
    // denser. Body/button text drops to the editor body size (still at or
    // above the theme minimum).
    for (ts, size) in [
        (egui::TextStyle::Body, BODY_SIZE),
        (egui::TextStyle::Button, BODY_SIZE),
        (egui::TextStyle::Small, CAPTION_SIZE),
        (egui::TextStyle::Monospace, MONO_SIZE),
    ] {
        if let Some(font) = style.text_styles.get_mut(&ts) {
            font.size = size;
        }
    }
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    style.spacing.item_spacing = egui::vec2(8.0, 5.0);
    // Solid, always-drawn bars: the player must see that a pane scrolls.
    style.spacing.scroll = egui::style::ScrollStyle::solid();
}

/// The bible's double-line frame with square iron corners, filled with
/// Parchment. `rect` is the whole editor.
pub fn paint_window_frame(painter: &egui::Painter, rect: Rect) {
    let line = Stroke::new(2.0_f32, palette::FRAME);
    painter.rect_filled(rect, 0.0, palette::SURFACE);
    painter.rect_stroke(rect.shrink(1.0), 0.0, line, egui::StrokeKind::Middle);
    painter.rect_stroke(rect.shrink(5.0), 0.0, line, egui::StrokeKind::Middle);
    let corner = egui::vec2(8.0, 8.0);
    for c in [
        rect.left_top(),
        rect.right_top() - egui::vec2(corner.x, 0.0),
        rect.left_bottom() - egui::vec2(0.0, corner.y),
        rect.right_bottom() - corner,
    ] {
        painter.rect_filled(Rect::from_min_size(c, corner), 0.0, palette::FRAME);
    }
}

/// One inset pane: fill plus a 1 px Basalt border.
pub fn paint_pane(painter: &egui::Painter, rect: Rect, fill: Color32) {
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0_f32, palette::FRAME),
        egui::StrokeKind::Inside,
    );
}

/// Oxblood title bar: title on the left, key hints on the right.
pub fn title_bar(ui: &mut egui::Ui, title: &str, hint: &str) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, palette::ACCENT);
    ui.horizontal_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(title)
                .color(palette::TEXT_ON_DARK)
                .strong()
                .size(15.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new(hint)
                    .color(palette::TEXT_ON_DARK)
                    .size(CAPTION_SIZE),
            );
        });
    });
}

/// Small Oxblood section caption (GOAL, RESULT, ...).
pub fn caption(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .color(palette::ACCENT)
            .strong()
            .size(CAPTION_SIZE),
    );
}

/// Muted one-line note (button hints, copy feedback).
pub fn note(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        RichText::new(text.into())
            .color(palette::TEXT_MUTED)
            .size(CAPTION_SIZE),
    );
}

/// A button that is shown but cannot be pressed, with muted text and no
/// chip fill, so it reads as "not yet" rather than as broken.
pub fn disabled_button(ui: &mut egui::Ui, text: &str) {
    ui.add_enabled(
        false,
        egui::Button::new(RichText::new(text).color(palette::TEXT_MUTED))
            .fill(palette::SURFACE)
            .stroke(Stroke::new(1.0_f32, palette::FRAME)),
    );
}

/// Aged-paper inset chip with plain monospace text, wrapped to the pane.
pub fn code_chip(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(palette::CHIP_BG)
        .stroke(Stroke::new(1.0_f32, palette::FRAME))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(text).font(mono_font()).color(palette::TEXT)).wrap(),
            );
        });
}

/// The verdict banner: a coloured strip with the title, then the one-line
/// reason underneath. The reason goes through the tutorials' inline
/// markdown renderer, so `` `code` `` in a banner line or a server note
/// (e.g. "`unsafe` allowed") is drawn as an inline-code chip, not as raw
/// backticks.
pub fn draw_banner(ui: &mut egui::Ui, b: &Banner) {
    let (fill, fg) = match b.style {
        BannerStyle::Pass => (palette::BANNER_PASS, palette::TEXT_ON_DARK),
        BannerStyle::Unverified => (palette::BANNER_UNVERIFIED, palette::TEXT),
        BannerStyle::Fail => (palette::BANNER_FAIL, palette::TEXT_ON_DARK),
        BannerStyle::NotChecked => (palette::BANNER_NOT_CHECKED, palette::TEXT_ON_DARK),
        BannerStyle::Pending => (palette::BANNER_PENDING, palette::TEXT),
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, palette::FRAME))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(b.title).color(fg).strong().size(BODY_SIZE));
        });
    let scale = ui_theme::markdown::TextScale {
        body: BODY_SIZE,
        ..ui_theme::markdown::TextScale::PANEL
    };
    let job = ui_theme::markdown::inline_job(
        &ui_theme::markdown::parse_inline(&b.reason),
        scale,
        ui.available_width(),
    );
    ui.add(egui::Label::new(job).wrap());
}

/// Is `cleaned` rustc-shaped (headers, `-->` locations, ASCII art)?
pub fn is_rustc_like(cleaned: &str) -> bool {
    cleaned.lines().any(|l| {
        matches!(
            diagnostics::line_role(l),
            LineRole::ErrorHeader | LineRole::WarningHeader | LineRole::Location
        )
    })
}

/// The first rustc error line, wrapped, in error text. Drawn ABOVE the
/// scrolling output so it stays readable however the block is scrolled.
pub fn first_error(ui: &mut egui::Ui, line: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(line)
                .font(mono_font())
                .color(palette::ERROR_TEXT)
                .strong(),
        )
        .wrap(),
    );
}

/// The rustc output as one non-wrapping monospace block. rustc's ASCII
/// art (`|`, `^^^`, `-->`) assumes no wrapping, so lines are never broken;
/// the caller puts this inside a two-way `ScrollArea`. Each line role takes
/// its colour from the code surface `theme` (`out_*`): on ink, error
/// headers Pink quartz, `-->` Bright teal, `help:` Spring meadow, warnings
/// Brass leaf; on parchment (v2.1) Oxblood / muted / plain / Bronze.
pub fn rustc_job(cleaned: &str, theme: &CodeTheme) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (i, line) in cleaned.lines().enumerate() {
        if i > 0 {
            job.append(
                "\n",
                0.0,
                egui::TextFormat::simple(mono_font(), theme.out_plain),
            );
        }
        let color = match diagnostics::line_role(line) {
            LineRole::ErrorHeader => theme.out_error,
            LineRole::WarningHeader => theme.out_warning,
            LineRole::Location => theme.out_location,
            LineRole::Hint => theme.out_hint,
            LineRole::Plain => theme.out_plain,
        };
        job.append(line, 0.0, egui::TextFormat::simple(mono_font(), color));
    }
    job.wrap.max_width = f32::INFINITY;
    job
}

/// Compiler / grader output. rustc-shaped text is a non-wrapping block
/// ([`rustc_job`]), framed on the code page when the surface asks for it
/// (ink); anything else is wrapped monospace prose on the parchment pane.
pub fn draw_output(ui: &mut egui::Ui, cleaned: &str, theme: &CodeTheme) {
    if cleaned.trim().is_empty() {
        return;
    }
    if is_rustc_like(cleaned) {
        let job = rustc_job(cleaned, theme);
        if theme.framed_output {
            ui_theme::markdown::code_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add(egui::Label::new(job).extend());
            });
        } else {
            ui.add(egui::Label::new(job).extend());
        }
    } else {
        ui.add(
            egui::Label::new(
                RichText::new(cleaned)
                    .font(mono_font())
                    .color(palette::TEXT),
            )
            .wrap(),
        );
    }
}

/// Expected vs actual, stacked (the result column is narrow).
pub fn draw_expected_actual(ui: &mut egui::Ui, expected: Option<&str>, actual: Option<&str>) {
    if let Some(e) = expected {
        caption(ui, "EXPECTED");
        code_chip(ui, e);
    }
    if let Some(a) = actual {
        caption(ui, "YOUR PROGRAM GAVE");
        code_chip(ui, if a.is_empty() { "(nothing)" } else { a });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustc_job_never_wraps_and_keeps_every_line() {
        let out = "error[E0384]: cannot assign twice\n --> src/main.rs:3:5\n  |\n3 |     answer = 43;\n  |     ^^^^^^^^^^^ cannot assign twice to immutable variable";
        for theme in [&ui_theme::INK_CODE, &ui_theme::PARCHMENT_CODE] {
            let job = rustc_job(out, theme);
            assert_eq!(job.text, out);
            assert!(job.wrap.max_width.is_infinite());
            assert_eq!(job.sections[0].format.color, theme.out_error);
        }
        let ink = rustc_job(out, &ui_theme::INK_CODE);
        let colour_of = |needle: &str| {
            ink.sections
                .iter()
                .find(|s| ink.text[s.byte_range.clone()].contains(needle))
                .map(|s| s.format.color)
        };
        assert_eq!(colour_of("-->"), Some(ui_theme::BRIGHT_TEAL));
        assert_eq!(colour_of("answer = 43"), Some(ui_theme::PARCHMENT_CREAM));
        assert!(is_rustc_like(out));
        assert!(!is_rustc_like("Ferris clicks approvingly."));
    }

    #[test]
    fn editor_sizes_respect_the_theme_minimum() {
        for s in [BODY_SIZE, MONO_SIZE, CAPTION_SIZE] {
            assert!(s >= ui_theme::MIN_TEXT_SIZE);
        }
    }
}
