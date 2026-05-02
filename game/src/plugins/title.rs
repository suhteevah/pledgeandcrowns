// SPDX-License-Identifier: MIT
//! Title screen. REF-10 painted backdrop scaled to fill the window;
//! Space transitions to GameState::InGame.

use crate::plugins::state::GameState;
use bevy::prelude::*;

const TITLE_NATIVE_W: f32 = 640.0;
const TITLE_NATIVE_H: f32 = 360.0;
const TITLE_SCALE: f32 = 2.0;

#[derive(Component)]
struct TitleArt;

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("TitlePlugin::build");
        app.add_systems(OnEnter(GameState::Title), spawn_title_art)
            .add_systems(OnExit(GameState::Title), despawn_title_art)
            .add_systems(Update, start_on_space.run_if(in_state(GameState::Title)));
    }
}

fn spawn_title_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!("spawning title art (REF-10)");
    commands.spawn((
        Sprite {
            image: asset_server.load("sprites/title.png"),
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
