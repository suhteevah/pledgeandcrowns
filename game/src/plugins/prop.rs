// SPDX-License-Identifier: MIT
//! Village props: oaks (REF-67, 32x48), cottages (REF-68, 64x64), the
//! district landmarks (the Sealed Stacks, REF-95, the Throne Dais, REF-96,
//! and the Great Furnace, REF-98, the Bell Tower, the market stall and the
//! Checker's Gate, the Guildhall, the Mage's Tower and the tavern, all
//! 64x64) and the district yard props (REF-99..102, REF-104..106,
//! REF-108..110, REF-112..121, REF-124..129, REF-132..137, REF-140..145,
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
    SPRITE_PROP_ANVIL, SPRITE_PROP_BARRELS, SPRITE_PROP_BELL_TOWER, SPRITE_PROP_BELLOWS,
    SPRITE_PROP_BENCH, SPRITE_PROP_BUCKET_SHOVEL, SPRITE_PROP_CHECKERS_GATE,
    SPRITE_PROP_CHICKEN_COOP, SPRITE_PROP_COAL_HEAP, SPRITE_PROP_COTTAGE, SPRITE_PROP_CRATES,
    SPRITE_PROP_FENCE, SPRITE_PROP_GREAT_FURNACE, SPRITE_PROP_GRINDSTONE, SPRITE_PROP_HANDCART,
    SPRITE_PROP_INGOTS, SPRITE_PROP_LANTERN_POST, SPRITE_PROP_MARKET_STALL, SPRITE_PROP_OAK,
    SPRITE_PROP_PLANTER, SPRITE_PROP_QUENCH_TROUGH, SPRITE_PROP_SEALED_STACKS,
    SPRITE_PROP_SIGNPOST, SPRITE_PROP_STRONGBOX, SPRITE_PROP_THRONE_DAIS, SPRITE_PROP_TOOL_RACK,
    SPRITE_PROP_WEAPON_RACK, SPRITE_PROP_WELL, SPRITE_PROP_WOODPILE, SPRITE_PROP_WORKBENCH,
};
use crate::assets::{
    SPRITE_PROP_CAULDRON, SPRITE_PROP_CRYSTAL, SPRITE_PROP_FIREPIT, SPRITE_PROP_GUILD_BANNER,
    SPRITE_PROP_GUILDHALL, SPRITE_PROP_KEG_RACK, SPRITE_PROP_LECTERN, SPRITE_PROP_LEDGER_DESK,
    SPRITE_PROP_MAGES_TOWER, SPRITE_PROP_NOTICE_BOARD, SPRITE_PROP_ORRERY, SPRITE_PROP_ROUND_TABLE,
    SPRITE_PROP_RUNE_STONE, SPRITE_PROP_SCROLL_SHELF, SPRITE_PROP_TAB_SLATE, SPRITE_PROP_TAVERN,
    SPRITE_PROP_TAVERN_SIGN, SPRITE_PROP_TELESCOPE, SPRITE_PROP_TRESTLE_TABLE,
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
    /// The Bell Tower (REF-103, approved 2026-11-18), the Hearthstone
    /// Square landmark: 64x64, placed at [`LANDMARKS`].
    BellTower,
    /// Hearthstone props (REF-104..106, REF-115, approved 2026-11-18),
    /// 32x32 each, placed by [`DISTRICT_PROPS`]. `SignpostWest` is the
    /// REF-106 signpost mirrored (its header: "mirror it for a
    /// left-pointing sign"), the same art drawn with `flip_x`.
    Well,
    Woodpile,
    Signpost,
    SignpostWest,
    ChickenCoop,
    /// The market stall (REF-107, approved 2026-11-18), the Market Row
    /// landmark: 64x64, placed at [`LANDMARKS`].
    MarketStall,
    /// Market Row props (REF-108..110, approved 2026-11-18), 32x32 each.
    Crates,
    Barrels,
    Handcart,
    /// The Checker's Gate (REF-111, approved 2026-11-18), The Commons
    /// landmark: 64x64, placed at [`LANDMARKS`], solid only at its two
    /// pillars so the player walks through the arch.
    CheckersGate,
    /// The Commons props (REF-112..114, approved 2026-11-18), 32x32 each.
    /// Fence pieces placed side by side 32 px apart join into one run.
    LanternPost,
    Bench,
    Fence,
    /// The density pass (approved 2026-11-18), 32x32 each: the Forge yard
    /// set (REF-116..121) and the strongbox (REF-128) and planter
    /// (REF-129), placed in any plaza where they fit.
    Grindstone,
    Ingots,
    Bellows,
    WeaponRack,
    Workbench,
    BucketShovel,
    Strongbox,
    Planter,
    /// The Guildhall (REF-123, approved 2026-11-18), the Guildhall Quarter
    /// landmark: 64x64, placed at [`LANDMARKS`].
    Guildhall,
    /// Guildhall Quarter props (REF-124..127, approved 2026-11-18), 32x32
    /// each. The notice board is solid only at its two posts, the banner
    /// only at its foot; the scroll shelf serves the Tower too.
    LedgerDesk,
    NoticeBoard,
    GuildBanner,
    ScrollShelf,
    /// The Mage's Tower (REF-131, approved 2026-11-18), the Act 4 district
    /// landmark: 64x64, placed at [`LANDMARKS`].
    MagesTower,
    /// The Mage's Tower props (REF-132..137, approved 2026-11-18), 32x32
    /// each. The orrery and the lectern are solid at their stands, the
    /// telescope at its tripod's feet.
    Orrery,
    RuneStone,
    Lectern,
    Crystal,
    Cauldron,
    Telescope,
    /// The tavern (REF-139, approved 2026-11-18), the Tavern Row landmark:
    /// 64x64, placed at [`LANDMARKS`].
    Tavern,
    /// Tavern Row props (REF-140..145, approved 2026-11-18), 32x32 each.
    /// The tavern sign's post is at the left of its sprite, solid at the
    /// foot; the tab slate is solid at its legs.
    TrestleTable,
    KegRack,
    Firepit,
    TavernSign,
    RoundTable,
    TabSlate,
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

/// REF-103 "solid from the base to the eave": rows 18..61 (eave to base),
/// cols 14..49 (the eave, the widest part of those rows).
const BELL_TOWER_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 18.0, 2.0, 46.0)];
/// REF-104 "solid over the ring": rows 18..29, cols 4..27.
const WELL_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 12.0, 2.0, 14.0)];
/// REF-105 "solid over the stack": rows 16..28, cols 4..27.
const WOODPILE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 12.0, 3.0, 16.0)];
/// REF-106 "solid only at the post": the post below the board, rows
/// 16..29, cols 14..17. Symmetric about the anchor, so the mirrored sign
/// shares it.
const SIGNPOST_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 2.0, 2.0, 16.0)];
/// REF-115 "solid over the coop, not the hen": rows 13..29 (eave line to
/// the legs and ramp), cols 4..24; the hen (cols 25..30) stays walkable.
const CHICKEN_COOP_SOLID: [Aabb; 1] = [Aabb {
    min: Vec2::new(-12.0, 2.0),
    max: Vec2::new(9.0, 19.0),
}];
/// REF-107 "solid from the counter's foot to the awning": rows 8..56,
/// cols 4..59.
const MARKET_STALL_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 28.0, 7.0, 56.0)];
/// REF-108 "solid over the stack": rows 4..29, cols 3..28.
const CRATES_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 13.0, 2.0, 28.0)];
/// REF-109 "solid over the group": rows 5..29, cols 3..28.
const BARRELS_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 13.0, 2.0, 27.0)];
/// REF-110 "solid over the body and wheel": rows 12..29, cols 8..27 (the
/// handle to the west, cols 2..7, is not solid).
const HANDCART_SOLID: [Aabb; 1] = [Aabb {
    min: Vec2::new(-8.0, 2.0),
    max: Vec2::new(12.0, 20.0),
}];
/// REF-111 "solid at the two pillars only": rows 18..61 (below the arch
/// spring to the base), cols 6..21 and 42..57. The 20 px between them is
/// the arch the player walks through.
const CHECKERS_GATE_SOLIDS: [Aabb; 2] = [
    Aabb {
        min: Vec2::new(-26.0, 2.0),
        max: Vec2::new(-10.0, 46.0),
    },
    Aabb {
        min: Vec2::new(10.0, 2.0),
        max: Vec2::new(26.0, 46.0),
    },
];
/// REF-112 "solid only at the foot": rows 27..30, cols 11..20.
const LANTERN_POST_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 5.0, 1.0, 5.0)];
/// REF-113 "solid over the seat": rows 18..22, cols 3..28.
const BENCH_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 13.0, 9.0, 14.0)];
/// REF-114 "solid along the rails": rows 13..23 (upper rail top to lower
/// rail bottom), the full 32 px width, so pieces 32 px apart join.
const FENCE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 16.0, 8.0, 19.0)];
/// REF-116 "solid over the frame": rows 14..29 (the uprights' tops to
/// their feet, crossbar and water tray included), cols 5..26. The wheel
/// above stands behind it and is not solid.
const GRINDSTONE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 11.0, 2.0, 18.0)];
/// REF-117 "solid over the stack": rows 18..30, cols 2..29.
const INGOTS_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 14.0, 1.0, 14.0)];
/// REF-118 "solid over the stand": rows 19..29, cols 4..24.
const BELLOWS_SOLID: [Aabb; 1] = [Aabb {
    min: Vec2::new(-12.0, 2.0),
    max: Vec2::new(9.0, 13.0),
}];
/// REF-119 "solid over the rack": rows 8..29 (top rail to feet), cols
/// 3..28. The sword hilts above are not solid.
const WEAPON_RACK_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 13.0, 2.0, 24.0)];
/// REF-120 "solid over the bench": rows 15..29 (bench top to feet), cols
/// 1..30.
const WORKBENCH_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 15.0, 2.0, 17.0)];
/// REF-121 "solid over the bucket": rows 14..29 (handle to base), cols
/// 3..14. The shovel to the east is not solid.
const BUCKET_SHOVEL_SOLID: [Aabb; 1] = [Aabb {
    min: Vec2::new(-13.0, 2.0),
    max: Vec2::new(-1.0, 18.0),
}];
/// REF-128 "solid over the chest": rows 12..29, cols 4..27.
const STRONGBOX_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 12.0, 2.0, 20.0)];
/// REF-129 "solid over the planter": rows 21..29, cols 7..24. The bush
/// above it is not solid.
const PLANTER_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 9.0, 2.0, 11.0)];

// --- Acts 3-5 (approved 2026-11-18) -------------------------------------
/// REF-123 "solid from the steps to the eave": rows 18..62 (the cornice
/// to the bottom step), cols 4..59 (the cornice, the widest part).
const GUILDHALL_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 28.0, 1.0, 46.0)];
/// REF-124 "solid over the desk": rows 15..29 (desk top to feet), cols
/// 4..27. The ledger, inkpot and quill on top are not solid.
const LEDGER_DESK_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 12.0, 2.0, 17.0)];
/// REF-125 "solid only at the two posts": rows 23..29, cols 5..7 and
/// 24..26. The 16 px between them stays open under the board.
const NOTICE_BOARD_SOLIDS: [Aabb; 2] = [
    Aabb {
        min: Vec2::new(-11.0, 2.0),
        max: Vec2::new(-8.0, 9.0),
    },
    Aabb {
        min: Vec2::new(8.0, 2.0),
        max: Vec2::new(11.0, 9.0),
    },
];
/// REF-126 "solid only at the foot": the stone foot, rows 27..30, cols
/// 11..20.
const GUILD_BANNER_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 5.0, 1.0, 5.0)];
/// REF-127 "solid over the unit": rows 3..29, cols 4..27.
const SCROLL_SHELF_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 12.0, 2.0, 29.0)];
/// REF-131 "solid from the base to the eave": rows 22..61 (the gold band
/// under the roof to the base), cols 14..49 (the band, the widest part).
const MAGES_TOWER_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 18.0, 2.0, 42.0)];
/// REF-132 "solid at the stand": the post and its foot, rows 23..30,
/// cols 9..22. The rings and worlds above are not solid.
const ORRERY_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 7.0, 1.0, 9.0)];
/// REF-133 "solid over the stone": rows 3..29, cols 9..22 (the grass at
/// its foot, cols 6..8, is not solid).
const RUNE_STONE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 7.0, 2.0, 29.0)];
/// REF-134 "solid at the post": the post and its foot, rows 16..30, cols
/// 9..22. The tome above is not solid.
const LECTERN_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 7.0, 1.0, 16.0)];
/// REF-135 "solid over the pedestal": rows 20..30, cols 9..22.
const CRYSTAL_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 7.0, 1.0, 12.0)];
/// REF-136 "solid over the cauldron": rows 11..30 (the rim to the legs'
/// feet), cols 3..28. The fire's last pixels (row 31) and the motes above
/// are not solid.
const CAULDRON_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 13.0, 1.0, 21.0)];
/// REF-137 "solid at the tripod's feet": rows 29..30, cols 8..23.
const TELESCOPE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 8.0, 1.0, 3.0)];
/// REF-139 "solid from the step to the eave": rows 21..62 (the eave to
/// the step), cols 2..61 (the eave, the widest part).
const TAVERN_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 30.0, 1.0, 43.0)];
/// REF-140 "solid over the table": rows 14..29 (table top to trestle
/// feet), cols 1..30. The tankards and the loaf are not solid.
const TRESTLE_TABLE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 15.0, 2.0, 18.0)];
/// REF-141 "solid over the rack": rows 5..29, cols 3..28.
const KEG_RACK_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 13.0, 2.0, 27.0)];
/// REF-142 "solid over the stone ring": rows 22..29 (the ring and the
/// spit posts' feet), cols 3..28. The flames and the spit are not solid.
const FIREPIT_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 13.0, 2.0, 10.0)];
/// REF-143 "solid at the foot": the post's stone foot at the LEFT of the
/// sprite, rows 27..30, cols 2..11. The hanging board is not solid.
const TAVERN_SIGN_SOLID: [Aabb; 1] = [Aabb {
    min: Vec2::new(-14.0, 1.0),
    max: Vec2::new(-4.0, 5.0),
}];
/// REF-144 "solid over the table": rows 9..29 (the top to the leg's
/// foot), cols 4..27. The stools' outer halves (cols 0..3, 28..31) and
/// the candle are not solid.
const ROUND_TABLE_SOLID: [Aabb; 1] = [Aabb::from_base(0.0, 12.0, 2.0, 23.0)];
/// REF-145 "solid at the legs": rows 24..29, cols 6..8 and 23..25.
const TAB_SLATE_SOLIDS: [Aabb; 2] = [
    Aabb {
        min: Vec2::new(-10.0, 2.0),
        max: Vec2::new(-7.0, 8.0),
    },
    Aabb {
        min: Vec2::new(7.0, 2.0),
        max: Vec2::new(10.0, 8.0),
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
            PropKind::BellTower => SPRITE_PROP_BELL_TOWER,
            PropKind::Well => SPRITE_PROP_WELL,
            PropKind::Woodpile => SPRITE_PROP_WOODPILE,
            PropKind::Signpost | PropKind::SignpostWest => SPRITE_PROP_SIGNPOST,
            PropKind::ChickenCoop => SPRITE_PROP_CHICKEN_COOP,
            PropKind::MarketStall => SPRITE_PROP_MARKET_STALL,
            PropKind::Crates => SPRITE_PROP_CRATES,
            PropKind::Barrels => SPRITE_PROP_BARRELS,
            PropKind::Handcart => SPRITE_PROP_HANDCART,
            PropKind::CheckersGate => SPRITE_PROP_CHECKERS_GATE,
            PropKind::LanternPost => SPRITE_PROP_LANTERN_POST,
            PropKind::Bench => SPRITE_PROP_BENCH,
            PropKind::Fence => SPRITE_PROP_FENCE,
            PropKind::Grindstone => SPRITE_PROP_GRINDSTONE,
            PropKind::Ingots => SPRITE_PROP_INGOTS,
            PropKind::Bellows => SPRITE_PROP_BELLOWS,
            PropKind::WeaponRack => SPRITE_PROP_WEAPON_RACK,
            PropKind::Workbench => SPRITE_PROP_WORKBENCH,
            PropKind::BucketShovel => SPRITE_PROP_BUCKET_SHOVEL,
            PropKind::Strongbox => SPRITE_PROP_STRONGBOX,
            PropKind::Planter => SPRITE_PROP_PLANTER,
            PropKind::Guildhall => SPRITE_PROP_GUILDHALL,
            PropKind::LedgerDesk => SPRITE_PROP_LEDGER_DESK,
            PropKind::NoticeBoard => SPRITE_PROP_NOTICE_BOARD,
            PropKind::GuildBanner => SPRITE_PROP_GUILD_BANNER,
            PropKind::ScrollShelf => SPRITE_PROP_SCROLL_SHELF,
            PropKind::MagesTower => SPRITE_PROP_MAGES_TOWER,
            PropKind::Orrery => SPRITE_PROP_ORRERY,
            PropKind::RuneStone => SPRITE_PROP_RUNE_STONE,
            PropKind::Lectern => SPRITE_PROP_LECTERN,
            PropKind::Crystal => SPRITE_PROP_CRYSTAL,
            PropKind::Cauldron => SPRITE_PROP_CAULDRON,
            PropKind::Telescope => SPRITE_PROP_TELESCOPE,
            PropKind::Tavern => SPRITE_PROP_TAVERN,
            PropKind::TrestleTable => SPRITE_PROP_TRESTLE_TABLE,
            PropKind::KegRack => SPRITE_PROP_KEG_RACK,
            PropKind::Firepit => SPRITE_PROP_FIREPIT,
            PropKind::TavernSign => SPRITE_PROP_TAVERN_SIGN,
            PropKind::RoundTable => SPRITE_PROP_ROUND_TABLE,
            PropKind::TabSlate => SPRITE_PROP_TAB_SLATE,
        }
    }

    /// True for a kind drawn as its sprite mirrored left-right (the
    /// west-pointing signpost).
    pub fn mirrored(self) -> bool {
        matches!(self, PropKind::SignpostWest)
    }

    /// True for a 64x64 district landmark (placed by [`LANDMARKS`]).
    pub fn is_landmark(self) -> bool {
        matches!(
            self,
            PropKind::ThroneDais
                | PropKind::SealedStacks
                | PropKind::GreatFurnace
                | PropKind::BellTower
                | PropKind::MarketStall
                | PropKind::CheckersGate
                | PropKind::Guildhall
                | PropKind::MagesTower
                | PropKind::Tavern
        )
    }

    /// Native sprite size in pixels (displayed at 1x, like characters).
    pub fn size(self) -> Vec2 {
        match self {
            PropKind::Oak => Vec2::new(32.0, 48.0),
            PropKind::Cottage
            | PropKind::ThroneDais
            | PropKind::SealedStacks
            | PropKind::GreatFurnace
            | PropKind::BellTower
            | PropKind::MarketStall
            | PropKind::CheckersGate
            | PropKind::Guildhall
            | PropKind::MagesTower
            | PropKind::Tavern => Vec2::new(64.0, 64.0),
            PropKind::Anvil
            | PropKind::QuenchTrough
            | PropKind::CoalHeap
            | PropKind::ToolRack
            | PropKind::Well
            | PropKind::Woodpile
            | PropKind::Signpost
            | PropKind::SignpostWest
            | PropKind::ChickenCoop
            | PropKind::Crates
            | PropKind::Barrels
            | PropKind::Handcart
            | PropKind::LanternPost
            | PropKind::Bench
            | PropKind::Fence
            | PropKind::Grindstone
            | PropKind::Ingots
            | PropKind::Bellows
            | PropKind::WeaponRack
            | PropKind::Workbench
            | PropKind::BucketShovel
            | PropKind::Strongbox
            | PropKind::Planter
            | PropKind::LedgerDesk
            | PropKind::NoticeBoard
            | PropKind::GuildBanner
            | PropKind::ScrollShelf
            | PropKind::Orrery
            | PropKind::RuneStone
            | PropKind::Lectern
            | PropKind::Crystal
            | PropKind::Cauldron
            | PropKind::Telescope
            | PropKind::TrestleTable
            | PropKind::KegRack
            | PropKind::Firepit
            | PropKind::TavernSign
            | PropKind::RoundTable
            | PropKind::TabSlate => Vec2::new(32.0, 32.0),
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
            PropKind::BellTower => 2.0,    // REF-103 row 61 is the base's bottom edge
            PropKind::Well => 2.0,         // REF-104 row 29 is the ring's bottom edge
            PropKind::Woodpile => 3.0,     // REF-105 row 28 is the bottom logs' edge
            PropKind::Signpost | PropKind::SignpostWest => 1.0, // REF-106 row 30: the tufts
            PropKind::ChickenCoop => 2.0,  // REF-115 row 29 is the legs' bottom
            PropKind::MarketStall => 7.0,  // REF-107 row 56 is the counter's foot
            PropKind::Crates => 2.0,       // REF-108 row 29 is the ground crates' edge
            PropKind::Barrels => 2.0,      // REF-109 row 29 is the front barrels' edge
            PropKind::Handcart => 2.0,     // REF-110 row 29 is the wheel's bottom
            PropKind::CheckersGate => 2.0, // REF-111 row 61 is the pillar bases' edge
            PropKind::LanternPost => 1.0,  // REF-112 row 30 is the foot's bottom edge
            PropKind::Bench => 3.0,        // REF-113 row 28 is the legs' bottom
            PropKind::Fence => 3.0,        // REF-114 row 28 is the posts' bottom
            PropKind::Grindstone => 2.0,   // REF-116 row 29 is the frame's feet
            PropKind::Ingots => 1.0,       // REF-117 row 30 is the pallet's bottom
            PropKind::Bellows => 2.0,      // REF-118 row 29 is the stand's feet
            PropKind::WeaponRack => 2.0,   // REF-119 row 29 is the rack's feet
            PropKind::Workbench => 2.0,    // REF-120 row 29 is the legs' bottom
            PropKind::BucketShovel => 1.0, // REF-121 row 30 is the shovel's tip
            PropKind::Strongbox => 2.0,    // REF-128 row 29 is the chest's bottom
            PropKind::Planter => 2.0,      // REF-129 row 29 is the planter's bottom
            PropKind::Guildhall => 1.0,    // REF-123 row 62 is the bottom step's edge
            PropKind::LedgerDesk => 2.0,   // REF-124 row 29 is the legs' bottom
            PropKind::NoticeBoard => 2.0,  // REF-125 row 29 is the posts' bottom
            PropKind::GuildBanner => 1.0,  // REF-126 row 30 is the foot's bottom edge
            PropKind::ScrollShelf => 2.0,  // REF-127 row 29 is the unit's bottom
            PropKind::MagesTower => 2.0,   // REF-131 row 61 is the base's bottom edge
            PropKind::Orrery => 1.0,       // REF-132 row 30 is the foot's bottom edge
            PropKind::RuneStone => 2.0,    // REF-133 row 29 is the stone's foot
            PropKind::Lectern => 1.0,      // REF-134 row 30 is the foot's bottom edge
            PropKind::Crystal => 1.0,      // REF-135 row 30 is the pedestal's bottom
            PropKind::Cauldron => 0.0,     // REF-136 row 31 is the fire's last pixels
            PropKind::Telescope => 1.0,    // REF-137 row 30 is the tripod's feet
            PropKind::Tavern => 1.0,       // REF-139 row 62 is the step's bottom edge
            PropKind::TrestleTable => 2.0, // REF-140 row 29 is the trestles' feet
            PropKind::KegRack => 2.0,      // REF-141 row 29 is the rack's bottom
            PropKind::Firepit => 2.0,      // REF-142 row 29 is the ring's bottom edge
            PropKind::TavernSign => 1.0,   // REF-143 row 30 is the foot's bottom edge
            PropKind::RoundTable => 2.0,   // REF-144 row 29 is the stools' feet
            PropKind::TabSlate => 2.0,     // REF-145 row 29 is the legs' bottom
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
            PropKind::BellTower => &BELL_TOWER_SOLID,
            PropKind::Well => &WELL_SOLID,
            PropKind::Woodpile => &WOODPILE_SOLID,
            PropKind::Signpost | PropKind::SignpostWest => &SIGNPOST_SOLID,
            PropKind::ChickenCoop => &CHICKEN_COOP_SOLID,
            PropKind::MarketStall => &MARKET_STALL_SOLID,
            PropKind::Crates => &CRATES_SOLID,
            PropKind::Barrels => &BARRELS_SOLID,
            PropKind::Handcart => &HANDCART_SOLID,
            PropKind::CheckersGate => &CHECKERS_GATE_SOLIDS,
            PropKind::LanternPost => &LANTERN_POST_SOLID,
            PropKind::Bench => &BENCH_SOLID,
            PropKind::Fence => &FENCE_SOLID,
            PropKind::Grindstone => &GRINDSTONE_SOLID,
            PropKind::Ingots => &INGOTS_SOLID,
            PropKind::Bellows => &BELLOWS_SOLID,
            PropKind::WeaponRack => &WEAPON_RACK_SOLID,
            PropKind::Workbench => &WORKBENCH_SOLID,
            PropKind::BucketShovel => &BUCKET_SHOVEL_SOLID,
            PropKind::Strongbox => &STRONGBOX_SOLID,
            PropKind::Planter => &PLANTER_SOLID,
            PropKind::Guildhall => &GUILDHALL_SOLID,
            PropKind::LedgerDesk => &LEDGER_DESK_SOLID,
            PropKind::NoticeBoard => &NOTICE_BOARD_SOLIDS,
            PropKind::GuildBanner => &GUILD_BANNER_SOLID,
            PropKind::ScrollShelf => &SCROLL_SHELF_SOLID,
            PropKind::MagesTower => &MAGES_TOWER_SOLID,
            PropKind::Orrery => &ORRERY_SOLID,
            PropKind::RuneStone => &RUNE_STONE_SOLID,
            PropKind::Lectern => &LECTERN_SOLID,
            PropKind::Crystal => &CRYSTAL_SOLID,
            PropKind::Cauldron => &CAULDRON_SOLID,
            PropKind::Telescope => &TELESCOPE_SOLID,
            PropKind::Tavern => &TAVERN_SOLID,
            PropKind::TrestleTable => &TRESTLE_TABLE_SOLID,
            PropKind::KegRack => &KEG_RACK_SOLID,
            PropKind::Firepit => &FIREPIT_SOLID,
            PropKind::TavernSign => &TAVERN_SIGN_SOLID,
            PropKind::RoundTable => &ROUND_TABLE_SOLID,
            PropKind::TabSlate => &TAB_SLATE_SOLIDS,
        }
    }

    /// The solid parts of a prop anchored at `base`, in world space.
    pub fn solids_at(self, base: Vec2) -> impl Iterator<Item = Aabb> {
        self.solids_local().iter().map(move |s| s.translated(base))
    }

    /// Bounding box of the solid footprint relative to the anchor
    /// (bottom-centre): for a one-box kind, the box itself.
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
    /// Great Furnace, anvil, quench trough, coal heap and the demo plaza
    /// set (REF-103..115): see the consts above. Tool rack and Checker's
    /// Gate: the box round both feet / pillars; only those parts collide
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
    /// to feet). The demo plaza set (REF-103..115): every opaque pixel, so
    /// a roof, awning, lintel, lantern or sign board fades over a character
    /// behind it; the mirrored signpost mirrors its box.
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
            // REF-103 cols 14..49, rows 0..61.
            PropKind::BellTower => Aabb::from_base(0.0, 18.0, 2.0, 64.0),
            // REF-104 cols 3..28, rows 2..29.
            PropKind::Well => Aabb::from_base(0.0, 13.0, 2.0, 30.0),
            PropKind::Woodpile => WOODPILE_SOLID[0],
            // REF-106 cols 4..29 (the board points east), rows 6..30.
            PropKind::Signpost => Aabb {
                min: Vec2::new(-12.0, 1.0),
                max: Vec2::new(14.0, 26.0),
            },
            PropKind::SignpostWest => Aabb {
                min: Vec2::new(-14.0, 1.0),
                max: Vec2::new(12.0, 26.0),
            },
            // REF-115 cols 4..29 (the hen included), rows 3..29.
            PropKind::ChickenCoop => Aabb {
                min: Vec2::new(-12.0, 2.0),
                max: Vec2::new(14.0, 29.0),
            },
            PropKind::MarketStall => MARKET_STALL_SOLID[0],
            PropKind::Crates => CRATES_SOLID[0],
            PropKind::Barrels => BARRELS_SOLID[0],
            // REF-110 cols 2..27 (handle to rest leg), rows 8..29.
            PropKind::Handcart => Aabb {
                min: Vec2::new(-14.0, 2.0),
                max: Vec2::new(12.0, 24.0),
            },
            // REF-111 cols 4..59 (the lintel), rows 2..61.
            PropKind::CheckersGate => Aabb::from_base(0.0, 28.0, 2.0, 62.0),
            // REF-112 cols 11..20, rows 1..30.
            PropKind::LanternPost => Aabb::from_base(0.0, 5.0, 1.0, 31.0),
            // REF-113 cols 3..28, rows 10..28.
            PropKind::Bench => Aabb::from_base(0.0, 13.0, 3.0, 22.0),
            // REF-114 cols 0..31, rows 7..28.
            PropKind::Fence => Aabb::from_base(0.0, 16.0, 3.0, 25.0),
            // The density pass (REF-116..121, 128, 129): every opaque pixel.
            // REF-116 cols 5..26, rows 4..29 (the wheel included).
            PropKind::Grindstone => Aabb::from_base(0.0, 11.0, 2.0, 28.0),
            PropKind::Ingots => INGOTS_SOLID[0],
            // REF-118 cols 0..29 (handle to nozzle), rows 5..29.
            PropKind::Bellows => Aabb {
                min: Vec2::new(-16.0, 2.0),
                max: Vec2::new(14.0, 27.0),
            },
            // REF-119 cols 3..28, rows 1..29 (pommels to feet).
            PropKind::WeaponRack => Aabb::from_base(0.0, 13.0, 2.0, 31.0),
            // REF-120 cols 1..30, rows 9..29 (the vice on top).
            PropKind::Workbench => Aabb::from_base(0.0, 15.0, 2.0, 23.0),
            // REF-121 cols 3..27, rows 1..30 (the shovel's grip to its tip).
            PropKind::BucketShovel => Aabb {
                min: Vec2::new(-13.0, 1.0),
                max: Vec2::new(12.0, 31.0),
            },
            PropKind::Strongbox => STRONGBOX_SOLID[0],
            // REF-129 cols 7..24, rows 2..29 (the bush above the planter).
            PropKind::Planter => Aabb::from_base(0.0, 9.0, 2.0, 30.0),
            // Acts 3-5 (REF-123..127, 131..137, 139..145): every opaque
            // pixel, so a roof, banner, board, tome, ring or sign fades
            // over a character behind it.
            // REF-123 cols 4..59, rows 0..62 (pennant to bottom step).
            PropKind::Guildhall => Aabb::from_base(0.0, 28.0, 1.0, 64.0),
            // REF-124 cols 4..27, rows 5..29 (the quill's tip to the feet).
            PropKind::LedgerDesk => Aabb::from_base(0.0, 12.0, 2.0, 27.0),
            // REF-125 cols 2..29, rows 4..29 (the roof to the posts).
            PropKind::NoticeBoard => Aabb::from_base(0.0, 14.0, 2.0, 28.0),
            // REF-126 cols 5..26, rows 0..30 (the finial to the foot).
            PropKind::GuildBanner => Aabb::from_base(0.0, 11.0, 1.0, 32.0),
            PropKind::ScrollShelf => SCROLL_SHELF_SOLID[0],
            // REF-131 cols 14..49, rows 0..61 (the finial to the base).
            PropKind::MagesTower => Aabb::from_base(0.0, 18.0, 2.0, 64.0),
            // REF-132 cols 4..27, rows 1..30 (the rings and worlds).
            PropKind::Orrery => Aabb::from_base(0.0, 12.0, 1.0, 31.0),
            // REF-133 cols 6..22, rows 3..29 (the grass at its foot).
            PropKind::RuneStone => Aabb {
                min: Vec2::new(-10.0, 2.0),
                max: Vec2::new(7.0, 29.0),
            },
            // REF-134 cols 5..26, rows 4..30 (the tome to the foot).
            PropKind::Lectern => Aabb::from_base(0.0, 11.0, 1.0, 28.0),
            // REF-135 cols 9..22, rows 2..30 (the crystal's tip).
            PropKind::Crystal => Aabb::from_base(0.0, 7.0, 1.0, 30.0),
            // REF-136 cols 3..28, rows 3..31 (the top mote to the fire).
            PropKind::Cauldron => Aabb::from_base(0.0, 13.0, 0.0, 29.0),
            // REF-137 cols 4..26, rows 3..30 (the eyepiece to the feet).
            PropKind::Telescope => Aabb {
                min: Vec2::new(-12.0, 1.0),
                max: Vec2::new(11.0, 29.0),
            },
            // REF-139 cols 2..61, rows 0..62 (the chimney to the step).
            PropKind::Tavern => Aabb::from_base(0.0, 30.0, 1.0, 64.0),
            // REF-140 cols 1..30, rows 7..29 (the tankards' foam).
            PropKind::TrestleTable => Aabb::from_base(0.0, 15.0, 2.0, 25.0),
            PropKind::KegRack => KEG_RACK_SOLID[0],
            // REF-142 cols 3..28, rows 4..29 (the spit posts' tops).
            PropKind::Firepit => Aabb::from_base(0.0, 13.0, 2.0, 28.0),
            // REF-143 cols 2..27, rows 2..30 (the post's cap to its foot).
            PropKind::TavernSign => Aabb {
                min: Vec2::new(-14.0, 1.0),
                max: Vec2::new(12.0, 30.0),
            },
            // REF-144 cols 0..31, rows 3..29 (the candle flame, the stools).
            PropKind::RoundTable => Aabb::from_base(0.0, 16.0, 2.0, 29.0),
            // REF-145 cols 3..28, rows 3..29 (the frame to the legs).
            PropKind::TabSlate => Aabb::from_base(0.0, 13.0, 2.0, 29.0),
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
/// heads the Forge yard, the Smelter at its mouth. The free demo's plazas
/// (approved by Matt 2026-11-18): the Bell Tower (REF-103) heads
/// Hearthstone Square, the Bellringer beside it; the market stall
/// (REF-107) heads Market Row, the Twin at its counter; the Checker's Gate
/// (REF-111) heads The Commons, the Auditor at the arch. Acts 3-5
/// (approved by Matt 2026-11-18): the Guildhall (REF-123) heads the
/// Guildhall Quarter, the ledger desk before its steps; the Mage's Tower
/// (REF-131) its court, Vexis at the lectern before the door; the tavern
/// (REF-139) Tavern Row, the Barkeep and the Bouncer at its door. Their
/// bosses (the Guildmaster, Vexis, the Barkeep) are ordinary 32 px
/// villagers who hold a trial, so no boss stands centred on a plot here.
pub const LANDMARKS: &[(usize, PropKind)] = &[
    (9, PropKind::SealedStacks),
    (10, PropKind::ThroneDais),
    (6, PropKind::GreatFurnace),
    (0, PropKind::BellTower),
    (1, PropKind::MarketStall),
    (2, PropKind::CheckersGate),
    (3, PropKind::Guildhall),
    (4, PropKind::MagesTower),
    (5, PropKind::Tavern),
];

/// A district's own yard props: `(district index, kind, offset of the
/// bottom-centre anchor from the plaza centre)`. They stand ON the plaza
/// paving, clear of its street mouths and of the routes to every NPC (see
/// `tests`). A kind is used wherever it belongs and as often as it fits.
///
/// Each plaza is composed, not scattered: small scenes with open paving
/// between them, two-tile lanes from every street mouth to the centre and
/// to the landmark (`tests::lanes_from_the_streets_are_two_tiles_wide`),
/// and the cast gathered at the props people would stand at (the groups
/// are listed on each cast in `world.rs`). Approved sets: the Forge
/// (REF-99..102), the free demo's three plazas (REF-104..106,
/// REF-108..110, REF-112..115) and the density pass (REF-116..121 Forge
/// yard set, REF-128 strongbox, REF-129 planter) and the Acts 3-5 sets
/// (REF-124..127 Guildhall, REF-132..137 Mage's Tower, REF-140..145
/// Tavern Row), all approved by Matt 2026-11-18. Same-y fence pieces 32 px apart form one run
/// (`tests::fences_form_joined_runs`); the Commons runs meet the Checker's
/// Gate pillars, so the gate is a boundary you pass through.
pub const DISTRICT_PROPS: &[(usize, PropKind, Vec2)] = &[
    // --- The Forge (Great Furnace base (0, 156); street in from the east).
    // The furnace mouth: anvil and quench trough either side, the
    // bellows pointed at the furnace from the west, a water bucket east.
    (6, PropKind::Anvil, Vec2::new(-48.0, 120.0)),
    (6, PropKind::QuenchTrough, Vec2::new(48.0, 120.0)),
    (6, PropKind::Bellows, Vec2::new(-60.0, 176.0)),
    (6, PropKind::BucketShovel, Vec2::new(64.0, 180.0)),
    // Fuel to the north-west: two coal heaps.
    (6, PropKind::CoalHeap, Vec2::new(-108.0, 176.0)),
    (6, PropKind::CoalHeap, Vec2::new(-140.0, 204.0)),
    // The work corner to the north-east: tool rack, the finished swords,
    // the workbench, and the day's ingots by the east wall.
    (6, PropKind::ToolRack, Vec2::new(120.0, 200.0)),
    (6, PropKind::WeaponRack, Vec2::new(152.0, 200.0)),
    (6, PropKind::Workbench, Vec2::new(200.0, 196.0)),
    (6, PropKind::Ingots, Vec2::new(200.0, 100.0)),
    // A grindstone by the west wall.
    (6, PropKind::Grindstone, Vec2::new(-200.0, 100.0)),
    // The stock yard to the south: fuel west, iron east.
    (6, PropKind::CoalHeap, Vec2::new(-192.0, -200.0)),
    (6, PropKind::Woodpile, Vec2::new(-160.0, -208.0)),
    (6, PropKind::Barrels, Vec2::new(-120.0, -208.0)),
    (6, PropKind::BucketShovel, Vec2::new(-200.0, -152.0)),
    (6, PropKind::Ingots, Vec2::new(120.0, -200.0)),
    (6, PropKind::Ingots, Vec2::new(152.0, -200.0)),
    (6, PropKind::Crates, Vec2::new(184.0, -200.0)),
    (6, PropKind::Handcart, Vec2::new(184.0, -152.0)),
    // A finishing station in the south middle.
    (6, PropKind::Workbench, Vec2::new(-40.0, -168.0)),
    (6, PropKind::Grindstone, Vec2::new(8.0, -168.0)),
    (6, PropKind::Anvil, Vec2::new(56.0, -168.0)),
    (6, PropKind::WeaponRack, Vec2::new(104.0, -152.0)),
    // A second fuel store in the west lower half.
    (6, PropKind::CoalHeap, Vec2::new(-176.0, -80.0)),
    (6, PropKind::Woodpile, Vec2::new(-208.0, -48.0)),
    (6, PropKind::BucketShovel, Vec2::new(-136.0, -72.0)),
    // A second smithing station in the west middle.
    (6, PropKind::Anvil, Vec2::new(-136.0, 32.0)),
    (6, PropKind::QuenchTrough, Vec2::new(-184.0, 32.0)),
    // Lantern posts either side of the street mouth, deliveries south of
    // it, more ingots north.
    (6, PropKind::LanternPost, Vec2::new(220.0, 52.0)),
    (6, PropKind::LanternPost, Vec2::new(220.0, -60.0)),
    (6, PropKind::Crates, Vec2::new(168.0, -104.0)),
    (6, PropKind::Barrels, Vec2::new(200.0, -104.0)),
    (6, PropKind::Ingots, Vec2::new(168.0, 100.0)),
    // --- Hearthstone Square (Bell Tower base (0, 196); four streets meet
    // at the spawn, and the cross between them stays open).
    // The well before the tower, the Cooper's barrels west of it, crates
    // and a handcart east, two-tile lanes between.
    (0, PropKind::Well, Vec2::new(0.0, 120.0)),
    (0, PropKind::Barrels, Vec2::new(-88.0, 124.0)),
    (0, PropKind::Barrels, Vec2::new(-120.0, 124.0)),
    (0, PropKind::Crates, Vec2::new(88.0, 124.0)),
    (0, PropKind::Handcart, Vec2::new(124.0, 124.0)),
    // Planters flanking the tower, well clear of the way round it.
    (0, PropKind::Planter, Vec2::new(-88.0, 228.0)),
    (0, PropKind::Planter, Vec2::new(88.0, 228.0)),
    // North-west: the Smith's corner, woodpile, anvil and bucket.
    (0, PropKind::Woodpile, Vec2::new(-248.0, 228.0)),
    (0, PropKind::Woodpile, Vec2::new(-216.0, 228.0)),
    (0, PropKind::BucketShovel, Vec2::new(-176.0, 232.0)),
    (0, PropKind::Anvil, Vec2::new(-176.0, 184.0)),
    // North-east: the chicken yard behind a five-piece fence, open at
    // its west end.
    (0, PropKind::Fence, Vec2::new(140.0, 148.0)),
    (0, PropKind::Fence, Vec2::new(172.0, 148.0)),
    (0, PropKind::Fence, Vec2::new(204.0, 148.0)),
    (0, PropKind::Fence, Vec2::new(236.0, 148.0)),
    (0, PropKind::Fence, Vec2::new(268.0, 148.0)),
    (0, PropKind::ChickenCoop, Vec2::new(196.0, 212.0)),
    (0, PropKind::Crates, Vec2::new(244.0, 212.0)),
    // East street: lantern posts either side, the signpost to The
    // Commons north of it.
    (0, PropKind::LanternPost, Vec2::new(264.0, 40.0)),
    (0, PropKind::LanternPost, Vec2::new(264.0, -56.0)),
    (0, PropKind::Signpost, Vec2::new(224.0, 56.0)),
    // West street, the same, the mirrored signpost to Market Row.
    (0, PropKind::LanternPost, Vec2::new(-264.0, 40.0)),
    (0, PropKind::LanternPost, Vec2::new(-264.0, -56.0)),
    (0, PropKind::SignpostWest, Vec2::new(-224.0, 56.0)),
    // South street: lantern posts either side of the mouth.
    (0, PropKind::LanternPost, Vec2::new(-48.0, -268.0)),
    (0, PropKind::LanternPost, Vec2::new(64.0, -268.0)),
    // South-east: the Borrow Checker's two lantern posts, stores in the
    // corner.
    (0, PropKind::LanternPost, Vec2::new(112.0, -208.0)),
    (0, PropKind::LanternPost, Vec2::new(240.0, -208.0)),
    (0, PropKind::Barrels, Vec2::new(264.0, -264.0)),
    (0, PropKind::Crates, Vec2::new(232.0, -264.0)),
    // South-west: a seating corner (two benches, a planter between),
    // planters by Ferris, clear of the walk from the spawn to him.
    (0, PropKind::Bench, Vec2::new(-248.0, -136.0)),
    (0, PropKind::Planter, Vec2::new(-216.0, -136.0)),
    (0, PropKind::Bench, Vec2::new(-184.0, -136.0)),
    (0, PropKind::Planter, Vec2::new(-268.0, -268.0)),
    (0, PropKind::Planter, Vec2::new(-188.0, -276.0)),
    // Stores south-east of Ferris, off the walk to him.
    (0, PropKind::Crates, Vec2::new(-120.0, -256.0)),
    (0, PropKind::Barrels, Vec2::new(-88.0, -256.0)),
    // Planters lining the east, west and south approaches to the spawn.
    (0, PropKind::Planter, Vec2::new(-136.0, 56.0)),
    (0, PropKind::Planter, Vec2::new(-136.0, -72.0)),
    (0, PropKind::Planter, Vec2::new(136.0, 56.0)),
    (0, PropKind::Planter, Vec2::new(136.0, -72.0)),
    (0, PropKind::Planter, Vec2::new(-56.0, -136.0)),
    (0, PropKind::Planter, Vec2::new(72.0, -136.0)),
    // A bench looking onto the east street.
    (0, PropKind::Bench, Vec2::new(200.0, -112.0)),
    // --- Market Row (stall base (0, 156); street in from the east).
    // The market: stock either side of the stall, the strongbox at its
    // west end, a loaded handcart parked east.
    (1, PropKind::Strongbox, Vec2::new(-48.0, 180.0)),
    (1, PropKind::Crates, Vec2::new(-80.0, 196.0)),
    (1, PropKind::Barrels, Vec2::new(-80.0, 164.0)),
    (1, PropKind::Barrels, Vec2::new(-112.0, 196.0)),
    (1, PropKind::Barrels, Vec2::new(48.0, 180.0)),
    (1, PropKind::Crates, Vec2::new(80.0, 196.0)),
    (1, PropKind::Crates, Vec2::new(80.0, 164.0)),
    (1, PropKind::Handcart, Vec2::new(128.0, 176.0)),
    (1, PropKind::Planter, Vec2::new(-200.0, 196.0)),
    (1, PropKind::Planter, Vec2::new(200.0, 196.0)),
    // The street mouth: lantern posts either side, the signpost to the
    // hub.
    (1, PropKind::LanternPost, Vec2::new(220.0, 40.0)),
    (1, PropKind::LanternPost, Vec2::new(220.0, -56.0)),
    (1, PropKind::Signpost, Vec2::new(184.0, 56.0)),
    // South-east, by the street: a handcart being unloaded.
    (1, PropKind::Handcart, Vec2::new(144.0, -104.0)),
    (1, PropKind::Crates, Vec2::new(176.0, -104.0)),
    (1, PropKind::Barrels, Vec2::new(208.0, -104.0)),
    (1, PropKind::Barrels, Vec2::new(168.0, -200.0)),
    (1, PropKind::Crates, Vec2::new(200.0, -200.0)),
    // South-west: a seating corner.
    (1, PropKind::LanternPost, Vec2::new(-208.0, -136.0)),
    (1, PropKind::Bench, Vec2::new(-160.0, -136.0)),
    (1, PropKind::Planter, Vec2::new(-120.0, -136.0)),
    (1, PropKind::Bench, Vec2::new(-80.0, -136.0)),
    (1, PropKind::Planter, Vec2::new(-208.0, -200.0)),
    // The west wall: more stock.
    (1, PropKind::Barrels, Vec2::new(-208.0, 48.0)),
    (1, PropKind::Crates, Vec2::new(-208.0, 16.0)),
    // A second pitch in the west middle: a handcart and its stock.
    (1, PropKind::Handcart, Vec2::new(-120.0, 40.0)),
    (1, PropKind::Crates, Vec2::new(-88.0, 40.0)),
    (1, PropKind::Barrels, Vec2::new(-88.0, 72.0)),
    // Planters lining the way in from the street.
    (1, PropKind::Planter, Vec2::new(120.0, 64.0)),
    (1, PropKind::Planter, Vec2::new(120.0, -56.0)),
    // South middle: a lantern post, a bench and a planter.
    (1, PropKind::LanternPost, Vec2::new(0.0, -196.0)),
    (1, PropKind::Bench, Vec2::new(32.0, -200.0)),
    (1, PropKind::Planter, Vec2::new(64.0, -200.0)),
    (1, PropKind::Planter, Vec2::new(32.0, -120.0)),
    // West, below the centre: goods waiting to be shelved.
    (1, PropKind::Crates, Vec2::new(-160.0, -56.0)),
    (1, PropKind::Barrels, Vec2::new(-128.0, -56.0)),
    (1, PropKind::Crates, Vec2::new(-160.0, -24.0)),
    // --- The Commons (Checker's Gate base (0, 156); street in from the
    // west). A fence run either side of the gate, the inner pieces
    // meeting its pillars, so the north is a court entered through the
    // arch or round the fence ends.
    (2, PropKind::Fence, Vec2::new(-168.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(-136.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(-104.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(-72.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(-40.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(40.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(72.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(104.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(136.0, 156.0)),
    (2, PropKind::Fence, Vec2::new(168.0, 156.0)),
    // The court: a bench and a planter either side of the arch.
    (2, PropKind::Bench, Vec2::new(-72.0, 196.0)),
    (2, PropKind::Planter, Vec2::new(-104.0, 200.0)),
    (2, PropKind::Bench, Vec2::new(72.0, 196.0)),
    (2, PropKind::Planter, Vec2::new(104.0, 200.0)),
    // The approach to the gate: lantern posts, planters outside them.
    (2, PropKind::LanternPost, Vec2::new(-40.0, 40.0)),
    (2, PropKind::LanternPost, Vec2::new(40.0, 40.0)),
    (2, PropKind::Planter, Vec2::new(-88.0, 88.0)),
    (2, PropKind::Planter, Vec2::new(88.0, 88.0)),
    // The street mouth: lantern posts either side, the signpost to the
    // hub.
    (2, PropKind::LanternPost, Vec2::new(-220.0, 40.0)),
    (2, PropKind::LanternPost, Vec2::new(-220.0, -56.0)),
    (2, PropKind::SignpostWest, Vec2::new(-184.0, 56.0)),
    // South-east: a seating corner, two benches with a planter between
    // and a lantern post at the end.
    (2, PropKind::Bench, Vec2::new(64.0, -152.0)),
    (2, PropKind::Planter, Vec2::new(92.0, -152.0)),
    (2, PropKind::Bench, Vec2::new(120.0, -152.0)),
    (2, PropKind::LanternPost, Vec2::new(176.0, -124.0)),
    // South-west: the stores.
    (2, PropKind::Barrels, Vec2::new(-200.0, -136.0)),
    (2, PropKind::Crates, Vec2::new(-200.0, -168.0)),
    (2, PropKind::Crates, Vec2::new(-168.0, -200.0)),
    (2, PropKind::Handcart, Vec2::new(-128.0, -200.0)),
    // West middle: a bench between two planters, south of the street.
    (2, PropKind::Planter, Vec2::new(-136.0, -56.0)),
    (2, PropKind::Bench, Vec2::new(-104.0, -60.0)),
    (2, PropKind::Planter, Vec2::new(-72.0, -56.0)),
    // East middle: a bench and a planter.
    (2, PropKind::Planter, Vec2::new(120.0, 56.0)),
    (2, PropKind::Bench, Vec2::new(152.0, 56.0)),
    // South middle: the commons well and its bucket.
    (2, PropKind::Well, Vec2::new(-48.0, -176.0)),
    (2, PropKind::BucketShovel, Vec2::new(-8.0, -176.0)),
    // Planters at the court's two ends, and a second bench between
    // planters in the east middle.
    (2, PropKind::Planter, Vec2::new(-200.0, 200.0)),
    (2, PropKind::Planter, Vec2::new(200.0, 200.0)),
    (2, PropKind::Planter, Vec2::new(120.0, -24.0)),
    (2, PropKind::Bench, Vec2::new(152.0, -24.0)),
    (2, PropKind::Planter, Vec2::new(184.0, -24.0)),
    // --- Guildhall Quarter (Guildhall base (0, 156); street in from the
    // east). Formal and symmetric about the avenue x = 0.
    // The hall between a pair of banners, an archive wing either side.
    (3, PropKind::GuildBanner, Vec2::new(-48.0, 168.0)),
    (3, PropKind::GuildBanner, Vec2::new(48.0, 168.0)),
    (3, PropKind::ScrollShelf, Vec2::new(-112.0, 196.0)),
    (3, PropKind::ScrollShelf, Vec2::new(-144.0, 196.0)),
    (3, PropKind::Strongbox, Vec2::new(-184.0, 196.0)),
    (3, PropKind::ScrollShelf, Vec2::new(112.0, 196.0)),
    (3, PropKind::ScrollShelf, Vec2::new(144.0, 196.0)),
    (3, PropKind::Strongbox, Vec2::new(184.0, 196.0)),
    // The ledger desk before the steps, a notice board in each wing.
    (3, PropKind::LedgerDesk, Vec2::new(0.0, 76.0)),
    (3, PropKind::NoticeBoard, Vec2::new(-168.0, 112.0)),
    (3, PropKind::NoticeBoard, Vec2::new(168.0, 112.0)),
    // Planters lining the avenue from the south garden to the hall.
    (3, PropKind::Planter, Vec2::new(-72.0, 116.0)),
    (3, PropKind::Planter, Vec2::new(72.0, 116.0)),
    (3, PropKind::Planter, Vec2::new(-72.0, 24.0)),
    (3, PropKind::Planter, Vec2::new(72.0, 24.0)),
    (3, PropKind::Planter, Vec2::new(-72.0, -72.0)),
    (3, PropKind::Planter, Vec2::new(72.0, -72.0)),
    (3, PropKind::Planter, Vec2::new(-72.0, -136.0)),
    (3, PropKind::Planter, Vec2::new(72.0, -136.0)),
    // The south garden: a bench between two planters either side.
    (3, PropKind::Planter, Vec2::new(-192.0, -120.0)),
    (3, PropKind::Bench, Vec2::new(-152.0, -120.0)),
    (3, PropKind::Planter, Vec2::new(-112.0, -120.0)),
    (3, PropKind::Planter, Vec2::new(112.0, -120.0)),
    (3, PropKind::Bench, Vec2::new(152.0, -120.0)),
    (3, PropKind::Planter, Vec2::new(192.0, -120.0)),
    // The south wall: a banner pair closing the avenue, lantern corners.
    (3, PropKind::GuildBanner, Vec2::new(-40.0, -220.0)),
    (3, PropKind::GuildBanner, Vec2::new(40.0, -220.0)),
    (3, PropKind::LanternPost, Vec2::new(-200.0, -212.0)),
    (3, PropKind::LanternPost, Vec2::new(200.0, -212.0)),
    // The street mouth: lantern posts either side, the signpost.
    (3, PropKind::LanternPost, Vec2::new(220.0, 44.0)),
    (3, PropKind::LanternPost, Vec2::new(220.0, -72.0)),
    (3, PropKind::Signpost, Vec2::new(184.0, -72.0)),
    // The west wall answers the mouth with its own lantern pair.
    (3, PropKind::LanternPost, Vec2::new(-220.0, 44.0)),
    (3, PropKind::LanternPost, Vec2::new(-220.0, -72.0)),
    // Banners flanking the wings' outer corners, benches either side of
    // the middle, benches along the south wall.
    (3, PropKind::GuildBanner, Vec2::new(-216.0, 152.0)),
    (3, PropKind::GuildBanner, Vec2::new(216.0, 152.0)),
    (3, PropKind::Bench, Vec2::new(-152.0, -40.0)),
    (3, PropKind::Bench, Vec2::new(152.0, -40.0)),
    (3, PropKind::Bench, Vec2::new(-136.0, -212.0)),
    (3, PropKind::Bench, Vec2::new(136.0, -212.0)),
    // --- The Mage's Tower (tower base (0, 156); street in from the west).
    // A crystal either side of the door, the lectern before it, a pair of
    // rune stones flanking the approach.
    (4, PropKind::Crystal, Vec2::new(-40.0, 164.0)),
    (4, PropKind::Crystal, Vec2::new(40.0, 164.0)),
    (4, PropKind::Lectern, Vec2::new(0.0, 88.0)),
    (4, PropKind::RuneStone, Vec2::new(-96.0, 140.0)),
    (4, PropKind::RuneStone, Vec2::new(96.0, 140.0)),
    // The reading corner to the north-west: shelves, a lectern, a bench.
    (4, PropKind::ScrollShelf, Vec2::new(-112.0, 196.0)),
    (4, PropKind::ScrollShelf, Vec2::new(-144.0, 196.0)),
    (4, PropKind::ScrollShelf, Vec2::new(-176.0, 196.0)),
    (4, PropKind::Lectern, Vec2::new(-144.0, 140.0)),
    (4, PropKind::Bench, Vec2::new(-200.0, 140.0)),
    // The observatory to the north-east: two telescopes and a crystal.
    (4, PropKind::Telescope, Vec2::new(112.0, 196.0)),
    (4, PropKind::Crystal, Vec2::new(144.0, 196.0)),
    (4, PropKind::Telescope, Vec2::new(176.0, 196.0)),
    // The orrery to the east, a crystal by it.
    (4, PropKind::Orrery, Vec2::new(168.0, 64.0)),
    (4, PropKind::Crystal, Vec2::new(216.0, 120.0)),
    // The stone ring in the south: eight rune stones, 88 px round
    // (0, -116), the cauldron at its heart.
    (4, PropKind::RuneStone, Vec2::new(81.0, -82.0)),
    (4, PropKind::RuneStone, Vec2::new(34.0, -35.0)),
    (4, PropKind::RuneStone, Vec2::new(-34.0, -35.0)),
    (4, PropKind::RuneStone, Vec2::new(-81.0, -82.0)),
    (4, PropKind::RuneStone, Vec2::new(-81.0, -150.0)),
    (4, PropKind::RuneStone, Vec2::new(-34.0, -197.0)),
    (4, PropKind::RuneStone, Vec2::new(34.0, -197.0)),
    (4, PropKind::RuneStone, Vec2::new(81.0, -150.0)),
    (4, PropKind::Cauldron, Vec2::new(0.0, -128.0)),
    // The street mouth: crystals either side, the signpost.
    (4, PropKind::Crystal, Vec2::new(-220.0, 44.0)),
    (4, PropKind::Crystal, Vec2::new(-220.0, -72.0)),
    (4, PropKind::SignpostWest, Vec2::new(-184.0, 56.0)),
    // South-west: a bench between planters.
    (4, PropKind::Planter, Vec2::new(-208.0, -200.0)),
    (4, PropKind::Bench, Vec2::new(-168.0, -200.0)),
    (4, PropKind::Planter, Vec2::new(-128.0, -200.0)),
    // South-east: a second reading spot.
    (4, PropKind::ScrollShelf, Vec2::new(168.0, -208.0)),
    (4, PropKind::ScrollShelf, Vec2::new(200.0, -208.0)),
    (4, PropKind::Lectern, Vec2::new(184.0, -152.0)),
    // A third telescope by the reading corner, a crystal in the west
    // middle, standing stones by the walls, a bench and a crystal in the
    // east lower half.
    (4, PropKind::Telescope, Vec2::new(-200.0, 96.0)),
    (4, PropKind::Crystal, Vec2::new(-120.0, 56.0)),
    (4, PropKind::RuneStone, Vec2::new(-200.0, -104.0)),
    (4, PropKind::RuneStone, Vec2::new(200.0, -40.0)),
    (4, PropKind::Bench, Vec2::new(152.0, -104.0)),
    (4, PropKind::Crystal, Vec2::new(200.0, -104.0)),
    // --- Tavern Row (tavern base (0, 156); streets in from the north,
    // behind the tavern, the south, and west and east at y = -104).
    // The tavern's flanks: keg racks, barrels and crates, clear of the
    // way round it from the north street.
    (5, PropKind::KegRack, Vec2::new(-112.0, 196.0)),
    (5, PropKind::KegRack, Vec2::new(-144.0, 196.0)),
    (5, PropKind::Barrels, Vec2::new(-176.0, 196.0)),
    (5, PropKind::Crates, Vec2::new(-208.0, 196.0)),
    (5, PropKind::KegRack, Vec2::new(96.0, 196.0)),
    (5, PropKind::KegRack, Vec2::new(128.0, 196.0)),
    (5, PropKind::Barrels, Vec2::new(160.0, 196.0)),
    (5, PropKind::Crates, Vec2::new(192.0, 196.0)),
    // The door: the tab slate west of it, a tavern sign east.
    (5, PropKind::TabSlate, Vec2::new(-88.0, 128.0)),
    (5, PropKind::TavernSign, Vec2::new(88.0, 128.0)),
    // The firepit to the west, a bench either side, logs and barrels.
    (5, PropKind::Firepit, Vec2::new(-96.0, 24.0)),
    (5, PropKind::Bench, Vec2::new(-96.0, -28.0)),
    (5, PropKind::Bench, Vec2::new(-96.0, 72.0)),
    (5, PropKind::Woodpile, Vec2::new(-176.0, 64.0)),
    (5, PropKind::Barrels, Vec2::new(-208.0, 24.0)),
    // The trestle tables to the east, benches behind them, a barrel.
    (5, PropKind::TrestleTable, Vec2::new(96.0, 48.0)),
    (5, PropKind::TrestleTable, Vec2::new(176.0, 48.0)),
    (5, PropKind::Bench, Vec2::new(96.0, 88.0)),
    (5, PropKind::Bench, Vec2::new(176.0, 88.0)),
    (5, PropKind::Barrels, Vec2::new(216.0, 120.0)),
    // The south: round tables and stools either side of the south
    // street, lantern posts between.
    (5, PropKind::RoundTable, Vec2::new(-64.0, -176.0)),
    (5, PropKind::RoundTable, Vec2::new(-136.0, -208.0)),
    (5, PropKind::RoundTable, Vec2::new(-208.0, -176.0)),
    (5, PropKind::RoundTable, Vec2::new(64.0, -176.0)),
    (5, PropKind::RoundTable, Vec2::new(136.0, -208.0)),
    (5, PropKind::RoundTable, Vec2::new(208.0, -176.0)),
    (5, PropKind::LanternPost, Vec2::new(-104.0, -120.0)),
    (5, PropKind::LanternPost, Vec2::new(104.0, -120.0)),
    // The middle row, between the two cross-street mouths: a round table
    // and a trestle table either side of the way down from the door.
    (5, PropKind::RoundTable, Vec2::new(-160.0, -72.0)),
    (5, PropKind::TrestleTable, Vec2::new(-72.0, -80.0)),
    (5, PropKind::TrestleTable, Vec2::new(72.0, -80.0)),
    (5, PropKind::RoundTable, Vec2::new(160.0, -72.0)),
    // More stock: a keg rack and barrels by the west wall, crates by the
    // east barrels.
    (5, PropKind::KegRack, Vec2::new(-208.0, 128.0)),
    (5, PropKind::Barrels, Vec2::new(-176.0, 128.0)),
    (5, PropKind::Crates, Vec2::new(216.0, 152.0)),
    // A tavern sign at the west, east, south and north street mouths.
    (5, PropKind::TavernSign, Vec2::new(-216.0, -56.0)),
    (5, PropKind::TavernSign, Vec2::new(200.0, -56.0)),
    (5, PropKind::TavernSign, Vec2::new(-64.0, -236.0)),
    (5, PropKind::TavernSign, Vec2::new(-72.0, 204.0)),
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
            k if k.is_landmark() => landmarks += 1,
            _ => yard += 1,
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
                flip_x: kind.mirrored(),
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
                // Two plaza props are checked pixel by pixel in
                // `district_props_stand_on_their_plaza`.
                if ref_src(*ka).is_some() && ref_src(*kb).is_some() {
                    continue;
                }
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
        // An NPC is a centre-anchored sprite of its roster `native_px`
        // (32 px; the bosses 64 px). Gathered NPCs stand right beside a
        // prop's sprite but never on it.
        for (i, spec) in NPC_ROSTER.iter().enumerate() {
            let s = district_slot(i);
            let half = spec.native_px * 0.5;
            let npc = Aabb::from_base(s.x, half, s.y - half, s.y + half);
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

    const ALL_KINDS: [PropKind; 50] = [
        PropKind::Guildhall,
        PropKind::LedgerDesk,
        PropKind::NoticeBoard,
        PropKind::GuildBanner,
        PropKind::ScrollShelf,
        PropKind::MagesTower,
        PropKind::Orrery,
        PropKind::RuneStone,
        PropKind::Lectern,
        PropKind::Crystal,
        PropKind::Cauldron,
        PropKind::Telescope,
        PropKind::Tavern,
        PropKind::TrestleTable,
        PropKind::KegRack,
        PropKind::Firepit,
        PropKind::TavernSign,
        PropKind::RoundTable,
        PropKind::TabSlate,
        PropKind::Grindstone,
        PropKind::Ingots,
        PropKind::Bellows,
        PropKind::WeaponRack,
        PropKind::Workbench,
        PropKind::BucketShovel,
        PropKind::Strongbox,
        PropKind::Planter,
        PropKind::Oak,
        PropKind::Cottage,
        PropKind::ThroneDais,
        PropKind::SealedStacks,
        PropKind::GreatFurnace,
        PropKind::Anvil,
        PropKind::QuenchTrough,
        PropKind::CoalHeap,
        PropKind::ToolRack,
        PropKind::BellTower,
        PropKind::Well,
        PropKind::Woodpile,
        PropKind::Signpost,
        PropKind::SignpostWest,
        PropKind::ChickenCoop,
        PropKind::MarketStall,
        PropKind::Crates,
        PropKind::Barrels,
        PropKind::Handcart,
        PropKind::CheckersGate,
        PropKind::LanternPost,
        PropKind::Bench,
        PropKind::Fence,
    ];

    /// The approved ref grid behind a plaza prop's sprite (landmarks and
    /// district props), or `None` for the village oak and cottage.
    fn ref_src(kind: PropKind) -> Option<&'static str> {
        Some(match kind {
            PropKind::Oak | PropKind::Cottage => return None,
            PropKind::SealedStacks => {
                include_str!("../../../design/art/refs/ref-95-sealedstacks.jsx")
            }
            PropKind::ThroneDais => include_str!("../../../design/art/refs/ref-96-thronedais.jsx"),
            PropKind::GreatFurnace => {
                include_str!("../../../design/art/refs/ref-98-greatfurnace.jsx")
            }
            PropKind::Anvil => include_str!("../../../design/art/refs/ref-99-anvil.jsx"),
            PropKind::QuenchTrough => {
                include_str!("../../../design/art/refs/ref-100-quenchtrough.jsx")
            }
            PropKind::CoalHeap => include_str!("../../../design/art/refs/ref-101-coalheap.jsx"),
            PropKind::ToolRack => include_str!("../../../design/art/refs/ref-102-toolrack.jsx"),
            PropKind::BellTower => include_str!("../../../design/art/refs/ref-103-belltower.jsx"),
            PropKind::Well => include_str!("../../../design/art/refs/ref-104-well.jsx"),
            PropKind::Woodpile => include_str!("../../../design/art/refs/ref-105-woodpile.jsx"),
            PropKind::Signpost | PropKind::SignpostWest => {
                include_str!("../../../design/art/refs/ref-106-signpost.jsx")
            }
            PropKind::ChickenCoop => {
                include_str!("../../../design/art/refs/ref-115-chickencoop.jsx")
            }
            PropKind::MarketStall => {
                include_str!("../../../design/art/refs/ref-107-marketstall.jsx")
            }
            PropKind::Crates => include_str!("../../../design/art/refs/ref-108-crates.jsx"),
            PropKind::Barrels => include_str!("../../../design/art/refs/ref-109-barrels.jsx"),
            PropKind::Handcart => include_str!("../../../design/art/refs/ref-110-handcart.jsx"),
            PropKind::CheckersGate => {
                include_str!("../../../design/art/refs/ref-111-checkersgate.jsx")
            }
            PropKind::LanternPost => {
                include_str!("../../../design/art/refs/ref-112-lanternpost.jsx")
            }
            PropKind::Bench => include_str!("../../../design/art/refs/ref-113-bench.jsx"),
            PropKind::Fence => include_str!("../../../design/art/refs/ref-114-fence.jsx"),
            PropKind::Grindstone => {
                include_str!("../../../design/art/refs/ref-116-grindstone.jsx")
            }
            PropKind::Ingots => include_str!("../../../design/art/refs/ref-117-ingots.jsx"),
            PropKind::Bellows => include_str!("../../../design/art/refs/ref-118-bellows.jsx"),
            PropKind::WeaponRack => {
                include_str!("../../../design/art/refs/ref-119-weaponrack.jsx")
            }
            PropKind::Workbench => include_str!("../../../design/art/refs/ref-120-workbench.jsx"),
            PropKind::BucketShovel => {
                include_str!("../../../design/art/refs/ref-121-bucketshovel.jsx")
            }
            PropKind::Strongbox => include_str!("../../../design/art/refs/ref-128-strongbox.jsx"),
            PropKind::Planter => include_str!("../../../design/art/refs/ref-129-planter.jsx"),
            PropKind::Guildhall => include_str!("../../../design/art/refs/ref-123-guildhall.jsx"),
            PropKind::LedgerDesk => {
                include_str!("../../../design/art/refs/ref-124-ledgerdesk.jsx")
            }
            PropKind::NoticeBoard => {
                include_str!("../../../design/art/refs/ref-125-noticeboard.jsx")
            }
            PropKind::GuildBanner => {
                include_str!("../../../design/art/refs/ref-126-guildbanner.jsx")
            }
            PropKind::ScrollShelf => {
                include_str!("../../../design/art/refs/ref-127-scrollshelf.jsx")
            }
            PropKind::MagesTower => {
                include_str!("../../../design/art/refs/ref-131-magestower.jsx")
            }
            PropKind::Orrery => include_str!("../../../design/art/refs/ref-132-orrery.jsx"),
            PropKind::RuneStone => include_str!("../../../design/art/refs/ref-133-runestone.jsx"),
            PropKind::Lectern => include_str!("../../../design/art/refs/ref-134-lectern.jsx"),
            PropKind::Crystal => include_str!("../../../design/art/refs/ref-135-crystal.jsx"),
            PropKind::Cauldron => include_str!("../../../design/art/refs/ref-136-cauldron.jsx"),
            PropKind::Telescope => include_str!("../../../design/art/refs/ref-137-telescope.jsx"),
            PropKind::Tavern => include_str!("../../../design/art/refs/ref-139-tavern.jsx"),
            PropKind::TrestleTable => {
                include_str!("../../../design/art/refs/ref-140-trestletable.jsx")
            }
            PropKind::KegRack => include_str!("../../../design/art/refs/ref-141-kegrack.jsx"),
            PropKind::Firepit => include_str!("../../../design/art/refs/ref-142-firepit.jsx"),
            PropKind::TavernSign => {
                include_str!("../../../design/art/refs/ref-143-tavernsign.jsx")
            }
            PropKind::RoundTable => {
                include_str!("../../../design/art/refs/ref-144-roundtable.jsx")
            }
            PropKind::TabSlate => include_str!("../../../design/art/refs/ref-145-tabslate.jsx"),
        })
    }

    /// The rows of a ref's pixel grid.
    fn grid_rows(src: &str) -> Vec<&str> {
        src.lines()
            .filter_map(|l| l.trim_start().strip_prefix('\'')?.split('\'').next())
            .collect()
    }

    /// World pixels (the bottom-left corner of each) covered by an opaque
    /// pixel of a plaza prop anchored at `base` (mirrored kinds drawn
    /// mirrored), or `None` for a kind without a ref grid.
    fn opaque_pixels(kind: PropKind, base: Vec2) -> Option<std::collections::HashSet<(i32, i32)>> {
        let grid = grid_rows(ref_src(kind)?);
        let (w, h) = (grid[0].len() as i32, grid.len() as i32);
        let (bx, by) = (base.x.round() as i32, base.y.round() as i32);
        let mut out = std::collections::HashSet::new();
        for (r, row) in grid.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                if ch == '.' {
                    continue;
                }
                let c = if kind.mirrored() {
                    w - 1 - c as i32
                } else {
                    c as i32
                };
                out.insert((bx + c - w / 2, by + h - 1 - r as i32));
            }
        }
        Some(out)
    }

    /// True when two props visibly overlap: for two plaza props, any
    /// opaque pixel in common (a fence may run into a gate pillar's empty
    /// margin); otherwise their whole sprite rectangles.
    fn props_overlap(a: (PropKind, Vec2), b: (PropKind, Vec2)) -> bool {
        match (opaque_pixels(a.0, a.1), opaque_pixels(b.0, b.1)) {
            (Some(pa), Some(pb)) => !pa.is_disjoint(&pb),
            _ => a.0.sprite_rect_at(a.1).overlaps(&b.0.sprite_rect_at(b.1)),
        }
    }

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
        let grid = grid_rows(src);
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
                        !props_overlap((kind, base), (*k, *b)),
                        "{kind:?} at {base:?} overlaps the {k:?} at {b:?}"
                    );
                }
            }
        }
        // Each dressed plaza is dressed densely across its area (the
        // density pass): landmark included, 28 to 48 props in Hearthstone
        // (600 px across), 22 to 44 in each 520 px plaza.
        for (name, range) in [
            ("Hearthstone Square", 28..=48),
            ("Market Row", 22..=44),
            ("The Commons", 22..=44),
            ("The Forge", 22..=44),
            ("Guildhall Quarter", 22..=44),
            ("The Mage's Tower", 22..=44),
            ("Tavern Row", 22..=44),
        ] {
            let d = district_index(name);
            let n = DISTRICT_PROPS.iter().filter(|(pd, _, _)| *pd == d).count()
                + LANDMARKS.iter().filter(|(ld, _)| *ld == d).count();
            assert!(range.contains(&n), "{name} has {n} props");
        }
        // The Forge yard still has its four REF-99..102 props.
        let forge: Vec<PropKind> = DISTRICT_PROPS
            .iter()
            .filter(|(d, _, _)| DISTRICTS[*d].name == "The Forge")
            .map(|(_, k, _)| *k)
            .collect();
        for k in [
            PropKind::Anvil,
            PropKind::QuenchTrough,
            PropKind::CoalHeap,
            PropKind::ToolRack,
        ] {
            assert!(forge.contains(&k), "the Forge lost its {k:?}");
        }
    }

    /// The walking lanes are two tiles (32 px) wide: a body 32 px wide
    /// and 32 px tall, flood-filled from each dressed plaza's street
    /// mouths against every solid, reaches the plaza centre and the front
    /// of the landmark (a tile in front of its base). So nothing in the
    /// dressing narrows a way in below two tiles, including the way round
    /// the Bell Tower at the end of Hearthstone's north street and the
    /// lanes either side of the well.
    #[test]
    fn lanes_from_the_streets_are_two_tiles_wide() {
        use std::collections::{HashSet, VecDeque};
        // Centred on the player's foot box, two tiles each way.
        let body = Aabb {
            min: Vec2::new(-16.0, -29.0),
            max: Vec2::new(16.0, 3.0),
        };
        let solids = every_solid();
        for d in dressed_districts() {
            let dist = &DISTRICTS[d];
            let n = (dist.plaza_half / 4.0).floor() as i32;
            let at = |i: i32, j: i32| dist.center + Vec2::new(i as f32, j as f32) * 4.0;
            let free = |c: (i32, i32)| {
                let b = body.translated(at(c.0, c.1));
                c.0.abs() <= n && c.1.abs() <= n && !solids.iter().any(|s| b.overlaps(s))
            };
            let mut seen: HashSet<(i32, i32)> = HashSet::new();
            let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
            for m in street_mouths(dist) {
                let o = ((m - dist.center) / 4.0).round();
                let c = (o.x as i32, o.y as i32);
                if free(c) && seen.insert(c) {
                    queue.push_back(c);
                }
            }
            assert!(!seen.is_empty(), "{}: no street mouth fits", dist.name);
            while let Some((i, j)) = queue.pop_front() {
                for next in [(i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1)] {
                    if free(next) && seen.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
            let mut targets = vec![("the centre", dist.center)];
            if let Some(k) = LANDMARKS.iter().find(|(ld, _)| *ld == d).map(|(_, k)| *k) {
                let base = crate::plugins::world::landmark_plot_base(d).expect("plot");
                targets.push(("the landmark", Vec2::new(base.x, base.y - 16.0)));
                assert!(k.is_landmark());
            }
            for (what, p) in targets {
                let o = ((p - dist.center) / 4.0).round();
                assert!(
                    seen.contains(&(o.x as i32, o.y as i32)),
                    "{}: no two-tile lane to {what} at {p:?}",
                    dist.name
                );
            }
        }
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

    // --- The demo plazas (R1 wave A2) ---------------------------------------

    /// Index into `DISTRICTS` of the district named `name`.
    fn district_index(name: &str) -> usize {
        DISTRICTS
            .iter()
            .position(|d| d.name == name)
            .unwrap_or_else(|| panic!("no district {name}"))
    }

    /// Each demo footprint is exactly what its ref header says is solid,
    /// re-derived from the approved pixel grid; the visual box is the art's
    /// opaque bounds (mirrored for the west signpost); the ground line is
    /// the art's bottom row.
    #[test]
    fn demo_footprints_match_their_ref_headers() {
        type Part = (
            std::ops::RangeInclusive<usize>,
            Option<std::ops::RangeInclusive<usize>>,
        );
        // (kind, header phrase, solid parts as (rows, cols), visual rows)
        let cases: Vec<(PropKind, &str, Vec<Part>, std::ops::RangeInclusive<usize>)> = vec![
            // Eave (row 18) to base (row 61).
            (
                PropKind::BellTower,
                "solid from\n// the base to the eave",
                vec![(18..=61, None)],
                0..=61,
            ),
            (
                PropKind::Well,
                "solid over the ring",
                vec![(18..=29, None)],
                2..=29,
            ),
            (
                PropKind::Woodpile,
                "solid over the stack",
                vec![(16..=28, None)],
                16..=28,
            ),
            // The post below the board, without the grass tufts.
            (
                PropKind::Signpost,
                "solid only at the post",
                vec![(16..=29, Some(14..=17))],
                6..=30,
            ),
            (
                PropKind::SignpostWest,
                "solid only at the post",
                vec![(16..=29, Some(14..=17))],
                6..=30,
            ),
            // Eave line to legs and ramp, west of the hen (cols 25..30).
            (
                PropKind::ChickenCoop,
                "solid over the coop, not the hen",
                vec![(13..=29, Some(0..=24))],
                3..=29,
            ),
            (
                PropKind::MarketStall,
                "solid from the\n// counter's foot to the awning",
                vec![(8..=56, None)],
                8..=56,
            ),
            (
                PropKind::Crates,
                "solid over the stack",
                vec![(4..=29, None)],
                4..=29,
            ),
            (
                PropKind::Barrels,
                "solid over the group",
                vec![(5..=29, None)],
                5..=29,
            ),
            // Body and wheel; the handle (cols 2..7) is not solid.
            (
                PropKind::Handcart,
                "solid over the body\n// and wheel",
                vec![(12..=29, Some(8..=27))],
                8..=29,
            ),
            // The two pillars, below the arch spring.
            (
                PropKind::CheckersGate,
                "Solid at the two pillars only",
                vec![(18..=61, Some(0..=31)), (18..=61, Some(32..=63))],
                0..=61,
            ),
            (
                PropKind::LanternPost,
                "solid only at the foot",
                vec![(27..=30, None)],
                0..=30,
            ),
            (
                PropKind::Bench,
                "solid over the seat",
                vec![(18..=22, None)],
                10..=28,
            ),
            // Upper rail top to lower rail bottom, edge to edge.
            (
                PropKind::Fence,
                "solid along the rails",
                vec![(13..=23, None)],
                7..=28,
            ),
            // The density pass (REF-116..121, REF-128, REF-129).
            // The frame: the uprights' tops (row 14) to their feet.
            (
                PropKind::Grindstone,
                "solid over the frame",
                vec![(14..=29, None)],
                4..=29,
            ),
            (
                PropKind::Ingots,
                "solid over the stack",
                vec![(18..=30, None)],
                18..=30,
            ),
            (
                PropKind::Bellows,
                "solid over the stand",
                vec![(19..=29, None)],
                5..=29,
            ),
            // Top rail (row 8) to the feet; the hilts above are not solid.
            (
                PropKind::WeaponRack,
                "solid over the rack",
                vec![(8..=29, None)],
                1..=29,
            ),
            (
                PropKind::Workbench,
                "solid over\n// the bench",
                vec![(15..=29, None)],
                9..=29,
            ),
            // The bucket and its handle, west of the shovel (cols 17..27).
            (
                PropKind::BucketShovel,
                "solid over the\n// bucket",
                vec![(14..=29, Some(0..=15))],
                1..=30,
            ),
            (
                PropKind::Strongbox,
                "solid over the\n// chest",
                vec![(12..=29, None)],
                12..=29,
            ),
            (
                PropKind::Planter,
                "solid over the planter",
                vec![(21..=29, None)],
                2..=29,
            ),
        ];
        for (kind, phrase, parts, visual) in cases {
            let src = ref_src(kind).expect("a ref");
            assert!(
                src.replace('\r', "").contains(phrase),
                "{kind:?}: header lost {phrase:?}"
            );
            let want: Vec<Aabb> = parts
                .into_iter()
                .map(|(rows, cols)| grid_box(src, rows, cols))
                .collect();
            assert_eq!(kind.solids_local(), want.as_slice(), "{kind:?} solid");
            let mut v = grid_box(src, visual, None);
            if kind.mirrored() {
                v = Aabb {
                    min: Vec2::new(-v.max.x, v.min.y),
                    max: Vec2::new(-v.min.x, v.max.y),
                };
            }
            assert_eq!(kind.visual_local(), v, "{kind:?} visual");
            let opaque = grid_box(src, 0..=kind.size().y as usize - 1, None);
            assert_eq!(kind.ground_offset(), opaque.min.y, "{kind:?} ground line");
        }
        // The arch between the gate's pillars fits a player's feet.
        let gate = PropKind::CheckersGate.solids_local();
        let foot = crate::plugins::player::PLAYER_FOOT;
        assert!(gate[1].min.x - gate[0].max.x > foot.max.x - foot.min.x);
        // The signpost's post is symmetric, so the mirrored sign shares it.
        let post = PropKind::Signpost.solid_local();
        assert_eq!(post.min.x, -post.max.x);
        assert!(PropKind::SignpostWest.mirrored() && !PropKind::Signpost.mirrored());
    }

    /// The three demo landmarks each stand once on their plaza's
    /// back-centre plot, on clear flagstone with a tile of paving round
    /// them and room to walk past either side. In front of the base a
    /// player draws in front; behind the bell tower's eave or the gate's
    /// lintel they fade; the stall is solid to its top, so nobody stands
    /// behind it.
    #[test]
    fn the_demo_landmarks_head_their_plazas() {
        for (name, k) in [
            ("Hearthstone Square", PropKind::BellTower),
            ("Market Row", PropKind::MarketStall),
            ("The Commons", PropKind::CheckersGate),
        ] {
            let d = district_index(name);
            assert!(LANDMARKS.contains(&(d, k)), "{name}: {k:?}");
            assert!(k.is_landmark());
            let base = crate::plugins::world::landmark_plot_base(d).expect("plot");
            let placed: Vec<Vec2> = all_props()
                .into_iter()
                .filter(|(kk, _)| *kk == k)
                .map(|(_, b)| b)
                .collect();
            assert_eq!(placed, vec![base], "one {k:?}, at the back-centre");
            assert_eq!(k.size(), Vec2::splat(64.0));
            let plot = k.sprite_rect_at(base);
            let grown = Aabb {
                min: plot.min - Vec2::splat(15.0),
                max: plot.max + Vec2::splat(15.0),
            };
            assert!(
                rect_all(&grown, Ground::Flagstone),
                "{k:?}: not on clear flagstone"
            );
            let solid = k.solid_at(base);
            for side in [
                Aabb::from_base(solid.min.x - 16.0, 16.0, solid.min.y, solid.max.y),
                Aabb::from_base(solid.max.x + 16.0, 16.0, solid.min.y, solid.max.y),
            ] {
                assert!(rect_all(&side, Ground::Flagstone), "{k:?}: no way round");
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
            let fades = occludes(
                player_body_at_feet(behind),
                ysort_z(behind.y),
                k.visual_at(base),
                prop_z,
            );
            if k == PropKind::MarketStall {
                assert_eq!(k.visual_at(base), solid, "nothing to stand behind");
            } else {
                assert!(fades, "{k:?}: no fade behind it");
            }
        }
    }

    /// The Checker's Gate is a gate: a player walks north through the
    /// arch, between the pillars, into the court behind; walking into a
    /// pillar stops them.
    #[test]
    fn the_checkers_gate_is_passable_through_its_arch() {
        use crate::plugins::player::PLAYER_FOOT;
        let d = district_index("The Commons");
        let base = crate::plugins::world::landmark_plot_base(d).expect("plot");
        let solids = every_solid();
        let start = Vec2::new(base.x, base.y - 40.0);
        let court = Vec2::new(base.x, base.y + 70.0);
        let end = slide_move(start, court - start, PLAYER_FOOT, &solids);
        assert!(end.distance(court) < 0.01, "stopped in the arch at {end:?}");
        // And back out again.
        let end = slide_move(court, start - court, PLAYER_FOOT, &solids);
        assert!(
            end.distance(start) < 0.01,
            "stopped on the way out at {end:?}"
        );
        // Up into the west pillar: stopped below it.
        let pillar = PropKind::CheckersGate
            .solids_at(base)
            .next()
            .expect("pillar");
        let start = Vec2::new((pillar.min.x + pillar.max.x) * 0.5, base.y - 40.0);
        let end = slide_move(start, Vec2::new(0.0, 110.0), PLAYER_FOOT, &solids);
        assert!(
            PLAYER_FOOT.translated(end).max.y <= pillar.min.y + 0.01,
            "walked through the pillar: {end:?}"
        );
    }

    /// Fence pieces at the same height 32 px apart are one run: their
    /// rails meet edge to edge, so no player squeezes between them. Every
    /// run has at least three pieces. In The Commons the two runs end on
    /// the Checker's Gate, their rails meeting its pillars, so the only
    /// ways past are the arch and the open ends (the flood fill in
    /// `every_talk_spot_in_a_dressed_district_is_reachable` proves nothing
    /// is sealed off).
    #[test]
    fn fences_form_joined_runs() {
        use crate::plugins::player::PLAYER_FOOT;
        let solids = every_solid();
        let foot_w = PLAYER_FOOT.max.x - PLAYER_FOOT.min.x;
        let mut runs_checked = 0usize;
        for (d, district) in DISTRICTS.iter().enumerate() {
            let mut pieces: Vec<Vec2> = DISTRICT_PROPS
                .iter()
                .filter(|(pd, k, _)| *pd == d && *k == PropKind::Fence)
                .map(|(_, _, off)| district.center + *off)
                .collect();
            pieces.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
            // Split into runs: same y, 32 px steps.
            let mut runs: Vec<Vec<Vec2>> = Vec::new();
            for p in pieces {
                match runs.last_mut() {
                    Some(run) if run.last().is_some_and(|q| q.y == p.y && p.x - q.x == 32.0) => {
                        run.push(p)
                    }
                    _ => runs.push(vec![p]),
                }
            }
            for run in &runs {
                assert!(
                    run.len() >= 3,
                    "{}: a fence run of {}",
                    district.name,
                    run.len()
                );
                for pair in run.windows(2) {
                    let a = PropKind::Fence.solid_at(pair[0]);
                    let b = PropKind::Fence.solid_at(pair[1]);
                    assert_eq!(a.max.x, b.min.x, "rails do not meet at {:?}", pair[1]);
                }
                runs_checked += 1;
            }
        }
        assert!(runs_checked >= 3, "Hearthstone's run and the Commons' two");
        // The Commons: one run either side of the gate, each meeting a
        // pillar with no gap a player's feet fit through.
        let d = district_index("The Commons");
        let base = crate::plugins::world::landmark_plot_base(d).expect("plot");
        let pillars: Vec<Aabb> = PropKind::CheckersGate.solids_at(base).collect();
        let fences: Vec<Aabb> = DISTRICT_PROPS
            .iter()
            .filter(|(pd, k, _)| *pd == d && *k == PropKind::Fence)
            .map(|(_, _, off)| PropKind::Fence.solid_at(DISTRICTS[d].center + *off))
            .collect();
        let west = fences
            .iter()
            .filter(|f| f.max.x <= base.x)
            .map(|f| f.max.x)
            .fold(f32::MIN, f32::max);
        let east = fences
            .iter()
            .filter(|f| f.min.x >= base.x)
            .map(|f| f.min.x)
            .fold(f32::MAX, f32::min);
        assert!(
            west >= pillars[0].min.x,
            "west run stops short of the pillar"
        );
        assert!(
            east <= pillars[1].max.x,
            "east run stops short of the pillar"
        );
        // Straight north next to each pillar's outer face: stopped by rail
        // or pillar, never through.
        for x in [
            pillars[0].min.x - foot_w * 0.5,
            pillars[1].max.x + foot_w * 0.5,
        ] {
            let start = Vec2::new(x, base.y - 30.0);
            let end = slide_move(start, Vec2::new(0.0, 100.0), PLAYER_FOOT, &solids);
            assert!(
                end.y < base.y + 20.0,
                "slipped past the gate at x={x}: {end:?}"
            );
        }
    }

    /// Hearthstone Square stays readable for the first minute: no prop
    /// within four tiles of the spawn, the spawn's feet free, and a
    /// straight walk from the spawn reaches Ferris's talk spot (Ferris
    /// stands where the grid put him before the square was dressed).
    /// Coming down the north street the Bell Tower stands across its end,
    /// as the Sealed Stacks do in the Library: the player steps round it
    /// and walks on to the spawn.
    #[test]
    fn hearthstone_keeps_the_spawn_and_ferris_clear() {
        use crate::plugins::npc::TALK_SPOT_OFFSET;
        use crate::plugins::player::PLAYER_FOOT;
        use crate::plugins::world::PLAYER_SPAWN;
        let solids = every_solid();
        let around = Aabb::from_base(
            PLAYER_SPAWN.x,
            64.0,
            PLAYER_SPAWN.y - 64.0,
            PLAYER_SPAWN.y + 64.0,
        );
        for (k, b) in all_props() {
            assert!(
                !k.sprite_rect_at(b).overlaps(&around),
                "{k:?} at {b:?} crowds the spawn"
            );
        }
        assert!(
            solids
                .iter()
                .all(|s| !PLAYER_FOOT.translated(PLAYER_SPAWN).overlaps(s))
        );
        let ferris = NPC_ROSTER
            .iter()
            .position(|s| s.name == "Ferris")
            .expect("Ferris");
        assert_eq!(ferris, 0);
        assert_eq!(
            district_slot(ferris),
            Vec2::new(-228.0, -228.0),
            "Ferris moved"
        );
        let talk = district_slot(ferris) + TALK_SPOT_OFFSET;
        let end = slide_move(PLAYER_SPAWN, talk - PLAYER_SPAWN, PLAYER_FOOT, &solids);
        assert!(
            end.distance(talk) < 0.01,
            "the way to Ferris is blocked at {end:?}"
        );
        // The east, west and south streets lead straight to the spawn.
        let d = &DISTRICTS[district_index("Hearthstone Square")];
        let h = d.plaza_half + 40.0;
        for from in [Vec2::new(h, 8.0), Vec2::new(-h, 8.0), Vec2::new(8.0, -h)] {
            let end = slide_move(from, PLAYER_SPAWN - from, PLAYER_FOOT, &solids);
            assert!(
                end.distance(PLAYER_SPAWN) < 0.01,
                "from {from:?} stopped at {end:?}"
            );
        }
        // The north street: down to the tower, which stops a straight walk,
        // then round either side of it and down the two-tile lane between
        // the well and the barrels (west) or the crates (east).
        let tower = PropKind::BellTower
            .solid_at(crate::plugins::world::landmark_plot_base(0).expect("plot"));
        let street = Vec2::new(8.0, h);
        let straight = slide_move(street, PLAYER_SPAWN - street, PLAYER_FOOT, &solids);
        assert!(
            straight.y > tower.min.y,
            "walked through the tower: {straight:?}"
        );
        let above = tower.max.y - PLAYER_FOOT.min.y + 2.0;
        for side in [-1.0, 1.0] {
            let mut p = street;
            for to in [
                Vec2::new(8.0, above),
                Vec2::new(44.0 * side, above),
                Vec2::new(44.0 * side, 60.0),
                PLAYER_SPAWN,
            ] {
                p = slide_move(p, to - p, PLAYER_FOOT, &solids);
                assert!(p.distance(to) < 0.01, "stopped at {p:?} heading for {to:?}");
            }
        }
    }

    // --- Acts 3-5: the Guildhall Quarter, the Mage's Tower, Tavern Row ------

    /// Each Act 3-5 footprint is exactly what its ref header says is solid,
    /// re-derived from the approved pixel grid; the visual box is the art's
    /// opaque bounds; the ground line is the art's bottom row.
    #[test]
    fn act3_to_5_footprints_match_their_ref_headers() {
        type Part = (
            std::ops::RangeInclusive<usize>,
            Option<std::ops::RangeInclusive<usize>>,
        );
        // (kind, header phrase, solid parts as (rows, cols), visual rows)
        let cases: Vec<(PropKind, &str, Vec<Part>, std::ops::RangeInclusive<usize>)> = vec![
            // The cornice (row 18) to the bottom step.
            (
                PropKind::Guildhall,
                "solid\n// from the steps to the eave",
                vec![(18..=62, None)],
                0..=62,
            ),
            // Desk top (row 15) to the feet; not the ledger or the quill.
            (
                PropKind::LedgerDesk,
                "solid over the desk",
                vec![(15..=29, None)],
                5..=29,
            ),
            // The two posts below the board.
            (
                PropKind::NoticeBoard,
                "solid only at the two posts",
                vec![(23..=29, Some(0..=15)), (23..=29, Some(16..=31))],
                4..=29,
            ),
            // The stone foot.
            (
                PropKind::GuildBanner,
                "solid only at the foot",
                vec![(27..=30, None)],
                0..=30,
            ),
            (
                PropKind::ScrollShelf,
                "solid over the unit",
                vec![(3..=29, None)],
                3..=29,
            ),
            // The gold band under the roof (row 22) to the base.
            (
                PropKind::MagesTower,
                "solid from the base to\n// the eave",
                vec![(22..=61, None)],
                0..=61,
            ),
            // The post and its foot, under the rings.
            (
                PropKind::Orrery,
                "solid at the stand",
                vec![(23..=30, None)],
                1..=30,
            ),
            // The stone, not the grass at its foot (cols 6..8).
            (
                PropKind::RuneStone,
                "solid\n// over the stone",
                vec![(3..=29, Some(9..=22))],
                3..=29,
            ),
            // The post and its foot, under the tome.
            (
                PropKind::Lectern,
                "solid at the post",
                vec![(16..=30, None)],
                4..=30,
            ),
            (
                PropKind::Crystal,
                "solid over the pedestal",
                vec![(20..=30, None)],
                2..=30,
            ),
            // The rim (row 11) to the legs' feet; not the fire's last row.
            (
                PropKind::Cauldron,
                "solid over the cauldron",
                vec![(11..=30, None)],
                3..=31,
            ),
            (
                PropKind::Telescope,
                "solid at the tripod's feet",
                vec![(29..=30, None)],
                3..=30,
            ),
            // The eave (row 21) to the step.
            (
                PropKind::Tavern,
                "solid from the step to the eave",
                vec![(21..=62, None)],
                0..=62,
            ),
            // The table top (row 14) to the trestle feet.
            (
                PropKind::TrestleTable,
                "solid over the table",
                vec![(14..=29, None)],
                7..=29,
            ),
            (
                PropKind::KegRack,
                "solid over\n// the rack",
                vec![(5..=29, None)],
                5..=29,
            ),
            // The stone ring and the posts' feet; not the flames.
            (
                PropKind::Firepit,
                "solid over the stone ring",
                vec![(22..=29, None)],
                4..=29,
            ),
            // The post's foot, at the LEFT of the sprite.
            (
                PropKind::TavernSign,
                "solid at the foot",
                vec![(27..=30, None)],
                2..=30,
            ),
            // The table and its leg, inside the two stools.
            (
                PropKind::RoundTable,
                "solid over the table",
                vec![(9..=29, Some(4..=27))],
                3..=29,
            ),
            // The two legs below the slate.
            (
                PropKind::TabSlate,
                "solid at the legs",
                vec![(24..=29, Some(0..=15)), (24..=29, Some(16..=31))],
                3..=29,
            ),
        ];
        for (kind, phrase, parts, visual) in cases {
            let src = ref_src(kind).expect("a ref");
            assert!(
                src.replace('\r', "").contains(phrase),
                "{kind:?}: header lost {phrase:?}"
            );
            let want: Vec<Aabb> = parts
                .into_iter()
                .map(|(rows, cols)| grid_box(src, rows, cols))
                .collect();
            assert_eq!(kind.solids_local(), want.as_slice(), "{kind:?} solid");
            assert_eq!(
                kind.visual_local(),
                grid_box(src, visual, None),
                "{kind:?} visual"
            );
            let opaque = grid_box(src, 0..=kind.size().y as usize - 1, None);
            assert_eq!(kind.ground_offset(), opaque.min.y, "{kind:?} ground line");
        }
        // A player's feet fit between the notice board's posts and the tab
        // slate's legs (they walk under the board).
        let foot = crate::plugins::player::PLAYER_FOOT;
        for k in [PropKind::NoticeBoard, PropKind::TabSlate] {
            let s = k.solids_local();
            assert_eq!(s.len(), 2, "{k:?}");
            assert!(s[1].min.x - s[0].max.x > foot.max.x - foot.min.x, "{k:?}");
        }
        // The tavern sign's post is at the left: its solid foot lies wholly
        // west of the anchor, the board hangs east of it.
        let sign = PropKind::TavernSign.solid_local();
        assert!(sign.max.x < 0.0, "the sign's foot is not at the left");
        assert!(PropKind::TavernSign.visual_local().max.x > 8.0);
    }

    /// The three Act 3-5 landmarks each stand once on their plaza's
    /// back-centre plot, on a clear tile of their district's own paving all
    /// round, with room to walk past either side; a player in front of the
    /// base draws in front, behind the eave the roof fades over them.
    #[test]
    fn the_act3_to_5_landmarks_head_their_plazas() {
        for (name, k) in [
            ("Guildhall Quarter", PropKind::Guildhall),
            ("The Mage's Tower", PropKind::MagesTower),
            ("Tavern Row", PropKind::Tavern),
        ] {
            let d = district_index(name);
            let ground = DISTRICTS[d].interior_ground();
            assert_ne!(ground, Ground::Flagstone, "{name} has its own ground");
            assert!(LANDMARKS.contains(&(d, k)), "{name}: {k:?}");
            assert!(k.is_landmark());
            let base = crate::plugins::world::landmark_plot_base(d).expect("plot");
            let placed: Vec<Vec2> = all_props()
                .into_iter()
                .filter(|(kk, _)| *kk == k)
                .map(|(_, b)| b)
                .collect();
            assert_eq!(placed, vec![base], "one {k:?}, at the back-centre");
            assert_eq!(k.size(), Vec2::splat(64.0));
            assert_eq!(
                crate::plugins::world::back_landmark_base(d),
                None,
                "{name}: no boss grid"
            );
            let plot = k.sprite_rect_at(base);
            let grown = Aabb {
                min: plot.min - Vec2::splat(15.0),
                max: plot.max + Vec2::splat(15.0),
            };
            assert!(rect_all(&grown, ground), "{k:?}: not on clear {ground:?}");
            let solid = k.solid_at(base);
            for side in [
                Aabb::from_base(solid.min.x - 16.0, 16.0, solid.min.y, solid.max.y),
                Aabb::from_base(solid.max.x + 16.0, 16.0, solid.min.y, solid.max.y),
            ] {
                assert!(rect_all(&side, ground), "{k:?}: no way round");
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
            assert!(
                occludes(
                    player_body_at_feet(behind),
                    ysort_z(behind.y),
                    k.visual_at(base),
                    prop_z
                ),
                "{k:?}: no fade behind it"
            );
        }
    }

    /// The Guildhall Quarter is symmetric about its avenue: every prop but
    /// the signpost at the street mouth has a twin mirrored across x = 0
    /// (banners come in pairs), and the ledger desk stands on the avenue
    /// itself.
    #[test]
    fn the_guildhall_quarter_is_symmetric() {
        let d = district_index("Guildhall Quarter");
        let props: Vec<(PropKind, Vec2)> = DISTRICT_PROPS
            .iter()
            .filter(|(pd, _, _)| *pd == d)
            .map(|(_, k, off)| (*k, *off))
            .collect();
        let mut banners = 0usize;
        for &(k, off) in &props {
            if k == PropKind::GuildBanner {
                banners += 1;
            }
            // The signpost at the street mouth points down the one street.
            if k != PropKind::Signpost {
                let twin = Vec2::new(-off.x, off.y);
                assert!(
                    props.contains(&(k, twin)),
                    "the {k:?} at {off:?} has no twin at {twin:?}"
                );
            }
        }
        assert!(banners >= 6 && banners % 2 == 0, "{banners} banners");
        assert!(props.contains(&(PropKind::LedgerDesk, Vec2::new(0.0, 76.0))));
    }

    /// The Mage's Tower court has its stone ring: eight rune stones at one
    /// distance round the cauldron, evenly spread, with a two-tile gap
    /// between neighbours so the ring is walked through, not round.
    #[test]
    fn the_rune_stones_ring_the_cauldron() {
        let d = district_index("The Mage's Tower");
        let cauldron = DISTRICT_PROPS
            .iter()
            .find(|(pd, k, _)| *pd == d && *k == PropKind::Cauldron)
            .map(|(_, _, off)| *off)
            .expect("a cauldron");
        // The ring's centre: the middle of the cauldron's solid. Stones
        // are measured by their anchors (the foot of each stone).
        let s = PropKind::Cauldron.solid_local();
        let heart = cauldron + Vec2::new(0.0, (s.min.y + s.max.y) * 0.5);
        let mut stones: Vec<Vec2> = DISTRICT_PROPS
            .iter()
            .filter(|(pd, k, off)| {
                *pd == d && *k == PropKind::RuneStone && off.distance(heart) < 120.0
            })
            .map(|(_, _, off)| *off)
            .collect();
        assert_eq!(stones.len(), 8, "the ring has eight stones");
        let r0 = stones[0].distance(heart);
        for st in &stones {
            let r = st.distance(heart);
            assert!(
                (r - r0).abs() <= 2.0,
                "stone {st:?} off the ring ({r} vs {r0})"
            );
        }
        let angle = |p: &Vec2| (p.y - heart.y).atan2(p.x - heart.x);
        stones.sort_by(|a, b| angle(a).total_cmp(&angle(b)));
        for i in 0..stones.len() {
            let a = PropKind::RuneStone.solid_at(stones[i]);
            let b = PropKind::RuneStone.solid_at(stones[(i + 1) % stones.len()]);
            let gap = (b.min.x - a.max.x)
                .max(a.min.x - b.max.x)
                .max(b.min.y - a.max.y)
                .max(a.min.y - b.max.y);
            assert!(gap >= 32.0, "stones {i} and next leave {gap} px");
        }
    }
}
