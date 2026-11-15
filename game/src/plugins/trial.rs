// SPDX-License-Identifier: MIT
//! Boss trials: one NPC, an ordered run of stage missions, one encounter.
//!
//! v1.0 has no combat (finish plan decision D1). A boss is a *trial*: a
//! short chain of ordinary missions owned by one NPC and presented as a
//! single encounter with stages.
//!
//! # Data model
//!
//! - Every stage is an ordinary [`Mission`] in the registry, so the
//!   graders, the solutions table, the server expectations and the save
//!   (cleared/opened sets keyed by mission id) need nothing new.
//! - A [`Trial`] groups N consecutive registry missions as stages 1..N and
//!   names the NPC that owns them by that NPC's roster key (its
//!   `NpcSpec::mission_id`). The roster is unchanged: one entry per NPC.
//! - An NPC owns [`owned_missions`]: its roster mission, then its trial's
//!   stages (the roster mission is not repeated when it is stage 1, which
//!   is how a trial with no prologue, such as the Act 9 Librarian, is
//!   written). The Borrow Checker keeps `borrow_preview` early in Act 2 as
//!   his prologue and holds the Bridge trial at the end of Act 2.
//!
//! # The NPC's current mission
//!
//! [`current_mission`] decides what talking to an NPC is about: the first
//! owned mission that is not cleared, if it is unlocked; otherwise (the
//! next owned mission is still locked) the last cleared owned mission, so a
//! prologue NPC behaves like any cleared villager until the trial opens;
//! otherwise the first uncleared one (locked). With everything cleared it
//! is the last owned mission, which for a trial is its final stage.
//!
//! # Dialogue status of a multi-stage owner
//!
//! The status is that of the current mission, so for a trial NPC:
//! `NotYetAvailable` while stage 1 is locked and nothing the NPC owns is
//! cleared; `Offer` when the current stage is unlocked and its offer was
//! never heard; `InProgress` when it was; `Cleared` only once every stage
//! is cleared (the current mission is then the final stage).
//!
//! # Progression
//!
//! Stages sit at a fixed point in the strict-linear registry order, so the
//! usual prereq chain already gates them: stage 1 needs the mission before
//! it, each stage needs the previous one, and the mission after the trial
//! needs the final stage. `registry_trials_are_contiguous` (unit test)
//! keeps the stages adjacent and in order.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::plugins::editor::EditorState;
use crate::plugins::mission::{ActiveMission, CompletionView, Mission, MissionRegistry};
use crate::plugins::progress::MissionProgress;
use crate::plugins::ui_theme::{self, Pip};

/// One boss trial.
#[derive(Debug)]
pub struct Trial {
    /// Stable id (logs only).
    pub id: &'static str,
    /// Display name: "Trial: {name} - stage i of N".
    pub name: &'static str,
    /// Roster key (`NpcSpec::mission_id`) of the NPC who owns the trial.
    pub npc: &'static str,
    /// Stage mission ids, in order. Consecutive in the registry.
    pub stages: &'static [&'static str],
    /// What the owner says on the trial-complete panel. Makes no claim
    /// about Rust (hard rule 3 applies to anything that does).
    pub complete_line: &'static str,
}

/// Every trial in the game.
pub const TRIALS: &[Trial] = &[Trial {
    id: "bridge",
    name: "The Bridge",
    npc: "borrow_preview",
    stages: &["bridge_toll", "bridge_quill", "bridge_lantern"],
    complete_line: "\"You may cross. The Old Tongue speaks through you.\"",
}];

/// The trial `mission_id` is a stage of, with its 0-based stage index.
pub fn trial_of(mission_id: &str) -> Option<(&'static Trial, usize)> {
    TRIALS.iter().find_map(|t| {
        t.stages
            .iter()
            .position(|s| *s == mission_id)
            .map(|i| (t, i))
    })
}

/// The trial owned by the NPC with roster key `npc`.
pub fn trial_for_npc(npc: &str) -> Option<&'static Trial> {
    TRIALS.iter().find(|t| t.npc == npc)
}

/// Mission ids the NPC with roster key `npc` owns, in play order.
pub fn owned_missions(npc: &'static str) -> Vec<&'static str> {
    let mut out = vec![npc];
    if let Some(t) = trial_for_npc(npc) {
        out.extend(t.stages.iter().copied().filter(|s| *s != npc));
    }
    out
}

/// What talking to the NPC with roster key `npc` is about right now (see
/// the module docs for the rule). `None` only if none of its missions is
/// in the registry.
pub fn current_mission<'r>(
    registry: &'r MissionRegistry,
    progress: &MissionProgress,
    npc: &'static str,
) -> Option<&'r Mission> {
    let owned: Vec<&Mission> = owned_missions(npc)
        .into_iter()
        .filter_map(|id| registry.find(id))
        .collect();
    let first_open = owned.iter().position(|m| !progress.is_cleared(m.id));
    let pick = match first_open {
        None => owned.last().copied(),
        Some(i) if owned[i].is_unlocked(progress) => Some(owned[i]),
        Some(i) => owned[..i]
            .iter()
            .rev()
            .find(|m| progress.is_cleared(m.id))
            .copied()
            .or(Some(owned[i])),
    };
    tracing::trace!(npc, current = ?pick.map(|m| m.id), "trial::current_mission");
    pick
}

/// True once every stage of `trial` is cleared.
pub fn is_complete(trial: &Trial, progress: &MissionProgress) -> bool {
    trial.stages.iter().all(|s| progress.is_cleared(s))
}

/// Pip states for `trial`, with `focus` (the stage in play or on view)
/// drawn as current.
pub fn pips(trial: &Trial, progress: &MissionProgress, focus: &str) -> Vec<Pip> {
    trial
        .stages
        .iter()
        .map(|s| {
            if *s == focus {
                Pip::Current
            } else if progress.is_cleared(s) {
                Pip::Cleared
            } else {
                Pip::Locked
            }
        })
        .collect()
}

/// "Trial: The Bridge - stage 2 of 3", for stage index `index`.
pub fn stage_label(trial: &Trial, index: usize) -> String {
    format!(
        "Trial: {} - stage {} of {}",
        trial.name,
        index + 1,
        trial.stages.len()
    )
}

/// A trial header ready to draw: label plus pips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageStrip {
    pub label: String,
    pub pips: Vec<Pip>,
}

impl StageStrip {
    /// The strip for `mission_id` if it is a trial stage.
    pub fn for_mission(mission_id: &str, progress: &MissionProgress) -> Option<Self> {
        let (trial, index) = trial_of(mission_id)?;
        Some(Self {
            label: stage_label(trial, index),
            pips: pips(trial, progress, mission_id),
        })
    }

    /// Draw the label and the pip row on one line.
    pub fn show(&self, ui: &mut egui::Ui, size: f32) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&self.label)
                    .size(size)
                    .color(ui_theme::TEXT_HEADING)
                    .strong(),
            );
            ui.add_space(6.0);
            ui_theme::stage_pips(ui, &self.pips, size);
        });
    }
}

/// Trial bookkeeping between frames.
#[derive(Resource, Default, Debug)]
pub struct TrialView {
    /// The trial stage currently in the editor and whether it was already
    /// cleared when it was opened.
    armed: Option<(&'static str, bool)>,
    /// A stage cleared during this visit to the editor; its panel shows
    /// once the editor closes.
    pub just_cleared: Option<&'static str>,
}

/// Digit keys that pick a stage on a trial panel (stage 1 = `1`).
const STAGE_KEYS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

/// Notice a trial stage being cleared in the editor; when the editor then
/// closes, open the completion panel on it (the "stage cleared" panel, or
/// the trial-complete panel after the final stage).
pub fn watch_stage_clears(
    editor: Res<EditorState>,
    active: Res<ActiveMission>,
    progress: Res<MissionProgress>,
    mut view: ResMut<TrialView>,
    mut completion: ResMut<CompletionView>,
) {
    if let Some(m) = active.current.as_ref()
        && let Some((trial, index)) = trial_of(m.id)
    {
        let stage = trial.stages[index];
        let cleared = progress.is_cleared(stage);
        match view.armed {
            Some((id, _)) if id == stage => {}
            _ => {
                tracing::debug!(trial = trial.id, stage, index, cleared, "trial stage armed");
                view.armed = Some((stage, cleared));
            }
        }
        if view.armed == Some((stage, false)) && cleared {
            view.armed = Some((stage, true));
            view.just_cleared = Some(stage);
            tracing::info!(
                trial = trial.id,
                stage,
                stage_no = index + 1,
                of = trial.stages.len(),
                complete = is_complete(trial, &progress),
                "trial stage cleared"
            );
        }
    } else if view.armed.is_some() {
        view.armed = None;
    }
    if !editor.open
        && completion.mission_id.is_none()
        && let Some(stage) = view.just_cleared.take()
    {
        tracing::info!(
            stage,
            "editor closed after a stage clear: showing trial panel"
        );
        completion.mission_id = Some(stage.to_string());
    }
}

/// Keys on a trial's completion panel: `N` opens the next stage straight
/// in the editor (no walk back to the NPC), digits switch the panel to a
/// cleared stage so any stage can be re-read.
#[allow(clippy::too_many_arguments)]
pub fn trial_panel_keys(
    keys: Res<ButtonInput<KeyCode>>,
    registry: Res<MissionRegistry>,
    mut progress: ResMut<MissionProgress>,
    mut completion: ResMut<CompletionView>,
    mut editor: ResMut<EditorState>,
    mut active: ResMut<ActiveMission>,
    mut view: ResMut<TrialView>,
) {
    if editor.open {
        return;
    }
    let Some((trial, index)) = completion.mission_id.as_deref().and_then(trial_of) else {
        return;
    };
    if keys.just_pressed(KeyCode::KeyN) {
        let next = trial
            .stages
            .get(index + 1)
            .and_then(|id| registry.find(id))
            .filter(|m| m.is_unlocked(&progress));
        match next {
            Some(m) => {
                tracing::info!(
                    trial = trial.id,
                    from = trial.stages[index],
                    to = m.id,
                    "trial: next stage from the completion panel"
                );
                completion.mission_id = None;
                view.just_cleared = None;
                // Started without its offer dialogue: count it as heard,
                // so the next talk is the in-progress nudge.
                if !progress.is_opened(m.id) {
                    progress.mark_opened(m.id);
                }
                crate::plugins::mission::start_mission(&mut editor, &mut active, m.clone());
            }
            None => tracing::debug!(
                trial = trial.id,
                stage = trial.stages[index],
                "trial: N pressed with no playable next stage"
            ),
        }
        return;
    }
    for (k, key) in STAGE_KEYS.iter().enumerate().take(trial.stages.len()) {
        if !keys.just_pressed(*key) {
            continue;
        }
        let stage = trial.stages[k];
        if progress.is_cleared(stage) {
            tracing::debug!(trial = trial.id, stage, "trial panel: stage selected");
            completion.mission_id = Some(stage.to_string());
            active.current = registry.find(stage).cloned();
        } else {
            tracing::debug!(
                trial = trial.id,
                stage,
                "trial panel: stage not cleared yet"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_trials_are_contiguous() {
        let reg = MissionRegistry::default();
        for t in TRIALS {
            let first = reg
                .missions
                .iter()
                .position(|m| m.id == t.stages[0])
                .unwrap_or_else(|| panic!("{}: stage 1 not in registry", t.id));
            for (i, s) in t.stages.iter().enumerate() {
                let m = &reg.missions[first + i];
                assert_eq!(m.id, *s, "{}: stages must be consecutive", t.id);
            }
            assert!(t.stages.len() >= 2, "{}: a trial has stages", t.id);
            assert!(
                crate::plugins::npc::NPC_ROSTER
                    .iter()
                    .any(|n| n.mission_id == t.npc),
                "{}: owner is not a roster NPC",
                t.id
            );
            let owner_name = crate::plugins::npc::NPC_ROSTER
                .iter()
                .find(|n| n.mission_id == t.npc)
                .map(|n| n.name);
            for s in t.stages {
                assert_eq!(
                    reg.find(s).map(|m| m.npc_name),
                    owner_name,
                    "{s}: stage speaker must be the owner"
                );
            }
        }
    }

    #[test]
    fn owned_missions_lists_prologue_then_stages() {
        assert_eq!(
            owned_missions("borrow_preview"),
            vec![
                "borrow_preview",
                "bridge_toll",
                "bridge_quill",
                "bridge_lantern"
            ]
        );
        assert_eq!(owned_missions("mut_binding"), vec!["mut_binding"]);
    }

    fn cleared_through(reg: &MissionRegistry, last: &str) -> MissionProgress {
        let mut p = MissionProgress::default();
        for m in &reg.missions {
            p.mark_cleared(m.id);
            if m.id == last {
                break;
            }
        }
        p
    }

    #[test]
    fn current_mission_walks_prologue_window_and_stages() {
        let reg = MissionRegistry::default();
        let cur = |p: &MissionProgress| current_mission(&reg, p, "borrow_preview").map(|m| m.id);
        // Fresh: the prologue (locked until double_function is cleared).
        assert_eq!(cur(&MissionProgress::default()), Some("borrow_preview"));
        // Prologue cleared, trial still locked: behaves as a cleared NPC.
        let p = cleared_through(&reg, "borrow_preview");
        assert_eq!(cur(&p), Some("borrow_preview"));
        let p = cleared_through(&reg, "enum_match");
        assert_eq!(cur(&p), Some("borrow_preview"));
        // The mission before the trial cleared: stage 1 is offered.
        let p = cleared_through(&reg, "while_loop");
        assert_eq!(cur(&p), Some("bridge_toll"));
        let p = cleared_through(&reg, "bridge_toll");
        assert_eq!(cur(&p), Some("bridge_quill"));
        let p = cleared_through(&reg, "bridge_quill");
        assert_eq!(cur(&p), Some("bridge_lantern"));
        // All cleared: the final stage (the trial-complete panel).
        let p = cleared_through(&reg, "bridge_lantern");
        assert_eq!(cur(&p), Some("bridge_lantern"));
        // A plain NPC is always its own mission.
        assert_eq!(
            current_mission(&reg, &p, "mut_binding").map(|m| m.id),
            Some("mut_binding")
        );
    }

    #[test]
    fn pips_mark_cleared_current_and_locked() {
        let reg = MissionRegistry::default();
        let p = cleared_through(&reg, "bridge_toll");
        let t = &TRIALS[0];
        assert_eq!(
            pips(t, &p, "bridge_quill"),
            vec![Pip::Cleared, Pip::Current, Pip::Locked]
        );
        assert_eq!(stage_label(t, 1), "Trial: The Bridge - stage 2 of 3");
        assert!(!is_complete(t, &p));
        let p = cleared_through(&reg, "bridge_lantern");
        assert!(is_complete(t, &p));
        assert_eq!(
            pips(t, &p, "bridge_toll"),
            vec![Pip::Current, Pip::Cleared, Pip::Cleared]
        );
    }

    #[test]
    fn trial_of_and_strip() {
        assert_eq!(
            trial_of("bridge_quill").map(|(t, i)| (t.id, i)),
            Some(("bridge", 1))
        );
        assert!(trial_of("borrow_preview").is_none());
        assert!(StageStrip::for_mission("mut_binding", &MissionProgress::default()).is_none());
        let s = StageStrip::for_mission("bridge_toll", &MissionProgress::default())
            .expect("stage 1 strip");
        assert_eq!(s.label, "Trial: The Bridge - stage 1 of 3");
        assert_eq!(s.pips, vec![Pip::Current, Pip::Locked, Pip::Locked]);
    }
}
