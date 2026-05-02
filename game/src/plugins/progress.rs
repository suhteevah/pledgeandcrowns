// SPDX-License-Identifier: MIT
//! Tracks which missions the player has cleared. Persisted to disk via
//! `bincode 2` so progression survives across runs. Save format is
//! versioned; future schema changes bump `SAVE_VERSION` and add a
//! migration arm in `load_from`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Bumped any time `SaveFile`'s on-disk shape changes. `load_from`
/// rejects anything it doesn't recognize so we never partially-decode
/// stale data into a current-shape struct.
const SAVE_VERSION: u32 = 1;

#[derive(Resource, Default, Debug)]
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

    /// Serialize to disk via bincode 2. Creates parent dirs if needed.
    /// Atomic-ish: writes to `<path>.tmp` then renames, so a crash
    /// mid-write doesn't corrupt the live save.
    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        let path = validated_save_path(path)?;
        let file = SaveFile {
            version: SAVE_VERSION,
            cleared: self.cleared.iter().cloned().collect(),
        };
        let bytes = bincode::serde::encode_to_vec(&file, bincode::config::standard())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("bincode.tmp");
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, &path)?;
        tracing::debug!(?path, bytes = bytes.len(), "save written");
        Ok(())
    }

    /// Load from disk. Returns `Ok(default)` if the file is missing
    /// (first run); errors only on corruption / version mismatch /
    /// unreadable I/O.
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let path = validated_save_path(path)?;
        if !path.exists() {
            tracing::debug!(?path, "no save found; starting fresh");
            return Ok(Self::default());
        }
        let bytes = std::fs::read(&path)?;
        let (file, _): (SaveFile, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())?;
        if file.version != SAVE_VERSION {
            anyhow::bail!(
                "save version mismatch: file is v{}, current schema is v{}",
                file.version,
                SAVE_VERSION
            );
        }
        let mut cleared = HashSet::new();
        for id in file.cleared {
            cleared.insert(id);
        }
        tracing::info!(?path, cleared = cleared.len(), "save loaded");
        Ok(Self { cleared })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SaveFile {
    version: u32,
    cleared: Vec<String>,
}

/// The only filename this module is willing to read/write. Defense in
/// depth: callers can't accidentally (or maliciously) point save/load
/// at `/etc/passwd` or similar — the filename is fixed and rejected if
/// it doesn't match. The directory is still flexible (env-resolved in
/// production, tempdir in tests).
const SAVE_FILENAME: &str = "save.bincode";

fn validated_save_path(path: &Path) -> anyhow::Result<PathBuf> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("save path has no filename component"))?;
    if name != SAVE_FILENAME {
        anyhow::bail!("save path filename must be `{SAVE_FILENAME}`, got `{name}`");
    }
    Ok(path.to_path_buf())
}

/// Resolve the on-disk save path. Windows: `%APPDATA%/pledge-and-crown/save.bincode`.
/// Unix: `$XDG_DATA_HOME` (or `~/.local/share`)`/pledge-and-crown/save.bincode`.
/// Falls back to the current directory if none of those resolve — a
/// portable last resort that keeps tests/CI from blowing up in odd
/// sandboxes.
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
    dir.join("pledge-and-crown").join("save.bincode")
}

pub struct ProgressPlugin;

impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("ProgressPlugin::build");
        let path = default_save_path();
        let progress = MissionProgress::load_from(&path).unwrap_or_else(|e| {
            // Corrupt or version-mismatched save: log loudly and fall back
            // to a fresh state. Never silently use a broken save, never
            // panic the game over a save-file issue.
            tracing::warn!(?path, "failed to load save: {e}; starting fresh");
            MissionProgress::default()
        });
        app.insert_resource(progress)
            .insert_resource(SavePath(path))
            .add_systems(Update, autosave_on_change);
    }
}

#[derive(Resource)]
pub struct SavePath(pub PathBuf);

/// Persist whenever `MissionProgress` is mutated. Bevy's change
/// detection only fires on `ResMut` access, so this is effectively
/// "save on mark_cleared" for the cost of one comparison per frame.
fn autosave_on_change(progress: Res<MissionProgress>, save_path: Res<SavePath>) {
    if !progress.is_changed() {
        return;
    }
    if let Err(e) = progress.save_to(&save_path.0) {
        tracing::warn!("autosave failed: {e}");
    }
}
