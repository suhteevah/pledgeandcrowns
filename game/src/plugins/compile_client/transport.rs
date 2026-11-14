// SPDX-License-Identifier: MIT
//! The one HTTP call: `POST {base}/compile[-real]` with optional bearer
//! auth and a timeout. Same `reqwest` code on both targets:
//!
//! - **Native**: reqwest's hyper backend needs a Tokio reactor, which a
//!   Bevy task pool does not provide (calling it there panics with "no
//!   reactor running"). [`post_compile_blocking`] runs the request on a
//!   current-thread Tokio runtime; the plugin calls it on a short-lived
//!   worker thread so the frame loop never waits.
//! - **Web**: reqwest compiles to the browser's `fetch` (its timeout uses
//!   an `AbortController`). The future is `!Send` and is driven by
//!   `wasm_bindgen_futures::spawn_local` via Bevy's `IoTaskPool`.

use serde::Serialize;

use super::config::ApiEndpoint;
use super::outcome::{HttpReply, TransportError};

#[derive(Serialize)]
struct CompileRequest<'a> {
    source: &'a str,
    encounter_id: &'a str,
}

/// Classify a reqwest error. Takes ownership so the URL can be dropped
/// from the player-facing text (it is already in our logs).
fn describe(e: reqwest::Error, endpoint: &ApiEndpoint) -> TransportError {
    if e.is_timeout() {
        TransportError::Timeout(endpoint.timeout.as_secs())
    } else if is_connect(&e) {
        TransportError::Connect
    } else {
        TransportError::Other(e.without_url().to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_connect(e: &reqwest::Error) -> bool {
    e.is_connect()
}

/// In a browser, `fetch` reports refused connections, DNS failures and
/// CORS blocks identically (a `TypeError`), surfaced by reqwest as a
/// request error. Treat them all as "couldn't connect".
#[cfg(target_arch = "wasm32")]
fn is_connect(e: &reqwest::Error) -> bool {
    e.is_request()
}

/// Send one compile request and collect status, `Retry-After` and body.
/// Never logs the token.
pub async fn post_compile(
    endpoint: &ApiEndpoint,
    source: &str,
    encounter_id: &str,
) -> Result<HttpReply, TransportError> {
    let url = endpoint.compile_url();
    tracing::info!(
        url = %url,
        mode = ?endpoint.mode,
        authed = endpoint.token.is_some(),
        timeout_s = endpoint.timeout.as_secs(),
        encounter = encounter_id,
        source_bytes = source.len(),
        "compile-api: sending request"
    );
    let mut req = reqwest::Client::new()
        .post(&url)
        .json(&CompileRequest {
            source,
            encounter_id,
        })
        .timeout(endpoint.timeout);
    if let Some(token) = &endpoint.token {
        req = req.bearer_auth(token.expose());
    }
    let resp = req.send().await.map_err(|e| {
        tracing::warn!(url = %url, "compile-api: transport error: {e}");
        describe(e, endpoint)
    })?;
    let status = resp.status().as_u16();
    let retry_after = resp
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = resp.text().await.map_err(|e| {
        tracing::warn!(url = %url, status, "compile-api: reading body failed: {e}");
        describe(e, endpoint)
    })?;
    tracing::debug!(
        url = %url,
        status,
        retry_after = ?retry_after,
        body_bytes = body.len(),
        "compile-api: reply received"
    );
    Ok(HttpReply {
        status,
        retry_after,
        body,
    })
}

/// Native: run [`post_compile`] to completion on a fresh current-thread
/// Tokio runtime. Blocks the calling thread; call it off the main loop.
#[cfg(not(target_arch = "wasm32"))]
pub fn post_compile_blocking(
    endpoint: &ApiEndpoint,
    source: &str,
    encounter_id: &str,
) -> Result<HttpReply, TransportError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| {
            tracing::error!("compile-api: could not start network runtime: {e}");
            TransportError::Other(format!("could not start network runtime: {e}"))
        })?;
    rt.block_on(post_compile(endpoint, source, encounter_id))
}
