// SPDX-License-Identifier: MIT
//! Sends the editor's code to the compile-API and feeds the verdict back.
//!
//! - [`config`]: the [`CompileApiConfig`] resource (URL, token, mode,
//!   timeout) from env on native or `window.PLEDGE_CONFIG` on web.
//! - [`transport`]: the HTTP call (reqwest on both targets).
//! - [`outcome`]: pure mapping of the reply to a player-facing verdict,
//!   plus the fallback/clear policy (documented there).
//!
//! The request never blocks the frame loop: native runs it on a
//! short-lived worker thread, web on the browser event loop. Either way
//! the [`Verdict`] comes back through a `crossbeam-channel` that
//! [`drain_results`] polls each frame.

pub mod config;
pub mod outcome;
pub mod transport;

use crate::plugins::editor::EditorState;
use crate::plugins::progress::MissionProgress;
use crate::plugins::stub_grader::stub_verdict;
use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, unbounded};

pub use config::{ApiEndpoint, ApiTarget, CompileApiConfig, CompileMode};
pub use outcome::{ApiOutcome, Grade, Unverified, Verdict};

/// What a request ships back to the main loop.
struct CompileOutcome {
    encounter_id: String,
    verdict: Verdict,
}

#[derive(Resource)]
pub struct CompileChannel {
    sender: Sender<CompileOutcome>,
    receiver: Receiver<CompileOutcome>,
}

impl Default for CompileChannel {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

pub struct CompileClientPlugin;

impl Plugin for CompileClientPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("CompileClientPlugin::build");
        // A test or embedder may insert its own config first.
        if !app.world().contains_resource::<CompileApiConfig>() {
            let cfg = CompileApiConfig::load();
            app.insert_resource(cfg);
        }
        if let Some(cfg) = app.world().get_resource::<CompileApiConfig>() {
            tracing::info!("compile-api client: {}", cfg.summary());
        }
        app.init_resource::<CompileChannel>()
            .add_systems(Update, (dispatch_pending_compile, drain_results));
    }
}

/// Grade `source` against a configured server, end to end, off the main
/// thread: request, classify, then apply the fallback/clear policy.
/// (Async so the web path can await `fetch`; on native the request
/// itself blocks the worker thread it runs on.)
async fn grade_remote(endpoint: ApiEndpoint, source: String, encounter_id: String) -> Verdict {
    #[cfg(not(target_arch = "wasm32"))]
    let result = transport::post_compile_blocking(&endpoint, &source, &encounter_id);
    #[cfg(target_arch = "wasm32")]
    let result = transport::post_compile(&endpoint, &source, &encounter_id).await;

    let api_outcome = outcome::classify(endpoint.mode, result);
    log_outcome(&encounter_id, &api_outcome);
    outcome::verdict_with_server(&api_outcome, &encounter_id, || {
        tracing::warn!("compile-api: falling back to offline stub grader for `{encounter_id}`");
        stub_verdict(&encounter_id, &source)
    })
}

fn log_outcome(encounter_id: &str, o: &ApiOutcome) {
    match o {
        ApiOutcome::Graded(g) => tracing::info!(
            encounter = encounter_id,
            grade = ?g.grade,
            verified = matches!(g.grade, outcome::Grade::Passed),
            note = ?g.note,
            "compile-api: graded"
        ),
        ApiOutcome::RateLimited { retry_after_secs } => tracing::warn!(
            encounter = encounter_id,
            retry_after_secs = ?retry_after_secs,
            "compile-api: rate limited (429)"
        ),
        ApiOutcome::Busy { retry_after_secs } => tracing::warn!(
            encounter = encounter_id,
            retry_after_secs = ?retry_after_secs,
            "compile-api: build queue full (503)"
        ),
        ApiOutcome::Unauthorized => tracing::error!(
            encounter = encounter_id,
            "compile-api: unauthorized (401); check the configured API token"
        ),
        ApiOutcome::TooLarge => {
            tracing::warn!(
                encounter = encounter_id,
                "compile-api: source too large (413)"
            )
        }
        ApiOutcome::Offline { reason } => {
            tracing::warn!(
                encounter = encounter_id,
                "compile-api: unavailable: {reason}"
            )
        }
    }
}

/// If the editor's `compile_pending` flag is set, start a grading job and
/// clear the flag. Policy per target lives in [`outcome`].
fn dispatch_pending_compile(
    mut state: ResMut<EditorState>,
    channel: Res<CompileChannel>,
    config: Res<CompileApiConfig>,
) {
    if !state.compile_pending {
        return;
    }
    state.compile_pending = false;

    let source = state.source.clone();
    let encounter_id = state.encounter_id.clone();
    let sender = channel.sender.clone();

    match &config.target {
        ApiTarget::Offline => {
            tracing::info!(
                "grading `{encounter_id}` offline ({} bytes, demo mode: stub is canonical)",
                source.len()
            );
            let verdict =
                outcome::verdict_offline_mode(&encounter_id, stub_verdict(&encounter_id, &source));
            send(&sender, encounter_id, verdict);
        }
        ApiTarget::Misconfigured(e) => {
            tracing::warn!(
                "grading `{encounter_id}` with offline stub: client config invalid ({e})"
            );
            let api_outcome = ApiOutcome::Offline {
                reason: format!("the game's server settings are invalid: {e}"),
            };
            let verdict = outcome::verdict_with_server(&api_outcome, &encounter_id, || {
                stub_verdict(&encounter_id, &source)
            });
            send(&sender, encounter_id, verdict);
        }
        ApiTarget::Remote(endpoint) => {
            state.last_compile_result = Some("compiling…".to_string());
            tracing::info!(
                "dispatching compile job for `{encounter_id}` ({} bytes)",
                source.len()
            );
            spawn_remote(endpoint.clone(), source, encounter_id, sender);
        }
    }
}

fn send(sender: &Sender<CompileOutcome>, encounter_id: String, verdict: Verdict) {
    if sender
        .send(CompileOutcome {
            encounter_id,
            verdict,
        })
        .is_err()
    {
        tracing::error!("compile result channel closed; verdict dropped");
    }
}

/// Native: a named worker thread per request (the request needs its own
/// Tokio runtime; see [`transport`]).
#[cfg(not(target_arch = "wasm32"))]
fn spawn_remote(
    endpoint: ApiEndpoint,
    source: String,
    encounter_id: String,
    sender: Sender<CompileOutcome>,
) {
    let fallback_sender = sender.clone();
    let fallback_id = encounter_id.clone();
    let fallback_source = source.clone();
    let spawned = std::thread::Builder::new()
        .name("pnc-compile".to_string())
        .spawn(move || {
            let verdict =
                bevy::tasks::block_on(grade_remote(endpoint, source, encounter_id.clone()));
            send(&sender, encounter_id, verdict);
        });
    if let Err(e) = spawned {
        tracing::error!("could not start compile worker thread: {e}");
        let api_outcome = ApiOutcome::Offline {
            reason: format!("could not start a network worker: {e}"),
        };
        let verdict = outcome::verdict_with_server(&api_outcome, &fallback_id, || {
            stub_verdict(&fallback_id, &fallback_source)
        });
        send(&fallback_sender, fallback_id, verdict);
    }
}

/// Web: the fetch future runs on the browser event loop
/// (`IoTaskPool` -> `wasm_bindgen_futures::spawn_local`).
#[cfg(target_arch = "wasm32")]
fn spawn_remote(
    endpoint: ApiEndpoint,
    source: String,
    encounter_id: String,
    sender: Sender<CompileOutcome>,
) {
    bevy::tasks::IoTaskPool::get()
        .spawn(async move {
            let verdict = grade_remote(endpoint, source, encounter_id.clone()).await;
            send(&sender, encounter_id, verdict);
        })
        .detach();
}

fn drain_results(
    channel: Res<CompileChannel>,
    mut editor: ResMut<EditorState>,
    mut progress: ResMut<MissionProgress>,
) {
    while let Ok(CompileOutcome {
        encounter_id,
        verdict,
    }) = channel.receiver.try_recv()
    {
        tracing::info!(
            "compile result: encounter={} ok={} persist={} grade={:?} {}",
            encounter_id,
            verdict.ok,
            verdict.persist_clear,
            verdict.grade,
            verdict.formatted
        );
        if verdict.ok && verdict.persist_clear {
            if !matches!(verdict.grade, Some(outcome::Grade::Passed)) {
                // Policy: an unverified pass still clears the mission; the
                // editor labels it NOT VERIFIED / pattern check only.
                tracing::info!(
                    "unverified pass for `{encounter_id}` saved as a clear ({:?})",
                    verdict.grade
                );
            }
            progress.mark_cleared(&encounter_id);
        } else if verdict.ok {
            tracing::info!(
                "non-persisting pass for `{encounter_id}` (offline fallback; server configured but unavailable)"
            );
        }
        editor.set_result(verdict.formatted, verdict.detail);
    }
}
