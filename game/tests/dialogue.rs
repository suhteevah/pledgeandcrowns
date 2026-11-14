// SPDX-License-Identifier: MIT
//! Dialogue: conversation selection, paging/typewriter state machine, line
//! coverage for the whole roster, and the F -> dialogue -> mission handoff.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use pledge_and_crown::plugins::dialogue::lines::{AUTHORED, GENERIC};
use pledge_and_crown::plugins::dialogue::{
    Dialogue, DialogueHandoff, DialoguePlugin, DialogueRun, Finish, MAX_LINE_CHARS, MissionStatus,
    Press, line_templates, mission_status, portrait_scale, render_line, select_conversation,
};
use pledge_and_crown::plugins::editor::EditorState;
use pledge_and_crown::plugins::mission::{MissionLockedAttempt, MissionPlugin, MissionRegistry};
use pledge_and_crown::plugins::npc::{NPC_ROSTER, NearbyEntry, NearbyNpc};
use pledge_and_crown::plugins::progress::ProgressPlugin;
use pledge_and_crown::plugins::state::{GameState, StatePlugin};

// --- selection -----------------------------------------------------------

#[test]
fn status_classification_covers_every_case() {
    // (cleared, unlocked, opened)
    assert_eq!(mission_status(true, true, true), MissionStatus::Cleared);
    assert_eq!(mission_status(true, false, false), MissionStatus::Cleared);
    assert_eq!(
        mission_status(false, false, false),
        MissionStatus::NotYetAvailable
    );
    assert_eq!(
        mission_status(false, false, true),
        MissionStatus::NotYetAvailable
    );
    assert_eq!(mission_status(false, true, false), MissionStatus::Offer);
    assert_eq!(mission_status(false, true, true), MissionStatus::InProgress);
}

#[test]
fn offer_and_in_progress_open_the_mission_others_just_close() {
    let id = "borrow_preview";
    for (status, finish) in [
        (MissionStatus::NotYetAvailable, Finish::Close),
        (MissionStatus::Offer, Finish::OpenMission),
        (MissionStatus::InProgress, Finish::OpenMission),
        (MissionStatus::Cleared, Finish::OpenMission),
    ] {
        let c = select_conversation(id, status, Some("Ferris"));
        assert_eq!(c.finish, finish, "{status:?}");
        assert!(!c.lines.is_empty(), "{status:?} produced no lines");
    }
}

#[test]
fn not_yet_available_names_the_next_npc() {
    let reg = MissionRegistry::default();
    let progress = Default::default();
    let next = reg.next_locked(&progress).map(|m| m.npc_name);
    assert_eq!(next, Some("Ferris"));
    // Authored and generic NPCs both point at whoever is next.
    for npc in ["double_function", "weak_ref"] {
        let c = select_conversation(npc, MissionStatus::NotYetAvailable, next);
        assert!(
            c.lines.iter().any(|l| l.contains("Ferris")),
            "{npc}: hint must name the next NPC, got {:?}",
            c.lines
        );
        assert!(c.lines.iter().all(|l| !l.contains("{next}")));
    }
    // No next mission: a neutral fallback, never a raw placeholder.
    let c = select_conversation("weak_ref", MissionStatus::NotYetAvailable, None);
    assert!(c.lines.iter().all(|l| !l.contains('{')));
}

#[test]
fn authored_npc_uses_authored_lines_others_use_generic() {
    assert_eq!(
        line_templates("borrow_preview", MissionStatus::Offer),
        AUTHORED
            .iter()
            .find(|r| r.npc_id == "borrow_preview")
            .map(|r| r.lines.offer)
            .unwrap()
    );
    assert_eq!(
        line_templates("weak_ref", MissionStatus::Cleared),
        GENERIC.cleared
    );
    assert_eq!(
        line_templates("not_a_real_npc", MissionStatus::Offer),
        GENERIC.offer
    );
}

// --- coverage ------------------------------------------------------------

#[test]
fn every_roster_npc_has_lines_for_every_status() {
    for spec in NPC_ROSTER {
        for status in MissionStatus::ALL {
            let c = select_conversation(spec.mission_id, status, Some("Ferris"));
            assert!(
                !c.lines.is_empty() && c.lines.iter().all(|l| !l.trim().is_empty()),
                "{} has no usable {status:?} line",
                spec.mission_id
            );
        }
    }
}

#[test]
fn authored_rows_are_real_unique_and_cover_hearthstone_square() {
    let ids: Vec<&str> = AUTHORED.iter().map(|r| r.npc_id).collect();
    let mut dedup = ids.clone();
    dedup.sort_unstable();
    dedup.dedup();
    assert_eq!(dedup.len(), ids.len(), "duplicate authored npc_id");
    for id in &ids {
        assert!(
            NPC_ROSTER.iter().any(|s| s.mission_id == *id),
            "authored row {id} is not a roster NPC"
        );
    }
    let first_nine: Vec<&str> = NPC_ROSTER.iter().take(9).map(|s| s.mission_id).collect();
    assert_eq!(
        ids, first_nine,
        "authored rows must be the first 9 in roster order"
    );
}

/// Every authored and generic line is non-empty and fits the box
/// (`MAX_LINE_CHARS`) even with the longest roster name in `{next}`.
#[test]
fn all_lines_fit_the_box() {
    let longest = NPC_ROSTER
        .iter()
        .map(|s| s.name)
        .max_by_key(|n| n.chars().count())
        .unwrap();
    let tables = AUTHORED
        .iter()
        .map(|r| (r.npc_id, r.lines))
        .chain(std::iter::once(("GENERIC", GENERIC)));
    for (id, table) in tables {
        for status in MissionStatus::ALL {
            let lines = table.for_status(status);
            if id != "GENERIC" {
                assert!(
                    !lines.is_empty() && lines.len() <= 3,
                    "{id} {status:?}: authored rows carry 1-3 lines"
                );
            }
            for t in lines {
                let l = render_line(t, Some(longest));
                assert!(!l.trim().is_empty(), "{id} {status:?}: empty line");
                assert!(
                    l.chars().count() <= MAX_LINE_CHARS,
                    "{id} {status:?}: {} chars > {MAX_LINE_CHARS}: {l}",
                    l.chars().count()
                );
            }
        }
    }
}

// --- paging / typewriter -------------------------------------------------

fn run(lines: &[&str]) -> DialogueRun {
    DialogueRun::new(lines.iter().map(|s| s.to_string()).collect())
}

#[test]
fn typewriter_reveals_over_time() {
    let mut r = run(&["hello"]);
    assert_eq!(r.visible(), "");
    r.tick(0.05, 40.0); // 2 chars
    assert_eq!(r.visible(), "he");
    r.tick(0.0125, 40.0); // 0.5 char carried
    assert_eq!(r.visible(), "he");
    r.tick(0.0125, 40.0); // carry completes a char
    assert_eq!(r.visible(), "hel");
    r.tick(10.0, 40.0);
    assert_eq!(r.visible(), "hello");
    assert!(r.is_line_complete());
}

#[test]
fn typewriter_is_char_boundary_safe() {
    let mut r = run(&["déjà vu"]);
    r.tick(0.1, 30.0); // 3 chars
    assert_eq!(r.visible(), "déj");
}

#[test]
fn first_press_completes_second_advances_last_finishes() {
    let mut r = run(&["one", "two"]);
    assert_eq!(r.press(), Press::Revealed);
    assert_eq!(r.visible(), "one");
    assert_eq!(r.press(), Press::NextPage);
    assert_eq!(r.page(), 1);
    assert_eq!(r.visible(), "", "new page restarts the typewriter");
    r.tick(10.0, 45.0);
    assert_eq!(r.press(), Press::Finished);
    // Finished is sticky: further presses do not wander off the end.
    assert_eq!(r.press(), Press::Finished);
    assert_eq!(r.line(), "two");
}

#[test]
fn empty_conversation_finishes_immediately() {
    let mut r = run(&[]);
    assert!(r.is_line_complete());
    assert_eq!(r.press(), Press::Finished);
}

#[test]
fn portrait_scale_is_integer_and_fits() {
    assert_eq!(portrait_scale(32.0, 64.0), 2.0);
    assert_eq!(portrait_scale(64.0, 64.0), 1.0);
    assert_eq!(portrait_scale(48.0, 64.0), 1.0);
    assert_eq!(portrait_scale(128.0, 64.0), 1.0, "never below 1x");
    assert_eq!(portrait_scale(0.0, 64.0), 1.0);
}

// --- headless app: F -> dialogue -> mission ------------------------------

fn app_with_nearby(name: &'static str, mission_id: &'static str) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(ProgressPlugin)
        .add_plugins(MissionPlugin)
        .add_plugins(DialoguePlugin)
        .init_resource::<EditorState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(NearbyNpc {
            current: Some(NearbyEntry { name, mission_id }),
        });
    app.insert_state(GameState::InGame);
    app.update(); // settle state transitions
    app
}

/// Tap a key for one frame. Without InputPlugin nobody clears
/// `just_pressed`, so release + clear by hand afterwards.
fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(key);
    keys.clear();
}

fn dialogue_open(app: &App) -> bool {
    app.world().resource::<Dialogue>().is_open()
}

fn editor_open(app: &App) -> bool {
    app.world().resource::<EditorState>().open
}

#[test]
fn f_opens_dialogue_then_last_line_opens_the_mission() {
    let mut app = app_with_nearby("Ferris", "intro_let_binding");
    tap(&mut app, KeyCode::KeyF);
    assert!(dialogue_open(&app), "F must open the dialogue");
    assert!(!editor_open(&app), "the editor waits for the dialogue");

    let pages = app
        .world()
        .resource::<Dialogue>()
        .active
        .as_ref()
        .map(|a| a.run.page_count())
        .unwrap();
    // Each page: one press to complete, one to advance/finish.
    for _ in 0..(pages * 2 - 1) {
        tap(&mut app, KeyCode::Space);
        assert!(dialogue_open(&app));
        assert!(!editor_open(&app));
    }
    tap(&mut app, KeyCode::Enter);
    assert!(!dialogue_open(&app), "last press closes the dialogue");
    assert!(editor_open(&app), "...and opens the mission like F used to");
    assert_eq!(
        app.world().resource::<EditorState>().encounter_id,
        "intro_let_binding"
    );
    assert!(!app.world_mut().resource_mut::<DialogueHandoff>().take());
}

#[test]
fn reopening_after_the_mission_uses_in_progress() {
    let mut app = app_with_nearby("Ferris", "intro_let_binding");
    tap(&mut app, KeyCode::KeyF);
    while dialogue_open(&app) {
        tap(&mut app, KeyCode::KeyF);
    }
    app.world_mut().resource_mut::<EditorState>().open = false;
    tap(&mut app, KeyCode::KeyF);
    let status = app
        .world()
        .resource::<Dialogue>()
        .active
        .as_ref()
        .map(|a| a.status);
    assert_eq!(status, Some(MissionStatus::InProgress));
}

#[test]
fn esc_closes_dialogue_without_opening_the_mission() {
    let mut app = app_with_nearby("Ferris", "intro_let_binding");
    tap(&mut app, KeyCode::KeyF);
    assert!(dialogue_open(&app));
    tap(&mut app, KeyCode::Escape);
    assert!(!dialogue_open(&app));
    assert!(!editor_open(&app));
}

#[test]
fn locked_npc_talks_and_still_fires_the_locked_sting() {
    let mut app = app_with_nearby("Trait Mage", "double_function");
    tap(&mut app, KeyCode::KeyF);
    let status = app
        .world()
        .resource::<Dialogue>()
        .active
        .as_ref()
        .map(|a| a.status);
    assert_eq!(status, Some(MissionStatus::NotYetAvailable));
    let n = app
        .world_mut()
        .resource_mut::<Messages<MissionLockedAttempt>>()
        .drain()
        .count();
    assert_eq!(n, 1, "exactly one locked-attempt message");
    while dialogue_open(&app) {
        tap(&mut app, KeyCode::KeyF);
    }
    assert!(!editor_open(&app), "a locked NPC never opens the editor");
}

#[test]
fn movement_keys_read_released_while_dialogue_is_open() {
    let mut app = app_with_nearby("Ferris", "intro_let_binding");
    tap(&mut app, KeyCode::KeyF);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();
    let keys = app.world().resource::<ButtonInput<KeyCode>>();
    assert!(!keys.pressed(KeyCode::KeyW), "W must not reach movement");
    assert!(!keys.just_pressed(KeyCode::KeyW));
}
