// SPDX-License-Identifier: MIT
//! Mission registry + the F-press dialogue/editor handoff.
//!
//! A mission is a pairing of (npc_prompt, starter_code, encounter_id).
//! When the player stands by an NPC and presses F, this plugin loads
//! the mission's starter code into the editor and pops it open. The
//! editor's existing Compile button (wired in `compile_client`) sends
//! `encounter_id` to the API so server-side validation can branch on
//! which mission is being graded.

use crate::plugins::editor::EditorState;
use crate::plugins::npc::NearbyNpc;
use crate::plugins::progress::MissionProgress;
use crate::plugins::state::GameState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

#[derive(Clone)]
pub struct Mission {
    pub id: &'static str,
    pub npc_name: &'static str,
    pub prompt: &'static str,
    pub starter_code: &'static str,
}

#[derive(Resource)]
pub struct MissionRegistry {
    pub missions: Vec<Mission>,
}

impl Default for MissionRegistry {
    fn default() -> Self {
        Self {
            missions: vec![
                // Starters are deliberately neutral templates — they
                // must NOT contain the grader's required tokens, or
                // the player could submit them unchanged and pass.
                // The contract suite (game/tests/contract.rs) enforces
                // this invariant.
                Mission {
                    id: "intro_let_binding",
                    npc_name: "Ferris",
                    prompt: "Greetings! Bind the integer forty-two to a variable named answer.",
                    starter_code: "fn main() {\n    let _todo = 0;\n    println!(\"{_todo}\");\n}\n",
                },
                Mission {
                    id: "double_function",
                    npc_name: "Trait Mage",
                    prompt: "Define a function `double` that returns its `i32` argument multiplied by two.",
                    starter_code: "fn _todo() {}\n\nfn main() {\n    // call your function with 21 and print the result\n}\n",
                },
                Mission {
                    id: "borrow_preview",
                    npc_name: "The Borrow Checker",
                    prompt: "You are not yet ready. But try: take an immutable reference to a binding named `value` and pass it to a print macro.",
                    starter_code: "fn main() {\n    let value = String::from(\"sample\");\n    // build a reference to the binding above and print it\n}\n",
                },
            ],
        }
    }
}

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("MissionPlugin::build");
        app.init_resource::<MissionRegistry>()
            .init_resource::<ActiveMission>()
            .add_systems(
                Update,
                handle_interact_key.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_interaction_prompt.run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource, Default)]
pub struct ActiveMission {
    pub current: Option<Mission>,
}

fn handle_interact_key(
    keys: Res<ButtonInput<KeyCode>>,
    nearby: Res<NearbyNpc>,
    registry: Res<MissionRegistry>,
    mut editor: ResMut<EditorState>,
    mut active: ResMut<ActiveMission>,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Some(entry) = nearby.current.as_ref() else {
        return;
    };
    let Some(mission) = registry
        .missions
        .iter()
        .find(|m| m.id == entry.mission_id)
        .cloned()
    else {
        tracing::warn!(
            "nearby NPC {} references unknown mission {}",
            entry.name,
            entry.mission_id
        );
        return;
    };

    tracing::info!(
        "starting mission {} from {} ({})",
        mission.id,
        mission.npc_name,
        entry.name
    );

    editor.source = mission.starter_code.to_string();
    editor.last_compile_result = None;
    editor.encounter_id = mission.id.to_string();
    editor.open = true;
    active.current = Some(mission);
}

fn draw_interaction_prompt(
    mut contexts: EguiContexts,
    nearby: Res<NearbyNpc>,
    editor: Res<EditorState>,
    active: Res<ActiveMission>,
    progress: Res<MissionProgress>,
) {
    if editor.open {
        return;
    }
    let Some(entry) = nearby.current.as_ref() else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let cleared = progress.is_cleared(entry.mission_id);
    let mark = if cleared { "[done] " } else { "" };

    egui::Window::new("interact")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -24.0))
        .show(ctx, |ui| {
            ui.label(format!("{mark}[F] talk to {}", entry.name));
            if let Some(m) = active.current.as_ref().filter(|m| m.id == entry.mission_id) {
                ui.small(m.prompt);
            }
            ui.small(format!("missions cleared: {}", progress.cleared_count()));
        });
}
