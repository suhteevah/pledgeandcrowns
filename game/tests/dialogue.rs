// SPDX-License-Identifier: MIT
//! Dialogue: conversation selection, the NotYetAvailable pointer, inline
//! code spans, paging/typewriter state machine, line coverage for the whole
//! roster, and the F -> dialogue -> mission handoff.
//!
//! No test here reads the wall clock: the pure typewriter takes explicit
//! `dt`s, and the headless app runs `TimeUpdateStrategy::ManualDuration`
//! (zero by default), so `Time<Real>` only moves when a test says so.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use pledge_and_crown::plugins::dialogue::lines::{
    AUTHORED, GENERIC, NOT_YET_ELSEWHERE, NOT_YET_NOTHING,
};
use pledge_and_crown::plugins::dialogue::{
    CHARS_PER_SEC, Dialogue, DialogueHandoff, DialoguePlugin, DialogueRun, Finish, MAX_LINE_CHARS,
    MissionStatus, NextHint, Press, Span, line_templates, mission_status, next_hint, parse_spans,
    portrait_scale, render_line, reveal_spans, select_conversation, visible_len,
};
use pledge_and_crown::plugins::editor::EditorState;
use pledge_and_crown::plugins::mission::{MissionLockedAttempt, MissionPlugin, MissionRegistry};
use pledge_and_crown::plugins::npc::{NPC_ROSTER, NearbyEntry, NearbyNpc};
use pledge_and_crown::plugins::progress::{MissionProgress, ProgressPlugin};
use pledge_and_crown::plugins::state::{GameState, StatePlugin};

const FERRIS: NextHint = NextHint::DirectPrereq("Ferris");

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
        let c = select_conversation(id, status, FERRIS);
        assert_eq!(c.finish, finish, "{status:?}");
        assert!(!c.lines.is_empty(), "{status:?} produced no lines");
    }
}

// --- NotYetAvailable pointer ----------------------------------------------

fn progress_with(ids: &[&str]) -> MissionProgress {
    let mut p = MissionProgress::default();
    for id in ids {
        p.mark_cleared(id);
    }
    p
}

/// Ids of the first `n` missions in registry order.
fn first_n(reg: &MissionRegistry, n: usize) -> Vec<&'static str> {
    reg.missions.iter().take(n).map(|m| m.id).collect()
}

fn not_yet_line(reg: &MissionRegistry, p: &MissionProgress, speaker: &str) -> Vec<String> {
    let m = reg.find(speaker).expect("speaker in registry");
    assert_eq!(
        mission_status(p.is_cleared(m.id), m.is_unlocked(p), false),
        MissionStatus::NotYetAvailable,
        "{speaker} should be locked here"
    );
    select_conversation(
        speaker,
        MissionStatus::NotYetAvailable,
        next_hint(reg, p, speaker),
    )
    .lines
}

#[test]
fn fresh_save_late_npc_sends_you_to_ferris_without_claiming_to_be_next() {
    let reg = MissionRegistry::default();
    let p = MissionProgress::default();
    // Mid-roster speaker (inside the demo too): Ferris is next, but not
    // this NPC's prerequisite, so no "then come back to me".
    let speaker = reg.missions[15].id;
    assert_eq!(next_hint(&reg, &p, speaker), NextHint::Elsewhere("Ferris"));
    assert_eq!(
        not_yet_line(&reg, &p, speaker),
        vec!["I'm not ready for you yet. See Ferris first.".to_string()]
    );
    // The second NPC's direct prerequisite IS Ferris: their own voice.
    let second = reg.missions[1].id;
    assert_eq!(next_hint(&reg, &p, second), FERRIS);
    let lines = not_yet_line(&reg, &p, second);
    assert!(lines.iter().any(|l| l.contains("Ferris")), "{lines:?}");
    assert!(lines.iter().all(|l| !l.contains("not ready")), "{lines:?}");
}

#[test]
fn mid_game_save_points_at_the_next_playable_npc() {
    let reg = MissionRegistry::default();
    let p = progress_with(&first_n(&reg, 6));
    let next = &reg.missions[6];
    let speaker = reg.missions[15].id;
    assert_eq!(
        next_hint(&reg, &p, speaker),
        NextHint::Elsewhere(next.npc_name)
    );
    let lines = not_yet_line(&reg, &p, speaker);
    assert_eq!(
        lines,
        vec![format!(
            "I'm not ready for you yet. See {} first.",
            next.npc_name
        )]
    );
    assert!(lines.iter().all(|l| !l.contains("Ferris")));
}

#[test]
fn save_cleared_up_to_the_prereq_names_it_as_the_prereq() {
    let reg = MissionRegistry::default();
    // Generic-voiced speaker (index 10) and an authored one (index 3).
    for idx in [10usize, 3] {
        let speaker = &reg.missions[idx];
        let pre = &reg.missions[idx - 1];
        assert_eq!(speaker.prereq, Some(pre.id), "linear registry");
        let p = progress_with(&first_n(&reg, idx - 1));
        assert_eq!(
            next_hint(&reg, &p, speaker.id),
            NextHint::DirectPrereq(pre.npc_name)
        );
        let lines = not_yet_line(&reg, &p, speaker.id);
        assert_eq!(
            lines,
            line_templates(speaker.id, MissionStatus::NotYetAvailable)
                .iter()
                .map(|t| render_line(t, Some(pre.npc_name)))
                .collect::<Vec<_>>()
        );
        assert!(lines.iter().any(|l| l.contains(pre.npc_name)), "{lines:?}");
    }
}

#[test]
fn save_cleared_up_to_the_speaker_is_an_offer_not_a_refusal() {
    let reg = MissionRegistry::default();
    let speaker = &reg.missions[10];
    let p = progress_with(&first_n(&reg, 10));
    assert_eq!(
        mission_status(
            p.is_cleared(speaker.id),
            speaker.is_unlocked(&p),
            p.is_opened(speaker.id)
        ),
        MissionStatus::Offer
    );
}

#[test]
fn nothing_playable_gives_a_neutral_line() {
    let mut reg = MissionRegistry::default();
    let last = reg.missions.len() - 1;
    reg.missions[last].available = false;
    let all: Vec<&str> = reg.missions[..last].iter().map(|m| m.id).collect();
    let p = progress_with(&all);
    let speaker = reg.missions[last].id;
    assert_eq!(next_hint(&reg, &p, speaker), NextHint::Nothing);
    let c = select_conversation(speaker, MissionStatus::NotYetAvailable, NextHint::Nothing);
    assert_eq!(c.lines, vec![NOT_YET_NOTHING.to_string()]);
    assert_eq!(c.finish, Finish::Close);
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
            let c = select_conversation(spec.mission_id, status, FERRIS);
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
    for t in [NOT_YET_ELSEWHERE, NOT_YET_NOTHING] {
        let l = render_line(t, Some(longest));
        assert!(l.chars().count() <= MAX_LINE_CHARS, "{l}");
    }
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
                assert_eq!(
                    l.matches('`').count() % 2,
                    0,
                    "{id} {status:?}: unpaired backtick: {l}"
                );
                assert!(
                    l.chars().count() <= MAX_LINE_CHARS,
                    "{id} {status:?}: {} chars > {MAX_LINE_CHARS}: {l}",
                    l.chars().count()
                );
            }
        }
    }
}

// --- inline code spans ----------------------------------------------------

fn plain(t: &str) -> Span {
    Span::new(t, false)
}

fn code(t: &str) -> Span {
    Span::new(t, true)
}

#[test]
fn backticks_become_code_spans_and_vanish() {
    assert_eq!(
        parse_spans("Mark it `mut`, then strike."),
        vec![plain("Mark it "), code("mut"), plain(", then strike.")]
    );
    assert_eq!(
        parse_spans("`let` then `x`"),
        vec![code("let"), plain(" then "), code("x")]
    );
    assert_eq!(parse_spans("no code"), vec![plain("no code")]);
    assert_eq!(parse_spans(""), Vec::<Span>::new());
    // Empty pair contributes nothing; neighbours merge.
    assert_eq!(parse_spans("a``b"), vec![plain("ab")]);
    // A stray backtick is dropped, never shown, and does not open code.
    assert_eq!(parse_spans("odd ` tick"), vec![plain("odd  tick")]);
    for line in ["`a` b `c`", "odd ` tick", "x `&mut T` y"] {
        let spans = parse_spans(line);
        assert!(spans.iter().all(|s| !s.text.contains('`')), "{line}");
    }
}

#[test]
fn reveal_counts_visible_chars_and_never_half_opens_code() {
    let spans = parse_spans("with `let`, go");
    assert_eq!(visible_len(&spans), "with let, go".len());
    assert_eq!(reveal_spans(&spans, 0), Vec::<Span>::new());
    assert_eq!(reveal_spans(&spans, 5), vec![plain("with ")]);
    // Two chars into the code: a short, closed code span.
    assert_eq!(reveal_spans(&spans, 7), vec![plain("with "), code("le")]);
    assert_eq!(reveal_spans(&spans, 8), vec![plain("with "), code("let")]);
    assert_eq!(reveal_spans(&spans, 999), spans);
}

#[test]
fn typewriter_skips_backticks() {
    let mut r = run(&["a `b` c"]);
    r.tick(3.0 / 40.0, 40.0); // 3 visible chars
    assert_eq!(r.visible(), "a b");
    assert_eq!(r.visible_spans(), vec![plain("a "), code("b")]);
    r.tick(1.0, 40.0);
    assert!(r.is_line_complete());
    assert_eq!(r.visible(), "a b c");
    assert_eq!(r.line(), "a `b` c", "the raw line keeps its markup");
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

/// Headless app on a manual clock: `Time<Real>` advances by exactly the
/// `TimeUpdateStrategy` duration per update (zero unless a test sets it),
/// so the typewriter never races the test.
fn app_with_nearby(name: &'static str, mission_id: &'static str) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
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
    assert!(
        app.world()
            .resource::<MissionProgress>()
            .is_opened("intro_let_binding"),
        "opened is stored in the (saved) progress, not session memory"
    );
}

#[test]
fn opened_progress_survives_into_a_new_app_as_in_progress() {
    // Simulates a restart: the save carried `opened`, so the first talk
    // after loading is the nudge, not the greeting again.
    let mut app = app_with_nearby("Ferris", "intro_let_binding");
    app.world_mut()
        .resource_mut::<MissionProgress>()
        .mark_opened("intro_let_binding");
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
fn typewriter_follows_the_bevy_clock_not_the_wall_clock() {
    let mut app = app_with_nearby("Ferris", "intro_let_binding");
    tap(&mut app, KeyCode::KeyF);
    let visible = |app: &App| {
        app.world()
            .resource::<Dialogue>()
            .active
            .as_ref()
            .map(|a| a.run.visible())
            .unwrap_or_default()
    };
    // Frozen clock: nothing types no matter how many frames run.
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(visible(&app), "");
    // 100 ms of Bevy time = floor(4.5) chars at 45 chars/s.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.update();
    let expect = (0.1 * CHARS_PER_SEC).floor() as usize;
    assert_eq!(visible(&app).chars().count(), expect);
    assert_eq!(visible(&app), "Welc");
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
