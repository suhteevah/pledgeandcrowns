// SPDX-License-Identifier: MIT
//! Tracks which missions the player has cleared. Used to gate NPC
//! dialogue, change interaction prompts, and (later) unlock new
//! zones. Persistence is a follow-up; for now this lives in-memory
//! and resets on app restart.

use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Default, Debug)]
pub struct MissionProgress {
    cleared: HashSet<String>,
}

impl MissionProgress {
    pub fn mark_cleared(&mut self, encounter_id: &str) {
        if self.cleared.insert(encounter_id.to_string()) {
            tracing::info!("mission cleared: {encounter_id}");
        }
    }
    pub fn is_cleared(&self, encounter_id: &str) -> bool {
        self.cleared.contains(encounter_id)
    }
    pub fn cleared_count(&self) -> usize {
        self.cleared.len()
    }
}

pub struct ProgressPlugin;

impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("ProgressPlugin::build");
        app.init_resource::<MissionProgress>();
    }
}
