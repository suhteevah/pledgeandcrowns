// SPDX-License-Identifier: MIT
//! In-game Cast Editor (`design/02-mechanics.md` §The Cast Editor).
//!
//! Layout ([`layout`]): a fixed fraction of the screen, centred, with an
//! Oxblood title bar, a toolbar (Compile / Reset to starter / Show
//! solution), the code page on the left, and a right-hand column holding
//! the goal pane (or the revealed solution) above the result pane. Every
//! pane has a fixed rectangle and scrolls internally, so the editor never
//! grows past the window edge.
//!
//! Keys: `E` opens the editor when a mission is active, `Esc` closes it,
//! `Ctrl+Enter` compiles. Other plugins read `EditorState::open` (audio,
//! player movement, pause handling); that field's meaning is unchanged.
//!
//! Colours come only from [`palette`], aliases of the shared `ui_theme`
//! (bible §UI Style, v2.1). Tutorial text, fenced code and the revealed
//! solution are drawn by the shared `ui_theme::markdown` /
//! `ui_theme::highlight`. Pure logic lives in the submodules so it can be
//! unit-tested without a renderer; egui drawing helpers are in [`widgets`].

pub mod diagnostics;
pub mod layout;
pub mod palette;
pub mod session;
pub mod solutions;
pub mod widgets;

use crate::plugins::mission::{ActiveMission, Mission};
use crate::plugins::ui_theme::{highlight, markdown};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use diagnostics::{ParsedResult, ResultDetail, ResultKind, ResultTag, SERVER_PREAMBLE_LINES};
use session::{FailureTracker, SolutionGate};
use std::collections::BTreeSet;

#[derive(Resource)]
pub struct EditorState {
    pub open: bool,
    pub source: String,
    pub last_compile_result: Option<String>,
    /// Structured extras (server note, expected/actual) for the result in
    /// `last_compile_result`. Ignored unless its `for_result` matches that
    /// string, so a plugin that clears or replaces only the string can
    /// never show stale extras.
    pub last_result_detail: Option<ResultDetail>,
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
            last_result_detail: None,
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

    /// Show a verdict: the editor string plus its structured extras.
    pub fn set_result(&mut self, formatted: String, detail: ResultDetail) {
        tracing::debug!(
            bytes = formatted.len(),
            note = detail.note.is_some(),
            expected = detail.expected.is_some(),
            "editor: result set"
        );
        self.last_compile_result = Some(formatted);
        self.last_result_detail = Some(detail);
    }

    /// The extras for the current result, if they belong to it.
    pub fn current_detail(&self) -> Option<&ResultDetail> {
        let current = self.last_compile_result.as_deref()?;
        self.last_result_detail
            .as_ref()
            .filter(|d| d.for_result == current)
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
    /// The solution pane replaces the goal pane while this is set.
    show_solution: bool,
    /// "Copy into editor" asked to confirm overwriting an edited buffer.
    confirm_copy: bool,
    /// One-line feedback under the solution buttons.
    copy_note: Option<&'static str>,
    /// Encounter the transient toggles belong to; a change clears them.
    ui_encounter: String,
}

impl EditorSession {
    /// Failed compiles recorded for `mission_id` this session.
    pub fn failures(&self, mission_id: &str) -> u32 {
        self.tracker.failures(mission_id)
    }

    /// Scripted-capture hook (`screenshot` plugin): mark a compile for
    /// `mission_id` as in flight, exactly as the Compile button does,
    /// without dispatching a network request. The next verdict written to
    /// `EditorState::last_compile_result` is then counted by the normal
    /// `track_compile_results` system. Never called in normal play.
    pub fn note_compile_requested(&mut self, mission_id: &str) {
        tracing::debug!("scripted: compile marked in flight for {mission_id}");
        self.awaiting_result_for = Some(mission_id.to_string());
    }

    /// Scripted-capture hook (`screenshot` plugin): show the reference
    /// solution for `encounter_id`, as clicking "Show solution" does. The
    /// editor still hides it unless the solution gate is open (3 failed
    /// compiles), so this cannot reveal anything the player couldn't.
    /// Never called in normal play.
    pub fn reveal_solution(&mut self, encounter_id: &str) {
        tracing::debug!("scripted: reveal solution for {encounter_id}");
        self.ui_encounter = encounter_id.to_string();
        self.clear_toggles();
        self.show_solution = true;
    }

    /// Scripted-capture hook (`screenshot` plugin): back to the goal pane
    /// with no pending confirmations, as on a fresh mission. Never called
    /// in normal play.
    pub fn reset_view(&mut self) {
        tracing::debug!("scripted: editor view reset");
        self.clear_toggles();
    }

    fn clear_toggles(&mut self) {
        self.confirm_reset = false;
        self.show_solution = false;
        self.confirm_copy = false;
        self.copy_note = None;
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
    state.last_result_detail = None;
    session.awaiting_result_for = Some(state.encounter_id.clone());
}

/// A child `Ui` confined (and clipped) to `rect`, with `margin` padding.
fn pane_ui(ui: &mut egui::Ui, rect: egui::Rect, margin: f32) -> egui::Ui {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(margin))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child
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
        session.clear_toggles();
    }

    let mission = active
        .current
        .as_ref()
        .filter(|m| m.id == state.encounter_id);

    let screen = ctx.content_rect();
    let has_result = state.last_compile_result.is_some();
    let lay = layout::editor_layout(screen.size(), has_result);
    let offset = screen.min.to_vec2();
    let r = |rect: egui::Rect| rect.translate(offset);

    egui::Area::new(egui::Id::new("cast_editor"))
        .order(egui::Order::Middle)
        .fixed_pos(r(lay.window).min)
        .constrain(false)
        .show(ctx, |ui| {
            widgets::apply_parchment_style(ui.style_mut());
            let (window, _) = ui.allocate_exact_size(lay.window.size(), egui::Sense::hover());
            widgets::paint_window_frame(ui.painter(), window);

            let npc = mission.map_or("", |m| m.npc_name);
            let title = if npc.is_empty() {
                "CAST EDITOR".to_string()
            } else {
                format!("CAST EDITOR  ·  {npc}")
            };
            let mut title_ui = pane_ui(ui, r(lay.title), 0.0);
            widgets::title_bar(&mut title_ui, &title, "Esc closes  ·  Ctrl+Enter compiles");

            let Some(mission) = mission else {
                let mut body = pane_ui(ui, r(lay.code).union(r(lay.result)), 12.0);
                draw_empty_state(&mut body);
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
            let revealed = match gate {
                SolutionGate::Available(s) if session.show_solution => Some(s),
                _ => {
                    session.show_solution = false;
                    None
                }
            };

            // Snapshot the result: the panes below may edit `state`.
            let result_raw = state.last_compile_result.clone();
            let detail = state.current_detail().cloned();
            let parsed = result_raw.as_deref().map(diagnostics::parse_result);
            let error_lines: BTreeSet<usize> = match parsed.as_ref() {
                Some(p) if p.kind != ResultKind::Pass => {
                    diagnostics::error_lines(p.body, SERVER_PREAMBLE_LINES)
                        .into_iter()
                        .collect()
                }
                _ => BTreeSet::new(),
            };

            let mut bar = pane_ui(ui, r(lay.toolbar), 2.0);
            bar.horizontal_centered(|ui| {
                let compile = egui::Button::new(
                    egui::RichText::new("Compile")
                        .color(palette::TEXT_ON_DARK)
                        .strong(),
                )
                .fill(palette::ACCENT);
                if ui.add(compile).clicked() {
                    compile_now = true;
                }
                draw_reset_controls(ui, state, session, mission.starter_code);
                draw_solution_button(ui, session, gate);
            });

            widgets::paint_pane(ui.painter(), r(lay.code), palette::PAGE_BG);
            let mut code_ui = pane_ui(ui, r(lay.code), 2.0);
            draw_code(&mut code_ui, &mut state.source, &error_lines);

            widgets::paint_pane(ui.painter(), r(lay.goal), palette::SURFACE);
            let mut goal_ui = pane_ui(ui, r(lay.goal), 8.0);
            match revealed {
                Some(code) => draw_solution(&mut goal_ui, state, session, code),
                None => draw_goal(&mut goal_ui, mission),
            }

            widgets::paint_pane(ui.painter(), r(lay.result), palette::SURFACE);
            let mut result_ui = pane_ui(ui, r(lay.result), 8.0);
            draw_result(&mut result_ui, parsed.as_ref(), detail.as_ref());

            if compile_now {
                request_compile(state, session);
            }
        });
}

fn draw_empty_state(ui: &mut egui::Ui) {
    widgets::caption(ui, "NO MISSION LOADED");
    ui.label("Walk up to an NPC and press F to start a mission. Esc closes this window.");
}

fn draw_reset_controls(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    session: &mut EditorSession,
    starter: &str,
) {
    if session.confirm_reset {
        ui.label(
            egui::RichText::new("Discard your changes?")
                .color(palette::ACCENT)
                .strong(),
        );
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
                "Back to goal"
            } else {
                "Show solution"
            };
            if ui.button(label).clicked() {
                session.show_solution = !session.show_solution;
                session.confirm_copy = false;
                session.copy_note = None;
                tracing::info!("solution pane toggled to {}", session.show_solution);
            }
        }
        SolutionGate::Unauthored => {
            widgets::disabled_button(ui, "Show solution");
            widgets::note(ui, "No reference solution for this mission yet.");
        }
        SolutionGate::Locked { remaining } => {
            widgets::disabled_button(ui, "Show solution");
            let plural = if remaining == 1 { "" } else { "s" };
            widgets::note(
                ui,
                format!("Unlocks after {remaining} more failed compile{plural}."),
            );
        }
    }
}

fn draw_goal(ui: &mut egui::Ui, mission: &Mission) {
    egui::ScrollArea::vertical()
        .id_salt("goal_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            widgets::caption(ui, "GOAL");
            markdown::show_scaled(ui, mission.prompt, markdown::TextScale::COMPACT);
            ui.add_space(4.0);
            ui.separator();
            markdown::show_scaled(ui, mission.tutorial, markdown::TextScale::COMPACT);
        });
}

/// The revealed reference solution, in place of the goal pane, with
/// "Copy into editor" (confirming before it overwrites edited code) and
/// "Back to goal".
fn draw_solution(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    session: &mut EditorSession,
    code: &str,
) {
    widgets::caption(ui, "REFERENCE SOLUTION");
    ui.horizontal_wrapped(|ui| {
        if session.confirm_copy {
            ui.label(
                egui::RichText::new("Replace your code with this?")
                    .color(palette::ACCENT)
                    .strong(),
            );
            if ui.button("Yes, replace").clicked() {
                tracing::info!("solution copied into editor for {}", state.encounter_id);
                state.source = code.to_string();
                session.confirm_copy = false;
                session.copy_note = Some("Copied into the editor. Press Compile to check it.");
            }
            if ui.button("Cancel").clicked() {
                tracing::debug!("copy solution cancelled");
                session.confirm_copy = false;
            }
        } else {
            if ui.button("Copy into editor").clicked() {
                if session::reset_needs_confirm(&state.source, code) {
                    tracing::debug!("copy solution over edited buffer; asking to confirm");
                    session.confirm_copy = true;
                    session.copy_note = None;
                } else {
                    tracing::debug!("copy solution: buffer already matches");
                    session.copy_note = Some("Your code already matches the solution.");
                }
            }
            if ui.button("Back to goal").clicked() {
                tracing::info!("solution pane closed");
                session.show_solution = false;
                session.copy_note = None;
            }
        }
    });
    if let Some(note) = session.copy_note {
        widgets::note(ui, note);
    }
    ui.add_space(4.0);
    let font = widgets::mono_font();
    markdown::code_frame().show(ui, |ui| {
        egui::ScrollArea::both()
            .id_salt("solution_scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Long prose comments are re-flowed to the pane for
                // reading; code lines are untouched (they still scroll
                // sideways). The copy button above copies `code`, the
                // original text, never this display copy.
                let glyph = ui.fonts_mut(|f| f.glyph_width(&font, 'M')).max(1.0);
                let cols = solution_view_columns(ui.available_width(), glyph);
                let shown = highlight::wrap_comment_lines(code, cols);
                let job = highlight::layout_job(&shown, font.clone(), &BTreeSet::new());
                ui.add(egui::Label::new(job).extend());
            });
    });
}

/// Monospace columns that fit `width` points of solution view (never so
/// few that a comment re-flows one word per line).
fn solution_view_columns(width: f32, glyph_width: f32) -> usize {
    let cols = (width / glyph_width).floor();
    if cols.is_finite() && cols > 0.0 {
        // Truncation is the intent: whole columns only.
        (cols as usize).max(24)
    } else {
        24
    }
}

fn draw_result(
    ui: &mut egui::Ui,
    parsed: Option<&ParsedResult<'_>>,
    detail: Option<&ResultDetail>,
) {
    let Some(p) = parsed else {
        widgets::caption(ui, "RESULT");
        ui.label(
            egui::RichText::new("Nothing yet. Press Compile (Ctrl+Enter) to check your code.")
                .color(palette::TEXT_MUTED),
        );
        return;
    };
    let banner = diagnostics::banner(p, detail.and_then(|d| d.note.as_deref()));
    widgets::draw_banner(ui, &banner);
    if p.kind == ResultKind::Pending {
        return;
    }
    ui.add_space(2.0);
    let cleaned = diagnostics::clean_for_display(p.body);
    let wrong = p.tag == ResultTag::WrongOutput;
    let expected = detail.and_then(|d| d.expected.as_deref());
    let actual = detail.and_then(|d| d.actual.as_deref());
    let show_expected = wrong && (expected.is_some() || actual.is_some());
    // rustc output scrolls as a non-wrapping block; pin its first error
    // line above it (wrapped) so the headline is always readable.
    let pinned = if !show_expected && widgets::is_rustc_like(&cleaned) {
        diagnostics::first_error_line(&cleaned)
    } else {
        None
    };
    if let Some(first) = pinned {
        widgets::first_error(ui, first);
        ui.add_space(2.0);
    }
    // A fresh scroll state per result, so a new verdict starts at its top
    // left. Both directions: rustc's ASCII art must not wrap.
    let salt = ("result_scroll", p.body.len(), p.body.lines().count());
    egui::ScrollArea::both()
        .id_salt(salt)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if show_expected {
                widgets::draw_expected_actual(ui, expected, actual);
                if let Some(extra) = diagnostics::wrong_output_extra(&cleaned) {
                    ui.add_space(4.0);
                    widgets::draw_output(ui, extra);
                }
            } else {
                if p.kind == ResultKind::Pass && !cleaned.is_empty() && !p.offline {
                    widgets::caption(ui, "OUTPUT");
                }
                widgets::draw_output(ui, &cleaned);
            }
        });
}

/// Code page: Parchment fill, line-number gutter with error lines marked,
/// and the highlighted TextEdit filling the pane. Wrapping is off so each
/// gutter row lines up with exactly one source line; the page scrolls in
/// both directions inside its fixed rectangle.
fn draw_code(ui: &mut egui::Ui, source: &mut String, error_lines: &BTreeSet<usize>) {
    let font_id = widgets::mono_font();
    let gutter = gutter_job(source, &font_id, error_lines);
    let page = ui.available_size();

    egui::ScrollArea::both()
        .id_salt("source_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    // Match the TextEdit's top margin.
                    ui.add_space(4.0);
                    paint_gutter(ui, gutter, error_lines);
                });
                let layout_font = font_id.clone();
                let errs = error_lines.clone();
                let mut layouter =
                    move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap_width: f32| {
                        let job = highlight::layout_job(text.as_str(), layout_font.clone(), &errs);
                        ui.ctx().fonts_mut(|f| f.layout_job(job))
                    };
                let width = ui.available_width().max(200.0);
                ui.add(
                    egui::TextEdit::multiline(source)
                        .code_editor()
                        .font(font_id.clone())
                        .frame(false)
                        .margin(egui::Margin::symmetric(6, 4))
                        .desired_width(width)
                        .min_size(egui::vec2(width, (page.y - 8.0).max(0.0)))
                        .layouter(&mut layouter),
                );
            });
        });
}

/// Width of the Alarm scarlet bar beside an error line's number.
const ERROR_BAR_WIDTH: f32 = 3.0;

/// Draw the gutter numbers, with an Alarm scarlet bar (a mark, not text)
/// left of every line rustc flagged. The number itself stays Oxblood on an
/// Aged paper band: scarlet text would be 3.6:1 (bible v2.1).
fn paint_gutter(ui: &mut egui::Ui, job: egui::text::LayoutJob, error_lines: &BTreeSet<usize>) {
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let size = galley.size() + egui::vec2(ERROR_BAR_WIDTH, 0.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let rows = galley.rows.len().max(1);
    // One row per source line: the gutter job never wraps.
    let row_h = galley.size().y / rows as f32;
    for &line in error_lines {
        if line == 0 || line > rows {
            continue;
        }
        let top = rect.top() + row_h * (line - 1) as f32;
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left(), top),
            egui::vec2(ERROR_BAR_WIDTH, row_h),
        );
        ui.painter().rect_filled(bar, 0.0, palette::ERROR_MARK);
    }
    ui.painter().galley(
        rect.min + egui::vec2(ERROR_BAR_WIDTH, 0.0),
        galley,
        palette::TEXT,
    );
}

fn gutter_job(
    source: &str,
    font_id: &egui::FontId,
    error_lines: &BTreeSet<usize>,
) -> egui::text::LayoutJob {
    let line_count = source.matches('\n').count() + 1;
    let width = line_count.to_string().len().max(2);
    let mut job = egui::text::LayoutJob::default();
    for n in 1..=line_count {
        let is_err = error_lines.contains(&n);
        let text = if n == line_count {
            format!(" {n:>width$}")
        } else {
            format!(" {n:>width$}\n")
        };
        job.append(
            &text,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: if is_err {
                    palette::GUTTER_ERROR
                } else {
                    palette::GUTTER
                },
                background: if is_err {
                    palette::CHIP_BG
                } else {
                    egui::Color32::TRANSPARENT
                },
                ..Default::default()
            },
        );
    }
    job
}
