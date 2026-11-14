// SPDX-License-Identifier: MIT
//! The scripted capture list. Order matters: title scenes run first (the
//! game boots on the title), then the world vantages, then the in-game UI.
//!
//! World vantages keep their historical `shot-<label>.png` names
//! (`pitch/capture-world.ps1` and the pitch deck depend on them); every
//! other capture is `scene-<name>.png`.
//!
//! Every state change goes through the game's own resources: `NextState`,
//! `MenuUi`, `MissionProgress`, `EditorState`, `CompletionView`,
//! `DialogueMemory`, the two `EditorSession` hooks, and in-app key taps /
//! holds on Bevy's `ButtonInput` (F to talk, WASD to walk).

use bevy::prelude::*;

use super::collision::{PUSH_SECS, feet_inside_any, gap_to_wall, probe_wall, probes};
use super::{Action, Scene, ShotLog, WorldFn};
use crate::plugins::compile_client::CompileMode;
use crate::plugins::compile_client::outcome::{HttpReply, classify, verdict_with_server};
use crate::plugins::dialogue::{Dialogue, DialogueMemory};
use crate::plugins::editor::{EditorSession, EditorState};
use crate::plugins::menu::{MenuScreen, MenuUi};
use crate::plugins::mission::{ActiveMission, CompletionView, MissionRegistry};
use crate::plugins::npc::NPC_ROSTER;
use crate::plugins::player::Player;
use crate::plugins::progress::{MissionProgress, SavedPlayerPosition, Settings};
use crate::plugins::prop::{MEADOW_OAKS, PropKind, Solid};
use crate::plugins::state::{GameState, PauseState};
use crate::plugins::world::district_slot;

/// First mission (Ferris). The editor/recap/dialogue scenes use it.
const FIRST_MISSION: &str = "intro_let_binding";
/// Missions marked cleared for the "Continue available" title scene.
const CONTINUE_CLEARED: &[&str] = &["intro_let_binding", "double_function", "borrow_preview"];
/// Texture-bind settle after the first Title -> InGame transition.
const ENTER_GAME_SETTLE: f32 = 2.5;

/// Historical world vantages (label, player position). Ferris (the
/// next-quest NPC, golden '!') sits lower-left of `ferris-marker`.
const VANTAGES: &[(&str, Vec2)] = &[
    ("ferris-marker", Vec2::new(-180.0, -200.0)),
    ("south", Vec2::new(0.0, -470.0)),
    ("street", Vec2::new(0.0, 470.0)),
    ("meadow", Vec2::new(-180.0, 560.0)),
    ("junction", Vec2::new(-330.0, 0.0)),
    ("treeline", Vec2::new(-720.0, 980.0)),
];

/// Player edit with one rustc error on (player) line 3.
const BUGGY_SOURCE: &str =
    "fn main() {\n    let answer = 42;\n    answer = 43;\n    println!(\"{answer}\");\n}\n";

/// A canned rustc failure in the exact shape `compile_client` writes for a
/// graded failure: server text verbatim, so rustc reports the line one
/// below the player's (the server prepends one `#![allow]` line).
const CANNED_FAIL: &str = "[fail:compile] [real]\n--stderr--\n\
error[E0384]: cannot assign twice to immutable variable `answer`\n \
--> src/main.rs:4:5\n  |\n\
3 |     let answer = 42;\n  |         ------ first assignment to `answer`\n\
4 |     answer = 43;\n  |     ^^^^^^^^^^^ cannot assign twice to immutable variable\n  |\n\
help: consider making this binding mutable\n  |\n\
3 |     let mut answer = 42;\n  |         +++\n\n\
error: aborting due to 1 previous error\n\n\
For more information about this error, try `rustc --explain E0384`.";

/// A longer edit with several rustc errors and a warning (player lines;
/// the server reports each one line lower).
const LONG_SOURCE: &str = "fn take(v: String) {}\n\nfn main() {\n    let answer = 42;\n    answer = 43;\n    let n: i32 = \"five\";\n    let s = String::from(\"hi\");\n    take(s);\n    println!(\"{} {}\", s, y);\n}\n";

/// `/compile-real` reply for [`LONG_SOURCE`]: four errors and a warning,
/// as the server sends it (cargo progress lines included).
const LONG_FAIL_JSON: &str = r#"{"ok":false,"stdout":"[real]","stderr":"   Compiling player_submission v0.0.0 (/build)\nerror[E0425]: cannot find value `y` in this scope\n  --> src/main.rs:10:28\n   |\n10 |     println!(\"{} {}\", s, y);\n   |                            ^ help: a local variable with a similar name exists: `s`\n\nwarning: unused variable: `n`\n --> src/main.rs:7:9\n  |\n7 |     let n: i32 = \"five\";\n  |         ^ help: if this is intentional, prefix it with an underscore: `_n`\n  |\n  = note: `#[warn(unused_variables)]` on by default\n\nerror[E0308]: mismatched types\n --> src/main.rs:7:18\n  |\n7 |     let n: i32 = \"five\";\n  |            ---   ^^^^^^ expected `i32`, found `&str`\n  |            |\n  |            expected due to this\n\nerror[E0384]: cannot assign twice to immutable variable `answer`\n --> src/main.rs:6:5\n  |\n5 |     let answer = 42;\n  |         ------ first assignment to `answer`\n6 |     answer = 43;\n  |     ^^^^^^^^^^^ cannot assign twice to immutable variable\n  |\nhelp: consider making this binding mutable\n  |\n5 |     let mut answer = 42;\n  |         +++\n\nerror[E0382]: borrow of moved value: `s`\n  --> src/main.rs:10:23\n   |\n8  |     let s = String::from(\"hi\");\n   |         - move occurs because `s` has type `String`, which does not implement the `Copy` trait\n9  |     take(s);\n   |          - value moved here\n10 |     println!(\"{} {}\", s, y);\n   |                       ^ value borrowed here after move\n   |\nnote: consider changing this parameter type in function `take` to borrow instead if owning the value isn't necessary\n  --> src/main.rs:2:12\n   |\n2  | fn take(v: String) {}\n   |    ----    ^^^^^^ this parameter takes ownership of the value\n\nSome errors have detailed explanations: E0308, E0382, E0384, E0425.\nFor more information about an error, try `rustc --explain E0308`.\nwarning: `player_submission` (bin \"player_submission\") generated 1 warning\nerror: could not compile `player_submission` (bin \"player_submission\") due to 4 previous errors; 1 warning emitted","grade":"compile_error","behaviour_checked":false,"note":"did not build"}"#;

const WRONG_SOURCE: &str = "fn main() {\n    let answer = 41;\n    println!(\"{answer}\");\n}\n";
/// `/compile-real` wrong-output reply, verbatim server shape.
const WRONG_OUTPUT_JSON: &str = r#"{"ok":false,"stdout":"[real] 41","stderr":"Your program ran, but did not do what this mission asks.\nExpected a line reading `42`\nGot:\n41","grade":"wrong_output","behaviour_checked":true,"expected":"a line reading `42`","actual":"41","note":"output checked against the mission"}"#;
/// `/compile-real` compile-only mission: accepted, behaviour not checked.
const UNVERIFIED_JSON: &str = r#"{"ok":true,"stdout":"[real] compiled\n-- not checked: behaviour not checked on this server: wasm32-wasip1 has no threads, so this program is compiled and statically checked but never run","stderr":"","grade":"compiled_not_checked","behaviour_checked":false,"note":"behaviour not checked on this server: wasm32-wasip1 has no threads, so this program is compiled and statically checked but never run"}"#;
/// `/compile-real` verified pass.
const VERIFIED_JSON: &str = r#"{"ok":true,"stdout":"[real] 42\n-- checked: does what the mission asks","stderr":"","grade":"passed","behaviour_checked":true,"note":"output checked against the mission"}"#;
/// `/compile` (pattern route) pass: no grade fields at all.
const PATTERN_JSON: &str =
    r#"{"ok":true,"stdout":"Ferris clicks approvingly: `answer` is bound to 42.","stderr":""}"#;

const PASSING_SOURCE: &str = "fn main() {\n    let answer = 42;\n    println!(\"{answer}\");\n}\n";
const CANNED_PASS: &str = "[ok] 42";

fn act(f: impl Fn(&mut World) + Send + Sync + 'static) -> Action {
    Action::Do(Box::new(f))
}

fn boxed(f: impl Fn(&mut World) + Send + Sync + 'static) -> Option<WorldFn> {
    Some(Box::new(f))
}

fn scene(name: &str, actions: Vec<Action>, settle: f32, teardown: Option<WorldFn>) -> Scene {
    Scene {
        file: format!("scene-{name}.png"),
        actions,
        settle,
        teardown,
    }
}

// --- World helpers ---------------------------------------------------------

/// Snap player + camera to `pos`.
fn teleport(world: &mut World, pos: Vec2) {
    let mut pq = world.query_filtered::<&mut Transform, (With<Player>, Without<Camera2d>)>();
    let mut moved = 0usize;
    for mut t in pq.iter_mut(world) {
        t.translation.x = pos.x;
        t.translation.y = pos.y;
        moved += 1;
    }
    let mut cq = world.query_filtered::<&mut Transform, With<Camera2d>>();
    for mut t in cq.iter_mut(world) {
        t.translation.x = pos.x;
        t.translation.y = pos.y;
    }
    if moved == 0 {
        tracing::warn!(?pos, "teleport: no player entity");
    } else {
        tracing::debug!(?pos, "teleported player + camera");
    }
}

fn teleport_to(pos: Vec2) -> Action {
    act(move |w| teleport(w, pos))
}

fn player_pos(world: &mut World) -> Option<Vec2> {
    let mut q = world.query_filtered::<&Transform, With<Player>>();
    q.iter(world).next().map(|t| t.translation.truncate())
}

/// Fresh in-memory progress: nothing cleared, no saved position.
fn fresh_progress(world: &mut World) {
    world.resource_mut::<MissionProgress>().reset();
    world.resource_mut::<SavedPlayerPosition>().0 = None;
    world.resource_mut::<DialogueMemory>().opened.clear();
    tracing::debug!("scene: progress reset (in-memory)");
}

fn clear_missions(world: &mut World, ids: &[&str]) {
    let mut p = world.resource_mut::<MissionProgress>();
    for id in ids {
        p.mark_cleared(id);
    }
    tracing::debug!(?ids, "scene: missions marked cleared (in-memory)");
}

/// Initial in-memory state for the whole run.
pub(super) fn boot_state(world: &mut World) {
    fresh_progress(world);
    // Silence the run without hiding the sliders: master 0 mutes all
    // audio (a capture run shares the machine with someone working).
    let mut s = world.resource_mut::<Settings>();
    s.master_volume = 0;
    s.music_volume = 70;
    s.sfx_volume = 80;
    tracing::info!("scene: master volume 0 for the run (in-memory only)");
}

fn npc_slot(mission_id: &str) -> Option<Vec2> {
    NPC_ROSTER
        .iter()
        .position(|s| s.mission_id == mission_id)
        .map(district_slot)
}

/// Stand just south-west of an NPC, inside the talk radius.
fn beside_npc(mission_id: &'static str) -> Action {
    act(move |w| match npc_slot(mission_id) {
        Some(slot) => teleport(w, slot + Vec2::new(-14.0, -14.0)),
        None => tracing::warn!(mission_id, "no NPC slot"),
    })
}

fn close_dialogue(world: &mut World) {
    world.resource_mut::<Dialogue>().active = None;
}

/// Open the editor on `mission_id` with `source`, as the mission F-press does.
fn open_editor(world: &mut World, mission_id: &str, source: &str) {
    let mission = world
        .resource::<MissionRegistry>()
        .find(mission_id)
        .cloned();
    let Some(mission) = mission else {
        tracing::warn!(mission_id, "open_editor: unknown mission");
        return;
    };
    {
        let mut ed = world.resource_mut::<EditorState>();
        ed.source = source.to_string();
        ed.last_compile_result = None;
        ed.encounter_id = mission.id.to_string();
        ed.open = true;
    }
    world.resource_mut::<ActiveMission>().current = Some(mission);
}

/// One failed compile: mark in flight, then deliver the canned rustc text
/// through `EditorState::last_compile_result` (where `compile_client`
/// writes real verdicts). `track_compile_results` counts it next frame.
fn canned_failure(world: &mut World) {
    world
        .resource_mut::<EditorSession>()
        .note_compile_requested(FIRST_MISSION);
    let mut ed = world.resource_mut::<EditorState>();
    ed.source = BUGGY_SOURCE.to_string();
    ed.last_compile_result = Some(CANNED_FAIL.to_string());
}

/// Show a server reply in the editor exactly as a real compile would:
/// the canned JSON goes through the client's own classifier and verdict
/// mapper. Display only (no failure is counted).
fn show_server_reply(world: &mut World, mode: CompileMode, json: &str) {
    let outcome = classify(
        mode,
        Ok(HttpReply {
            status: 200,
            retry_after: None,
            body: json.to_string(),
        }),
    );
    let verdict = verdict_with_server(&outcome, FIRST_MISSION, || None);
    tracing::info!(grade = ?verdict.grade, "scene: canned server reply shown");
    world
        .resource_mut::<EditorState>()
        .set_result(verdict.formatted, verdict.detail);
}

/// Resize the primary window in-app, through Bevy's `Window` component
/// (no OS input, no focus change).
fn set_window_size(world: &mut World, w: f32, h: f32) {
    let mut q = world.query_filtered::<&mut Window, With<bevy::window::PrimaryWindow>>();
    let mut n = 0usize;
    for mut win in q.iter_mut(world) {
        win.resolution.set(w, h);
        n += 1;
    }
    tracing::info!(w, h, windows = n, "scene: primary window resized");
}

fn close_editor(world: &mut World) {
    let mut ed = world.resource_mut::<EditorState>();
    ed.open = false;
    ed.last_compile_result = None;
}

// --- Scene list --------------------------------------------------------------

pub(super) fn all_scenes() -> Vec<Scene> {
    let mut v = Vec::new();
    title_scenes(&mut v);
    world_scenes(&mut v);
    ysort_scenes(&mut v);
    collision_scenes(&mut v);
    dialogue_scenes(&mut v);
    editor_scenes(&mut v);
    editor_result_scenes(&mut v);
    overlay_scenes(&mut v);
    v
}

fn title_scenes(v: &mut Vec<Scene>) {
    v.push(scene(
        "title-fresh",
        vec![
            Action::EnterState(GameState::Title, 1.0),
            act(|w| {
                fresh_progress(w);
                w.resource_mut::<MenuUi>().open(MenuScreen::Main);
            }),
        ],
        0.8,
        None,
    ));
    v.push(scene(
        "title-continue",
        vec![act(|w| {
            clear_missions(w, CONTINUE_CLEARED);
            w.resource_mut::<SavedPlayerPosition>().0 = Some(Vec2::ZERO);
            w.resource_mut::<MenuUi>().open(MenuScreen::Main);
        })],
        0.8,
        boxed(fresh_progress),
    ));
}

fn world_scenes(v: &mut Vec<Scene>) {
    for (label, pos) in VANTAGES {
        v.push(Scene {
            file: format!("shot-{label}.png"),
            actions: vec![
                Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
                teleport_to(*pos),
            ],
            settle: 0.7,
            teardown: None,
        });
    }
    // Quest marker: fresh progress, so Ferris carries the golden '!'.
    let ferris = npc_slot(FIRST_MISSION).unwrap_or(Vec2::ZERO);
    v.push(scene(
        "quest-marker",
        vec![
            Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
            act(fresh_progress),
            teleport_to(ferris + Vec2::new(-40.0, -40.0)),
        ],
        0.8,
        None,
    ));
}

/// Player feet just in front of / just behind a prop's ground line.
/// Front: feet top flush with the solid's bottom (sprite overlaps the lower
/// wall/trunk; player must draw OVER it). Behind: feet bottom flush with
/// the solid's top (roof/canopy must draw OVER the player).
fn ysort_scenes(v: &mut Vec<Scene>) {
    let foot = crate::plugins::player::PLAYER_FOOT;
    let cottage = super::collision::probe_cottage_base();
    let oak = MEADOW_OAKS.first().copied();
    let cases: [(&'static str, PropKind, Option<Vec2>, f32); 2] = [
        ("cottage", PropKind::Cottage, cottage, 10.0),
        ("oak", PropKind::Oak, oak, 6.0),
    ];
    for (name, kind, base, x_off) in cases {
        let Some(base) = base else {
            tracing::warn!(name, "ysort prop missing; scene skipped");
            continue;
        };
        let solid = kind.solid_at(base);
        let front = Vec2::new(base.x + x_off, solid.min.y - foot.max.y);
        let behind = Vec2::new(base.x + x_off, solid.max.y - foot.min.y);
        for (side, pos) in [("front", front), ("behind", behind)] {
            v.push(scene(
                &format!("ysort-{name}-{side}"),
                vec![
                    Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
                    teleport_to(pos),
                    act(move |_w| {
                        tracing::info!(name, side, ?pos, "ysort scene: player placed");
                    }),
                ],
                0.6,
                None,
            ));
        }
    }
}

fn collision_scenes(v: &mut Vec<Scene>) {
    for p in probes() {
        let label = p.label;
        v.push(scene(
            &format!("collision-{label}"),
            vec![
                Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
                teleport_to(p.start),
                Action::Wait(0.2),
                Action::Hold(p.key, PUSH_SECS),
                Action::Wait(0.1),
                act(move |w| {
                    let solids: Vec<_> = {
                        let mut q = w.query::<&Solid>();
                        q.iter(w).map(|s| s.0).collect()
                    };
                    let Some(end) = player_pos(w) else {
                        tracing::warn!(label, "collision probe: no player");
                        return;
                    };
                    let inside = feet_inside_any(end, &solids);
                    let gap = probe_wall().map(|wall| gap_to_wall(&p, end, &wall));
                    let line = format!(
                        "collision {label}: start=({:.1},{:.1}) final=({:.1},{:.1}) \
                         gap_to_wall={gap:?} inside_solid={inside}",
                        p.start.x, p.start.y, end.x, end.y
                    );
                    if inside {
                        tracing::error!("{line}");
                    } else {
                        tracing::info!("{line}");
                    }
                    w.resource_mut::<ShotLog>().0.push(line);
                }),
            ],
            0.3,
            None,
        ));
    }
}

fn dialogue_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    let late_npc = NPC_ROSTER.last().map_or(FIRST_MISSION, |s| s.mission_id);

    // Offer, captured mid-typewriter (45 chars/s; ~15 chars after 0.35 s).
    v.push(scene(
        "dialogue-offer-typing",
        vec![
            enter(),
            act(fresh_progress),
            beside_npc(FIRST_MISSION),
            Action::Wait(0.4),
            Action::Tap(KeyCode::KeyF),
        ],
        0.35,
        boxed(close_dialogue),
    ));
    // The same four statuses with the first line fully revealed (second F).
    let statuses: [(&'static str, &'static str, WorldFn); 4] = [
        ("offer", FIRST_MISSION, Box::new(fresh_progress)),
        (
            "inprogress",
            FIRST_MISSION,
            Box::new(|w: &mut World| {
                fresh_progress(w);
                w.resource_mut::<DialogueMemory>()
                    .opened
                    .insert(FIRST_MISSION);
            }),
        ),
        (
            "cleared",
            FIRST_MISSION,
            Box::new(|w: &mut World| {
                fresh_progress(w);
                clear_missions(w, &[FIRST_MISSION]);
            }),
        ),
        ("notyet", late_npc, Box::new(fresh_progress)),
    ];
    for (name, npc, prep) in statuses {
        v.push(scene(
            &format!("dialogue-{name}"),
            vec![
                enter(),
                Action::Do(prep),
                beside_npc(npc),
                Action::Wait(0.4),
                Action::Tap(KeyCode::KeyF),
                Action::Wait(0.3),
                Action::Tap(KeyCode::KeyF),
                act(move |w| {
                    let open = w
                        .resource::<Dialogue>()
                        .active
                        .as_ref()
                        .map(|a| (a.speaker, a.status));
                    match open {
                        Some((speaker, status)) => {
                            tracing::info!(name, speaker, ?status, "dialogue scene open")
                        }
                        None => tracing::warn!(name, "dialogue scene: no dialogue open"),
                    }
                }),
            ],
            0.4,
            boxed(|w| {
                close_dialogue(w);
                fresh_progress(w);
            }),
        ));
    }
}

fn editor_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    let ferris = npc_slot(FIRST_MISSION).unwrap_or(Vec2::ZERO);
    v.push(scene(
        "editor-starter",
        vec![
            enter(),
            teleport_to(ferris + Vec2::new(-40.0, -40.0)),
            act(|w| {
                let starter = w
                    .resource::<MissionRegistry>()
                    .find(FIRST_MISSION)
                    .map(|m| m.starter_code)
                    .unwrap_or_default();
                open_editor(w, FIRST_MISSION, starter);
            }),
        ],
        0.8,
        None,
    ));
    v.push(scene(
        "editor-fail",
        vec![enter(), act(canned_failure), Action::Wait(0.2)],
        0.5,
        None,
    ));
    v.push(scene(
        "editor-pass",
        vec![
            enter(),
            act(|w| {
                let mut ed = w.resource_mut::<EditorState>();
                ed.source = PASSING_SOURCE.to_string();
                ed.last_compile_result = Some(CANNED_PASS.to_string());
            }),
        ],
        0.5,
        None,
    ));
    // Two more failures (editor-fail recorded the first), then reveal.
    v.push(scene(
        "editor-solution",
        vec![
            enter(),
            act(canned_failure),
            Action::Wait(0.15),
            act(canned_failure),
            Action::Wait(0.15),
            act(|w| {
                let n = w.resource::<EditorSession>().failures(FIRST_MISSION);
                if n < crate::plugins::editor::session::SOLUTION_UNLOCK_FAILURES {
                    tracing::warn!(n, "editor-solution: gate still locked");
                } else {
                    tracing::info!(n, "editor-solution: gate open");
                }
                w.resource_mut::<EditorSession>()
                    .reveal_solution(FIRST_MISSION);
            }),
            Action::Wait(0.15),
        ],
        0.5,
        boxed(close_editor),
    ));
}

/// Result-pane scenes for the honest-grades work. Each reopens the editor
/// on the first mission with a matching buffer and a canned server reply.
fn editor_result_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    let cases: [(&str, &'static str, CompileMode, &'static str); 5] = [
        (
            "editor-fail-long",
            LONG_SOURCE,
            CompileMode::Real,
            LONG_FAIL_JSON,
        ),
        (
            "editor-wrong-output",
            WRONG_SOURCE,
            CompileMode::Real,
            WRONG_OUTPUT_JSON,
        ),
        (
            "editor-unverified-pass",
            PASSING_SOURCE,
            CompileMode::Real,
            UNVERIFIED_JSON,
        ),
        (
            "editor-verified-pass",
            PASSING_SOURCE,
            CompileMode::Real,
            VERIFIED_JSON,
        ),
        (
            "editor-pattern-pass",
            PASSING_SOURCE,
            CompileMode::Pattern,
            PATTERN_JSON,
        ),
    ];
    for (name, source, mode, json) in cases {
        v.push(scene(
            name,
            vec![
                enter(),
                act(move |w| {
                    w.resource_mut::<EditorSession>().reset_view();
                    open_editor(w, FIRST_MISSION, source);
                    show_server_reply(w, mode, json);
                }),
                Action::Wait(0.2),
            ],
            0.5,
            boxed(close_editor),
        ));
    }
    // The smallest supported window: long failure, then the solution
    // (the gate is open: editor-fail + editor-solution recorded three
    // failures). Restores 1280x720 afterwards.
    v.push(scene(
        "editor-960x540-fail",
        vec![
            enter(),
            act(|w| set_window_size(w, 960.0, 540.0)),
            act(|w| {
                w.resource_mut::<EditorSession>().reset_view();
                open_editor(w, FIRST_MISSION, LONG_SOURCE);
                show_server_reply(w, CompileMode::Real, LONG_FAIL_JSON);
            }),
            Action::Wait(1.0),
        ],
        0.6,
        None,
    ));
    v.push(scene(
        "editor-960x540-solution",
        vec![
            act(|w| {
                w.resource_mut::<EditorSession>()
                    .reveal_solution(FIRST_MISSION);
            }),
            Action::Wait(0.3),
        ],
        0.5,
        boxed(|w| {
            close_editor(w);
            set_window_size(w, 1280.0, 720.0);
        }),
    ));
}

fn overlay_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    v.push(scene(
        "recap",
        vec![
            enter(),
            act(|w| {
                close_editor(w);
                clear_missions(w, &[FIRST_MISSION]);
                let mission = w.resource::<MissionRegistry>().find(FIRST_MISSION).cloned();
                w.resource_mut::<ActiveMission>().current = mission;
                w.resource_mut::<CompletionView>().mission_id = Some(FIRST_MISSION.to_string());
            }),
        ],
        0.6,
        boxed(|w| {
            w.resource_mut::<CompletionView>().mission_id = None;
            fresh_progress(w);
        }),
    ));
    v.push(scene(
        "pause",
        vec![
            enter(),
            act(|w| {
                w.resource_mut::<NextState<PauseState>>()
                    .set(PauseState::Paused);
            }),
            Action::Wait(0.3),
        ],
        0.4,
        None,
    ));
    v.push(scene(
        "settings",
        vec![
            act(|w| {
                let paused = w
                    .get_resource::<State<PauseState>>()
                    .is_some_and(|s| *s.get() == PauseState::Paused);
                if !paused {
                    tracing::warn!("settings scene: not paused");
                }
                w.resource_mut::<MenuUi>().open(MenuScreen::Settings);
            }),
            Action::Wait(0.2),
        ],
        0.4,
        boxed(|w| {
            w.resource_mut::<MenuUi>().open(MenuScreen::Main);
            w.resource_mut::<NextState<PauseState>>()
                .set(PauseState::Running);
        }),
    ));
}
