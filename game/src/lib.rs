// SPDX-License-Identifier: MIT
//! Pledge & Crown - library root.
//!
//! `main.rs` initialises tracing and calls `run()` here. All Bevy
//! plugin assembly lives below this line so that integration tests
//! can construct a headless `App` without going through `main`.

use bevy::prelude::*;

pub mod assets;
pub mod plugins;

/// Build and run the game. Blocks until the window closes.
pub fn run() -> anyhow::Result<()> {
    tracing::debug!("building bevy App");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pledge & Crown".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(plugins::CorePlugin)
        .run();

    tracing::info!("bevy app exited cleanly");
    Ok(())
}
