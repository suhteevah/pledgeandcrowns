// SPDX-License-Identifier: MIT
//! Pause menu, settings screen, Esc routing and the shared menu widgets the
//! title screen also uses.
//!
//! Keyboard model (title + pause): Up/Down or W/S move the highlight,
//! Enter or Space activates it, Esc backs out (see
//! [`crate::plugins::state::esc_action`]). Mouse hover moves the highlight
//! and a click activates. On the settings screen Left/Right or A/D nudge
//! the highlighted slider by [`SLIDER_STEP`].
//!
//! While paused, gameplay keyboard input is swallowed (`ButtonInput` reset
//! in `PreUpdate`) so F/E/WASD do nothing; egui reads raw keyboard
//! messages, so the menus keep working. `Time<Virtual>` is frozen by
//! `StatePlugin`.

use bevy::input::InputSystems;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::plugins::editor::EditorState;
use crate::plugins::mission::{CompletionView, EpilogueView, MissionRegistry};
use crate::plugins::progress::{MissionProgress, SaveRequest, SettingKind, Settings};
use crate::plugins::state::{EscAction, EscContext, GameState, PauseState, esc_action};
use crate::plugins::ui_theme;

/// How far one Left/Right press moves a volume slider.
pub const SLIDER_STEP: u8 = 5;
/// Settings grid: label column (a menu row) and value column widths.
const SETTINGS_LABEL_WIDTH: f32 = 170.0;
const SETTINGS_VALUE_WIDTH: f32 = 48.0;

/// Which screen of the current menu (title or pause) is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuScreen {
    #[default]
    Main,
    Settings,
    /// Title only: "erase progress and start over?"
    ConfirmNewGame,
}

/// Shared menu UI state. Reset when the title screen or pause menu opens.
#[derive(Resource, Debug, Default)]
pub struct MenuUi {
    pub screen: MenuScreen,
    pub selected: usize,
}

impl MenuUi {
    pub fn open(&mut self, screen: MenuScreen) {
        tracing::debug!(?screen, "menu screen");
        self.screen = screen;
        self.selected = 0;
    }
    pub fn submenu_open(&self) -> bool {
        self.screen != MenuScreen::Main
    }
}

/// Pause menu entries, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseItem {
    Resume,
    Settings,
    QuitToTitle,
}

impl PauseItem {
    pub const ALL: [PauseItem; 3] = [
        PauseItem::Resume,
        PauseItem::Settings,
        PauseItem::QuitToTitle,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PauseItem::Resume => "Resume",
            PauseItem::Settings => "Settings",
            PauseItem::QuitToTitle => "Quit to Title",
        }
    }
}

/// Move a highlight by `delta` rows, wrapping. `len == 0` yields 0.
pub fn step_selection(selected: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let len_i = len as i64;
    let next = (selected.min(len - 1) as i64 + i64::from(delta)).rem_euclid(len_i);
    next as usize
}

/// Nudge a 0..=100 slider value by `delta` steps of [`SLIDER_STEP`],
/// saturating at both ends.
pub fn nudge_volume(value: u8, delta: i32) -> u8 {
    let v = i32::from(value) + delta * i32::from(SLIDER_STEP);
    v.clamp(0, i32::from(Settings::VOLUME_MAX)) as u8
}

/// One frame of menu keyboard input, read from egui.
#[derive(Debug, Clone, Copy, Default)]
pub struct NavInput {
    pub vertical: i32,
    pub horizontal: i32,
    pub activate: bool,
}

pub fn read_nav(ctx: &egui::Context) -> NavInput {
    ctx.input(|i| {
        let up = i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::W);
        let down = i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::S);
        let left = i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::A);
        let right = i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::D);
        NavInput {
            vertical: i32::from(down) - i32::from(up),
            horizontal: i32::from(right) - i32::from(left),
            activate: i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space),
        }
    })
}

/// Draw a vertical list of menu buttons. Returns the index activated this
/// frame (by click or Enter/Space on the highlight). Buttons give up egui
/// keyboard focus so Enter is handled exactly once, here.
pub fn menu_list(
    ui: &mut egui::Ui,
    labels: &[&str],
    selected: &mut usize,
    nav: NavInput,
) -> Option<usize> {
    if labels.is_empty() {
        return None;
    }
    *selected = step_selection(*selected, labels.len(), nav.vertical);
    let mut activated = None;
    for (i, label) in labels.iter().enumerate() {
        // Selection is the Oxblood bar + gold markers (ui_theme), not text.
        let resp = ui_theme::menu_item(ui, label, i == *selected, ui_theme::MENU_ITEM_WIDTH);
        if resp.hovered() {
            *selected = i;
        }
        if resp.clicked() {
            activated = Some(i);
        }
        if resp.has_focus() {
            resp.surrender_focus();
        }
    }
    if activated.is_none() && nav.activate {
        activated = Some(*selected);
    }
    activated
}

/// Result of one frame of the settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsOutcome {
    Stay,
    Back,
}

/// The settings screen body, shared by the title and pause menus. Rows
/// come from [`SettingKind::ALL`] plus a trailing Back row. Takes the
/// current settings by value and returns the edited copy; callers write
/// it back only if it differs, so audio's change detection fires on real
/// edits only.
pub fn settings_panel(
    ui: &mut egui::Ui,
    current: Settings,
    menu: &mut MenuUi,
    nav: NavInput,
) -> (SettingsOutcome, Settings) {
    let rows = SettingKind::ALL.len() + 1; // + Back
    let back_idx = rows - 1;
    menu.selected = step_selection(menu.selected, rows, nav.vertical);
    let mut edited = current;
    ui.vertical_centered(|ui| {
        ui_theme::heading(ui, "Settings");
    });
    ui_theme::rule(ui);
    // Three aligned columns: label (a menu row, so the highlight reads the
    // same as every other menu), slider, value.
    egui::Grid::new("settings_grid")
        .num_columns(3)
        .spacing([12.0, 8.0])
        .min_row_height(ui_theme::MENU_ITEM_HEIGHT)
        .show(ui, |ui| {
            for (i, kind) in SettingKind::ALL.into_iter().enumerate() {
                let mut value = edited.get(kind);
                if i == menu.selected && nav.horizontal != 0 {
                    value = nudge_volume(value, nav.horizontal);
                }
                let label =
                    ui_theme::menu_item(ui, kind.label(), i == menu.selected, SETTINGS_LABEL_WIDTH);
                let resp = ui
                    .scope(|ui| {
                        // Filled part of the rail in the anchor mid-tone.
                        ui.visuals_mut().selection.bg_fill = ui_theme::WINEFLESH;
                        ui.add(
                            egui::Slider::new(&mut value, 0..=Settings::VOLUME_MAX)
                                .show_value(false),
                        )
                    })
                    .inner;
                ui.add_sized(
                    [SETTINGS_VALUE_WIDTH, ui_theme::MENU_ITEM_HEIGHT],
                    egui::Label::new(
                        egui::RichText::new(format!("{value}%"))
                            .size(ui_theme::BODY_SIZE)
                            .color(ui_theme::TEXT),
                    ),
                );
                ui.end_row();
                if label.hovered() || resp.hovered() || resp.dragged() {
                    menu.selected = i;
                }
                if resp.has_focus() {
                    resp.surrender_focus();
                }
                if value != edited.get(kind) {
                    tracing::debug!(
                        ?kind,
                        from = edited.get(kind),
                        to = value,
                        "setting changed"
                    );
                    edited.set(kind, value);
                }
            }
        });
    ui_theme::rule(ui);
    let back_selected = menu.selected == back_idx;
    let resp = ui
        .vertical_centered(|ui| {
            let resp = ui_theme::menu_item(ui, "Back", back_selected, ui_theme::MENU_ITEM_WIDTH);
            ui_theme::key_hint(
                ui,
                &[
                    ("Up/Down", "choose"),
                    ("Left/Right", "adjust"),
                    ("Esc", "back"),
                ],
            );
            resp
        })
        .inner;
    if resp.hovered() {
        menu.selected = back_idx;
    }
    if resp.has_focus() {
        resp.surrender_focus();
    }
    let back = resp.clicked() || (back_selected && nav.activate);
    let outcome = if back {
        SettingsOutcome::Back
    } else {
        SettingsOutcome::Stay
    };
    (outcome, edited)
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("MenuPlugin::build");
        app.init_resource::<MenuUi>()
            .add_systems(OnEnter(GameState::Title), reset_menu)
            .add_systems(OnEnter(PauseState::Paused), (reset_menu, request_save))
            .add_systems(OnExit(GameState::InGame), request_save)
            .add_systems(
                PreUpdate,
                (route_escape, swallow_gameplay_input_while_paused)
                    .chain()
                    .after(InputSystems),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_pause_menu.run_if(in_state(PauseState::Paused)),
            );
    }
}

fn reset_menu(mut menu: ResMut<MenuUi>) {
    menu.open(MenuScreen::Main);
}

fn request_save(mut requests: MessageWriter<SaveRequest>) {
    tracing::debug!("menu: requesting save");
    requests.write(SaveRequest);
}

/// Translate an Esc press into a state change using the pure
/// [`esc_action`] rules. Runs in `PreUpdate`, before the editor and
/// mission systems see the key in `Update`, and reads their state
/// read-only: if the editor or an Esc-dismissable overlay is open, the
/// press is left for them.
#[allow(clippy::too_many_arguments)]
fn route_escape(
    keys: Res<ButtonInput<KeyCode>>,
    state: Option<Res<State<GameState>>>,
    pause: Option<Res<State<PauseState>>>,
    editor: Option<Res<EditorState>>,
    completion: Option<Res<CompletionView>>,
    epilogue: Option<Res<EpilogueView>>,
    progress: Option<Res<MissionProgress>>,
    registry: Option<Res<MissionRegistry>>,
    mut menu: ResMut<MenuUi>,
    mut next_pause: ResMut<NextState<PauseState>>,
    mut requests: MessageWriter<SaveRequest>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    let Some(state) = state else {
        return;
    };
    let paused = pause.is_some_and(|p| *p.get() == PauseState::Paused);
    let editor_open = editor.is_some_and(|e| e.open);
    let completion_open = completion.as_ref().is_some_and(|c| c.mission_id.is_some());
    // Mirrors mission::dismiss_epilogue_on_escape's visibility predicate.
    let epilogue_open = match (&epilogue, &progress, &registry) {
        (Some(ep), Some(pr), Some(reg)) => {
            !ep.dismissed_this_session
                && !completion_open
                && !editor_open
                && pr.cleared_count() == reg.missions.len()
        }
        _ => false,
    };
    let ctx = EscContext {
        state: *state.get(),
        paused,
        editor_open,
        overlay_open: completion_open || epilogue_open,
        submenu_open: menu.submenu_open(),
    };
    let action = esc_action(ctx);
    tracing::debug!(?ctx, ?action, "Esc pressed");
    match action {
        EscAction::Ignore => {}
        EscAction::Pause => {
            tracing::info!("InGame -> Paused");
            next_pause.set(PauseState::Paused);
        }
        EscAction::Resume => {
            tracing::info!("Paused -> InGame");
            next_pause.set(PauseState::Running);
        }
        EscAction::CloseSubmenu => {
            if menu.screen == MenuScreen::Settings {
                requests.write(SaveRequest);
            }
            menu.open(MenuScreen::Main);
        }
    }
}

/// While paused, clear gameplay keyboard state so `Update` systems in
/// other plugins (movement, F-to-talk, E-to-edit) see no input. Same
/// technique as bevy_egui's `absorb_bevy_input_system`.
fn swallow_gameplay_input_while_paused(
    pause: Option<Res<State<PauseState>>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    if pause.is_some_and(|p| *p.get() == PauseState::Paused) {
        keys.reset_all();
    }
}

fn draw_pause_menu(
    mut contexts: EguiContexts,
    mut menu: ResMut<MenuUi>,
    mut settings: ResMut<Settings>,
    mut next_pause: ResMut<NextState<PauseState>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut requests: MessageWriter<SaveRequest>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let nav = read_nav(ctx);
    ui_theme::panel_area(
        "pause_menu",
        egui::Align2::CENTER_CENTER,
        egui::Vec2::ZERO,
        egui::Order::Foreground,
    )
    .show(ctx, |ui| {
        ui_theme::framed_panel(ui, |ui| match menu.screen {
            MenuScreen::Settings => {
                let (outcome, edited) = settings_panel(ui, *settings, &mut menu, nav);
                if edited != *settings {
                    *settings = edited;
                }
                if outcome == SettingsOutcome::Back {
                    requests.write(SaveRequest);
                    menu.open(MenuScreen::Main);
                }
            }
            // ConfirmNewGame is title-only; fall back to the main list.
            MenuScreen::Main | MenuScreen::ConfirmNewGame => {
                ui.set_width(ui_theme::MENU_ITEM_WIDTH + 40.0);
                ui.vertical_centered(|ui| {
                    ui_theme::heading(ui, "Paused");
                    ui_theme::rule(ui);
                    let labels: Vec<&str> = PauseItem::ALL.iter().map(|i| i.label()).collect();
                    let mut sel = menu.selected;
                    let activated = menu_list(ui, &labels, &mut sel, nav);
                    menu.selected = sel;
                    match activated.and_then(|i| PauseItem::ALL.get(i).copied()) {
                        Some(PauseItem::Resume) => {
                            tracing::info!("pause menu: resume");
                            next_pause.set(PauseState::Running);
                        }
                        Some(PauseItem::Settings) => menu.open(MenuScreen::Settings),
                        Some(PauseItem::QuitToTitle) => {
                            tracing::info!("pause menu: quit to title");
                            next_game.set(GameState::Title);
                        }
                        None => {}
                    }
                    ui.add_space(4.0);
                    ui_theme::key_hint(ui, &[("Enter", "choose"), ("Esc", "resume")]);
                });
            }
        });
    });
}
