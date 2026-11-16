// SPDX-License-Identifier: MIT
//! Pledge & Crown - main entry point.
//!
//! Bevy's `LogPlugin` (in `DefaultPlugins`) owns the tracing subscriber.
//! We configure it via `LogPlugin::filter` in `lib.rs` rather than calling
//! `tracing_subscriber::fmt().init()` here — two subscribers fight over
//! the global default and emit a `set_global_default` warning at boot.
//!
//! A release build on Windows uses the GUI subsystem, so players get no
//! console window behind the game. Its stdout goes nowhere, which is why
//! native builds also write the log to a file next to the save
//! (`lib.rs`, `log_file_layer`).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

fn main() -> anyhow::Result<()> {
    pledge_and_crown::run()
}
