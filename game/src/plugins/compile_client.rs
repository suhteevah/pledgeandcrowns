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
#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{Receiver, Sender, unbounded};
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
const COMPILE_URL: &str = "http://127.0.0.1:7878/compile";

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize)]
struct CompileRequest {
    source: String,
    encounter_id: String,
}

#[cfg(not(target_arch = "wasm32"))]
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
    /// True iff this verdict should persist as a `MissionProgress`
    /// clear. On native: only live-API responses count (stub-fallback
    /// passes leave this false so the player re-runs once the server
    /// is back). On wasm: the stub IS the canonical grader (no local
    /// server is reachable from a browser), so stub passes count.
    persist_clear: bool,
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
///
/// Native: hits the live API at `127.0.0.1:7878`, falls back to the
/// in-process stub if unreachable (stub passes do NOT count as clears).
/// Wasm: skips the network entirely and uses the stub as the canonical
/// grader (CORS + no localhost from a browser tab make a live API call
/// pointless on web). Stub passes DO count as clears in that case.
#[cfg(not(target_arch = "wasm32"))]
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
            let (formatted, ok, persist_clear) = match send_compile(
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
                persist_clear,
            });
        })
        .detach();
}

#[cfg(target_arch = "wasm32")]
fn dispatch_pending_compile(mut state: ResMut<EditorState>, channel: Res<CompileChannel>) {
    if !state.compile_pending {
        return;
    }
    state.compile_pending = false;

    let source = state.source.clone();
    let encounter_id = state.encounter_id.clone();

    tracing::info!(
        "dispatching compile job ({} bytes, wasm stub path)",
        source.len()
    );

    let (formatted, ok) = match stub_verdict(&encounter_id, &source) {
        Some(v) => (format_stub_wasm(&v), v.ok),
        None => (
            format!("[wasm] no stub grader for encounter `{}`", encounter_id),
            false,
        ),
    };
    let _ = channel.sender.send(CompileOutcome {
        encounter_id,
        formatted,
        ok,
        // Stub IS the canonical grader on wasm — passes persist.
        persist_clear: ok,
    });
}

fn drain_results(
    channel: Res<CompileChannel>,
    mut editor: ResMut<EditorState>,
    mut progress: ResMut<MissionProgress>,
) {
    while let Ok(outcome) = channel.receiver.try_recv() {
        tracing::info!(
            "compile result: encounter={} ok={} persist={} {}",
            outcome.encounter_id,
            outcome.ok,
            outcome.persist_clear,
            outcome.formatted
        );
        if outcome.ok && outcome.persist_clear {
            progress.mark_cleared(&outcome.encounter_id);
        } else if outcome.ok && !outcome.persist_clear {
            tracing::info!(
                "non-persisting pass for `{}` (server was unreachable, stub fallback)",
                outcome.encounter_id
            );
        }
        editor.last_compile_result = Some(outcome.formatted);
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
fn format_response(r: &CompileResponse) -> String {
    let tag = if r.ok { "ok" } else { "fail" };
    let mut out = format!("[{tag}] {}", r.stdout);
    if !r.stderr.is_empty() {
        out.push_str("\n--stderr--\n");
        out.push_str(&r.stderr);
    }
    out
}

/// Format a stub verdict for display in native fallback mode. The
/// leading `[offline mode]` banner is load-bearing: any caller
/// persisting these strings can use it to detect "this was not a real
/// server response."
#[cfg(not(target_arch = "wasm32"))]
fn format_stub(v: &StubVerdict) -> String {
    let tag = if v.ok { "ok" } else { "fail" };
    let body = if v.ok { &v.stdout } else { &v.stderr };
    let mut out = format!("[offline mode — server unreachable]\n[{tag}] {body}");
    if v.ok && !v.stderr.is_empty() {
        out.push_str("\n--stderr--\n");
        out.push_str(&v.stderr);
    }
    out
}

/// Wasm-canonical stub formatter. No "offline" banner — the stub IS
/// the grader on web, so dressing it up as a fallback would mislead
/// the player.
#[cfg(target_arch = "wasm32")]
fn format_stub_wasm(v: &StubVerdict) -> String {
    let tag = if v.ok { "ok" } else { "fail" };
    let body = if v.ok { &v.stdout } else { &v.stderr };
    let mut out = format!("[{tag}] {body}");
    if v.ok && !v.stderr.is_empty() {
        out.push_str("\n--stderr--\n");
        out.push_str(&v.stderr);
    }
    out
}
