// SPDX-License-Identifier: MIT
//! Self-driving screenshot mode: scripted capture scenes of the live game
//! WITHOUT any OS-level focus stealing or input injection.
//!
//! Entirely inert unless the `PNC_SHOTS` env var is set. When active it walks
//! a list of [`Scene`]s (see [`scenes`]): each one is a list of setup
//! [`Action`]s, a settle time, one capture and an optional teardown. Captures
//! use Bevy's own `Screenshot` API (reads the GPU render target, so the window
//! can be unfocused / occluded) and land in `PNC_SHOTS_DIR`.
//!
//! Setup drives the game through its own resources and states (the same
//! `NextState`, `MenuUi`, `EditorState`, ... the UI writes) and, where a real
//! input path is the point of the scene (walking into a wall, F to talk), by
//! writing Bevy's in-app `ButtonInput<KeyCode>` resource from
//! [`inject_scripted_keys`]. Nothing here touches the OS: no `keybd_event`, no
//! `SetForegroundWindow`, nothing that can land in whatever the user is
//! actually typing into. The window opens unfocused (`lib.rs`).
//!
//! Saves: the progress plugin disables save writes when `PNC_SHOTS` is set;
//! the driver re-checks `SavePolicy` at boot and refuses to run if writes are
//! somehow enabled. Every progress change a scene makes is in-memory only.
//!
//! When the list is done the driver writes a `DONE` file to the output dir
//! (every scene with OK/FAIL, plus the collision probe results) and exits the
//! app by itself. `scripts/capture-scenes.ps1` waits for that file.

mod collision;
mod scenes;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use bevy::input::InputSystems;
use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::plugins::progress::SavePolicy;
use crate::plugins::state::GameState;

/// A setup/teardown step that mutates the world directly.
pub type WorldFn = Box<dyn Fn(&mut World) + Send + Sync>;

/// One step of a scene's setup script. Durations are real seconds, so a
/// paused game (frozen virtual time) does not stall the driver.
pub enum Action {
    /// Run a closure against the world.
    Do(WorldFn),
    /// Wait this long.
    Wait(f32),
    /// Request a `GameState` and wait until it is active. If a transition
    /// was needed, wait a further `settle` seconds for textures to bind.
    EnterState(GameState, f32),
    /// Tap a key once through Bevy's `ButtonInput` (in-app only).
    Tap(KeyCode),
    /// Hold a key through Bevy's `ButtonInput` for this long (in-app only).
    Hold(KeyCode, f32),
}

/// One capture.
pub struct Scene {
    /// Output file name (no directory), e.g. `scene-pause.png`.
    pub file: String,
    pub actions: Vec<Action>,
    /// Hold after the last action before capturing.
    pub settle: f32,
    /// Runs after the capture has been written.
    pub teardown: Option<WorldFn>,
}

/// Keys the driver wants pressed this frame. Applied by
/// [`inject_scripted_keys`] right after Bevy's keyboard system.
#[derive(Resource, Default)]
pub struct ScriptedKeys {
    taps: VecDeque<KeyCode>,
    last_tap: Option<KeyCode>,
    held: Option<KeyCode>,
    last_held: Option<KeyCode>,
}

/// Free-form lines the scenes append (collision results etc.); copied into
/// the `DONE` file.
#[derive(Resource, Default)]
pub struct ShotLog(pub Vec<String>);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Boot,
    Action(usize),
    Settle,
    Flush,
    Finished,
}

#[derive(Resource)]
struct ShotDriver {
    dir: PathBuf,
    scenes: Vec<Scene>,
    index: usize,
    step: Step,
    /// Real seconds spent in the current step / action.
    timer: f32,
    action_started: bool,
    /// EnterState: whether this action had to request a transition.
    transitioned: bool,
    /// Flush: when the screenshot file first appeared.
    seen_at: Option<f32>,
    results: Vec<(String, Result<(), String>)>,
}

/// How long to wait for a requested screenshot file to appear.
const FLUSH_TIMEOUT: f32 = 6.0;
/// Extra hold after the file appears, so the write has surely finished.
const FLUSH_GRACE: f32 = 0.25;
/// Initial hold on the title screen while the backdrop texture binds.
const BOOT_SECS: f32 = 1.5;

pub struct ShotPlugin;

impl Plugin for ShotPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var("PNC_SHOTS").is_err() {
            return; // shipped builds never touch this
        }
        let dir = PathBuf::from(std::env::var("PNC_SHOTS_DIR").unwrap_or_else(|_| ".".into()));
        let done = dir.join("DONE");
        if done.exists() {
            if let Err(e) = std::fs::remove_file(&done) {
                tracing::warn!("could not remove stale {}: {e}", done.display());
            }
        }
        let mut scenes = scenes::all_scenes();
        // Optional comma-separated filter on output file names, for a short
        // run that captures only some scenes (e.g. `PNC_SHOTS_ONLY=trial-`).
        if let Ok(only) = std::env::var("PNC_SHOTS_ONLY") {
            let wanted: Vec<&str> = only.split(',').filter(|s| !s.is_empty()).collect();
            scenes.retain(|s| wanted.iter().any(|w| s.file.contains(w)));
            tracing::info!(?wanted, kept = scenes.len(), "PNC_SHOTS_ONLY filter");
        }
        tracing::info!(
            "screenshot mode ON; {} scenes; writing to {}",
            scenes.len(),
            dir.display()
        );
        app.insert_resource(ShotDriver {
            dir,
            scenes,
            index: 0,
            step: Step::Boot,
            timer: 0.0,
            action_started: false,
            transitioned: false,
            seen_at: None,
            results: Vec::new(),
        })
        .init_resource::<ScriptedKeys>()
        .init_resource::<ShotLog>()
        .add_systems(
            PreUpdate,
            inject_scripted_keys
                .in_set(InputSystems)
                .after(keyboard_input_system)
                .before(crate::plugins::dialogue::dialogue_input),
        )
        .add_systems(Update, drive_shots);
    }
}

/// Apply [`ScriptedKeys`] to Bevy's in-app keyboard state. A tap is pressed
/// for exactly one frame and released on the next, so the following tap of
/// the same key registers as a fresh `just_pressed`. A held key is
/// re-pressed every frame (dialogue releases movement keys while open).
fn inject_scripted_keys(mut sk: ResMut<ScriptedKeys>, mut keys: ResMut<ButtonInput<KeyCode>>) {
    if let Some(k) = sk.last_tap.take() {
        keys.release(k);
    }
    if let Some(k) = sk.taps.pop_front() {
        tracing::debug!(?k, "scripted tap (in-app ButtonInput)");
        keys.press(k);
        sk.last_tap = Some(k);
    }
    match (sk.held, sk.last_held) {
        (Some(k), _) => {
            keys.press(k);
            sk.last_held = Some(k);
        }
        (None, Some(k)) => {
            tracing::debug!(?k, "scripted hold released");
            keys.release(k);
            sk.last_held = None;
        }
        (None, None) => {}
    }
}

fn drive_shots(world: &mut World) {
    let dt = world.resource::<Time<Real>>().delta_secs();
    world.resource_scope(|world, mut driver: Mut<ShotDriver>| driver.tick(world, dt));
}

impl ShotDriver {
    fn tick(&mut self, world: &mut World, dt: f32) {
        self.timer += dt;
        match self.step {
            Step::Boot => self.boot(world),
            Step::Action(i) => self.run_action(world, i),
            Step::Settle => {
                if self.timer >= self.scenes[self.index].settle {
                    let path = self.current_path();
                    if path.exists() {
                        if let Err(e) = std::fs::remove_file(&path) {
                            tracing::warn!("could not remove old {}: {e}", path.display());
                        }
                    }
                    world
                        .spawn(Screenshot::primary_window())
                        .observe(save_to_disk(path.clone()));
                    tracing::info!(
                        "scene {}: screenshot requested",
                        self.scenes[self.index].file
                    );
                    self.enter(Step::Flush);
                }
            }
            Step::Flush => self.flush(world),
            Step::Finished => {}
        }
    }

    fn enter(&mut self, step: Step) {
        self.step = step;
        self.timer = 0.0;
        self.action_started = false;
        self.seen_at = None;
    }

    fn current_path(&self) -> PathBuf {
        self.dir.join(&self.scenes[self.index].file)
    }

    fn boot(&mut self, world: &mut World) {
        if self.timer < BOOT_SECS {
            return;
        }
        let writes = world.get_resource::<SavePolicy>().map(|p| p.writes_enabled);
        if writes != Some(false) {
            tracing::error!(
                ?writes,
                "screenshot mode: save writes are NOT disabled; refusing to run scenes"
            );
            self.results.push((
                "BOOT".into(),
                Err(format!(
                    "save writes not disabled (SavePolicy = {writes:?})"
                )),
            ));
            self.finish(world);
            return;
        }
        tracing::info!("screenshot mode: verified save writes disabled; scenes are in-memory only");
        scenes::boot_state(world);
        self.start_scene(0);
    }

    fn start_scene(&mut self, index: usize) {
        self.index = index;
        if index >= self.scenes.len() {
            self.enter(Step::Finished);
            return;
        }
        tracing::info!(
            "scene {}/{}: {} - setup",
            index + 1,
            self.scenes.len(),
            self.scenes[index].file
        );
        self.enter(Step::Action(0));
    }

    fn run_action(&mut self, world: &mut World, i: usize) {
        let scene = &self.scenes[self.index];
        let Some(action) = scene.actions.get(i) else {
            self.enter(Step::Settle);
            return;
        };
        let first = !self.action_started;
        self.action_started = true;
        let done = match action {
            Action::Do(f) => {
                f(world);
                true
            }
            Action::Wait(secs) => self.timer >= *secs,
            Action::EnterState(target, settle) => {
                if first {
                    let current = world.get_resource::<State<GameState>>().map(|s| *s.get());
                    self.transitioned = current != Some(*target);
                    if self.transitioned {
                        tracing::info!(?current, ?target, "scene: requesting state change");
                        world.resource_mut::<NextState<GameState>>().set(*target);
                    }
                }
                let now = world.get_resource::<State<GameState>>().map(|s| *s.get());
                let needed = if self.transitioned { *settle } else { 0.0 };
                now == Some(*target) && self.timer >= needed
            }
            Action::Tap(k) => {
                world.resource_mut::<ScriptedKeys>().taps.push_back(*k);
                true
            }
            Action::Hold(k, secs) => {
                if first {
                    world.resource_mut::<ScriptedKeys>().held = Some(*k);
                }
                if self.timer >= *secs {
                    world.resource_mut::<ScriptedKeys>().held = None;
                    true
                } else {
                    false
                }
            }
        };
        if done {
            self.enter(Step::Action(i + 1));
        }
    }

    fn flush(&mut self, world: &mut World) {
        let path = self.current_path();
        let present = std::fs::metadata(&path).is_ok_and(|m| m.len() > 0);
        if present && self.seen_at.is_none() {
            self.seen_at = Some(self.timer);
        }
        let flushed = self.seen_at.is_some_and(|t| self.timer - t >= FLUSH_GRACE);
        if !flushed && self.timer < FLUSH_TIMEOUT {
            return;
        }
        let file = self.scenes[self.index].file.clone();
        let result = if flushed {
            tracing::info!("scene {file}: captured -> {}", path.display());
            Ok(())
        } else {
            tracing::error!("scene {file}: no screenshot after {FLUSH_TIMEOUT}s");
            Err(format!("no file after {FLUSH_TIMEOUT}s"))
        };
        self.results.push((file, result));
        if let Some(teardown) = self.scenes[self.index].teardown.as_ref() {
            teardown(world);
        }
        if self.index + 1 >= self.scenes.len() {
            self.finish(world);
        } else {
            self.start_scene(self.index + 1);
        }
    }

    fn finish(&mut self, world: &mut World) {
        self.enter(Step::Finished);
        let log = world
            .get_resource::<ShotLog>()
            .map(|l| l.0.clone())
            .unwrap_or_default();
        let body = done_report(&self.results, &log);
        let done = self.dir.join("DONE");
        match write_atomic(&done, &body) {
            Ok(()) => tracing::info!("screenshot mode finished; wrote {}", done.display()),
            Err(e) => tracing::error!("could not write {}: {e}", done.display()),
        }
        world.write_message(AppExit::Success);
    }
}

/// Contents of the `DONE` marker.
fn done_report(results: &[(String, Result<(), String>)], log: &[String]) -> String {
    let ok = results.iter().filter(|(_, r)| r.is_ok()).count();
    let failed = results.len() - ok;
    let mut s = format!("pnc-shots DONE\nsave_writes=disabled\ncaptured={ok} failed={failed}\n");
    for (file, r) in results {
        match r {
            Ok(()) => s.push_str(&format!("OK   {file}\n")),
            Err(e) => s.push_str(&format!("FAIL {file}: {e}\n")),
        }
    }
    for line in log {
        s.push_str(&format!("LOG  {line}\n"));
    }
    s
}

/// Write via a temp file + rename so the launcher never reads a half file.
fn write_atomic(path: &Path, body: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_files_are_unique_and_keep_the_pitch_vantages() {
        let scenes = scenes::all_scenes();
        let mut names: Vec<&str> = scenes.iter().map(|s| s.file.as_str()).collect();
        // pitch/capture-world.ps1 depends on these four names.
        for keep in [
            "shot-ferris-marker.png",
            "shot-south.png",
            "shot-street.png",
            "shot-meadow.png",
        ] {
            assert!(names.contains(&keep), "missing {keep}");
        }
        let n = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate scene file names");
        assert!(
            scenes
                .iter()
                .all(|s| s.file.starts_with("shot-") || s.file.starts_with("scene-"))
        );
    }

    #[test]
    fn done_report_lists_every_scene() {
        let results = vec![
            ("scene-a.png".to_string(), Ok(())),
            ("scene-b.png".to_string(), Err("no file".to_string())),
        ];
        let r = done_report(&results, &["collision west ok".to_string()]);
        assert!(r.contains("captured=1 failed=1"));
        assert!(r.contains("OK   scene-a.png"));
        assert!(r.contains("FAIL scene-b.png: no file"));
        assert!(r.contains("LOG  collision west ok"));
    }
}
