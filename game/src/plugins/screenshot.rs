// SPDX-License-Identifier: MIT
//! Self-driving screenshot mode for capturing marketing/pitch shots of the
//! live world WITHOUT any OS-level focus stealing or input injection.
//!
//! Entirely inert unless the `PNC_SHOTS` env var is set. When active it skips
//! the title screen, teleports player + camera through a list of framed
//! vantages, and uses Bevy's own `Screenshot` API (reads the GPU render target,
//! so the window can be unfocused / occluded) to write one PNG per vantage to
//! `PNC_SHOTS_DIR`. The launcher just waits for the files to appear, then closes
//! the process — no `keybd_event`, no `SetForegroundWindow`, nothing that can
//! land in whatever the user is actually typing into.

use crate::plugins::player::Player;
use crate::plugins::state::GameState;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

struct Vantage {
    label: &'static str,
    pos: Vec2,
}

// Hand-picked frames. Player spawns at (0,0) in Hearthstone Square (cobble
// plaza, +/-300). Ferris (the next-quest NPC, golden '!') sits at (-240,-240).
const VANTAGES: &[Vantage] = &[
    // The marker money-shot: Ferris's golden '!' framed lower-left, grass +
    // treeline catching the left/bottom edges.
    Vantage {
        label: "ferris-marker",
        pos: Vec2::new(-180.0, -200.0),
    },
    // South spoke: the dirt street leaving the square south through grass.
    Vantage {
        label: "south",
        pos: Vec2::new(0.0, -470.0),
    },
    // North spoke: the dirt street grid leaving the square into grass.
    Vantage {
        label: "street",
        pos: Vec2::new(0.0, 470.0),
    },
    // Open meadow + treeline border, away from cobble.
    Vantage {
        label: "meadow",
        pos: Vec2::new(-180.0, 560.0),
    },
];

#[derive(Resource)]
struct ShotState {
    phase: u32,
    timer: f32,
    index: usize,
    dir: String,
}

pub struct ShotPlugin;

impl Plugin for ShotPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var("PNC_SHOTS").is_err() {
            return; // shipped builds never touch this
        }
        let dir = std::env::var("PNC_SHOTS_DIR").unwrap_or_else(|_| ".".to_string());
        tracing::info!("screenshot mode ON; writing to {dir}");
        app.insert_resource(ShotState {
            phase: 0,
            timer: 0.0,
            index: 0,
            dir,
        })
        .add_systems(Update, drive_shots);
    }
}

// The phase state-machine reads clearer as "match arm = phase, inner if = its
// exit condition" than as match guards, so silence the collapse suggestions.
#[allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::collapsible_match
)]
fn drive_shots(
    time: Res<Time>,
    mut st: ResMut<ShotState>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut player_q: Query<&mut Transform, (With<Player>, Without<Camera2d>)>,
    mut cam_q: Query<&mut Transform, With<Camera2d>>,
) {
    st.timer += time.delta_secs();
    match st.phase {
        // Jump straight into the world.
        0 => {
            if *state.get() != GameState::InGame {
                next.set(GameState::InGame);
            }
            if st.timer > 0.3 {
                st.phase = 1;
                st.timer = 0.0;
            }
        }
        // Let the tilemap + sprite textures finish binding.
        1 => {
            if st.timer > 2.0 {
                st.phase = 2;
                st.timer = 0.0;
            }
        }
        // Position player + camera for the current vantage.
        2 => {
            if st.index >= VANTAGES.len() {
                st.phase = 9;
                st.timer = 0.0;
                return;
            }
            let pos = VANTAGES[st.index].pos;
            if let Ok(mut p) = player_q.single_mut() {
                p.translation.x = pos.x;
                p.translation.y = pos.y;
            }
            if let Ok(mut c) = cam_q.single_mut() {
                c.translation.x = pos.x;
                c.translation.y = pos.y;
            }
            st.phase = 3;
            st.timer = 0.0;
        }
        // Hold a few frames so the camera-follow lerp + markers settle, capture.
        3 => {
            if st.timer > 0.7 {
                let path = format!("{}/shot-{}.png", st.dir, VANTAGES[st.index].label);
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path.clone()));
                tracing::info!("requested screenshot: {path}");
                st.phase = 4;
                st.timer = 0.0;
            }
        }
        // Give the async save time to flush, then advance.
        4 => {
            if st.timer > 0.6 {
                st.index += 1;
                st.phase = 2;
                st.timer = 0.0;
            }
        }
        // Done — idle. The launcher waits for the PNGs then closes us.
        _ => {}
    }
}
