// SPDX-License-Identifier: MIT
//! Title screen. REF-10 painted backdrop scaled to fill the window, with
//! an egui menu: Continue / New Game / Settings / Quit.
//!
//! Space or Enter activates the highlighted item, and the highlight starts
//! on the first entry — Continue when a save with progress exists, New
//! Game otherwise — so a bare Space/Enter still means "Continue or New
//! Game" exactly as before the menu existed. New Game over an existing
//! save asks for confirmation (defaulting to Cancel) before wiping it.
//!
//! The screenshot driver (`PNC_SHOTS`) sets `GameState::InGame` directly,
//! so nothing here stands in its way.

use crate::assets::SPRITE_TITLE;
use crate::plugins::menu::{
    MenuScreen, MenuUi, SettingsOutcome, menu_list, read_nav, settings_panel,
};
use crate::plugins::mission::MissionRegistry;
use crate::plugins::progress::{MissionProgress, SaveRequest, SavedPlayerPosition, Settings};
use crate::plugins::state::GameState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

const TITLE_NATIVE_W: f32 = 640.0;
const TITLE_NATIVE_H: f32 = 360.0;
// Camera is FixedVertical at 180 world units; the title art is 360
// pixels tall natively, so 0.5 maps it 1:1 to the visible area without
// crop. (Was 2.0, which made the title 4x the viewport — only the
// center parchment bg showed, which is why the wasm screen looked
// "empty" until this was caught.)
const TITLE_SCALE: f32 = 0.5;

/// Quitting is a native-only concept; a browser tab has no "exit".
const QUIT_AVAILABLE: bool = !cfg!(target_arch = "wasm32");

#[derive(Component)]
struct TitleArt;

/// Title menu entries, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleItem {
    Continue,
    NewGame,
    Settings,
    Quit,
}

impl TitleItem {
    pub fn label(self) -> &'static str {
        match self {
            TitleItem::Continue => "Continue",
            TitleItem::NewGame => "New Game",
            TitleItem::Settings => "Settings",
            TitleItem::Quit => "Quit",
        }
    }
}

/// The title menu for this run. Continue only appears (first) when there
/// is progress to continue; Quit only where quitting makes sense.
pub fn title_items(has_progress: bool, quit_available: bool) -> Vec<TitleItem> {
    let mut items = Vec::with_capacity(4);
    if has_progress {
        items.push(TitleItem::Continue);
    }
    items.push(TitleItem::NewGame);
    items.push(TitleItem::Settings);
    if quit_available {
        items.push(TitleItem::Quit);
    }
    items
}

/// Rows of the New Game confirmation. Cancel is first so a reflexive
/// Enter never erases a save.
const CONFIRM_ITEMS: [&str; 2] = ["Cancel", "Erase progress and start"];

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("TitlePlugin::build");
        app.add_systems(OnEnter(GameState::Title), (center_camera, spawn_title_art))
            .add_systems(OnExit(GameState::Title), despawn_title_art)
            .add_systems(
                EguiPrimaryContextPass,
                draw_title_menu.run_if(in_state(GameState::Title)),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_title_menu(
    mut contexts: EguiContexts,
    mut progress: ResMut<MissionProgress>,
    mut saved_pos: ResMut<SavedPlayerPosition>,
    mut settings: ResMut<Settings>,
    mut menu: ResMut<MenuUi>,
    registry: Res<MissionRegistry>,
    mut next: ResMut<NextState<GameState>>,
    mut requests: MessageWriter<SaveRequest>,
    mut exits: MessageWriter<AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let nav = read_nav(ctx);
    let cleared = progress.cleared_count();
    let total = registry.missions.len();
    let has_progress = cleared > 0 || saved_pos.0.is_some();

    egui::Window::new("title_menu")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -32.0))
        .show(ctx, |ui| match menu.screen {
            MenuScreen::Main => {
                ui.vertical_centered(|ui| {
                    if cleared == 0 {
                        ui.small("a Rust kingdom awaits its pledge");
                    } else if cleared < total {
                        ui.small(format!(
                            "your pledge so far: {cleared} / {total} encounters cleared"
                        ));
                    } else {
                        ui.small(format!(
                            "the realm has crowned you - {cleared} / {total} cleared"
                        ));
                    }
                    ui.separator();
                    let items = title_items(has_progress, QUIT_AVAILABLE);
                    let labels: Vec<&str> = items.iter().map(|i| i.label()).collect();
                    let mut sel = menu.selected;
                    let activated = menu_list(ui, &labels, &mut sel, nav);
                    menu.selected = sel;
                    match activated.and_then(|i| items.get(i).copied()) {
                        Some(TitleItem::Continue) => {
                            tracing::info!(cleared, pos = ?saved_pos.0, "title: continue -> InGame");
                            next.set(GameState::InGame);
                        }
                        Some(TitleItem::NewGame) if has_progress => {
                            menu.open(MenuScreen::ConfirmNewGame);
                        }
                        Some(TitleItem::NewGame) => {
                            tracing::info!("title: new game (no prior progress) -> InGame");
                            next.set(GameState::InGame);
                        }
                        Some(TitleItem::Settings) => menu.open(MenuScreen::Settings),
                        Some(TitleItem::Quit) => {
                            tracing::info!("title: quit");
                            requests.write(SaveRequest);
                            exits.write(AppExit::Success);
                        }
                        None => {}
                    }
                });
            }
            MenuScreen::ConfirmNewGame => {
                ui.vertical_centered(|ui| {
                    ui.heading("Start a new game?");
                    ui.label(format!(
                        "This erases your saved progress ({cleared} / {total} encounters cleared)."
                    ));
                    ui.separator();
                    let mut sel = menu.selected;
                    let activated = menu_list(ui, &CONFIRM_ITEMS, &mut sel, nav);
                    menu.selected = sel;
                    match activated {
                        Some(0) => menu.open(MenuScreen::Main),
                        Some(_) => {
                            tracing::warn!(cleared, "title: new game confirmed; wiping progress");
                            progress.reset();
                            saved_pos.0 = None;
                            requests.write(SaveRequest);
                            next.set(GameState::InGame);
                        }
                        None => {}
                    }
                    ui.small("[Esc] back");
                });
            }
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
        });

    // Controls panel pinned to the bottom-right. Discoverable without
    // cluttering in-game UI.
    egui::Window::new("title_controls")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-12.0, -12.0))
        .show(ctx, |ui| {
            ui.small("controls");
            ui.separator();
            ui.small("[WASD]  walk");
            ui.small("[F]     talk to NPC");
            ui.small("[E]     open code sandbox");
            ui.small("[Esc]   close panel / pause");
        });
}

fn spawn_title_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!("spawning title art (REF-10)");
    commands.spawn((
        Sprite {
            image: asset_server.load(SPRITE_TITLE),
            custom_size: Some(Vec2::new(
                TITLE_NATIVE_W * TITLE_SCALE,
                TITLE_NATIVE_H * TITLE_SCALE,
            )),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
        TitleArt,
    ));
}

/// The title art sits at the world origin; after Quit to Title the camera
/// is still wherever the player was, so bring it home.
fn center_camera(mut cam_q: Query<&mut Transform, With<Camera2d>>) {
    for mut t in cam_q.iter_mut() {
        t.translation.x = 0.0;
        t.translation.y = 0.0;
    }
    tracing::debug!("title: camera recentred on origin");
}

fn despawn_title_art(mut commands: Commands, query: Query<Entity, With<TitleArt>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
