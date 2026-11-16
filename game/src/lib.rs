// SPDX-License-Identifier: MIT
//! Pledge & Crown - library root.
//!
//! `main.rs` only calls `run()` here; tracing is set up by Bevy's
//! `LogPlugin`, configured below (console, plus a log file on native, see
//! [`log_file`]). All Bevy plugin assembly lives below this line so that
//! integration tests can construct a headless `App` without going through
//! `main`.
//!
//! Hard rule 1 (CLAUDE.md): no `unsafe` in shipped game code. `forbid`
//! makes it a compile error anywhere in this crate, and no inner `allow`
//! can relax it. Act 9's lessons put `unsafe` in the *player's* programs:
//! starters, solutions and grader patterns are string literals here, and
//! those programs only ever run in the server's wasm sandbox.
#![forbid(unsafe_code)]

use bevy::asset::{AssetMetaCheck, AssetMode, AssetPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;

pub mod assets;
#[cfg(not(target_arch = "wasm32"))]
pub mod log_file;
pub mod plugins;

/// `LogPlugin::custom_layer`: the log file on native, nothing on the web.
fn extra_log_layer(app: &mut App) -> Option<bevy::log::BoxedLayer> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        log_file::log_file_layer(app)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = app;
        None
    }
}

/// Build and run the game. Blocks until the window closes.
pub fn run() -> anyhow::Result<()> {
    // Screenshot mode (PNC_SHOTS) opens the window UNFOCUSED so an automated
    // capture run can never steal keyboard focus from whatever the user is
    // doing. Normal play focuses as usual.
    let shot_mode = std::env::var("PNC_SHOTS").is_ok();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pledge & Crown".into(),
                    resolution: (1280u32, 720u32).into(),
                    focused: !shot_mode,
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                // Per global rule: verbose logging. `debug` for our crate,
                // quieter for Bevy internals (very chatty otherwise).
                filter: "pledge_and_crown=debug,bevy=info,wgpu=warn,naga=warn".into(),
                level: bevy::log::Level::DEBUG,
                custom_layer: extra_log_layer,
                ..default()
            })
            .set(AssetPlugin {
                // Bevy's `web` feature toggles AssetMode::Processed
                // by default, which expects a `.meta` sidecar next
                // to every asset. We don't generate those, so the
                // sprites never bind on wasm. Force Unprocessed.
                mode: AssetMode::Unprocessed,
                // Belt-and-suspenders for wasm: AssetMetaCheck::Never
                // skips the `<asset>.meta` HEAD probes the asset
                // server otherwise issues for every load. On wasm
                // those 404s aren't fatal but they pollute the
                // network panel and slow first-frame asset binding.
                // (See Bevy issue #18002 / PR #10623.)
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            // Pixel-art sampler — nearest-neighbor everywhere.
            // Without this, Bevy's default linear sampler softens
            // the 1:1-pixel art and (more importantly on wasm) can
            // tickle WebGL2 sampler-creation paths that fail
            // silently for some texture formats.
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(plugins::CorePlugin);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Startup, log_file::report_log_file);
    app.run();

    tracing::info!("bevy app exited cleanly");
    Ok(())
}
