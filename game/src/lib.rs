// SPDX-License-Identifier: MIT
//! Pledge & Crown - library root.
//!
//! `main.rs` initialises tracing and calls `run()` here. All Bevy
//! plugin assembly lives below this line so that integration tests
//! can construct a headless `App` without going through `main`.

use bevy::asset::{AssetMode, AssetPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;

pub mod assets;
pub mod plugins;

/// Build and run the game. Blocks until the window closes.
pub fn run() -> anyhow::Result<()> {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pledge & Crown".into(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    // Per global rule: verbose logging. `debug` for our crate,
                    // quieter for Bevy internals (very chatty otherwise).
                    filter: "pledge_and_crown=debug,bevy=info,wgpu=warn,naga=warn".into(),
                    level: bevy::log::Level::DEBUG,
                    ..default()
                })
                .set(AssetPlugin {
                    // Bevy's `web` feature toggles AssetMode::Processed
                    // by default, which expects a `.meta` sidecar next
                    // to every asset. We don't generate those, so the
                    // sprites never bind on wasm. Force Unprocessed.
                    mode: AssetMode::Unprocessed,
                    ..default()
                })
                // Pixel-art sampler — nearest-neighbor everywhere.
                // Without this, Bevy's default linear sampler softens
                // the 1:1-pixel art and (more importantly on wasm) can
                // tickle WebGL2 sampler-creation paths that fail
                // silently for some texture formats.
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(plugins::CorePlugin)
        .run();

    tracing::info!("bevy app exited cleanly");
    Ok(())
}
