// SPDX-License-Identifier: MIT
//! Talks to the local `compile-api` server.
//!
//! Uses Bevy's `IoTaskPool` so the editor UI thread never blocks on
//! network. Submitting a job pushes the result through a
//! `crossbeam-channel`; a polling system drains it into `EditorState`.
//! Real wasmtime sandbox arrives server-side per
//! `design/05-tech-architecture.md` §2.

use crate::plugins::editor::EditorState;
use crate::plugins::progress::MissionProgress;
use crate::plugins::stub_grader::{StubVerdict, stub_verdict};
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{Receiver, Sender, unbounded};
use serde::{Deserialize, Serialize};

const COMPILE_URL: &str = "http://127.0.0.1:7878/compile";

#[derive(Serialize)]
struct CompileRequest {
    source: String,
    encounter_id: String,
}

#[derive(Deserialize, Debug)]
struct CompileResponse {
    ok: bool,
    stdout: String,
    stderr: String,
}

/// What the worker task ships back to the main loop after a request.
struct CompileOutcome {
    encounter_id: String,
    formatted: String,
    ok: bool,
    /// True iff `formatted` came from the live `compile-api` server.
    /// Stub-fallback verdicts (server unreachable) leave this `false`
    /// so `drain_results` will NOT call `MissionProgress::mark_cleared`
    /// — the player must re-run their solution once the server is back
    /// to get a real, persisted clear.
    from_server: bool,
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
        app.init_resource::<CompileChannel>()
            .add_systems(Update, (dispatch_pending_compile, drain_results));
    }
}

/// If the editor's `compile_pending` flag is set, fire an async task
/// against the compile-API and clear the flag.
fn dispatch_pending_compile(mut state: ResMut<EditorState>, channel: Res<CompileChannel>) {
    if !state.compile_pending {
        return;
    }
    state.compile_pending = false;

    let source = state.source.clone();
    let sender = channel.sender.clone();
    let encounter_id = state.encounter_id.clone();

    state.last_compile_result = Some("compiling…".to_string());
    tracing::info!("dispatching compile job ({} bytes)", source.len());

    let encounter_for_task = encounter_id.clone();
    IoTaskPool::get()
        .spawn(async move {
            let (formatted, ok, from_server) = match send_compile(
                source.clone(),
                encounter_for_task.clone(),
            )
            .await
            {
                Ok(resp) => (format_response(&resp), resp.ok, true),
                Err(e) => {
                    // Live API unreachable. Try the offline stub so a
                    // demo machine without the server isn't dead-ended.
                    tracing::warn!(
                        "compile-api unreachable ({e}); attempting offline stub for encounter `{}`",
                        encounter_for_task
                    );
                    match stub_verdict(&encounter_for_task, &source) {
                        Some(v) => (format_stub(&v), v.ok, false),
                        None => (format!("[client error] {e}"), false, false),
                    }
                }
            };
            let _ = sender.send(CompileOutcome {
                encounter_id: encounter_for_task,
                formatted,
                ok,
                from_server,
            });
        })
        .detach();
}

fn drain_results(
    channel: Res<CompileChannel>,
    mut editor: ResMut<EditorState>,
    mut progress: ResMut<MissionProgress>,
) {
    while let Ok(outcome) = channel.receiver.try_recv() {
        tracing::info!(
            "compile result: encounter={} ok={} {}",
            outcome.encounter_id,
            outcome.ok,
            outcome.formatted
        );
        // ONLY persist clears that came from the live API. Stub-fallback
        // passes are explicitly NOT cleared so the player re-runs against
        // the real grader once the server is back.
        if outcome.ok && outcome.from_server {
            progress.mark_cleared(&outcome.encounter_id);
        } else if outcome.ok && !outcome.from_server {
            tracing::info!(
                "stub pass for `{}` — NOT marking cleared (server was unreachable)",
                outcome.encounter_id
            );
        }
        editor.last_compile_result = Some(outcome.formatted);
    }
}

async fn send_compile(source: String, encounter_id: String) -> anyhow::Result<CompileResponse> {
    let body = CompileRequest {
        source,
        encounter_id,
    };
    let resp = reqwest::Client::new()
        .post(COMPILE_URL)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<CompileResponse>()
        .await?;
    Ok(resp)
}

fn format_response(r: &CompileResponse) -> String {
    let tag = if r.ok { "ok" } else { "fail" };
    let mut out = format!("[{tag}] {}", r.stdout);
    if !r.stderr.is_empty() {
        out.push_str("\n--stderr--\n");
        out.push_str(&r.stderr);
    }
    out
}

/// Format a stub verdict for display. The leading `[offline mode]`
/// banner is load-bearing: any caller persisting these strings can use
/// it to detect "this was not a real server response."
fn format_stub(v: &StubVerdict) -> String {
    let tag = if v.ok { "ok" } else { "fail" };
    let body = if v.ok { &v.stdout } else { &v.stderr };
    let mut out = format!("[offline mode — server unreachable]\n[{tag}] {body}");
    // Mirror the live-path stderr block when both are populated, even
    // though the stub keeps them mutually exclusive today.
    if v.ok && !v.stderr.is_empty() {
        out.push_str("\n--stderr--\n");
        out.push_str(&v.stderr);
    }
    out
}
