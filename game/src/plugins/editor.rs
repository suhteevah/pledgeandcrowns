// SPDX-License-Identifier: MIT
//! In-game code editor — toggled with `E`. Hand-rolled Rust syntax
//! highlighter on top of `egui::TextEdit::multiline`. Built this way
//! because `egui_code_editor` 0.2 pins egui 0.34 while `bevy_egui` 0.39
//! ships egui 0.33; the diamond is unfixable upstream as of 2026-05-02.
//!
//! Color roles match `design/03-art-style-bible.md` §UI Style — keyword
//! Wineflesh, type Main teal, string Forest, comment Stone grey, number
//! Old gold, error Alarm scarlet, cursor Abyssal.

use crate::plugins::mission::ActiveMission;
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
    /// Mission identifier sent to the compile-API so it can branch
    /// validation per encounter. Set by the mission system on F-press;
    /// defaults to the freeform sandbox encounter.
    pub encounter_id: String,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            open: false,
            source: STARTER_CODE.to_string(),
            last_compile_result: None,
            compile_pending: false,
            encounter_id: "freeform".to_string(),
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

fn draw_editor(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    active: Res<ActiveMission>,
) {
    if !state.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Code Editor")
        .default_width(960.0)
        .default_height(540.0)
        .show(ctx, |ui| {
            ui.label("press E to close, Compile to evaluate");
            ui.separator();

            egui::SidePanel::left("tutorial_panel")
                .resizable(true)
                .default_width(360.0)
                .show_inside(ui, |ui| {
                    if let Some(mission) = active.current.as_ref() {
                        ui.heading(mission.npc_name);
                        ui.label(mission.prompt);
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| draw_tutorial(ui, mission.tutorial));
                    } else {
                        ui.heading("Freeform sandbox");
                        ui.label(
                            "Walk up to an NPC and press F to load a mission. \
                             Anything you submit here goes through the freeform \
                             grader, which only echoes byte counts.",
                        );
                    }
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                let style = ui.style().clone();
                let ctx_clone = ui.ctx().clone();
                let mut layouter =
                    move |_ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                        let mut job = highlight_rust(text.as_str(), &style);
                        job.wrap.max_width = wrap_width;
                        ctx_clone.fonts_mut(|f| f.layout_job(job))
                    };

                egui::ScrollArea::vertical()
                    .id_salt("source_scroll")
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut state.source)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_rows(18)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        );
                    });

                ui.separator();
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
        });
}

/// Lightweight tutorial renderer. Recognises `## Heading` lines and
/// triple-backtick code fences; everything else is wrapped body text.
fn draw_tutorial(ui: &mut egui::Ui, body: &str) {
    let mut in_code = false;
    let mut code_buf = String::new();
    for line in body.split('\n') {
        if line.trim() == "```" {
            if in_code {
                ui.add(
                    egui::TextEdit::multiline(&mut code_buf.clone())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
                code_buf.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }
        if in_code {
            if !code_buf.is_empty() {
                code_buf.push('\n');
            }
            code_buf.push_str(line);
        } else if let Some(rest) = line.strip_prefix("## ") {
            ui.add_space(6.0);
            ui.heading(rest);
        } else if line.is_empty() {
            ui.add_space(4.0);
        } else {
            ui.label(line);
        }
    }
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
