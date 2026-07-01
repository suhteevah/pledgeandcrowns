// SPDX-License-Identifier: MIT
//! Floating quest markers above NPCs, in the WoW idiom:
//! a golden `!` over the next NPC you should talk to, and a golden `?`
//! over a mission you've started but not yet turned in (cleared).
//!
//! The glyphs are built procedurally as tiny pixel-art `Image`s (a gold fill
//! with an auto-derived dark outline) so they match the game's pixel aesthetic
//! without shipping a font or new art asset. Marker selection keys off the
//! same strict-linear progression the rest of the game uses: exactly one
//! mission is "next" at a time (`MissionRegistry::next_locked`).

use crate::plugins::mission::{ActiveMission, MissionRegistry};
use crate::plugins::npc::Npc;
use crate::plugins::progress::MissionProgress;
use crate::plugins::state::GameState;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// World-units the marker floats above the NPC's head (plus half the NPC's
/// own height, computed per-NPC at attach time so the 64px Borrow Checker
/// gets the same visual clearance as a 32px villager).
const MARKER_GAP: f32 = 14.0;
/// Idle bob amplitude / speed (world units / radians-per-second).
const BOB_AMP: f32 = 2.0;
const BOB_SPEED: f32 = 3.0;

// Bright neon-leaning gold-yellow fill + near-black outline. Marker glyphs are
// a gameplay affordance (HUD), so they run hotter than the world's Heraldic
// Code palette on purpose — they have to pop against grass and cobble alike.
const GOLD: [u8; 4] = [0xFF, 0xDE, 0x2A, 0xFF];
const OUTLINE: [u8; 4] = [0x1B, 0x08, 0x10, 0xFF];

// Glyph fills: 'G' = gold pixel, anything else = transparent. The outline is
// derived automatically (any empty pixel touching a gold one). One world unit
// per source pixel, so these read at ~13wu tall — proportionate to a 32wu NPC.
const EXCLAM: &[&str] = &[
    ".GGG.", ".GGG.", ".GGG.", ".GGG.", ".GGG.", ".GGG.", ".GGG.", ".GGG.", ".....", ".GGG.",
    ".GGG.",
];
const QUESTION: &[&str] = &[
    ".GGGGG.", "GG...GG", "GG...GG", "....GG.", "...GG..", "..GG...", "..GG...", "..GG...",
    ".......", "..GG...", "..GG...",
];

#[derive(Resource)]
pub struct QuestMarkerAssets {
    exclam: Handle<Image>,
    question: Handle<Image>,
}

/// Marks an NPC that already has a marker child, so we attach exactly once.
#[derive(Component)]
struct HasMarker;

/// The floating marker entity itself (a child of its NPC).
#[derive(Component)]
struct QuestMarker {
    base_y: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MarkerKind {
    None,
    /// Golden `!` — this is the next mission and you haven't opened it yet.
    Available,
    /// Golden `?` — you've started the next mission; come back and turn it in.
    TurnIn,
}

/// Pure marker-selection rule. Strict-linear: only the single `next` uncleared
/// mission ever gets a marker; it flips to a turn-in `?` once it's the active
/// (opened) mission. Cleared and not-yet-reachable NPCs show nothing.
fn marker_kind(
    mission_id: &str,
    next: Option<&str>,
    cleared: bool,
    active: Option<&str>,
) -> MarkerKind {
    if cleared {
        return MarkerKind::None;
    }
    if next == Some(mission_id) {
        if active == Some(mission_id) {
            MarkerKind::TurnIn
        } else {
            MarkerKind::Available
        }
    } else {
        MarkerKind::None
    }
}

pub struct QuestMarkerPlugin;

impl Plugin for QuestMarkerPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("QuestMarkerPlugin::build");
        app.add_systems(Startup, build_marker_assets).add_systems(
            Update,
            (attach_markers, update_markers, bob_markers).run_if(in_state(GameState::InGame)),
        );
    }
}

fn build_marker_assets(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let exclam = images.add(glyph_image(EXCLAM));
    let question = images.add(glyph_image(QUESTION));
    tracing::info!("built quest-marker glyphs (! and ?)");
    commands.insert_resource(QuestMarkerAssets { exclam, question });
}

/// Rasterize a glyph-fill grid into an RGBA pixel-art `Image`. The grid is
/// padded by one transparent pixel on every side, and every transparent pixel
/// that touches a gold pixel (8-neighbourhood) becomes outline — a clean,
/// uniform 1px dark border with zero hand-authoring.
fn glyph_image(fill: &[&str]) -> Image {
    let inner_h = fill.len();
    let inner_w = fill[0].chars().count();
    let w = inner_w + 2;
    let h = inner_h + 2;

    let mut gold = vec![vec![false; w]; h];
    for (y, row) in fill.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch == 'G' {
                gold[y + 1][x + 1] = true;
            }
        }
    }

    let mut data = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            if gold[y][x] {
                data[i..i + 4].copy_from_slice(&GOLD);
            } else if touches_gold(&gold, x, y, w, h) {
                data[i..i + 4].copy_from_slice(&OUTLINE);
            }
            // else: leave fully transparent
        }
    }

    let mut img = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    // Pixel-art: nearest-neighbour, never soften the glyph when scaled.
    img.sampler = ImageSampler::nearest();
    img
}

fn touches_gold(gold: &[Vec<bool>], x: usize, y: usize, w: usize, h: usize) -> bool {
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0
                && nx < w as i32
                && ny >= 0
                && ny < h as i32
                && gold[ny as usize][nx as usize]
            {
                return true;
            }
        }
    }
    false
}

/// Give every NPC a (hidden) marker child exactly once. Runs each frame but
/// only touches NPCs without `HasMarker`, so it's a no-op after the first.
#[allow(clippy::type_complexity)] // standard Bevy filtered-query shape
fn attach_markers(
    mut commands: Commands,
    markers: Res<QuestMarkerAssets>,
    npcs: Query<(Entity, &Sprite), (With<Npc>, Without<HasMarker>)>,
) {
    for (entity, sprite) in &npcs {
        let half_h = sprite.custom_size.map(|s| s.y * 0.5).unwrap_or(16.0);
        let base_y = half_h + MARKER_GAP;
        commands
            .entity(entity)
            .insert(HasMarker)
            .with_children(|c| {
                c.spawn((
                    Sprite {
                        image: markers.exclam.clone(),
                        ..default()
                    },
                    // z well above NPCs (5) and player so the marker always floats on top.
                    Transform::from_xyz(0.0, base_y, 50.0),
                    Visibility::Hidden,
                    QuestMarker { base_y },
                ));
            });
    }
}

/// Each frame, choose the right glyph (or hide) for every NPC's marker based
/// on current progression.
fn update_markers(
    registry: Res<MissionRegistry>,
    progress: Res<MissionProgress>,
    active: Res<ActiveMission>,
    markers: Res<QuestMarkerAssets>,
    npcs: Query<(&Npc, &Children)>,
    mut marker_q: Query<(&mut Sprite, &mut Visibility), With<QuestMarker>>,
) {
    let next = registry.next_locked(&progress).map(|m| m.id);
    let active_id = active.current.as_ref().map(|m| m.id);

    for (npc, children) in &npcs {
        let kind = marker_kind(
            npc.mission_id,
            next,
            progress.is_cleared(npc.mission_id),
            active_id,
        );
        for child in children.iter() {
            let Ok((mut sprite, mut vis)) = marker_q.get_mut(child) else {
                continue;
            };
            match kind {
                MarkerKind::None => *vis = Visibility::Hidden,
                MarkerKind::Available => {
                    sprite.image = markers.exclam.clone();
                    *vis = Visibility::Visible;
                }
                MarkerKind::TurnIn => {
                    sprite.image = markers.question.clone();
                    *vis = Visibility::Visible;
                }
            }
        }
    }
}

/// Gentle vertical bob so the markers read as "alive".
fn bob_markers(time: Res<Time>, mut q: Query<(&mut Transform, &QuestMarker)>) {
    let t = time.elapsed_secs();
    for (mut tf, marker) in &mut q {
        tf.translation.y = marker.base_y + (t * BOB_SPEED).sin() * BOB_AMP;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_uncleared_unopened_shows_available() {
        assert_eq!(
            marker_kind("a", Some("a"), false, None),
            MarkerKind::Available
        );
    }

    #[test]
    fn next_uncleared_but_opened_shows_turn_in() {
        assert_eq!(
            marker_kind("a", Some("a"), false, Some("a")),
            MarkerKind::TurnIn
        );
    }

    #[test]
    fn cleared_npc_shows_no_marker() {
        assert_eq!(
            marker_kind("a", Some("b"), true, Some("a")),
            MarkerKind::None
        );
    }

    #[test]
    fn locked_future_npc_shows_no_marker() {
        assert_eq!(marker_kind("c", Some("a"), false, None), MarkerKind::None);
    }

    #[test]
    fn no_next_mission_shows_nothing() {
        assert_eq!(marker_kind("a", None, false, None), MarkerKind::None);
    }
}
