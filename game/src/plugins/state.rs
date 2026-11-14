// SPDX-License-Identifier: MIT
//! Top-level app state machine.
//!
//! ```text
//!   Title ──Continue/New Game──▶ InGame ┬─ PauseState::Running ◀─Esc─┐
//!     ▲                                 └─ PauseState::Paused  ──Esc─┘
//!     └───────────── Quit to Title (from the pause menu) ──────────┘
//! ```
//!
//! `Paused` is a [`SubStates`] of `GameState::InGame` rather than a third
//! `GameState` variant on purpose: the world, NPCs and player all spawn in
//! `OnEnter(GameState::InGame)`, so a sibling `Paused` state would re-run
//! those spawns on every resume (and fire `OnExit(InGame)`, which stops the
//! music). As a sub-state, pausing leaves `GameState::InGame` untouched —
//! entities stay put and music keeps playing — while `in_state(
//! PauseState::Paused)` / `in_state(PauseState::Running)` are available as
//! run conditions.
//!
//! The Esc rules live in [`esc_action`], a pure function, so they are unit
//! tested without a Bevy `App`.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{TileStorage, TilemapId};

use crate::plugins::npc::Npc;
use crate::plugins::player::Player;
use crate::plugins::prop::Prop;

#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Title,
    InGame,
}

/// Pause layer. Only exists while `GameState::InGame` is active; Bevy
/// removes it (firing `OnExit(PauseState::Paused)` if needed) when the
/// game returns to the title screen.
#[derive(SubStates, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[source(GameState = GameState::InGame)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

/// Everything the Esc rule needs to know, gathered by the input system
/// from live resources. Plain data so tests can build it by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EscContext {
    pub state: GameState,
    pub paused: bool,
    /// The code editor owns Esc while it is open (Esc closes it).
    pub editor_open: bool,
    /// An in-world overlay that Esc dismisses (mission recap, epilogue).
    pub overlay_open: bool,
    /// A sub-screen of a menu is showing (Settings, New Game confirm).
    pub submenu_open: bool,
}

/// What a single Esc press should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscAction {
    /// Someone else owns this Esc (editor, overlay) or there is nothing
    /// to back out of.
    Ignore,
    Pause,
    Resume,
    /// Back out of a menu sub-screen to its parent menu.
    CloseSubmenu,
}

/// The Esc state-transition rules.
///
/// - Title: Esc backs out of a sub-screen; otherwise does nothing.
/// - In game, running: Esc pauses, unless the editor or an Esc-dismissable
///   overlay is open (those consume the press themselves).
/// - In game, paused: Esc backs out of a sub-screen, else resumes.
pub fn esc_action(ctx: EscContext) -> EscAction {
    match (ctx.state, ctx.paused) {
        (GameState::Title, _) => {
            if ctx.submenu_open {
                EscAction::CloseSubmenu
            } else {
                EscAction::Ignore
            }
        }
        (GameState::InGame, false) => {
            if ctx.editor_open || ctx.overlay_open {
                EscAction::Ignore
            } else {
                EscAction::Pause
            }
        }
        (GameState::InGame, true) => {
            if ctx.submenu_open {
                EscAction::CloseSubmenu
            } else {
                EscAction::Resume
            }
        }
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("StatePlugin::build");
        app.init_state::<GameState>()
            .add_sub_state::<PauseState>()
            .add_systems(OnEnter(PauseState::Paused), freeze_virtual_time)
            .add_systems(OnExit(PauseState::Paused), thaw_virtual_time)
            .add_systems(OnExit(GameState::InGame), despawn_session_entities);
    }
}

/// Stop `Time<Virtual>` so every `Res<Time>`-driven animation (camera
/// lerp, quest-marker bob, movement) holds still while paused. Audio runs
/// on kira's own clock and egui on `Time<Real>`, so neither is affected.
fn freeze_virtual_time(mut time: ResMut<Time<Virtual>>) {
    tracing::info!("paused: freezing virtual time");
    time.pause();
}

fn thaw_virtual_time(mut time: ResMut<Time<Virtual>>) {
    tracing::info!("unpaused: resuming virtual time");
    time.unpause();
}

/// Leaving `InGame` (Quit to Title) must tear down what the gameplay
/// plugins spawned in `OnEnter(GameState::InGame)`; otherwise the next
/// Continue / New Game would spawn a second village, cast and player on
/// top of the first. NPC children (quest markers) go with their parent
/// because `despawn` is recursive.
///
/// TODO(world/npc owners): tagging spawns with `DespawnOnExit(GameState::
/// InGame)` would make this sweep unnecessary; until then this list must
/// cover every root entity spawned on `OnEnter(GameState::InGame)`.
#[allow(clippy::type_complexity)]
fn despawn_session_entities(
    mut commands: Commands,
    q: Query<
        Entity,
        Or<(
            With<Player>,
            With<Npc>,
            With<Prop>,
            With<TileStorage>,
            With<TilemapId>,
        )>,
    >,
) {
    let mut n = 0usize;
    for e in q.iter() {
        commands.entity(e).try_despawn();
        n += 1;
    }
    tracing::info!(despawned = n, "left InGame: swept session entities");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(state: GameState, paused: bool) -> EscContext {
        EscContext {
            state,
            paused,
            editor_open: false,
            overlay_open: false,
            submenu_open: false,
        }
    }

    #[test]
    fn running_esc_pauses() {
        assert_eq!(esc_action(ctx(GameState::InGame, false)), EscAction::Pause);
    }

    #[test]
    fn paused_esc_resumes() {
        assert_eq!(esc_action(ctx(GameState::InGame, true)), EscAction::Resume);
    }
}
