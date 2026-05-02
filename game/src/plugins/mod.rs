// SPDX-License-Identifier: MIT
//! Plugin assembly. Each subsystem is its own Bevy plugin so integration
//! tests can mount them in isolation. See `design/05-tech-architecture.md`
//! section 4 for the planned layout.

use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub mod compile_client;
pub mod editor;
pub mod mission;
pub mod npc;
pub mod player;
pub mod progress;
pub mod state;
pub mod stub_grader;
pub mod title;
pub mod world;

/// Bootstrap plugin. Spawns the camera and mounts the day-6 subsystems
/// (world background + player). Editor, compile-client, sandbox, town,
/// combat, party, persistence, audio land in subsequent days.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("CorePlugin::build");
        app.add_systems(Startup, setup_camera).add_plugins((
            state::StatePlugin,
            title::TitlePlugin,
            progress::ProgressPlugin,
            world::WorldPlugin,
            player::PlayerPlugin,
            npc::NpcPlugin,
            editor::EditorPlugin,
            mission::MissionPlugin,
            compile_client::CompileClientPlugin,
        ));
    }
}

fn setup_camera(mut commands: Commands) {
    tracing::info!("spawning 2d camera (fixed vertical projection, 180 world units tall)");
    let mut proj = OrthographicProjection::default_2d();
    proj.scaling_mode = ScalingMode::FixedVertical {
        viewport_height: 180.0,
    };
    commands.spawn((Camera2d, Projection::Orthographic(proj)));
}
