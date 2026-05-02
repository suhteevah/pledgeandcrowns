// SPDX-License-Identifier: MIT
//! Audit: minimal-app boot smoke tests. Each plugin that doesn't need
//! a window or renderer is mounted on a `MinimalPlugins` App, ticked
//! once, and asserted not to panic and to have published its
//! resources.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use pledge_and_crown::plugins::mission::{ActiveMission, MissionPlugin, MissionRegistry};
use pledge_and_crown::plugins::progress::{MissionProgress, ProgressPlugin};
use pledge_and_crown::plugins::state::{GameState, StatePlugin};

/// MissionPlugin includes an Egui draw system; we can mount the plugin
/// to verify resources, but the draw system never fires without
/// EguiPlugin's schedule. Bevy is fine with the schedule being absent.
#[test]
fn progress_plugin_publishes_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ProgressPlugin);
    app.update();
    let progress = app.world().resource::<MissionProgress>();
    assert_eq!(progress.cleared_count(), 0);
}

#[test]
fn state_plugin_starts_in_title() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(StatePlugin);
    app.update();
    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::Title);
}

#[test]
fn mission_plugin_publishes_registry() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(MissionPlugin);
    app.update();
    let registry = app.world().resource::<MissionRegistry>();
    // Lower-bound check rather than exact: curriculum grows over time.
    // The registry/contract suites enforce per-mission invariants.
    assert!(
        registry.missions.len() >= 5,
        "MissionRegistry should ship at least the 5 prelude missions, got {}",
        registry.missions.len()
    );
    let active = app.world().resource::<ActiveMission>();
    assert!(
        active.current.is_none(),
        "ActiveMission must default to None"
    );
}

#[test]
fn mission_progress_round_trip() {
    let mut p = MissionProgress::default();
    assert_eq!(p.cleared_count(), 0);
    p.mark_cleared("intro_let_binding");
    assert!(p.is_cleared("intro_let_binding"));
    assert_eq!(p.cleared_count(), 1);
    // Idempotent: marking again does not double-count.
    p.mark_cleared("intro_let_binding");
    assert_eq!(p.cleared_count(), 1);
    p.mark_cleared("double_function");
    assert_eq!(p.cleared_count(), 2);
    assert!(!p.is_cleared("borrow_preview"));
}

#[test]
fn mission_progress_disk_round_trip() {
    // Use a per-test subdir under the system temp dir so parallel runs
    // don't stomp on each other. The validator pins the filename to
    // `save.bincode` — we honor that here.
    let dir = std::env::temp_dir().join(format!(
        "pledge-and-crown-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("save.bincode");

    let mut p = MissionProgress::default();
    p.mark_cleared("intro_let_binding");
    p.mark_cleared("double_function");
    p.save_to(&path).expect("save_to should succeed");

    let loaded = MissionProgress::load_from(&path).expect("load_from should succeed");
    assert_eq!(loaded.cleared_count(), 2);
    assert!(loaded.is_cleared("intro_let_binding"));
    assert!(loaded.is_cleared("double_function"));
    assert!(!loaded.is_cleared("borrow_preview"));

    // Loading from a missing path is a fresh state, not an error.
    let missing = dir.join("does_not_exist").join("save.bincode");
    let fresh = MissionProgress::load_from(&missing).unwrap();
    assert_eq!(fresh.cleared_count(), 0);

    // Filename validation rejects anything other than `save.bincode`.
    let bad = dir.join("etc-passwd");
    let err = MissionProgress::load_from(&bad).unwrap_err();
    assert!(
        err.to_string().contains("save.bincode"),
        "rejection message should name the required filename, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
