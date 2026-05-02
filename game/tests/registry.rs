// SPDX-License-Identifier: MIT
//! Audit: invariants on the mission registry and NPC roster.

use pledge_and_crown::assets::ALL_SPRITE_PATHS;
use pledge_and_crown::plugins::mission::MissionRegistry;
use pledge_and_crown::plugins::npc::NPC_ROSTER;
use std::collections::HashSet;

#[test]
fn mission_ids_are_unique() {
    let reg = MissionRegistry::default();
    let mut seen = HashSet::new();
    for m in &reg.missions {
        assert!(seen.insert(m.id), "duplicate mission id: {}", m.id);
    }
}

#[test]
fn every_mission_has_nonempty_metadata() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        assert!(!m.id.is_empty(), "mission with empty id");
        assert!(
            !m.npc_name.is_empty(),
            "mission {} has empty npc_name",
            m.id
        );
        assert!(!m.prompt.is_empty(), "mission {} has empty prompt", m.id);
        assert!(
            !m.starter_code.trim().is_empty(),
            "mission {} has empty starter_code",
            m.id
        );
    }
}

#[test]
fn every_npc_mission_id_resolves() {
    let reg = MissionRegistry::default();
    let known: HashSet<&str> = reg.missions.iter().map(|m| m.id).collect();
    for npc in NPC_ROSTER {
        assert!(
            known.contains(npc.mission_id),
            "NPC `{}` references unknown mission id `{}`",
            npc.name,
            npc.mission_id
        );
    }
}

#[test]
fn every_npc_sprite_is_in_the_asset_registry() {
    let registered: HashSet<&str> = ALL_SPRITE_PATHS.iter().copied().collect();
    for npc in NPC_ROSTER {
        assert!(
            registered.contains(npc.sprite_path),
            "NPC `{}` uses sprite `{}` that isn't in ALL_SPRITE_PATHS — won't be audited for existence",
            npc.name,
            npc.sprite_path
        );
    }
}

#[test]
fn npc_names_are_unique() {
    let mut seen = HashSet::new();
    for npc in NPC_ROSTER {
        assert!(seen.insert(npc.name), "duplicate NPC name: {}", npc.name);
    }
}
