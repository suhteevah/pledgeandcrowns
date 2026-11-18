// SPDX-License-Identifier: MIT
//! Village props: oaks (REF-67, 32x48), cottages (REF-68, 64x64), the
//! district landmarks (the Sealed Stacks, REF-95, the Throne Dais, REF-96,
//! and the Great Furnace, REF-98, all 64x64) and the district yard props
//! (the Forge's anvil, quench trough, coal heap and tool rack, REF-99..102,
//! 32x32).
//!
//! Props are ordinary sprites, not tiles. Each one is anchored bottom-centre
//! at its ground line, y-sorted (lower on screen = drawn in front) and owns
//! solid [`Aabb`] footprints ([`PropKind::solids_local`], one box for most,
//! two for the tool rack's feet) that `player.rs` refuses to walk into.
//!
//! Placement is data: [`COTTAGE_SPOTS`], [`MEADOW_OAKS`], [`LANDMARKS`] and
//! [`DISTRICT_PROPS`] are hand-picked const tables, and the border treeline
//! is a deterministic function of the map size (fixed spacing + hash
//! jitter), so every run builds the same village. Dressing another district
//! is a data change: name its ground in `world::DISTRICTS`, add its landmark
//! to [`LANDMARKS`], its yard to [`DISTRICT_PROPS`] and its cast spots to
//! the district. `tests` below check every prop sits on grass (or, for a
//! district's own props, on its plaza paving), clear of roads, street
//! mouths, NPC slots and each other, and that every talk spot is reachable.
//!
//! The same y-sort system also orders the player and NPC sprites, by querying
//! their existing marker components (`Player`, `Npc`) — npc.rs is untouched.
//!
//! Occlusion fade: a roof or canopy that y-sorts in front of a character can
//! hide it completely. [`fade_occluders`] fades any prop whose opaque art
//! ([`PropKind::visual_local`]) overlaps a character drawn behind it down to
//! [`OCCLUDER_ALPHA`] over [`FADE_SECS`], and back to opaque once clear. The
//! decision ([`occludes`]) and the ramp ([`step_alpha`]) are pure functions.

use crate::assets::{
    SPRITE_PROP_ANVIL, SPRITE_PROP_COAL_HEAP, SPRITE_PROP_COTTAGE, SPRITE_PROP_GREAT_FURNACE,
    SPRITE_PROP_OAK, SPRITE_PROP_QUENCH_TROUGH, SPRITE_PROP_SEALED_STACKS, SPRITE_PROP_THRONE_DAIS,
    SPRITE_PROP_TOOL_RACK,
};
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

    /// Smallest box containing both.
    pub fn union(self, other: Aabb) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
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
    /// The Sealed Stacks (REF-95, approved 2026-11-16), the Act 9 district
    /// landmark: 64x64, placed at [`LANDMARKS`].
    SealedStacks,
    /// The Great Furnace (REF-98, approved 2026-11-18), the Act 6 district
    /// landmark: 64x64, placed at [`LANDMARKS`].
    GreatFurnace,
    /// Forge yard props (REF-99..102, approved 2026-11-18), 32x32 each,
    /// placed by [`DISTRICT_PROPS`].
    Anvil,
    QuenchTrough,
    CoalHeap,
    ToolRack,
}

// Solid footprints, relative to the bottom-centre anchor. A ref grid of
// width W and height H maps column `c` to x = c - W/2 (its left edge) and
// row `r` to y = H - 1 - r (its bottom edge); `tests` re-derive every box
// below from the ref grids.
const OAK_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 6.0, 3.0, 11.0)];
const COTTAGE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 27.0, 5.0, 27.0)];
const THRONE_DAIS_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 30.0, 3.0, 20.0)];
const SEALED_STACKS_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 26.0, 3.0, 62.0)];
/// REF-98 "solid from the plinth to the hood": rows 20..61 (hood top to
/// plinth bottom), cols 7..56 (the plinth, the widest part).
const GREAT_FURNACE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 25.0, 2.0, 44.0)];
/// REF-99 "solid over the stump's footprint": the stump, rows 19..30,
/// cols 7..24. The anvil on top overhangs west (the horn) and is not solid.
const ANVIL_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 9.0, 1.0, 13.0)];
/// REF-100 "solid over the whole trough": rows 14..29, cols 4..27.
const QUENCH_TROUGH_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 12.0, 2.0, 18.0)];
/// REF-101 "solid over the mound": rows 16..28, cols 2..29.
const COAL_HEAP_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 14.0, 3.0, 16.0)];
/// REF-102 "solid only at the two feet": rows 28..30, cols 4..8 and
/// 23..27. The 14 px between them stays open (the ground shows through).
const TOOL_RACK_SOLIDS: [Aabb; 2] = [
    Aabb {
        min: Vec2::new(-12.0, 1.0),
        max: Vec2::new(-7.0, 4.0),
    },
    Aabb {
        min: Vec2::new(7.0, 1.0),
        max: Vec2::new(12.0, 4.0),
    },
];

impl PropKind {
    pub fn sprite_path(self) -> &'static str {
        match self {
            PropKind::Oak => SPRITE_PROP_OAK,
            PropKind::Cottage => SPRITE_PROP_COTTAGE,
            PropKind::ThroneDais => SPRITE_PROP_THRONE_DAIS,
            PropKind::SealedStacks => SPRITE_PROP_SEALED_STACKS,
            PropKind::GreatFurnace => SPRITE_PROP_GREAT_FURNACE,
            PropKind::Anvil => SPRITE_PROP_ANVIL,
            PropKind::QuenchTrough => SPRITE_PROP_QUENCH_TROUGH,
            PropKind::CoalHeap => SPRITE_PROP_COAL_HEAP,
            PropKind::ToolRack => SPRITE_PROP_TOOL_RACK,
        }
    }

    /// Native sprite size in pixels (displayed at 1x, like characters).
    pub fn size(self) -> Vec2 {
        match self {
            PropKind::Oak => Vec2::new(32.0, 48.0),
            PropKind::Cottage
            | PropKind::ThroneDais
            | PropKind::SealedStacks
            | PropKind::GreatFurnace => Vec2::new(64.0, 64.0),
            PropKind::Anvil | PropKind::QuenchTrough | PropKind::CoalHeap | PropKind::ToolRack => {
                Vec2::new(32.0, 32.0)
            }
        }
    }

    /// Ground line above the sprite's bottom edge (the art has empty rows
    /// below the trunk / footing). Used as the y-sort point so a player
    /// standing just in front of a wall sorts in front of it.
    pub fn ground_offset(self) -> f32 {
        match self {
            PropKind::Oak => 3.0,          // REF-67 row 44 is the trunk base
            PropKind::Cottage => 5.0,      // REF-68 row 58 is the footing
            PropKind::ThroneDais => 3.0,   // REF-96 row 60 is the bottom step's edge
            PropKind::SealedStacks => 3.0, // REF-95 row 60 is the threshold's edge
            PropKind::GreatFurnace => 2.0, // REF-98 row 61 is the plinth's bottom edge
            PropKind::Anvil => 1.0,        // REF-99 row 30 is the stump's bottom edge
            PropKind::QuenchTrough => 2.0, // REF-100 row 29 is the feet's bottom
            PropKind::CoalHeap => 3.0,     // REF-101 row 28 is the mound's bottom edge
            PropKind::ToolRack => 1.0,     // REF-102 row 30 is the feet's bottom edge
        }
    }

    /// Every solid part of the footprint, relative to the anchor
    /// (bottom-centre). One box for every kind but the tool rack, which is
    /// solid only at its two feet. These are what collide.
    pub fn solids_local(self) -> &'static [Aabb] {
        match self {
            PropKind::Oak => &OAK_SOLID,
            PropKind::Cottage => &COTTAGE_SOLID,
            PropKind::ThroneDais => &THRONE_DAIS_SOLID,
            PropKind::SealedStacks => &SEALED_STACKS_SOLID,
            PropKind::GreatFurnace => &GREAT_FURNACE_SOLID,
            PropKind::Anvil => &ANVIL_SOLID,
            PropKind::QuenchTrough => &QUENCH_TROUGH_SOLID,
            PropKind::CoalHeap => &COAL_HEAP_SOLID,
            PropKind::ToolRack => &TOOL_RACK_SOLIDS,
        }
    }

    /// The solid parts of a prop anchored at `base`, in world space.
    pub fn solids_at(self, base: Vec2) -> impl Iterator<Item = Aabb> {
        self.solids_local().iter().map(move |s| s.translated(base))
    }

    /// Bounding box of the solid footprint relative to the anchor
    /// (bottom-centre): for a one-box kind, the box itself.
    /// Oak: the trunk base (REF-67 cols 10..21, rows 36..44).
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
    /// Sealed Stacks: the whole tower (REF-95 cols 6..57, rows 2..60 =
    /// 52x59, merlons to threshold). It is one stone face with no roof to
    /// walk behind, so the player goes round it, never into it; the six
    /// empty columns either side and the empty rows above and below stay
    /// walkable.
    /// Great Furnace, anvil, quench trough, coal heap: see the consts above.
    /// Tool rack: the box round both feet; only the feet collide
    /// ([`PropKind::solids_local`]).
    pub fn solid_local(self) -> Aabb {
        let parts = self.solids_local();
        parts
            .iter()
            .skip(1)
            .fold(parts[0], |acc, part| acc.union(*part))
    }

    /// [`PropKind::solid_local`] for a prop anchored at `base`. A bounding
    /// box: collide against [`PropKind::solids_at`].
    pub fn solid_at(self, base: Vec2) -> Aabb {
        self.solid_local().translated(base)
    }

    /// Opaque bounds of the art relative to the anchor, used for the
    /// occlusion fade. Oak: REF-67 cols 2..29, rows 1..44. Cottage: REF-68
    /// cols 2..61 (the eave), rows 0..58 (chimney top to footing). Throne
    /// Dais: REF-96 cols 2..61 (the bottom step), rows 0..60 (finial to the
    /// bottom step). Sealed Stacks: REF-95 cols 6..57, rows 2..60, the same
    /// box as its solid, so no character ever stands behind it. Great
    /// Furnace: REF-98 cols 7..56, rows 0..61 (chimney cap to plinth), so
    /// the chimney fades over a player behind the hood. Anvil: REF-99 cols
    /// 4..24, rows 9..30 (the horn overhangs west). Quench trough and coal
    /// heap: their solid. Tool rack: REF-102 cols 4..27, rows 4..30 (beam
    /// to feet).
    pub fn visual_local(self) -> Aabb {
        match self {
            PropKind::Oak => Aabb::from_base(0.0, 14.0, 3.0, 47.0),
            PropKind::Cottage => Aabb::from_base(0.0, 30.0, 5.0, 64.0),
            PropKind::ThroneDais => Aabb::from_base(0.0, 30.0, 3.0, 64.0),
            PropKind::SealedStacks => Aabb::from_base(0.0, 26.0, 3.0, 62.0),
            PropKind::GreatFurnace => Aabb::from_base(0.0, 25.0, 2.0, 64.0),
            PropKind::Anvil => Aabb {
                min: Vec2::new(-12.0, 1.0),
                max: Vec2::new(9.0, 23.0),
            },
            PropKind::QuenchTrough => QUENCH_TROUGH_SOLID[0],
            PropKind::CoalHeap => COAL_HEAP_SOLID[0],
            PropKind::ToolRack => Aabb::from_base(0.0, 12.0, 1.0, 28.0),
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
    // the plaza itself (`LANDMARKS`).
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
/// [`world::landmark_plot_base`](crate::plugins::world::landmark_plot_base).
/// In the boss districts the boss stands centred directly in front of it:
/// the Sealed Stacks (REF-95, approved by Matt 2026-11-16) stand behind the
/// Librarian, the Throne Dais (REF-96, approved by Matt 2026-11-15) behind
/// the Compiler. The Great Furnace (REF-98, approved by Matt 2026-11-18)
/// heads the Forge yard, the Smelter at its mouth.
pub const LANDMARKS: &[(usize, PropKind)] = &[
    (9, PropKind::SealedStacks),
    (10, PropKind::ThroneDais),
    (6, PropKind::GreatFurnace),
];

/// A district's own yard props: `(district index, kind, offset of the
/// bottom-centre anchor from the plaza centre)`. They stand ON the plaza
/// paving, clear of its street mouths and of the routes to every NPC (see
/// `tests`). Approved sets so far: the Forge (REF-99..102, approved by Matt
/// 2026-11-18), composed round the Great Furnace (its plot's base is at
/// (0, 156)): anvil and quench trough either side of the furnace mouth,
/// the coal heap to the furnace's west, the tool rack against the back
/// wall to its east.
pub const DISTRICT_PROPS: &[(usize, PropKind, Vec2)] = &[
    (6, PropKind::Anvil, Vec2::new(-72.0, 92.0)),
    (6, PropKind::QuenchTrough, Vec2::new(72.0, 88.0)),
    (6, PropKind::CoalHeap, Vec2::new(-96.0, 168.0)),
    (6, PropKind::ToolRack, Vec2::new(104.0, 196.0)),
];

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
        match crate::plugins::world::landmark_plot_base(d) {
            Some(base) => out.push((kind, base)),
            None => tracing::warn!(?kind, "landmark references unknown district {d}; skipped"),
        }
    }
    for &(d, kind, offset) in DISTRICT_PROPS {
        match DISTRICTS.get(d) {
            Some(district) => out.push((kind, district.center + offset)),
            None => tracing::warn!(
                ?kind,
                "district prop references unknown district {d}; skipped"
            ),
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
    let mut yard = 0usize;
    let mut extra_solids = 0usize;
    for (kind, base) in &props {
        match kind {
            PropKind::Oak => oaks += 1,
            PropKind::Cottage => cottages += 1,
            PropKind::ThroneDais | PropKind::SealedStacks | PropKind::GreatFurnace => {
                landmarks += 1
            }
            PropKind::Anvil | PropKind::QuenchTrough | PropKind::CoalHeap | PropKind::ToolRack => {
                yard += 1
            }
        }
        tracing::trace!(?kind, x = base.x, y = base.y, "spawning prop");
        let mut solids = kind.solids_at(*base);
        let Some(first) = solids.next() else {
            tracing::warn!(?kind, "prop kind has no solid footprint; skipped");
            continue;
        };
        commands.spawn((
            Sprite {
                image: asset_server.load(kind.sprite_path()),
                custom_size: Some(kind.size()),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(base.x, base.y, ysort_z(base.y + kind.ground_offset())),
            Prop { kind: *kind },
            Solid(first),
        ));
        // Further solid parts (the tool rack's second foot) are bare
        // collider entities: `player::move_player` reads every `Solid`.
        for part in solids {
            tracing::trace!(?kind, ?part, "spawning extra prop collider");
            commands.spawn(Solid(part));
            extra_solids += 1;
        }
    }
    tracing::info!(
        "spawned {} props ({oaks} oaks, {cottages} cottages, {landmarks} landmarks, \
         {yard} yard props, {extra_solids} extra colliders)",
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
        for k in ALL_KINDS {
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

    /// A landmark `kind`'s plot at `base` in district `d` (64x64,
    /// bottom-centre):
    /// on flagstone with a clear tile to the cobble ring on every side
    /// (the sprite grown by 16 px is still all flagstone), centred on the
    /// plaza, at its back (north) half, clear of every other prop and of
    /// every NPC sprite; the district's boss (its last roster slot,
    /// 64 px) stands centred directly in front of it, his sprite top at
    /// most 16 px below the plot and never overlapping it; and there is
    /// room to walk round it (two tiles of flagstone either side of its
    /// solid footprint).
    fn landmark_plot_is_sound(d: usize, kind: PropKind, base: Vec2, what: &str) {
        let dist = &DISTRICTS[d];
        let plot = kind.sprite_rect_at(base);
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
            if (k, b) == (kind, base) {
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
        let solid = kind.solid_at(base);
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
            if let Some(d) = plaza_prop_district(kind, base) {
                // Landmarks and district props stand ON their plaza (its
                // interior paving, clear of the cobble ring); see
                // `landmark_plot_is_sound` and `district_props_stand_on_their_plaza`.
                assert!(
                    rect_all(&r, DISTRICTS[d].interior_ground()),
                    "{kind:?} at {base:?} leaves {}'s paving",
                    DISTRICTS[d].name
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

    /// The Sealed Stacks (REF-95) are spawned exactly once, ON the Library
    /// plaza at its back-centre with the Librarian centred in front of
    /// them, as a solid, y-sorted 64x64 prop. The tower is solid from
    /// threshold to merlons, so a player in front of it is drawn in front
    /// and none is ever behind it. The main street meets the plaza at its
    /// north edge, right behind the tower: a player coming down the street
    /// steps aside round the tower and walks on to the Librarian's talk
    /// spot, which is free.
    #[test]
    fn the_sealed_stacks_stand_behind_the_librarian() {
        use crate::plugins::npc::TALK_SPOT_OFFSET;
        use crate::plugins::player::PLAYER_FOOT;
        let stacks: Vec<Vec2> = all_props()
            .into_iter()
            .filter(|(k, _)| *k == PropKind::SealedStacks)
            .map(|(_, b)| b)
            .collect();
        let library = DISTRICTS
            .iter()
            .position(|d| d.name == "The Forbidden Library")
            .expect("the Library district");
        assert!(LANDMARKS.contains(&(library, PropKind::SealedStacks)));
        let base = crate::plugins::world::back_landmark_base(library).expect("spot");
        assert_eq!(stacks, vec![base], "one tower, at the back-centre");
        let k = PropKind::SealedStacks;
        landmark_plot_is_sound(library, k, base, "the Sealed Stacks");
        assert_eq!(k.size(), Vec2::new(64.0, 64.0));
        assert_eq!(k.sprite_path(), crate::assets::SPRITE_PROP_SEALED_STACKS);
        let solid = k.solid_at(base);
        assert_eq!(k.visual_at(base), solid, "nothing to stand behind");
        let prop_z = ysort_z(base.y + k.ground_offset());
        let front = Vec2::new(base.x, solid.min.y - 6.0);
        assert!(!occludes(
            player_body_at_feet(front),
            ysort_z(front.y),
            k.visual_at(base),
            prop_z
        ));

        // The Librarian's sprite is clear of the tower and his talk spot
        // of every solid.
        let first: usize = DISTRICTS[..library].iter().map(|x| x.count).sum();
        let boss = first + DISTRICTS[library].count - 1;
        assert_eq!(NPC_ROSTER[boss].name, "The Librarian");
        let librarian = district_slot(boss);
        assert!(
            librarian.y + 32.0 <= solid.min.y,
            "the Librarian is under it"
        );
        let talk = librarian + TALK_SPOT_OFFSET;
        let solids = every_solid();
        assert!(
            solids
                .iter()
                .all(|s| !PLAYER_FOOT.translated(talk).overlaps(s)),
            "the Librarian's talk spot is blocked"
        );

        // In from the street: from the street north of the plaza down to
        // the tower, a step aside (west, then east), and on south to the
        // talk spot. Every leg arrives where it aims.
        let dist = &DISTRICTS[library];
        let street = Vec2::new(base.x, dist.center.y + dist.plaza_half + 64.0);
        let up_against = solid.max.y - PLAYER_FOOT.min.y;
        let clear_y = up_against + 2.0;
        for dx in [-1.0, 1.0] {
            let aside = base.x + dx * (solid.max.x - solid.min.x);
            let legs = [
                Vec2::new(base.x, clear_y),
                Vec2::new(aside, clear_y),
                Vec2::new(aside, talk.y),
                talk,
            ];
            let mut p = street;
            for to in legs {
                p = slide_move(p, to - p, PLAYER_FOOT, &solids);
                assert!(p.distance(to) < 0.01, "stopped at {p:?} heading for {to:?}");
            }
        }
        // Straight down the street the tower stops the player: it is solid.
        let p = slide_move(
            street,
            Vec2::new(0.0, talk.y - street.y),
            PLAYER_FOOT,
            &solids,
        );
        assert!(p.y >= up_against - 0.01, "walked into the tower: {p:?}");
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
        assert!(LANDMARKS.contains(&(throne, PropKind::ThroneDais)));
        let base = crate::plugins::world::back_landmark_base(throne).expect("spot");
        assert_eq!(dais, vec![base], "one dais, at the back-centre");
        landmark_plot_is_sound(throne, PropKind::ThroneDais, base, "the Throne Dais");
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

    // --- District dressing (the Forge, R1 wave A1) -------------------------

    const ALL_KINDS: [PropKind; 9] = [
        PropKind::Oak,
        PropKind::Cottage,
        PropKind::ThroneDais,
        PropKind::SealedStacks,
        PropKind::GreatFurnace,
        PropKind::Anvil,
        PropKind::QuenchTrough,
        PropKind::CoalHeap,
        PropKind::ToolRack,
    ];

    /// Every solid part of every prop, as `player::move_player` sees them.
    fn every_solid() -> Vec<Aabb> {
        all_props()
            .iter()
            .flat_map(|(k, b)| k.solids_at(*b))
            .collect()
    }

    /// The district a landmark or district prop at `base` stands in, or
    /// `None` for the village props (oaks, cottages) that stand on grass.
    fn plaza_prop_district(kind: PropKind, base: Vec2) -> Option<usize> {
        let landmark = LANDMARKS
            .iter()
            .find(|(d, k)| {
                *k == kind && crate::plugins::world::landmark_plot_base(*d) == Some(base)
            })
            .map(|(d, _)| *d);
        let yard = DISTRICT_PROPS
            .iter()
            .find(|(d, k, off)| *k == kind && DISTRICTS[*d].center + *off == base)
            .map(|(d, _, _)| *d);
        landmark.or(yard)
    }

    /// Districts dressed with a landmark or yard props.
    fn dressed_districts() -> Vec<usize> {
        let mut out: Vec<usize> = LANDMARKS
            .iter()
            .map(|(d, _)| *d)
            .chain(DISTRICT_PROPS.iter().map(|(d, _, _)| *d))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Box over the opaque pixels of `rows` (and `cols`, if given) of a
    /// ref grid, in anchor-relative coordinates: column `c` is x = c - W/2,
    /// row `r` is y = H - 1 - r (bottom edges).
    fn grid_box(
        src: &str,
        rows: std::ops::RangeInclusive<usize>,
        cols: Option<std::ops::RangeInclusive<usize>>,
    ) -> Aabb {
        let grid: Vec<&str> = src
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix('\'')?.split('\'').next())
            .collect();
        let (w, h) = (grid[0].len() as f32, grid.len() as f32);
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for r in rows {
            for (c, ch) in grid[r].chars().enumerate() {
                if ch == '.' || cols.as_ref().is_some_and(|cs| !cs.contains(&c)) {
                    continue;
                }
                let x = c as f32 - w * 0.5;
                let y = h - 1.0 - r as f32;
                min = min.min(Vec2::new(x, y));
                max = max.max(Vec2::new(x + 1.0, y + 1.0));
            }
        }
        Aabb { min, max }
    }

    /// Each Forge footprint is exactly what its ref header says is solid,
    /// re-derived from the approved pixel grid; the visual box is the art's
    /// opaque bounds; the ground line is the art's bottom row.
    #[test]
    fn forge_footprints_match_their_ref_headers() {
        const R98: &str = include_str!("../../../design/art/refs/ref-98-greatfurnace.jsx");
        const R99: &str = include_str!("../../../design/art/refs/ref-99-anvil.jsx");
        const R100: &str = include_str!("../../../design/art/refs/ref-100-quenchtrough.jsx");
        const R101: &str = include_str!("../../../design/art/refs/ref-101-coalheap.jsx");
        const R102: &str = include_str!("../../../design/art/refs/ref-102-toolrack.jsx");
        assert!(R98.contains("Solid from the plinth to the hood."));
        assert!(R99.contains("over the stump's footprint"));
        assert!(R100.contains("solid over the whole trough"));
        assert!(R101.contains("solid over the mound"));
        assert!(R102.contains("solid only at the two feet"));
        // (kind, ref, solid parts as (rows, cols), visual rows)
        type Part = (
            std::ops::RangeInclusive<usize>,
            Option<std::ops::RangeInclusive<usize>>,
        );
        let cases: [(PropKind, &str, Vec<Part>, std::ops::RangeInclusive<usize>); 5] = [
            // Hood top (row 20) to plinth bottom (row 61).
            (PropKind::GreatFurnace, R98, vec![(20..=61, None)], 0..=61),
            // The stump: rows 19 (cut top outline) to 30.
            (PropKind::Anvil, R99, vec![(19..=30, None)], 9..=30),
            (PropKind::QuenchTrough, R100, vec![(14..=29, None)], 14..=29),
            (PropKind::CoalHeap, R101, vec![(16..=28, None)], 16..=28),
            // The two feet, rows 28..30, left and right halves.
            (
                PropKind::ToolRack,
                R102,
                vec![(28..=30, Some(0..=15)), (28..=30, Some(16..=31))],
                4..=30,
            ),
        ];
        for (kind, src, parts, visual) in cases {
            let want: Vec<Aabb> = parts
                .into_iter()
                .map(|(rows, cols)| grid_box(src, rows, cols))
                .collect();
            assert_eq!(kind.solids_local(), want.as_slice(), "{kind:?} solid");
            assert_eq!(
                kind.visual_local(),
                grid_box(src, visual.clone(), None),
                "{kind:?} visual"
            );
            let opaque = grid_box(src, 0..=kind.size().y as usize - 1, None);
            assert_eq!(kind.ground_offset(), opaque.min.y, "{kind:?} ground line");
        }
        // The tool rack is open between its feet: a player's feet fit there.
        let rack = PropKind::ToolRack.solids_local();
        assert_eq!(rack.len(), 2);
        let gap = rack[1].min.x - rack[0].max.x;
        let foot = crate::plugins::player::PLAYER_FOOT;
        assert!(
            gap > foot.max.x - foot.min.x,
            "the rack's feet leave {gap} px"
        );
        assert_eq!(
            PropKind::ToolRack.solid_local(),
            rack[0].union(rack[1]),
            "bounding box of both feet"
        );
    }

    /// The Great Furnace is spawned exactly once, on the Forge's landmark
    /// plot at the back-centre, on clear slab, with room to walk round it.
    /// A player in front of its plinth draws in front; behind the hood the
    /// chimney fades over them. The Smelter stands at its mouth.
    #[test]
    fn the_great_furnace_heads_the_forge_yard() {
        let forge = DISTRICTS
            .iter()
            .position(|d| d.name == "The Forge")
            .expect("the Forge");
        assert!(LANDMARKS.contains(&(forge, PropKind::GreatFurnace)));
        let base = crate::plugins::world::landmark_plot_base(forge).expect("plot");
        let k = PropKind::GreatFurnace;
        let furnaces: Vec<Vec2> = all_props()
            .into_iter()
            .filter(|(kk, _)| *kk == k)
            .map(|(_, b)| b)
            .collect();
        assert_eq!(furnaces, vec![base], "one furnace, at the back-centre");
        assert_eq!(k.size(), Vec2::new(64.0, 64.0));
        assert_eq!(k.sprite_path(), crate::assets::SPRITE_PROP_GREAT_FURNACE);
        let plot = k.sprite_rect_at(base);
        // One clear tile of slab all round (grown a hair under 16 px: the
        // Forge's interior ends exactly one tile above the plot, and a
        // sample on that tile edge would round into the cobble ring).
        let grown = Aabb {
            min: plot.min - Vec2::splat(15.0),
            max: plot.max + Vec2::splat(15.0),
        };
        assert!(rect_all(&grown, Ground::ForgeSlab), "not on clear slab");
        for lane in mouth_lanes(&DISTRICTS[forge]) {
            assert!(!plot.overlaps(&lane), "the furnace blocks the street");
        }
        let solid = k.solid_at(base);
        for side in [
            Aabb::from_base(solid.min.x - 16.0, 16.0, solid.min.y, solid.max.y),
            Aabb::from_base(solid.max.x + 16.0, 16.0, solid.min.y, solid.max.y),
        ] {
            assert!(
                rect_all(&side, Ground::ForgeSlab),
                "no way round the furnace"
            );
        }
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
        // The Smelter is at the furnace mouth: centred on it, in front.
        let smelter = NPC_ROSTER
            .iter()
            .position(|s| s.name == "The Smelter")
            .expect("the Smelter");
        let p = district_slot(smelter);
        assert_eq!(p.x, base.x, "the Smelter faces the mouth");
        let top = p.y + NPC_ROSTER[smelter].native_px * 0.5;
        assert!(
            top <= plot.min.y && plot.min.y - top <= 16.0,
            "Smelter top {top}"
        );
    }

    /// Every district prop is spawned once at its plaza centre + offset,
    /// its whole sprite on that plaza's interior paving (never on the
    /// cobble ring; the tool rack stands against the back wall), and no
    /// two props' sprites overlap.
    #[test]
    fn district_props_stand_on_their_plaza() {
        let props = all_props();
        for &(d, kind, offset) in DISTRICT_PROPS {
            let dist = &DISTRICTS[d];
            let base = dist.center + offset;
            assert_eq!(
                props.iter().filter(|p| **p == (kind, base)).count(),
                1,
                "{kind:?} in {}",
                dist.name
            );
            let r = kind.sprite_rect_at(base);
            assert!(
                rect_all(&r, dist.interior_ground()),
                "{kind:?} in {} is not on its {:?}",
                dist.name,
                dist.interior_ground()
            );
            for (k, b) in &props {
                if (*k, *b) != (kind, base) {
                    assert!(
                        !k.sprite_rect_at(*b).overlaps(&r),
                        "{kind:?} overlaps the {k:?} at {b:?}"
                    );
                }
            }
        }
        // The Forge yard is the four approved REF-99..102 props.
        let forge: Vec<PropKind> = DISTRICT_PROPS
            .iter()
            .filter(|(d, _, _)| DISTRICTS[*d].name == "The Forge")
            .map(|(_, k, _)| *k)
            .collect();
        assert_eq!(
            forge,
            [
                PropKind::Anvil,
                PropKind::QuenchTrough,
                PropKind::CoalHeap,
                PropKind::ToolRack
            ]
        );
    }

    /// Points just inside `d`'s plaza edge where a street meets it (the
    /// ground one tile outside is dirt), every 4 px.
    fn street_mouths(d: &crate::plugins::world::District) -> Vec<Vec2> {
        let h = d.plaza_half;
        let mut out = Vec::new();
        let mut s = -h;
        while s <= h {
            for (inside, outside) in [
                (Vec2::new(s, h - 8.0), Vec2::new(s, h + 8.0)),
                (Vec2::new(s, -h + 8.0), Vec2::new(s, -h - 8.0)),
                (Vec2::new(-h + 8.0, s), Vec2::new(-h - 8.0, s)),
                (Vec2::new(h - 8.0, s), Vec2::new(h + 8.0, s)),
            ] {
                if ground_at(d.center + outside) == Ground::Dirt {
                    out.push(d.center + inside);
                }
            }
            s += 4.0;
        }
        out
    }

    /// The lanes in from `d`'s streets: from each street mouth point, four
    /// tiles straight into the plaza, half a tile either side.
    fn mouth_lanes(d: &crate::plugins::world::District) -> Vec<Aabb> {
        street_mouths(d)
            .into_iter()
            .map(|m| {
                let inward = (d.center - m).normalize().round() * 64.0;
                Aabb {
                    min: m.min(m + inward) - Vec2::splat(8.0),
                    max: m.max(m + inward) + Vec2::splat(8.0),
                }
            })
            .collect()
    }

    /// No district prop blocks a street mouth: no yard prop's sprite within
    /// four tiles inward of where a street meets its plaza. (A back-centre
    /// landmark may face a street on purpose: the Sealed Stacks stand
    /// across the main street's end and the player steps round them, see
    /// `the_sealed_stacks_stand_behind_the_librarian`; the Great Furnace
    /// keeps clear of the Forge's east street.)
    #[test]
    fn district_props_keep_the_street_mouths_open() {
        for d in dressed_districts() {
            let dist = &DISTRICTS[d];
            let lanes = mouth_lanes(dist);
            assert!(!lanes.is_empty(), "{} has no street", dist.name);
            for &(pd, k, off) in DISTRICT_PROPS {
                if pd != d {
                    continue;
                }
                let r = k.sprite_rect_at(dist.center + off);
                for lane in &lanes {
                    assert!(
                        !r.overlaps(lane),
                        "the {k:?} blocks {}'s street lane {lane:?}",
                        dist.name
                    );
                }
            }
        }
    }

    /// A set of 4 px lattice points, as offsets from a plaza centre.
    type Lattice = std::collections::HashSet<(i32, i32)>;

    /// Flood-fill a player's feet over a 4 px lattice covering `d`'s plaza
    /// (ring included), from every street mouth, against `solids`. Returns
    /// (reached, free) lattice points as offsets from the plaza centre.
    fn flood_plaza(d: &crate::plugins::world::District, solids: &[Aabb]) -> (Lattice, Lattice) {
        use crate::plugins::player::PLAYER_FOOT;
        use std::collections::{HashSet, VecDeque};
        let n = (d.plaza_half / 4.0).floor() as i32;
        let at = |i: i32, j: i32| d.center + Vec2::new(i as f32, j as f32) * 4.0;
        let near: Vec<Aabb> = solids
            .iter()
            .filter(|s| {
                s.overlaps(&Aabb::from_base(
                    d.center.x,
                    d.plaza_half + 32.0,
                    d.center.y - d.plaza_half - 32.0,
                    d.center.y + d.plaza_half + 32.0,
                ))
            })
            .copied()
            .collect();
        let free_at = |i: i32, j: i32| {
            let foot = PLAYER_FOOT.translated(at(i, j));
            !near.iter().any(|s| foot.overlaps(s))
        };
        let free: HashSet<(i32, i32)> = (-n..=n)
            .flat_map(|i| (-n..=n).map(move |j| (i, j)))
            .filter(|&(i, j)| free_at(i, j))
            .collect();
        let mut seen: HashSet<(i32, i32)> = HashSet::new();
        let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
        for m in street_mouths(d) {
            let o = ((m - d.center) / 4.0).round();
            let cell = (o.x as i32, o.y as i32);
            if free.contains(&cell) && seen.insert(cell) {
                queue.push_back(cell);
            }
        }
        while let Some((i, j)) = queue.pop_front() {
            for next in [(i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1)] {
                if free.contains(&next) && seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        (seen, free)
    }

    /// In every dressed district the whole plaza floor not under a solid is
    /// one walkable region reached from its streets (so no pocket a saved
    /// position could restore into), and every NPC's talk spot is in it:
    /// clear of every solid and walked to from the nearest lattice point.
    #[test]
    fn every_talk_spot_in_a_dressed_district_is_reachable() {
        use crate::plugins::npc::TALK_SPOT_OFFSET;
        use crate::plugins::player::PLAYER_FOOT;
        let solids = every_solid();
        for d in dressed_districts() {
            let dist = &DISTRICTS[d];
            let (reached, free) = flood_plaza(dist, &solids);
            let mut pockets: Vec<&(i32, i32)> = free.difference(&reached).collect();
            pockets.sort_unstable();
            assert!(
                pockets.is_empty(),
                "{}: {} free spots cut off, e.g. {:?}",
                dist.name,
                pockets.len(),
                pockets.iter().take(5).collect::<Vec<_>>()
            );
            let first: usize = DISTRICTS[..d].iter().map(|x| x.count).sum();
            for (i, spec) in NPC_ROSTER.iter().enumerate().skip(first).take(dist.count) {
                let name = spec.name;
                let talk = district_slot(i) + TALK_SPOT_OFFSET;
                assert!(
                    solids
                        .iter()
                        .all(|s| !PLAYER_FOOT.translated(talk).overlaps(s)),
                    "{name}'s talk spot {talk:?} is inside a solid"
                );
                let o = ((talk - dist.center) / 4.0).round();
                let from = (o.x as i32, o.y as i32);
                assert!(reached.contains(&from), "{name}: talk spot not reached");
                let start = dist.center + Vec2::new(from.0 as f32, from.1 as f32) * 4.0;
                let end = slide_move(start, talk - start, PLAYER_FOOT, &solids);
                assert!(end.distance(talk) < 0.01, "{name}: stopped at {end:?}");
            }
        }
    }

    /// The tool rack collides only at its feet: walking into a foot stops
    /// the player, walking between the feet does not.
    #[test]
    fn the_tool_rack_is_solid_only_at_its_feet() {
        use crate::plugins::player::PLAYER_FOOT;
        let (_, base) = all_props()
            .into_iter()
            .find(|(k, _)| *k == PropKind::ToolRack)
            .expect("a tool rack");
        let solids = every_solid();
        let feet: Vec<Aabb> = PropKind::ToolRack.solids_at(base).collect();
        // From the south, straight up between the feet and on behind.
        let start = Vec2::new(base.x, base.y - 20.0);
        let to = Vec2::new(base.x, base.y + 40.0);
        let end = slide_move(start, to - start, PLAYER_FOOT, &solids);
        assert!(
            end.distance(to) < 0.01,
            "blocked between the feet at {end:?}"
        );
        // From the south, up into the west foot: stopped below it.
        let start = Vec2::new(feet[0].min.x + 2.0, base.y - 20.0);
        let end = slide_move(start, Vec2::new(0.0, 60.0), PLAYER_FOOT, &solids);
        assert!(
            PLAYER_FOOT.translated(end).max.y <= feet[0].min.y + 0.01,
            "walked through the west foot: {end:?}"
        );
    }
}
