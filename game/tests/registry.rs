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
        assert!(
            !m.tutorial.trim().is_empty(),
            "mission {} has empty tutorial — players need to actually be taught",
            m.id
        );
        assert!(
            m.tutorial.contains("## "),
            "mission {} tutorial has no `## Section` headers — please structure it",
            m.id
        );
    }
}

#[test]
fn every_tutorial_meets_minimum_substance() {
    let reg = MissionRegistry::default();
    for m in &reg.missions {
        // 200 chars is roughly 30-40 words — cut-off below "explains anything".
        assert!(
            m.tutorial.len() >= 200,
            "mission {} tutorial is only {} chars; tutorials should be 100-200 words to actually teach",
            m.id,
            m.tutorial.len()
        );
        // Sanity: must show at least one fenced code example.
        assert!(
            m.tutorial.contains("```"),
            "mission {} tutorial has no code-fenced syntax example",
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

#[test]
fn first_mission_has_no_prereq() {
    let reg = MissionRegistry::default();
    let first = reg.missions.first().expect("registry must have missions");
    assert!(
        first.prereq.is_none(),
        "first mission `{}` must have no prereq — players need a starting point",
        first.id
    );
}

#[test]
fn every_prereq_resolves_to_a_real_mission() {
    let reg = MissionRegistry::default();
    let known: HashSet<&str> = reg.missions.iter().map(|m| m.id).collect();
    for m in &reg.missions {
        if let Some(prev) = m.prereq {
            assert!(
                known.contains(prev),
                "mission `{}` prereq `{}` is not a known mission id",
                m.id,
                prev
            );
        }
    }
}

#[test]
fn prereq_chain_is_acyclic() {
    let reg = MissionRegistry::default();
    // Walk each mission's prereq chain. If we ever revisit an id within
    // a single chain, there's a cycle. Bound by registry size as a hard
    // safety belt in case the visited-set logic is wrong.
    let by_id: std::collections::HashMap<&str, &str> = reg
        .missions
        .iter()
        .filter_map(|m| m.prereq.map(|p| (m.id, p)))
        .collect();
    for m in &reg.missions {
        let mut visited = HashSet::new();
        let mut cur = m.id;
        for _ in 0..reg.missions.len() + 1 {
            if !visited.insert(cur) {
                panic!("prereq cycle detected starting from mission `{}`", m.id);
            }
            match by_id.get(cur) {
                Some(prev) => cur = prev,
                None => break,
            }
        }
    }
}

#[test]
fn every_mission_is_reachable_from_a_root() {
    // BFS from no-prereq missions; every mission should be hit.
    let reg = MissionRegistry::default();
    let mut reachable: HashSet<&str> = reg
        .missions
        .iter()
        .filter(|m| m.prereq.is_none())
        .map(|m| m.id)
        .collect();
    // Iterate until fixed point. With strict-linear prereqs this is
    // O(n²) worst-case, fine for ≤100 missions.
    loop {
        let before = reachable.len();
        for m in &reg.missions {
            if let Some(prev) = m.prereq
                && reachable.contains(prev)
            {
                reachable.insert(m.id);
            }
        }
        if reachable.len() == before {
            break;
        }
    }
    for m in &reg.missions {
        assert!(
            reachable.contains(m.id),
            "mission `{}` is unreachable from any root — orphaned prereq chain",
            m.id
        );
    }
}
