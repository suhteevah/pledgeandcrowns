// SPDX-License-Identifier: MIT
//! Plugin assembly. Each subsystem is its own Bevy plugin so integration
//! tests can mount them in isolation. See `design/05-tech-architecture.md`
//! section 4 for the planned layout.

use bevy::prelude::*;

pub mod editor;
pub mod player;
pub mod world;

/// Bootstrap plugin. Spawns the camera and mounts the day-6 subsystems
/// (world background + player). Editor, compile-client, sandbox, town,
/// combat, party, persistence, audio land in subsequent days.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("CorePlugin::build");
        app.add_systems(Startup, setup_camera).add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            editor::EditorPlugin,
        ));
    }
}

fn setup_camera(mut commands: Commands) {
    tracing::info!("spawning 2d camera");
    commands.spawn(Camera2d);
}
