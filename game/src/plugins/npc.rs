// SPDX-License-Identifier: MIT
//! NPCs scattered around Hearthstone Village. Each NPC carries a
//! mission ID; standing within INTERACT_RADIUS and pressing F opens
//! the dialogue UI bound to that mission.
//!
//! Mission registry + dialogue + editor handoff live in `mission.rs`.
//! This plugin only owns NPC spawning and proximity detection.

use crate::plugins::player::Player;
use bevy::prelude::*;

const NPC_SCALE: f32 = 1.5;
const INTERACT_RADIUS_PX: f32 = 28.0;

#[derive(Component)]
pub struct Npc {
    pub name: &'static str,
    pub mission_id: &'static str,
}

/// Set by the proximity system each frame. `mission.rs` reads this
/// to know whether to show the interaction prompt and route F-presses.
#[derive(Resource, Default)]
pub struct NearbyNpc {
    pub current: Option<NearbyEntry>,
}

#[derive(Clone)]
pub struct NearbyEntry {
    pub name: &'static str,
    pub mission_id: &'static str,
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("NpcPlugin::build");
        app.init_resource::<NearbyNpc>()
            .add_systems(Startup, spawn_npcs)
            .add_systems(Update, detect_nearby_npc);
    }
}

fn spawn_npcs(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Ferris — the friendly tutorial guide. Just east of spawn on the road.
    spawn_npc(
        &mut commands,
        &asset_server,
        "Ferris",
        "intro_let_binding",
        "sprites/ferris.png",
        Vec2::new(48.0, 0.0),
        32.0,
    );

    // Trait Mage — Act 1 second mission, by the well plaza.
    spawn_npc(
        &mut commands,
        &asset_server,
        "Trait Mage",
        "double_function",
        "sprites/player.png", // placeholder — Trait Mage art arrives later
        Vec2::new(-48.0, 32.0),
        32.0,
    );

    // The Borrow Checker — south of road as a lurking ominous figure
    // for the Act 2 boss preview mission.
    spawn_npc(
        &mut commands,
        &asset_server,
        "The Borrow Checker",
        "borrow_preview",
        "sprites/borrow_checker.png",
        Vec2::new(96.0, -64.0),
        64.0,
    );

    tracing::info!("spawned 3 NPCs");
}

fn spawn_npc(
    commands: &mut Commands,
    asset_server: &AssetServer,
    name: &'static str,
    mission_id: &'static str,
    sprite_path: &'static str,
    pos: Vec2,
    native_px: f32,
) {
    let display = native_px * NPC_SCALE;
    commands.spawn((
        Sprite {
            image: asset_server.load(sprite_path),
            custom_size: Some(Vec2::splat(display)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
        Npc { name, mission_id },
    ));
}

fn detect_nearby_npc(
    player_q: Query<&Transform, With<Player>>,
    npc_q: Query<(&Transform, &Npc)>,
    mut nearby: ResMut<NearbyNpc>,
) {
    let Ok(player) = player_q.single() else {
        nearby.current = None;
        return;
    };
    let player_xy = player.translation.truncate();

    let mut best: Option<(f32, NearbyEntry)> = None;
    for (transform, npc) in npc_q.iter() {
        let dist = transform.translation.truncate().distance(player_xy);
        if dist <= INTERACT_RADIUS_PX && best.as_ref().map(|(d, _)| dist < *d).unwrap_or(true) {
            best = Some((
                dist,
                NearbyEntry {
                    name: npc.name,
                    mission_id: npc.mission_id,
                },
            ));
        }
    }
    nearby.current = best.map(|(_, e)| e);
}
