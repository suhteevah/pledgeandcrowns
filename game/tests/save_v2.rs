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

/// A v2 save written before the Borrow Checker's trial existed (its
/// cleared and opened sets hold only pre-trial mission ids) loads
/// unchanged, and the trial slots into its progression: trial stages are
/// ordinary missions keyed by id, so no format change was needed.
#[test]
fn pre_trial_v2_save_loads_unchanged_and_offers_the_trial() {
    use pledge_and_crown::plugins::mission::MissionRegistry;
    use pledge_and_crown::plugins::trial;

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
    #[derive(Serialize)]
    struct Ext {
        magic: [u8; 4],
        opened: Vec<String>,
    }
    let reg = MissionRegistry::default();
    // Everything the old registry had up to and including `impl_method`
    // (the old order: the 21 Acts 1-2 missions, then Act 3).
    let mut cleared: Vec<String> = reg
        .missions
        .iter()
        .map(|m| m.id)
        .filter(|id| trial::trial_of(id).is_none())
        .take(22)
        .map(str::to_string)
        .collect();
    assert_eq!(cleared.last().map(String::as_str), Some("impl_method"));
    cleared.sort();
    let opened = vec!["assoc_new".to_string(), "borrow_preview".to_string()];
    let mut bytes = bincode::serde::encode_to_vec(
        &V2 {
            version: 2,
            cleared: cleared.clone(),
            settings: S {
                a: 70,
                b: 60,
                c: 50,
            },
            player_position: Some([12.5, -3.0]),
        },
        bincode::config::standard(),
    )
    .unwrap();
    bytes.extend(
        bincode::serde::encode_to_vec(
            &Ext {
                magic: *b"PNCx",
                opened: opened.clone(),
            },
            bincode::config::standard(),
        )
        .unwrap(),
    );

    let data = SaveData::decode(&bytes).expect("pre-trial v2 save must load");
    let p = &data.progress;
    assert_eq!(p.cleared_count(), cleared.len());
    for id in &cleared {
        assert!(p.is_cleared(id), "lost {id}");
    }
    for id in &opened {
        assert!(p.is_opened(id), "lost opened {id}");
    }
    assert_eq!(data.player_position, Some(Vec2::new(12.5, -3.0)));
    assert_eq!(data.settings.master_volume, 70);
    // Re-encoding and decoding again changes nothing.
    assert_eq!(SaveData::decode(&data.encode().unwrap()).unwrap(), data);

    // The trial is the next thing to do; the Act 3 save point stays usable.
    assert_eq!(reg.next_locked(p).map(|m| m.id), Some("bridge_toll"));
    assert_eq!(
        trial::current_mission(&reg, p, "borrow_preview").map(|m| m.id),
        Some("bridge_toll")
    );
    let unlocked = |id: &str| reg.find(id).is_some_and(|m| m.is_unlocked(p));
    assert!(unlocked("bridge_toll"));
    assert!(!unlocked("bridge_quill"));
    // A cleared impl_method still unlocks the next Act 3 mission (which a
    // demo build does not include at all).
    assert_eq!(unlocked("assoc_new"), !cfg!(feature = "demo"));
}

/// A v2 save written before the Act 3-5 trials existed, cleared past the
/// end of Act 5 (every mission of the 60-mission registry up to and
/// including `hashmap_basic`, Act 6's first), loads unchanged. Each trial
/// stage is an ordinary mission keyed by id, so no format change was
/// needed: every trial's stage 1 is unlocked and is what its owner offers,
/// stage 2 stays locked, and the Act 6 progress already made stays usable.
#[test]
fn save_cleared_past_acts_3_to_5_offers_each_new_trial() {
    use pledge_and_crown::plugins::mission::MissionRegistry;
    use pledge_and_crown::plugins::trial;

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
    let reg = MissionRegistry::default();
    let new_stage = |id: &str| trial::trial_of(id).is_some_and(|(t, _)| t.id != "bridge");
    // The old registry order: every current mission except the new stages.
    let mut cleared = Vec::new();
    for m in reg.missions.iter().filter(|m| !new_stage(m.id)) {
        cleared.push(m.id.to_string());
        if m.id == "hashmap_basic" {
            break;
        }
    }
    assert_eq!(
        cleared.len(),
        43,
        "the old Acts 1-5 plus Act 6's first mission"
    );
    let bytes = bincode::serde::encode_to_vec(
        &V2 {
            version: 2,
            cleared: cleared.clone(),
            settings: S {
                a: 80,
                b: 60,
                c: 40,
            },
            player_position: Some([-4.0, 9.5]),
        },
        bincode::config::standard(),
    )
    .unwrap();

    let data = SaveData::decode(&bytes).expect("pre-trial v2 save must load");
    let p = &data.progress;
    assert_eq!(p.cleared_count(), cleared.len());
    for id in &cleared {
        assert!(p.is_cleared(id), "lost {id}");
    }
    assert_eq!(data.player_position, Some(Vec2::new(-4.0, 9.5)));
    assert_eq!(SaveData::decode(&data.encode().unwrap()).unwrap(), data);

    let unlocked = |id: &str| reg.find(id).is_some_and(|m| m.is_unlocked(p));
    let full = !cfg!(feature = "demo");
    for (owner, stage1, stage2) in [
        ("impl_method", "wraith_forms", "wraith_essence"),
        ("trait_def", "chimera_heads", "chimera_bound"),
        ("result_match", "wager_stake", "wager_from"),
    ] {
        let offered = trial::current_mission(&reg, p, owner).map(|m| m.id);
        if full {
            assert_eq!(offered, Some(stage1), "{owner} offers its trial");
        } else {
            // The demo has none of these missions: the owner stays its prologue.
            assert_eq!(offered, Some(owner), "{owner} in the demo");
        }
        assert_eq!(unlocked(stage1), full, "{stage1}");
        assert!(!unlocked(stage2), "{stage2} waits for {stage1}");
    }
    // The earliest unfinished mission is the first new trial.
    if full {
        assert_eq!(reg.next_locked(p).map(|m| m.id), Some("wraith_forms"));
    }
    // Progress already made in Act 6 stays usable.
    assert_eq!(unlocked("iter_filter"), full);
}

/// A v2 save written before the Act 6-8 trials existed, cleared past the
/// end of Act 8 (every mission of the 69-mission registry, ending on
/// `weak_ref`), loads unchanged. No format change: each new trial's stage
/// 1 is unlocked and is what its owner offers, stage 2 stays locked, and
/// the earliest unfinished mission is the Forge-Golem's first stage.
#[test]
fn save_cleared_past_act_8_offers_each_new_trial() {
    use pledge_and_crown::plugins::mission::MissionRegistry;
    use pledge_and_crown::plugins::trial;

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
    const NEW_TRIALS: [&str; 3] = ["forge_golem", "tide_hydra", "memory_warden"];
    let reg = MissionRegistry::default();
    let new_stage = |id: &str| trial::trial_of(id).is_some_and(|(t, _)| NEW_TRIALS.contains(&t.id));
    // The old registry: every current mission except the new stages.
    let cleared: Vec<String> = reg
        .missions
        .iter()
        .filter(|m| !new_stage(m.id))
        .map(|m| m.id.to_string())
        .collect();
    assert_eq!(cleared.len(), 69, "the whole 69-mission registry");
    assert_eq!(cleared.last().map(String::as_str), Some("weak_ref"));
    let bytes = bincode::serde::encode_to_vec(
        &V2 {
            version: 2,
            cleared: cleared.clone(),
            settings: S {
                a: 70,
                b: 50,
                c: 30,
            },
            player_position: Some([12.0, -3.5]),
        },
        bincode::config::standard(),
    )
    .unwrap();

    let data = SaveData::decode(&bytes).expect("pre-trial v2 save must load");
    let p = &data.progress;
    assert_eq!(p.cleared_count(), cleared.len());
    for id in &cleared {
        assert!(p.is_cleared(id), "lost {id}");
    }
    assert_eq!(data.player_position, Some(Vec2::new(12.0, -3.5)));
    assert_eq!(SaveData::decode(&data.encode().unwrap()).unwrap(), data);

    let unlocked = |id: &str| reg.find(id).is_some_and(|m| m.is_unlocked(p));
    let full = !cfg!(feature = "demo");
    for (owner, stage1, stage2) in [
        ("hashmap_basic", "golem_parts", "golem_sum"),
        ("thread_spawn", "hydra_senders", "hydra_threads"),
        ("box_basic", "warden_box", "warden_shared"),
    ] {
        let offered = trial::current_mission(&reg, p, owner).map(|m| m.id);
        if full {
            assert_eq!(offered, Some(stage1), "{owner} offers its trial");
        } else {
            // The demo has none of these missions: the owner stays its prologue.
            assert_eq!(offered, Some(owner), "{owner} in the demo");
        }
        assert_eq!(unlocked(stage1), full, "{stage1}");
        assert!(!unlocked(stage2), "{stage2} waits for {stage1}");
    }
    if full {
        assert_eq!(reg.next_locked(p).map(|m| m.id), Some("golem_parts"));
    }
    // Not everything is cleared any more: no epilogue for this save.
    assert!(p.cleared_count() < reg.missions.len());
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
