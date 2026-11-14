// SPDX-License-Identifier: MIT
//! Save game: cleared missions, settings and the player's last position.
//! Persisted with `bincode 2` to a file on native and to `localStorage`
//! on wasm.
//!
//! ## Format and migration
//!
//! Every save starts with a `version: u32`. [`SaveData::decode`] peeks at
//! that header and dispatches:
//!
//! - **v1** (`{ version, cleared }`): migrated losslessly — the cleared set
//!   carries over, settings take their defaults, no saved position.
//! - **v2** (`{ version, cleared, settings, player_position }`): current.
//! - anything else: [`SaveError::UnsupportedVersion`].
//!
//! The wire structs (`SaveFileV1`, `SaveFileV2`, `SettingsV2`) are frozen.
//! Adding a setting means adding a field to [`Settings`], a new
//! `SaveFileV3`, bumping [`SAVE_VERSION`] and adding a migration arm —
//! old files keep decoding through their own frozen struct.
//!
//! ## Failure policy
//!
//! A missing save is a fresh start. An unreadable/corrupt save is never
//! fatal and never silently discarded: it is logged at `warn`, moved aside
//! (`save.bincode.corrupt-<unix-secs>` on native, a `:corrupt` key on
//! wasm) and the game starts fresh. If that backup fails, saving is
//! disabled for the session so the broken file is not overwritten.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::plugins::player::Player;
use crate::plugins::state::GameState;

/// Current on-disk schema version. See the module docs for migration.
pub const SAVE_VERSION: u32 = 2;

/// How often the in-game autosave fires to capture the player's position
/// (mission clears and menu actions save immediately).
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(30);

/// localStorage key for v2+ saves. v1 saves live under the old key and
/// are read (and migrated) when no v2 save exists yet.
#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY: &str = "pledge-and-crown:save:v2";
#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY_V1: &str = "pledge-and-crown:save:v1";
#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY_CORRUPT: &str = "pledge-and-crown:save:corrupt";

/// The only filename this module is willing to read/write. Defense in
/// depth: callers can't point save/load at an arbitrary file.
const SAVE_FILENAME: &str = "save.bincode";

/// Everything that can go wrong reading or writing a save.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("save path has no filename component")]
    NoFilename,
    #[error("save path filename must be `{SAVE_FILENAME}`, got `{0}`")]
    BadFilename(String),
    #[error("save I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("save encode failed: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("save decode failed: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("unsupported save version v{found} (this build reads v1..=v{SAVE_VERSION})")]
    UnsupportedVersion { found: u32 },
    #[error("save has {extra} trailing bytes after the v{version} payload")]
    TrailingBytes { version: u32, extra: usize },
    #[error("browser storage: {0}")]
    Storage(String),
}

// ---------------------------------------------------------------------------
// In-memory model (Bevy resources)
// ---------------------------------------------------------------------------

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct MissionProgress {
    cleared: HashSet<String>,
}

impl MissionProgress {
    pub fn mark_cleared(&mut self, encounter_id: &str) {
        if self.cleared.insert(encounter_id.to_string()) {
            tracing::info!("mission cleared: {encounter_id}");
        }
    }
    pub fn is_cleared(&self, encounter_id: &str) -> bool {
        self.cleared.contains(encounter_id)
    }
    pub fn cleared_count(&self) -> usize {
        self.cleared.len()
    }
    /// Forget every clear (New Game).
    pub fn reset(&mut self) {
        tracing::info!(was = self.cleared.len(), "mission progress reset");
        self.cleared.clear();
    }

    /// Write a save holding just this progress (default settings, no
    /// position). Kept for tests/tools; the game itself saves through
    /// [`SaveData`] so settings and position are not clobbered.
    pub fn save_to(&self, path: &Path) -> Result<(), SaveError> {
        SaveData {
            progress: self.clone(),
            ..SaveData::default()
        }
        .save_to(path)
    }

    /// Load the progress part of a save. Missing file → empty progress.
    pub fn load_from(path: &Path) -> Result<Self, SaveError> {
        SaveData::load_from(path).map(|d| d.progress)
    }
}

/// Player-tunable settings. Volumes are percentages, 0..=100.
///
/// Adding a setting: add the field here, add a [`SettingKind`] row so the
/// settings screen renders it, and add a new save version (see module
/// docs) so it persists.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub master_volume: u8,
    pub music_volume: u8,
    pub sfx_volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            master_volume: 80,
            music_volume: 70,
            sfx_volume: 80,
        }
    }
}

/// One row on the settings screen. The menu iterates [`SettingKind::ALL`],
/// so a new row only needs a variant here plus its `get`/`set` arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKind {
    MasterVolume,
    MusicVolume,
    SfxVolume,
}

impl SettingKind {
    pub const ALL: [SettingKind; 3] = [
        SettingKind::MasterVolume,
        SettingKind::MusicVolume,
        SettingKind::SfxVolume,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SettingKind::MasterVolume => "Master volume",
            SettingKind::MusicVolume => "Music volume",
            SettingKind::SfxVolume => "SFX volume",
        }
    }
}

impl Settings {
    pub const VOLUME_MAX: u8 = 100;

    pub fn get(&self, kind: SettingKind) -> u8 {
        match kind {
            SettingKind::MasterVolume => self.master_volume,
            SettingKind::MusicVolume => self.music_volume,
            SettingKind::SfxVolume => self.sfx_volume,
        }
    }

    /// Set a value, clamped to its valid range.
    pub fn set(&mut self, kind: SettingKind, value: u8) {
        let v = value.min(Self::VOLUME_MAX);
        match kind {
            SettingKind::MasterVolume => self.master_volume = v,
            SettingKind::MusicVolume => self.music_volume = v,
            SettingKind::SfxVolume => self.sfx_volume = v,
        }
    }

    /// Copy with every value clamped into range (applied to decoded saves
    /// so a hand-edited file can't push a volume past 100).
    pub fn clamped(mut self) -> Self {
        for kind in SettingKind::ALL {
            self.set(kind, self.get(kind));
        }
        self
    }
}

/// Where the player should stand when the world next spawns. Loaded from
/// the save, kept current while playing, cleared by New Game. `None` means
/// the default spawn point.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct SavedPlayerPosition(pub Option<Vec2>);

/// One complete save snapshot.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SaveData {
    pub progress: MissionProgress,
    pub settings: Settings,
    pub player_position: Option<Vec2>,
}

// ---------------------------------------------------------------------------
// Wire format (frozen per version)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct VersionHeader {
    version: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SaveFileV1 {
    version: u32,
    cleared: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SettingsV2 {
    master_volume: u8,
    music_volume: u8,
    sfx_volume: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct SaveFileV2 {
    version: u32,
    cleared: Vec<String>,
    settings: SettingsV2,
    player_position: Option<[f32; 2]>,
}

fn bincode_config() -> bincode::config::Configuration {
    bincode::config::standard()
}

/// Decode a whole buffer as `T`, rejecting trailing garbage.
fn decode_exact<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    version: u32,
) -> Result<T, SaveError> {
    let (value, read) = bincode::serde::decode_from_slice::<T, _>(bytes, bincode_config())?;
    if read != bytes.len() {
        return Err(SaveError::TrailingBytes {
            version,
            extra: bytes.len() - read,
        });
    }
    Ok(value)
}

impl SaveData {
    /// True when there is something worth continuing: a cleared mission
    /// or a remembered position.
    pub fn has_progress(&self) -> bool {
        self.progress.cleared_count() > 0 || self.player_position.is_some()
    }

    /// Serialize as the current version (v2). The cleared set is sorted so
    /// identical saves produce identical bytes.
    pub fn encode(&self) -> Result<Vec<u8>, SaveError> {
        let mut cleared: Vec<String> = self.progress.cleared.iter().cloned().collect();
        cleared.sort();
        let file = SaveFileV2 {
            version: SAVE_VERSION,
            cleared,
            settings: SettingsV2 {
                master_volume: self.settings.master_volume,
                music_volume: self.settings.music_volume,
                sfx_volume: self.settings.sfx_volume,
            },
            player_position: self.player_position.map(|p| [p.x, p.y]),
        };
        Ok(bincode::serde::encode_to_vec(&file, bincode_config())?)
    }

    /// Deserialize any supported version, migrating older ones.
    pub fn decode(bytes: &[u8]) -> Result<Self, SaveError> {
        let (header, _) =
            bincode::serde::decode_from_slice::<VersionHeader, _>(bytes, bincode_config())?;
        tracing::debug!(
            version = header.version,
            bytes = bytes.len(),
            "decoding save"
        );
        match header.version {
            1 => {
                let v1: SaveFileV1 = decode_exact(bytes, 1)?;
                tracing::info!(cleared = v1.cleared.len(), "migrating v1 save to v2");
                Ok(Self {
                    progress: MissionProgress {
                        cleared: v1.cleared.into_iter().collect(),
                    },
                    settings: Settings::default(),
                    player_position: None,
                })
            }
            2 => {
                let v2: SaveFileV2 = decode_exact(bytes, 2)?;
                let player_position = match v2.player_position {
                    Some([x, y]) if x.is_finite() && y.is_finite() => Some(Vec2::new(x, y)),
                    Some(bad) => {
                        tracing::warn!(?bad, "save has a non-finite position; ignoring it");
                        None
                    }
                    None => None,
                };
                Ok(Self {
                    progress: MissionProgress {
                        cleared: v2.cleared.into_iter().collect(),
                    },
                    settings: Settings {
                        master_volume: v2.settings.master_volume,
                        music_volume: v2.settings.music_volume,
                        sfx_volume: v2.settings.sfx_volume,
                    }
                    .clamped(),
                    player_position,
                })
            }
            found => Err(SaveError::UnsupportedVersion { found }),
        }
    }
}

// ---------------------------------------------------------------------------
// Native persistence
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn validated_save_path(path: &Path) -> Result<PathBuf, SaveError> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(SaveError::NoFilename)?;
    if name != SAVE_FILENAME {
        return Err(SaveError::BadFilename(name.to_string()));
    }
    Ok(path.to_path_buf())
}

#[cfg(not(target_arch = "wasm32"))]
impl SaveData {
    /// Write atomically-ish: `<path>.tmp` then rename, so a crash
    /// mid-write never corrupts the live save.
    pub fn save_to(&self, path: &Path) -> Result<(), SaveError> {
        let path = validated_save_path(path)?;
        let bytes = self.encode()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("bincode.tmp");
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, &path)?;
        tracing::debug!(?path, bytes = bytes.len(), "save written");
        Ok(())
    }

    /// Load from disk. Missing file → `Ok(default)`; corrupt or unknown
    /// version → `Err` (see [`SaveData::load_or_recover`] for the policy
    /// the game applies).
    pub fn load_from(path: &Path) -> Result<Self, SaveError> {
        let path = validated_save_path(path)?;
        if !path.exists() {
            tracing::debug!(?path, "no save found; starting fresh");
            return Ok(Self::default());
        }
        let bytes = std::fs::read(&path)?;
        let data = Self::decode(&bytes)?;
        tracing::info!(
            ?path,
            cleared = data.progress.cleared_count(),
            has_position = data.player_position.is_some(),
            "save loaded"
        );
        Ok(data)
    }

    /// Load, and on any failure back the bad file up next to the save and
    /// start fresh. Never panics, never silently drops the old bytes.
    pub fn load_or_recover(path: &Path) -> LoadOutcome {
        match Self::load_from(path) {
            Ok(data) => LoadOutcome {
                data,
                writes_enabled: true,
            },
            Err(e) => {
                tracing::warn!(?path, "failed to load save: {e}; starting fresh");
                let writes_enabled = match backup_corrupt_save(path) {
                    Ok(Some(backup)) => {
                        tracing::warn!(?backup, "unreadable save backed up");
                        true
                    }
                    Ok(None) => true,
                    Err(be) => {
                        tracing::error!(
                            ?path,
                            "could not back up unreadable save ({be}); \
                             saving disabled this session to avoid overwriting it"
                        );
                        false
                    }
                };
                LoadOutcome {
                    data: Self::default(),
                    writes_enabled,
                }
            }
        }
    }
}

/// Move an unreadable save aside as `save.bincode.corrupt-<unix-secs>`.
/// Returns the backup path, or `None` if there was nothing to move.
#[cfg(not(target_arch = "wasm32"))]
fn backup_corrupt_save(path: &Path) -> Result<Option<PathBuf>, SaveError> {
    let path = validated_save_path(path)?;
    if !path.exists() {
        return Ok(None);
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut backup = path.with_file_name(format!("{SAVE_FILENAME}.corrupt-{stamp}"));
    let mut n = 1u32;
    while backup.exists() {
        backup = path.with_file_name(format!("{SAVE_FILENAME}.corrupt-{stamp}-{n}"));
        n += 1;
    }
    std::fs::rename(&path, &backup)?;
    Ok(Some(backup))
}

// ---------------------------------------------------------------------------
// Wasm persistence (localStorage)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
impl SaveData {
    pub fn save_to(&self, _path: &Path) -> Result<(), SaveError> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let bytes = self.encode()?;
        let encoded = STANDARD.encode(&bytes);
        local_storage()?
            .set_item(LOCAL_STORAGE_KEY, &encoded)
            .map_err(|e| SaveError::Storage(format!("set_item failed: {e:?}")))?;
        tracing::debug!(bytes = bytes.len(), "localStorage save written");
        Ok(())
    }

    /// Reads the v2 key, falling back to the v1 key (migrated). Storage
    /// being unavailable (private mode, headless) is a fresh start.
    pub fn load_from(_path: &Path) -> Result<Self, SaveError> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let Some(encoded) = read_raw_save() else {
            tracing::debug!("no localStorage save found; starting fresh");
            return Ok(Self::default());
        };
        let bytes = STANDARD
            .decode(&encoded)
            .map_err(|e| SaveError::Storage(format!("base64 decode: {e}")))?;
        let data = Self::decode(&bytes)?;
        tracing::info!(
            cleared = data.progress.cleared_count(),
            "localStorage save loaded"
        );
        Ok(data)
    }

    pub fn load_or_recover(path: &Path) -> LoadOutcome {
        match Self::load_from(path) {
            Ok(data) => LoadOutcome {
                data,
                writes_enabled: true,
            },
            Err(e) => {
                tracing::warn!("failed to load localStorage save: {e}; starting fresh");
                let writes_enabled = match (read_raw_save(), local_storage()) {
                    (Some(raw), Ok(storage)) => {
                        match storage.set_item(LOCAL_STORAGE_KEY_CORRUPT, &raw) {
                            Ok(()) => {
                                tracing::warn!(
                                    key = LOCAL_STORAGE_KEY_CORRUPT,
                                    "unreadable save backed up"
                                );
                                true
                            }
                            Err(be) => {
                                tracing::error!(
                                    "could not back up unreadable save ({be:?}); \
                                     saving disabled this session"
                                );
                                false
                            }
                        }
                    }
                    _ => true,
                };
                LoadOutcome {
                    data: Self::default(),
                    writes_enabled,
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn read_raw_save() -> Option<String> {
    let storage = match local_storage() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("localStorage unavailable: {e}");
            return None;
        }
    };
    for key in [LOCAL_STORAGE_KEY, LOCAL_STORAGE_KEY_V1] {
        match storage.get_item(key) {
            Ok(Some(v)) => return Some(v),
            Ok(None) => {}
            Err(e) => tracing::warn!(key, "localStorage get_item failed: {e:?}"),
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Result<web_sys::Storage, SaveError> {
    let window = web_sys::window().ok_or_else(|| SaveError::Storage("no `window`".into()))?;
    window
        .local_storage()
        .map_err(|e| SaveError::Storage(format!("window.localStorage threw: {e:?}")))?
        .ok_or_else(|| SaveError::Storage("window.localStorage is null".into()))
}

/// Result of [`SaveData::load_or_recover`].
#[derive(Debug)]
pub struct LoadOutcome {
    pub data: SaveData,
    /// False when an unreadable save could not be backed up; the session
    /// then never writes, so the original bytes survive for inspection.
    pub writes_enabled: bool,
}

/// Resolve the on-disk save path. Windows: `%APPDATA%/pledge-and-crown/save.bincode`.
/// Unix: `$XDG_DATA_HOME` (or `~/.local/share`)`/pledge-and-crown/save.bincode`.
/// Falls back to the current directory if none of those resolve.
pub fn default_save_path() -> PathBuf {
    let dir = if let Ok(p) = std::env::var("APPDATA") {
        PathBuf::from(p)
    } else if let Ok(p) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(p)
    } else if let Ok(p) = std::env::var("HOME") {
        PathBuf::from(p).join(".local").join("share")
    } else {
        PathBuf::from(".")
    };
    dir.join("pledge-and-crown").join(SAVE_FILENAME)
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct SavePath(pub PathBuf);

/// Whether this session may write the save. Off when an unreadable save
/// couldn't be backed up, and in screenshot mode (`PNC_SHOTS`), whose
/// scripted teleports must not overwrite the player's real position.
#[derive(Resource, Debug, Clone, Copy)]
pub struct SavePolicy {
    pub writes_enabled: bool,
}

/// Send to write the save at the end of this frame (menus use it on
/// pause, on leaving Settings, on New Game and on Quit to Title).
#[derive(Message, Debug, Clone, Copy, Default)]
pub struct SaveRequest;

#[derive(Resource)]
struct AutosaveTimer(Timer);

pub struct ProgressPlugin;

impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("ProgressPlugin::build");
        let path = default_save_path();
        let outcome = SaveData::load_or_recover(&path);
        let shot_mode = std::env::var("PNC_SHOTS").is_ok();
        let writes_enabled = outcome.writes_enabled && !shot_mode;
        if shot_mode {
            tracing::info!("screenshot mode: save writes disabled");
        }
        app.insert_resource(outcome.data.progress)
            .insert_resource(outcome.data.settings)
            .insert_resource(SavedPlayerPosition(outcome.data.player_position))
            .insert_resource(SavePath(path))
            .insert_resource(SavePolicy { writes_enabled })
            .insert_resource(AutosaveTimer(Timer::new(
                AUTOSAVE_INTERVAL,
                TimerMode::Repeating,
            )))
            .add_message::<SaveRequest>()
            .add_systems(
                Update,
                (
                    restore_player_position,
                    track_player_position,
                    tick_autosave,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(Last, persist_save);
    }
}

/// When the player entity appears, move it (and snap the camera) to the
/// saved position. Runs from `ProgressPlugin` so `player.rs` needs no
/// changes: `OnEnter(InGame)` runs in `StateTransition`, before `Update`,
/// so `Added<Player>` is seen on the very frame the player spawns.
fn restore_player_position(
    saved: Res<SavedPlayerPosition>,
    mut player_q: Query<&mut Transform, (Added<Player>, Without<Camera2d>)>,
    mut cam_q: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Some(pos) = saved.0 else {
        return;
    };
    for mut t in player_q.iter_mut() {
        t.translation.x = pos.x;
        t.translation.y = pos.y;
        tracing::info!(?pos, "restored player to saved position");
        if let Ok(mut cam) = cam_q.single_mut() {
            cam.translation.x = pos.x;
            cam.translation.y = pos.y;
        }
    }
}

/// Mirror the live player position into [`SavedPlayerPosition`] so every
/// save (and a later Continue) uses where the player actually is.
fn track_player_position(
    player_q: Query<&Transform, With<Player>>,
    mut saved: ResMut<SavedPlayerPosition>,
) {
    let Ok(t) = player_q.single() else {
        return;
    };
    let now = Some(t.translation.truncate());
    if saved.0 != now {
        saved.0 = now;
    }
}

fn tick_autosave(
    time: Res<Time>,
    mut timer: ResMut<AutosaveTimer>,
    mut requests: MessageWriter<SaveRequest>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        tracing::debug!("periodic autosave");
        requests.write(SaveRequest);
    }
}

/// Write the save when progress changes, when a [`SaveRequest`] arrives,
/// or when the app is exiting. The initial insert (`is_added`) does not
/// count as a change, so merely booting never rewrites the file.
#[allow(clippy::too_many_arguments)]
fn persist_save(
    progress: Res<MissionProgress>,
    settings: Res<Settings>,
    position: Res<SavedPlayerPosition>,
    save_path: Res<SavePath>,
    policy: Res<SavePolicy>,
    mut requests: MessageReader<SaveRequest>,
    mut exits: MessageReader<AppExit>,
) {
    let progress_changed = progress.is_changed() && !progress.is_added();
    let requested = requests.read().count() > 0;
    let exiting = exits.read().count() > 0;
    if !(progress_changed || requested || exiting) {
        return;
    }
    if !policy.writes_enabled {
        tracing::debug!(
            progress_changed,
            requested,
            exiting,
            "save skipped: writes disabled"
        );
        return;
    }
    let data = SaveData {
        progress: progress.clone(),
        settings: *settings,
        player_position: position.0,
    };
    match data.save_to(&save_path.0) {
        Ok(()) => tracing::debug!(progress_changed, requested, exiting, "autosave ok"),
        Err(e) => tracing::warn!("autosave failed: {e}"),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::env;

    fn temp_save_path() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tid = format!("{:?}", std::thread::current().id());
        let safe_tid: String = tid.chars().filter(|c| c.is_alphanumeric()).collect();
        env::temp_dir()
            .join(format!("pledge-progress-test-{nanos}-{safe_tid}"))
            .join(SAVE_FILENAME)
    }

    #[test]
    fn round_trip_preserves_cleared_set() {
        let path = temp_save_path();
        let mut p = MissionProgress::default();
        p.mark_cleared("intro_let_binding");
        p.mark_cleared("borrow_preview");
        p.save_to(&path).expect("save should succeed");

        let loaded = MissionProgress::load_from(&path).expect("load should succeed");
        assert!(loaded.is_cleared("intro_let_binding"));
        assert!(loaded.is_cleared("borrow_preview"));
        assert!(!loaded.is_cleared("never_cleared"));
        assert_eq!(loaded.cleared_count(), 2);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let path = temp_save_path();
        let loaded = MissionProgress::load_from(&path).expect("missing file is not an error");
        assert_eq!(loaded.cleared_count(), 0);
    }

    #[test]
    fn save_path_filename_is_locked() {
        let bad = std::env::temp_dir().join("not-the-right-name.txt");
        let err = MissionProgress::default()
            .save_to(&bad)
            .expect_err("non-canonical filename must be rejected");
        assert!(
            err.to_string().contains("save.bincode"),
            "error should mention the canonical filename: {err}"
        );
    }

    #[test]
    fn corrupt_save_load_is_an_error() {
        let path = temp_save_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"this is not bincode at all").unwrap();
        assert!(
            MissionProgress::load_from(&path).is_err(),
            "corrupt bincode must error, not silently default"
        );
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn save_overwrites_existing_atomically() {
        let path = temp_save_path();
        let mut p1 = MissionProgress::default();
        p1.mark_cleared("a");
        p1.save_to(&path).unwrap();

        let mut p2 = MissionProgress::default();
        p2.mark_cleared("a");
        p2.mark_cleared("b");
        p2.save_to(&path).unwrap();

        let loaded = MissionProgress::load_from(&path).unwrap();
        assert_eq!(loaded.cleared_count(), 2);
        assert!(loaded.is_cleared("a"));
        assert!(loaded.is_cleared("b"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn encode_is_deterministic() {
        let mut d = SaveData::default();
        for id in ["c", "a", "b"] {
            d.progress.mark_cleared(id);
        }
        assert_eq!(d.encode().unwrap(), d.clone().encode().unwrap());
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut bytes = SaveData::default().encode().unwrap();
        bytes.push(0);
        assert!(matches!(
            SaveData::decode(&bytes),
            Err(SaveError::TrailingBytes { version: 2, .. })
        ));
    }

    #[test]
    fn non_finite_position_is_dropped() {
        let d = SaveData {
            player_position: Some(Vec2::new(f32::NAN, 1.0)),
            ..SaveData::default()
        };
        let back = SaveData::decode(&d.encode().unwrap()).unwrap();
        assert_eq!(back.player_position, None);
    }

    #[test]
    fn settings_set_clamps() {
        let mut s = Settings::default();
        s.set(SettingKind::MusicVolume, 250);
        assert_eq!(s.music_volume, 100);
    }
}
