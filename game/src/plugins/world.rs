// SPDX-License-Identifier: MIT
//! World rendering — background sprite + the camera that frames it.
//!
//! For MVP day-6 we use the REF-09 world-map painting as a single
//! large background sprite (native 320x180, scaled 4x to fill the
//! 1280x720 window). A real tilemap (`bevy_ecs_tilemap`) lands when
//! Hearthstone Village is implemented.

use bevy::prelude::*;

const BG_SCALE: f32 = 4.0;
const BG_NATIVE_W: f32 = 320.0;
const BG_NATIVE_H: f32 = 180.0;

/// World bounds in world-space (centered on origin).
pub const WORLD_HALF_W: f32 = BG_NATIVE_W * BG_SCALE * 0.5;
pub const WORLD_HALF_H: f32 = BG_NATIVE_H * BG_SCALE * 0.5;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("WorldPlugin::build");
        app.add_systems(Startup, spawn_world);
    }
}

fn spawn_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!("spawning world background (REF-09 worldmap, scale {BG_SCALE}x)");
    commands.spawn((
        Sprite {
            image: asset_server.load("sprites/worldmap.png"),
            custom_size: Some(Vec2::new(BG_NATIVE_W * BG_SCALE, BG_NATIVE_H * BG_SCALE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
