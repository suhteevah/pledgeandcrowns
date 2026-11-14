// SPDX-License-Identifier: MIT
//! Hearthstone Village world rendering.
//!
//! The village is a *designed* layout, not procedural noise: a central plaza
//! (Hearthstone Square) with an orthogonal dirt-road street grid out to a ring
//! of eight themed district plazas. NPCs are placed by [`district_slot`] into
//! those plazas in roster order. Oaks and cottages are sprites, not tiles —
//! see `prop.rs`.
//!
//! Tiles come from the REF-66 terrain v2 atlas (22 tiles, 8 columns). Two
//! tilemap layers share it:
//!
//! 1. **Base** — every tile is classified [`Ground::Grass`], `Cobble` or `Dirt`
//!    by [`ground_at_tile`]; the variant (grass_a/b/tuft/flowers, cobble_a/b,
//!    dirt_a/b) is picked by a deterministic per-tile hash. No tinting.
//! 2. **Overlay** — transparent grass-edge tiles on non-grass tiles that border
//!    grass, chosen by the pure [`fringe_for`] from the 8 neighbours.
//!
//! Orientation: `bevy_ecs_tilemap` tile `y` grows *upward* in world space,
//! which is also screen-up for our 2D camera, and atlas images are drawn
//! upright. So the art's "north" (top of the tile image) is tile `y + 1`.

use crate::assets::SPRITE_TILES_VILLAGE_V2;
use crate::plugins::state::GameState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

const TILE_PX: f32 = 16.0;
const MAP_W: u32 = 180;
const MAP_H: u32 = 140;

/// World bounds in world-space (centered on origin). `player` clamps movement
/// to these so the avatar can't walk off the tilemap.
pub const WORLD_HALF_W: f32 = (MAP_W as f32) * TILE_PX * 0.5;
pub const WORLD_HALF_H: f32 = (MAP_H as f32) * TILE_PX * 0.5;

// --- Tile indices into the REF-66 atlas (8 columns, row-major) ----
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

/// z of the overlay tilemap; the base layer sits at 0. Y-sorted sprites live
/// above both (see `prop::ysort_z`).
const OVERLAY_Z: f32 = 0.5;

/// A village quarter: a square cobble plaza centered at `center`, home to
/// `count` roster NPCs (assigned sequentially by [`district_slot`]). Cottages
/// ring the plaza (see `prop::COTTAGE_SPOTS`).
pub struct District {
    pub name: &'static str,
    pub center: Vec2,
    pub plaza_half: f32,
    pub count: usize,
}

/// Hub + a ring of eight quarters, well separated on a big map. `count`s sum to
/// the 57-NPC roster (9 + 8*6). Roster order fills them, so the earliest
/// (Act 1-2) NPCs gather in the central square and later acts spread to the ring.
pub const DISTRICTS: &[District] = &[
    District {
        name: "Hearthstone Square",
        center: Vec2::new(0.0, 0.0),
        plaza_half: 300.0,
        count: 9,
    },
    District {
        name: "Market Row",
        center: Vec2::new(-960.0, 0.0),
        plaza_half: 260.0,
        count: 6,
    },
    District {
        name: "The Commons",
        center: Vec2::new(960.0, 0.0),
        plaza_half: 260.0,
        count: 6,
    },
    District {
        name: "Guildhall Quarter",
        center: Vec2::new(-720.0, 700.0),
        plaza_half: 260.0,
        count: 6,
    },
    District {
        name: "Tradesmen's End",
        center: Vec2::new(720.0, 700.0),
        plaza_half: 260.0,
        count: 6,
    },
    District {
        name: "Tavern Row",
        center: Vec2::new(0.0, 800.0),
        plaza_half: 260.0,
        count: 6,
    },
    District {
        name: "The Wharf",
        center: Vec2::new(-720.0, -700.0),
        plaza_half: 260.0,
        count: 6,
    },
    District {
        name: "Stonebridge",
        center: Vec2::new(720.0, -700.0),
        plaza_half: 260.0,
        count: 6,
    },
    District {
        name: "The Old Gate",
        center: Vec2::new(0.0, -800.0),
        plaza_half: 260.0,
        count: 6,
    },
];

/// The street grid: a main cross through the hub plus two cross-streets that
/// pick up the corner quarters. Every segment is axis-aligned.
const ROADS: &[(Vec2, Vec2)] = &[
    (Vec2::new(-960.0, 0.0), Vec2::new(960.0, 0.0)), // main street, E-W
    (Vec2::new(0.0, -800.0), Vec2::new(0.0, 800.0)), // main street, N-S
    (Vec2::new(-720.0, 700.0), Vec2::new(720.0, 700.0)), // upper cross-street
    (Vec2::new(-720.0, -700.0), Vec2::new(720.0, -700.0)), // lower cross-street
];

/// Roads are `2 * ROAD_HALF_TILES + 1` tiles wide (3), centred on a tile row
/// or column so they come out symmetric.
const ROAD_HALF_TILES: u32 = 1;

// Wide spacing: sprites are 32 units, so 240 leaves ~6 sprite-widths between
// neighbours — they read as individuals scattered across a plaza, not a huddle.
const NPC_GRID_SPACING: f32 = 240.0;

/// World-space position for the NPC at `index` in the roster: a centered grid
/// slot inside that NPC's district. Replaces the old hand-typed scatter.
pub fn district_slot(index: usize) -> Vec2 {
    let mut acc = 0usize;
    for d in DISTRICTS {
        if index < acc + d.count {
            let local = index - acc;
            let cols = 3usize;
            let rows = d.count.div_ceil(cols);
            let col = local % cols;
            let row = local / cols;
            let x = d.center.x + (col as f32 - (cols as f32 - 1.0) * 0.5) * NPC_GRID_SPACING;
            let y = d.center.y + (row as f32 - (rows as f32 - 1.0) * 0.5) * NPC_GRID_SPACING;
            return Vec2::new(x, y);
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
            .add_systems(OnEnter(GameState::InGame), spawn_village);
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
    Cobble,
    Dirt,
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

/// Classify tile `(x, y)`: district plazas are cobble, the street grid is
/// dirt, everything else (including the map border, where the oak treeline
/// props stand) is grass.
pub fn ground_at_tile(x: u32, y: u32) -> Ground {
    let w = tile_to_world(x, y);
    let in_plaza = DISTRICTS.iter().any(|d| {
        (w.x - d.center.x).abs() <= d.plaza_half && (w.y - d.center.y).abs() <= d.plaza_half
    });
    if in_plaza {
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
    }
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
        "spawning Hearthstone Village terrain v2 ({MAP_W}x{MAP_H}, {} districts)",
        DISTRICTS.len()
    );

    let texture = asset_server.load(SPRITE_TILES_VILLAGE_V2);
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
    let (mut grass, mut cobble, mut dirt, mut overlays) = (0usize, 0usize, 0usize, 0usize);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let pos = TilePos { x, y };
            let g = ground(x, y);
            match g {
                Ground::Grass => grass += 1,
                Ground::Cobble => cobble += 1,
                Ground::Dirt => dirt += 1,
            }
            let tile = commands
                .spawn(TileBundle {
                    position: pos,
                    tilemap_id: TilemapId(base_entity),
                    texture_index: TileTextureIndex(base_tile(x, y, g)),
                    ..default()
                })
                .id();
            base_storage.set(&pos, tile);

            if let Some(idx) = overlay_at(x, y, &ground) {
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
    tracing::debug!(grass, cobble, dirt, overlays, "terrain classified");

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
    fn plaza_centres_are_cobble_and_border_is_grass() {
        for d in DISTRICTS {
            assert_eq!(ground_at(d.center), Ground::Cobble, "{}", d.name);
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
}
