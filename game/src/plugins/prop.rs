// SPDX-License-Identifier: MIT
//! Village props: oaks (REF-67, 32x48), cottages (REF-68, 64x64) and the
//! district landmarks (the Throne Dais, REF-96, 64x64).
//!
//! Props are ordinary sprites, not tiles. Each one is anchored bottom-centre
//! at its ground line, y-sorted (lower on screen = drawn in front) and owns a
//! solid [`Aabb`] footprint that `player.rs` refuses to walk into.
//!
//! Placement is data: [`COTTAGE_SPOTS`] and [`MEADOW_OAKS`] are hand-picked
//! const tables, and the border treeline is a deterministic function of the
//! map size (fixed spacing + hash jitter), so every run builds the same
//! village. `tests` below check every prop sits on grass, clear of roads,
//! plazas, NPC slots and each other.
//!
//! The same y-sort system also orders the player and NPC sprites, by querying
//! their existing marker components (`Player`, `Npc`) — npc.rs is untouched.
//!
//! Occlusion fade: a roof or canopy that y-sorts in front of a character can
//! hide it completely. [`fade_occluders`] fades any prop whose opaque art
//! ([`PropKind::visual_local`]) overlaps a character drawn behind it down to
//! [`OCCLUDER_ALPHA`] over [`FADE_SECS`], and back to opaque once clear. The
//! decision ([`occludes`]) and the ramp ([`step_alpha`]) are pure functions.

use crate::assets::{SPRITE_PROP_COTTAGE, SPRITE_PROP_OAK, SPRITE_PROP_THRONE_DAIS};
use crate::plugins::animation::{SpriteLift, unlifted_anchor};
use crate::plugins::npc::Npc;
use crate::plugins::player::Player;
use crate::plugins::state::GameState;
use crate::plugins::world::{DISTRICTS, WORLD_HALF_H, WORLD_HALF_W};
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Axis-aligned box in world space. `min` is the bottom-left corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    /// Box spanning `half_w` either side of `x`, from `y0` up to `y1`.
    pub const fn from_base(x: f32, half_w: f32, y0: f32, y1: f32) -> Self {
        Self {
            min: Vec2::new(x - half_w, y0),
            max: Vec2::new(x + half_w, y1),
        }
    }

    pub fn translated(self, by: Vec2) -> Self {
        Self {
            min: self.min + by,
            max: self.max + by,
        }
    }

    /// Strict overlap: boxes that only touch along an edge do NOT overlap, so
    /// the player can stand flush against a wall and still slide along it.
    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

/// Largest single step `slide_move` takes, so a long frame (hitch, alt-tab)
/// can't tunnel the player straight through a 12px oak trunk.
const MAX_STEP_PX: f32 = 4.0;

/// Pure collision helper: move a body whose footprint is `foot` (relative to
/// `pos`) by `delta`, never ending a step inside any of `solids`. Axes are
/// resolved separately — X first, then Y — so pushing diagonally into a wall
/// slides along it instead of sticking.
pub fn slide_move(pos: Vec2, delta: Vec2, foot: Aabb, solids: &[Aabb]) -> Vec2 {
    let steps = (delta.abs().max_element() / MAX_STEP_PX).ceil().max(1.0) as u32;
    let step = delta / steps as f32;
    let blocked = |p: Vec2| {
        let f = foot.translated(p);
        solids.iter().any(|s| f.overlaps(s))
    };
    let mut p = pos;
    for _ in 0..steps {
        let try_x = Vec2::new(p.x + step.x, p.y);
        if !blocked(try_x) {
            p = try_x;
        }
        let try_y = Vec2::new(p.x, p.y + step.y);
        if !blocked(try_y) {
            p = try_y;
        }
    }
    p
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    Oak,
    Cottage,
    /// The Throne Dais (REF-96, approved 2026-11-15), the Act 10 district
    /// landmark: 64x64, placed at [`LANDMARKS`].
    ThroneDais,
}

impl PropKind {
    pub fn sprite_path(self) -> &'static str {
        match self {
            PropKind::Oak => SPRITE_PROP_OAK,
            PropKind::Cottage => SPRITE_PROP_COTTAGE,
            PropKind::ThroneDais => SPRITE_PROP_THRONE_DAIS,
        }
    }

    /// Native sprite size in pixels (displayed at 1x, like characters).
    pub fn size(self) -> Vec2 {
        match self {
            PropKind::Oak => Vec2::new(32.0, 48.0),
            PropKind::Cottage | PropKind::ThroneDais => Vec2::new(64.0, 64.0),
        }
    }

    /// Ground line above the sprite's bottom edge (the art has empty rows
    /// below the trunk / footing). Used as the y-sort point so a player
    /// standing just in front of a wall sorts in front of it.
    pub fn ground_offset(self) -> f32 {
        match self {
            PropKind::Oak => 3.0,        // REF-67 row 44 is the trunk base
            PropKind::Cottage => 5.0,    // REF-68 row 58 is the footing
            PropKind::ThroneDais => 3.0, // REF-96 row 60 is the bottom step's edge
        }
    }

    /// Solid footprint relative to the anchor (bottom-centre).
    /// Oak: the trunk base (REF-67 cols 10..21, rows 36..44).
    /// Cottage: the wall rows only (REF-68 cols 5..58, rows 37..58 = 54x22),
    /// so the player can walk behind the roof. Deliberately NOT widened to
    /// the 60 px roof eave (cols 2..61): the eave overhangs the wall by only
    /// 3 px, while the player's body reaches ~5 px past its feet box, so a
    /// wider box would still overlap the eave and only push the player off
    /// the wall. The occlusion fade keeps a character under the eave visible.
    /// Throne Dais: the three steps (REF-96 cols 2..61, rows 44..60 =
    /// 60x17), so the player cannot climb onto the dais but can walk behind
    /// the throne and banners, which fade like a roof.
    pub fn solid_local(self) -> Aabb {
        match self {
            PropKind::Oak => Aabb::from_base(0.0, 6.0, 3.0, 11.0),
            PropKind::Cottage => Aabb::from_base(0.0, 27.0, 5.0, 27.0),
            PropKind::ThroneDais => Aabb::from_base(0.0, 30.0, 3.0, 20.0),
        }
    }

    pub fn solid_at(self, base: Vec2) -> Aabb {
        self.solid_local().translated(base)
    }

    /// Opaque bounds of the art relative to the anchor, used for the
    /// occlusion fade. Oak: REF-67 cols 2..29, rows 1..44. Cottage: REF-68
    /// cols 2..61 (the eave), rows 0..58 (chimney top to footing). Throne
    /// Dais: REF-96 cols 2..61 (the bottom step), rows 0..60 (finial to the
    /// bottom step).
    pub fn visual_local(self) -> Aabb {
        match self {
            PropKind::Oak => Aabb::from_base(0.0, 14.0, 3.0, 47.0),
            PropKind::Cottage => Aabb::from_base(0.0, 30.0, 5.0, 64.0),
            PropKind::ThroneDais => Aabb::from_base(0.0, 30.0, 3.0, 64.0),
        }
    }

    pub fn visual_at(self, base: Vec2) -> Aabb {
        self.visual_local().translated(base)
    }

    /// Full sprite rectangle for a prop anchored at `base`.
    pub fn sprite_rect_at(self, base: Vec2) -> Aabb {
        let s = self.size();
        Aabb::from_base(base.x, s.x * 0.5, base.y, base.y + s.y)
    }
}

/// A placed prop. Its `Transform` translation is the bottom-centre anchor.
#[derive(Component, Clone, Copy, Debug)]
pub struct Prop {
    pub kind: PropKind,
}

/// World-space solid box. Read by `player::move_player`.
#[derive(Component, Clone, Copy, Debug)]
pub struct Solid(pub Aabb);

/// Which edge of a district plaza a cottage stands outside.
#[derive(Clone, Copy, Debug)]
pub enum Side {
    North,
    South,
    West,
    East,
}

/// Gap between a plaza's cobble edge and the nearest edge of a cottage sprite.
const COTTAGE_GAP: f32 = 24.0;

/// Cottage placements: (district index into `DISTRICTS`, plaza side, offset
/// along that side from the district centre). Sides that would push a cottage
/// off the map or onto a street are simply not used for that district.
pub const COTTAGE_SPOTS: &[(usize, Side, f32)] = &[
    // Hearthstone Square
    (0, Side::North, -170.0),
    (0, Side::North, 170.0),
    (0, Side::South, -170.0),
    (0, Side::South, 170.0),
    // Market Row
    (1, Side::North, -100.0),
    (1, Side::South, -100.0),
    (1, Side::West, -150.0),
    // The Commons
    (2, Side::North, 100.0),
    (2, Side::South, 100.0),
    (2, Side::East, -150.0),
    // Guildhall Quarter
    (3, Side::North, -130.0),
    (3, Side::North, 130.0),
    (3, Side::West, 0.0),
    // The Mage's Tower
    (4, Side::North, -130.0),
    (4, Side::North, 130.0),
    (4, Side::East, 0.0),
    // Tavern Row
    (5, Side::South, -150.0),
    (5, Side::South, 150.0),
    (5, Side::West, 100.0),
    (5, Side::East, 100.0),
    // The Forge
    (6, Side::North, -130.0),
    (6, Side::North, 130.0),
    (6, Side::West, 0.0),
    // The Wharf
    (7, Side::North, -130.0),
    (7, Side::North, 130.0),
    (7, Side::East, 0.0),
    // The Vault
    (8, Side::North, -150.0),
    (8, Side::North, 150.0),
    (8, Side::West, -100.0),
    (8, Side::East, -100.0),
    // The Forbidden Library. North is the main street and south is the
    // treeline, so cottages stand west and east. Its landmark stands on
    // the plaza itself (`PENDING_LANDMARKS`).
    (9, Side::West, -100.0),
    (9, Side::East, -100.0),
    (9, Side::East, 100.0),
    // The Throne (Act 10), the same way; the dais stands on the plaza
    // (`LANDMARKS`).
    (10, Side::West, -100.0),
    (10, Side::West, 100.0),
    (10, Side::East, -100.0),
];

/// District landmarks that are approved and spawned: `(district index,
/// kind)`. A landmark is a 64x64 bottom-centre-anchored prop standing ON
/// its plaza at the back-centre (north edge), placed by
/// [`world::back_landmark_base`](crate::plugins::world::back_landmark_base);
/// the district's boss stands centred directly in front of it. The Throne
/// Dais (REF-96, approved by Matt 2026-11-15) stands behind the Compiler.
pub const LANDMARKS: &[(usize, PropKind)] = &[(10, PropKind::ThroneDais)];

/// Landmarks reserved but not spawned, `(district index, ref)`, at the same
/// back-centre spot: REF-95 (The Sealed Stacks) is a draft awaiting Matt's
/// approval, its spot behind the Librarian. Wiring it is a new `PropKind`
/// moved to [`LANDMARKS`]; `pending_landmark_spots_are_clear` keeps the
/// spot on flagstone, clear of every prop and NPC, so that is all it takes.
pub const PENDING_LANDMARKS: &[(usize, &str)] = &[(9, "REF-95 The Sealed Stacks")];

/// Free-standing oaks in open meadow, between the quarters.
pub const MEADOW_OAKS: &[Vec2] = &[
    Vec2::new(-400.0, 450.0),
    Vec2::new(400.0, 450.0),
    Vec2::new(-400.0, -450.0),
    Vec2::new(400.0, -450.0),
    Vec2::new(-1200.0, 500.0),
    Vec2::new(1200.0, 500.0),
    Vec2::new(-1200.0, -500.0),
    Vec2::new(1200.0, -500.0),
    Vec2::new(-1250.0, 900.0),
    Vec2::new(1250.0, -900.0),
];

/// Treeline spacing along the top/bottom and left/right map edges.
const TREELINE_STEP_X: f32 = 26.0;
const TREELINE_STEP_Y: f32 = 34.0;
/// Max +/- jitter applied to each treeline oak (deterministic per index).
const TREELINE_JITTER: f32 = 6.0;

/// Small integer hash -> [-1, 1]. Deterministic, no RNG dependency.
fn jitter(i: u32, salt: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 13;
    (h % 2001) as f32 / 1000.0 - 1.0
}

/// Bottom-centre anchor of the cottage at `spot`.
pub fn cottage_base(district: usize, side: Side, offset: f32) -> Option<Vec2> {
    let d = DISTRICTS.get(district)?;
    let size = PropKind::Cottage.size();
    let edge = d.plaza_half + COTTAGE_GAP;
    Some(match side {
        Side::North => Vec2::new(d.center.x + offset, d.center.y + edge),
        Side::South => Vec2::new(d.center.x + offset, d.center.y - edge - size.y),
        Side::West => Vec2::new(d.center.x - edge - size.x * 0.5, d.center.y + offset),
        Side::East => Vec2::new(d.center.x + edge + size.x * 0.5, d.center.y + offset),
    })
}

/// Every prop in the village, in a stable order.
pub fn all_props() -> Vec<(PropKind, Vec2)> {
    let mut out = Vec::new();

    // Border treeline. Top/bottom rows run the full width; side columns fill
    // between them. Oaks are anchored so the canopy stays inside the map.
    let oak = PropKind::Oak.size();
    let top_y = WORLD_HALF_H - oak.y - 2.0;
    let bottom_y = -WORLD_HALF_H + 2.0;
    let mut i = 0u32;
    // Inset by the jitter so the end oaks can't poke off the map sides.
    let mut x = -WORLD_HALF_W + oak.x * 0.5 + TREELINE_JITTER;
    while x <= WORLD_HALF_W - oak.x * 0.5 - TREELINE_JITTER {
        for (row_y, salt) in [(top_y, 1u32), (bottom_y, 2u32)] {
            let j = Vec2::new(jitter(i, salt), jitter(i, salt + 10)) * TREELINE_JITTER;
            // Keep the canopy on the map: only jitter inward vertically.
            let jy = if salt == 1 { -j.y.abs() } else { j.y.abs() };
            out.push((PropKind::Oak, Vec2::new(x + j.x, row_y + jy)));
        }
        i += 1;
        x += TREELINE_STEP_X;
    }
    let left_x = -WORLD_HALF_W + oak.x * 0.5 + 2.0;
    let right_x = WORLD_HALF_W - oak.x * 0.5 - 2.0;
    let mut y = bottom_y + TREELINE_STEP_Y;
    while y <= top_y - TREELINE_STEP_Y {
        for (col_x, salt) in [(left_x, 3u32), (right_x, 4u32)] {
            let j = Vec2::new(jitter(i, salt), jitter(i, salt + 10)) * TREELINE_JITTER;
            let jx = if salt == 3 { j.x.abs() } else { -j.x.abs() };
            out.push((PropKind::Oak, Vec2::new(col_x + jx, y + j.y)));
        }
        i += 1;
        y += TREELINE_STEP_Y;
    }

    out.extend(MEADOW_OAKS.iter().map(|p| (PropKind::Oak, *p)));

    for &(d, side, offset) in COTTAGE_SPOTS {
        match cottage_base(d, side, offset) {
            Some(base) => out.push((PropKind::Cottage, base)),
            None => tracing::warn!("cottage spot references unknown district {d}; skipped"),
        }
    }
    for &(d, kind) in LANDMARKS {
        match crate::plugins::world::back_landmark_base(d) {
            Some(base) => out.push((kind, base)),
            None => tracing::warn!(?kind, "landmark references unknown district {d}; skipped"),
        }
    }
    out
}

/// Y-sorted sprites live in this z band: above both tilemap layers (0, 0.5),
/// below the quest markers (children at local z +50).
const YSORT_Z_MIN: f32 = 2.0;
const YSORT_Z_SPAN: f32 = 6.0;

/// Map a ground-line world y to a z in the y-sort band. Lower on screen
/// (smaller y) gives a larger z, so it draws in front.
pub fn ysort_z(ground_y: f32) -> f32 {
    let t = ((WORLD_HALF_H - ground_y) / (2.0 * WORLD_HALF_H)).clamp(0.0, 1.0);
    YSORT_Z_MIN + t * YSORT_Z_SPAN
}

/// Alpha a prop fades to while it hides a character. 0.45 left an oak canopy
/// nearly invisible against grass in captures; 0.62 keeps the prop readable
/// while the character still shows through.
pub const OCCLUDER_ALPHA: f32 = 0.62;
/// Time a full fade (opaque <-> [`OCCLUDER_ALPHA`]) takes.
pub const FADE_SECS: f32 = 0.15;

/// The part of a character sprite that counts as "the character": the
/// middle half of its width (the 32 px art is mostly transparent at the
/// sides) over its full height. `center` is the sprite centre.
pub fn character_body(center: Vec2, size: Vec2) -> Aabb {
    Aabb::from_base(
        center.x,
        size.x * 0.25,
        center.y - size.y * 0.5,
        center.y + size.y * 0.5,
    )
}

/// True when a prop drawn in front of a character (higher z) covers part
/// of it, i.e. the prop should fade so the character is not lost.
pub fn occludes(character: Aabb, character_z: f32, prop: Aabb, prop_z: f32) -> bool {
    prop_z > character_z && prop.overlaps(&character)
}

/// Fade target for a prop.
pub fn fade_target(occluding: bool) -> f32 {
    if occluding { OCCLUDER_ALPHA } else { 1.0 }
}

/// Move `current` toward `target` at the rate that crosses the full fade
/// range in [`FADE_SECS`], never overshooting.
pub fn step_alpha(current: f32, target: f32, dt: f32) -> f32 {
    let max = (1.0 - OCCLUDER_ALPHA) / FADE_SECS * dt.max(0.0);
    current + (target - current).clamp(-max, max)
}

pub struct PropPlugin;

impl Plugin for PropPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("PropPlugin::build");
        app.add_systems(OnEnter(GameState::InGame), spawn_props)
            .add_systems(
                PostUpdate,
                (ysort, fade_occluders)
                    .chain()
                    .run_if(in_state(GameState::InGame))
                    .before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

fn spawn_props(mut commands: Commands, asset_server: Res<AssetServer>) {
    let props = all_props();
    let mut oaks = 0usize;
    let mut cottages = 0usize;
    let mut landmarks = 0usize;
    for (kind, base) in &props {
        match kind {
            PropKind::Oak => oaks += 1,
            PropKind::Cottage => cottages += 1,
            PropKind::ThroneDais => landmarks += 1,
        }
        tracing::trace!(?kind, x = base.x, y = base.y, "spawning prop");
        commands.spawn((
            Sprite {
                image: asset_server.load(kind.sprite_path()),
                custom_size: Some(kind.size()),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(base.x, base.y, ysort_z(base.y + kind.ground_offset())),
            Prop { kind: *kind },
            Solid(kind.solid_at(*base)),
        ));
    }
    tracing::info!(
        "spawned {} props ({oaks} oaks, {cottages} cottages, {landmarks} landmarks)",
        props.len()
    );
}

type YSorted = Or<(With<Prop>, With<Player>, With<Npc>)>;

/// Re-derive z from the ground line for every prop, the player and every NPC.
/// Characters are centre-anchored sprites, so their ground line is the sprite
/// bottom; props add their art's ground offset. A character's animation lift
/// (pass frame, idle bob) is removed first, so it never changes draw order.
#[allow(clippy::type_complexity)] // standard Bevy query tuple
fn ysort(
    mut q: Query<
        (
            &mut Transform,
            &Sprite,
            &Anchor,
            Option<&SpriteLift>,
            Option<&Prop>,
        ),
        YSorted,
    >,
) {
    let mut changed = 0usize;
    for (mut tf, sprite, anchor, lift, prop) in q.iter_mut() {
        let h = sprite.custom_size.map(|s| s.y).unwrap_or(0.0);
        let bottom = tf.translation.y - (unlifted_anchor(anchor, lift, h).y + 0.5) * h;
        let ground = bottom + prop.map(|p| p.kind.ground_offset()).unwrap_or(0.0);
        let z = ysort_z(ground);
        if (tf.translation.z - z).abs() > f32::EPSILON {
            tf.translation.z = z;
            changed += 1;
        }
    }
    if changed > 0 {
        tracing::debug!(changed, "ysort updated z");
    }
}

type Characters = (Or<(With<Player>, With<Npc>)>, Without<Prop>);

/// Fade props that hide the player or an NPC (see module docs).
fn fade_occluders(
    time: Res<Time>,
    chars: Query<(&Transform, &Sprite, &Anchor, Option<&SpriteLift>), Characters>,
    mut props: Query<(&Transform, &Prop, &mut Sprite)>,
    mut last_faded: Local<usize>,
) {
    let bodies: Vec<(Aabb, f32)> = chars
        .iter()
        .map(|(tf, sprite, anchor, lift)| {
            let size = sprite.custom_size.unwrap_or(Vec2::ZERO);
            let center = tf.translation.truncate() - unlifted_anchor(anchor, lift, size.y) * size;
            (character_body(center, size), tf.translation.z)
        })
        .collect();
    let dt = time.delta_secs();
    let mut faded = 0usize;
    for (tf, prop, mut sprite) in props.iter_mut() {
        let rect = prop.kind.visual_at(tf.translation.truncate());
        let z = tf.translation.z;
        let hiding = bodies.iter().any(|(b, bz)| occludes(*b, *bz, rect, z));
        if hiding {
            faded += 1;
        }
        let current = sprite.color.alpha();
        let next = step_alpha(current, fade_target(hiding), dt);
        if (next - current).abs() > f32::EPSILON {
            sprite.color.set_alpha(next);
        }
    }
    if faded != *last_faded {
        tracing::debug!(faded, was = *last_faded, "occluding props changed");
        *last_faded = faded;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::npc::NPC_ROSTER;
    use crate::plugins::world::{Ground, district_slot, ground_at};

    #[test]
    fn aabb_overlap_is_strict() {
        let a = Aabb::from_base(0.0, 5.0, 0.0, 10.0);
        assert!(a.overlaps(&Aabb::from_base(4.0, 5.0, 5.0, 15.0)));
        // Touching edges do not overlap.
        assert!(!a.overlaps(&Aabb::from_base(10.0, 5.0, 0.0, 10.0)));
        assert!(!a.overlaps(&Aabb::from_base(0.0, 5.0, 10.0, 20.0)));
        assert!(!a.overlaps(&Aabb::from_base(30.0, 5.0, 0.0, 10.0)));
    }

    #[test]
    fn slide_move_free_space_moves_fully() {
        let foot = Aabb::from_base(0.0, 2.0, 0.0, 2.0);
        let p = slide_move(Vec2::ZERO, Vec2::new(10.0, -7.0), foot, &[]);
        assert!((p - Vec2::new(10.0, -7.0)).length() < 1e-4);
    }

    #[test]
    fn slide_move_blocks_and_slides_along_wall() {
        // Wall to the east, x in [10, 20], tall.
        let wall = Aabb::from_base(15.0, 5.0, -100.0, 100.0);
        let foot = Aabb::from_base(0.0, 2.0, 0.0, 2.0);
        let p = slide_move(Vec2::ZERO, Vec2::new(20.0, 6.0), foot, &[wall]);
        assert!(p.x + 2.0 <= 10.0 + 1e-4, "entered the wall: {p:?}");
        assert!(p.x > 0.0, "did not approach the wall: {p:?}");
        assert!((p.y - 6.0).abs() < 1e-4, "did not slide along it: {p:?}");
    }

    #[test]
    fn slide_move_does_not_tunnel_on_long_frames() {
        let trunk = PropKind::Oak.solid_at(Vec2::new(0.0, 0.0));
        let foot = Aabb::from_base(0.0, 6.0, -16.0, -10.0);
        // Start well left, one huge step to the right through the trunk.
        let start = Vec2::new(-40.0, 20.0);
        let p = slide_move(start, Vec2::new(200.0, 0.0), foot, &[trunk]);
        assert!(!foot.translated(p).overlaps(&trunk));
        assert!(p.x < 0.0, "tunnelled through the trunk: {p:?}");
    }

    #[test]
    fn ysort_lower_draws_in_front() {
        assert!(ysort_z(-100.0) > ysort_z(100.0));
        assert!(ysort_z(-WORLD_HALF_H) <= YSORT_Z_MIN + YSORT_Z_SPAN);
        assert!(ysort_z(WORLD_HALF_H) >= YSORT_Z_MIN);
    }

    fn player_body_at_feet(feet_bottom: Vec2) -> Aabb {
        character_body(feet_bottom + Vec2::new(0.0, 16.0), Vec2::splat(32.0))
    }

    #[test]
    fn roof_fades_when_the_player_stands_behind_the_cottage() {
        let base = Vec2::new(100.0, 200.0);
        let k = PropKind::Cottage;
        let prop_z = ysort_z(base.y + k.ground_offset());
        let solid = k.solid_at(base);
        // Feet flush with the back wall: behind, roof covers the sprite.
        let feet = Vec2::new(base.x, solid.max.y);
        assert!(occludes(
            player_body_at_feet(feet),
            ysort_z(feet.y),
            k.visual_at(base),
            prop_z
        ));
        // Feet flush with the front wall: player draws in front, no fade.
        let feet = Vec2::new(base.x, solid.min.y - 6.0);
        assert!(!occludes(
            player_body_at_feet(feet),
            ysort_z(feet.y),
            k.visual_at(base),
            prop_z
        ));
        // Far behind (north of the chimney): nothing to fade.
        let feet = Vec2::new(base.x, base.y + 70.0);
        assert!(!occludes(
            player_body_at_feet(feet),
            ysort_z(feet.y),
            k.visual_at(base),
            prop_z
        ));
    }

    #[test]
    fn eave_fades_when_the_player_stands_beside_the_wall() {
        let base = Vec2::new(0.0, 0.0);
        let k = PropKind::Cottage;
        let solid = k.solid_at(base);
        // Feet box (half 6) flush with the east wall, mid-wall height.
        for x in [solid.max.x + 6.0, solid.min.x - 6.0] {
            let feet = Vec2::new(x, (solid.min.y + solid.max.y) * 0.5 - 3.0);
            assert!(
                occludes(
                    player_body_at_feet(feet),
                    ysort_z(feet.y),
                    k.visual_at(base),
                    ysort_z(base.y + k.ground_offset())
                ),
                "eave must fade with the player at x={x}"
            );
        }
    }

    #[test]
    fn canopy_fades_when_the_player_is_behind_the_oak() {
        let base = Vec2::new(-400.0, 450.0);
        let k = PropKind::Oak;
        let feet = Vec2::new(base.x + 6.0, k.solid_at(base).max.y);
        assert!(occludes(
            player_body_at_feet(feet),
            ysort_z(feet.y),
            k.visual_at(base),
            ysort_z(base.y + k.ground_offset())
        ));
        // Beside the oak but well clear of the canopy: no fade.
        let feet = Vec2::new(base.x + 40.0, base.y + 10.0);
        assert!(!occludes(
            player_body_at_feet(feet),
            ysort_z(feet.y),
            k.visual_at(base),
            ysort_z(base.y + k.ground_offset())
        ));
    }

    #[test]
    fn alpha_ramps_in_about_150ms_and_never_overshoots() {
        let mut a = 1.0;
        let dt = 1.0 / 60.0;
        let mut frames = 0;
        while (a - OCCLUDER_ALPHA).abs() > 1e-5 {
            a = step_alpha(a, fade_target(true), dt);
            assert!(a >= OCCLUDER_ALPHA - 1e-6);
            frames += 1;
            assert!(frames < 100, "fade never settled");
        }
        assert!((9..=10).contains(&frames), "~0.15 s at 60 Hz, got {frames}");
        for _ in 0..20 {
            a = step_alpha(a, fade_target(false), dt);
            assert!(a <= 1.0 + 1e-6);
        }
        assert!((a - 1.0).abs() < 1e-5, "restores when clear");
        assert_eq!(step_alpha(0.7, 1.0, -1.0), 0.7, "negative dt is a no-op");
    }

    #[test]
    fn visual_rect_covers_the_solid_and_sits_in_the_sprite() {
        for k in [PropKind::Oak, PropKind::Cottage, PropKind::ThroneDais] {
            let v = k.visual_local();
            let s = k.solid_local();
            let sprite = k.sprite_rect_at(Vec2::ZERO);
            assert!(v.min.x <= s.min.x && v.max.x >= s.max.x && v.min.y <= s.min.y);
            assert!(v.min.x >= sprite.min.x && v.max.x <= sprite.max.x);
            assert!(v.min.y >= sprite.min.y && v.max.y <= sprite.max.y);
        }
    }

    #[test]
    fn district_gets_two_to_four_cottages() {
        assert_eq!(DISTRICTS.len(), 11, "hub, eight quarters, Library, Throne");
        for (i, d) in DISTRICTS.iter().enumerate() {
            let n = COTTAGE_SPOTS.iter().filter(|(di, _, _)| *di == i).count();
            assert!((2..=4).contains(&n), "{} has {n} cottages", d.name);
        }
    }

    /// Sample every 4px across `r` and require `want` under all of it.
    fn rect_all(r: &Aabb, want: Ground) -> bool {
        let mut y = r.min.y;
        while y <= r.max.y {
            let mut x = r.min.x;
            while x <= r.max.x {
                if ground_at(Vec2::new(x, y)) != want {
                    return false;
                }
                x += 4.0;
            }
            y += 4.0;
        }
        true
    }

    /// Sample every 4px across `r` and require grass under all of it.
    fn rect_all_grass(r: &Aabb) -> bool {
        rect_all(r, Ground::Grass)
    }

    /// A landmark plot at `base` in district `d` (64x64, bottom-centre):
    /// on flagstone with a clear tile to the cobble ring on every side
    /// (the sprite grown by 16 px is still all flagstone), centred on the
    /// plaza, at its back (north) half, clear of every prop but `skip`
    /// and of every NPC sprite; the district's boss (its last roster slot,
    /// 64 px) stands centred directly in front of it, his sprite top at
    /// most 16 px below the plot and never overlapping it; and there is
    /// room to walk round it (two tiles of flagstone either side of its
    /// steps).
    fn landmark_plot_is_sound(d: usize, base: Vec2, what: &str, skip: Option<Vec2>) {
        let dist = &DISTRICTS[d];
        let plot = PropKind::ThroneDais.sprite_rect_at(base);
        let grown = Aabb {
            min: plot.min - Vec2::splat(16.0),
            max: plot.max + Vec2::splat(16.0),
        };
        assert!(
            rect_all(&grown, Ground::Flagstone),
            "{what}: not on clear flagstone"
        );
        assert_eq!(base.x, dist.center.x, "{what}: centred");
        assert!(base.y > dist.center.y, "{what}: in the back (north) half");
        for (k, b) in all_props() {
            if Some(b) == skip {
                continue;
            }
            assert!(
                !k.sprite_rect_at(b).overlaps(&plot),
                "{what} overlaps the {k:?} at {b:?}"
            );
        }
        for (i, spec) in NPC_ROSTER.iter().enumerate() {
            let s = district_slot(i);
            let half = spec.native_px * 0.5;
            let npc = Aabb::from_base(s.x, half, s.y - half, s.y + half);
            assert!(!npc.overlaps(&plot), "{what} covers NPC slot {i}");
        }
        let first: usize = DISTRICTS[..d].iter().map(|x| x.count).sum();
        let boss = first + dist.count - 1;
        assert_eq!(NPC_ROSTER[boss].native_px, 64.0, "{what}: boss is 64 px");
        let p = district_slot(boss);
        assert_eq!(p.x, base.x, "{what}: boss centred in front");
        let top = p.y + 32.0;
        assert!(
            top <= plot.min.y && plot.min.y - top <= 16.0,
            "{what}: boss top {top} vs plot bottom {}",
            plot.min.y
        );
        let solid = PropKind::ThroneDais.solid_at(base);
        for side in [
            Aabb::from_base(solid.min.x - 16.0, 16.0, solid.min.y, solid.max.y),
            Aabb::from_base(solid.max.x + 16.0, 16.0, solid.min.y, solid.max.y),
        ] {
            assert!(rect_all(&side, Ground::Flagstone), "{what}: no way round");
        }
    }

    #[test]
    fn every_prop_sits_on_grass_inside_the_map() {
        for (kind, base) in all_props() {
            let r = kind.sprite_rect_at(base);
            assert!(
                r.min.x >= -WORLD_HALF_W
                    && r.max.x <= WORLD_HALF_W
                    && r.min.y >= -WORLD_HALF_H
                    && r.max.y <= WORLD_HALF_H,
                "{kind:?} at {base:?} leaves the map"
            );
            if kind == PropKind::ThroneDais {
                // Landmarks stand ON their plaza (flagstone, clear of the
                // cobble ring); see `landmark_plot_is_sound`.
                assert!(
                    rect_all(&r, Ground::Flagstone),
                    "{kind:?} at {base:?} leaves the flagstone"
                );
            } else {
                assert!(
                    rect_all_grass(&r),
                    "{kind:?} at {base:?} overlaps cobble or road"
                );
            }
        }
    }

    #[test]
    fn cottages_and_meadow_oaks_do_not_overlap_each_other() {
        let props = all_props();
        let big: Vec<(PropKind, Aabb)> = props
            .iter()
            .filter(|(k, b)| *k != PropKind::Oak || MEADOW_OAKS.contains(b))
            .map(|(k, b)| (*k, k.sprite_rect_at(*b)))
            .collect();
        for (i, (ka, a)) in big.iter().enumerate() {
            for (kb, b) in big.iter().skip(i + 1) {
                assert!(!a.overlaps(b), "{ka:?} {a:?} overlaps {kb:?} {b:?}");
            }
        }
        // Treeline oaks may overlap each other (that's the forest), but never
        // a cottage.
        for (k, b) in &props {
            if *k != PropKind::Oak {
                continue;
            }
            let r = k.sprite_rect_at(*b);
            for (kc, c) in &big {
                if *kc != PropKind::Oak {
                    assert!(!r.overlaps(c), "oak at {b:?} overlaps the {kc:?}");
                }
            }
        }
    }

    #[test]
    fn no_prop_covers_an_npc_slot() {
        // An NPC is a 32px (Borrow Checker: 64px) centre-anchored sprite.
        for i in 0..NPC_ROSTER.len() {
            let s = district_slot(i);
            let npc = Aabb::from_base(s.x, 32.0, s.y - 32.0, s.y + 32.0);
            for (kind, base) in all_props() {
                assert!(
                    !kind.sprite_rect_at(base).overlaps(&npc),
                    "{kind:?} at {base:?} covers NPC slot {i} at {s:?}"
                );
            }
        }
    }

    /// The pending landmark spot (REF-95, art not approved, nothing
    /// spawned) is the Library plaza's back-centre, with the Librarian
    /// centred in front of it, so the approved art lands where the Throne
    /// Dais does in its district.
    #[test]
    fn pending_landmark_spots_are_clear() {
        assert_eq!(PENDING_LANDMARKS.len(), 1, "only REF-95 waits");
        let library = DISTRICTS
            .iter()
            .position(|d| d.name == "The Forbidden Library")
            .expect("the Library district");
        for &(d, art) in PENDING_LANDMARKS {
            assert_eq!(d, library, "{art}");
            let base = crate::plugins::world::back_landmark_base(d)
                .unwrap_or_else(|| panic!("{art}: no back landmark in district {d}"));
            landmark_plot_is_sound(d, base, art, None);
        }
    }

    /// The Throne Dais (REF-96) is spawned exactly once, ON the Throne
    /// plaza at its back-centre with the Compiler centred in front of it,
    /// as a solid, y-sorted 64x64 prop; the player can stand just in front
    /// of its bottom step (drawn in front) and behind the throne (it fades).
    #[test]
    fn the_throne_dais_stands_at_its_reserved_spot() {
        let dais: Vec<Vec2> = all_props()
            .into_iter()
            .filter(|(k, _)| *k == PropKind::ThroneDais)
            .map(|(_, b)| b)
            .collect();
        let throne = DISTRICTS
            .iter()
            .position(|d| d.name == "The Throne")
            .expect("the Throne district");
        assert_eq!(LANDMARKS, &[(throne, PropKind::ThroneDais)]);
        let base = crate::plugins::world::back_landmark_base(throne).expect("spot");
        assert_eq!(dais, vec![base], "one dais, at the back-centre");
        landmark_plot_is_sound(throne, base, "the Throne Dais", Some(base));
        assert!(
            PENDING_LANDMARKS.iter().all(|(d, ..)| *d != throne),
            "the Throne's spot is no longer pending"
        );
        let k = PropKind::ThroneDais;
        assert_eq!(k.size(), Vec2::new(64.0, 64.0));
        assert_eq!(k.sprite_path(), crate::assets::SPRITE_PROP_THRONE_DAIS);
        let solid = k.solid_at(base);
        let prop_z = ysort_z(base.y + k.ground_offset());
        let front = Vec2::new(base.x, solid.min.y - 6.0);
        assert!(!occludes(
            player_body_at_feet(front),
            ysort_z(front.y),
            k.visual_at(base),
            prop_z
        ));
        let behind = Vec2::new(base.x, solid.max.y);
        assert!(occludes(
            player_body_at_feet(behind),
            ysort_z(behind.y),
            k.visual_at(base),
            prop_z
        ));
    }

    #[test]
    fn player_spawn_is_clear() {
        let foot = Aabb::from_base(0.0, 6.0, -16.0, -10.0);
        for (kind, base) in all_props() {
            assert!(!foot.overlaps(&kind.solid_at(base)));
        }
    }
}
