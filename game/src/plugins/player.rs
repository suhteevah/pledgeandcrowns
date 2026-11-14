// SPDX-License-Identifier: MIT
//! Player entity, sprite, and 4-directional movement.
//!
//! Native sprite resolution is 32x32 (REF-01); displayed at scale 1x, i.e.
//! 2 tiles tall, in proportion to the 16px village tiles (was 2x = 4 tiles,
//! which dwarfed the world).
//! Movement is WASD or arrow keys. Speed is constant for MVP — no
//! animation states yet. Bounded by `world::WORLD_HALF_*` so the
//! player can't walk off the painted background.

use crate::assets::SPRITE_PLAYER;
use crate::plugins::prop::{Aabb, Solid, slide_move, ysort_z};
use crate::plugins::state::GameState;
use crate::plugins::world::{WORLD_HALF_H, WORLD_HALF_W};
use bevy::prelude::*;

/// The player's collision footprint relative to the sprite centre: a small
/// box at the feet (the sprite is 32px, centre-anchored), so the head and
/// shoulders can overlap a roof or canopy while the feet stop at the wall.
/// Public so the screenshot plugin's collision probes can assert against it.
pub const PLAYER_FOOT: Aabb = Aabb::from_base(0.0, 6.0, -16.0, -10.0);

/// Camera lerp factor per second; 1.0 = snap, 0.0 = never moves.
const CAMERA_FOLLOW_SPEED: f32 = 6.0;

const PLAYER_SCALE: f32 = 1.0;
const PLAYER_NATIVE: f32 = 32.0;
const PLAYER_DISPLAY: f32 = PLAYER_NATIVE * PLAYER_SCALE;
// Scaled up with the village: the 180x140 map is huge, so a brisk walk keeps
// crossing between far-flung quarters from being a slog.
const PLAYER_SPEED_PX_PER_SEC: f32 = 420.0;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("PlayerPlugin::build");
        app.add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(
                Update,
                (move_player, camera_follow_player)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn camera_follow_player(
    time: Res<Time>,
    player_q: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut cam_q: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(player) = player_q.single() else {
        return;
    };
    let Ok(mut cam) = cam_q.single_mut() else {
        return;
    };
    let target = player.translation.truncate();
    let current = cam.translation.truncate();
    let alpha = (CAMERA_FOLLOW_SPEED * time.delta_secs()).clamp(0.0, 1.0);
    let next = current.lerp(target, alpha);

    // Clamp the camera centre so the void beyond the tilemap never enters the
    // viewport. Half-extents derive from the camera's FixedVertical view, so
    // this stays correct if the zoom (`plugins::VIEW_HEIGHT`) changes.
    let view_half = Vec2::new(
        crate::plugins::VIEW_HEIGHT * crate::plugins::VIEW_ASPECT * 0.5,
        crate::plugins::VIEW_HEIGHT * 0.5,
    );
    let max = Vec2::new(WORLD_HALF_W - view_half.x, WORLD_HALF_H - view_half.y);
    let min = -max;
    cam.translation.x = next.x.clamp(min.x.min(0.0), max.x.max(0.0));
    cam.translation.y = next.y.clamp(min.y.min(0.0), max.y.max(0.0));
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!("spawning player at origin (REF-01, scale {PLAYER_SCALE}x)");
    commands.spawn((
        Sprite {
            image: asset_server.load(SPRITE_PLAYER),
            custom_size: Some(Vec2::splat(PLAYER_DISPLAY)),
            ..default()
        },
        // z is owned by prop::ysort from here on; seed it consistently.
        Transform::from_xyz(0.0, 0.0, ysort_z(-PLAYER_DISPLAY * 0.5)),
        Player,
    ));
}

/// Public (unchanged behaviour) so the screenshot plugin's headless
/// collision test can run the real movement system in a minimal `App`.
pub fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    editor: Res<crate::plugins::editor::EditorState>,
    mut query: Query<&mut Transform, With<Player>>,
    solids_q: Query<&Solid>,
) {
    // Don't bleed gameplay movement through to the editor — typing
    // WASD in your code would otherwise still walk the player around
    // in the background while you were trying to write Rust.
    if editor.open {
        return;
    }

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
    let pos = transform.translation.truncate();
    let solids: Vec<Aabb> = solids_q.iter().map(|s| s.0).collect();
    let moved = slide_move(pos, delta, PLAYER_FOOT, &solids);
    if (moved - (pos + delta)).length_squared() > 1e-6 {
        tracing::trace!(?pos, ?delta, ?moved, "player movement clipped by a prop");
    }
    transform.translation.x = moved.x.clamp(-WORLD_HALF_W, WORLD_HALF_W);
    transform.translation.y = moved.y.clamp(-WORLD_HALF_H, WORLD_HALF_H);
}
