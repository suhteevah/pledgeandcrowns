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

/// localStorage key the wasm build reads/writes. Versioned alongside
/// `SAVE_VERSION` — bumping the schema means picking a new key so
/// players' v1 saves don't try to decode as v2.
#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY: &str = "pledge-and-crown:save:v1";

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
    ///
    /// Wasm: filesystem isn't available; this is a no-op. Progress is
    /// kept in-memory for the tab's lifetime. A localStorage path is
    /// the planned next step (gated on the wasm image-binding bug
    /// shipping first — no point persisting progress in a build that
    /// can't render).
    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    pub fn save_to(&self, _path: &Path) -> anyhow::Result<()> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let file = SaveFile {
            version: SAVE_VERSION,
            cleared: self.cleared.iter().cloned().collect(),
        };
        let bytes = bincode::serde::encode_to_vec(&file, bincode::config::standard())
            .map_err(|e| anyhow::anyhow!("bincode encode: {e}"))?;
        let encoded = STANDARD.encode(&bytes);
        let storage = local_storage()?;
        storage
            .set_item(LOCAL_STORAGE_KEY, &encoded)
            .map_err(|e| anyhow::anyhow!("localStorage set_item failed: {e:?}"))?;
        tracing::debug!(
            bytes = bytes.len(),
            encoded_chars = encoded.len(),
            "localStorage save written"
        );
        Ok(())
    }

    /// Load from disk. Returns `Ok(default)` if the file is missing
    /// (first run); errors only on corruption / version mismatch /
    /// unreadable I/O.
    ///
    /// Wasm: returns default (fresh-start). See `save_to` for the
    /// persistence-on-web roadmap note.
    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    pub fn load_from(_path: &Path) -> anyhow::Result<Self> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let storage = match local_storage() {
            Ok(s) => s,
            Err(e) => {
                // localStorage unavailable (Safari private mode, headless,
                // restricted environment). Not an error — start fresh.
                tracing::warn!("localStorage unavailable: {e}; starting fresh");
                return Ok(Self::default());
            }
        };
        let encoded = match storage.get_item(LOCAL_STORAGE_KEY) {
            Ok(Some(v)) => v,
            Ok(None) => {
                tracing::debug!("no localStorage save found; starting fresh");
                return Ok(Self::default());
            }
            Err(e) => {
                tracing::warn!("localStorage get_item failed: {e:?}; starting fresh");
                return Ok(Self::default());
            }
        };
        let bytes = STANDARD
            .decode(&encoded)
            .map_err(|e| anyhow::anyhow!("base64 decode: {e}"))?;
        let (file, _): (SaveFile, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .map_err(|e| anyhow::anyhow!("bincode decode: {e}"))?;
        if file.version != SAVE_VERSION {
            anyhow::bail!(
                "save version mismatch: localStorage has v{}, current schema is v{}",
                file.version,
                SAVE_VERSION
            );
        }
        let cleared: HashSet<String> = file.cleared.into_iter().collect();
        tracing::info!(cleared = cleared.len(), "localStorage save loaded");
        Ok(Self { cleared })
    }
}

/// Resolve the browser's localStorage handle. Returns Err if the
/// window or storage object is missing — caller decides whether that's
/// fatal (save) or fall-through-to-default (load).
#[cfg(target_arch = "wasm32")]
fn local_storage() -> anyhow::Result<web_sys::Storage> {
    let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("no `window` global"))?;
    window
        .local_storage()
        .map_err(|e| anyhow::anyhow!("window.localStorage threw: {e:?}"))?
        .ok_or_else(|| anyhow::anyhow!("window.localStorage is null"))
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
#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILENAME: &str = "save.bincode";

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::env;

    fn temp_save_path() -> PathBuf {
        // Per-test directory under the system temp dir. Uniqueness via
        // nanos + thread id keeps parallel test runners from clobbering
        // each other.
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

        // Cleanup — best-effort, test passes even if it can't remove.
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let path = temp_save_path();
        // Don't create anything at the path.
        let loaded = MissionProgress::load_from(&path).expect("missing file is not an error");
        assert_eq!(loaded.cleared_count(), 0);
    }

    #[test]
    fn save_path_filename_is_locked() {
        let bad = std::env::temp_dir().join("not-the-right-name.txt");
        let p = MissionProgress::default();
        let err = p
            .save_to(&bad)
            .expect_err("non-canonical filename must be rejected");
        let msg = format!("{err}");
        assert!(
            msg.contains("save.bincode"),
            "error should mention the canonical filename: {msg}"
        );
    }

    #[test]
    fn corrupt_save_load_is_an_error() {
        let path = temp_save_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"this is not bincode at all").unwrap();
        let result = MissionProgress::load_from(&path);
        assert!(
            result.is_err(),
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
}
