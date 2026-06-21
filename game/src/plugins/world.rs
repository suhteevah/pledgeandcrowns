// SPDX-License-Identifier: MIT
//! Hearthstone Village world rendering.
//!
//! The village is a *designed* layout, not procedural noise: a central plaza
//! (Hearthstone Square) with an orthogonal dirt-road street grid out to a ring
//! of eight themed district plazas, the whole thing framed by a treeline. NPCs
//! are placed by [`district_slot`] into those plazas in roster order, spaced
//! generously so the cast reads as crowds gathered in quarters rather than a
//! clump.
//!
//! Tiles come from the REF-04 16-tile village atlas (4x4). That atlas has no
//! building walls/roofs yet, so a district reads as a cobble plaza + connecting
//! road + a signature prop (well, cart, hay, signpost, bush) rather than a
//! literal building — a tile-art expansion through claude.ai/design is the
//! follow-up for real architecture.

use crate::assets::SPRITE_TILES_VILLAGE;
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

// --- Tile indices into the REF-04 atlas (4x4, row-major) ----------
// Row 0: grass, tallGrass, dirtStraight, dirtCorner
// Row 1: cobble, fenceHoriz, fenceCorner, wellTop
// Row 2: wellBot, flowerYellow, flowerRed, bush
// Row 3: sapling, hayBale, cart, signpost
const T_GRASS: u32 = 0;
const T_TALL_GRASS: u32 = 1;
const T_DIRT_STRAIGHT: u32 = 2;
const T_COBBLE: u32 = 4;
const T_WELL_BOT: u32 = 8;
const T_FLOWER_YELLOW: u32 = 9;
const T_FLOWER_RED: u32 = 10;
const T_BUSH: u32 = 11;
const T_SAPLING: u32 = 12;
const T_HAY_BALE: u32 = 13;
const T_CART: u32 = 14;
const T_SIGNPOST: u32 = 15;

/// A village quarter: a square cobble plaza centered at `center`, home to
/// `count` roster NPCs (assigned sequentially by [`district_slot`]), marked
/// with a signature `prop` tile so it reads distinct at a glance.
pub struct District {
    pub name: &'static str,
    pub center: Vec2,
    pub plaza_half: f32,
    pub count: usize,
    pub prop: u32,
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
        prop: T_WELL_BOT,
    },
    District {
        name: "Market Row",
        center: Vec2::new(-960.0, 0.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_CART,
    },
    District {
        name: "The Commons",
        center: Vec2::new(960.0, 0.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_HAY_BALE,
    },
    District {
        name: "Guildhall Quarter",
        center: Vec2::new(-720.0, 700.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_SIGNPOST,
    },
    District {
        name: "Tradesmen's End",
        center: Vec2::new(720.0, 700.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_HAY_BALE,
    },
    District {
        name: "Tavern Row",
        center: Vec2::new(0.0, 800.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_CART,
    },
    District {
        name: "The Wharf",
        center: Vec2::new(-720.0, -700.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_BUSH,
    },
    District {
        name: "Stonebridge",
        center: Vec2::new(720.0, -700.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_SIGNPOST,
    },
    District {
        name: "The Old Gate",
        center: Vec2::new(0.0, -800.0),
        plaza_half: 260.0,
        count: 6,
        prop: T_CART,
    },
];

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

/// Distance from point `p` to segment `a`-`b`.
fn point_seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 <= f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Designed village painter: treeline border, then district plazas (cobble +
/// signature prop), then an orthogonal dirt street grid, then open ground
/// (grass with sparse deterministic flowers).
fn pick_tile(x: u32, y: u32) -> u32 {
    let w = tile_to_world(x, y);

    // Treeline: a 2-tile wooded border frames the village.
    if !(2..MAP_W - 2).contains(&x) || !(2..MAP_H - 2).contains(&y) {
        let h = (x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) % 3;
        return if h == 0 { T_BUSH } else { T_SAPLING };
    }

    // District plazas: cobble, with the signature prop at the plaza's NW corner.
    for d in DISTRICTS {
        if (w.x - d.center.x).abs() <= d.plaza_half && (w.y - d.center.y).abs() <= d.plaza_half {
            let prop_tile = world_to_tile(Vec2::new(
                d.center.x - d.plaza_half + TILE_PX,
                d.center.y + d.plaza_half - TILE_PX,
            ));
            if (x, y) == prop_tile {
                return d.prop;
            }
            return T_COBBLE;
        }
    }

    // Roads: an orthogonal street grid. The dirt tile is a *straight* piece, so
    // diagonal spokes rendered as jagged staircases — instead we run a main cross
    // through the hub plus two cross-streets that pick up the corner quarters.
    // Every segment is axis-aligned, so each reads as clean 2-tile-wide dirt.
    const ROADS: &[(Vec2, Vec2)] = &[
        (Vec2::new(-960.0, 0.0), Vec2::new(960.0, 0.0)), // main street, E-W
        (Vec2::new(0.0, -800.0), Vec2::new(0.0, 800.0)), // main street, N-S
        (Vec2::new(-720.0, 700.0), Vec2::new(720.0, 700.0)), // upper cross-street
        (Vec2::new(-720.0, -700.0), Vec2::new(720.0, -700.0)), // lower cross-street
    ];
    for (a, b) in ROADS {
        if point_seg_dist(w, *a, *b) < TILE_PX * 1.6 {
            return T_DIRT_STRAIGHT;
        }
    }

    // Open ground: grass with deterministic decoration — a bit livelier now,
    // more flower clusters and tufts to give the green field some life.
    let h = (x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) % 100;
    match h {
        0..=3 => T_FLOWER_YELLOW,
        4..=7 => T_FLOWER_RED,
        8..=17 => T_TALL_GRASS,
        18..=20 => T_BUSH,
        _ => T_GRASS,
    }
}

/// Per-tile multiply-tint that breaks up flat repeating expanses. Deterministic
/// per tile; uses a different hash basis than `pick_tile` so the shading doesn't
/// line up with the decoration pattern.
///
/// Stone/dirt get a grayscale lightness wobble (weathering). Grass gets a *green
/// tone* wobble across the forest/meadow ramp — warm sunlit, cool shade, fresh
/// meadow — so the field reads as patchy natural growth, not one flat wash.
fn tile_tint(x: u32, y: u32, idx: u32) -> Color {
    let h = (x.wrapping_mul(2654435761) ^ y.wrapping_mul(2246822519)) % 1000;
    match idx {
        T_COBBLE => {
            let v = 0.78 + (h % 23) as f32 * 0.01; // 0.78..=1.00, weathered stone
            Color::srgb(v, v, v)
        }
        T_DIRT_STRAIGHT => {
            let v = 0.84 + (h % 17) as f32 * 0.01; // 0.84..=1.00, packed dirt
            Color::srgb(v, v, v)
        }
        T_GRASS | T_TALL_GRASS => match h % 5 {
            0 => Color::srgb(0.84, 0.92, 0.78), // deeper, cooler (toward Pine)
            1 => Color::srgb(1.08, 1.04, 0.82), // warm, sunlit (toward Hayfield)
            2 => Color::srgb(0.94, 1.07, 0.90), // fresh, bright (Spring meadow)
            3 => Color::srgb(0.90, 0.98, 0.85), // shaded
            _ => Color::WHITE,                  // base tone
        },
        _ => Color::WHITE,
    }
}

fn spawn_village(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!(
        "spawning designed Hearthstone Village tilemap ({MAP_W}x{MAP_H}, {} districts)",
        DISTRICTS.len()
    );

    let texture = asset_server.load(SPRITE_TILES_VILLAGE);
    let map_size = TilemapSize { x: MAP_W, y: MAP_H };
    let tile_size = TilemapTileSize {
        x: TILE_PX,
        y: TILE_PX,
    };
    let grid_size: TilemapGridSize = tile_size.into();
    let map_type = TilemapType::default();

    let tilemap_entity = commands.spawn_empty().id();
    let mut storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let pos = TilePos { x, y };
            let idx = pick_tile(x, y);
            let tile = commands
                .spawn(TileBundle {
                    position: pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(idx),
                    color: TileColor(tile_tint(x, y, idx)),
                    ..default()
                })
                .id();
            storage.set(&pos, tile);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage,
        texture: TilemapTexture::Single(texture),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..default()
    });
}
