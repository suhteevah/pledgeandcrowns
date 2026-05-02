// SPDX-License-Identifier: MIT
//! Talks to the local `compile-api` server.
//!
//! Uses Bevy's `IoTaskPool` so the editor UI thread never blocks on
//! network. Submitting a job pushes the result through a
//! `crossbeam-channel`; a polling system drains it into `EditorState`.
//! Real wasmtime sandbox arrives server-side per
//! `design/05-tech-architecture.md` §2.

use crate::plugins::editor::EditorState;
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

#[derive(Resource)]
pub struct CompileChannel {
    sender: Sender<String>,
    receiver: Receiver<String>,
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
    let encounter_id = "day-6-stub".to_string();

    state.last_compile_result = Some("compiling…".to_string());
    tracing::info!("dispatching compile job ({} bytes)", source.len());

    IoTaskPool::get()
        .spawn(async move {
            let msg = match send_compile(source, encounter_id).await {
                Ok(resp) => format_response(&resp),
                Err(e) => format!("[client error] {e}"),
            };
            // Recipient may have dropped (app closing); discard send errors.
            let _ = sender.send(msg);
        })
        .detach();
}

fn drain_results(channel: Res<CompileChannel>, mut state: ResMut<EditorState>) {
    while let Ok(msg) = channel.receiver.try_recv() {
        tracing::info!("compile result: {msg}");
        state.last_compile_result = Some(msg);
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
