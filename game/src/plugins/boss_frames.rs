// SPDX-License-Identifier: MIT
//! Boss intro and defeat frames: a boss with drawn frames swaps its idle
//! sprite for an intro frame when its trial begins and for a defeat frame
//! when its trial is passed, then goes back to idle.
//!
//! # When
//!
//! - **Intro** ("the moment the trial begins"): a conversation opens with the
//!   boss about one of his trial's stages that is open and not cleared (the
//!   dialogue status is `Offer` or `InProgress` and the conversation carries
//!   a trial header). The frame holds while that conversation is open and
//!   for [`INTRO_SECS`] after it closes. The conversation is the last moment
//!   the boss is on screen before the editor covers the world, so the frame
//!   reads there; when it ends in the editor, the tail runs out behind it.
//! - **Defeat** ("the moment the player's code passes the trial"): the
//!   boss's trial goes from not complete to complete (every stage cleared)
//!   while in game. That happens in the editor, which covers the world, and
//!   the trial-complete panel (and, after the Compiler, the epilogue) then
//!   sits over the boss, so the [`DEFEAT_SECS`] beat only counts while none
//!   of those is up: the player closes them and sees him kneel for 4 s.
//!   After that he stands idle again, so a cleared boss in the world is not
//!   kneeling forever. A boss whose trial is already complete when the
//!   world spawns (a loaded save) starts idle and never kneels.
//!
//! Both beats count `Time<Virtual>`, which the pause menu freezes.
//!
//! # How
//!
//! [`BOSS_FRAMES`] maps a boss's roster key to its three sprite paths. At
//! spawn, [`attach_boss_frames`] gives each NPC in the table a [`BossFrames`]
//! component (other NPCs, the six bosses without frames among them, get
//! nothing and are never touched). [`drive_boss_frames`] advances the pose
//! ([`BossPose`], a pure state machine) and swaps `Sprite::image` only when
//! the frame changes. The frames share the idle sprite's canvas, size and
//! anchor, so nothing else changes: the idle bob (`SpriteLift`, via the
//! anchor), y-sort and the prop fade read the transform and anchor, the
//! quest marker is a child placed from `custom_size`.

use bevy::prelude::*;

use crate::assets::{
    SPRITE_BORROW_CHECKER, SPRITE_BORROW_CHECKER_DEFEAT, SPRITE_BORROW_CHECKER_INTRO,
    SPRITE_COMPILER, SPRITE_COMPILER_DEFEAT, SPRITE_COMPILER_INTRO, SPRITE_LIBRARIAN,
    SPRITE_LIBRARIAN_DEFEAT, SPRITE_LIBRARIAN_INTRO,
};
use crate::plugins::dialogue::{Dialogue, MissionStatus};
use crate::plugins::editor::EditorState;
use crate::plugins::mission::{CompletionView, EpilogueView, MissionRegistry, epilogue_due};
use crate::plugins::npc::Npc;
use crate::plugins::progress::MissionProgress;
use crate::plugins::state::GameState;
use crate::plugins::trial;

/// Seconds the intro frame holds after the trial conversation closes.
pub const INTRO_SECS: f32 = 1.2;
/// Seconds the defeat frame holds once nothing covers the world.
pub const DEFEAT_SECS: f32 = 4.0;

/// One boss's frames. `npc` is the roster key (`NpcSpec::mission_id`).
#[derive(Debug, PartialEq, Eq)]
pub struct BossFrameSet {
    pub npc: &'static str,
    pub idle: &'static str,
    pub intro: &'static str,
    pub defeat: &'static str,
}

impl BossFrameSet {
    /// The sprite path that shows `frame`.
    pub fn path(&self, frame: BossFrame) -> &'static str {
        match frame {
            BossFrame::Idle => self.idle,
            BossFrame::Intro => self.intro,
            BossFrame::Defeat => self.defeat,
        }
    }
}

/// Every boss with drawn intro and defeat frames (approved 2026-11-19).
pub const BOSS_FRAMES: &[BossFrameSet] = &[
    // REF-03 idle, REF-180 intro, REF-181 defeat.
    BossFrameSet {
        npc: "borrow_preview",
        idle: SPRITE_BORROW_CHECKER,
        intro: SPRITE_BORROW_CHECKER_INTRO,
        defeat: SPRITE_BORROW_CHECKER_DEFEAT,
    },
    // REF-93 idle, REF-182 intro, REF-183 defeat.
    BossFrameSet {
        npc: "librarian_index",
        idle: SPRITE_LIBRARIAN,
        intro: SPRITE_LIBRARIAN_INTRO,
        defeat: SPRITE_LIBRARIAN_DEFEAT,
    },
    // REF-94 idle, REF-184 intro, REF-185 defeat.
    BossFrameSet {
        npc: "compiler_alloc",
        idle: SPRITE_COMPILER,
        intro: SPRITE_COMPILER_INTRO,
        defeat: SPRITE_COMPILER_DEFEAT,
    },
];

/// The frames of the NPC with roster key `npc`, if it has any.
pub fn frames_for(npc: &str) -> Option<&'static BossFrameSet> {
    BOSS_FRAMES.iter().find(|f| f.npc == npc)
}

/// Which of the three pictures is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossFrame {
    Idle,
    Intro,
    Defeat,
}

/// What keeps a beat from counting down this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hold {
    /// A conversation with this boss about an open trial stage is up.
    pub trial_dialogue: bool,
    /// The editor, a completion panel or the epilogue covers the world.
    pub world_covered: bool,
}

/// A boss's pose and the time left on its beat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BossPose {
    Idle,
    Intro { left: f32 },
    Defeat { left: f32 },
}

impl BossPose {
    pub fn frame(self) -> BossFrame {
        match self {
            BossPose::Idle => BossFrame::Idle,
            BossPose::Intro { .. } => BossFrame::Intro,
            BossPose::Defeat { .. } => BossFrame::Defeat,
        }
    }

    /// The trial begins: an idle boss lights up. A kneeling boss keeps
    /// kneeling (his trial is complete, so this cannot normally happen).
    pub fn trial_begins(self) -> Self {
        match self {
            BossPose::Idle => BossPose::Intro { left: INTRO_SECS },
            other => other,
        }
    }

    /// The trial is passed: kneel, whatever the pose.
    pub fn trial_passed(self) -> Self {
        BossPose::Defeat { left: DEFEAT_SECS }
    }

    /// Advance the beat by `dt` seconds unless `hold` keeps it; back to
    /// idle when it runs out. The intro holds while its conversation is
    /// open, the defeat while the world is covered.
    pub fn tick(self, dt: f32, hold: Hold) -> Self {
        let dt = dt.max(0.0);
        match self {
            BossPose::Idle => BossPose::Idle,
            BossPose::Intro { left } if hold.trial_dialogue => BossPose::Intro { left },
            BossPose::Defeat { left } if hold.world_covered => BossPose::Defeat { left },
            BossPose::Intro { left } if left > dt => BossPose::Intro { left: left - dt },
            BossPose::Defeat { left } if left > dt => BossPose::Defeat { left: left - dt },
            BossPose::Intro { .. } | BossPose::Defeat { .. } => BossPose::Idle,
        }
    }
}

/// Frame state of a boss with drawn frames.
#[derive(Component, Debug)]
pub struct BossFrames {
    pub set: &'static BossFrameSet,
    pub pose: BossPose,
    /// The frame the sprite shows now.
    pub shown: BossFrame,
    /// Whether the boss's trial was complete last frame.
    pub was_complete: bool,
    /// Idle, intro, defeat handles (none headless).
    handles: Option<[Handle<Image>; 3]>,
}

impl BossFrames {
    /// A boss idle, `complete` saying whether his trial is already done
    /// (a loaded save): that never shows the defeat frame.
    pub fn new(set: &'static BossFrameSet, complete: bool) -> Self {
        Self {
            set,
            pose: BossPose::Idle,
            shown: BossFrame::Idle,
            was_complete: complete,
            handles: None,
        }
    }

    /// The sprite path for the frame showing now.
    pub fn shown_path(&self) -> &'static str {
        self.set.path(self.shown)
    }

    /// One frame of the state machine: `complete` is whether the trial is
    /// complete now. Returns the new frame when it changed.
    pub fn step(&mut self, complete: bool, dt: f32, hold: Hold) -> Option<(BossFrame, BossFrame)> {
        if complete && !self.was_complete {
            self.pose = self.pose.trial_passed();
        } else if hold.trial_dialogue {
            self.pose = self.pose.trial_begins();
        }
        self.was_complete = complete;
        self.pose = self.pose.tick(dt, hold);
        let now = self.pose.frame();
        (now != self.shown).then(|| {
            let from = self.shown;
            self.shown = now;
            (from, now)
        })
    }
}

pub struct BossFramesPlugin;

impl Plugin for BossFramesPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("BossFramesPlugin::build");
        app.add_systems(
            Update,
            (attach_boss_frames, drive_boss_frames)
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
    }
}

/// Give each newly spawned NPC that has frames its [`BossFrames`].
pub fn attach_boss_frames(
    mut commands: Commands,
    npcs: Query<(Entity, &Npc), Added<Npc>>,
    progress: Option<Res<MissionProgress>>,
    asset_server: Option<Res<AssetServer>>,
) {
    for (entity, npc) in &npcs {
        let Some(set) = frames_for(npc.mission_id) else {
            continue;
        };
        let complete = match (trial::trial_for_npc(set.npc), progress.as_deref()) {
            (Some(t), Some(p)) => trial::is_complete(t, p),
            _ => false,
        };
        let mut frames = BossFrames::new(set, complete);
        frames.handles = asset_server
            .as_deref()
            .map(|s| [s.load(set.idle), s.load(set.intro), s.load(set.defeat)]);
        tracing::debug!(
            boss = set.npc,
            trial_complete = complete,
            "boss frames attached (idle)"
        );
        commands.entity(entity).insert(frames);
    }
}

/// Advance every boss's pose and swap its sprite when the frame changes.
#[allow(clippy::too_many_arguments)]
pub fn drive_boss_frames(
    time: Res<Time>,
    progress: Option<Res<MissionProgress>>,
    dialogue: Option<Res<Dialogue>>,
    editor: Option<Res<EditorState>>,
    completion: Option<Res<CompletionView>>,
    epilogue: Option<Res<EpilogueView>>,
    registry: Option<Res<MissionRegistry>>,
    mut bosses: Query<(&mut BossFrames, &mut Sprite)>,
) {
    let epilogue_up = match (progress.as_deref(), registry.as_deref(), epilogue) {
        (Some(p), Some(r), Some(e)) => !e.dismissed_this_session && epilogue_due(p, r),
        _ => false,
    };
    let world_covered = editor.is_some_and(|e| e.open)
        || completion.is_some_and(|c| c.mission_id.is_some())
        || epilogue_up;
    // The boss a trial conversation is with, if one is up.
    let talking_trial = dialogue
        .as_deref()
        .and_then(|d| d.active.as_ref())
        .and_then(|a| {
            (a.stage.is_some()
                && matches!(a.status, MissionStatus::Offer | MissionStatus::InProgress))
            .then_some(a.npc_id)
        });
    let dt = time.delta_secs();
    for (mut frames, mut sprite) in &mut bosses {
        let complete = match (trial::trial_for_npc(frames.set.npc), progress.as_deref()) {
            (Some(t), Some(p)) => trial::is_complete(t, p),
            _ => false,
        };
        let hold = Hold {
            trial_dialogue: talking_trial == Some(frames.set.npc),
            world_covered,
        };
        let Some((from, to)) = frames.step(complete, dt, hold) else {
            continue;
        };
        tracing::debug!(boss = frames.set.npc, ?from, ?to, "boss frame switch");
        if let Some(handles) = frames.handles.as_ref() {
            let i = match to {
                BossFrame::Idle => 0,
                BossFrame::Intro => 1,
                BossFrame::Defeat => 2,
            };
            sprite.image = handles[i].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exact in binary, so the step counts are exact.
    const DT: f32 = 0.125;
    const FREE: Hold = Hold {
        trial_dialogue: false,
        world_covered: false,
    };
    const TALKING: Hold = Hold {
        trial_dialogue: true,
        world_covered: false,
    };
    const COVERED: Hold = Hold {
        trial_dialogue: false,
        world_covered: true,
    };

    fn checker() -> BossFrames {
        BossFrames::new(frames_for("borrow_preview").expect("checker frames"), false)
    }

    /// Steps until the frame changes or `max` steps pass; the step count.
    fn steps_until_change(f: &mut BossFrames, complete: bool, hold: Hold, max: usize) -> usize {
        (1..=max)
            .find(|_| f.step(complete, DT, hold).is_some())
            .unwrap_or(max + 1)
    }

    #[test]
    fn idle_intro_idle_on_the_timer() {
        let mut f = checker();
        assert_eq!(f.shown, BossFrame::Idle);
        assert_eq!(
            f.step(false, DT, TALKING),
            Some((BossFrame::Idle, BossFrame::Intro))
        );
        assert_eq!(f.shown_path(), SPRITE_BORROW_CHECKER_INTRO);
        // Held for as long as the trial conversation is open.
        for _ in 0..100 {
            assert_eq!(f.step(false, DT, TALKING), None);
        }
        // Then INTRO_SECS (10 steps of 0.125 s) and back to idle.
        let n = steps_until_change(&mut f, false, FREE, 100);
        assert_eq!(n, 10, "intro tail is {INTRO_SECS} s");
        assert_eq!(f.shown, BossFrame::Idle);
        assert_eq!(f.shown_path(), SPRITE_BORROW_CHECKER);
        // The editor covering the world does not hold the intro.
        f.step(false, DT, TALKING);
        assert_eq!(steps_until_change(&mut f, false, COVERED, 100), 10);
    }

    #[test]
    fn pass_defeat_idle_on_the_timer() {
        let mut f = checker();
        f.step(false, DT, TALKING);
        // Passed in the editor: kneels at once, held while covered.
        assert_eq!(
            f.step(true, DT, COVERED),
            Some((BossFrame::Intro, BossFrame::Defeat))
        );
        for _ in 0..100 {
            assert_eq!(f.step(true, DT, COVERED), None);
        }
        let n = steps_until_change(&mut f, true, FREE, 100);
        assert_eq!(n, 32, "defeat beat is {DEFEAT_SECS} s once uncovered");
        assert_eq!(f.shown, BossFrame::Idle);
        // A cleared boss stays idle, and talking to him never re-triggers.
        for _ in 0..100 {
            assert_eq!(f.step(true, DT, FREE), None);
        }
    }

    #[test]
    fn a_boss_loaded_as_cleared_starts_idle() {
        let set = frames_for("librarian_index").expect("librarian frames");
        let mut f = BossFrames::new(set, true);
        assert_eq!(f.shown, BossFrame::Idle);
        for _ in 0..100 {
            assert_eq!(f.step(true, DT, FREE), None);
        }
        assert_eq!(f.shown_path(), SPRITE_LIBRARIAN);
    }

    #[test]
    fn frozen_time_freezes_the_beats() {
        let mut f = checker();
        f.step(true, DT, FREE);
        assert_eq!(f.shown, BossFrame::Defeat);
        let before = f.pose;
        assert_eq!(
            before,
            BossPose::Defeat {
                left: DEFEAT_SECS - DT
            }
        );
        // Paused: Time<Virtual> delta is zero.
        for _ in 0..1000 {
            assert_eq!(f.step(true, 0.0, FREE), None);
        }
        assert_eq!(f.pose, before);
    }

    #[test]
    fn the_table_is_the_three_bosses_with_drawn_frames() {
        let keys: Vec<&str> = BOSS_FRAMES.iter().map(|f| f.npc).collect();
        assert_eq!(
            keys,
            ["borrow_preview", "librarian_index", "compiler_alloc"]
        );
        for f in BOSS_FRAMES {
            let spec = crate::plugins::npc::NPC_ROSTER
                .iter()
                .find(|s| s.mission_id == f.npc)
                .unwrap_or_else(|| panic!("{} not in the roster", f.npc));
            assert_eq!(
                spec.sprite_path, f.idle,
                "{}: idle is the roster sprite",
                f.npc
            );
            assert_eq!(spec.native_px, 64.0, "{}", f.npc);
            assert!(
                trial::trial_for_npc(f.npc).is_some(),
                "{} owns a trial",
                f.npc
            );
        }
        // The other six trial owners have no frames.
        let without: Vec<&str> = trial::TRIALS
            .iter()
            .map(|t| t.npc)
            .filter(|n| frames_for(n).is_none())
            .collect();
        assert_eq!(without.len(), 6, "{without:?}");
    }
}
