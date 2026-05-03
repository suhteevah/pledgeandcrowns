// SPDX-License-Identifier: MIT
//! Title screen. REF-10 painted backdrop scaled to fill the window;
//! Space transitions to GameState::InGame.

use crate::assets::SPRITE_TITLE;
use crate::plugins::mission::MissionRegistry;
use crate::plugins::progress::MissionProgress;
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

#[derive(Component)]
struct TitleArt;

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("TitlePlugin::build");
        app.add_systems(OnEnter(GameState::Title), spawn_title_art)
            .add_systems(OnExit(GameState::Title), despawn_title_art)
            .add_systems(Update, start_on_space.run_if(in_state(GameState::Title)))
            .add_systems(
                EguiPrimaryContextPass,
                draw_title_overlay.run_if(in_state(GameState::Title)),
            );
    }
}

fn draw_title_overlay(
    mut contexts: EguiContexts,
    progress: Res<MissionProgress>,
    registry: Res<MissionRegistry>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let cleared = progress.cleared_count();
    let total = registry.missions.len();

    egui::Window::new("title_prompt")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -48.0))
        .show(ctx, |ui| {
            ui.heading("Press [Space] to begin");
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

fn despawn_title_art(mut commands: Commands, query: Query<Entity, With<TitleArt>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

fn start_on_space(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        tracing::info!("title -> InGame");
        next.set(GameState::InGame);
    }
}
