// SPDX-License-Identifier: MIT
//! Hearthstone Village world rendering.
//!
//! The village is a *designed* layout, not procedural noise: a central plaza
//! (Hearthstone Square) with an orthogonal dirt-road street grid out to a ring
//! of eight themed district plazas. NPCs are placed by [`district_slot`] into
//! those plazas in roster order. Oaks and cottages are sprites, not tiles —
//! see `prop.rs`.
//!
//! Tiles come from the REF-76 village atlas (40 tiles, 8 columns): the 22
//! REF-66 terrain v2 tiles, the 2 REF-75 plaza flagstones, the 2 REF-97
//! Forge slabs, the 2 REF-122 Guildhall floor squares, the 2 REF-130 tower
//! court slabs, the 2 REF-138 tavern planks, the 2 REF-154 vault slabs,
//! the 2 REF-162 carpet runner tiles and the 4 REF-146 Wharf tiles (two
//! dock planks, two harbour water frames), copied verbatim
//! (render-refs tests guard the copy). A tilemap binds one texture, which
//! is why every plaza ground shares the terrain sheet instead of having its
//! own; a new district ground is appended to it, so existing indices never
//! move. Two tilemap layers share it:
//!
//! 1. **Base** — every tile is classified [`Ground::Grass`], `Cobble`,
//!    `Dirt`, `Flagstone` or a district ground (`ForgeSlab`, `GuildFloor`,
//!    `TowerCourt`, `TavernPlanks`, `VaultFloor`, `DockPlanks`) or harbour
//!    `Water` ([`Harbour`]) by [`ground_at_tile`]. Grass, cobble
//!    and dirt variants are picked by a deterministic per-tile hash; a
//!    paving's two tiles are laid by its own rule ([`Ground::paving`],
//!    [`Lay`]: mixed, one in n, a chequer, or a ring round the landmark).
//!    No tinting. Each plaza is its district's interior ground
//!    ([`District::interior`], flagstone unless named) inside a one-tile
//!    cobble border (see [`plaza_zone`]), so cobble is always the paving
//!    that meets grass and roads.
//! 2. **Overlay** — transparent grass-edge tiles on non-grass tiles that border
//!    grass, chosen by the pure [`fringe_for`] from the 8 neighbours, and
//!    the REF-162 carpet runner laid over a plaza's paving ([`Runner`]);
//!    the two never meet (a runner lies on interior tiles only).
//!
//! Harbour water (the Wharf) is a base-layer ground inside the cobble
//! ring: its two REF-146 frames are laid in a chequer and swapped on a slow
//! timer by [`swell_water`], and its tiles are merged into a few solid
//! boxes ([`water_solids`]) so nobody walks on it.
//!
//! Orientation: `bevy_ecs_tilemap` tile `y` grows *upward* in world space,
//! which is also screen-up for our 2D camera, and atlas images are drawn
//! upright. So the art's "north" (top of the tile image) is tile `y + 1`.

use crate::assets::SPRITE_TILES_VILLAGE_V3;
use crate::plugins::state::GameState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

const TILE_PX: f32 = 16.0;
const MAP_W: u32 = 180;
/// 220 tiles (was 140): the north-south main street runs on to two more
/// districts, the Forbidden Library (Act 9) at its south end and the
/// Throne (Act 10) at its north end. Even, like 140, so every tile centre
/// keeps the same world coordinates relative to the origin.
const MAP_H: u32 = 220;

/// World bounds in world-space (centered on origin). `player` clamps movement
/// to these so the avatar can't walk off the tilemap.
pub const WORLD_HALF_W: f32 = (MAP_W as f32) * TILE_PX * 0.5;
pub const WORLD_HALF_H: f32 = (MAP_H as f32) * TILE_PX * 0.5;

// --- Tile indices into the REF-76 atlas (8 columns, row-major) ----
// Index = declaration order in `design/art/refs/ref-76-village-atlas.jsx`;
// `atlas_constants_match_ref76_tile_order` pins every constant below to it.
const T_GRASS_A: u32 = 0;
const T_GRASS_B: u32 = 1;
const T_GRASS_TUFT: u32 = 2;
const T_FLOWERS_GOLD: u32 = 3;
const T_FLOWERS_ROSE: u32 = 4;
const T_COBBLE_A: u32 = 5;
const T_COBBLE_B: u32 = 6;
const T_DIRT_A: u32 = 7;
const T_DIRT_B: u32 = 8;
// 9 = water_a: in the atlas, not yet used by the layout.
const T_FRINGE_N: u32 = 10;
const T_FRINGE_S: u32 = 11;
const T_FRINGE_W: u32 = 12;
const T_FRINGE_E: u32 = 13;
const T_CORNER_NW: u32 = 14;
const T_CORNER_NE: u32 = 15;
const T_CORNER_SW: u32 = 16;
const T_CORNER_SE: u32 = 17;
const T_NUB_NW: u32 = 18;
const T_NUB_NE: u32 = 19;
const T_NUB_SW: u32 = 20;
const T_NUB_SE: u32 = 21;
/// REF-75 plaza flagstone, two interchangeable variants.
const T_FLAG_A: u32 = 22;
const T_FLAG_B: u32 = 23;
/// REF-97 Forge yard slab: `soot_a` plain, `soot_b` with a scorch mark
/// (about one tile in six, see [`SOOT_B_ONE_IN`]).
const T_SOOT_A: u32 = 24;
const T_SOOT_B: u32 = 25;
/// One Forge slab tile in this many carries the scorch mark (REF-97).
const SOOT_B_ONE_IN: u32 = 6;
/// REF-122 Guildhall floor: aged-paper and stone-grey squares, laid as a
/// chequer (light where tile x + tile y is even).
const T_HALL_LIGHT: u32 = 26;
const T_HALL_DARK: u32 = 27;
/// REF-130 tower court: a plain slab and a slab with a teal rune inlaid,
/// the rune slab laid in a ring round the tower ([`RUNE_RING_RADIUS_PX`]).
const T_COURT_PLAIN: u32 = 28;
const T_COURT_RUNE: u32 = 29;
/// Radius of the REF-130 rune ring, from the centre of the Mage's Tower
/// plot to the tile centres that carry the rune slab.
const RUNE_RING_RADIUS_PX: f32 = 80.0;
/// REF-138 tavern floorboards, two tiles mixed freely.
const T_PLANK_A: u32 = 30;
const T_PLANK_B: u32 = 31;
/// REF-154 vault floor: a plain basalt slab and a slab with a gold ring
/// inlaid, the ring on about one tile in six ([`VAULT_RING_ONE_IN`]).
const T_VAULT_PLAIN: u32 = 32;
const T_VAULT_RING: u32 = 33;
/// One vault slab in this many carries the gold ring (REF-154).
const VAULT_RING_ONE_IN: u32 = 6;
/// REF-162 carpet runner, laid in the overlay layer over a plaza's ground
/// ([`Runner`]): `runner_mid` tiles vertically, `runner_end` is its fringed
/// south end.
const T_RUNNER_MID: u32 = 34;
const T_RUNNER_END: u32 = 35;
/// REF-146 Wharf dock planks, two tiles mixed freely.
const T_DOCK_A: u32 = 36;
const T_DOCK_B: u32 = 37;
/// REF-146 harbour water: two frames of the same tile (REF-146 names them
/// `water_a` and `water_b`; REF-76 keys them `harbour_a` and `harbour_b`
/// because REF-66 already has a `water_a`). Laid in a chequer and swapped
/// on a slow timer ([`swell_water`]), so neighbours are always out of
/// phase.
pub const T_HARBOUR_A: u32 = 38;
pub const T_HARBOUR_B: u32 = 39;
/// Seconds between harbour water frames. REF-146 asks for "every half
/// second for a slow swell"; a touch slower reads calmer at play zoom.
pub const WATER_FRAME_SECS: f32 = 0.6;

/// z of the overlay tilemap; the base layer sits at 0. Y-sorted sprites live
/// above both (see `prop::ysort_z`).
const OVERLAY_Z: f32 = 0.5;

/// A village quarter: a square paved plaza centered at `center`, home to
/// `count` roster NPCs (assigned sequentially by [`district_slot`]). Cottages
/// ring the plaza (see `prop::COTTAGE_SPOTS`).
pub struct District {
    pub name: &'static str,
    pub center: Vec2,
    pub plaza_half: f32,
    pub count: usize,
    /// Paving inside the one-tile cobble ring. `None` keeps the village's
    /// REF-75 flagstone; a dressed district names its own ground (the Forge:
    /// [`Ground::ForgeSlab`]). Must be a paving ([`Ground::is_plaza_interior`]).
    pub interior: Option<Ground>,
    /// Hand-placed cast: one offset from `center` per roster slot of this
    /// district, in roster order, each NPC standing at the thing they work
    /// at. Empty = the default grid ([`district_slot`]). A slot past the
    /// end of the list keeps its grid spot: the Library and the Throne list
    /// their six villagers and leave the boss (the last slot) where the
    /// grid stands him, centred before the back landmark.
    pub cast: &'static [Vec2],
    /// A carpet runner (REF-162) laid over the plaza's ground, or `None`.
    pub runner: Option<Runner>,
    /// Harbour water inside the plaza (the Wharf), or `None`.
    pub harbour: Option<Harbour>,
}

/// Harbour water laid inside a plaza's cobble ring: every interior tile
/// whose centre lies in `basin` is [`Ground::Water`], except the tiles of
/// a jetty, which stay the plaza's paving and run out into it. Both are
/// plaza-local pixel rectangles on tile edges. Water is not walkable: its
/// tiles are merged into a few solid boxes ([`water_solids`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Harbour {
    pub basin: Rect,
    pub jetties: &'static [Rect],
}

/// The Wharf's harbour: a basin six tiles wide along the east side, the
/// side with no street, from the south row of the interior to the north
/// row (x 144..240, y -244..252), and a jetty three tiles wide and four
/// long running out into it on the street's line (the lower cross-street
/// meets the plaza's west edge at y -20..28), leaving two tiles of water
/// beyond its head. The cobble ring round the basin is the quay's rim.
const WHARF_HARBOUR: Harbour = Harbour {
    basin: Rect {
        min: Vec2::new(144.0, -244.0),
        max: Vec2::new(240.0, 252.0),
    },
    jetties: &[Rect {
        min: Vec2::new(144.0, -20.0),
        max: Vec2::new(208.0, 28.0),
    }],
};

/// A REF-162 carpet runner: `tiles_wide` tiles side by side, centred on the
/// plaza's x, over the interior tiles whose centres lie from `from_y` up
/// to `to_y` (offsets from the plaza centre). The southmost row is
/// `runner_end` (the fringe), the rest `runner_mid`. Walkable; drawn in the
/// overlay layer, so the paving shows through the fringe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Runner {
    pub tiles_wide: u32,
    pub from_y: f32,
    pub to_y: f32,
}

/// The runner laid from the plaza's south edge up to the back landmark
/// (the Sealed Stacks, the Throne Dais): two tiles wide, because the
/// plaza's centre line x = 0 falls on a tile seam and a one-tile runner
/// would sit half a tile off the landmark's door. Its top row runs under
/// the landmark's bottom edge; its fringed end is the plaza's last
/// interior row.
const LANDMARK_RUNNER: Runner = Runner {
    tiles_wide: 2,
    from_y: -248.0,
    to_y: 160.0,
};

impl District {
    /// The ground inside this district's cobble ring.
    pub fn interior_ground(&self) -> Ground {
        self.interior.unwrap_or(Ground::Flagstone)
    }
}

/// Least distance between the centres of any two NPCs (in a hand-placed
/// cast, the people gathered at one prop): twice the talk radius. The F
/// key talks to the nearest NPC whose centre is within
/// [`INTERACT_RADIUS_PX`](crate::plugins::npc::INTERACT_RADIUS_PX) of the
/// player's centre, so with centres this far apart the two talk circles
/// never overlap: wherever the player stands, at most one NPC is in range,
/// and the prompt never flips between two people of a group. It also puts
/// every other NPC out of range of an NPC's own talk spot (28 + 26 = 54
/// px, under this).
pub const CAST_MIN_SPACING: f32 = 2.0 * crate::plugins::npc::INTERACT_RADIUS_PX;

/// The Forge cast, offsets from the plaza centre, in roster order. The
/// yard (`prop::LANDMARKS`, `prop::DISTRICT_PROPS`) has the Great Furnace
/// at the back-centre (base (0, 156)) with the anvil and the quench trough
/// either side of its mouth, the bellows and two coal heaps to the west,
/// the work corner (tool rack, weapon rack, workbench) to the north-east,
/// and a stock yard and finishing station across the south half; the
/// street enters through the east edge at about y = 0. Two gatherings:
/// the furnace mouth (Smelter, Riveter, Bondsmith) and the work corner
/// (Tallywright, Keymaster); the Sifter works the coal alone.
const FORGE_CAST: &[Vec2] = &[
    // The Keymaster (`hashmap_basic`): at the workbench, keeping the
    // ledger of what the work corner turns out.
    Vec2::new(208.0, 164.0),
    // The Sifter (`iter_filter`): west of the coal heaps.
    Vec2::new(-152.0, 176.0),
    // The Smelter (`iter_fold`): in front of the furnace mouth.
    Vec2::new(0.0, 124.0),
    // The Tallywright (`iter_enumerate`): counting the finished swords
    // at the weapon rack.
    Vec2::new(152.0, 168.0),
    // The Riveter (`iter_zip`): at the anvil, on its horn (west) side.
    Vec2::new(-88.0, 116.0),
    // The Bondsmith (`closure_move`): east of the quench trough.
    Vec2::new(88.0, 116.0),
];

/// The Hearthstone Square cast (Acts 1-2 prelude), offsets from the plaza
/// centre (the player's spawn), in roster order. The square
/// (`prop::LANDMARKS`, `prop::DISTRICT_PROPS`) has the Bell Tower at the
/// back-centre (base (0, 196)) with the well in front of it (barrels west,
/// crates and a handcart east), the Smith's corner to the north-west, the
/// fenced chicken yard to the north-east, lantern posts and a signpost at
/// the side streets, the Borrow Checker's lantern posts in the south-east
/// and a seating corner in the south-west. The four streets meet at the
/// centre and the cross between them stays open: nothing stands within
/// 64 px of the spawn. Two gatherings: the well (Oracle, Cooper, Herald)
/// and the Borrow Checker's lanterns (the Checker and the Trait Mage).
const HEARTHSTONE_CAST: &[Vec2] = &[
    // Ferris (`intro_let_binding`), the first quest ("find Ferris"):
    // exactly where the grid stood him before the square was dressed,
    // with open paving between him and the spawn.
    Vec2::new(-228.0, -228.0),
    // The Trait Mage (`double_function`): with the Borrow Checker, at
    // the west lantern post.
    Vec2::new(112.0, -152.0),
    // The Borrow Checker (`borrow_preview`, then the trial that ends with
    // the Lifetime Lantern, `bridge_lantern`): 64 px, between his two
    // lantern posts.
    Vec2::new(176.0, -176.0),
    // The Smith (`mut_binding`): at the woodpile in his corner.
    Vec2::new(-232.0, 196.0),
    // The Cartographer (`if_else_sign`, "one road at the fork"): at the
    // east signpost, where the street leaves for The Commons.
    Vec2::new(184.0, 64.0),
    // The Bellringer (`loop_break`, the rope and the bell): at the foot of
    // the Bell Tower, east side.
    Vec2::new(56.0, 212.0),
    // The Oracle (`match_option`): at the well, in front of it, the first
    // villager north of the spawn.
    Vec2::new(0.0, 92.0),
    // The Herald (`struct_basic`): at the well's east side, announcing to
    // the people drawing water.
    Vec2::new(40.0, 132.0),
    // The Cooper (`vec_iter`, "the Cooper's barrel is the lesson"): at
    // the well's west side, his barrels behind him.
    Vec2::new(-40.0, 132.0),
];

/// The Market Row cast, offsets from the plaza centre, in roster order.
/// The market stall stands at the back-centre (base (0, 156)) with its
/// stock either side (the strongbox, crates, barrels, a parked handcart);
/// a second handcart is being unloaded by the street mouth (east edge,
/// between two lantern posts and a signpost), more stock stands in the
/// south-east and along the west wall, and a seating corner fills the
/// south-west. Two gatherings: the stall's counter (the Drillmaster, the
/// Twin and the Forgewright) and the handcart (the Tinker and the
/// Pilgrim); the Linguist reads alone at the bench.
const MARKET_ROW_CAST: &[Vec2] = &[
    // The Twin (`tuple_destructure`): keeping the stall, at the middle of
    // its counter.
    Vec2::new(0.0, 140.0),
    // The Tinker (`while_loop`, "tightens, checks, tightens again"): at
    // the handle of the handcart by the street.
    Vec2::new(104.0, -88.0),
    // The Forgewright (`borrow_mut`): at the counter's east end.
    Vec2::new(56.0, 140.0),
    // The Linguist (`string_vs_str`): at the bench, book in hand.
    Vec2::new(-160.0, -168.0),
    // The Pilgrim (`option_unwrap_or`, the road-walker with the lit
    // lantern): just in from the street, at the handcart.
    Vec2::new(144.0, -136.0),
    // The Drillmaster (`for_in_range`, counting cadence): at the
    // counter's west end.
    Vec2::new(-56.0, 140.0),
];

/// The Commons cast, offsets from the plaza centre, in roster order. The
/// Checker's Gate stands at the back-centre (base (0, 156)) with a fence
/// run either side, so the north of the plaza is a small court (benches,
/// planters) entered through the arch or round the fence ends. Lantern
/// posts and planters line the approach to the gate, two lantern posts
/// and a signpost mark the street mouth (west edge), a seating corner
/// fills the south-east and the stores the south-west. Two gatherings:
/// the gate (the Auditor and the Heraldic Sage, either side of the arch)
/// and the seating corner (the Reckoner, the Chronicler and the
/// Alchemist); the Quartermaster minds the stores alone.
const COMMONS_CAST: &[Vec2] = &[
    // The Reckoner (`closure_basic`, the ledger clerk): at the west
    // bench of the seating corner.
    Vec2::new(64.0, -184.0),
    // The Quartermaster (`slice_basic`, the stores): among the crates and
    // the handcart.
    Vec2::new(-144.0, -152.0),
    // The Auditor (`result_question_mark`, who stops at the first
    // discrepancy): at the gate, west of the arch.
    Vec2::new(-44.0, 124.0),
    // The Chronicler (`derive_debug`): at the east bench.
    Vec2::new(120.0, -184.0),
    // The Alchemist (`iter_map_collect`): at the seating corner's lantern
    // post, by its teal flame.
    Vec2::new(176.0, -156.0),
    // The Heraldic Sage (`enum_match`): at the gate, east of the arch.
    Vec2::new(44.0, 124.0),
];

/// The Guildhall Quarter cast (Act 3), offsets from the plaza centre, in
/// roster order. The quarter is formal and symmetric: the Guildhall at the
/// back-centre (base (0, 156)) between a pair of guild banners, archive
/// wings of scroll shelves and strongboxes either side of it, the ledger
/// desk in front of its steps, a notice board in each wing, planters
/// lining the avenue, and a garden of benches to the south; the street
/// enters through the east edge at about y = 0. Three gatherings: the
/// ledger desk (the Guildmaster and the Recruiter, who both work on the
/// guild's `Hero` record: its methods, its `new`), the west notice board
/// (the Surveyor and the Locksmith) and the east one (the Armorer and the
/// Porter).
const GUILDHALL_CAST: &[Vec2] = &[
    // The Guildmaster (`impl_method`, then the Variant Wraith trial): at
    // the ledger desk, west side, before the hall steps.
    Vec2::new(-40.0, 72.0),
    // The Recruiter (`assoc_new`, `Hero::new`): signing recruits in at
    // the ledger desk, east side.
    Vec2::new(40.0, 72.0),
    // The Locksmith (`if_let`): at the west notice board, east of it.
    Vec2::new(-128.0, 100.0),
    // The Porter (`while_let`, the stack he works through): at the east
    // notice board, west of it, putting up the day's postings.
    Vec2::new(128.0, 100.0),
    // The Surveyor (`tuple_struct`, `Meters`): at the west notice board,
    // west of it, posting measurements.
    Vec2::new(-208.0, 100.0),
    // The Armorer (`enum_data_match`, items that carry data): at the east
    // notice board, east of it.
    Vec2::new(208.0, 100.0),
];

/// The Mage's Tower cast (Act 4), offsets from the plaza centre, in
/// roster order. The court is scholarly and a little strange: the tower
/// at the back-centre (base (0, 156)) with a crystal either side of its
/// door, the lectern before it, a reading corner of scroll shelves to the
/// north-west, the orrery and the telescopes to the east, and a ring of
/// rune stones round the cauldron in the south; rune slabs ring the tower
/// in the floor. The street enters through the west edge at about y = 0.
/// Three gatherings: the lectern (Vexis and the Lanternkeeper), the orrery
/// (the Wandwright and the Loremaster) and the cauldron (the Conjurer and
/// the Familiar).
const TOWER_CAST: &[Vec2] = &[
    // Vexis the Archmage (`trait_def`, then the Chimera trial): at the
    // lectern before his tower, west side.
    Vec2::new(-40.0, 84.0),
    // The Wandwright (`generic_fn`): at the orrery, west side.
    Vec2::new(128.0, 60.0),
    // The Conjurer (`generic_struct`, `Pair<T>`): at the cauldron, west
    // side, inside the stone ring.
    Vec2::new(-44.0, -124.0),
    // The Familiar (`dyn_trait`, elements of every kind in one `Vec`): at
    // the cauldron, east side.
    Vec2::new(44.0, -124.0),
    // The Lanternkeeper (`lifetimes`, the borrow that must not outlive
    // its book): reading at the lectern, east side.
    Vec2::new(40.0, 84.0),
    // The Loremaster (`assoc_type`): at the orrery, east side.
    Vec2::new(208.0, 60.0),
];

/// The Tavern Row cast (Act 5), offsets from the plaza centre, in roster
/// order. The row is crowded and warm: the tavern at the back-centre
/// (base (0, 156)) with the tab slate by its door and keg racks and
/// barrels along its flanks, the firepit to the west, trestle tables to
/// the east, round tables and stools across the south, and a tavern sign
/// at each street mouth. Streets enter from the north (behind the tavern),
/// the south, and the west and east at about y = -104. Three gatherings:
/// the tavern door (the Barkeep, by his tab slate, and the Bouncer), the
/// firepit (the Interpreter and the Mixologist) and the trestle tables
/// (the Tabkeeper and the Cellarer).
const TAVERN_CAST: &[Vec2] = &[
    // The Barkeep (`result_match`, then the Wager trial): at the tavern
    // door, west side, by the tab slate.
    Vec2::new(-40.0, 124.0),
    // The Bouncer (`custom_error`): at the tavern door, east side.
    Vec2::new(40.0, 124.0),
    // The Interpreter (`from_error`): at the firepit, east side.
    Vec2::new(-56.0, 20.0),
    // The Mixologist (`option_map`): at the firepit, west side.
    Vec2::new(-136.0, 20.0),
    // The Tabkeeper (`and_then`, the chained tab): at the west trestle
    // table.
    Vec2::new(96.0, 16.0),
    // The Cellarer (`unwrap_or_else`, the cellar's fallback): at the east
    // trestle table.
    Vec2::new(176.0, 16.0),
];

/// The Wharf cast (Act 7), offsets from the plaza centre, in roster order.
/// The Wharf is a working harbour on dock planks: the harbour crane at the
/// back-centre (base (0, 156)) with cargo stacked to its west, the basin
/// along the east side with a jetty out into it, the signal mast on the
/// north-east quay, a spare anchor on the south-east quay, a fish market
/// and a net loft in the west. One street, through the west edge at
/// y -20..28, straight down the promenade to the jetty. Three gatherings:
/// the crane (the Dockmaster, `thread::spawn`, and the Harbormaster,
/// scoped threads: many hands, one load, everyone joined before it is
/// lifted, as REF-147's header puts it), the signal mast (the Signaler,
/// channels, and the Tideforecaster, async: messages sent and awaited,
/// REF-153) and the anchor by the water (the Lighthouse Keeper, shared
/// state behind a lock, and the Tidewatch, atomics, at the bollard). Nobody
/// stands on the water, within a tile of it, or at the stores.
const WHARF_CAST: &[Vec2] = &[
    // The Dockmaster (`thread_spawn`): at the crane's quay, west side.
    Vec2::new(-40.0, 132.0),
    // The Lighthouse Keeper (`arc_mutex`): at the anchor, west side.
    Vec2::new(24.0, -108.0),
    // The Signaler (`mpsc_channel`): at the foot of the signal mast, west.
    Vec2::new(60.0, 204.0),
    // The Tidewatch (`atomic`): at the anchor, east side, by the bollard.
    Vec2::new(104.0, -108.0),
    // The Harbormaster (`thread_scope`): at the crane's quay, east side.
    Vec2::new(32.0, 132.0),
    // The Tideforecaster (`async_fn`): below the signal mast, at the
    // water's edge.
    Vec2::new(100.0, 164.0),
];

/// The Vault cast (Act 8), offsets from the plaza centre, in roster order.
/// The Vault is a strongroom on the darkest floor in the game: the vault
/// door at the back-centre (base (0, 156)), a treasury behind a chain
/// cordon to its west (coin piles and a strongbox), the wardens' key rack
/// and an iron safe to its east, the chained casket in the south-west and
/// the scales behind a second chain in the south-east, and stores and a
/// second cordon along the south wall. Four streets: the main street
/// through the north (behind the door) and south edges, the lower
/// cross-street through the west and east edges at about y = 104. Three
/// gatherings: the chained casket (the Vaultwright, `Box<T>`, as its
/// header asks, and the Strongbox, `Rc<RefCell<T>>`), the scales (the
/// Sharekeeper and the Ghostkeeper, strong and weak counts) and the key
/// rack (the Warden and the Swapwarden, `RefCell` and `Cell`).
const VAULT_CAST: &[Vec2] = &[
    // The Vaultwright (`box_basic`): at the chained casket, west side.
    Vec2::new(-176.0, 4.0),
    // The Sharekeeper (`rc_basic`): at the scales, west side, the chain
    // of many holders behind.
    Vec2::new(96.0, 4.0),
    // The Warden (`refcell`): at the key rack, west side.
    Vec2::new(112.0, 204.0),
    // The Swapwarden (`cell`): at the key rack, east side.
    Vec2::new(192.0, 204.0),
    // The Strongbox (`rc_refcell`): at the chained casket, east side.
    Vec2::new(-96.0, 4.0),
    // The Ghostkeeper (`weak_ref`): at the scales, east side.
    Vec2::new(176.0, 4.0),
];

/// The Forbidden Library cast (Act 9), offsets from the plaza centre, in
/// roster order: the six villagers only. The Librarian, the last slot,
/// keeps the grid spot centred before the Sealed Stacks ([`district_slot`],
/// [`BACK_LANDMARK_DISTRICTS`]). The Library is a hall of stacks: the
/// Sealed Stacks at the back-centre with a candelabra either side, the
/// REF-162 runner from the south wall up to them, three rows of tall
/// bookshelves either side of the runner, reading corners in the north
/// (a desk to the west, a globe to the east) and a lectern in the south.
/// The only street meets the plaza behind the Stacks. Three gatherings:
/// the reading desk (the Shelfmark and the Bookbinder, both working
/// through raw pointers), the globe (the Envoy and the Courier, who carry
/// things across borders and threads) and the lectern (the Macro Monk,
/// with the Sexton at the candelabra beside it).
const LIBRARY_CAST: &[Vec2] = &[
    // The Shelfmark (`raw_pointer`): at the reading desk, west side.
    Vec2::new(-192.0, 180.0),
    // The Envoy (`extern_c`): at the globe, west side.
    Vec2::new(112.0, 180.0),
    // The Macro Monk (`macro_rules`): at the lectern, west side.
    Vec2::new(-152.0, -196.0),
    // The Sexton (`drop_trait`): east of the lectern, at the candelabra.
    Vec2::new(-72.0, -196.0),
    // The Courier (`send_sync`): at the globe, east side.
    Vec2::new(192.0, 180.0),
    // The Bookbinder (`maybe_uninit`): at the reading desk, east side.
    Vec2::new(-112.0, 180.0),
];

/// The Throne cast (Act 10), offsets from the plaza centre, in roster
/// order: the six villagers only; the Compiler, the last slot, keeps the
/// grid spot centred before the Throne Dais. The Throne is a hall of
/// state: the dais at the back-centre between two royal banners, the
/// REF-162 runner two tiles wide from the street mouth to the dais, an
/// aisle of columns with braziers between them, guard statues at the
/// street mouth, the Crown on its plinth beside the top of the aisle,
/// and a court across the north. The only street enters through the
/// south edge. Three gatherings: the Crown (the Assayer, who weighs its
/// gold, and the Registrar), the globe (the Hermit and the Astronomer)
/// and the reading desk (the Copyist and the Choirmaster).
const THRONE_CAST: &[Vec2] = &[
    // The Assayer (`prealloc`): at the Crown, west side.
    Vec2::new(-144.0, 84.0),
    // The Copyist (`cow_borrow`): at the reading desk, west side.
    Vec2::new(-200.0, 188.0),
    // The Astronomer (`const_eval`): at the globe, east side.
    Vec2::new(144.0, 84.0),
    // The Choirmaster (`chunks_exact`): at the reading desk, east side.
    Vec2::new(-120.0, 188.0),
    // The Hermit (`no_std`): at the globe, west side.
    Vec2::new(64.0, 84.0),
    // The Registrar (`semver_caret`): at the Crown, east side, on the
    // way to the dais.
    Vec2::new(-64.0, 84.0),
];

/// Hub + a ring of eight quarters + two districts at the ends of the
/// north-south main street. `count`s sum to the 71-NPC roster
/// (9 + 8*6 + 7 + 7). Roster order fills them, so the earliest (Act 1-2)
/// NPCs gather in the central square, Acts 3-8 spread round the ring (one
/// act per quarter, each named for its act), Act 9 stands in the
/// Forbidden Library and Act 10 at the Throne.
pub const DISTRICTS: &[District] = &[
    // Acts 1-2 (the village prelude, the Borrow Checker among them) fill
    // the hub and the two quarters on the east-west street: the free
    // demo's plazas, dressed in R1 wave A2 (approved 2026-11-18) with a
    // landmark each (`prop::LANDMARKS`), their props
    // (`prop::DISTRICT_PROPS`) and the cast at them. Flagstone ground.
    District {
        name: "Hearthstone Square",
        center: Vec2::new(0.0, 0.0),
        plaza_half: 300.0,
        count: 9,
        interior: None,
        cast: HEARTHSTONE_CAST,
        runner: None,
        harbour: None,
    },
    District {
        name: "Market Row",
        center: Vec2::new(-960.0, 0.0),
        plaza_half: 260.0,
        count: 6,
        interior: None,
        cast: MARKET_ROW_CAST,
        runner: None,
        harbour: None,
    },
    District {
        name: "The Commons",
        center: Vec2::new(960.0, 0.0),
        plaza_half: 260.0,
        count: 6,
        interior: None,
        cast: COMMONS_CAST,
        runner: None,
        harbour: None,
    },
    // Acts 3-5 (approved 2026-11-18), dressed like the Forge: each its own
    // ground (REF-122 chequered hall floor, REF-130 court slabs with a
    // rune ring, REF-138 planks), a landmark (`prop::LANDMARKS`), props
    // (`prop::DISTRICT_PROPS`) and the cast gathered at them.
    // Act 3: the Guildhall Quarter.
    District {
        name: "Guildhall Quarter",
        center: Vec2::new(-720.0, 700.0),
        plaza_half: 260.0,
        count: 6,
        interior: Some(Ground::GuildFloor),
        cast: GUILDHALL_CAST,
        runner: None,
        harbour: None,
    },
    // Act 4: the Trait Mage's Tower (Vexis the Archmage's cast).
    District {
        name: "The Mage's Tower",
        center: Vec2::new(720.0, 700.0),
        plaza_half: 260.0,
        count: 6,
        interior: Some(Ground::TowerCourt),
        cast: TOWER_CAST,
        runner: None,
        harbour: None,
    },
    // Act 5: the Tavern of Tribulations.
    District {
        name: "Tavern Row",
        center: Vec2::new(0.0, 800.0),
        plaza_half: 260.0,
        count: 6,
        interior: Some(Ground::TavernPlanks),
        cast: TAVERN_CAST,
        runner: None,
        harbour: None,
    },
    // Act 6: the Iterator Forge, the first dressed district (R1 wave A1,
    // approved 2026-11-18): REF-97 soot slab inside the cobble ring, the
    // Great Furnace (REF-98) at the back-centre (`prop::LANDMARKS`), the
    // yard props in `prop::DISTRICT_PROPS`, and the cast at their work.
    District {
        name: "The Forge",
        center: Vec2::new(-720.0, -700.0),
        plaza_half: 260.0,
        count: 6,
        interior: Some(Ground::ForgeSlab),
        cast: FORGE_CAST,
        runner: None,
        harbour: None,
    },
    // Act 7: the Concurrent Coast (approved 2026-11-18): REF-146 dock
    // planks, the harbour basin along the east side with a jetty out into
    // it (animated REF-146 water, solid), the harbour crane (REF-147) at
    // the back-centre, its props and the cast gathered at them.
    District {
        name: "The Wharf",
        center: Vec2::new(720.0, -700.0),
        plaza_half: 260.0,
        count: 6,
        interior: Some(Ground::DockPlanks),
        cast: WHARF_CAST,
        runner: None,
        harbour: Some(WHARF_HARBOUR),
    },
    // Act 8: the Vault of Pointers (approved 2026-11-18): REF-154 vault
    // floor, the vault door (REF-155) at the back-centre, its props and
    // the cast gathered at them.
    District {
        name: "The Vault",
        center: Vec2::new(0.0, -800.0),
        plaza_half: 260.0,
        count: 6,
        interior: Some(Ground::VaultFloor),
        cast: VAULT_CAST,
        runner: None,
        harbour: None,
    },
    // Act 9: past the Vault, at the south end of the main street. Six
    // villagers at their props (approved 2026-11-18) and the Librarian
    // (64 px), who keeps his grid spot alone in the centre of the back
    // (north) row, in front of the Sealed Stacks (REF-95,
    // `prop::PropKind::SealedStacks`) at the plaza's back-centre. Flagstone
    // with the REF-162 runner up to the Stacks.
    District {
        name: "The Forbidden Library",
        center: Vec2::new(0.0, -1400.0),
        plaza_half: 260.0,
        count: 7,
        interior: None,
        cast: LIBRARY_CAST,
        runner: Some(LANDMARK_RUNNER),
        harbour: None,
    },
    // Act 10: past Tavern Row, at the north end of the main street. Six
    // villagers at their props (approved 2026-11-18) and the Compiler
    // (64 px), alone in the centre of the back (north) row on his grid
    // spot, in front of the Throne Dais (REF-96,
    // `prop::PropKind::ThroneDais`) at the plaza's back-centre. Flagstone
    // with the REF-162 runner, two tiles wide, up to the dais.
    District {
        name: "The Throne",
        center: Vec2::new(0.0, 1400.0),
        plaza_half: 260.0,
        count: 7,
        interior: None,
        cast: THRONE_CAST,
        runner: Some(LANDMARK_RUNNER),
        harbour: None,
    },
];

/// The street grid: a main cross through the hub plus two cross-streets that
/// pick up the corner quarters. Every segment is axis-aligned. The
/// north-south street runs through Tavern Row and the Vault on to the
/// Throne and the Forbidden Library.
const ROADS: &[(Vec2, Vec2)] = &[
    (Vec2::new(-960.0, 0.0), Vec2::new(960.0, 0.0)), // main street, E-W
    (Vec2::new(0.0, -1400.0), Vec2::new(0.0, 1400.0)), // main street, N-S
    (Vec2::new(-720.0, 700.0), Vec2::new(720.0, 700.0)), // upper cross-street
    (Vec2::new(-720.0, -700.0), Vec2::new(720.0, -700.0)), // lower cross-street
];

/// Roads are `2 * ROAD_HALF_TILES + 1` tiles wide (3), centred on a tile row
/// or column so they come out symmetric.
const ROAD_HALF_TILES: u32 = 1;

// Wide spacing: sprites are 32 units, so 240 leaves ~6 sprite-widths between
// neighbours — they read as individuals scattered across a plaza, not a huddle.
const NPC_GRID_SPACING: f32 = 240.0;

/// Paving an NPC sprite's edge must keep inside its plaza's `plaza_half`:
/// one clear flagstone tile (16) + the cobble ring (16) + up to half a
/// tile (8) because the paved edge lands on the tile grid, not exactly on
/// `plaza_half`. An NPC's centre therefore stays half its sprite plus this
/// inside the plaza edge: 56 px for a 32 px villager, 72 px for a 64 px
/// boss (the Borrow Checker, the Librarian). The size comes from the
/// roster's `native_px`, which `tests/art_wiring.rs` pins to the PNG. With
/// 240 px spacing the outer columns of the 260-half ring plazas stood ON
/// the cobble ring; the grid compresses to fit (ring plazas: 204 px, the
/// Library and the Throne with their 64 px bosses: 188 px across, the hub
/// with the Borrow Checker: 228 px). The Library and the Throne also keep
/// their plaza's back-centre for a 64x64 landmark
/// ([`BACK_LANDMARK_DISTRICTS`]), so their three rows are 152 px apart,
/// shifted south. `tests/npc_placement.rs` checks the real tiles under
/// every NPC.
const NPC_EDGE_MARGIN: f32 = 40.0;

/// Where the player spawns on a new game (`player::spawn_player`): the hub
/// crossroads. The hub's centre grid slot falls exactly here, so its NPC
/// steps [`NPC_SPAWN_CLEARANCE`] north instead of standing on the player.
pub const PLAYER_SPAWN: Vec2 = Vec2::ZERO;
/// A 32 px sprite plus two clear tiles between the NPC and a spawned player.
const NPC_SPAWN_CLEARANCE: f32 = 64.0;

/// Grid spacing along one axis for `n` slots in a plaza of half-size `half`
/// whose largest sprite is `sprite_px`: [`NPC_GRID_SPACING`], compressed so
/// the outermost slots keep half that sprite plus [`NPC_EDGE_MARGIN`] from
/// the plaza edge.
fn axis_spacing(n: usize, half: f32, sprite_px: f32) -> f32 {
    let reach = (n as f32 - 1.0) * 0.5;
    if reach <= 0.0 {
        return NPC_GRID_SPACING;
    }
    let clearance = sprite_px * 0.5 + NPC_EDGE_MARGIN;
    NPC_GRID_SPACING.min((half - clearance).max(0.0) / reach)
}

/// Native size of the largest sprite among roster slots
/// `first..first + count` (32 px if none, e.g. an empty district).
fn largest_sprite_px(first: usize, count: usize) -> f32 {
    crate::plugins::npc::NPC_ROSTER
        .iter()
        .skip(first)
        .take(count)
        .map(|s| s.native_px)
        .fold(32.0, f32::max)
}

/// Columns of every district grid. Rows grow with the district's `count`;
/// a short last row is centred (the Library's seventh slot, the
/// Librarian, stands alone in the middle of its back row).
const NPC_GRID_COLS: usize = 3;

/// Districts with a 64x64 landmark at the back-centre of their plaza (the
/// north edge): the Forbidden Library (REF-95, the Sealed Stacks) and the
/// Throne (REF-96, the Throne Dais). Each district's boss stands
/// in the back row, centred, directly in front of it.
pub const BACK_LANDMARK_DISTRICTS: &[&str] = &["The Forbidden Library", "The Throne"];

/// Size of a back landmark sprite (REF-95/96 are 64x64).
pub const BACK_LANDMARK_PX: f32 = 64.0;

/// How far a back landmark's sprite stays inside the plaza edge: one clear
/// flagstone tile (16) + the cobble ring (16) + up to half a tile (8),
/// the same margin an NPC sprite keeps ([`NPC_EDGE_MARGIN`]).
const BACK_LANDMARK_EDGE_MARGIN: f32 = 40.0;

/// Gap between the top of the back-row boss's sprite and the bottom of the
/// landmark's sprite, so the two never overlap.
pub const BACK_LANDMARK_GAP: f32 = 8.0;

/// The landmark plot of `d`: centred on the plaza, its top edge
/// [`BACK_LANDMARK_EDGE_MARGIN`] inside the plaza's north edge.
fn landmark_plot_of(d: &District) -> Vec2 {
    Vec2::new(
        d.center.x,
        d.center.y + d.plaza_half - BACK_LANDMARK_EDGE_MARGIN - BACK_LANDMARK_PX,
    )
}

fn back_landmark_base_of(d: &District) -> Option<Vec2> {
    BACK_LANDMARK_DISTRICTS
        .contains(&d.name)
        .then(|| landmark_plot_of(d))
}

/// Bottom-centre anchor of the 64x64 landmark plot at the back-centre of
/// district `district`'s plaza, on its interior paving. Every district has
/// one; `prop::LANDMARKS` says which are built. `None` only for an unknown
/// district index.
pub fn landmark_plot_base(district: usize) -> Option<Vec2> {
    DISTRICTS.get(district).map(landmark_plot_of)
}

/// Bottom-centre anchor of district `district`'s back landmark, if its
/// cast grid is laid out in front of one with the boss centred before it
/// ([`BACK_LANDMARK_DISTRICTS`]): the same spot as [`landmark_plot_base`].
pub fn back_landmark_base(district: usize) -> Option<Vec2> {
    DISTRICTS.get(district).and_then(back_landmark_base_of)
}

/// World-space position for the NPC at `index` in the roster: a centered grid
/// slot inside that NPC's district. Replaces the old hand-typed scatter.
pub fn district_slot(index: usize) -> Vec2 {
    let mut acc = 0usize;
    for d in DISTRICTS {
        if index < acc + d.count {
            let local = index - acc;
            // A dressed district places its cast by hand, at their work.
            if let Some(offset) = d.cast.get(local) {
                return d.center + *offset;
            }
            let cols = NPC_GRID_COLS;
            let rows = d.count.div_ceil(cols);
            let col = local % cols;
            let row = local / cols;
            // Slots in this slot's row: a full row, or what is left over.
            let in_row = (d.count - row * cols).min(cols);
            let px = largest_sprite_px(acc, d.count);
            let sx = axis_spacing(cols, d.plaza_half, px);
            let x = d.center.x + (col as f32 - (in_row as f32 - 1.0) * 0.5) * sx;
            let y = match back_landmark_base_of(d) {
                // A landmark stands at the back-centre: the rows fill the
                // span from the front edge clearance up to just in front of
                // its steps, so the back-row boss stands centred before it.
                Some(base) if rows > 1 => {
                    let half_px = px * 0.5;
                    let lo = d.center.y - d.plaza_half + half_px + NPC_EDGE_MARGIN;
                    let hi = base.y - BACK_LANDMARK_GAP - half_px;
                    let sy = NPC_GRID_SPACING.min((hi - lo).max(0.0) / (rows as f32 - 1.0));
                    hi - (rows - 1 - row) as f32 * sy
                }
                _ => {
                    let sy = axis_spacing(rows, d.plaza_half, px);
                    d.center.y + (row as f32 - (rows as f32 - 1.0) * 0.5) * sy
                }
            };
            let mut slot = Vec2::new(x, y);
            if slot.distance(PLAYER_SPAWN) < NPC_SPAWN_CLEARANCE {
                slot.y = PLAYER_SPAWN.y + NPC_SPAWN_CLEARANCE;
            }
            return slot;
        }
        acc += d.count;
    }
    // Safety net if the roster ever outgrows total district capacity: a row
    // along the south edge, deterministic per index so they never stack.
    Vec2::new(
        ((index % 9) as f32 - 4.0) * NPC_GRID_SPACING,
        -WORLD_HALF_H + 64.0,
    )
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("WorldPlugin::build");
        app.add_plugins(TilemapPlugin)
            .init_resource::<WaterSwell>()
            .add_systems(OnEnter(GameState::InGame), spawn_village)
            .add_systems(Update, swell_water.run_if(in_state(GameState::InGame)));
    }
}

/// A harbour water tile in the base tilemap. `odd` is its chequer square
/// (tile x + y odd), so neighbours always show different frames.
#[derive(Component, Clone, Copy, Debug)]
pub struct WaterTile {
    pub odd: bool,
}

/// The harbour swell: which water frame the even squares show, flipped
/// every [`WATER_FRAME_SECS`].
#[derive(Resource, Debug)]
pub struct WaterSwell {
    timer: Timer,
    flipped: bool,
}

impl Default for WaterSwell {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(WATER_FRAME_SECS, TimerMode::Repeating),
            flipped: false,
        }
    }
}

/// The REF-146 water frame a tile on chequer square `odd` shows: the even
/// squares `harbour_a` and the odd ones `harbour_b`, the other way round
/// while the swell is `flipped`. Only ever one of the two water indices.
pub fn water_frame(odd: bool, flipped: bool) -> u32 {
    if odd ^ flipped {
        T_HARBOUR_B
    } else {
        T_HARBOUR_A
    }
}

/// Swap every harbour water tile to its other frame each time the swell
/// timer runs out. No per-frame logging: the swap itself is the whole job.
pub fn swell_water(
    time: Res<Time>,
    mut swell: ResMut<WaterSwell>,
    mut tiles: Query<(&WaterTile, &mut TileTextureIndex)>,
) {
    let flips = swell.timer.tick(time.delta()).times_finished_this_tick();
    if flips % 2 == 0 {
        return;
    }
    swell.flipped = !swell.flipped;
    let flipped = swell.flipped;
    for (w, mut idx) in &mut tiles {
        idx.0 = water_frame(w.odd, flipped);
    }
}

/// World-space center of tile `(x, y)` for a center-anchored tilemap. Matches
/// `bevy_ecs_tilemap`'s `TilemapAnchor::Center` layout so the tile painter and
/// `district_slot` share one coordinate frame.
fn tile_to_world(x: u32, y: u32) -> Vec2 {
    Vec2::new(
        (x as f32 - (MAP_W as f32 - 1.0) * 0.5) * TILE_PX,
        (y as f32 - (MAP_H as f32 - 1.0) * 0.5) * TILE_PX,
    )
}

/// Inverse of [`tile_to_world`]: nearest tile index to a world point, clamped.
fn world_to_tile(w: Vec2) -> (u32, u32) {
    let tx = (w.x / TILE_PX + (MAP_W as f32 - 1.0) * 0.5).round();
    let ty = (w.y / TILE_PX + (MAP_H as f32 - 1.0) * 0.5).round();
    (
        (tx.clamp(0.0, MAP_W as f32 - 1.0)) as u32,
        (ty.clamp(0.0, MAP_H as f32 - 1.0)) as u32,
    )
}

/// What a tile is made of. Drives both the base variant and the overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    Grass,
    /// Plaza border ring (REF-66 cobble).
    Cobble,
    Dirt,
    /// Plaza interior (REF-75 flagstone), the default district ground.
    Flagstone,
    /// Plaza interior of the Forge (REF-97 soot slab), named by the
    /// district's [`District::interior`].
    ForgeSlab,
    /// Plaza interior of the Guildhall Quarter (REF-122 chequered floor).
    GuildFloor,
    /// Plaza interior of the Mage's Tower (REF-130 court slabs).
    TowerCourt,
    /// Plaza interior of Tavern Row (REF-138 floorboards).
    TavernPlanks,
    /// Plaza interior of the Vault (REF-154 basalt slabs).
    VaultFloor,
    /// Plaza interior of the Wharf (REF-146 dock planks).
    DockPlanks,
    /// Harbour water inside a plaza ([`Harbour`], REF-146 water frames).
    /// Not a paving: nobody stands on it and it is solid.
    Water,
}

/// How a paving's two tiles are laid across a plaza: per-district ground
/// data ([`Ground::paving`]), so each district ground states its own rule
/// in one place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lay {
    /// Either tile on any square, half each, by the per-tile hash
    /// (REF-75 flagstone, REF-138 planks).
    Mix,
    /// The second tile on about one square in `n`, by the per-tile hash
    /// (REF-97: the scorch mark on one slab in six).
    OneIn(u32),
    /// A chequer: the first tile where tile x + tile y is even, the second
    /// where it is odd, never at random (REF-122).
    Chequer,
    /// The second tile on a one-tile ring of `radius` px round the centre
    /// of the district's landmark plot, the first everywhere else (REF-130:
    /// the rune slabs ring the Mage's Tower).
    RingRoundLandmark { radius: f32 },
}

/// A plaza paving: its two atlas tiles and how they are laid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paving {
    pub first: u32,
    pub second: u32,
    pub lay: Lay,
}

impl Ground {
    /// The paving a plaza interior of this ground is laid in, or `None`
    /// for grass, cobble and dirt.
    pub fn paving(self) -> Option<Paving> {
        let (first, second, lay) = match self {
            Ground::Grass | Ground::Cobble | Ground::Dirt | Ground::Water => return None,
            Ground::Flagstone => (T_FLAG_A, T_FLAG_B, Lay::Mix),
            Ground::ForgeSlab => (T_SOOT_A, T_SOOT_B, Lay::OneIn(SOOT_B_ONE_IN)),
            Ground::GuildFloor => (T_HALL_LIGHT, T_HALL_DARK, Lay::Chequer),
            Ground::TowerCourt => (
                T_COURT_PLAIN,
                T_COURT_RUNE,
                Lay::RingRoundLandmark {
                    radius: RUNE_RING_RADIUS_PX,
                },
            ),
            Ground::TavernPlanks => (T_PLANK_A, T_PLANK_B, Lay::Mix),
            Ground::VaultFloor => (T_VAULT_PLAIN, T_VAULT_RING, Lay::OneIn(VAULT_RING_ONE_IN)),
            Ground::DockPlanks => (T_DOCK_A, T_DOCK_B, Lay::Mix),
        };
        Some(Paving { first, second, lay })
    }

    /// True for the pavings a plaza interior can be laid in (flagstone or
    /// a district's own ground), false for grass, cobble, dirt and water.
    pub fn is_plaza_interior(self) -> bool {
        self.paving().is_some()
    }
}

/// Where a tile sits relative to one plaza. Ordered so that `max` over all
/// plazas gives the tile's zone (plazas never overlap).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlazaZone {
    Outside,
    /// Inside the plaza with at least one of its 8 neighbours outside: the
    /// one-tile cobble ring, including the corners and where roads enter.
    Border,
    /// Inside the plaza with all 8 neighbours inside too: flagstone.
    Interior,
}

/// Pure zone rule for tile `(x, y)` given an `inside` test (in signed tile
/// coordinates, so callers can answer "outside" for off-map neighbours).
pub fn classify_plaza(x: i64, y: i64, inside: impl Fn(i64, i64) -> bool) -> PlazaZone {
    if !inside(x, y) {
        return PlazaZone::Outside;
    }
    let all_neighbours_inside = (-1..=1)
        .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
        .all(|(dx, dy)| inside(x + dx, y + dy));
    if all_neighbours_inside {
        PlazaZone::Interior
    } else {
        PlazaZone::Border
    }
}

/// True if the centre of tile `(x, y)` lies inside `d`'s plaza square.
/// Off-map tiles are never inside.
fn tile_in_plaza(d: &District, x: i64, y: i64) -> bool {
    if x < 0 || y < 0 || x >= MAP_W as i64 || y >= MAP_H as i64 {
        return false;
    }
    let w = tile_to_world(x as u32, y as u32);
    (w.x - d.center.x).abs() <= d.plaza_half && (w.y - d.center.y).abs() <= d.plaza_half
}

/// Zone of tile `(x, y)` for district `d`.
pub fn plaza_zone(d: &District, x: u32, y: u32) -> PlazaZone {
    classify_plaza(x as i64, y as i64, |tx, ty| tile_in_plaza(d, tx, ty))
}

/// True if tile `(x, y)` lies on a road: within `ROAD_HALF_TILES` (Chebyshev
/// distance, in tiles) of an axis-aligned road segment.
fn on_road(x: u32, y: u32) -> bool {
    ROADS.iter().any(|(a, b)| {
        let (ax, ay) = world_to_tile(*a);
        let (bx, by) = world_to_tile(*b);
        let cx = x.clamp(ax.min(bx), ax.max(bx));
        let cy = y.clamp(ay.min(by), ay.max(by));
        x.abs_diff(cx) <= ROAD_HALF_TILES && y.abs_diff(cy) <= ROAD_HALF_TILES
    })
}

/// Classify tile `(x, y)`: plaza interiors are their district's ground
/// ([`District::interior_ground`], flagstone unless the district names
/// another) inside a one-tile cobble border, the street grid is dirt,
/// everything else (including the map border, where the oak treeline props
/// stand) is grass.
pub fn ground_at_tile(x: u32, y: u32) -> Ground {
    // Plazas never overlap, so the highest zone over all districts is the
    // tile's zone and, for an interior tile, its district is unique.
    let (zone, district) = DISTRICTS
        .iter()
        .map(|d| (plaza_zone(d, x, y), Some(d)))
        .max_by_key(|(z, _)| *z)
        .unwrap_or((PlazaZone::Outside, None));
    if zone == PlazaZone::Interior {
        match district {
            Some(d) if in_harbour_water(d, x, y) => Ground::Water,
            Some(d) => d.interior_ground(),
            None => Ground::Flagstone,
        }
    } else if zone == PlazaZone::Border {
        Ground::Cobble
    } else if on_road(x, y) {
        Ground::Dirt
    } else {
        Ground::Grass
    }
}

/// Ground under a world-space point (clamped to the map).
pub fn ground_at(w: Vec2) -> Ground {
    let (x, y) = world_to_tile(w);
    ground_at_tile(x, y)
}

/// True when tile `(x, y)` (an interior tile of `d`) lies in `d`'s
/// harbour basin and on no jetty.
fn in_harbour_water(d: &District, x: u32, y: u32) -> bool {
    let Some(h) = d.harbour else {
        return false;
    };
    let local = tile_to_world(x, y) - d.center;
    h.basin.contains(local) && !h.jetties.iter().any(|j| j.contains(local))
}

/// True when the world point `w` lies on a harbour water tile.
pub fn on_water(w: Vec2) -> bool {
    ground_at(w) == Ground::Water
}

/// The harbour water as solid boxes, in world space: each row of water
/// tiles split into runs, and a run merged into the box above it when they
/// span the same columns. So the Wharf's basin is three boxes (north of
/// the jetty, beyond its head, south of it), not one per tile. Together
/// they cover exactly the water tiles and never overlap.
pub fn water_solids() -> Vec<crate::plugins::prop::Aabb> {
    use crate::plugins::prop::Aabb;
    let mut out: Vec<Aabb> = Vec::new();
    for d in DISTRICTS.iter().filter(|d| d.harbour.is_some()) {
        let (x0, y0) = world_to_tile(d.center - Vec2::splat(d.plaza_half));
        let (x1, y1) = world_to_tile(d.center + Vec2::splat(d.plaza_half));
        // Boxes still growing upward, and the row they last reached.
        let mut open: Vec<Aabb> = Vec::new();
        for y in y0..=y1 {
            let mut runs: Vec<Aabb> = Vec::new();
            for x in x0..=x1 {
                if ground_at_tile(x, y) != Ground::Water {
                    continue;
                }
                let c = tile_to_world(x, y);
                let tile = Aabb {
                    min: c - Vec2::splat(TILE_PX * 0.5),
                    max: c + Vec2::splat(TILE_PX * 0.5),
                };
                match runs.last_mut() {
                    Some(run) if run.max.x == tile.min.x => run.max.x = tile.max.x,
                    _ => runs.push(tile),
                }
            }
            let mut next: Vec<Aabb> = Vec::new();
            for run in runs {
                let above = open.iter().position(|o| {
                    o.min.x == run.min.x && o.max.x == run.max.x && o.max.y == run.min.y
                });
                match above {
                    Some(i) => {
                        let mut grown = open.swap_remove(i);
                        grown.max.y = run.max.y;
                        next.push(grown);
                    }
                    None => next.push(run),
                }
            }
            out.append(&mut open);
            open = next;
        }
        out.append(&mut open);
    }
    out
}

/// Deterministic per-tile hash (murmur3 finaliser). Well mixed, so variant
/// picks don't line up into stripes or a checkerboard the way a plain
/// `x * a ^ y * b` hash does.
fn tile_hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut h = x
        .wrapping_mul(0x8DA6_B343)
        .wrapping_add(y.wrapping_mul(0xD816_3841))
        .wrapping_add(salt.wrapping_mul(0xCB1A_B31F));
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h
}

/// Atlas index for the base layer at `(x, y)`.
fn base_tile(x: u32, y: u32, ground: Ground) -> u32 {
    let h = tile_hash(x, y, 0);
    match ground {
        Ground::Grass => match h % 100 {
            0..=1 => T_FLOWERS_GOLD,
            2..=3 => T_FLOWERS_ROSE,
            4..=11 => T_GRASS_TUFT,
            12..=55 => T_GRASS_A,
            _ => T_GRASS_B,
        },
        Ground::Cobble => {
            if h % 2 == 0 {
                T_COBBLE_A
            } else {
                T_COBBLE_B
            }
        }
        Ground::Dirt => {
            if h % 2 == 0 {
                T_DIRT_A
            } else {
                T_DIRT_B
            }
        }
        Ground::Flagstone
        | Ground::ForgeSlab
        | Ground::GuildFloor
        | Ground::TowerCourt
        | Ground::TavernPlanks
        | Ground::VaultFloor
        | Ground::DockPlanks => match ground.paving() {
            Some(p) => {
                if lays_second(p.lay, x, y, h) {
                    p.second
                } else {
                    p.first
                }
            }
            None => {
                tracing::warn!(?ground, "paving ground without a paving; flagstone used");
                T_FLAG_A
            }
        },
        // The swell's first frame; `swell_water` swaps it from there.
        Ground::Water => water_frame((x + y) % 2 == 1, false),
    }
}

/// True when the paving rule `lay` puts its second tile on tile `(x, y)`
/// (per-tile hash `h`).
fn lays_second(lay: Lay, x: u32, y: u32, h: u32) -> bool {
    match lay {
        Lay::Mix => h % 2 == 1,
        Lay::OneIn(n) => h % n.max(1) == 0,
        Lay::Chequer => (x + y) % 2 == 1,
        Lay::RingRoundLandmark { radius } => on_landmark_ring(x, y, radius),
    }
}

/// Centre of district `d`'s landmark plot (the middle of its 64x64 sprite).
fn landmark_plot_centre(d: &District) -> Vec2 {
    landmark_plot_of(d) + Vec2::new(0.0, BACK_LANDMARK_PX * 0.5)
}

/// True when tile `(x, y)` is an interior tile of some district whose
/// centre lies within half a tile of `radius` from that district's
/// landmark plot centre: a one-tile ring.
fn on_landmark_ring(x: u32, y: u32, radius: f32) -> bool {
    let Some(d) = DISTRICTS
        .iter()
        .find(|d| plaza_zone(d, x, y) == PlazaZone::Interior)
    else {
        return false;
    };
    let r = tile_to_world(x, y).distance(landmark_plot_centre(d));
    (r - radius).abs() < TILE_PX * 0.5
}

/// The runner tile laid over tile `(x, y)` (REF-162 `runner_mid`, or
/// `runner_end` on the runner's southmost row), or `None` off every
/// district's [`Runner`]. Only interior tiles carry one, and they never
/// carry a grass-edge overlay, so the overlay layer holds both.
fn runner_tile(x: u32, y: u32) -> Option<u32> {
    let w = tile_to_world(x, y);
    DISTRICTS.iter().find_map(|d| {
        let r = d.runner?;
        if plaza_zone(d, x, y) != PlazaZone::Interior {
            return None;
        }
        let local = w - d.center;
        let half = r.tiles_wide as f32 * TILE_PX * 0.5;
        if local.x.abs() >= half || local.y < r.from_y || local.y > r.to_y {
            return None;
        }
        Some(if local.y - TILE_PX < r.from_y {
            T_RUNNER_END
        } else {
            T_RUNNER_MID
        })
    })
}

/// True when the world point `w` lies on a carpet runner's tile.
pub fn on_runner(w: Vec2) -> bool {
    let (x, y) = world_to_tile(w);
    runner_tile(x, y).is_some()
}

/// Pick the grass-edge overlay for a NON-grass tile from which of its eight
/// neighbours are grass (`n` = the tile to the north, and so on).
///
/// - two adjacent sides (n+w, n+e, s+w, s+e) -> the matching `corner_*`
///   (checked first, in that order);
/// - one side -> the matching `fringe_*` ("grass creeps in from that edge");
/// - no side but a diagonal -> the matching `nub_*` (concave corner).
///
/// One overlay tile per cell is a hard limit, so ambiguous cases resolve
/// deterministically by that priority: opposite sides (n+s, a 1-tile strip)
/// show only the first matching fringe, three sides show one corner, and
/// several grass diagonals show only the first of nw, ne, sw, se. The village
/// layout (plazas, 3-wide roads) never produces 1-tile strips, so in practice
/// only multi-diagonal cells (none in the current map) would lose detail.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub fn fringe_for(
    n: bool,
    s: bool,
    w: bool,
    e: bool,
    nw: bool,
    ne: bool,
    sw: bool,
    se: bool,
) -> Option<u32> {
    let corner = match (n, s, w, e) {
        (true, _, true, _) => Some(T_CORNER_NW),
        (true, _, _, true) => Some(T_CORNER_NE),
        (_, true, true, _) => Some(T_CORNER_SW),
        (_, true, _, true) => Some(T_CORNER_SE),
        _ => None,
    };
    if corner.is_some() {
        return corner;
    }
    if n {
        return Some(T_FRINGE_N);
    }
    if s {
        return Some(T_FRINGE_S);
    }
    if w {
        return Some(T_FRINGE_W);
    }
    if e {
        return Some(T_FRINGE_E);
    }
    if nw {
        return Some(T_NUB_NW);
    }
    if ne {
        return Some(T_NUB_NE);
    }
    if sw {
        return Some(T_NUB_SW);
    }
    if se {
        return Some(T_NUB_SE);
    }
    None
}

/// Overlay tile for `(x, y)` given a ground lookup, or `None` for grass tiles
/// and non-grass tiles with no grass around them. Off-map neighbours count as
/// "not grass". North is `y + 1` (see module docs).
fn overlay_at(x: u32, y: u32, ground: &impl Fn(u32, u32) -> Ground) -> Option<u32> {
    if ground(x, y) == Ground::Grass {
        return None;
    }
    let grass = |dx: i32, dy: i32| {
        let nx = x as i64 + dx as i64;
        let ny = y as i64 + dy as i64;
        if nx < 0 || ny < 0 || nx >= MAP_W as i64 || ny >= MAP_H as i64 {
            return false;
        }
        ground(nx as u32, ny as u32) == Ground::Grass
    };
    fringe_for(
        grass(0, 1),
        grass(0, -1),
        grass(-1, 0),
        grass(1, 0),
        grass(-1, 1),
        grass(1, 1),
        grass(-1, -1),
        grass(1, -1),
    )
}

fn spawn_village(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!(
        "spawning Hearthstone Village terrain v2 + flagstone plazas ({MAP_W}x{MAP_H}, {} districts)",
        DISTRICTS.len()
    );

    let texture = asset_server.load(SPRITE_TILES_VILLAGE_V3);
    let map_size = TilemapSize { x: MAP_W, y: MAP_H };
    let tile_size = TilemapTileSize {
        x: TILE_PX,
        y: TILE_PX,
    };
    let grid_size: TilemapGridSize = tile_size.into();
    let map_type = TilemapType::default();

    // Classify once; both layers read it.
    let grounds: Vec<Ground> = (0..MAP_H)
        .flat_map(|y| (0..MAP_W).map(move |x| ground_at_tile(x, y)))
        .collect();
    let ground = |x: u32, y: u32| grounds[(y * MAP_W + x) as usize];

    let base_entity = commands.spawn_empty().id();
    let overlay_entity = commands.spawn_empty().id();
    let mut base_storage = TileStorage::empty(map_size);
    let mut overlay_storage = TileStorage::empty(map_size);
    let (mut grass, mut cobble, mut dirt, mut flagstone, mut district_ground, mut overlays) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let mut runners = 0usize;
    let mut water = 0usize;

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let pos = TilePos { x, y };
            let g = ground(x, y);
            match g {
                Ground::Grass => grass += 1,
                Ground::Cobble => cobble += 1,
                Ground::Dirt => dirt += 1,
                Ground::Flagstone => flagstone += 1,
                Ground::ForgeSlab
                | Ground::GuildFloor
                | Ground::TowerCourt
                | Ground::TavernPlanks
                | Ground::VaultFloor
                | Ground::DockPlanks => district_ground += 1,
                Ground::Water => water += 1,
            }
            let mut tile = commands.spawn(TileBundle {
                position: pos,
                tilemap_id: TilemapId(base_entity),
                texture_index: TileTextureIndex(base_tile(x, y, g)),
                ..default()
            });
            if g == Ground::Water {
                tile.insert(WaterTile {
                    odd: (x + y) % 2 == 1,
                });
            }
            let tile = tile.id();
            base_storage.set(&pos, tile);

            let runner = runner_tile(x, y);
            if runner.is_some() {
                runners += 1;
            }
            if let Some(idx) = overlay_at(x, y, &ground).or(runner) {
                overlays += 1;
                let tile = commands
                    .spawn(TileBundle {
                        position: pos,
                        tilemap_id: TilemapId(overlay_entity),
                        texture_index: TileTextureIndex(idx),
                        ..default()
                    })
                    .id();
                overlay_storage.set(&pos, tile);
            }
        }
    }
    tracing::debug!(
        grass,
        cobble,
        dirt,
        flagstone,
        district_ground,
        overlays,
        runners,
        water,
        "terrain classified"
    );

    // Harbour water is not walkable: its merged boxes are colliders like a
    // prop's (`player::move_player` and the saved-position restore read
    // every `Solid`), despawned when the session ends.
    let solids = water_solids();
    for s in &solids {
        tracing::debug!(min = ?s.min, max = ?s.max, "spawning harbour water collider");
        commands.spawn((
            crate::plugins::prop::Solid(*s),
            DespawnOnExit(GameState::InGame),
        ));
    }
    tracing::debug!(
        water_tiles = water,
        colliders = solids.len(),
        frame_secs = WATER_FRAME_SECS,
        "harbour water laid: chequered frames, swell on a timer"
    );

    commands.entity(base_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: base_storage,
        texture: TilemapTexture::Single(texture.clone()),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..default()
    });
    commands.entity(overlay_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: overlay_storage,
        texture: TilemapTexture::Single(texture),
        tile_size,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0.0, 0.0, OVERLAY_Z),
        ..default()
    });
    tracing::info!("village terrain spawned: base + {overlays} overlay tiles");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shorthand: neighbours given as a string of the grass directions.
    fn f(dirs: &str) -> Option<u32> {
        let has = |d: &str| dirs.split(',').any(|x| x == d);
        fringe_for(
            has("n"),
            has("s"),
            has("w"),
            has("e"),
            has("nw"),
            has("ne"),
            has("sw"),
            has("se"),
        )
    }

    #[test]
    fn no_grass_means_no_overlay() {
        assert_eq!(f(""), None);
    }

    #[test]
    fn single_edges_pick_fringes() {
        assert_eq!(f("n"), Some(T_FRINGE_N));
        assert_eq!(f("s"), Some(T_FRINGE_S));
        assert_eq!(f("w"), Some(T_FRINGE_W));
        assert_eq!(f("e"), Some(T_FRINGE_E));
        // A side wins over the diagonals next to it (they're implied).
        assert_eq!(f("n,nw,ne"), Some(T_FRINGE_N));
    }

    #[test]
    fn adjacent_sides_pick_convex_corners() {
        assert_eq!(f("n,w"), Some(T_CORNER_NW));
        assert_eq!(f("n,e"), Some(T_CORNER_NE));
        assert_eq!(f("s,w"), Some(T_CORNER_SW));
        assert_eq!(f("s,e"), Some(T_CORNER_SE));
        assert_eq!(f("n,w,nw,ne,sw"), Some(T_CORNER_NW));
    }

    #[test]
    fn lone_diagonals_pick_nubs() {
        assert_eq!(f("nw"), Some(T_NUB_NW));
        assert_eq!(f("ne"), Some(T_NUB_NE));
        assert_eq!(f("sw"), Some(T_NUB_SW));
        assert_eq!(f("se"), Some(T_NUB_SE));
    }

    #[test]
    fn multiple_diagonals_resolve_deterministically() {
        assert_eq!(f("ne,sw"), Some(T_NUB_NE));
        assert_eq!(f("nw,se"), Some(T_NUB_NW));
    }

    #[test]
    fn overlay_uses_north_as_y_plus_one() {
        // Cobble everywhere except the row above y=5: the tile at y=5 must get
        // fringe_n, the tile at y=6 is grass and gets nothing.
        let ground = |_x: u32, y: u32| {
            if y >= 6 {
                Ground::Grass
            } else {
                Ground::Cobble
            }
        };
        assert_eq!(overlay_at(10, 5, &ground), Some(T_FRINGE_N));
        assert_eq!(overlay_at(10, 6, &ground), None);
        assert_eq!(overlay_at(10, 4, &ground), None);
    }

    #[test]
    fn roads_are_three_tiles_wide_and_solid_dirt() {
        // Midway along the main E-W street, between the hub and Market Row.
        let (x, y) = world_to_tile(Vec2::new(-500.0, 0.0));
        let dirt: Vec<u32> = (y.saturating_sub(4)..=y + 4)
            .filter(|&ty| ground_at_tile(x, ty) == Ground::Dirt)
            .collect();
        assert_eq!(dirt.len(), 3, "road rows at x={x}: {dirt:?}");
        assert!(dirt.windows(2).all(|p| p[1] == p[0] + 1), "not contiguous");
    }

    #[test]
    fn plaza_centres_are_their_district_ground_and_map_border_is_grass() {
        for d in DISTRICTS {
            assert_eq!(ground_at(d.center), d.interior_ground(), "{}", d.name);
        }
        for x in 0..MAP_W {
            assert_eq!(ground_at_tile(x, 0), Ground::Grass);
            assert_eq!(ground_at_tile(x, MAP_H - 1), Ground::Grass);
        }
    }

    #[test]
    fn base_variants_stay_in_their_family() {
        for (x, y) in [(3, 3), (90, 70), (17, 101), (150, 40)] {
            let g = ground_at_tile(x, y);
            let t = base_tile(x, y, g);
            let ok = match g {
                Ground::Grass => t <= T_FLOWERS_ROSE,
                Ground::Cobble => t == T_COBBLE_A || t == T_COBBLE_B,
                Ground::Dirt => t == T_DIRT_A || t == T_DIRT_B,
                Ground::Flagstone => t == T_FLAG_A || t == T_FLAG_B,
                Ground::ForgeSlab => t == T_SOOT_A || t == T_SOOT_B,
                Ground::GuildFloor => t == T_HALL_LIGHT || t == T_HALL_DARK,
                Ground::TowerCourt => t == T_COURT_PLAIN || t == T_COURT_RUNE,
                Ground::TavernPlanks => t == T_PLANK_A || t == T_PLANK_B,
                Ground::VaultFloor => t == T_VAULT_PLAIN || t == T_VAULT_RING,
                Ground::DockPlanks => t == T_DOCK_A || t == T_DOCK_B,
                Ground::Water => t == T_HARBOUR_A || t == T_HARBOUR_B,
            };
            assert!(ok, "tile {t} wrong for {g:?}");
        }
    }

    #[test]
    fn grass_variants_are_mixed_not_uniform() {
        let mut counts = [0usize; 5];
        for y in 0..40 {
            for x in 0..40 {
                counts[base_tile(x, y, Ground::Grass) as usize] += 1;
            }
        }
        assert!(counts.iter().all(|&c| c > 0), "{counts:?}");
        assert!(counts[T_GRASS_A as usize] + counts[T_GRASS_B as usize] > 1200);
    }

    // --- Flagstone plazas (REF-75 via the REF-76 atlas) ---------------------

    #[test]
    fn classify_plaza_on_a_small_square() {
        // A 5x5 plaza covering tiles 0..=4 on both axes.
        let inside = |x: i64, y: i64| (0..=4).contains(&x) && (0..=4).contains(&y);
        let z = |x, y| classify_plaza(x, y, inside);
        // Corners and edge midpoints are the one-tile border.
        for (x, y) in [
            (0, 0),
            (4, 0),
            (0, 4),
            (4, 4),
            (2, 0),
            (0, 2),
            (4, 2),
            (2, 4),
        ] {
            assert_eq!(z(x, y), PlazaZone::Border, "({x},{y})");
        }
        // The 3x3 core, including the tiles diagonally inside each corner.
        for (x, y) in [(1, 1), (3, 1), (1, 3), (3, 3), (2, 2)] {
            assert_eq!(z(x, y), PlazaZone::Interior, "({x},{y})");
        }
        // Just outside, including the diagonal past a corner.
        for (x, y) in [(-1, -1), (5, 5), (-1, 2), (2, 5)] {
            assert_eq!(z(x, y), PlazaZone::Outside, "({x},{y})");
        }
    }

    /// Inclusive tile bounds `(x0, x1, y0, y1)` of a district's plaza.
    fn plaza_bounds(d: &District) -> (u32, u32, u32, u32) {
        let xs: Vec<u32> = (0..MAP_W)
            .filter(|&x| (0..MAP_H).any(|y| tile_in_plaza(d, x as i64, y as i64)))
            .collect();
        let ys: Vec<u32> = (0..MAP_H)
            .filter(|&y| (0..MAP_W).any(|x| tile_in_plaza(d, x as i64, y as i64)))
            .collect();
        (xs[0], xs[xs.len() - 1], ys[0], ys[ys.len() - 1])
    }

    #[test]
    fn every_plaza_is_its_ground_inside_a_one_tile_cobble_ring() {
        for d in DISTRICTS {
            let (x0, x1, y0, y1) = plaza_bounds(d);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let ring = x == x0 || x == x1 || y == y0 || y == y1;
                    let want = if ring {
                        Ground::Cobble
                    } else if in_harbour_water(d, x, y) {
                        // The Wharf's basin lies inside the ring too.
                        Ground::Water
                    } else {
                        d.interior_ground()
                    };
                    assert_eq!(ground_at_tile(x, y), want, "{} ({x},{y})", d.name);
                }
            }
            // Each corner: cobble, flagstone diagonally inside, not paving
            // diagonally outside.
            for (cx, cy, ix, iy, ox, oy) in [
                (x0, y0, x0 + 1, y0 + 1, x0 - 1, y0 - 1),
                (x1, y0, x1 - 1, y0 + 1, x1 + 1, y0 - 1),
                (x0, y1, x0 + 1, y1 - 1, x0 - 1, y1 + 1),
                (x1, y1, x1 - 1, y1 - 1, x1 + 1, y1 + 1),
            ] {
                assert_eq!(plaza_zone(d, cx, cy), PlazaZone::Border, "{}", d.name);
                assert_eq!(plaza_zone(d, ix, iy), PlazaZone::Interior, "{}", d.name);
                assert_eq!(plaza_zone(d, ox, oy), PlazaZone::Outside, "{}", d.name);
                assert_eq!(ground_at_tile(ox, oy), Ground::Grass, "{}", d.name);
            }
        }
    }

    #[test]
    fn road_entering_the_hub_meets_the_cobble_border() {
        // The N-S main street enters Hearthstone Square through its north and
        // south edges: dirt outside, cobble on the ring, flagstone inside, for
        // all three road columns.
        let hub = &DISTRICTS[0];
        let (_, _, y0, y1) = plaza_bounds(hub);
        let (rx, _) = world_to_tile(Vec2::ZERO);
        for x in rx - ROAD_HALF_TILES..=rx + ROAD_HALF_TILES {
            assert_eq!(ground_at_tile(x, y1 + 1), Ground::Dirt, "north road x={x}");
            assert_eq!(ground_at_tile(x, y1), Ground::Cobble, "north ring x={x}");
            assert_eq!(ground_at_tile(x, y1 - 1), Ground::Flagstone, "x={x}");
            assert_eq!(ground_at_tile(x, y0 - 1), Ground::Dirt, "south road x={x}");
            assert_eq!(ground_at_tile(x, y0), Ground::Cobble, "south ring x={x}");
            assert_eq!(ground_at_tile(x, y0 + 1), Ground::Flagstone, "x={x}");
        }
    }

    #[test]
    fn plaza_interiors_only_touch_paving_and_never_get_an_overlay() {
        // Cobble is always the tile between a plaza interior (flagstone or a
        // district ground) and grass/dirt, so the grass-edge overlay logic
        // never has to deal with interior paving.
        let ground = |x: u32, y: u32| ground_at_tile(x, y);
        for y in 1..MAP_H - 1 {
            for x in 1..MAP_W - 1 {
                let here = ground(x, y);
                if !here.is_plaza_interior() {
                    continue;
                }
                assert_eq!(overlay_at(x, y, &ground), None, "({x},{y})");
                for (dx, dy) in [
                    (-1i32, -1i32),
                    (0, -1),
                    (1, -1),
                    (-1, 0),
                    (1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                ] {
                    let n = ground(x.wrapping_add_signed(dx), y.wrapping_add_signed(dy));
                    // The Wharf's dock is the one paving that meets water.
                    let quay = here == Ground::DockPlanks && n == Ground::Water;
                    assert!(
                        n == here || n == Ground::Cobble || quay,
                        "{here:?} ({x},{y}) touches {n:?}"
                    );
                }
            }
        }
        // And the ring still gets grass edges: the hub's NW corner tile.
        let (x0, _, _, y1) = plaza_bounds(&DISTRICTS[0]);
        assert_eq!(overlay_at(x0, y1, &ground), Some(T_CORNER_NW));
    }

    #[test]
    fn flag_variants_are_roughly_half_each() {
        let (mut a, mut b) = (0usize, 0usize);
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                if ground_at_tile(x, y) == Ground::Flagstone {
                    match base_tile(x, y, Ground::Flagstone) {
                        T_FLAG_A => a += 1,
                        T_FLAG_B => b += 1,
                        t => panic!("flagstone picked tile {t}"),
                    }
                }
            }
        }
        let share = a as f64 / (a + b) as f64;
        assert!((0.45..=0.55).contains(&share), "flag_a {a}, flag_b {b}");
    }

    /// Every atlas constant names the tile at that position in REF-76.
    #[test]
    fn atlas_constants_match_ref76_tile_order() {
        const SRC: &str = include_str!("../../../design/art/refs/ref-76-village-atlas.jsx");
        let names: Vec<&str> = SRC
            .lines()
            .filter_map(|l| l.strip_prefix("  ")?.strip_suffix(": ["))
            .collect();
        assert_eq!(names.len(), 40, "REF-76 tile count: {names:?}");
        let expected: [(u32, &str); 39] = [
            (T_GRASS_A, "grass_a"),
            (T_GRASS_B, "grass_b"),
            (T_GRASS_TUFT, "grass_tuft"),
            (T_FLOWERS_GOLD, "flowers_gold"),
            (T_FLOWERS_ROSE, "flowers_rose"),
            (T_COBBLE_A, "cobble_a"),
            (T_COBBLE_B, "cobble_b"),
            (T_DIRT_A, "dirt_a"),
            (T_DIRT_B, "dirt_b"),
            (T_FRINGE_N, "fringe_n"),
            (T_FRINGE_S, "fringe_s"),
            (T_FRINGE_W, "fringe_w"),
            (T_FRINGE_E, "fringe_e"),
            (T_CORNER_NW, "corner_nw"),
            (T_CORNER_NE, "corner_ne"),
            (T_CORNER_SW, "corner_sw"),
            (T_CORNER_SE, "corner_se"),
            (T_NUB_NW, "nub_nw"),
            (T_NUB_NE, "nub_ne"),
            (T_NUB_SW, "nub_sw"),
            (T_NUB_SE, "nub_se"),
            (T_FLAG_A, "flag_a"),
            (T_FLAG_B, "flag_b"),
            (T_SOOT_A, "soot_a"),
            (T_SOOT_B, "soot_b"),
            (T_HALL_LIGHT, "hall_light"),
            (T_HALL_DARK, "hall_dark"),
            (T_COURT_PLAIN, "court_plain"),
            (T_COURT_RUNE, "court_rune"),
            (T_PLANK_A, "plank_a"),
            (T_PLANK_B, "plank_b"),
            (T_VAULT_PLAIN, "vault_plain"),
            (T_VAULT_RING, "vault_ring"),
            (T_RUNNER_MID, "runner_mid"),
            (T_RUNNER_END, "runner_end"),
            (T_DOCK_A, "dock_a"),
            (T_DOCK_B, "dock_b"),
            (T_HARBOUR_A, "harbour_a"),
            (T_HARBOUR_B, "harbour_b"),
        ];
        for (idx, name) in expected {
            assert_eq!(names[idx as usize], name, "atlas index {idx}");
        }
    }

    /// The atlas the world loads is exactly the render-refs output for REF-76.
    #[test]
    fn game_atlas_is_the_rendered_ref76() {
        let game = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let asset = std::fs::read(game.join("assets").join(SPRITE_TILES_VILLAGE_V3))
            .expect("read game atlas");
        let render =
            std::fs::read(game.join("../design/art/refs/png/REF76.png")).expect("read REF76.png");
        assert!(
            asset == render,
            "tiles_village_v3.png differs from REF76.png"
        );
    }

    // --- District grounds (REF-97 via the REF-76 atlas) --------------------

    /// The districts with a ground of their own, and that ground.
    const NAMED_GROUNDS: [(&str, Ground); 6] = [
        ("Guildhall Quarter", Ground::GuildFloor),
        ("The Mage's Tower", Ground::TowerCourt),
        ("Tavern Row", Ground::TavernPlanks),
        ("The Forge", Ground::ForgeSlab),
        ("The Vault", Ground::VaultFloor),
        ("The Wharf", Ground::DockPlanks),
    ];

    /// The Forge and the Act 3-5 quarters name their own interior ground;
    /// every other district names none and keeps flagstone. Every named
    /// ground is a paving, and no two districts share one.
    #[test]
    fn district_ground_lookup_defaults_to_flagstone() {
        for d in DISTRICTS {
            if let Some(g) = d.interior {
                assert!(g.is_plaza_interior(), "{} names {g:?}", d.name);
            }
            let want = NAMED_GROUNDS
                .iter()
                .find(|(name, _)| *name == d.name)
                .map_or(Ground::Flagstone, |(_, g)| *g);
            assert_eq!(d.interior_ground(), want, "{}", d.name);
        }
        for (name, g) in NAMED_GROUNDS {
            let d = DISTRICTS
                .iter()
                .find(|d| d.name == name)
                .unwrap_or_else(|| panic!("{name}"));
            let (x0, x1, y0, y1) = plaza_bounds(d);
            assert_eq!(ground_at_tile((x0 + x1) / 2, (y0 + y1) / 2), g, "{name}");
            assert_eq!(ground_at(d.center), g, "{name}");
            // Only this district's interior is laid in its ground.
            for y in 0..MAP_H {
                for x in 0..MAP_W {
                    if ground_at_tile(x, y) == g {
                        assert_eq!(plaza_zone(d, x, y), PlazaZone::Interior, "{name} ({x},{y})");
                    }
                }
            }
        }
        // The lookup goes through the tile classifier: the Forge's interior
        // is slab, its ring is still cobble and the street into it dirt.
        let forge = DISTRICTS
            .iter()
            .find(|d| d.name == "The Forge")
            .expect("the Forge");
        let (x0, x1, y0, y1) = plaza_bounds(forge);
        assert_eq!(
            ground_at_tile((x0 + x1) / 2, (y0 + y1) / 2),
            Ground::ForgeSlab
        );
        let (_, street_y) = world_to_tile(forge.center);
        assert_eq!(ground_at_tile(x1, street_y), Ground::Cobble, "east ring");
        assert_eq!(
            ground_at_tile(x1 + 1, street_y),
            Ground::Dirt,
            "east street"
        );
    }

    /// Every interior tile of district `name`, as (tile x, tile y, atlas
    /// index the base layer lays there).
    fn laid_tiles(name: &str) -> Vec<(u32, u32, u32)> {
        let d = DISTRICTS
            .iter()
            .find(|d| d.name == name)
            .unwrap_or_else(|| panic!("{name}"));
        let g = d.interior_ground();
        let (x0, x1, y0, y1) = plaza_bounds(d);
        (y0..=y1)
            .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
            .filter(|&(x, y)| plaza_zone(d, x, y) == PlazaZone::Interior)
            .map(|(x, y)| (x, y, base_tile(x, y, g)))
            .collect()
    }

    /// Each paving names its rule in one place, the Forge's and the
    /// flagstone's unchanged.
    #[test]
    fn each_paving_names_its_two_tiles_and_its_rule() {
        let p = |g: Ground| g.paving().expect("a paving");
        assert_eq!(p(Ground::Flagstone).lay, Lay::Mix);
        assert_eq!(p(Ground::ForgeSlab).lay, Lay::OneIn(SOOT_B_ONE_IN));
        assert_eq!(p(Ground::GuildFloor).lay, Lay::Chequer);
        assert_eq!(
            p(Ground::TowerCourt).lay,
            Lay::RingRoundLandmark {
                radius: RUNE_RING_RADIUS_PX
            }
        );
        assert_eq!(p(Ground::TavernPlanks).lay, Lay::Mix);
        assert_eq!(
            (p(Ground::GuildFloor).first, p(Ground::GuildFloor).second),
            (T_HALL_LIGHT, T_HALL_DARK)
        );
        assert_eq!(p(Ground::DockPlanks).lay, Lay::Mix);
        assert_eq!(
            (p(Ground::DockPlanks).first, p(Ground::DockPlanks).second),
            (T_DOCK_A, T_DOCK_B)
        );
        // Water is no paving: nobody stands on it.
        for g in [Ground::Grass, Ground::Cobble, Ground::Dirt, Ground::Water] {
            assert_eq!(g.paving(), None, "{g:?}");
            assert!(!g.is_plaza_interior());
        }
    }

    /// REF-122: the Guildhall floor is a chequer, light where tile x +
    /// tile y is even, dark where it is odd, never at random.
    #[test]
    fn the_guildhall_floor_is_a_chequer() {
        let tiles = laid_tiles("Guildhall Quarter");
        assert!(tiles.len() > 500, "{} interior tiles", tiles.len());
        for (x, y, t) in tiles {
            let want = if (x + y) % 2 == 0 {
                T_HALL_LIGHT
            } else {
                T_HALL_DARK
            };
            assert_eq!(t, want, "({x},{y})");
        }
    }

    /// REF-130: the rune slab rings the Mage's Tower, one tile wide at
    /// [`RUNE_RING_RADIUS_PX`] from the centre of its plot, and lies nowhere
    /// else; the rest is plain slab. The ring is long enough to read as one
    /// (the plaza edge and the tower hide part of it).
    #[test]
    fn the_rune_slab_rings_the_tower() {
        let d = DISTRICTS
            .iter()
            .find(|d| d.name == "The Mage's Tower")
            .expect("the Mage's Tower");
        let centre = landmark_plot_centre(d);
        let mut runes = 0usize;
        for (x, y, t) in laid_tiles("The Mage's Tower") {
            let r = tile_to_world(x, y).distance(centre);
            let on_ring = (r - RUNE_RING_RADIUS_PX).abs() < TILE_PX * 0.5;
            let want = if on_ring { T_COURT_RUNE } else { T_COURT_PLAIN };
            assert_eq!(t, want, "({x},{y}) r={r}");
            if t == T_COURT_RUNE {
                runes += 1;
                // Every rune tile has a ring neighbour (8-way): one ring,
                // not scattered specks.
                let near = (-1i32..=1)
                    .flat_map(|dy| (-1i32..=1).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| (dx, dy) != (0, 0))
                    .any(|(dx, dy)| {
                        let (nx, ny) = (x.wrapping_add_signed(dx), y.wrapping_add_signed(dy));
                        ground_at_tile(nx, ny) == Ground::TowerCourt
                            && base_tile(nx, ny, Ground::TowerCourt) == T_COURT_RUNE
                    });
                assert!(near, "lone rune tile at ({x},{y})");
            }
        }
        assert!((16..=60).contains(&runes), "{runes} rune tiles");
        // The ring passes in front of the tower door, across the approach.
        let front = landmark_plot_of(d) - Vec2::new(0.0, RUNE_RING_RADIUS_PX - 32.0);
        let (fx, fy) = world_to_tile(front);
        assert_eq!(base_tile(fx, fy, Ground::TowerCourt), T_COURT_RUNE);
    }

    /// REF-138: the two plank tiles are mixed freely, about half each.
    #[test]
    fn tavern_planks_are_mixed_about_half_each() {
        let tiles = laid_tiles("Tavern Row");
        let b = tiles.iter().filter(|(_, _, t)| *t == T_PLANK_B).count();
        let a = tiles.iter().filter(|(_, _, t)| *t == T_PLANK_A).count();
        assert_eq!(a + b, tiles.len());
        let share = a as f64 / tiles.len() as f64;
        assert!((0.4..=0.6).contains(&share), "plank_a {a}, plank_b {b}");
    }

    /// REF-97: `soot_b` (the scorch mark) on about one slab in six.
    #[test]
    fn soot_b_is_about_one_tile_in_six() {
        let (mut a, mut b) = (0usize, 0usize);
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                if ground_at_tile(x, y) == Ground::ForgeSlab {
                    match base_tile(x, y, Ground::ForgeSlab) {
                        T_SOOT_A => a += 1,
                        T_SOOT_B => b += 1,
                        t => panic!("forge slab picked tile {t}"),
                    }
                }
            }
        }
        assert!(a + b > 500, "the Forge interior is {} tiles", a + b);
        let share = b as f64 / (a + b) as f64;
        assert!((0.12..=0.22).contains(&share), "soot_a {a}, soot_b {b}");
    }

    /// REF-154: the gold ring slab on about one vault tile in six, the
    /// plain basalt slab everywhere else.
    #[test]
    fn vault_ring_is_about_one_tile_in_six() {
        assert_eq!(
            Ground::VaultFloor.paving().map(|p| p.lay),
            Some(Lay::OneIn(VAULT_RING_ONE_IN))
        );
        let tiles = laid_tiles("The Vault");
        assert!(tiles.len() > 500, "{} interior tiles", tiles.len());
        let ring = tiles.iter().filter(|(_, _, t)| *t == T_VAULT_RING).count();
        let plain = tiles.iter().filter(|(_, _, t)| *t == T_VAULT_PLAIN).count();
        assert_eq!(ring + plain, tiles.len());
        let share = ring as f64 / tiles.len() as f64;
        assert!((0.12..=0.22).contains(&share), "plain {plain}, ring {ring}");
    }

    /// REF-162: the Library and the Throne each lay a runner two tiles
    /// wide down their centre line, from the plaza's last interior row in
    /// the south (the fringed `runner_end`) up under their back landmark's
    /// bottom edge (`runner_mid`); no other district has one and no tile
    /// off a runner gets a runner tile. Runner tiles are interior tiles, so
    /// they never compete with a grass-edge overlay.
    #[test]
    fn the_runner_leads_up_to_the_back_landmarks() {
        let ground = |x: u32, y: u32| ground_at_tile(x, y);
        let mut laid = 0usize;
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                let Some(t) = runner_tile(x, y) else {
                    continue;
                };
                laid += 1;
                assert!(t == T_RUNNER_MID || t == T_RUNNER_END);
                assert_eq!(overlay_at(x, y, &ground), None, "({x},{y})");
                let d = DISTRICTS
                    .iter()
                    .find(|d| plaza_zone(d, x, y) == PlazaZone::Interior)
                    .expect("a runner tile inside a plaza");
                assert!(BACK_LANDMARK_DISTRICTS.contains(&d.name), "{}", d.name);
            }
        }
        for name in BACK_LANDMARK_DISTRICTS {
            let d = DISTRICTS
                .iter()
                .find(|d| d.name == *name)
                .expect("district");
            assert_eq!(d.runner, Some(LANDMARK_RUNNER), "{name}");
            let base = back_landmark_base_of(d).expect("plot");
            let (_, _, y0, _) = plaza_bounds(d);
            // The two columns either side of the centre line.
            let (cx, _) = world_to_tile(d.center + Vec2::new(8.0, 0.0));
            for x in [cx - 1, cx] {
                let rows: Vec<u32> = (0..MAP_H)
                    .filter(|&y| plaza_zone(d, x, y) == PlazaZone::Interior)
                    .filter(|&y| runner_tile(x, y).is_some())
                    .collect();
                // Contiguous, from the first interior row up.
                assert_eq!(rows.first(), Some(&(y0 + 1)), "{name} x={x}");
                assert!(rows.windows(2).all(|p| p[1] == p[0] + 1), "{name}");
                let top = rows.last().copied().expect("rows");
                assert_eq!(runner_tile(x, y0 + 1), Some(T_RUNNER_END), "{name}");
                for &y in &rows[1..] {
                    assert_eq!(runner_tile(x, y), Some(T_RUNNER_MID), "{name}");
                }
                // The top row runs under the landmark's bottom edge.
                let top_w = tile_to_world(x, top);
                assert!(top_w.y + TILE_PX * 0.5 > base.y, "{name}: stops short");
                assert!(top_w.y - TILE_PX * 0.5 < base.y + 8.0, "{name}: too far");
            }
            // One tile further out either side: no runner.
            assert!((0..MAP_H).all(|y| runner_tile(cx - 2, y).is_none()));
            assert!((0..MAP_H).all(|y| runner_tile(cx + 1, y).is_none()));
            // Walkable: the ground under it is still the plaza's paving.
            assert_eq!(ground_at(d.center), Ground::Flagstone, "{name}");
            assert!(on_runner(d.center), "{name}: the runner crosses the centre");
        }
        assert_eq!(laid, 2 * 2 * 26, "two runners, two tiles by 26 rows");
    }

    /// A hand-placed cast lists exactly one spot per roster slot, and
    /// `district_slot` returns it. In the two boss districts the list
    /// stops one short: the boss (the last slot) keeps his grid spot,
    /// centred before the back landmark, exactly where the grid stood him
    /// before the plaza was dressed.
    #[test]
    fn hand_placed_casts_cover_their_district() {
        let mut first = 0usize;
        for d in DISTRICTS {
            if !d.cast.is_empty() {
                let boss_on_grid = BACK_LANDMARK_DISTRICTS.contains(&d.name);
                let want = if boss_on_grid { d.count - 1 } else { d.count };
                assert_eq!(d.cast.len(), want, "{}", d.name);
                for (i, off) in d.cast.iter().enumerate() {
                    assert_eq!(district_slot(first + i), d.center + *off, "{}", d.name);
                }
                if boss_on_grid {
                    let base = back_landmark_base_of(d).expect("back landmark");
                    let boss = district_slot(first + d.count - 1);
                    // The grid's back row, centred: the span's top, the
                    // landmark gap and half the 64 px boss below the plot.
                    let want = Vec2::new(d.center.x, base.y - BACK_LANDMARK_GAP - 32.0);
                    assert_eq!(boss, want, "{}: the boss left his grid spot", d.name);
                }
            }
            first += d.count;
        }
        let forge = DISTRICTS
            .iter()
            .find(|d| d.name == "The Forge")
            .expect("the Forge");
        assert_eq!(forge.cast, FORGE_CAST);
    }

    /// Every district has a landmark plot at its back-centre; only the
    /// boss districts report it as a back landmark for their grid.
    #[test]
    fn landmark_plots_sit_at_the_back_centre() {
        for (i, d) in DISTRICTS.iter().enumerate() {
            let plot = landmark_plot_base(i).expect("plot");
            assert_eq!(plot.x, d.center.x, "{}", d.name);
            assert_eq!(
                plot.y + BACK_LANDMARK_PX,
                d.center.y + d.plaza_half - BACK_LANDMARK_EDGE_MARGIN,
                "{}",
                d.name
            );
            let back = back_landmark_base(i);
            if BACK_LANDMARK_DISTRICTS.contains(&d.name) {
                assert_eq!(back, Some(plot), "{}", d.name);
            } else {
                assert_eq!(back, None, "{}", d.name);
            }
        }
        assert_eq!(landmark_plot_base(DISTRICTS.len()), None);
    }

    // --- The Wharf's harbour (REF-146) ---------------------------------------

    fn wharf() -> &'static District {
        DISTRICTS
            .iter()
            .find(|d| d.name == "The Wharf")
            .expect("the Wharf")
    }

    /// The Wharf is the only district with water, and its water is the
    /// basin along the east side (no street there) minus the jetty: every
    /// interior tile with its centre at x >= 144 is water except the
    /// jetty's 4 x 3 tiles, which stay dock and continue the street's line;
    /// two tiles of water lie beyond the jetty's head. No water tile is on
    /// the cobble ring, the plaza's centre or the west street mouth.
    #[test]
    fn the_wharf_basin_lies_along_the_east_side_with_a_jetty() {
        let w = wharf();
        assert_eq!(w.harbour, Some(WHARF_HARBOUR));
        assert!(
            DISTRICTS
                .iter()
                .filter(|d| d.harbour.is_some())
                .all(|d| d.name == "The Wharf")
        );
        let (x0, x1, y0, y1) = plaza_bounds(w);
        let (mut water, mut jetty) = (0usize, 0usize);
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                let g = ground_at_tile(x, y);
                if g == Ground::Water {
                    assert_eq!(plaza_zone(w, x, y), PlazaZone::Interior, "({x},{y})");
                    water += 1;
                }
                if plaza_zone(w, x, y) != PlazaZone::Interior {
                    continue;
                }
                let local = tile_to_world(x, y) - w.center;
                let on_jetty =
                    (144.0..208.0).contains(&local.x) && (-20.0..28.0).contains(&local.y);
                if on_jetty {
                    jetty += 1;
                }
                let want = if local.x > 144.0 && !on_jetty {
                    Ground::Water
                } else {
                    Ground::DockPlanks
                };
                assert_eq!(g, want, "({x},{y}) local {local:?}");
            }
        }
        assert_eq!(jetty, 4 * 3, "a jetty four tiles long, three wide");
        // Six columns by the 31 interior rows, less the jetty.
        assert_eq!(water, 6 * 31 - 12);
        // The ring is cobble all round, the street meets dock.
        for y in y0..=y1 {
            assert_eq!(ground_at_tile(x1, y), Ground::Cobble);
        }
        assert_eq!(ground_at(w.center), Ground::DockPlanks);
        let (_, street_y) = world_to_tile(w.center + Vec2::new(0.0, 4.0));
        assert_eq!(
            ground_at_tile(x0 - 1, street_y),
            Ground::Dirt,
            "west street"
        );
        assert_eq!(ground_at_tile(x0 + 1, street_y), Ground::DockPlanks);
        // The jetty is on the street's three rows.
        for dy in [-1i32, 0, 1] {
            let y = street_y.wrapping_add_signed(dy);
            assert_eq!(ground_at_tile(x0 - 1, y), Ground::Dirt);
            let head = world_to_tile(w.center + Vec2::new(200.0, 0.0)).0;
            assert_eq!(ground_at_tile(head, y), Ground::DockPlanks, "jetty head");
            assert_eq!(
                ground_at_tile(head + 1, y),
                Ground::Water,
                "beyond the head"
            );
        }
    }

    /// Every tile in a plaza, as (x, y).
    fn tiles_of(d: &District) -> Vec<(u32, u32)> {
        let (x0, x1, y0, y1) = plaza_bounds(d);
        (y0..=y1)
            .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
            .collect()
    }

    /// The water colliders cover exactly the water tiles: every water
    /// tile's centre (and the whole tile) is inside one box, no other
    /// tile's centre is inside any, the boxes never overlap, their area is
    /// the water's area, and there are three (north of the jetty, beyond
    /// its head, south of it), not one per tile.
    #[test]
    fn water_solids_cover_exactly_the_water_tiles() {
        use crate::plugins::prop::Aabb;
        let solids = water_solids();
        assert_eq!(solids.len(), 3, "{solids:?}");
        let mut area = 0.0;
        for (i, a) in solids.iter().enumerate() {
            area += (a.max.x - a.min.x) * (a.max.y - a.min.y);
            for b in solids.iter().skip(i + 1) {
                assert!(!a.overlaps(b), "{a:?} overlaps {b:?}");
            }
        }
        let mut water = 0usize;
        for (x, y) in tiles_of(wharf()) {
            let c = tile_to_world(x, y);
            let tile = Aabb {
                min: c - Vec2::splat(TILE_PX * 0.5),
                max: c + Vec2::splat(TILE_PX * 0.5),
            };
            let holders: Vec<&Aabb> = solids
                .iter()
                .filter(|s| s.min.cmple(tile.min).all() && s.max.cmpge(tile.max).all())
                .collect();
            if ground_at_tile(x, y) == Ground::Water {
                water += 1;
                assert_eq!(holders.len(), 1, "water tile ({x},{y})");
                assert!(on_water(c));
            } else {
                assert!(holders.is_empty(), "({x},{y}) is solid but not water");
                assert!(
                    solids.iter().all(|s| !s.overlaps(&tile)),
                    "({x},{y}) is touched by a water box"
                );
            }
        }
        assert_eq!(area, water as f32 * TILE_PX * TILE_PX, "area");
        // The boxes line up with the basin and the jetty.
        let w = wharf();
        let local: Vec<(Vec2, Vec2)> = solids
            .iter()
            .map(|s| (s.min - w.center, s.max - w.center))
            .collect();
        for want in [
            (Vec2::new(144.0, -244.0), Vec2::new(240.0, -20.0)),
            (Vec2::new(208.0, -20.0), Vec2::new(240.0, 28.0)),
            (Vec2::new(144.0, 28.0), Vec2::new(240.0, 252.0)),
        ] {
            assert!(local.contains(&want), "{want:?} not in {local:?}");
        }
    }

    /// The harbour water is laid as a chequer of the two REF-146 frames,
    /// so neighbours are always out of phase.
    #[test]
    fn harbour_water_is_laid_in_a_chequer_of_its_two_frames() {
        let mut seen = std::collections::HashSet::new();
        for (x, y) in tiles_of(wharf()) {
            if ground_at_tile(x, y) != Ground::Water {
                continue;
            }
            let t = base_tile(x, y, Ground::Water);
            seen.insert(t);
            assert_eq!(t, water_frame((x + y) % 2 == 1, false));
            if ground_at_tile(x + 1, y) == Ground::Water {
                assert_ne!(t, base_tile(x + 1, y, Ground::Water), "({x},{y})");
            }
        }
        assert_eq!(
            seen,
            [T_HARBOUR_A, T_HARBOUR_B].into_iter().collect(),
            "both frames"
        );
    }

    /// The swell swaps every water tile between exactly the two REF-146
    /// frames, once per `WATER_FRAME_SECS`, with neighbours (odd and even
    /// squares) always showing different frames, and does nothing between
    /// swaps.
    #[test]
    fn the_swell_swaps_between_exactly_the_two_water_frames() {
        use bevy::ecs::system::RunSystemOnce;
        use std::time::Duration;
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(WaterSwell::default());
        let even = world
            .spawn((
                WaterTile { odd: false },
                TileTextureIndex(water_frame(false, false)),
            ))
            .id();
        let odd = world
            .spawn((
                WaterTile { odd: true },
                TileTextureIndex(water_frame(true, false)),
            ))
            .id();
        let index = |w: &World, e: Entity| w.get::<TileTextureIndex>(e).map(|t| t.0);
        let mut seen = std::collections::HashSet::new();
        let mut swaps = 0usize;
        // A quarter of the timer's own duration, a nanosecond over, so the
        // fourth step always reaches the next swap.
        let step = Duration::from_secs_f32(WATER_FRAME_SECS) / 4 + Duration::from_nanos(1);
        let mut last = index(&world, even);
        for _ in 0..40 {
            world.resource_mut::<Time>().advance_by(step);
            world.run_system_once(swell_water).expect("swell runs");
            let (e, o) = (index(&world, even), index(&world, odd));
            let (e, o) = (e.expect("even tile"), o.expect("odd tile"));
            assert!([T_HARBOUR_A, T_HARBOUR_B].contains(&e), "{e}");
            assert!([T_HARBOUR_A, T_HARBOUR_B].contains(&o), "{o}");
            assert_ne!(e, o, "neighbours in phase");
            seen.insert(e);
            if Some(e) != last {
                swaps += 1;
            }
            last = Some(e);
        }
        assert_eq!(seen.len(), 2);
        // 40 quarter-frames: one swap every four steps.
        assert_eq!(swaps, 10, "one swap per {WATER_FRAME_SECS} s");
        assert!((0.6..=0.8).contains(&WATER_FRAME_SECS));
    }
}
