// SPDX-License-Identifier: MIT
//! NPCs scattered around Hearthstone Village. Each NPC carries a
//! mission ID; standing within INTERACT_RADIUS and pressing F opens
//! the dialogue UI bound to that mission.
//!
//! Mission registry + dialogue + editor handoff live in `mission.rs`.
//! This plugin only owns NPC spawning and proximity detection.

use crate::assets::{SPRITE_BORROW_CHECKER, SPRITE_FERRIS, SPRITE_PLAYER};
use crate::plugins::player::Player;
use crate::plugins::state::GameState;
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
            .add_systems(OnEnter(GameState::InGame), spawn_npcs)
            .add_systems(
                Update,
                detect_nearby_npc.run_if(in_state(GameState::InGame)),
            );
    }
}

/// Static description of the NPCs that get spawned. Tests read this
/// to assert every NPC's mission_id resolves to a real mission.
pub const NPC_ROSTER: &[NpcSpec] = &[
    NpcSpec {
        name: "Ferris",
        mission_id: "intro_let_binding",
        sprite_path: SPRITE_FERRIS,
        pos: (48.0, 0.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "Trait Mage",
        mission_id: "double_function",
        // Placeholder art until the Trait Mage REF lands.
        sprite_path: SPRITE_PLAYER,
        pos: (-48.0, 32.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Borrow Checker",
        mission_id: "borrow_preview",
        sprite_path: SPRITE_BORROW_CHECKER,
        pos: (96.0, -64.0),
        native_px: 64.0,
    },
    NpcSpec {
        name: "The Smith",
        mission_id: "mut_binding",
        sprite_path: SPRITE_PLAYER, // placeholder until Smith art lands
        pos: (-96.0, -32.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Cartographer",
        mission_id: "if_else_sign",
        sprite_path: SPRITE_PLAYER, // placeholder
        pos: (0.0, 96.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Bellringer",
        mission_id: "loop_break",
        sprite_path: SPRITE_PLAYER, // placeholder until Bellringer art lands
        pos: (144.0, 32.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Oracle",
        mission_id: "match_option",
        sprite_path: SPRITE_PLAYER, // placeholder
        pos: (-144.0, 0.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Herald",
        mission_id: "struct_basic",
        sprite_path: SPRITE_PLAYER, // placeholder
        pos: (-32.0, -96.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Cooper",
        mission_id: "vec_iter",
        sprite_path: SPRITE_PLAYER, // placeholder
        pos: (160.0, -32.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Twin",
        mission_id: "tuple_destructure",
        sprite_path: SPRITE_PLAYER, // placeholder
        pos: (32.0, -64.0),
        native_px: 32.0,
    },
    NpcSpec {
        name: "The Tinker",
        mission_id: "while_loop",
        sprite_path: SPRITE_PLAYER, // placeholder
        pos: (-160.0, 64.0),
        native_px: 32.0,
    },
];

pub struct NpcSpec {
    pub name: &'static str,
    pub mission_id: &'static str,
    pub sprite_path: &'static str,
    pub pos: (f32, f32),
    pub native_px: f32,
}

fn spawn_npcs(mut commands: Commands, asset_server: Res<AssetServer>) {
    for spec in NPC_ROSTER {
        spawn_npc(
            &mut commands,
            &asset_server,
            spec.name,
            spec.mission_id,
            spec.sprite_path,
            Vec2::new(spec.pos.0, spec.pos.1),
            spec.native_px,
        );
    }
    tracing::info!("spawned {} NPCs", NPC_ROSTER.len());
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
