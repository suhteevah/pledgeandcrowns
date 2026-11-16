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
//! `MissionProgress::mark_opened`, the two `EditorSession` hooks, and in-app key taps /
//! holds on Bevy's `ButtonInput` (F to talk, WASD to walk).

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::collision::{PUSH_SECS, feet_inside_any, gap_to_wall, probe_wall, probes};
use super::{Action, Scene, ShotLog, WorldFn};
use crate::plugins::animation::{
    CharacterSheets, Facing, IdleBob, Scuttle, SpriteLift, WalkAnim, apply_frame, ferris_frame,
    player_frame,
};
use crate::plugins::compile_client::CompileMode;
use crate::plugins::compile_client::outcome::{HttpReply, classify, verdict_with_server};
use crate::plugins::dialogue::Dialogue;
use crate::plugins::editor::{EditorSession, EditorState};
use crate::plugins::menu::{MenuScreen, MenuUi};
use crate::plugins::mission::{
    ActiveMission, CompletionView, HideNextPinForCapture, MissionRegistry,
};
use crate::plugins::npc::{NPC_ROSTER, Npc, TALK_SPOT_OFFSET};
use crate::plugins::player::Player;
use crate::plugins::progress::{MissionProgress, SavedPlayerPosition, Settings};
use crate::plugins::prop::{MEADOW_OAKS, Prop, PropKind, Solid};
use crate::plugins::state::{GameState, PauseState};
use crate::plugins::ui_theme::{CodeSurface, DEFAULT_CODE_SURFACE, set_code_surface};
use crate::plugins::world::district_slot;

/// First mission (Ferris). The editor/recap/dialogue scenes use it.
const FIRST_MISSION: &str = "intro_let_binding";
/// Second mission: no failures recorded in a run, so its solution stays
/// locked (the toolbar shows the "Unlocks after ..." note).
const SECOND_MISSION: &str = "double_function";
/// Missions marked cleared for the "Continue available" title scene.
const CONTINUE_CLEARED: &[&str] = &["intro_let_binding", "double_function", "borrow_preview"];
/// The late-game villager the "notyet" dialogue scenes talk to: the
/// Ghostkeeper (`weak_ref`, Act 8), named outright so adding a roster entry
/// cannot change who these scenes show. Not a trial owner, so its locked
/// line is the generic "come back later" pointer.
const NOTYET_NPC: &str = "weak_ref";
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

/// [`LONG_SOURCE`] with a comment, a constant and a float added WITHOUT
/// moving any line, so [`LONG_FAIL_JSON`]'s locations still match: every
/// token class (keyword, type, string, comment, number, plain) on screen.
const SYNTAX_SOURCE: &str = "fn take(v: String) {} // takes ownership of `v`\nconst LIMIT: u32 = 3_000; const RATIO: f64 = 0.75;\nfn main() {\n    let answer = 42;\n    answer = 43;\n    let n: i32 = \"five\";\n    let s = String::from(\"hi\");\n    take(s);\n    println!(\"{} {}\", s, y);\n}\n";

/// The code-surface comparison sample (bible v2.2 candidate vs v2.1):
/// every token class several times over. Keywords, types with generics
/// and lifetimes, strings with format placeholders, integer / hex / float
/// literals, line, doc and block comments, a macro call and an attribute.
/// Player line 23 reassigns an immutable binding (see [`SURFACE_FAIL`]).
const SURFACE_SOURCE: &str = "//! Ferris's ledger: every token class on one page.
use std::collections::HashMap;

/* Block comment: a quest is a name and a reward. */
#[derive(Debug, Clone)]
struct Quest<'a> {
    name: &'a str,
    reward: u32, // gold crowns
}

fn ratio<T: Into<f64>>(won: T, total: f64) -> f64 {
    won.into() / total
}

fn main() {
    let quests: Vec<Quest<'static>> = vec![Quest { name: \"Ferris\", reward: 42 }];
    let mut ledger: HashMap<&str, u32> = HashMap::new();
    let rate = ratio(3, 4.0) * 100.0;
    for q in &quests {
        ledger.insert(q.name, q.reward + 0x10);
    }
    let total = 1_000;
    total = total + 1;
    println!(\"{} quests, total {total:>5}\", quests.len());
}
";

/// rustc's reply for [`SURFACE_SOURCE`] in the shape `compile_client`
/// writes for a graded failure (server lines are one below the player's).
/// One error with a `help:`, one warning with a `note:`.
const SURFACE_FAIL: &str = "[fail:compile] [real]\n--stderr--\n\
error[E0384]: cannot assign twice to immutable variable `total`\n  \
--> src/main.rs:24:5\n   |\n\
23 |     let total = 1_000;\n   |         ----- first assignment to `total`\n\
24 |     total = total + 1;\n   |     ^^^^^^^^^^^^^^^^^ cannot assign twice to immutable variable\n   |\n\
help: consider making this binding mutable\n   |\n\
23 |     let mut total = 1_000;\n   |         +++\n\n\
warning: unused variable: `rate`\n  \
--> src/main.rs:19:9\n   |\n\
19 |     let rate = ratio(3, 4.0) * 100.0;\n   |         ^^^^ help: if this is intentional, prefix it with an underscore: `_rate`\n   |\n   \
= note: `#[warn(unused_variables)]` on by default\n\n\
error: aborting due to 1 previous error; 1 warning emitted\n\n\
For more information about this error, try `rustc --explain E0384`.";

const WRONG_SOURCE: &str = "fn main() {\n    let answer = 41;\n    println!(\"{answer}\");\n}\n";
/// `/compile-real` wrong-output reply, verbatim server shape.
const WRONG_OUTPUT_JSON: &str = r#"{"ok":false,"stdout":"[real] 41","stderr":"Your program ran, but did not do what this mission asks.\nExpected a line reading `42`\nGot:\n41","grade":"wrong_output","behaviour_checked":true,"expected":"a line reading `42`","actual":"41","note":"output checked against the mission"}"#;
/// `/compile-real` compile-only mission: accepted, behaviour not checked.
const UNVERIFIED_JSON: &str = r#"{"ok":true,"stdout":"[real] compiled\n-- not checked: behaviour not checked on this server: wasm32-wasip1 has no threads, so this program is compiled and statically checked but never run","stderr":"","grade":"compiled_not_checked","behaviour_checked":false,"note":"behaviour not checked on this server: wasm32-wasip1 has no threads, so this program is compiled and statically checked but never run"}"#;
/// `/compile-real` verified pass.
const VERIFIED_JSON: &str = r#"{"ok":true,"stdout":"[real] 42\n-- checked: does what the mission asks","stderr":"","grade":"passed","behaviour_checked":true,"note":"output checked against the mission"}"#;
/// `/compile-real` pass of an Act 9 `UnsafeLateNonce` harness mission, the
/// note exactly as the server words it (8 checks, like `librarian_index`).
const UNSAFE_ALLOWED_JSON: &str = r#"{"ok":true,"stdout":"[real] 42\n-- checked: does what the mission asks","stderr":"","grade":"passed","behaviour_checked":true,"note":"checked by calling your code (8 checks; `unsafe` allowed, results keyed by the server only after your code finished)"}"#;
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

/// Fresh in-memory progress: nothing cleared or opened, no saved position.
fn fresh_progress(world: &mut World) {
    world.resource_mut::<MissionProgress>().reset();
    world.resource_mut::<SavedPlayerPosition>().0 = None;
    tracing::debug!("scene: progress reset (in-memory)");
}

/// Fresh progress, then clear every mission that comes before `id` in
/// registry order (in-memory). Unknown ids clear nothing.
fn clear_before(world: &mut World, id: &str) {
    fresh_progress(world);
    let ids: Vec<&'static str> = world
        .resource::<MissionRegistry>()
        .missions
        .iter()
        .map(|m| m.id)
        .take_while(|m| *m != id)
        .collect();
    if ids.len() == world.resource::<MissionRegistry>().missions.len() {
        tracing::warn!(id, "clear_before: mission not in registry; nothing cleared");
        return;
    }
    clear_missions(world, &ids);
}

/// The registry prerequisite of `id`, if any.
fn prereq_of(world: &World, id: &str) -> Option<&'static str> {
    world
        .resource::<MissionRegistry>()
        .find(id)
        .and_then(|m| m.prereq)
}

/// Log (and record in the DONE file) the lowest alpha among props whose
/// art overlaps the player's sprite, so a capture can be checked for the
/// occlusion fade without eyeballing it.
fn log_occluder_alpha(world: &mut World, label: &'static str) {
    let Some(p) = player_pos(world) else {
        tracing::warn!(label, "fade scene: no player");
        return;
    };
    let body = crate::plugins::prop::character_body(p, Vec2::splat(32.0));
    let mut q = world.query::<(&Transform, &Prop, &Sprite)>();
    let min_alpha = q
        .iter(world)
        .filter(|(t, prop, _)| {
            prop.kind
                .visual_at(t.translation.truncate())
                .overlaps(&body)
        })
        .map(|(_, _, s)| s.color.alpha())
        .fold(None, |acc: Option<f32>, a| {
            Some(acc.map_or(a, |m| m.min(a)))
        });
    let line = format!(
        "fade {label}: player=({:.1},{:.1}) overlapping_prop_min_alpha={min_alpha:?}",
        p.x, p.y
    );
    tracing::info!("{line}");
    world.resource_mut::<ShotLog>().0.push(line);
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

/// Stand in front of (south of) an NPC, inside the talk radius, as a
/// player walking up to them would (see `npc::TALK_SPOT_OFFSET`).
fn beside_npc(mission_id: &'static str) -> Action {
    act(move |w| match npc_slot(mission_id) {
        Some(slot) => teleport(w, slot + TALK_SPOT_OFFSET),
        None => tracing::warn!(mission_id, "no NPC slot"),
    })
}

fn close_dialogue(world: &mut World) {
    world.resource_mut::<Dialogue>().active = None;
}

/// Hide the "next mission" HUD pin for one capture (it covers the
/// top-left of the view). Undone by [`show_next_pin`] in the teardown.
fn hide_next_pin(world: &mut World) {
    world.insert_resource(HideNextPinForCapture);
}

fn show_next_pin(world: &mut World) {
    world.remove_resource::<HideNextPinForCapture>();
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
    district_npc_scenes(&mut v);
    animation_scenes(&mut v);
    ysort_scenes(&mut v);
    collision_scenes(&mut v);
    fade_scenes(&mut v);
    dialogue_scenes(&mut v);
    dialogue_fix_scenes(&mut v);
    editor_scenes(&mut v);
    editor_result_scenes(&mut v);
    code_surface_scenes(&mut v);
    trial_scenes(&mut v);
    overlay_scenes(&mut v);
    v
}

/// Roster key of the Borrow Checker, who owns the Bridge trial.
const TRIAL_NPC: &str = "borrow_preview";

/// Make `id` the active mission (what the prompt panel and `E` show).
fn set_active(world: &mut World, id: &str) {
    let mission = world.resource::<MissionRegistry>().find(id).cloned();
    if mission.is_none() {
        tracing::warn!(id, "set_active: unknown mission");
    }
    world.resource_mut::<ActiveMission>().current = mission;
}

/// Undo a trial scene: no panel, no editor, no active mission, fresh
/// progress, trial bookkeeping reset.
fn reset_trial_scene(world: &mut World) {
    close_dialogue(world);
    close_editor(world);
    world.resource_mut::<CompletionView>().mission_id = None;
    world.resource_mut::<ActiveMission>().current = None;
    *world.resource_mut::<crate::plugins::trial::TrialView>() = Default::default();
    fresh_progress(world);
}

/// The Borrow Checker's trial: the offer dialogue, the prompt (mission)
/// panel with stage pips at stage 1 and stage 2, the editor on a stage,
/// the "next stage" panel and the trial-complete panel.
fn trial_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    v.push(scene(
        "trial-offer",
        vec![
            enter(),
            act(|w| {
                reset_trial_scene(w);
                clear_before(w, "bridge_toll");
            }),
            beside_npc(TRIAL_NPC),
            Action::Wait(0.4),
            Action::Tap(KeyCode::KeyF),
            Action::Wait(0.3),
            Action::Tap(KeyCode::KeyF),
            act(|w| {
                let open = w.resource::<Dialogue>().active.as_ref().map(|a| {
                    (
                        a.mission_id,
                        a.status,
                        a.stage.as_ref().map(|s| s.label.clone()),
                    )
                });
                let line = format!("trial-offer dialogue: {open:?}");
                tracing::info!("{line}");
                w.resource_mut::<ShotLog>().0.push(line);
            }),
        ],
        0.4,
        boxed(reset_trial_scene),
    ));
    // The Act 3-10 trials: each owner's offer of stage 1, the same way.
    for (name, npc, stage1) in [
        ("trial-offer-wraith", "impl_method", "wraith_forms"),
        ("trial-offer-chimera", "trait_def", "chimera_heads"),
        ("trial-offer-wager", "result_match", "wager_stake"),
        ("trial-offer-golem", "hashmap_basic", "golem_parts"),
        ("trial-offer-hydra", "thread_spawn", "hydra_senders"),
        ("trial-offer-warden", "box_basic", "warden_box"),
        // Act 9: no prologue, the Librarian's roster mission is stage 1.
        (
            "trial-offer-librarian",
            "librarian_index",
            "librarian_index",
        ),
        // A Library villager's first offer (generic lines; no trial).
        ("dialogue-library-offer", "raw_pointer", "raw_pointer"),
        // Act 10: the Compiler, no prologue either; and a Throne villager.
        ("trial-offer-compiler", "compiler_alloc", "compiler_alloc"),
        ("dialogue-throne-offer", "prealloc", "prealloc"),
    ] {
        v.push(scene(
            name,
            vec![
                enter(),
                act(move |w| {
                    reset_trial_scene(w);
                    clear_before(w, stage1);
                }),
                beside_npc(npc),
                Action::Wait(0.4),
                Action::Tap(KeyCode::KeyF),
                Action::Wait(0.3),
                Action::Tap(KeyCode::KeyF),
                act(move |w| {
                    let open = w.resource::<Dialogue>().active.as_ref().map(|a| {
                        (
                            a.mission_id,
                            a.status,
                            a.stage.as_ref().map(|s| s.label.clone()),
                        )
                    });
                    let line = format!("{name} dialogue: {open:?}");
                    tracing::info!("{line}");
                    w.resource_mut::<ShotLog>().0.push(line);
                }),
            ],
            0.4,
            boxed(reset_trial_scene),
        ));
    }
    for (name, stage) in [
        ("trial-panel-stage1", "bridge_toll"),
        ("trial-panel-stage2", "bridge_quill"),
    ] {
        v.push(scene(
            name,
            vec![
                enter(),
                act(move |w| {
                    reset_trial_scene(w);
                    clear_before(w, stage);
                    w.resource_mut::<MissionProgress>().mark_opened(stage);
                    set_active(w, stage);
                }),
                beside_npc(TRIAL_NPC),
                Action::Wait(0.4),
            ],
            0.3,
            boxed(reset_trial_scene),
        ));
    }
    v.push(scene(
        "trial-editor",
        vec![
            enter(),
            act(|w| {
                reset_trial_scene(w);
                clear_before(w, "bridge_quill");
                let starter = w
                    .resource::<MissionRegistry>()
                    .find("bridge_quill")
                    .map(|m| m.starter_code)
                    .unwrap_or_default();
                open_editor(w, "bridge_quill", starter);
            }),
        ],
        0.8,
        boxed(reset_trial_scene),
    ));
    for (name, cleared_through, view) in [
        ("trial-next-stage", "bridge_quill", "bridge_toll"),
        ("trial-complete", "impl_method", "bridge_lantern"),
    ] {
        v.push(scene(
            name,
            vec![
                enter(),
                act(move |w| {
                    reset_trial_scene(w);
                    clear_before(w, cleared_through);
                    set_active(w, view);
                    w.resource_mut::<CompletionView>().mission_id = Some(view.to_string());
                }),
                Action::Wait(0.2),
            ],
            0.5,
            boxed(reset_trial_scene),
        ));
    }
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
    // New Game over a save: the confirmation screen.
    v.push(scene(
        "title-confirm",
        vec![act(|w| {
            clear_missions(w, CONTINUE_CLEARED);
            w.resource_mut::<MenuUi>().open(MenuScreen::ConfirmNewGame);
        })],
        0.6,
        boxed(|w| {
            w.resource_mut::<MenuUi>().open(MenuScreen::Main);
            fresh_progress(w);
        }),
    ));
    v.push(scene(
        "title-settings",
        vec![act(|w| {
            w.resource_mut::<MenuUi>().open(MenuScreen::Settings);
        })],
        0.6,
        boxed(|w| {
            w.resource_mut::<MenuUi>().open(MenuScreen::Main);
        }),
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
    // Hearthstone Square's north-east corner: flagstone interior, the
    // one-tile cobble ring turning the corner, the N-S street entering
    // through the north edge (left of frame) and two hub villagers.
    v.push(scene(
        "plaza-corner",
        vec![
            Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
            teleport_to(Vec2::new(150.0, 235.0)),
        ],
        0.7,
        None,
    ));
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
    // Each back landmark (the Sealed Stacks, REF-95; the Throne Dais,
    // REF-96) at its plaza's back-centre: the player stands beside it
    // (east of its footing, clear of every NPC), the landmark and the boss
    // in front of it in view. Logs the spot so the capture can be checked.
    for &(d, kind) in crate::plugins::prop::LANDMARKS {
        let Some(base) = crate::plugins::world::back_landmark_base(d) else {
            tracing::warn!(?kind, "landmark scene: no district {d}");
            continue;
        };
        use crate::plugins::prop::PropKind;
        let file = match kind {
            PropKind::SealedStacks => "landmark-sealed-stacks",
            PropKind::ThroneDais => "landmark-throne-dais",
            PropKind::Oak | PropKind::Cottage => "landmark-other",
        };
        let solid = kind.solid_at(base);
        let feet = Vec2::new(solid.max.x + 48.0, solid.min.y);
        v.push(scene(
            file,
            vec![
                Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
                act(hide_next_pin),
                teleport_to(feet),
                act(move |w| {
                    let line = format!(
                        "landmark {kind:?}: base=({:.0},{:.0}) solid=({:.0},{:.0})..({:.0},{:.0})",
                        base.x, base.y, solid.min.x, solid.min.y, solid.max.x, solid.max.y
                    );
                    tracing::info!("{line}");
                    w.resource_mut::<ShotLog>().0.push(line);
                }),
            ],
            0.7,
            boxed(show_next_pin),
        ));
    }
}

// --- Act 3-8 villagers and animation frames ----------------------------------

/// The districts that hold the Act 3-10 casts (roster order fills
/// `DISTRICTS` in order; index 0 is the hub, 1-2 hold Acts 1-2, 3-8 are
/// the ring quarters, 9 is the Forbidden Library, 10 the Throne).
const ACT_DISTRICTS: std::ops::RangeInclusive<usize> = 3..=10;

/// Lowercase, dash-separated file label for a district name.
fn district_label(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect::<String>()
        .to_ascii_lowercase()
        .replace(' ', "-")
}

/// One capture per Act 3-10 district, camera on the plaza centre: all six
/// of a ring quarter's villagers (3x2 grid) fit in the 280 px tall view.
/// The Library's and the Throne's 3x3 grids (152 px rows, shifted south of
/// the back-centre landmark) do not: their front capture centres between
/// the two villager rows, and `npcs-<district>-back` centres just in front
/// of the boss, the landmark spot behind him.
fn district_npc_scenes(v: &mut Vec<Scene>) {
    let mut first = 0usize;
    for (d, district) in crate::plugins::world::DISTRICTS.iter().enumerate() {
        let roster = first..first + district.count;
        first += district.count;
        if !ACT_DISTRICTS.contains(&d) {
            continue;
        }
        let names: Vec<&'static str> = NPC_ROSTER
            .get(roster)
            .unwrap_or_default()
            .iter()
            .map(|s| s.name)
            .collect();
        let label = district.name;
        // A district taller than one view (the Library's three rows: the
        // centre is a slot, the Courier's) is shot between its front two
        // rows, then again just in front of its last slot (the Librarian).
        let (center, back) = if district.count > 6 {
            let front = district_slot(first - district.count);
            let middle = district_slot(first - district.count + 3);
            let mid_y = (front.y + middle.y) * 0.5;
            let back = district_slot(first - 1);
            (Vec2::new(district.center.x, mid_y), Some(back))
        } else {
            (district.center, None)
        };
        // The "next mission" pin sits top-left and would cover the
        // top-left villager, so these line-ups hide it for the capture.
        v.push(scene(
            &format!("npcs-{}", district_label(label)),
            vec![
                Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
                act(hide_next_pin),
                teleport_to(center),
                act(move |w| {
                    let line = format!("npcs {label}: {}", names.join(", "));
                    tracing::info!("{line}");
                    w.resource_mut::<ShotLog>().0.push(line);
                }),
            ],
            0.7,
            boxed(show_next_pin),
        ));
        if let Some(back) = back {
            v.push(scene(
                &format!("npcs-{}-back", district_label(label)),
                vec![
                    Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
                    act(hide_next_pin),
                    // Beside the boss, so the landmark spot behind him
                    // is in view and the player hides nobody.
                    teleport_to(Vec2::new(back.x + 64.0, back.y - 20.0)),
                ],
                0.7,
                boxed(show_next_pin),
            ));
        }
    }
}

/// Open hub flagstone, clear of every NPC slot and prop, for the walk poses.
const WALK_SPOT: Vec2 = Vec2::new(120.0, -120.0);

/// Freeze the player on one walk frame. The animation system only drives
/// entities that carry `WalkAnim`, so removing it lets the forced frame
/// stand for the capture; [`restore_player_walk`] puts it back.
fn force_player_frame(world: &mut World, facing: Facing, cycle: usize) {
    let Some(sheets) = world.get_resource::<CharacterSheets>().cloned() else {
        tracing::warn!("walk scene: CharacterSheets not loaded");
        return;
    };
    let players: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, With<Player>>();
        q.iter(world).collect()
    };
    for e in &players {
        world.entity_mut(*e).remove::<WalkAnim>();
    }
    let frame = player_frame(facing, Some(cycle));
    let mut q = world.query_filtered::<(&mut Sprite, &mut Anchor, &mut SpriteLift), With<Player>>();
    for (mut sprite, mut anchor, mut lift) in q.iter_mut(world) {
        apply_frame(&mut sprite, &mut anchor, &mut lift, frame, &sheets.player);
    }
    tracing::info!(?facing, cycle, ?frame, "walk scene: player frame forced");
}

fn restore_player_walk(world: &mut World) {
    let players: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, With<Player>>();
        q.iter(world).collect()
    };
    for e in players {
        world.entity_mut(e).insert(WalkAnim::default());
    }
}

/// The Ferris NPC entity (the first mission's owner).
fn ferris_entity(world: &mut World) -> Option<Entity> {
    let mut q = world.query::<(Entity, &Npc)>();
    q.iter(world)
        .find(|(_, n)| n.mission_id == FIRST_MISSION)
        .map(|(e, _)| e)
}

/// Freeze Ferris on a scuttle frame (his scuttle and bob are removed for
/// the capture and restored by [`restore_ferris`]).
fn force_ferris_frame(world: &mut World, cycle: usize) {
    let Some(sheets) = world.get_resource::<CharacterSheets>().cloned() else {
        tracing::warn!("scuttle scene: CharacterSheets not loaded");
        return;
    };
    let Some(e) = ferris_entity(world) else {
        tracing::warn!("scuttle scene: no Ferris entity");
        return;
    };
    world.entity_mut(e).remove::<(Scuttle, IdleBob)>();
    let frame = ferris_frame(Some(cycle));
    let mut q = world.query::<(&mut Sprite, &mut Anchor, &mut SpriteLift)>();
    if let Ok((mut sprite, mut anchor, mut lift)) = q.get_mut(world, e) {
        apply_frame(&mut sprite, &mut anchor, &mut lift, frame, &sheets.ferris);
        tracing::info!(cycle, ?frame, "scuttle scene: Ferris frame forced");
    }
}

fn restore_ferris(world: &mut World) {
    let Some(e) = ferris_entity(world) else {
        return;
    };
    let index = NPC_ROSTER
        .iter()
        .position(|s| s.mission_id == FIRST_MISSION)
        .unwrap_or(0);
    let frame = ferris_frame(None);
    if let Some(sheets) = world.get_resource::<CharacterSheets>().cloned() {
        let mut q = world.query::<(&mut Sprite, &mut Anchor, &mut SpriteLift)>();
        if let Ok((mut sprite, mut anchor, mut lift)) = q.get_mut(world, e) {
            apply_frame(&mut sprite, &mut anchor, &mut lift, frame, &sheets.ferris);
        }
    }
    world
        .entity_mut(e)
        .insert((Scuttle::default(), IdleBob::for_roster_index(index)));
}

/// The player in each facing mid-stride (stride A, cycle frame 0) and Ferris
/// mid-scuttle (`walk_0`). Static captures, so the frames are forced.
fn animation_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    for (name, facing) in [
        ("down", Facing::Down),
        ("up", Facing::Up),
        ("left", Facing::Left),
        ("right", Facing::Right),
    ] {
        v.push(scene(
            &format!("walk-{name}"),
            vec![
                enter(),
                teleport_to(WALK_SPOT),
                Action::Wait(0.1),
                act(move |w| force_player_frame(w, facing, 0)),
            ],
            0.4,
            boxed(restore_player_walk),
        ));
    }
    let ferris = npc_slot(FIRST_MISSION).unwrap_or(Vec2::ZERO);
    v.push(scene(
        "ferris-scuttle",
        vec![
            enter(),
            teleport_to(ferris + Vec2::new(-40.0, -40.0)),
            Action::Wait(0.1),
            act(|w| force_ferris_frame(w, 0)),
        ],
        0.4,
        boxed(restore_ferris),
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
    let late_npc = NOTYET_NPC;

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
                w.resource_mut::<MissionProgress>()
                    .mark_opened(FIRST_MISSION);
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

/// Occlusion fade proofs: the player behind the cottage roof, behind the
/// oak canopy, and flush against the cottage's east and west walls (under
/// the eave). Each logs the overlapping props' alpha to the DONE file.
fn fade_scenes(v: &mut Vec<Scene>) {
    let foot = crate::plugins::player::PLAYER_FOOT;
    let foot_cy = (foot.min.y + foot.max.y) * 0.5;
    let mut cases: Vec<(&'static str, Vec2)> = Vec::new();
    if let Some(base) = super::collision::probe_cottage_base() {
        let wall = PropKind::Cottage.solid_at(base);
        let wall_cy = (wall.min.y + wall.max.y) * 0.5;
        cases.push((
            "cottage-behind",
            Vec2::new(base.x + 10.0, wall.max.y - foot.min.y),
        ));
        cases.push((
            "cottage-east",
            Vec2::new(wall.max.x - foot.min.x, wall_cy - foot_cy),
        ));
        cases.push((
            "cottage-west",
            Vec2::new(wall.min.x - foot.max.x, wall_cy - foot_cy),
        ));
    }
    if let Some(base) = MEADOW_OAKS.first().copied() {
        let trunk = PropKind::Oak.solid_at(base);
        cases.push((
            "oak-behind",
            Vec2::new(base.x + 6.0, trunk.max.y - foot.min.y),
        ));
    }
    for (label, pos) in cases {
        v.push(scene(
            &format!("fade-{label}"),
            vec![
                Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE),
                teleport_to(pos),
                Action::Wait(0.5),
                act(move |w| log_occluder_alpha(w, label)),
            ],
            0.1,
            None,
        ));
    }
}

/// Dialogue fixes: inline code (mid-typewriter and complete) and the
/// NotYetAvailable pointer for a mid-game save and a direct-prereq save.
fn dialogue_fix_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    let late_npc = NOTYET_NPC;
    // Ferris's second offer line has `let` around visible char 38-41
    // (45 chars/s): open, complete line 1, advance, then capture ~0.9 s
    // into line 2 (code chip half typed) and again once it is complete.
    let to_second_line = || {
        vec![
            enter(),
            act(fresh_progress),
            beside_npc(FIRST_MISSION),
            Action::Wait(0.4),
            Action::Tap(KeyCode::KeyF),
            Action::Wait(0.2),
            Action::Tap(KeyCode::KeyF),
            Action::Wait(0.2),
            Action::Tap(KeyCode::KeyF),
        ]
    };
    v.push(scene(
        "dialogue-code-typing",
        to_second_line(),
        0.88,
        boxed(close_dialogue),
    ));
    let mut complete = to_second_line();
    complete.push(Action::Wait(0.3));
    complete.push(Action::Tap(KeyCode::KeyF));
    v.push(scene(
        "dialogue-code-complete",
        complete,
        0.4,
        boxed(close_dialogue),
    ));

    // Mid-game: the first six missions cleared; the Ghostkeeper points at the
    // seventh mission's NPC, not at Ferris and not at "themselves next".
    let prep_mid: WorldFn = Box::new(|w: &mut World| {
        let seventh = w
            .resource::<MissionRegistry>()
            .missions
            .get(6)
            .map(|m| m.id);
        match seventh {
            Some(id) => clear_before(w, id),
            None => tracing::warn!("notyet-midgame: registry has < 7 missions"),
        }
    });
    // Everything before the speaker's prerequisite cleared: the speaker
    // names that prerequisite as the lesson right before theirs.
    let prep_prereq: WorldFn = Box::new(move |w: &mut World| match prereq_of(w, late_npc) {
        Some(pre) => clear_before(w, pre),
        None => tracing::warn!(late_npc, "notyet-prereq: speaker has no prereq"),
    });
    for (name, prep) in [("notyet-midgame", prep_mid), ("notyet-prereq", prep_prereq)] {
        v.push(scene(
            &format!("dialogue-{name}"),
            vec![
                enter(),
                Action::Do(prep),
                beside_npc(late_npc),
                Action::Wait(0.4),
                Action::Tap(KeyCode::KeyF),
                Action::Wait(0.3),
                Action::Tap(KeyCode::KeyF),
                act(move |w| {
                    let line = w
                        .resource::<Dialogue>()
                        .active
                        .as_ref()
                        .map(|a| (a.speaker, a.status, a.run.line().to_string()));
                    match line {
                        Some((speaker, status, text)) => {
                            let entry = format!("dialogue {name}: {speaker} {status:?}: {text}");
                            tracing::info!("{entry}");
                            w.resource_mut::<ShotLog>().0.push(entry);
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
    let cases: [(&str, &'static str, CompileMode, &'static str); 6] = [
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
            "editor-unsafe-allowed-pass",
            PASSING_SOURCE,
            CompileMode::Real,
            UNSAFE_ALLOWED_JSON,
        ),
        (
            "editor-pattern-pass",
            PASSING_SOURCE,
            CompileMode::Pattern,
            PATTERN_JSON,
        ),
    ];
    // Every syntax colour (bible v2.1) on the code page at 1280x720, with
    // the long failure's error marks. Same line numbers as LONG_SOURCE so
    // the canned rustc locations still land on the right lines.
    v.push(scene(
        "editor-syntax",
        vec![
            enter(),
            act(|w| {
                w.resource_mut::<EditorSession>().reset_view();
                open_editor(w, FIRST_MISSION, SYNTAX_SOURCE);
                show_server_reply(w, CompileMode::Real, LONG_FAIL_JSON);
            }),
            Action::Wait(0.2),
        ],
        0.5,
        boxed(close_editor),
    ));
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

/// Switch the code surface for one scene (display only).
fn set_surface(surface: CodeSurface) -> Action {
    act(move |_w| {
        tracing::info!(?surface, "scene: code surface");
        set_code_surface(surface);
    })
}

/// Teardown that runs `then` and puts the default code surface back.
fn restore_surface(then: impl Fn(&mut World) + Send + Sync + 'static) -> Option<WorldFn> {
    boxed(move |w| {
        then(w);
        set_code_surface(DEFAULT_CODE_SURFACE);
    })
}

/// Show `raw` (a formatted verdict, as `compile_client` writes it) without
/// counting a failure.
fn show_verdict(world: &mut World, source: &str, raw: &str) {
    let mut ed = world.resource_mut::<EditorState>();
    ed.source = source.to_string();
    ed.last_compile_result = Some(raw.to_string());
}

/// Ink (bible v2.2 candidate) and parchment (v2.1) captures of the same
/// screens from one build, for the owner's choice. Each scene sets its
/// surface, captures and restores [`DEFAULT_CODE_SURFACE`]. Runs after the
/// editor scenes, so the first mission's solution gate is already open.
fn code_surface_scenes(v: &mut Vec<Scene>) {
    let enter = || Action::EnterState(GameState::InGame, ENTER_GAME_SETTLE);
    for (suffix, surface) in [
        ("ink", CodeSurface::Ink),
        ("parchment", CodeSurface::Parchment),
    ] {
        v.push(scene(
            &format!("editor-syntax-{suffix}"),
            vec![
                enter(),
                set_surface(surface),
                act(|w| {
                    w.resource_mut::<EditorSession>().reset_view();
                    open_editor(w, FIRST_MISSION, SURFACE_SOURCE);
                    show_verdict(w, SURFACE_SOURCE, SURFACE_FAIL);
                }),
                Action::Wait(0.2),
            ],
            0.5,
            restore_surface(close_editor),
        ));
        v.push(scene(
            &format!("editor-fail-{suffix}"),
            vec![
                enter(),
                set_surface(surface),
                act(|w| {
                    w.resource_mut::<EditorSession>().reset_view();
                    open_editor(w, FIRST_MISSION, BUGGY_SOURCE);
                    show_verdict(w, BUGGY_SOURCE, CANNED_FAIL);
                }),
                Action::Wait(0.2),
            ],
            0.5,
            restore_surface(close_editor),
        ));
        v.push(scene(
            &format!("editor-solution-{suffix}"),
            vec![
                enter(),
                set_surface(surface),
                act(move |w| {
                    open_editor(w, FIRST_MISSION, BUGGY_SOURCE);
                    show_verdict(w, BUGGY_SOURCE, CANNED_FAIL);
                    let n = w.resource::<EditorSession>().failures(FIRST_MISSION);
                    if n < crate::plugins::editor::session::SOLUTION_UNLOCK_FAILURES {
                        tracing::warn!(n, "editor-solution-{suffix}: gate still locked");
                    }
                    w.resource_mut::<EditorSession>()
                        .reveal_solution(FIRST_MISSION);
                }),
                Action::Wait(0.2),
            ],
            0.5,
            restore_surface(|w| {
                w.resource_mut::<EditorSession>().reset_view();
                close_editor(w);
            }),
        ));
        v.push(scene(
            &format!("recap-{suffix}"),
            vec![
                enter(),
                set_surface(surface),
                act(|w| {
                    close_editor(w);
                    clear_missions(w, &[FIRST_MISSION]);
                    let mission = w.resource::<MissionRegistry>().find(FIRST_MISSION).cloned();
                    w.resource_mut::<ActiveMission>().current = mission;
                    w.resource_mut::<CompletionView>().mission_id = Some(FIRST_MISSION.to_string());
                }),
            ],
            0.6,
            restore_surface(|w| {
                w.resource_mut::<CompletionView>().mission_id = None;
                fresh_progress(w);
            }),
        ));
    }
    // 960x540 with a mission whose solution is still locked: the toolbar's
    // "Unlocks after N more failed compiles." note must not clip.
    v.push(scene(
        "editor-960x540-locked",
        vec![
            enter(),
            act(|w| set_window_size(w, 960.0, 540.0)),
            act(|w| {
                w.resource_mut::<EditorSession>().reset_view();
                open_editor(w, SECOND_MISSION, BUGGY_SOURCE);
                show_verdict(w, BUGGY_SOURCE, CANNED_FAIL);
            }),
            Action::Wait(1.0),
        ],
        0.6,
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
    // NPC prompt panel with the mission brief (an opened, uncleared
    // mission), plus the "next:" HUD pin.
    v.push(scene(
        "prompt",
        vec![
            enter(),
            act(|w| {
                fresh_progress(w);
                let mission = w.resource::<MissionRegistry>().find(FIRST_MISSION).cloned();
                w.resource_mut::<ActiveMission>().current = mission;
            }),
            beside_npc(FIRST_MISSION),
            Action::Wait(0.3),
        ],
        0.4,
        boxed(|w| {
            w.resource_mut::<ActiveMission>().current = None;
        }),
    ));
    // Every mission in this build cleared: the epilogue (full game) or the
    // end-of-demo panel (`demo` feature).
    let end_name = if cfg!(feature = "demo") {
        "demo-end"
    } else {
        "epilogue"
    };
    v.push(scene(
        end_name,
        vec![
            enter(),
            act(|w| {
                fresh_progress(w);
                let ids: Vec<&'static str> = w
                    .resource::<MissionRegistry>()
                    .missions
                    .iter()
                    .filter(|m| m.available)
                    .map(|m| m.id)
                    .collect();
                clear_missions(w, &ids);
            }),
            teleport_to(Vec2::new(0.0, -470.0)),
            Action::Wait(0.2),
        ],
        0.4,
        boxed(fresh_progress),
    ));
}
