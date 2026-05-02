// SPDX-License-Identifier: MIT
//! Hearthstone Village world rendering.
//!
//! Day-7: replaced the REF-09 painted backdrop with a real
//! `bevy_ecs_tilemap` grid of 16x16 Heraldic-Code village tiles
//! (REF-04 sample sheet, sliced into 16 tile types in a 4x4 atlas).
//!
//! The map is hand-laid out for the Day-7 vertical slice — mostly
//! grass, a horizontal dirt path through the middle, a stone well,
//! and a handful of flowers/bushes for life. Real procedural village
//! generation lands when Act 1 content arrives.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

const TILE_PX: f32 = 16.0;
const MAP_W: u32 = 30;
const MAP_H: u32 = 20;

/// World bounds in world-space (centered on origin). Used by `player`
/// to clamp movement so the avatar can't walk off the tilemap.
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
const T_WELL_TOP: u32 = 7;
const T_WELL_BOT: u32 = 8;
const T_FLOWER_YELLOW: u32 = 9;
const T_FLOWER_RED: u32 = 10;
const T_BUSH: u32 = 11;
const T_SAPLING: u32 = 12;
const T_SIGNPOST: u32 = 15;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("WorldPlugin::build");
        app.add_plugins(TilemapPlugin)
            .add_systems(Startup, spawn_village);
    }
}

/// Build a tiny Hearthstone Village. Procedural noise + Act 1 layout
/// land later; this is hand-tuned for the Day-7 vertical slice.
fn pick_tile(x: u32, y: u32) -> u32 {
    // Horizontal dirt road across the middle row.
    let mid = MAP_H / 2;
    if y == mid {
        return T_DIRT_STRAIGHT;
    }
    // Stone well two tiles tall, west of center.
    if x == 8 && y == mid + 3 {
        return T_WELL_BOT;
    }
    if x == 8 && y == mid + 4 {
        return T_WELL_TOP;
    }
    // Cobblestone plaza around the well.
    if (7..=9).contains(&x) && (mid + 2..=mid + 5).contains(&y) {
        return T_COBBLE;
    }
    // Signpost at the east edge of the road.
    if x == MAP_W - 2 && y == mid + 1 {
        return T_SIGNPOST;
    }
    // Sprinkle decorations using a cheap deterministic hash.
    let h = (x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) % 100;
    match h {
        0..=4 => T_FLOWER_YELLOW,
        5..=8 => T_FLOWER_RED,
        9..=12 => T_BUSH,
        13..=15 => T_SAPLING,
        16..=22 => T_TALL_GRASS,
        _ => T_GRASS,
    }
}

fn spawn_village(mut commands: Commands, asset_server: Res<AssetServer>) {
    tracing::info!("spawning Hearthstone Village tilemap ({MAP_W}x{MAP_H})");

    let texture = asset_server.load("sprites/tiles_village.png");
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
            let tile = commands
                .spawn(TileBundle {
                    position: pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(pick_tile(x, y)),
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
