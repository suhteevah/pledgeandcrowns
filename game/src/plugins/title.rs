// SPDX-License-Identifier: MIT
//! Title screen. REF-10 painted backdrop at the largest whole-number scale
//! that fits the window, centred, on a backdrop of the art's own colours
//! (Oxblood sky above the horizon, Inkblood below), with an egui menu:
//! Continue / New Game / Settings / Quit.
//!
//! The menu panel sits over the lower third of the art. That band of the
//! current title PNG is blank (a redraw is pending), so the panel covers
//! it on purpose rather than floating over the painted scene.
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
use crate::plugins::VIEW_HEIGHT;
use crate::plugins::menu::{
    MenuScreen, MenuUi, SettingsOutcome, menu_list, read_nav, settings_panel,
};
use crate::plugins::mission::MissionRegistry;
use crate::plugins::progress::{MissionProgress, SaveRequest, SavedPlayerPosition, Settings};
use crate::plugins::state::GameState;
use crate::plugins::ui_theme;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

/// Native size of the title art in pixels.
pub const TITLE_NATIVE_W: f32 = 640.0;
pub const TITLE_NATIVE_H: f32 = 360.0;
/// Edge length of each backdrop half, in world units: far larger than any
/// view, so the letterbox is always covered.
const BACKDROP_EXTENT: f32 = 100_000.0;
/// Gap between the menu panel and the art's bottom edge, in points.
const MENU_BOTTOM_GAP: f32 = 14.0;

/// Quitting is a native-only concept; a browser tab has no "exit".
const QUIT_AVAILABLE: bool = !cfg!(target_arch = "wasm32");

#[derive(Component)]
struct TitleArt;

/// The art sprite itself (resized every frame to stay pixel-exact).
#[derive(Component)]
struct TitleArtImage;

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

/// Largest whole-number scale at which the 640x360 art fits a window of
/// `phys_w` x `phys_h` physical pixels. Never below 1 (a tiny window
/// crops instead of blurring).
pub fn title_integer_scale(phys_w: f32, phys_h: f32) -> f32 {
    (phys_w / TITLE_NATIVE_W)
        .min(phys_h / TITLE_NATIVE_H)
        .floor()
        .max(1.0)
}

/// Sprite size in world units that shows the art at `scale` physical
/// pixels per art pixel, given the camera's fixed vertical view.
pub fn title_world_size(scale: f32, phys_h: f32, view_height: f32) -> Vec2 {
    if phys_h <= 0.0 {
        return Vec2::new(TITLE_NATIVE_W, TITLE_NATIVE_H);
    }
    let units_per_px = view_height / phys_h;
    Vec2::new(TITLE_NATIVE_W, TITLE_NATIVE_H) * scale * units_per_px
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
            .add_systems(Update, fit_title_art.run_if(in_state(GameState::Title)))
            .add_systems(
                EguiPrimaryContextPass,
                draw_title_menu.run_if(in_state(GameState::Title)),
            );
    }
}

/// Art rectangle inset from the window edges, in egui points:
/// (horizontal letterbox, vertical letterbox). Zero when the art fills
/// the window (1280x720 at 2x, 1920x1080 at 3x).
fn letterbox_points(window: Option<&Window>, pixels_per_point: f32) -> egui::Vec2 {
    let Some(w) = window else {
        return egui::Vec2::ZERO;
    };
    let (pw, ph) = (w.physical_width() as f32, w.physical_height() as f32);
    let k = title_integer_scale(pw, ph);
    let ppp = pixels_per_point.max(0.01);
    egui::vec2(
        ((pw - TITLE_NATIVE_W * k) * 0.5).max(0.0) / ppp,
        ((ph - TITLE_NATIVE_H * k) * 0.5).max(0.0) / ppp,
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_title_menu(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
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
    let inset = letterbox_points(windows.single().ok(), ctx.pixels_per_point());

    ui_theme::panel_area(
        "title_menu",
        egui::Align2::CENTER_BOTTOM,
        egui::vec2(0.0, -(inset.y + MENU_BOTTOM_GAP)),
        egui::Order::Foreground,
    )
    .show(ctx, |ui| {
        ui_theme::framed_panel(ui, |ui| match menu.screen {
            MenuScreen::Main => {
                ui.set_width(ui_theme::MENU_ITEM_WIDTH + 60.0);
                ui.vertical_centered(|ui| {
                    let tagline = if cleared == 0 {
                        "a Rust kingdom awaits its pledge".to_string()
                    } else if cleared < total {
                        format!("your pledge so far: {cleared} / {total} encounters cleared")
                    } else {
                        format!("the realm has crowned you - {cleared} / {total} cleared")
                    };
                    ui_theme::caption(ui, tagline);
                    ui_theme::rule(ui);
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
                ui.set_width(ui_theme::MENU_ITEM_WIDTH + 140.0);
                ui.vertical_centered(|ui| {
                    ui_theme::heading(ui, "Start a new game?");
                    ui.label(format!(
                        "This erases your saved progress ({cleared} / {total} encounters cleared)."
                    ));
                    ui_theme::rule(ui);
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
                    ui_theme::key_hint(ui, &[("Esc", "back")]);
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
    });

    // Controls card pinned to the art's bottom-right corner.
    ui_theme::panel_area(
        "title_controls",
        egui::Align2::RIGHT_BOTTOM,
        egui::vec2(-(inset.x + 12.0), -(inset.y + MENU_BOTTOM_GAP)),
        egui::Order::Middle,
    )
    .show(ctx, |ui| {
        ui_theme::framed_panel(ui, controls_card);
    });
}

/// The controls reference: key column in Oxblood, action column in body
/// text.
pub fn controls_card(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Controls")
            .size(ui_theme::SUBHEADING_SIZE)
            .color(ui_theme::TEXT_HEADING)
            .strong(),
    );
    egui::Grid::new("controls_grid")
        .num_columns(2)
        .spacing([12.0, 2.0])
        .show(ui, |ui| {
            for (key, action) in [
                ("WASD", "walk"),
                ("F", "talk to NPC"),
                ("E", "open code sandbox"),
                ("Esc", "close panel / pause"),
            ] {
                ui.label(
                    egui::RichText::new(key)
                        .size(ui_theme::SMALL_SIZE)
                        .color(ui_theme::OXBLOOD)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(action)
                        .size(ui_theme::SMALL_SIZE)
                        .color(ui_theme::TEXT),
                );
                ui.end_row();
            }
        });
}

fn bevy_color(c: egui::Color32) -> Color {
    Color::srgb_u8(c.r(), c.g(), c.b())
}

fn spawn_title_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!("spawning title art (REF-10) on an Oxblood / Inkblood backdrop");
    commands.spawn((
        Sprite {
            image: asset_server.load(SPRITE_TITLE),
            // Corrected to the pixel-exact size by `fit_title_art`.
            custom_size: Some(title_world_size(2.0, 720.0, VIEW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
        TitleArt,
        TitleArtImage,
    ));
    // Backdrop halves split at the art's horizontal centre line: the sky
    // colour of the art's top edge above, the horizon's Inkblood below
    // (it shows through the art's transparent lower bands too).
    for (color, y) in [
        (ui_theme::OXBLOOD, BACKDROP_EXTENT * 0.5),
        (ui_theme::INKBLOOD, -BACKDROP_EXTENT * 0.5),
    ] {
        commands.spawn((
            Sprite::from_color(bevy_color(color), Vec2::splat(BACKDROP_EXTENT)),
            Transform::from_xyz(0.0, y, 99.0),
            TitleArt,
        ));
    }
}

/// Keep the art at a whole-number pixel scale for the current window.
fn fit_title_art(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut art: Query<&mut Sprite, With<TitleArtImage>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (pw, ph) = (
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    let size = title_world_size(title_integer_scale(pw, ph), ph, VIEW_HEIGHT);
    for mut sprite in art.iter_mut() {
        if sprite.custom_size != Some(size) {
            tracing::debug!(pw, ph, ?size, "title art resized to integer scale");
            sprite.custom_size = Some(size);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_scale_is_the_largest_that_fits() {
        assert_eq!(title_integer_scale(1280.0, 720.0), 2.0);
        assert_eq!(title_integer_scale(1920.0, 1080.0), 3.0);
        assert_eq!(title_integer_scale(1366.0, 768.0), 2.0);
        assert_eq!(title_integer_scale(2560.0, 1080.0), 3.0);
        // Never below 1.
        assert_eq!(title_integer_scale(320.0, 200.0), 1.0);
    }

    #[test]
    fn world_size_maps_to_exact_physical_pixels() {
        // 720 px tall window, 280-unit view: 2x art is 720 px = 280 units.
        let s = title_world_size(2.0, 720.0, 280.0);
        assert!((s.y - 280.0).abs() < 1e-3);
        assert!((s.x - 280.0 * 16.0 / 9.0).abs() < 1e-3);
        // Degenerate window height falls back to native size.
        assert_eq!(title_world_size(2.0, 0.0, 280.0), Vec2::new(640.0, 360.0));
    }
}
