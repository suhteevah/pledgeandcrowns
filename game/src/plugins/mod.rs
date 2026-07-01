// SPDX-License-Identifier: MIT
//! Plugin assembly. Each subsystem is its own Bevy plugin so integration
//! tests can mount them in isolation. See `design/05-tech-architecture.md`
//! section 4 for the planned layout.

use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub mod audio;
pub mod compile_client;
pub mod editor;
pub mod mission;
pub mod npc;
pub mod player;
pub mod progress;
pub mod quest_marker;
pub mod screenshot;
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
            quest_marker::QuestMarkerPlugin,
            screenshot::ShotPlugin,
            editor::EditorPlugin,
            mission::MissionPlugin,
            compile_client::CompileClientPlugin,
            audio::GameAudioPlugin,
        ));
    }
}

/// Camera vertical view in world units. Zoomed out from the original 180 so the
/// designed village reads as a town at a glance instead of through a keyhole.
pub const VIEW_HEIGHT: f32 = 280.0;
/// Primary-window aspect ratio (see `lib.rs` WindowPlugin resolution 1280x720).
/// Combined with [`VIEW_HEIGHT`] to clamp the follow-camera in `player`.
pub const VIEW_ASPECT: f32 = 1280.0 / 720.0;

fn setup_camera(mut commands: Commands) {
    tracing::info!(
        "spawning 2d camera (fixed vertical projection, {VIEW_HEIGHT} world units tall)"
    );
    let mut proj = OrthographicProjection::default_2d();
    proj.scaling_mode = ScalingMode::FixedVertical {
        viewport_height: VIEW_HEIGHT,
    };
    commands.spawn((Camera2d, Projection::Orthographic(proj)));
}
