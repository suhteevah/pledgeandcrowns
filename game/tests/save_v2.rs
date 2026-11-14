// SPDX-License-Identifier: MIT
//! Save v2: v1 -> v2 migration, v2 round trip, corrupt-save recovery.
//!
//! The v1 fixture is built here from a local struct with the exact v1
//! wire shape (`{ version: u32, cleared: Vec<String> }`), independent of
//! the crate's private wire types, so the test pins the real on-disk
//! layout that shipped.

use bevy::math::Vec2;
use pledge_and_crown::plugins::progress::{
    MissionProgress, SAVE_VERSION, SaveData, SaveError, SettingKind, Settings,
};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct V1Fixture {
    version: u32,
    cleared: Vec<String>,
}

fn v1_bytes(cleared: &[&str]) -> Vec<u8> {
    let file = V1Fixture {
        version: 1,
        cleared: cleared.iter().map(|s| s.to_string()).collect(),
    };
    bincode::serde::encode_to_vec(&file, bincode::config::standard()).expect("encode v1 fixture")
}

fn temp_save_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir()
        .join(format!(
            "pledge-save-v2-{tag}-{}-{nanos}",
            std::process::id()
        ))
        .join("save.bincode")
}

#[test]
fn current_version_is_two() {
    assert_eq!(SAVE_VERSION, 2);
}

#[test]
fn v1_save_migrates_losslessly() {
    let ids = ["intro_let_binding", "double_function", "borrow_preview"];
    let data = SaveData::decode(&v1_bytes(&ids)).expect("v1 must decode");
    assert_eq!(data.progress.cleared_count(), ids.len());
    for id in ids {
        assert!(data.progress.is_cleared(id), "lost {id} in migration");
    }
    assert_eq!(data.settings, Settings::default());
    assert_eq!(data.player_position, None);
}

#[test]
fn v1_file_on_disk_migrates_and_rewrites_as_v2() {
    let path = temp_save_path("v1disk");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, v1_bytes(&["intro_let_binding"])).unwrap();

    let loaded = SaveData::load_from(&path).expect("v1 file must load");
    assert!(loaded.progress.is_cleared("intro_let_binding"));

    // Saving writes v2; reloading yields the same data.
    loaded.save_to(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        bytes.first().copied(),
        Some(2),
        "first varint is the version"
    );
    assert_eq!(SaveData::load_from(&path).unwrap(), loaded);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn v2_round_trip_preserves_everything() {
    let mut progress = MissionProgress::default();
    progress.mark_cleared("intro_let_binding");
    progress.mark_cleared("thread_spawn");
    let mut settings = Settings::default();
    settings.set(SettingKind::MasterVolume, 55);
    settings.set(SettingKind::MusicVolume, 0);
    settings.set(SettingKind::SfxVolume, 100);
    let data = SaveData {
        progress,
        settings,
        player_position: Some(Vec2::new(-240.5, 312.25)),
    };

    let back = SaveData::decode(&data.encode().unwrap()).unwrap();
    assert_eq!(back, data);

    let path = temp_save_path("v2rt");
    data.save_to(&path).unwrap();
    assert_eq!(SaveData::load_from(&path).unwrap(), data);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn unknown_version_is_an_error() {
    let bytes = bincode::serde::encode_to_vec(
        &V1Fixture {
            version: 99,
            cleared: vec![],
        },
        bincode::config::standard(),
    )
    .unwrap();
    assert!(matches!(
        SaveData::decode(&bytes),
        Err(SaveError::UnsupportedVersion { found: 99 })
    ));
}

#[test]
fn truncated_v2_is_an_error_not_a_panic() {
    let mut bytes = SaveData::default().encode().unwrap();
    bytes.truncate(bytes.len() - 1);
    assert!(SaveData::decode(&bytes).is_err());
    assert!(SaveData::decode(&[]).is_err());
}

#[test]
fn corrupt_save_is_backed_up_and_game_starts_fresh() {
    let path = temp_save_path("corrupt");
    let dir = path.parent().unwrap().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap();
    let garbage = b"this is not bincode at all".to_vec();
    std::fs::write(&path, &garbage).unwrap();

    let outcome = SaveData::load_or_recover(&path);
    assert_eq!(outcome.data, SaveData::default(), "must start fresh");
    assert!(outcome.writes_enabled, "backup succeeded, saving stays on");
    assert!(
        !path.exists(),
        "corrupt file moved aside, not left in place"
    );

    let backups: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("save.bincode.corrupt-")
        })
        .collect();
    assert_eq!(backups.len(), 1, "exactly one backup expected");
    assert_eq!(
        std::fs::read(backups[0].path()).unwrap(),
        garbage,
        "backup must hold the original bytes"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_save_recovers_to_fresh_without_backup() {
    let path = temp_save_path("missing");
    let outcome = SaveData::load_or_recover(&path);
    assert_eq!(outcome.data, SaveData::default());
    assert!(outcome.writes_enabled);
    assert!(!path.parent().unwrap().exists(), "nothing created on load");
}

#[test]
fn out_of_range_settings_are_clamped_on_load() {
    // Hand-built v2 payload with a volume of 250.
    #[derive(Serialize)]
    struct S {
        a: u8,
        b: u8,
        c: u8,
    }
    #[derive(Serialize)]
    struct V2 {
        version: u32,
        cleared: Vec<String>,
        settings: S,
        player_position: Option<[f32; 2]>,
    }
    let bytes = bincode::serde::encode_to_vec(
        &V2 {
            version: 2,
            cleared: vec![],
            settings: S {
                a: 250,
                b: 10,
                c: 20,
            },
            player_position: None,
        },
        bincode::config::standard(),
    )
    .unwrap();
    let data = SaveData::decode(&bytes).unwrap();
    assert_eq!(data.settings.master_volume, 100);
    assert_eq!(data.settings.music_volume, 10);
    assert_eq!(data.settings.sfx_volume, 20);
}

#[test]
fn has_progress_reflects_clears_or_position() {
    let mut d = SaveData::default();
    assert!(!d.has_progress());
    d.player_position = Some(Vec2::ZERO);
    assert!(d.has_progress());
    d.player_position = None;
    d.progress.mark_cleared("intro_let_binding");
    assert!(d.has_progress());
}
