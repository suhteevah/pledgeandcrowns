// SPDX-License-Identifier: MIT
//! In-game Cast Editor (`design/02-mechanics.md` §The Cast Editor).
//!
//! Layout: toolbar (Compile / Reset to starter / Show solution) on top,
//! code page with a line-number gutter in the centre, and a right-hand
//! column holding the scrollable goal pane above the result pane.
//!
//! Keys: `E` opens the editor when a mission is active, `Esc` closes it,
//! `Ctrl+Enter` compiles. Other plugins read `EditorState::open` (audio,
//! player movement, pause handling); that field's meaning is unchanged.
//!
//! Colours come only from [`palette`] (bible §UI Style). Pure logic
//! lives in the submodules so it can be unit-tested without a renderer.

pub mod diagnostics;
pub mod highlight;
pub mod palette;
pub mod session;
pub mod solutions;

use crate::plugins::mission::{ActiveMission, Mission};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use diagnostics::{ParsedResult, ResultKind, SERVER_PREAMBLE_LINES};
use session::{FailureTracker, SolutionGate};
use std::collections::BTreeSet;

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
        // No starter text: the buffer is only ever filled from a
        // mission's `starter_code` (mission.rs F-press, or `E` below).
        Self {
            open: false,
            source: String::new(),
            last_compile_result: None,
            compile_pending: false,
            encounter_id: "freeform".to_string(),
        }
    }
}

impl EditorState {
    /// True while the editor window is showing. Stable accessor for
    /// other plugins (pause handling); equivalent to reading `open`.
    pub fn is_open(&self) -> bool {
        self.open
    }
}

/// Editor-private session state: failure counts, the in-flight compile
/// being watched, and transient UI toggles.
#[derive(Resource, Default)]
pub struct EditorSession {
    tracker: FailureTracker,
    /// Mission id of the compile we're waiting on, if any. Captured at
    /// request time so a late result is attributed to the right mission.
    awaiting_result_for: Option<String>,
    confirm_reset: bool,
    show_solution: bool,
    /// Encounter the transient toggles belong to; a change clears them.
    ui_encounter: String,
}

impl EditorSession {
    /// Failed compiles recorded for `mission_id` this session.
    pub fn failures(&self, mission_id: &str) -> u32 {
        self.tracker.failures(mission_id)
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("EditorPlugin::build");
        // bevy_egui 0.37+ defaults to a bindless texture array (16 slots)
        // which requires `TEXTURE_BINDING_ARRAY`. WebGL2 doesn't have it,
        // so on wasm wgpu refuses to create the bind-group layout — and
        // that failure cascades into the wider `Image` GPU upload path,
        // killing all image-backed sprite rendering even though the
        // assets fetch fine. `None` falls back to per-texture bind groups,
        // which WebGL2 supports. Native works either way; doing this
        // unconditionally keeps the build target-agnostic.
        app.add_plugins(EguiPlugin {
            bindless_mode_array_size: None,
            ..EguiPlugin::default()
        })
        .init_resource::<EditorState>()
        .init_resource::<EditorSession>()
        .add_systems(Update, (toggle_editor, track_compile_results))
        .add_systems(EguiPrimaryContextPass, draw_editor);
    }
}

fn toggle_editor(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    active: Res<ActiveMission>,
) {
    // E opens the editor, Esc closes it. The previous E-toggles-both
    // behaviour meant typing the letter `e` in your code closed the
    // window mid-edit (Matt caught this on day-N). Now the close path
    // is Esc-only and the open path doesn't fire while the editor is
    // already open, so player typing can't accidentally toggle.
    if !state.open && keys.just_pressed(KeyCode::KeyE) {
        let Some(mission) = active.current.as_ref() else {
            tracing::debug!("E pressed with no active mission; editor stays closed");
            return;
        };
        if state.encounter_id != mission.id {
            // Active mission changed while the editor was closed (e.g.
            // the completion recap): load its starter so the code and
            // the goal pane agree.
            tracing::debug!(
                "editor buffer belonged to `{}`; loading starter for `{}`",
                state.encounter_id,
                mission.id
            );
            state.source = mission.starter_code.to_string();
            state.encounter_id = mission.id.to_string();
            state.last_compile_result = None;
        }
        state.open = true;
        tracing::info!("editor opened for mission {}", mission.id);
    } else if state.open && keys.just_pressed(KeyCode::Escape) {
        state.open = false;
        tracing::info!("editor closed via Escape");
    }
}

/// Watch for the result of a compile the editor requested and feed its
/// verdict into the failure counter.
fn track_compile_results(state: Res<EditorState>, mut session: ResMut<EditorSession>) {
    let Some(mission_id) = session.awaiting_result_for.clone() else {
        return;
    };
    let Some(raw) = state.last_compile_result.as_deref() else {
        return;
    };
    let kind = diagnostics::parse_result(raw).kind;
    if kind == ResultKind::Pending {
        return;
    }
    let failures = session.tracker.record(&mission_id, kind);
    session.awaiting_result_for = None;
    tracing::debug!(
        "compile verdict for {mission_id}: {kind:?} (failures this session: {failures})"
    );
}

/// Shared by the Compile button and Ctrl+Enter.
fn request_compile(state: &mut EditorState, session: &mut EditorSession) {
    tracing::info!(
        "compile requested for {} ({} source bytes)",
        state.encounter_id,
        state.source.len()
    );
    state.compile_pending = true;
    // Clear the previous verdict so `track_compile_results` can't mistake
    // it for the new one (the wasm path never shows `compiling…`).
    state.last_compile_result = None;
    session.awaiting_result_for = Some(state.encounter_id.clone());
}

fn draw_editor(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    mut session: ResMut<EditorSession>,
    active: Res<ActiveMission>,
) {
    if !state.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        tracing::debug!("draw_editor: no egui context this frame");
        return;
    };
    let state = &mut *state;
    let session = &mut *session;

    if session.ui_encounter != state.encounter_id {
        tracing::debug!(
            "editor encounter changed {} -> {}; clearing reset/solution toggles",
            session.ui_encounter,
            state.encounter_id
        );
        session.ui_encounter = state.encounter_id.clone();
        session.confirm_reset = false;
        session.show_solution = false;
    }

    let mission = active
        .current
        .as_ref()
        .filter(|m| m.id == state.encounter_id);

    let frame =
        egui::Frame::window(&ctx.style()).stroke(egui::Stroke::new(2.0_f32, palette::FRAME));
    egui::Window::new("Code Editor")
        .frame(frame)
        .default_width(1040.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            let Some(mission) = mission else {
                draw_empty_state(ui);
                return;
            };
            // Consume Ctrl+Enter before the TextEdit sees it, so it
            // compiles instead of inserting a newline.
            let mut compile_now = ui
                .ctx()
                .input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter));
            if compile_now {
                tracing::debug!("Ctrl+Enter pressed");
            }

            let solution = solutions::canonical_solution(mission.id);
            let gate = session::solution_gate(session.failures(mission.id), solution);
            if !matches!(gate, SolutionGate::Available(_)) {
                session.show_solution = false;
            }

            egui::TopBottomPanel::top("editor_toolbar").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Compile").on_hover_text("Ctrl+Enter").clicked() {
                        compile_now = true;
                    }
                    draw_reset_controls(ui, state, session, mission.starter_code);
                    draw_solution_button(ui, session, gate);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.weak("Esc closes  ·  Ctrl+Enter compiles");
                    });
                });
            });

            let parsed = state
                .last_compile_result
                .as_deref()
                .map(diagnostics::parse_result);
            let error_lines: BTreeSet<usize> = match parsed.as_ref() {
                Some(p) if p.kind != ResultKind::Pass => {
                    diagnostics::error_lines(p.body, SERVER_PREAMBLE_LINES)
                        .into_iter()
                        .collect()
                }
                _ => BTreeSet::new(),
            };

            egui::SidePanel::right("goal_panel")
                .resizable(true)
                .default_width(380.0)
                .show_inside(ui, |ui| {
                    egui::TopBottomPanel::bottom("result_panel")
                        .resizable(true)
                        .default_height(180.0)
                        .show_inside(ui, |ui| draw_result(ui, parsed.as_ref()));
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        let revealed = match gate {
                            SolutionGate::Available(s) if session.show_solution => Some(s),
                            _ => None,
                        };
                        draw_goal(ui, mission, revealed);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                draw_code(ui, &mut state.source, &error_lines);
            });

            if compile_now {
                request_compile(state, session);
            }
        });
}

fn draw_empty_state(ui: &mut egui::Ui) {
    ui.heading("No mission loaded");
    ui.label("Walk up to an NPC and press F to start a mission. Esc closes this window.");
}

fn draw_reset_controls(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    session: &mut EditorSession,
    starter: &str,
) {
    if session.confirm_reset {
        ui.colored_label(palette::ERROR, "Discard your changes?");
        if ui.button("Yes, reset").clicked() {
            tracing::info!("reset to starter confirmed for {}", state.encounter_id);
            state.source = starter.to_string();
            session.confirm_reset = false;
        }
        if ui.button("Cancel").clicked() {
            tracing::debug!("reset to starter cancelled");
            session.confirm_reset = false;
        }
    } else if ui.button("Reset to starter").clicked() {
        if session::reset_needs_confirm(&state.source, starter) {
            tracing::debug!("reset requested on edited buffer; asking to confirm");
            session.confirm_reset = true;
        } else {
            tracing::debug!("reset requested but buffer already matches starter");
        }
    }
}

fn draw_solution_button(ui: &mut egui::Ui, session: &mut EditorSession, gate: SolutionGate<'_>) {
    match gate {
        SolutionGate::Available(_) => {
            let label = if session.show_solution {
                "Hide solution"
            } else {
                "Show solution"
            };
            if ui.button(label).clicked() {
                session.show_solution = !session.show_solution;
                tracing::info!("solution visibility toggled to {}", session.show_solution);
            }
        }
        SolutionGate::Unauthored => {
            ui.add_enabled(false, egui::Button::new("Show solution"))
                .on_disabled_hover_text(
                    "No reference solution has been written for this mission yet.",
                );
        }
        SolutionGate::Locked { remaining } => {
            ui.add_enabled(false, egui::Button::new("Show solution"))
                .on_disabled_hover_text(format!(
                    "Unlocks after {} failed compiles on this mission ({remaining} to go).",
                    session::SOLUTION_UNLOCK_FAILURES
                ));
        }
    }
}

fn draw_goal(ui: &mut egui::Ui, mission: &Mission, solution: Option<&str>) {
    egui::ScrollArea::vertical()
        .id_salt("goal_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.heading(mission.npc_name);
            ui.strong("GOAL");
            ui.label(mission.prompt);
            ui.separator();
            draw_tutorial(ui, mission.tutorial);
            if let Some(code) = solution {
                ui.separator();
                ui.strong("REFERENCE SOLUTION");
                ui.add(
                    egui::TextEdit::multiline(&mut code.to_string())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
            }
        });
}

fn draw_result(ui: &mut egui::Ui, parsed: Option<&ParsedResult<'_>>) {
    ui.strong("COMPILER OUTPUT");
    let Some(p) = parsed else {
        ui.weak("(no output yet: press Compile or Ctrl+Enter)");
        return;
    };
    let (heading, color) = match p.kind {
        ResultKind::Pass => ("PASS", palette::PASS),
        ResultKind::Fail => ("FAIL", palette::ERROR),
        ResultKind::Pending => ("COMPILING…", palette::COMMENT),
        ResultKind::Error => ("ERROR", palette::ERROR),
    };
    ui.horizontal(|ui| {
        ui.colored_label(color, egui::RichText::new(heading).strong());
        if p.offline {
            ui.weak("offline grader (server unreachable)");
        }
    });
    if p.kind != ResultKind::Pending {
        egui::ScrollArea::vertical()
            .id_salt("result_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add(egui::Label::new(egui::RichText::new(p.body).monospace()).wrap());
            });
    }
}

/// Code page: Parchment fill, Basalt frame, line-number gutter with
/// error lines in Alarm scarlet, and the highlighted TextEdit. Wrapping
/// is off so each gutter row lines up with exactly one source line.
fn draw_code(ui: &mut egui::Ui, source: &mut String, error_lines: &BTreeSet<usize>) {
    {
        let visuals = ui.visuals_mut();
        visuals.extreme_bg_color = palette::PAGE_BG;
        visuals.text_cursor.stroke.color = palette::PLAIN;
    }
    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
    let gutter = gutter_job(source, &font_id, error_lines);

    egui::Frame::new()
        .fill(palette::PAGE_BG)
        .stroke(egui::Stroke::new(1.0_f32, palette::FRAME))
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("source_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            // Match the TextEdit's top margin (2 px).
                            ui.add_space(2.0);
                            ui.add(egui::Label::new(gutter).selectable(false));
                        });
                        let layout_font = font_id.clone();
                        let errs = error_lines.clone();
                        let mut layouter =
                            move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap_width: f32| {
                                let job = highlight::layout_job(
                                    text.as_str(),
                                    layout_font.clone(),
                                    &errs,
                                );
                                ui.ctx().fonts_mut(|f| f.layout_job(job))
                            };
                        ui.add(
                            egui::TextEdit::multiline(source)
                                .code_editor()
                                .margin(egui::Margin::symmetric(4, 2))
                                .desired_rows(18)
                                .desired_width(ui.available_width().max(200.0))
                                .layouter(&mut layouter),
                        );
                    });
                });
        });
}

fn gutter_job(
    source: &str,
    font_id: &egui::FontId,
    error_lines: &BTreeSet<usize>,
) -> egui::text::LayoutJob {
    let line_count = source.matches('\n').count() + 1;
    let width = line_count.to_string().len();
    let mut job = egui::text::LayoutJob::default();
    for n in 1..=line_count {
        let is_err = error_lines.contains(&n);
        let text = if n == line_count {
            format!("{n:>width$}")
        } else {
            format!("{n:>width$}\n")
        };
        job.append(
            &text,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: if is_err {
                    palette::ERROR
                } else {
                    palette::GUTTER
                },
                ..Default::default()
            },
        );
    }
    job
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
