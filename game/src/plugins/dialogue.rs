// SPDX-License-Identifier: MIT
//! NPC dialogue box: greeting before a mission, a nudge while it is in
//! progress, a "come back later" while it is locked, and a thank-you once
//! it is cleared.
//!
//! Flow: F near an NPC opens a conversation chosen by
//! [`select_conversation`] from (NPC id, [`MissionStatus`]). Lines reveal
//! with a typewriter; F / Space / Enter completes the line, a second press
//! advances, Esc closes. When an Offer or InProgress conversation finishes,
//! [`DialogueHandoff`] is raised and `mission::handle_interact_key` runs the
//! pre-dialogue F behaviour (open the mission in the editor).
//!
//! Input ownership: [`dialogue_input`] runs inside `InputSystems`, right
//! after Bevy's keyboard system and therefore before every other reader
//! (`menu::route_escape` is ordered `.after(InputSystems)`). While a
//! conversation is open it consumes all fresh key presses and releases the
//! movement keys, so the player cannot walk, E cannot open the editor, and
//! Esc closes the box instead of pausing (dialogue behaves as an overlay).
//! Screenshot mode (`PNC_SHOTS`) never presses F, so it is never blocked.

pub mod lines;

use std::collections::HashSet;

use bevy::input::InputSystems;
use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::plugins::editor::EditorState;
use crate::plugins::mission::{CompletionView, MissionLockedAttempt, MissionRegistry};
use crate::plugins::npc::{NPC_ROSTER, NearbyNpc};
use crate::plugins::progress::MissionProgress;
use crate::plugins::state::{GameState, PauseState};

use lines::{AUTHORED, GENERIC, StatusLines};

// --- Bible role colours (design/03-art-style-bible.md, UI style) ---------
/// Primary UI surface.
pub const PARCHMENT_CREAM: egui::Color32 = egui::Color32::from_rgb(0xFC, 0xEF, 0xC8);
/// Frame colour (shared with the code-editor frame).
pub const BASALT: egui::Color32 = egui::Color32::from_rgb(0x3E, 0x38, 0x33);
/// Primary brand: the speaker's name plate.
pub const OXBLOOD: egui::Color32 = egui::Color32::from_rgb(0x6B, 0x1F, 0x35);
/// Body text (universal outline colour, never pure black).
pub const COALBLACK: egui::Color32 = egui::Color32::from_rgb(0x16, 0x13, 0x13);

/// Longest line (in chars, after `{next}` substitution) the box shows.
/// The box is at most [`BOX_MAX_WIDTH`] points wide; minus the portrait
/// column that leaves ~620 pt of text, about 80 chars per row at the
/// 16 pt body size, so 120 chars wraps to two rows inside the three-row
/// body area with headroom for narrow windows.
pub const MAX_LINE_CHARS: usize = 120;
/// Typewriter speed.
pub const CHARS_PER_SEC: f32 = 45.0;
/// Portrait slot edge in points. Sprites are scaled by the largest integer
/// that fits (32 px sprites at 2x, 64 px sprites at 1x).
pub const PORTRAIT_SLOT: f32 = 64.0;
const BOX_MAX_WIDTH: f32 = 760.0;
const BODY_TEXT_SIZE: f32 = 16.0;
/// Substituted for `{next}` when no mission is left to point at.
const NEXT_FALLBACK: &str = "the others";

/// Keys that advance a conversation.
const ADVANCE_KEYS: [KeyCode; 4] = [
    KeyCode::KeyF,
    KeyCode::Space,
    KeyCode::Enter,
    KeyCode::NumpadEnter,
];
/// Held keys that must stop acting while the box is open.
const MOVEMENT_KEYS: [KeyCode; 8] = [
    KeyCode::KeyW,
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
];

// --- Pure selection ------------------------------------------------------

/// Where the player stands with an NPC's mission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MissionStatus {
    NotYetAvailable,
    Offer,
    InProgress,
    Cleared,
}

impl MissionStatus {
    pub const ALL: [MissionStatus; 4] = [
        MissionStatus::NotYetAvailable,
        MissionStatus::Offer,
        MissionStatus::InProgress,
        MissionStatus::Cleared,
    ];
}

/// Classify a mission. Cleared wins over everything (cleared missions are
/// always unlocked); a locked mission is NotYetAvailable even if it was
/// somehow opened.
pub fn mission_status(cleared: bool, unlocked: bool, opened: bool) -> MissionStatus {
    if cleared {
        MissionStatus::Cleared
    } else if !unlocked {
        MissionStatus::NotYetAvailable
    } else if opened {
        MissionStatus::InProgress
    } else {
        MissionStatus::Offer
    }
}

/// What happens when the last line is dismissed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    /// Just close the box.
    Close,
    /// Close and open the NPC's mission (the old F behaviour).
    OpenMission,
}

/// A chosen conversation, `{next}` already substituted. Never empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub lines: Vec<String>,
    pub finish: Finish,
}

impl StatusLines {
    pub fn for_status(&self, status: MissionStatus) -> &'static [&'static str] {
        match status {
            MissionStatus::NotYetAvailable => self.not_yet,
            MissionStatus::Offer => self.offer,
            MissionStatus::InProgress => self.in_progress,
            MissionStatus::Cleared => self.cleared,
        }
    }
}

/// Authored lines for `npc_id`/`status`, else the generic fallback.
pub fn line_templates(npc_id: &str, status: MissionStatus) -> &'static [&'static str] {
    AUTHORED
        .iter()
        .find(|row| row.npc_id == npc_id)
        .map(|row| row.lines.for_status(status))
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| GENERIC.for_status(status))
}

/// Fill a line template.
pub fn render_line(template: &str, next_npc: Option<&str>) -> String {
    template.replace("{next}", next_npc.unwrap_or(NEXT_FALLBACK))
}

/// Choose the conversation for an NPC. `next_npc` is the speaker of the
/// registry's next uncleared mission (`MissionRegistry::next_locked`),
/// used by the NotYetAvailable hint.
pub fn select_conversation(
    npc_id: &str,
    status: MissionStatus,
    next_npc: Option<&str>,
) -> Conversation {
    let lines = line_templates(npc_id, status)
        .iter()
        .map(|t| render_line(t, next_npc))
        .collect();
    let finish = match status {
        MissionStatus::Offer | MissionStatus::InProgress => Finish::OpenMission,
        MissionStatus::NotYetAvailable | MissionStatus::Cleared => Finish::Close,
    };
    Conversation { lines, finish }
}

// --- Pure paging / typewriter --------------------------------------------

/// Result of one advance press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// The line was still typing; it is now fully shown.
    Revealed,
    /// Moved to the next line.
    NextPage,
    /// The last line was dismissed; the conversation is over.
    Finished,
}

/// Paging + typewriter state for one conversation.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueRun {
    lines: Vec<String>,
    page: usize,
    /// Characters of the current line revealed so far.
    shown: usize,
    /// Fractional characters carried between ticks.
    carry: f32,
}

impl DialogueRun {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            page: 0,
            shown: 0,
            carry: 0.0,
        }
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn page_count(&self) -> usize {
        self.lines.len()
    }

    /// The full current line (empty once past the end).
    pub fn line(&self) -> &str {
        self.lines.get(self.page).map(String::as_str).unwrap_or("")
    }

    fn line_chars(&self) -> usize {
        self.line().chars().count()
    }

    /// The revealed prefix of the current line.
    pub fn visible(&self) -> &str {
        let line = self.line();
        match line.char_indices().nth(self.shown) {
            Some((byte, _)) => &line[..byte],
            None => line,
        }
    }

    pub fn is_line_complete(&self) -> bool {
        self.shown >= self.line_chars()
    }

    pub fn is_last_page(&self) -> bool {
        self.page + 1 >= self.lines.len()
    }

    /// Reveal more of the current line.
    pub fn tick(&mut self, dt_secs: f32, chars_per_sec: f32) {
        if self.is_line_complete() {
            return;
        }
        self.carry += dt_secs.max(0.0) * chars_per_sec.max(0.0);
        let whole = self.carry.floor();
        self.carry -= whole;
        // `whole` is non-negative and bounded by one frame of typing.
        self.shown = (self.shown + whole as usize).min(self.line_chars());
    }

    /// Handle an advance press: complete the line, else page, else finish.
    pub fn press(&mut self) -> Press {
        if !self.is_line_complete() {
            self.shown = self.line_chars();
            self.carry = 0.0;
            return Press::Revealed;
        }
        if self.is_last_page() {
            return Press::Finished;
        }
        self.page += 1;
        self.shown = 0;
        self.carry = 0.0;
        Press::NextPage
    }
}

// --- ECS -----------------------------------------------------------------

/// A conversation on screen.
pub struct ActiveDialogue {
    pub speaker: &'static str,
    pub npc_id: &'static str,
    pub status: MissionStatus,
    pub finish: Finish,
    pub run: DialogueRun,
    /// NPC sprite reused as the portrait; `None` headless.
    pub portrait: Option<Handle<Image>>,
    /// Native edge of the portrait sprite in px.
    pub portrait_native_px: f32,
}

/// The open conversation, if any.
#[derive(Resource, Default)]
pub struct Dialogue {
    pub active: Option<ActiveDialogue>,
}

impl Dialogue {
    pub fn is_open(&self) -> bool {
        self.active.is_some()
    }
}

/// Raised when a conversation ends in [`Finish::OpenMission`]; consumed by
/// `mission::handle_interact_key` the same frame.
#[derive(Resource, Default, Debug)]
pub struct DialogueHandoff {
    pending: bool,
}

impl DialogueHandoff {
    pub fn raise(&mut self) {
        self.pending = true;
    }

    /// Returns whether a handoff was pending, clearing it.
    pub fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

/// Missions whose panel was opened from dialogue this session (drives
/// InProgress vs Offer). Session-only; not saved.
#[derive(Resource, Default, Debug)]
pub struct DialogueMemory {
    pub opened: HashSet<&'static str>,
}

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("DialoguePlugin::build");
        app.init_resource::<Dialogue>()
            .init_resource::<DialogueHandoff>()
            .init_resource::<DialogueMemory>()
            .add_systems(
                PreUpdate,
                dialogue_input
                    .in_set(InputSystems)
                    .after(keyboard_input_system)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(Update, tick_typewriter.run_if(in_state(GameState::InGame)))
            .add_systems(OnExit(GameState::InGame), reset_dialogue)
            .add_systems(
                EguiPrimaryContextPass,
                draw_dialogue.run_if(in_state(GameState::InGame)),
            );
    }
}

/// Swallow gameplay input for this frame: fresh presses vanish (F, E, Esc
/// reach nobody else) and held movement keys read as released. Advance
/// keys stay "held", so OS key-repeat does not re-trigger them.
fn swallow_input(keys: &mut ButtonInput<KeyCode>) {
    keys.clear();
    for k in MOVEMENT_KEYS {
        keys.release(k);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn dialogue_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    pause: Option<Res<State<PauseState>>>,
    nearby: Option<Res<NearbyNpc>>,
    editor: Option<Res<EditorState>>,
    completion: Option<Res<CompletionView>>,
    registry: Option<Res<MissionRegistry>>,
    progress: Option<Res<MissionProgress>>,
    asset_server: Option<Res<AssetServer>>,
    mut dialogue: ResMut<Dialogue>,
    mut handoff: ResMut<DialogueHandoff>,
    mut memory: ResMut<DialogueMemory>,
    mut locked: MessageWriter<MissionLockedAttempt>,
) {
    if pause.is_some_and(|p| *p.get() == PauseState::Paused) {
        return;
    }

    // --- A conversation is open: it owns the keyboard. -----------------
    if let Some(active) = dialogue.active.as_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            tracing::debug!(npc = active.npc_id, "dialogue closed via Escape");
            dialogue.active = None;
        } else if keys.any_just_pressed(ADVANCE_KEYS) {
            let press = active.run.press();
            tracing::debug!(
                npc = active.npc_id,
                ?press,
                page = active.run.page(),
                pages = active.run.page_count(),
                "dialogue advance"
            );
            if press == Press::Finished {
                tracing::debug!(npc = active.npc_id, finish = ?active.finish, "dialogue finished");
                if active.finish == Finish::OpenMission {
                    memory.opened.insert(active.npc_id);
                    handoff.raise();
                }
                dialogue.active = None;
            }
        }
        swallow_input(&mut keys);
        return;
    }

    // --- Closed: F near an NPC opens one. ------------------------------
    // The editor owns F while open (it is the letter f), and the
    // completion recap keeps its own F-to-revisit path.
    if editor.is_some_and(|e| e.open) || completion.is_some_and(|c| c.mission_id.is_some()) {
        return;
    }
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Some(entry) = nearby.as_ref().and_then(|n| n.current.clone()) else {
        return;
    };
    let (Some(registry), Some(progress)) = (registry, progress) else {
        tracing::warn!("dialogue: registry/progress missing; F ignored");
        return;
    };
    let Some(mission) = registry.find(entry.mission_id) else {
        // mission.rs logs the unknown-mission case on its own path.
        return;
    };

    let status = mission_status(
        progress.is_cleared(mission.id),
        mission.is_unlocked(&progress),
        memory.opened.contains(entry.mission_id),
    );
    if status == MissionStatus::NotYetAvailable {
        // Keep the locked sting that F used to fire directly.
        locked.write(MissionLockedAttempt {
            mission_id: mission.id.to_string(),
        });
    }
    let next_npc = registry.next_locked(&progress).map(|m| m.npc_name);
    let convo = select_conversation(entry.mission_id, status, next_npc);

    let spec = NPC_ROSTER.iter().find(|s| s.mission_id == entry.mission_id);
    let portrait = spec
        .zip(asset_server.as_ref())
        .map(|(s, server)| server.load(s.sprite_path));
    let portrait_native_px = spec.map_or(PORTRAIT_SLOT, |s| s.native_px);

    tracing::debug!(
        npc = entry.mission_id,
        speaker = entry.name,
        ?status,
        lines = convo.lines.len(),
        "dialogue opened"
    );
    dialogue.active = Some(ActiveDialogue {
        speaker: entry.name,
        npc_id: entry.mission_id,
        status,
        finish: convo.finish,
        run: DialogueRun::new(convo.lines),
        portrait,
        portrait_native_px,
    });
    swallow_input(&mut keys);
}

/// Real time, so a frozen virtual clock never stalls the text.
fn tick_typewriter(time: Res<Time<Real>>, mut dialogue: ResMut<Dialogue>) {
    if let Some(active) = dialogue.active.as_mut() {
        active.run.tick(time.delta_secs(), CHARS_PER_SEC);
    }
}

fn reset_dialogue(
    mut dialogue: ResMut<Dialogue>,
    mut handoff: ResMut<DialogueHandoff>,
    mut memory: ResMut<DialogueMemory>,
) {
    tracing::debug!(was_open = dialogue.is_open(), "left InGame: dialogue reset");
    dialogue.active = None;
    handoff.take();
    memory.opened.clear();
}

/// Largest integer scale of `native` that fits `slot` (at least 1).
pub fn portrait_scale(native: f32, slot: f32) -> f32 {
    if native <= 0.0 {
        return 1.0;
    }
    (slot / native).floor().max(1.0)
}

fn draw_dialogue(mut contexts: EguiContexts, dialogue: Res<Dialogue>) {
    let Some(active) = dialogue.active.as_ref() else {
        return;
    };
    let portrait_id = active
        .portrait
        .as_ref()
        .map(|h| contexts.add_image(EguiTextureHandle::Strong(h.clone())));
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let width = (ctx.content_rect().width() - 32.0).clamp(240.0, BOX_MAX_WIDTH);
    let edge = active.portrait_native_px * portrait_scale(active.portrait_native_px, PORTRAIT_SLOT);
    let hint = if !active.run.is_line_complete() {
        "[F] skip   [Esc] close"
    } else if active.run.is_last_page() {
        "[F] done   [Esc] close"
    } else {
        "[F] next   [Esc] close"
    };
    let frame = |margin: i8| {
        egui::Frame::new()
            .fill(PARCHMENT_CREAM)
            .stroke(egui::Stroke::new(2.0_f32, BASALT))
            .inner_margin(egui::Margin::same(margin))
    };

    egui::Area::new(egui::Id::new("dialogue_box"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -16.0))
        .show(ctx, |ui| {
            // Two nested 2 px Basalt strokes with a parchment gap = the
            // bible's double-line frame. No shadow, square corners.
            frame(2).show(ui, |ui| {
                frame(10).show(ui, |ui| {
                    ui.set_width(width);
                    ui.horizontal_top(|ui| {
                        let slot = egui::Vec2::splat(PORTRAIT_SLOT);
                        ui.allocate_ui(slot, |ui| {
                            ui.set_min_size(slot);
                            if let Some(id) = portrait_id {
                                ui.add(egui::Image::new((id, egui::Vec2::splat(edge))));
                            }
                        });
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            egui::Frame::new()
                                .fill(OXBLOOD)
                                .inner_margin(egui::Margin::symmetric(8, 3))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(active.speaker)
                                            .color(PARCHMENT_CREAM)
                                            .strong(),
                                    );
                                });
                            ui.add_space(6.0);
                            ui.set_min_height(BODY_TEXT_SIZE * 3.6);
                            ui.label(
                                egui::RichText::new(active.run.visible())
                                    .color(COALBLACK)
                                    .size(BODY_TEXT_SIZE),
                            );
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label(egui::RichText::new(hint).color(BASALT).small());
                    });
                });
            });
        });
}
