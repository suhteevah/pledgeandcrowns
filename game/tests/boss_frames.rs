// SPDX-License-Identifier: MIT
//! Boss intro/defeat frames in a headless app: the frames follow the real
//! trial flow (F opens the trial offer, the editor, the server's clear, the
//! trial-complete panel), a boss without frames is never touched, a boss
//! already cleared at spawn stands idle, and pausing freezes the beats.
//!
//! The app has no `AssetServer`, so the sprite image is not swapped here;
//! the component's `shown` frame is what is checked (the swap itself is
//! seen in the `boss-*` capture scenes).

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use pledge_and_crown::plugins::boss_frames::{
    BOSS_FRAMES, BossFrame, BossFrames, BossFramesPlugin, DEFEAT_SECS, INTRO_SECS,
};
use pledge_and_crown::plugins::dialogue::{Dialogue, DialoguePlugin};
use pledge_and_crown::plugins::editor::EditorState;
use pledge_and_crown::plugins::mission::{CompletionView, MissionPlugin, MissionRegistry};
use pledge_and_crown::plugins::npc::{NearbyEntry, NearbyNpc, Npc};
use pledge_and_crown::plugins::progress::{MissionProgress, ProgressPlugin};
use pledge_and_crown::plugins::state::{GameState, PauseState, StatePlugin};

/// Frame step; exact in binary.
const STEP: f32 = 0.125;

/// The headless trial app with the Borrow Checker (frames) and the
/// Guildmaster (a trial owner without frames) spawned, the player beside
/// the Checker, every mission before `cleared_through` (inclusive) cleared
/// before the NPCs spawn.
fn app(cleared_through: Option<&str>) -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            STEP,
        )))
        .add_plugins(StatesPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(ProgressPlugin::in_memory())
        .add_plugins(MissionPlugin)
        .add_plugins(DialoguePlugin)
        .add_plugins(BossFramesPlugin)
        .init_resource::<EditorState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(NearbyNpc {
            current: Some(NearbyEntry {
                name: "The Borrow Checker",
                mission_id: "borrow_preview",
            }),
        });
    app.insert_state(GameState::InGame);
    if let Some(last) = cleared_through {
        clear_through(&mut app, last);
    }
    let checker = app
        .world_mut()
        .spawn((
            Sprite::default(),
            Npc {
                name: "The Borrow Checker",
                mission_id: "borrow_preview",
            },
        ))
        .id();
    let guildmaster = app
        .world_mut()
        .spawn((
            Sprite::default(),
            Npc {
                name: "The Guildmaster",
                mission_id: "impl_method",
            },
        ))
        .id();
    app.update();
    app.update();
    (app, checker, guildmaster)
}

fn clear_through(app: &mut App, last: &str) {
    let ids: Vec<&'static str> = MissionRegistry::default()
        .missions
        .iter()
        .map(|m| m.id)
        .collect();
    let mut p = app.world_mut().resource_mut::<MissionProgress>();
    for id in ids {
        p.mark_cleared(id);
        if id == last {
            break;
        }
    }
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(key);
    keys.clear();
}

fn finish_dialogue(app: &mut App) {
    for _ in 0..50 {
        if !app.world().resource::<Dialogue>().is_open() {
            return;
        }
        tap(app, KeyCode::KeyF);
    }
    panic!("dialogue never finished");
}

fn shown(app: &App, e: Entity) -> BossFrame {
    app.world()
        .get::<BossFrames>(e)
        .expect("boss has BossFrames")
        .shown
}

/// Updates until the boss's frame is no longer `frame`; the update count.
fn updates_while(app: &mut App, e: Entity, frame: BossFrame, max: usize) -> usize {
    for n in 1..=max {
        app.update();
        if shown(app, e) != frame {
            return n;
        }
    }
    max + 1
}

#[test]
fn the_trial_flow_shows_intro_then_defeat_then_idle() {
    let (mut app, checker, guildmaster) = app(Some("while_loop"));
    assert_eq!(shown(&app, checker), BossFrame::Idle);
    assert!(
        app.world().get::<BossFrames>(guildmaster).is_none(),
        "a boss without frames gets no BossFrames"
    );

    // F: the offer of stage 1 opens; he lights up and holds while it is up.
    tap(&mut app, KeyCode::KeyF);
    assert!(app.world().resource::<Dialogue>().is_open());
    assert_eq!(shown(&app, checker), BossFrame::Intro);
    for _ in 0..40 {
        app.update();
    }
    assert_eq!(
        shown(&app, checker),
        BossFrame::Intro,
        "held by the dialogue"
    );

    // The dialogue hands off to the editor; the intro tail runs out.
    finish_dialogue(&mut app);
    assert!(app.world().resource::<EditorState>().open);
    let n = updates_while(&mut app, checker, BossFrame::Intro, 100);
    assert!(
        (n as f32) * STEP >= INTRO_SECS - STEP && (n as f32) * STEP <= INTRO_SECS + 2.0 * STEP,
        "intro tail {n} updates"
    );

    // Stages 1 and 2 cleared: no defeat yet (the trial is not complete).
    for stage in ["bridge_toll", "bridge_quill"] {
        app.world_mut()
            .resource_mut::<MissionProgress>()
            .mark_cleared(stage);
        app.update();
        assert_eq!(shown(&app, checker), BossFrame::Idle, "{stage}");
    }

    // The final stage passes in the editor: he kneels, held while the
    // editor and then the trial-complete panel cover him.
    app.world_mut()
        .resource_mut::<MissionProgress>()
        .mark_cleared("bridge_lantern");
    app.update();
    assert_eq!(shown(&app, checker), BossFrame::Defeat);
    for _ in 0..80 {
        app.update();
    }
    assert_eq!(
        shown(&app, checker),
        BossFrame::Defeat,
        "held by the editor"
    );
    app.world_mut().resource_mut::<EditorState>().open = false;
    for _ in 0..80 {
        app.update();
    }
    assert!(
        app.world()
            .resource::<CompletionView>()
            .mission_id
            .is_some(),
        "a trial panel is up after the editor closes"
    );
    assert_eq!(shown(&app, checker), BossFrame::Defeat, "held by the panel");

    // Panel closed: DEFEAT_SECS, then idle for good.
    app.world_mut().resource_mut::<CompletionView>().mission_id = None;
    let n = updates_while(&mut app, checker, BossFrame::Defeat, 200);
    assert!(
        (n as f32) * STEP >= DEFEAT_SECS - STEP && (n as f32) * STEP <= DEFEAT_SECS + 2.0 * STEP,
        "defeat beat {n} updates"
    );
    assert_eq!(shown(&app, checker), BossFrame::Idle);

    // Talking to him now is the cleared recap: no intro.
    tap(&mut app, KeyCode::KeyF);
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(shown(&app, checker), BossFrame::Idle);
    assert!(app.world().get::<BossFrames>(guildmaster).is_none());
}

#[test]
fn the_prologue_and_a_locked_trial_do_not_light_him_up() {
    // Prologue offer (borrow_preview) and, later, the cleared-villager
    // window before the Bridge opens: neither is the trial.
    for cleared in [Some("double_function"), Some("borrow_preview")] {
        let (mut app, checker, _) = app(cleared);
        tap(&mut app, KeyCode::KeyF);
        assert!(app.world().resource::<Dialogue>().is_open());
        app.update();
        assert_eq!(shown(&app, checker), BossFrame::Idle, "{cleared:?}");
    }
}

#[test]
fn a_boss_cleared_in_the_loaded_save_stands_idle() {
    let (mut app, checker, _) = app(Some("bridge_lantern"));
    for _ in 0..50 {
        app.update();
    }
    assert_eq!(shown(&app, checker), BossFrame::Idle);
}

#[test]
fn pausing_freezes_the_defeat_beat() {
    let (mut app, checker, _) = app(Some("bridge_quill"));
    app.world_mut()
        .resource_mut::<MissionProgress>()
        .mark_cleared("bridge_lantern");
    app.update();
    assert_eq!(shown(&app, checker), BossFrame::Defeat);
    app.world_mut()
        .resource_mut::<NextState<PauseState>>()
        .set(PauseState::Paused);
    app.update();
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    for _ in 0..200 {
        app.update();
    }
    assert_eq!(
        shown(&app, checker),
        BossFrame::Defeat,
        "frozen while paused"
    );
    app.world_mut()
        .resource_mut::<NextState<PauseState>>()
        .set(PauseState::Running);
    let n = updates_while(&mut app, checker, BossFrame::Defeat, 200);
    assert!(n as f32 * STEP <= DEFEAT_SECS + 2.0 * STEP, "{n}");
}

#[test]
fn the_table_paths_exist_on_disk() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    assert_eq!(BOSS_FRAMES.len(), 3);
    for f in BOSS_FRAMES {
        for p in [f.idle, f.intro, f.defeat] {
            assert!(root.join(p).is_file(), "{}: {p} missing", f.npc);
        }
    }
}
