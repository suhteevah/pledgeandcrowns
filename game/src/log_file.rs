// SPDX-License-Identifier: MIT
//! Native log file: every log line also goes to
//! `pledge-and-crown.log` next to the save
//! (`%APPDATA%/pledge-and-crown/` on Windows), so a player has something to
//! attach to a bug report. A Windows release build has no console (see
//! `main.rs`), so without this its logs would go nowhere.
//!
//! One file per session: on start the previous session's log is kept as
//! `pledge-and-crown.log.1` and a fresh file is opened. Uses Bevy's own
//! `tracing_subscriber` (no new dependency), added as a
//! `LogPlugin::custom_layer`, so it sees the same filtered events as the
//! console output.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bevy::log::BoxedLayer;
use bevy::prelude::*;

/// File name of the current session's log.
pub const LOG_FILENAME: &str = "pledge-and-crown.log";
/// File name the previous session's log is kept under.
pub const PREVIOUS_LOG_FILENAME: &str = "pledge-and-crown.log.1";

/// Where the log file goes: beside the save file.
pub fn log_file_path() -> PathBuf {
    crate::plugins::progress::default_save_path().with_file_name(LOG_FILENAME)
}

/// Keep the previous log as `<name>.1` (replacing an older one), then
/// create a fresh, empty log at `path`. Creates the directory if needed.
pub fn rotate_and_create(path: &Path) -> io::Result<File> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if path.exists() {
        let previous = path.with_file_name(PREVIOUS_LOG_FILENAME);
        if previous.exists() {
            std::fs::remove_file(&previous)?;
        }
        std::fs::rename(path, &previous)?;
    }
    File::create(path)
}

/// What happened when the log file was opened, reported once logging is
/// up (the layer is built before the subscriber exists, so it cannot log).
#[derive(Resource, Debug)]
pub struct LogFileStatus(pub Result<PathBuf, String>);

/// `LogPlugin::custom_layer`: a plain-text fmt layer writing to the log
/// file. On failure the game runs with console logging only.
pub fn log_file_layer(app: &mut App) -> Option<BoxedLayer> {
    let path = log_file_path();
    match rotate_and_create(&path) {
        Ok(file) => {
            app.insert_resource(LogFileStatus(Ok(path)));
            Some(Box::new(
                bevy::log::tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_writer(Mutex::new(file)),
            ))
        }
        Err(e) => {
            app.insert_resource(LogFileStatus(Err(format!("{}: {e}", path.display()))));
            None
        }
    }
}

/// Startup system: say where the log file is (or why there is none).
pub fn report_log_file(status: Option<Res<LogFileStatus>>) {
    match status.as_deref() {
        Some(LogFileStatus(Ok(path))) => {
            tracing::info!(path = %path.display(), "writing the log to a file");
        }
        Some(LogFileStatus(Err(e))) => {
            tracing::warn!("could not open the log file ({e}); logging to the console only");
        }
        None => tracing::debug!("no log file configured"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pnc-log-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn creates_the_directory_and_an_empty_log() {
        let dir = temp_dir("create");
        let path = dir.join("nested").join(LOG_FILENAME);
        rotate_and_create(&path).expect("create log");
        assert!(path.exists());
        assert_eq!(std::fs::metadata(&path).expect("meta").len(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn keeps_exactly_one_previous_session() {
        let dir = temp_dir("rotate");
        let path = dir.join(LOG_FILENAME);
        let previous = dir.join(PREVIOUS_LOG_FILENAME);
        for session in ["first", "second", "third"] {
            let mut f = rotate_and_create(&path).expect("open log");
            io::Write::write_all(&mut f, session.as_bytes()).expect("write log");
        }
        assert_eq!(std::fs::read_to_string(&path).expect("current"), "third");
        assert_eq!(std::fs::read_to_string(&previous).expect("prev"), "second");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn log_sits_beside_the_save() {
        let save = crate::plugins::progress::default_save_path();
        assert_eq!(log_file_path().parent(), save.parent());
    }
}
