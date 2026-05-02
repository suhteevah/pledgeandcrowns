// SPDX-License-Identifier: MIT
//! In-game code editor — toggled with `E`. Hand-rolled Rust syntax
//! highlighter on top of `egui::TextEdit::multiline`. Built this way
//! because `egui_code_editor` 0.2 pins egui 0.34 while `bevy_egui` 0.39
//! ships egui 0.33; the diamond is unfixable upstream as of 2026-05-02.
//!
//! Color roles match `design/03-art-style-bible.md` §UI Style — keyword
//! Wineflesh, type Main teal, string Forest, comment Stone grey, number
//! Old gold, error Alarm scarlet, cursor Abyssal.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

const STARTER_CODE: &str = r#"// Day-6 stub. Real curriculum modules land in Acts 1+.
fn main() {
    let greeting = "hello, world";
    println!("{greeting}");
}
"#;

#[derive(Resource)]
pub struct EditorState {
    pub open: bool,
    pub source: String,
    pub last_compile_result: Option<String>,
    /// Set by the editor UI when "Compile" is clicked. The
    /// `compile_client` plugin reads this each frame, dispatches the
    /// network request, and resets it.
    pub compile_pending: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            open: false,
            source: STARTER_CODE.to_string(),
            last_compile_result: None,
            compile_pending: false,
        }
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("EditorPlugin::build");
        app.add_plugins(EguiPlugin::default())
            .init_resource::<EditorState>()
            .add_systems(Update, toggle_editor)
            .add_systems(EguiPrimaryContextPass, draw_editor);
    }
}

fn toggle_editor(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<EditorState>) {
    if keys.just_pressed(KeyCode::KeyE) {
        state.open = !state.open;
        tracing::info!("editor toggled: open={}", state.open);
    }
}

fn draw_editor(mut contexts: EguiContexts, mut state: ResMut<EditorState>) {
    if !state.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Code Editor")
        .default_width(640.0)
        .default_height(420.0)
        .show(ctx, |ui| {
            ui.label("press E to close, Compile to evaluate");

            // Syntax-highlight layouter. egui 0.33's `Ui::fonts` closure
            // gives `&Fonts` but `Fonts::layout_job` takes `&mut self` in
            // this release — so we route through `Ctx::fonts_mut` instead
            // (https://docs.rs/egui/0.33/egui/struct.Context.html#method.fonts_mut).
            let style = ui.style().clone();
            let ctx = ui.ctx().clone();
            let mut layouter =
                move |_ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                    let mut job = highlight_rust(text.as_str(), &style);
                    job.wrap.max_width = wrap_width;
                    ctx.fonts_mut(|f| f.layout_job(job))
                };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut state.source)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });

            ui.horizontal(|ui| {
                if ui.button("Compile").clicked() {
                    tracing::info!("compile clicked ({} source bytes)", state.source.len());
                    state.compile_pending = true;
                }
                if let Some(msg) = &state.last_compile_result {
                    ui.label(msg);
                }
            });
        });
}

// === Hand-rolled Rust syntax highlighter ============================
// Tokenises the source into runs and emits one egui LayoutSection per run
// with palette colours sourced from Heraldic Code v2.0.

const KW_COLOR: egui::Color32 = egui::Color32::from_rgb(0x98, 0x2D, 0x52); // Wineflesh
const TYPE_COLOR: egui::Color32 = egui::Color32::from_rgb(0x2A, 0x84, 0x82); // Main teal
const STRING_COLOR: egui::Color32 = egui::Color32::from_rgb(0x48, 0x7E, 0x40); // Forest
const COMMENT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x7A, 0x70, 0x64); // Stone grey
const NUMBER_COLOR: egui::Color32 = egui::Color32::from_rgb(0xD2, 0xA5, 0x3F); // Old gold
const PLAIN_COLOR: egui::Color32 = egui::Color32::from_rgb(0x06, 0x21, 0x23); // Abyssal

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

fn highlight_rust(src: &str, style: &egui::Style) -> egui::text::LayoutJob {
    let font_id = egui::TextStyle::Monospace.resolve(style);
    let mut job = egui::text::LayoutJob::default();
    let bytes = src.as_bytes();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];

        // Line comment: //... to EOL
        if b == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            let start = i;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            push(&mut job, src, start..i, font_id.clone(), COMMENT_COLOR);
            continue;
        }

        // String literal: "..." with simple escape skipping
        if b == b'"' {
            let start = i;
            i += 1;
            while i < n && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < n {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < n {
                i += 1;
            }
            push(&mut job, src, start..i, font_id.clone(), STRING_COLOR);
            continue;
        }

        // Number literal: leading digit, then digits/underscore/dot/letter
        if b.is_ascii_digit() {
            let start = i;
            while i < n
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
            {
                i += 1;
            }
            push(&mut job, src, start..i, font_id.clone(), NUMBER_COLOR);
            continue;
        }

        // Identifier / keyword / type
        if b == b'_' || b.is_ascii_alphabetic() {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &src[start..i];
            let color = if KEYWORDS.contains(&word) {
                KW_COLOR
            } else if word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                TYPE_COLOR
            } else {
                PLAIN_COLOR
            };
            push(&mut job, src, start..i, font_id.clone(), color);
            continue;
        }

        // Anything else (whitespace, punctuation): one byte at a time as plain.
        let start = i;
        i += 1;
        push(&mut job, src, start..i, font_id.clone(), PLAIN_COLOR);
    }

    job
}

fn push(
    job: &mut egui::text::LayoutJob,
    src: &str,
    range: std::ops::Range<usize>,
    font_id: egui::FontId,
    color: egui::Color32,
) {
    job.append(
        &src[range],
        0.0,
        egui::TextFormat {
            font_id,
            color,
            ..Default::default()
        },
    );
}
