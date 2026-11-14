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
//! - **Two channels.** Music beds play on [`MusicChannel`], one-shots on
//!   [`SfxChannel`], so the settings screen can mix them independently.
//!   Channel volume is `master x channel` (see [`volume_db`]) and is
//!   re-applied whenever [`Settings`] changes, so sliders act live.
//!   Pausing does not touch audio: `PauseState` is a sub-state of
//!   `InGame`, so `OnExit(GameState::InGame)` (which stops music) only
//!   fires on Quit to Title.

use crate::assets::{
    AUDIO_EDITOR_OPEN, AUDIO_EPILOGUE, AUDIO_MISSION_CLEAR, AUDIO_MISSION_LOCKED, AUDIO_TITLE,
    AUDIO_VILLAGE,
};
use crate::plugins::editor::EditorState;
use crate::plugins::mission::{
    CompletionView, EpilogueView, MissionLockedAttempt, MissionRegistry,
};
use crate::plugins::progress::{MissionProgress, Settings};
use crate::plugins::state::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// Typed `bevy_kira_audio` channel for looping music beds.
#[derive(Resource)]
pub struct MusicChannel;

/// Typed `bevy_kira_audio` channel for one-shot sound effects.
#[derive(Resource)]
pub struct SfxChannel;

pub type MusicAudio = AudioChannel<MusicChannel>;
pub type SfxAudio = AudioChannel<SfxChannel>;

/// Map a master and a channel slider (each 0..=100 percent) to the
/// channel gain in decibels. The two percentages multiply into a linear
/// amplitude, which is converted with `20 * log10(amp)`; anything at or
/// below kira's silence floor (-60 dB), including either slider at 0,
/// returns exactly `Decibels::SILENCE`.
pub fn volume_db(master: u8, channel: u8) -> f32 {
    let silence = Decibels::SILENCE.0;
    let m = f32::from(master.min(Settings::VOLUME_MAX)) / 100.0;
    let c = f32::from(channel.min(Settings::VOLUME_MAX)) / 100.0;
    let amp = m * c;
    if amp <= 0.0 {
        return silence;
    }
    (20.0 * amp.log10()).max(silence)
}

/// Cached `Handle<AudioSource>`s for every track we ever play. Loaded
/// once at startup; missing files yield handles that silently drop on
/// play.
#[derive(Resource, Default)]
pub struct AudioHandles {
    pub title: Handle<AudioSource>,
    pub village: Handle<AudioSource>,
    pub mission_clear: Handle<AudioSource>,
    pub mission_locked: Handle<AudioSource>,
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
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<SfxChannel>()
            .init_resource::<AudioHandles>()
            .init_resource::<AudioState>()
            .add_systems(Startup, (load_audio_handles, init_audio_state).chain())
            .add_systems(OnEnter(GameState::Title), play_title_music)
            .add_systems(OnExit(GameState::Title), stop_all_music)
            .add_systems(OnEnter(GameState::InGame), play_village_music)
            .add_systems(OnExit(GameState::InGame), stop_all_music)
            .add_systems(
                Update,
                apply_volume_settings.run_if(resource_exists_and_changed::<Settings>),
            )
            .add_systems(
                Update,
                (
                    sfx_on_mission_clear,
                    sfx_on_locked_attempt,
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
    handles.mission_locked = asset_server.load(AUDIO_MISSION_LOCKED);
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

/// Push the current [`Settings`] into both channels. Runs on the first
/// frame (resource added counts as changed) and on every slider move.
fn apply_volume_settings(settings: Res<Settings>, music: Res<MusicAudio>, sfx: Res<SfxAudio>) {
    let music_db = volume_db(settings.master_volume, settings.music_volume);
    let sfx_db = volume_db(settings.master_volume, settings.sfx_volume);
    tracing::debug!(
        master = settings.master_volume,
        music = settings.music_volume,
        sfx = settings.sfx_volume,
        music_db,
        sfx_db,
        "audio: applying volume settings"
    );
    music.set_volume(music_db);
    sfx.set_volume(sfx_db);
}

fn play_title_music(audio: Res<MusicAudio>, handles: Res<AudioHandles>) {
    tracing::info!("audio: playing title loop");
    audio.play(handles.title.clone()).looped();
}

fn play_village_music(audio: Res<MusicAudio>, handles: Res<AudioHandles>) {
    tracing::info!("audio: playing village ambient loop");
    audio.play(handles.village.clone()).looped();
}

fn stop_all_music(audio: Res<MusicAudio>) {
    tracing::debug!("audio: stopping music channel");
    audio.stop();
}

fn sfx_on_mission_clear(
    audio: Res<SfxAudio>,
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

/// Fire the `mission_locked` sting when the player F-presses a locked
/// NPC. Driven by the `MissionLockedAttempt` message emitted in
/// `mission::handle_interact_key` — no state-edge tracking needed since
/// the message itself is the edge. Drain the reader and play at most
/// once per frame so a single F-press never stacks the sting.
fn sfx_on_locked_attempt(
    audio: Res<SfxAudio>,
    handles: Res<AudioHandles>,
    mut attempts: MessageReader<MissionLockedAttempt>,
) {
    let mut fired = false;
    for attempt in attempts.read() {
        if !fired {
            tracing::debug!("audio: mission_locked sting ({})", attempt.mission_id);
            audio.play(handles.mission_locked.clone());
            fired = true;
        }
    }
}

fn sfx_on_editor_open(
    audio: Res<SfxAudio>,
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
    audio: Res<SfxAudio>,
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
