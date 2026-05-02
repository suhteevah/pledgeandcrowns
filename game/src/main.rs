// SPDX-License-Identifier: MIT
//! Pledge & Crown - main entry point.
//!
//! Bevy's `LogPlugin` (in `DefaultPlugins`) owns the tracing subscriber.
//! We configure it via `LogPlugin::filter` in `lib.rs` rather than calling
//! `tracing_subscriber::fmt().init()` here — two subscribers fight over
//! the global default and emit a `set_global_default` warning at boot.

fn main() -> anyhow::Result<()> {
    pledge_and_crown::run()
}
