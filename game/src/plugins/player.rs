// SPDX-License-Identifier: MIT
//! Player entity, sprite, and 4-directional movement.
//!
//! Native sprite resolution is 32x32 (REF-01); displayed at scale 2x.
//! Movement is WASD or arrow keys. Speed is constant for MVP — no
//! animation states yet. Bounded by `world::WORLD_HALF_*` so the
//! player can't walk off the painted background.

use crate::plugins::world::{WORLD_HALF_H, WORLD_HALF_W};
use bevy::prelude::*;

const PLAYER_SCALE: f32 = 2.0;
const PLAYER_NATIVE: f32 = 32.0;
const PLAYER_DISPLAY: f32 = PLAYER_NATIVE * PLAYER_SCALE;
const PLAYER_SPEED_PX_PER_SEC: f32 = 180.0;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("PlayerPlugin::build");
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, move_player);
    }
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!("spawning player at origin (REF-01, scale {PLAYER_SCALE}x)");
    commands.spawn((
        Sprite {
            image: asset_server.load("sprites/player.png"),
            custom_size: Some(Vec2::splat(PLAYER_DISPLAY)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        Player,
    ));
}

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    if dir == Vec2::ZERO {
        return;
    }

    let delta = dir.normalize() * PLAYER_SPEED_PX_PER_SEC * time.delta_secs();
    transform.translation.x =
        (transform.translation.x + delta.x).clamp(-WORLD_HALF_W, WORLD_HALF_W);
    transform.translation.y =
        (transform.translation.y + delta.y).clamp(-WORLD_HALF_H, WORLD_HALF_H);
}
