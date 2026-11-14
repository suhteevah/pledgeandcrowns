// SPDX-License-Identifier: AGPL-3.0-or-later
//! Accounting for the short-lived helper threads this crate spawns
//! (process-output readers, stdin writers, the wasm watchdog and the wasm
//! compile helper).
//!
//! Every helper is started through [`spawn`], which increments a
//! process-wide counter *before* the thread exists and decrements it when
//! the thread's closure returns or unwinds. [`live`] reads the counter.
//! Tests use it to prove that timed-out work leaves no thread behind
//! (security re-review P1-C); `/metrics` does not export it.

use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

static LIVE: AtomicUsize = AtomicUsize::new(0);

/// Decrements the counter when dropped (normal return or panic).
struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Number of helper threads that are currently alive.
pub fn live() -> usize {
    LIVE.load(Ordering::SeqCst)
}

/// Spawn a named helper thread that is counted by [`live`] for its whole
/// lifetime.
pub fn spawn<F, T>(name: &str, f: F) -> io::Result<JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    LIVE.fetch_add(1, Ordering::SeqCst);
    let guard = Guard;
    let result = thread::Builder::new()
        .name(format!("pnc-{name}"))
        .spawn(move || {
            let _guard = guard;
            f()
        });
    if let Err(e) = &result {
        // The closure (and with it the guard) was dropped by the failed
        // spawn, which already decremented the counter.
        tracing::error!(name, "helper_threads: spawn failed: {e}");
    }
    result
}
