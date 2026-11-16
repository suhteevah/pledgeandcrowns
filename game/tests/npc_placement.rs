// SPDX-License-Identifier: MIT
//! Audit: where every roster NPC stands, and the placement invariants.
//!
//! Pure geometry over the same functions the game spawns from
//! (`world::district_slot`, `world::ground_at`, `prop::all_props`): no
//! `App`, no plugins, nothing touches the save.
//!
//! `npc_ground_audit` prints one row per NPC (run with `--nocapture` to read
//! it); `npc_placement_invariants` is the permanent gate.

use bevy::prelude::*;
use pledge_and_crown::plugins::npc::{INTERACT_RADIUS_PX, NPC_ROSTER, TALK_SPOT_OFFSET};
use pledge_and_crown::plugins::player::PLAYER_FOOT;
use pledge_and_crown::plugins::prop::{Aabb, all_props};
use pledge_and_crown::plugins::world::{
    DISTRICTS, Ground, PLAYER_SPAWN, WORLD_HALF_H, WORLD_HALF_W, district_slot, ground_at,
};

const TILE: f32 = 16.0;
/// The player sprite is 32 px, centre-anchored.
const PLAYER_PX: f32 = 32.0;

fn square(center: Vec2, size: f32) -> Aabb {
    Aabb {
        min: center - Vec2::splat(size * 0.5),
        max: center + Vec2::splat(size * 0.5),
    }
}

/// Free space between two boxes along the axis where they are furthest
/// apart (negative when they overlap).
fn gap(a: &Aabb, b: &Aabb) -> f32 {
    let dx = (b.min.x - a.max.x).max(a.min.x - b.max.x);
    let dy = (b.min.y - a.max.y).max(a.min.y - b.max.y);
    dx.max(dy)
}

fn inside_map(r: &Aabb) -> bool {
    r.min.x >= -WORLD_HALF_W
        && r.max.x <= WORLD_HALF_W
        && r.min.y >= -WORLD_HALF_H
        && r.max.y <= WORLD_HALF_H
}

/// The NPC's sprite rectangle (centre-anchored, `native_px` square).
fn sprite_rect(i: usize) -> Aabb {
    let half = NPC_ROSTER[i].native_px * 0.5;
    let p = district_slot(i);
    Aabb {
        min: p - Vec2::splat(half),
        max: p + Vec2::splat(half),
    }
}

fn grow(r: Aabb, by: f32) -> Aabb {
    Aabb {
        min: r.min - Vec2::splat(by),
        max: r.max + Vec2::splat(by),
    }
}

/// Every ground kind under `r`, sampled at the centres of 4 px cells (so a
/// rect edge lying exactly on a tile boundary does not pick up the next tile).
fn kinds_under(r: Aabb) -> Vec<Ground> {
    let mut out: Vec<Ground> = Vec::new();
    let mut y = r.min.y + 2.0;
    while y < r.max.y {
        let mut x = r.min.x + 2.0;
        while x < r.max.x {
            let g = ground_at(Vec2::new(x, y));
            if !out.contains(&g) {
                out.push(g);
            }
            x += 4.0;
        }
        y += 4.0;
    }
    out
}

/// Index into `DISTRICTS` of the district that owns roster slot `i`.
fn district_of(i: usize) -> Option<usize> {
    let mut acc = 0;
    for (d, district) in DISTRICTS.iter().enumerate() {
        acc += district.count;
        if i < acc {
            return Some(d);
        }
    }
    None
}

/// One-word ground class for the audit table.
fn classify(i: usize) -> String {
    let rect = sprite_rect(i);
    if all_props()
        .iter()
        .any(|(k, b)| k.solid_at(*b).overlaps(&rect))
    {
        return "prop-footprint".into();
    }
    let under = kinds_under(rect);
    if under.len() > 1 || under.contains(&Ground::Cobble) {
        return "plaza-edge".into();
    }
    let base = match under.first() {
        Some(Ground::Flagstone) => "plaza-interior",
        Some(Ground::Dirt) => "path",
        Some(Ground::Grass) => "grass",
        Some(Ground::Cobble) | None => "plaza-edge",
    };
    if kinds_under(grow(rect, TILE)).len() > 1 {
        format!("{base}(<1 tile from edge)")
    } else {
        base.into()
    }
}

#[test]
fn npc_ground_audit() {
    let mut counts: Vec<(String, usize)> = Vec::new();
    println!("idx | npc | district | pos | feet tile | class");
    for (i, spec) in NPC_ROSTER.iter().enumerate() {
        let p = district_slot(i);
        let feet = ground_at(Vec2::new(p.x, p.y - spec.native_px * 0.5 + 2.0));
        let district = district_of(i).map_or("?", |d| DISTRICTS[d].name);
        let class = classify(i);
        println!(
            "{i:2} | {} | {district} | ({:.0},{:.0}) | {feet:?} | {class}",
            spec.name, p.x, p.y
        );
        match counts.iter_mut().find(|(c, _)| *c == class) {
            Some((_, n)) => *n += 1,
            None => counts.push((class, 1)),
        }
    }
    // The village has no water tiles yet (atlas tile 9 is unused).
    println!("counts: {counts:?} (water: 0, no water in the layout)");
    let total: usize = counts.iter().map(|(_, n)| n).sum();
    assert_eq!(total, NPC_ROSTER.len());
}

/// The seven NPCs from roster name `first` to `boss` stand in `district`,
/// and the 64 px boss stands alone, centred, in its back (north) row,
/// with the six villagers in front of him.
fn assert_cast_with_boss_at_the_back(district: &str, first: &str, boss: &str) {
    let d_index = DISTRICTS
        .iter()
        .position(|d| d.name == district)
        .unwrap_or_else(|| panic!("no district {district}"));
    let d = &DISTRICTS[d_index];
    assert_eq!(d.count, 7, "{district}");
    let first = NPC_ROSTER
        .iter()
        .position(|s| s.name == first)
        .unwrap_or_else(|| panic!("{first} not in the roster"));
    let boss_i = first + 6;
    assert_eq!(NPC_ROSTER[boss_i].name, boss);
    assert_eq!(NPC_ROSTER[boss_i].native_px, 64.0, "{boss}");
    for (i, spec) in NPC_ROSTER.iter().enumerate().skip(first).take(7) {
        assert_eq!(district_of(i), Some(d_index), "{}", spec.name);
    }
    let p = district_slot(boss_i);
    assert_eq!(p.x, d.center.x, "{boss} is centred in its row");
    for (i, spec) in NPC_ROSTER.iter().enumerate().skip(first).take(6) {
        assert!(
            district_slot(i).y < p.y,
            "{} stands in front of {boss}",
            spec.name
        );
    }
}

/// No NPC stands under a back landmark (the Throne Dais, spawned, and the
/// Library's reserved REF-95 spot, not spawned): no NPC sprite or talk
/// spot overlaps either 64x64 plot, and each district's 64 px boss stands
/// centred directly in front of (south of) its plot, his sprite ending at
/// most 16 px below it.
#[test]
fn no_npc_stands_under_a_landmark() {
    use pledge_and_crown::plugins::world::{
        BACK_LANDMARK_DISTRICTS, BACK_LANDMARK_PX, back_landmark_base,
    };
    assert_eq!(
        BACK_LANDMARK_DISTRICTS,
        &["The Forbidden Library", "The Throne"]
    );
    for name in BACK_LANDMARK_DISTRICTS {
        let d = DISTRICTS
            .iter()
            .position(|x| x.name == *name)
            .unwrap_or_else(|| panic!("no district {name}"));
        let base = back_landmark_base(d).expect("back landmark");
        let half = BACK_LANDMARK_PX * 0.5;
        let plot = Aabb::from_base(base.x, half, base.y, base.y + BACK_LANDMARK_PX);
        for (i, spec) in NPC_ROSTER.iter().enumerate() {
            let p = district_slot(i);
            let h = spec.native_px * 0.5;
            let sprite = Aabb::from_base(p.x, h, p.y - h, p.y + h);
            assert!(
                !sprite.overlaps(&plot),
                "{} under the {name} landmark",
                spec.name
            );
            let talk = p + TALK_SPOT_OFFSET;
            let foot = PLAYER_FOOT.translated(talk);
            assert!(!foot.overlaps(&plot), "{}'s talk spot under it", spec.name);
        }
        let first: usize = DISTRICTS[..d].iter().map(|x| x.count).sum();
        let boss = first + DISTRICTS[d].count - 1;
        let p = district_slot(boss);
        assert_eq!(p.x, base.x, "{name}: boss centred on the landmark");
        let top = p.y + NPC_ROSTER[boss].native_px * 0.5;
        assert!(
            top <= base.y && base.y - top <= 16.0,
            "{name}: {} stands just in front (top {top}, landmark base {})",
            NPC_ROSTER[boss].name,
            base.y
        );
    }
}

/// Every roster slot belongs to a district (none falls to the south-edge
/// safety net); the Act 9 cast stands in the Forbidden Library with the
/// 64 px Librarian alone and centred in its back (north) row; the Act 10
/// cast stands in the Throne the same way, the 64 px Compiler at the back
/// in front of the dais. The Compiler is the last roster entry.
#[test]
fn acts_9_and_10_stand_in_the_library_and_the_throne() {
    let total: usize = DISTRICTS.iter().map(|d| d.count).sum();
    assert_eq!(total, NPC_ROSTER.len(), "district counts cover the roster");
    assert_cast_with_boss_at_the_back("The Forbidden Library", "The Shelfmark", "The Librarian");
    assert_cast_with_boss_at_the_back("The Throne", "The Assayer", "The Compiler");
    assert_eq!(
        NPC_ROSTER.last().map(|s| s.name),
        Some("The Compiler"),
        "the Compiler ends the roster"
    );
}

/// The placement gate for every roster NPC, so a new act cannot put a
/// villager on a plaza wall again:
/// 1. the sprite and a one-tile margin around it are ONE walkable ground kind
///    (plaza interior, path or grass; never the cobble edge ring);
/// 2. the sprite stands inside its own district's plaza (no district swaps);
/// 3. no prop's sprite overlaps it (so not its collision box either);
/// 4. the talk spot in front (south) is in range, nearest to this NPC, on the
///    same ground, clear of every prop's collision box and every other NPC;
/// 5. any two NPC sprites are at least two tiles apart, and none is within
///    two tiles of the new-game player spawn.
#[test]
fn npc_placement_invariants() {
    assert!(TALK_SPOT_OFFSET.length() <= INTERACT_RADIUS_PX);
    let props = all_props();
    let rects: Vec<Aabb> = (0..NPC_ROSTER.len()).map(sprite_rect).collect();
    let mut failures: Vec<String> = Vec::new();
    for (i, spec) in NPC_ROSTER.iter().enumerate() {
        let name = spec.name;
        let pos = district_slot(i);
        let rect = rects[i];

        // 1. One walkable kind under the sprite and one tile around it.
        let around = kinds_under(grow(rect, TILE));
        let kind = match around.as_slice() {
            [k @ (Ground::Flagstone | Ground::Dirt | Ground::Grass)] => Some(*k),
            other => {
                failures.push(format!("{name}: sprite + 1 tile margin on {other:?}"));
                None
            }
        };
        if !inside_map(&rect) {
            failures.push(format!("{name}: leaves the map at {pos:?}"));
        }

        // 2. Inside its own district's plaza.
        match district_of(i).map(|d| &DISTRICTS[d]) {
            Some(d) => {
                let off = (rect.min - d.center).abs().max((rect.max - d.center).abs());
                if off.max_element() > d.plaza_half {
                    failures.push(format!("{name}: outside {}'s plaza", d.name));
                }
            }
            None => failures.push(format!("{name}: no district for roster slot {i}")),
        }

        // 3. Clear of every prop.
        for (k, b) in &props {
            if k.sprite_rect_at(*b).overlaps(&rect) {
                failures.push(format!("{name}: under the {k:?} at {b:?}"));
            }
        }

        // 4. The talk spot.
        let spot = pos + TALK_SPOT_OFFSET;
        let feet = PLAYER_FOOT.translated(spot);
        let body = square(spot, PLAYER_PX);
        let nearest = (0..NPC_ROSTER.len())
            .filter(|&j| district_slot(j).distance(spot) <= INTERACT_RADIUS_PX)
            .min_by(|&a, &b| {
                district_slot(a)
                    .distance(spot)
                    .total_cmp(&district_slot(b).distance(spot))
            });
        if nearest != Some(i) {
            failures.push(format!("{name}: talk spot {spot:?} reaches {nearest:?}"));
        }
        if !inside_map(&body) {
            failures.push(format!("{name}: talk spot {spot:?} off the map"));
        }
        if let Some(k) = kind {
            let under_feet = kinds_under(feet);
            if under_feet != vec![k] {
                failures.push(format!(
                    "{name}: talk spot feet on {under_feet:?}, NPC on {k:?}"
                ));
            }
        }
        for (k, b) in &props {
            if k.solid_at(*b).overlaps(&feet) {
                failures.push(format!("{name}: talk spot inside the {k:?} at {b:?}"));
            }
        }
        for (j, other) in rects.iter().enumerate() {
            if j != i && other.overlaps(&body) {
                failures.push(format!("{name}: talk spot overlaps {}", NPC_ROSTER[j].name));
            }
        }

        // 5. Spacing.
        for (j, other) in rects.iter().enumerate().skip(i + 1) {
            if gap(&rect, other) < 2.0 * TILE {
                failures.push(format!(
                    "{name}: within two tiles of {}",
                    NPC_ROSTER[j].name
                ));
            }
        }
        if gap(&rect, &square(PLAYER_SPAWN, PLAYER_PX)) < 2.0 * TILE {
            failures.push(format!("{name}: within two tiles of the player spawn"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} placement failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
