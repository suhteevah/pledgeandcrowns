// SPDX-License-Identifier: MIT
//! Collision probes: walk the player into one cottage's wall from each of
//! the four sides and check the feet never end inside a solid box.
//!
//! The same probe table drives two things:
//! - the `scene-collision-*` captures, which hold a movement key through
//!   Bevy's own `ButtonInput` so the real `player::move_player` system does
//!   the walking inside the running game, and
//! - the headless test below, which mounts that same `move_player` system in
//!   a `MinimalPlugins` app with a fixed 60 Hz clock.

use bevy::prelude::*;

use crate::plugins::player::PLAYER_FOOT;
use crate::plugins::prop::{Aabb, PropKind, Side, cottage_base};

/// The cottage the probes push against: Hearthstone Square, north side,
/// west of centre. Clear of NPC slots and other props.
pub const PROBE_COTTAGE: (usize, Side, f32) = (0, Side::North, -170.0);
/// How long each probe holds its movement key (simulated seconds).
pub const PUSH_SECS: f32 = 1.0;
/// Free space between the feet and the wall when a probe starts.
const APPROACH_GAP: f32 = 30.0;

/// One approach to the probe cottage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionProbe {
    /// Which face of the wall the player walks into.
    pub label: &'static str,
    /// Player (sprite centre) start position.
    pub start: Vec2,
    /// Movement key held for [`PUSH_SECS`].
    pub key: KeyCode,
}

/// Bottom-centre anchor of the probe cottage.
pub fn probe_cottage_base() -> Option<Vec2> {
    cottage_base(PROBE_COTTAGE.0, PROBE_COTTAGE.1, PROBE_COTTAGE.2)
}

/// The probe cottage's solid wall box.
pub fn probe_wall() -> Option<Aabb> {
    probe_cottage_base().map(|b| PropKind::Cottage.solid_at(b))
}

/// The four approaches, feet centred on the wall face they push into.
pub fn probes() -> Vec<CollisionProbe> {
    let Some(wall) = probe_wall() else {
        tracing::warn!("collision probe cottage missing; no probes");
        return Vec::new();
    };
    let foot = PLAYER_FOOT;
    let foot_cx = (foot.min.x + foot.max.x) * 0.5;
    let foot_cy = (foot.min.y + foot.max.y) * 0.5;
    let wall_cx = (wall.min.x + wall.max.x) * 0.5;
    let wall_cy = (wall.min.y + wall.max.y) * 0.5;
    vec![
        CollisionProbe {
            label: "west",
            start: Vec2::new(wall.min.x - foot.max.x - APPROACH_GAP, wall_cy - foot_cy),
            key: KeyCode::KeyD,
        },
        CollisionProbe {
            label: "east",
            start: Vec2::new(wall.max.x - foot.min.x + APPROACH_GAP, wall_cy - foot_cy),
            key: KeyCode::KeyA,
        },
        CollisionProbe {
            label: "south",
            start: Vec2::new(wall_cx - foot_cx, wall.min.y - foot.max.y - APPROACH_GAP),
            key: KeyCode::KeyW,
        },
        CollisionProbe {
            label: "north",
            start: Vec2::new(wall_cx - foot_cx, wall.max.y - foot.min.y + APPROACH_GAP),
            key: KeyCode::KeyS,
        },
    ]
}

/// True when the player's feet at `pos` overlap any of `solids`.
pub fn feet_inside_any(pos: Vec2, solids: &[Aabb]) -> bool {
    let f = PLAYER_FOOT.translated(pos);
    solids.iter().any(|s| f.overlaps(s))
}

/// Distance from the feet to the wall face the probe pushed into (0 when
/// flush). Negative would mean penetration.
pub fn gap_to_wall(probe: &CollisionProbe, pos: Vec2, wall: &Aabb) -> f32 {
    let f = PLAYER_FOOT.translated(pos);
    match probe.label {
        "west" => wall.min.x - f.max.x,
        "east" => f.min.x - wall.max.x,
        "south" => wall.min.y - f.max.y,
        _ => f.min.y - wall.max.y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::editor::EditorState;
    use crate::plugins::player::{Player, move_player};
    use crate::plugins::prop::{Solid, all_props};
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    const HZ: f32 = 60.0;

    /// Run the real `move_player` for `secs` of 60 Hz frames with `key`
    /// held, against every prop's solid box. Returns the final position.
    fn simulate_push(start: Vec2, key: KeyCode, secs: f32, solids: &[Aabb]) -> Vec2 {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / HZ,
            )))
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<EditorState>()
            .add_systems(Update, move_player);
        let player = app
            .world_mut()
            .spawn((Transform::from_xyz(start.x, start.y, 0.0), Player))
            .id();
        for s in solids {
            app.world_mut().spawn(Solid(*s));
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        // +1: the very first update has a zero delta.
        let frames = (secs * HZ).ceil() as u32 + 1;
        for _ in 0..frames {
            app.update();
        }
        let t = app
            .world()
            .get::<Transform>(player)
            .expect("player transform");
        t.translation.truncate()
    }

    fn village_solids() -> Vec<Aabb> {
        all_props().iter().map(|(k, b)| k.solid_at(*b)).collect()
    }

    #[test]
    fn probes_start_in_free_space() {
        let solids = village_solids();
        let ps = probes();
        assert_eq!(ps.len(), 4);
        for p in &ps {
            assert!(
                !feet_inside_any(p.start, &solids),
                "{} probe starts inside a solid at {:?}",
                p.label,
                p.start
            );
        }
    }

    #[test]
    fn walking_into_the_cottage_never_ends_inside_a_solid() {
        let solids = village_solids();
        let wall = probe_wall().expect("probe cottage exists");
        for p in probes() {
            let end = simulate_push(p.start, p.key, PUSH_SECS, &solids);
            assert!(
                !feet_inside_any(end, &solids),
                "{} probe ended inside a solid at {end:?}",
                p.label
            );
            let gap = gap_to_wall(&p, end, &wall);
            // It must actually have reached the wall (a full second at
            // 420 px/s is far more than the 30 px approach) and stopped
            // flush against it, within one 4 px collision sub-step.
            assert!(
                (0.0..=4.0).contains(&gap),
                "{} probe stopped {gap} px from the wall at {end:?}",
                p.label
            );
            assert!(
                (end - p.start).length() > APPROACH_GAP - 4.0,
                "{} probe barely moved: {:?} -> {end:?}",
                p.label,
                p.start
            );
        }
    }
}
