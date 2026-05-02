// SPDX-License-Identifier: MIT
//! Plugin assembly. Each subsystem is its own Bevy plugin so that
//! integration tests can mount them in isolation. See
//! `design/05-tech-architecture.md` section 4 for the planned layout.

use bevy::prelude::*;

/// Bootstrap plugin. Currently just spawns a 2D camera so the window
/// renders something. Real subsystems (world, party, combat, editor,
/// compile_client, sandbox, town, persistence, audio) land here as
/// they get implemented.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("CorePlugin::build");
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    tracing::info!("spawning 2d camera");
    commands.spawn(Camera2d);
}
