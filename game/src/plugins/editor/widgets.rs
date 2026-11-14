// SPDX-License-Identifier: MIT
//! egui drawing helpers for the Cast Editor, all in the bible's UI
//! language: Parchment surfaces, Basalt frames, Coalblack text, Oxblood
//! accents, Aged-paper chips. Colours come only from [`palette`]; the
//! decisions about WHAT to show live in the pure modules
//! ([`diagnostics`], [`markdown`], [`layout`](super::layout)).

use bevy_egui::egui::{self, Color32, FontFamily, FontId, Rect, RichText, Stroke, text::LayoutJob};

use super::diagnostics::{self, Banner, BannerStyle, LineRole};
use super::markdown::{self, Block, Span};
use super::palette;

/// Body text size inside the editor (points).
pub const BODY_SIZE: f32 = 13.5;
/// Monospace size for code chips and compiler output.
pub const MONO_SIZE: f32 = 13.0;

fn body_font() -> FontId {
    FontId::new(BODY_SIZE, FontFamily::Proportional)
}

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
    v.text_cursor.stroke = Stroke::new(2.0_f32, palette::PLAIN);
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
            ui.label(RichText::new(hint).color(palette::TEXT_ON_DARK).size(12.0));
        });
    });
}

/// Small Oxblood section caption (GOAL, RESULT, ...).
pub fn caption(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .color(palette::ACCENT)
            .strong()
            .size(12.0),
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

/// Aged-paper inset chip with monospace text, wrapped to the pane.
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

/// Render a tutorial (markdown subset) into the goal pane.
pub fn draw_markdown(ui: &mut egui::Ui, src: &str) {
    for block in markdown::parse(src) {
        match block {
            Block::Heading { level, text } => {
                ui.add_space(6.0);
                let size = match level {
                    1 => 18.0,
                    2 => 16.0,
                    _ => 14.5,
                };
                ui.label(
                    RichText::new(text)
                        .color(palette::ACCENT)
                        .strong()
                        .size(size),
                );
            }
            Block::Paragraph(spans) => inline(ui, "", &spans),
            Block::Bullet(spans) => inline(ui, "•  ", &spans),
            Block::Code(code) => {
                ui.add_space(2.0);
                code_chip(ui, &code);
                ui.add_space(2.0);
            }
        }
    }
}

fn inline(ui: &mut egui::Ui, prefix: &str, spans: &[Span]) {
    let mut job = LayoutJob::default();
    let text = egui::TextFormat {
        font_id: body_font(),
        color: palette::TEXT,
        ..Default::default()
    };
    if !prefix.is_empty() {
        job.append(prefix, 0.0, text.clone());
    }
    for span in spans {
        match span {
            Span::Text(s) => job.append(s, 0.0, text.clone()),
            Span::Code(s) => job.append(
                s,
                0.0,
                egui::TextFormat {
                    font_id: mono_font(),
                    color: palette::TEXT,
                    background: palette::CHIP_BG,
                    ..Default::default()
                },
            ),
            Span::Emph(s) => job.append(
                s,
                0.0,
                egui::TextFormat {
                    italics: true,
                    ..text.clone()
                },
            ),
            Span::Strong(s) => job.append(
                s,
                0.0,
                egui::TextFormat {
                    color: palette::ACCENT,
                    ..text.clone()
                },
            ),
        }
    }
    job.wrap.max_width = ui.available_width();
    ui.add(egui::Label::new(job).wrap());
}

/// The verdict banner: a coloured strip with the title, then the plain
/// one-line reason underneath.
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
            ui.label(RichText::new(b.title).color(fg).strong().size(14.0));
        });
    ui.add(
        egui::Label::new(
            RichText::new(&b.reason)
                .color(palette::TEXT)
                .size(BODY_SIZE - 0.5),
        )
        .wrap(),
    );
}

/// Compiler / grader output. rustc-shaped text is monospace with error
/// headers (and their `E0xxx` code) in Oxblood; prose is body text.
pub fn draw_output(ui: &mut egui::Ui, cleaned: &str) {
    if cleaned.is_empty() {
        return;
    }
    let rustc_like = cleaned.lines().any(|l| {
        matches!(
            diagnostics::line_role(l),
            LineRole::ErrorHeader | LineRole::WarningHeader | LineRole::Location
        )
    });
    if !rustc_like {
        ui.add(
            egui::Label::new(
                RichText::new(cleaned)
                    .font(mono_font())
                    .color(palette::TEXT),
            )
            .wrap(),
        );
        return;
    }
    let mut job = LayoutJob::default();
    let mut first = true;
    for line in cleaned.lines() {
        if !first {
            job.append(
                "\n",
                0.0,
                egui::TextFormat::simple(mono_font(), palette::TEXT),
            );
        }
        first = false;
        let color = match diagnostics::line_role(line) {
            LineRole::ErrorHeader => palette::ACCENT,
            LineRole::WarningHeader => palette::GUTTER,
            LineRole::Location => palette::TEXT_MUTED,
            LineRole::Hint | LineRole::Plain => palette::TEXT,
        };
        job.append(line, 0.0, egui::TextFormat::simple(mono_font(), color));
    }
    job.wrap.max_width = ui.available_width();
    ui.add(egui::Label::new(job).wrap());
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
