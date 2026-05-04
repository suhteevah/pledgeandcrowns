// SPDX-License-Identifier: MIT
//! Music + SFX. Backed by `bevy_kira_audio`. Tracks are baked offline
//! by `tools/synthwave-gen` (Stable Audio Open) and live under
//! `game/assets/audio/`.
//!
//! Design rules:
//! - **Fail open.** If a track file is missing (synthwave-gen hasn't
//!   run yet), the plugin's play calls become silent no-ops. The
//!   underlying AudioServer logs one warning per missing asset at
//!   startup; that's a feature — it tells you you haven't baked yet.
//! - **State-driven.** Title-state plays the title loop. InGame plays
//!   the village ambient loop. Transitions stop the previous bed.
//! - **Reactive SFX.** mission_clear fires when MissionProgress grows;
//!   editor_open fires on the false→true edge of EditorState.open;
//!   epilogue fires when the win-condition panel first becomes
//!   visible. All one-shots — no looping outside of music beds.

use crate::assets::{
    AUDIO_EDITOR_OPEN, AUDIO_EPILOGUE, AUDIO_MISSION_CLEAR, AUDIO_TITLE, AUDIO_VILLAGE,
};
use crate::plugins::editor::EditorState;
use crate::plugins::mission::{CompletionView, EpilogueView, MissionRegistry};
use crate::plugins::progress::MissionProgress;
use crate::plugins::state::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// Cached `Handle<AudioSource>`s for every track we ever play. Loaded
/// once at startup; missing files yield handles that silently drop on
/// play.
#[derive(Resource, Default)]
pub struct AudioHandles {
    pub title: Handle<AudioSource>,
    pub village: Handle<AudioSource>,
    pub mission_clear: Handle<AudioSource>,
    pub editor_open: Handle<AudioSource>,
    pub epilogue: Handle<AudioSource>,
}

/// Per-frame change tracking for one-shot SFX edges. Initialised in
/// `init_audio_state` so the first frame doesn't spuriously fire.
#[derive(Resource, Default)]
struct AudioState {
    last_cleared_count: usize,
    last_editor_open: bool,
    last_epilogue_visible: bool,
}

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("GameAudioPlugin::build");
        app.add_plugins(AudioPlugin)
            .init_resource::<AudioHandles>()
            .init_resource::<AudioState>()
            .add_systems(Startup, (load_audio_handles, init_audio_state).chain())
            .add_systems(OnEnter(GameState::Title), play_title_music)
            .add_systems(OnExit(GameState::Title), stop_all_music)
            .add_systems(OnEnter(GameState::InGame), play_village_music)
            .add_systems(OnExit(GameState::InGame), stop_all_music)
            .add_systems(
                Update,
                (
                    sfx_on_mission_clear,
                    sfx_on_editor_open,
                    sfx_on_epilogue_visible,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn load_audio_handles(asset_server: Res<AssetServer>, mut handles: ResMut<AudioHandles>) {
    tracing::info!("loading audio handles (missing files will warn-once and play silent)");
    handles.title = asset_server.load(AUDIO_TITLE);
    handles.village = asset_server.load(AUDIO_VILLAGE);
    handles.mission_clear = asset_server.load(AUDIO_MISSION_CLEAR);
    handles.editor_open = asset_server.load(AUDIO_EDITOR_OPEN);
    handles.epilogue = asset_server.load(AUDIO_EPILOGUE);
}

fn init_audio_state(progress: Res<MissionProgress>, mut state: ResMut<AudioState>) {
    // Seed last_cleared_count from the loaded save so re-entering the
    // game from a partially-completed save doesn't fire mission_clear
    // for every previously-cleared mission.
    state.last_cleared_count = progress.cleared_count();
    tracing::debug!(
        "audio state seeded: cleared_count={}",
        state.last_cleared_count
    );
}

fn play_title_music(audio: Res<Audio>, handles: Res<AudioHandles>) {
    tracing::info!("audio: playing title loop");
    audio.play(handles.title.clone()).looped();
}

fn play_village_music(audio: Res<Audio>, handles: Res<AudioHandles>) {
    tracing::info!("audio: playing village ambient loop");
    audio.play(handles.village.clone()).looped();
}

fn stop_all_music(audio: Res<Audio>) {
    tracing::debug!("audio: stopping all music");
    audio.stop();
}

fn sfx_on_mission_clear(
    audio: Res<Audio>,
    handles: Res<AudioHandles>,
    progress: Res<MissionProgress>,
    mut state: ResMut<AudioState>,
) {
    let now = progress.cleared_count();
    if now > state.last_cleared_count {
        tracing::info!(
            "audio: mission_clear sting ({} -> {})",
            state.last_cleared_count,
            now
        );
        audio.play(handles.mission_clear.clone());
    }
    state.last_cleared_count = now;
}

fn sfx_on_editor_open(
    audio: Res<Audio>,
    handles: Res<AudioHandles>,
    editor: Res<EditorState>,
    mut state: ResMut<AudioState>,
) {
    if editor.open && !state.last_editor_open {
        tracing::debug!("audio: editor_open sting");
        audio.play(handles.editor_open.clone());
    }
    state.last_editor_open = editor.open;
}

#[allow(clippy::too_many_arguments)]
fn sfx_on_epilogue_visible(
    audio: Res<Audio>,
    handles: Res<AudioHandles>,
    progress: Res<MissionProgress>,
    registry: Res<MissionRegistry>,
    epilogue: Res<EpilogueView>,
    completion: Res<CompletionView>,
    editor: Res<EditorState>,
    mut state: ResMut<AudioState>,
) {
    // Mirror draw_epilogue_panel's visibility predicate so the sting
    // fires exactly when the panel appears.
    let visible = !editor.open
        && completion.mission_id.is_none()
        && !epilogue.dismissed_this_session
        && progress.cleared_count() == registry.missions.len();
    if visible && !state.last_epilogue_visible {
        tracing::info!("audio: epilogue fanfare");
        audio.play(handles.epilogue.clone());
    }
    state.last_epilogue_visible = visible;
}
