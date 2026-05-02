// SPDX-License-Identifier: MIT
//! Top-level app state machine. Title screen on boot; Space drops the
//! player into Hearthstone Village. Saves and pause-menu states land
//! when persistence ships.

use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Title,
    InGame,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("StatePlugin::build");
        app.init_state::<GameState>();
    }
}
