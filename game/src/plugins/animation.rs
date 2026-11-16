// SPDX-License-Identifier: MIT
//! Character animation: the player's four-facing walk, Ferris's scuttle and
//! the NPCs' idle bob.
//!
//! Frames come from two approved tile sheets rendered by `render-refs`
//! (REF-77 player walk, REF-78 Ferris walk) plus the existing idle sprites
//! (REF-01 player, REF-02 Ferris). Each sheet is sliced as a texture atlas by
//! the ref's own column count; tile index = declaration order in the ref.
//!
//! Cycles (from the ref headers, 150 ms per frame, style bible
//! "Animation conventions"):
//! - player down:  `down_0`, idle (lifted 1 px), `down_2`, idle (lifted 1 px)
//! - player up:    `up_0`, `up_1` (lifted), `up_2`, `up_1` (lifted)
//! - player left:  `left_0`, `left_1` (lifted), `left_2`, `left_1` (lifted)
//! - player right: the left frames, flipped
//! - standing: idle (down), `up_1` (up), `left_1` (left / right flipped)
//! - Ferris, any direction: `walk_0`, idle, `walk_2`, idle (no lift)
//!
//! **Lifts never move the entity.** A 1 px lift (the pass frame, the NPC idle
//! bob) shifts the sprite's [`Anchor`], not its `Transform`, and the amount is
//! recorded in [`SpriteLift`] so `prop::ysort` and `prop::fade_occluders` can
//! subtract it again ([`unlifted_anchor`]). Collision, y-sort, the talk radius,
//! quest-marker placement (markers are `Transform` children) and
//! `tests/npc_placement.rs` therefore see exactly what they saw before.
//!
//! The pure parts (facing from an input vector, cycle index from elapsed
//! time, which frame a pose shows, bob phase and lift) are small functions
//! with unit tests at the bottom of this file.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::assets::{SPRITE_FERRIS, SPRITE_FERRIS_WALK, SPRITE_PLAYER, SPRITE_PLAYER_WALK};
use crate::plugins::editor::EditorState;
use crate::plugins::player::{Player, input_dir};
use crate::plugins::state::GameState;

/// Native size of one walk frame (both sheets).
pub const FRAME_PX: u32 = 32;
/// Walk frame time, per the style bible (150 ms per frame).
pub const WALK_FRAME_SECS: f32 = 0.15;
/// Frames in one walk / scuttle cycle.
pub const CYCLE_FRAMES: usize = 4;
/// How far the pass frame lifts the standing pose (REF-77 header).
pub const PASS_LIFT_PX: f32 = 1.0;

/// REF-77 sheet geometry: `REF77_COLS = 4`, eight tiles, so two rows.
pub const PLAYER_SHEET_COLS: u32 = 4;
pub const PLAYER_SHEET_ROWS: u32 = 2;
/// REF-77 tile indices, in the ref's declaration order. Checked against the
/// ref file by `tests/art_wiring.rs`.
pub const PLAYER_DOWN_0: usize = 0;
pub const PLAYER_DOWN_2: usize = 1;
pub const PLAYER_UP_0: usize = 2;
pub const PLAYER_UP_1: usize = 3;
pub const PLAYER_UP_2: usize = 4;
pub const PLAYER_LEFT_0: usize = 5;
pub const PLAYER_LEFT_1: usize = 6;
pub const PLAYER_LEFT_2: usize = 7;
/// Tile names of REF-77 in index order (the test compares them to the ref).
pub const PLAYER_SHEET_TILES: &[&str] = &[
    "down_0", "down_2", "up_0", "up_1", "up_2", "left_0", "left_1", "left_2",
];

/// REF-78 sheet geometry: `REF78_COLS = 2`, two tiles, one row.
pub const FERRIS_SHEET_COLS: u32 = 2;
pub const FERRIS_SHEET_ROWS: u32 = 1;
pub const FERRIS_WALK_0: usize = 0;
pub const FERRIS_WALK_2: usize = 1;
/// Tile names of REF-78 in index order.
pub const FERRIS_SHEET_TILES: &[&str] = &["walk_0", "walk_2"];

/// One full NPC bob (down then up), seconds. Slow on purpose: a breath, not
/// a hop.
pub const BOB_PERIOD_SECS: f32 = 2.4;
/// Height of the NPC idle bob.
pub const BOB_LIFT_PX: f32 = 1.0;
/// Ferris counts as scuttling until he has been still this long, so a
/// frame without movement in the middle of a walk does not flicker to idle.
const SCUTTLE_GRACE_SECS: f32 = 0.1;

/// The way a character faces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Facing {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

/// Facing for an input vector. No input keeps the current facing. A clear
/// winner axis decides; on an exact diagonal (two keys held) the current
/// facing is kept when it is one of the two directions, so walking
/// diagonally does not flip-flop, otherwise the horizontal one wins.
pub fn facing_from_input(dir: Vec2, current: Facing) -> Facing {
    if dir == Vec2::ZERO {
        return current;
    }
    let horizontal = if dir.x < 0.0 {
        Facing::Left
    } else {
        Facing::Right
    };
    let vertical = if dir.y > 0.0 {
        Facing::Up
    } else {
        Facing::Down
    };
    let (ax, ay) = (dir.x.abs(), dir.y.abs());
    if ax > ay {
        horizontal
    } else if ay > ax {
        vertical
    } else if current == horizontal || current == vertical {
        current
    } else {
        horizontal
    }
}

/// Index into the 4-frame cycle after `elapsed` seconds of walking.
pub fn cycle_index(elapsed: f32) -> usize {
    let steps = (elapsed.max(0.0) / WALK_FRAME_SECS).floor() as usize;
    steps % CYCLE_FRAMES
}

/// Where a frame's pixels come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameSource {
    /// The character's idle sprite (REF-01 / REF-02).
    Idle,
    /// A tile of the character's walk sheet.
    Sheet(usize),
}

/// One displayed frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub source: FrameSource,
    pub flip_x: bool,
    pub lift_px: f32,
}

impl Frame {
    const fn new(source: FrameSource, flip_x: bool, lift_px: f32) -> Self {
        Self {
            source,
            flip_x,
            lift_px,
        }
    }
}

/// The player's frame for `facing`; `cycle` is `Some(index)` while walking
/// and `None` while standing.
pub fn player_frame(facing: Facing, cycle: Option<usize>) -> Frame {
    use FrameSource::{Idle, Sheet};
    let (stride_a, stand, stride_b, flip) = match facing {
        Facing::Down => (Sheet(PLAYER_DOWN_0), Idle, Sheet(PLAYER_DOWN_2), false),
        Facing::Up => (
            Sheet(PLAYER_UP_0),
            Sheet(PLAYER_UP_1),
            Sheet(PLAYER_UP_2),
            false,
        ),
        Facing::Left => (
            Sheet(PLAYER_LEFT_0),
            Sheet(PLAYER_LEFT_1),
            Sheet(PLAYER_LEFT_2),
            false,
        ),
        Facing::Right => (
            Sheet(PLAYER_LEFT_0),
            Sheet(PLAYER_LEFT_1),
            Sheet(PLAYER_LEFT_2),
            true,
        ),
    };
    match cycle.map(|i| i % CYCLE_FRAMES) {
        None => Frame::new(stand, flip, 0.0),
        Some(0) => Frame::new(stride_a, flip, 0.0),
        Some(2) => Frame::new(stride_b, flip, 0.0),
        Some(_) => Frame::new(stand, flip, PASS_LIFT_PX),
    }
}

/// Ferris's frame: the same scuttle whichever way he goes, no lift.
pub fn ferris_frame(cycle: Option<usize>) -> Frame {
    let source = match cycle.map(|i| i % CYCLE_FRAMES) {
        Some(0) => FrameSource::Sheet(FERRIS_WALK_0),
        Some(2) => FrameSource::Sheet(FERRIS_WALK_2),
        _ => FrameSource::Idle,
    };
    Frame::new(source, false, 0.0)
}

/// Bob phase (0..1) for the NPC at roster `index`: golden-ratio spacing, so
/// neighbours never share a phase and no two roster NPCs line up exactly.
pub fn bob_phase(index: usize) -> f32 {
    const GOLDEN: f32 = 0.618_034;
    (index as f32 * GOLDEN).fract()
}

/// Bob lift at `elapsed` seconds for `phase`: [`BOB_LIFT_PX`] for the second
/// half of each [`BOB_PERIOD_SECS`], 0 for the first. Whole pixels only.
pub fn bob_lift(elapsed: f32, phase: f32) -> f32 {
    let t = (elapsed.max(0.0) / BOB_PERIOD_SECS + phase).fract();
    if t < 0.5 { 0.0 } else { BOB_LIFT_PX }
}

/// Anchor that draws a centre-anchored sprite `lift_px` higher without
/// moving its transform.
pub fn lifted_anchor(lift_px: f32, height: f32) -> Anchor {
    if height <= 0.0 {
        return Anchor::CENTER;
    }
    Anchor(Vec2::new(0.0, -lift_px / height))
}

/// The anchor a sprite would have without its [`SpriteLift`]: what y-sort
/// and the occluder fade use, so a lift never changes draw order or fading.
pub fn unlifted_anchor(anchor: &Anchor, lift: Option<&SpriteLift>, height: f32) -> Vec2 {
    let mut a = anchor.as_vec();
    if let Some(l) = lift
        && height > 0.0
    {
        a.y += l.0 / height;
    }
    a
}

/// Pixels the sprite is currently drawn above its transform (via its anchor).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct SpriteLift(pub f32);

/// Player walk state.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct WalkAnim {
    pub facing: Facing,
    pub moving: bool,
    /// Seconds since the current walk started (0 while standing).
    pub elapsed: f32,
}

/// What one [`WalkAnim::step`] changed (for logging).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkChange {
    pub facing: Option<(Facing, Facing)>,
    pub started: bool,
    pub stopped: bool,
}

impl WalkAnim {
    /// Advance by one frame of input `dir` and `dt` seconds.
    pub fn step(&mut self, dir: Vec2, dt: f32) -> WalkChange {
        let moving = dir != Vec2::ZERO;
        let facing = facing_from_input(dir, self.facing);
        let change = WalkChange {
            facing: (facing != self.facing).then_some((self.facing, facing)),
            started: moving && !self.moving,
            stopped: !moving && self.moving,
        };
        self.facing = facing;
        self.elapsed = if !moving || change.started {
            0.0
        } else {
            self.elapsed + dt.max(0.0)
        };
        self.moving = moving;
        change
    }

    /// The frame this state shows.
    pub fn frame(&self) -> Frame {
        player_frame(self.facing, self.moving.then(|| cycle_index(self.elapsed)))
    }
}

/// Ferris's scuttle state, driven by his own displacement (he scuttles
/// whenever his transform moves, whatever moves it).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Scuttle {
    pub last: Option<Vec2>,
    pub moving: bool,
    pub still_for: f32,
    pub elapsed: f32,
}

impl Scuttle {
    /// Advance with the current position. Returns `Some(true)` on scuttle
    /// start, `Some(false)` on stop.
    pub fn step(&mut self, pos: Vec2, dt: f32) -> Option<bool> {
        let dt = dt.max(0.0);
        let moved = self.last.is_some_and(|l| l.distance_squared(pos) > 1e-6);
        self.last = Some(pos);
        self.still_for = if moved { 0.0 } else { self.still_for + dt };
        let moving = moved || (self.moving && self.still_for < SCUTTLE_GRACE_SECS);
        let change = (moving != self.moving).then_some(moving);
        self.elapsed = match change {
            Some(true) => 0.0,
            _ if moving => self.elapsed + dt,
            _ => 0.0,
        };
        self.moving = moving;
        change
    }

    pub fn frame(&self) -> Frame {
        ferris_frame(self.moving.then(|| cycle_index(self.elapsed)))
    }
}

/// Idle bob for a standing NPC.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct IdleBob {
    pub phase: f32,
}

impl IdleBob {
    pub fn for_roster_index(index: usize) -> Self {
        Self {
            phase: bob_phase(index),
        }
    }
}

/// Image + atlas layout handles for one character.
#[derive(Clone, Debug)]
pub struct SheetSet {
    pub idle: Handle<Image>,
    pub sheet: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Handles for the animated characters, loaded once at startup.
#[derive(Resource, Clone, Debug)]
pub struct CharacterSheets {
    pub player: SheetSet,
    pub ferris: SheetSet,
}

/// Put `frame` on a centre-anchored sprite: image or atlas tile, flip, and
/// lift (via the anchor; see the module docs).
pub fn apply_frame(
    sprite: &mut Sprite,
    anchor: &mut Anchor,
    lift: &mut SpriteLift,
    frame: Frame,
    set: &SheetSet,
) {
    match frame.source {
        FrameSource::Idle => {
            if sprite.image != set.idle {
                sprite.image = set.idle.clone();
            }
            sprite.texture_atlas = None;
        }
        FrameSource::Sheet(index) => {
            if sprite.image != set.sheet {
                sprite.image = set.sheet.clone();
            }
            match sprite.texture_atlas.as_mut() {
                Some(atlas) if atlas.layout == set.layout => atlas.index = index,
                _ => {
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: set.layout.clone(),
                        index,
                    });
                }
            }
        }
    }
    sprite.flip_x = frame.flip_x;
    set_lift(sprite, anchor, lift, frame.lift_px);
}

/// Draw a centre-anchored sprite `px` above its transform.
pub fn set_lift(sprite: &Sprite, anchor: &mut Anchor, lift: &mut SpriteLift, px: f32) {
    if lift.0 == px {
        return;
    }
    let height = sprite.custom_size.map_or(FRAME_PX as f32, |s| s.y);
    *anchor = lifted_anchor(px, height);
    lift.0 = px;
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("AnimationPlugin::build");
        app.add_systems(Startup, load_character_sheets).add_systems(
            Update,
            (animate_player, animate_scuttle, bob_npcs)
                .run_if(in_state(GameState::InGame))
                .run_if(resource_exists::<CharacterSheets>),
        );
    }
}

fn load_character_sheets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut grid = |cols: u32, rows: u32| {
        layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(FRAME_PX),
            cols,
            rows,
            None,
            None,
        ))
    };
    let sheets = CharacterSheets {
        player: SheetSet {
            idle: asset_server.load(SPRITE_PLAYER),
            sheet: asset_server.load(SPRITE_PLAYER_WALK),
            layout: grid(PLAYER_SHEET_COLS, PLAYER_SHEET_ROWS),
        },
        ferris: SheetSet {
            idle: asset_server.load(SPRITE_FERRIS),
            sheet: asset_server.load(SPRITE_FERRIS_WALK),
            layout: grid(FERRIS_SHEET_COLS, FERRIS_SHEET_ROWS),
        },
    };
    tracing::info!(
        player_sheet = SPRITE_PLAYER_WALK,
        ferris_sheet = SPRITE_FERRIS_WALK,
        "character walk sheets loaded (REF-77 {PLAYER_SHEET_COLS}x{PLAYER_SHEET_ROWS}, \
         REF-78 {FERRIS_SHEET_COLS}x{FERRIS_SHEET_ROWS})"
    );
    commands.insert_resource(sheets);
}

/// Facing + walk cycle from the same input `move_player` reads (and, like
/// it, nothing while the editor is open).
fn animate_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    editor: Res<EditorState>,
    sheets: Res<CharacterSheets>,
    mut q: Query<(&mut WalkAnim, &mut Sprite, &mut Anchor, &mut SpriteLift), With<Player>>,
) {
    let dir = if editor.open {
        Vec2::ZERO
    } else {
        input_dir(&keys)
    };
    for (mut anim, mut sprite, mut anchor, mut lift) in &mut q {
        let change = anim.step(dir, time.delta_secs());
        if let Some((from, to)) = change.facing {
            tracing::debug!(?from, ?to, "player facing changed");
        }
        if change.started {
            tracing::debug!(facing = ?anim.facing, "player walk start");
        }
        if change.stopped {
            tracing::debug!(facing = ?anim.facing, "player walk stop");
        }
        let frame = anim.frame();
        apply_frame(&mut sprite, &mut anchor, &mut lift, frame, &sheets.player);
    }
}

/// Ferris scuttles whenever his transform moves.
fn animate_scuttle(
    time: Res<Time>,
    sheets: Res<CharacterSheets>,
    mut q: Query<(
        &Transform,
        &mut Scuttle,
        &mut Sprite,
        &mut Anchor,
        &mut SpriteLift,
    )>,
) {
    for (tf, mut scuttle, mut sprite, mut anchor, mut lift) in &mut q {
        match scuttle.step(tf.translation.truncate(), time.delta_secs()) {
            Some(true) => tracing::debug!("Ferris scuttle start"),
            Some(false) => tracing::debug!("Ferris scuttle stop"),
            None => {}
        }
        if scuttle.moving || sprite.texture_atlas.is_some() {
            let frame = scuttle.frame();
            apply_frame(&mut sprite, &mut anchor, &mut lift, frame, &sheets.ferris);
        }
    }
}

/// 1 px idle bob for standing NPCs (not while Ferris scuttles).
fn bob_npcs(
    time: Res<Time>,
    mut q: Query<(
        &IdleBob,
        Option<&Scuttle>,
        &Sprite,
        &mut Anchor,
        &mut SpriteLift,
    )>,
) {
    let t = time.elapsed_secs();
    for (bob, scuttle, sprite, mut anchor, mut lift) in &mut q {
        if scuttle.is_some_and(|s| s.moving) {
            continue;
        }
        set_lift(sprite, &mut anchor, &mut lift, bob_lift(t, bob.phase));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_follows_the_dominant_axis() {
        let f = Facing::Down;
        assert_eq!(facing_from_input(Vec2::new(-1.0, 0.0), f), Facing::Left);
        assert_eq!(facing_from_input(Vec2::new(1.0, 0.0), f), Facing::Right);
        assert_eq!(facing_from_input(Vec2::new(0.0, 1.0), f), Facing::Up);
        assert_eq!(
            facing_from_input(Vec2::new(0.0, -1.0), Facing::Up),
            Facing::Down
        );
        assert_eq!(facing_from_input(Vec2::new(0.3, -0.9), f), Facing::Down);
        assert_eq!(facing_from_input(Vec2::new(-0.9, 0.3), f), Facing::Left);
    }

    #[test]
    fn no_input_keeps_facing() {
        for f in [Facing::Down, Facing::Up, Facing::Left, Facing::Right] {
            assert_eq!(facing_from_input(Vec2::ZERO, f), f);
        }
    }

    #[test]
    fn exact_diagonal_keeps_a_matching_facing_else_horizontal() {
        let up_right = Vec2::new(1.0, 1.0);
        assert_eq!(facing_from_input(up_right, Facing::Up), Facing::Up);
        assert_eq!(facing_from_input(up_right, Facing::Right), Facing::Right);
        assert_eq!(facing_from_input(up_right, Facing::Down), Facing::Right);
        assert_eq!(
            facing_from_input(Vec2::new(-1.0, -1.0), Facing::Up),
            Facing::Left
        );
    }

    #[test]
    fn cycle_advances_every_150ms_and_wraps() {
        assert_eq!(cycle_index(0.0), 0);
        assert_eq!(cycle_index(0.149), 0);
        assert_eq!(cycle_index(0.151), 1);
        assert_eq!(cycle_index(0.31), 2);
        assert_eq!(cycle_index(0.46), 3);
        assert_eq!(cycle_index(0.61), 0);
        assert_eq!(cycle_index(-1.0), 0);
    }

    #[test]
    fn player_cycles_match_the_ref77_header() {
        use FrameSource::{Idle, Sheet};
        let cycle = |f| (0..4).map(|i| player_frame(f, Some(i))).collect::<Vec<_>>();
        let down = cycle(Facing::Down);
        assert_eq!(down[0], Frame::new(Sheet(PLAYER_DOWN_0), false, 0.0));
        assert_eq!(down[1], Frame::new(Idle, false, PASS_LIFT_PX));
        assert_eq!(down[2], Frame::new(Sheet(PLAYER_DOWN_2), false, 0.0));
        assert_eq!(down[3], down[1]);
        let up = cycle(Facing::Up);
        assert_eq!(up[0].source, Sheet(PLAYER_UP_0));
        assert_eq!(up[1], Frame::new(Sheet(PLAYER_UP_1), false, PASS_LIFT_PX));
        assert_eq!(up[2].source, Sheet(PLAYER_UP_2));
        let left = cycle(Facing::Left);
        assert_eq!(left[0].source, Sheet(PLAYER_LEFT_0));
        assert_eq!(left[1].source, Sheet(PLAYER_LEFT_1));
        assert_eq!(left[2].source, Sheet(PLAYER_LEFT_2));
        assert!(left.iter().all(|f| !f.flip_x));
        let right = cycle(Facing::Right);
        for (l, r) in left.iter().zip(&right) {
            assert_eq!(l.source, r.source, "right reuses the left frames");
            assert_eq!(l.lift_px, r.lift_px);
            assert!(r.flip_x, "right is the left frames flipped");
        }
    }

    #[test]
    fn standing_frames_are_unlifted_per_facing() {
        use FrameSource::{Idle, Sheet};
        assert_eq!(
            player_frame(Facing::Down, None),
            Frame::new(Idle, false, 0.0)
        );
        assert_eq!(
            player_frame(Facing::Up, None),
            Frame::new(Sheet(PLAYER_UP_1), false, 0.0)
        );
        assert_eq!(
            player_frame(Facing::Left, None),
            Frame::new(Sheet(PLAYER_LEFT_1), false, 0.0)
        );
        assert_eq!(
            player_frame(Facing::Right, None),
            Frame::new(Sheet(PLAYER_LEFT_1), true, 0.0)
        );
    }

    #[test]
    fn ferris_cycle_matches_the_ref78_header() {
        use FrameSource::{Idle, Sheet};
        let c: Vec<_> = (0..4).map(|i| ferris_frame(Some(i)).source).collect();
        assert_eq!(
            c,
            vec![Sheet(FERRIS_WALK_0), Idle, Sheet(FERRIS_WALK_2), Idle]
        );
        assert_eq!(ferris_frame(None).source, Idle);
        assert!((0..4).all(|i| ferris_frame(Some(i)).lift_px == 0.0));
    }

    #[test]
    fn sheet_tables_match_the_index_constants() {
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_DOWN_0], "down_0");
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_DOWN_2], "down_2");
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_UP_0], "up_0");
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_UP_1], "up_1");
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_UP_2], "up_2");
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_LEFT_0], "left_0");
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_LEFT_1], "left_1");
        assert_eq!(PLAYER_SHEET_TILES[PLAYER_LEFT_2], "left_2");
        assert_eq!(FERRIS_SHEET_TILES[FERRIS_WALK_0], "walk_0");
        assert_eq!(FERRIS_SHEET_TILES[FERRIS_WALK_2], "walk_2");
        assert!(PLAYER_SHEET_TILES.len() as u32 <= PLAYER_SHEET_COLS * PLAYER_SHEET_ROWS);
        assert!(FERRIS_SHEET_TILES.len() as u32 <= FERRIS_SHEET_COLS * FERRIS_SHEET_ROWS);
    }

    #[test]
    fn walk_step_reports_start_stop_and_turns_once() {
        let mut a = WalkAnim::default();
        let c = a.step(Vec2::new(-1.0, 0.0), 0.016);
        assert!(c.started && !c.stopped);
        assert_eq!(c.facing, Some((Facing::Down, Facing::Left)));
        assert_eq!(a.elapsed, 0.0, "a walk starts on stride A");
        assert_eq!(a.frame().source, FrameSource::Sheet(PLAYER_LEFT_0));
        let c = a.step(Vec2::new(-1.0, 0.0), 0.16);
        assert_eq!(c, WalkChange::default(), "steady walking logs nothing");
        assert_eq!(a.frame().lift_px, PASS_LIFT_PX);
        let c = a.step(Vec2::ZERO, 0.016);
        assert!(c.stopped && !c.started && c.facing.is_none());
        assert_eq!(a.facing, Facing::Left, "standing keeps the last facing");
        assert_eq!(a.frame(), player_frame(Facing::Left, None));
    }

    #[test]
    fn scuttle_follows_displacement_with_a_short_grace() {
        let mut s = Scuttle::default();
        assert_eq!(s.step(Vec2::ZERO, 0.016), None, "first sample never moves");
        assert_eq!(s.step(Vec2::new(1.0, 0.0), 0.016), Some(true));
        assert_eq!(s.frame().source, FrameSource::Sheet(FERRIS_WALK_0));
        // One still frame inside the grace keeps him scuttling.
        assert_eq!(s.step(Vec2::new(1.0, 0.0), 0.05), None);
        assert!(s.moving);
        assert_eq!(s.step(Vec2::new(1.0, 0.0), 0.06), Some(false));
        assert_eq!(s.frame().source, FrameSource::Idle);
    }

    #[test]
    fn bob_is_one_pixel_slow_and_phase_offset() {
        // Whole pixels only, and only 0 or 1.
        for i in 0..200 {
            let l = bob_lift(i as f32 * 0.05, 0.0);
            assert!(l == 0.0 || l == BOB_LIFT_PX);
        }
        assert_eq!(bob_lift(0.0, 0.0), 0.0);
        assert_eq!(bob_lift(BOB_PERIOD_SECS * 0.75, 0.0), BOB_LIFT_PX);
        // Slow: a half period is far longer than an idle frame.
        const { assert!(BOB_PERIOD_SECS * 0.5 >= 1.0) };
        // Neighbours in the roster do not share a phase.
        let phases: Vec<f32> = (0..crate::plugins::npc::NPC_ROSTER.len())
            .map(bob_phase)
            .collect();
        for w in phases.windows(2) {
            assert!((w[0] - w[1]).abs() > 0.1, "{w:?}");
        }
        assert!(phases.iter().all(|p| (0.0..1.0).contains(p)));
    }

    #[test]
    fn lift_moves_the_anchor_and_unlift_restores_it() {
        let h = 32.0;
        let a = lifted_anchor(1.0, h);
        // Anchor 1/32 below centre at the transform draws the sprite 1 px up.
        assert!((a.as_vec().y + 1.0 / h).abs() < 1e-6);
        let restored = unlifted_anchor(&a, Some(&SpriteLift(1.0)), h);
        assert!(restored.abs_diff_eq(Anchor::CENTER.as_vec(), 1e-6));
        assert_eq!(unlifted_anchor(&Anchor::CENTER, None, h), Vec2::ZERO);
        assert_eq!(lifted_anchor(1.0, 0.0), Anchor::CENTER);
    }
}
