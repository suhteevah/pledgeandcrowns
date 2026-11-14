// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one pinned Rust toolchain.
//!
//! The game's boss dialogue quotes real rustc diagnostics, which are only
//! word-for-word true for one rustc release, and the server builds player
//! code as edition 2024. So the server refuses to build with any other
//! rustc. [`PINNED_RUSTC_VERSION`] is the ONLY place the version is
//! recorded: the deploy README installs it with
//! `rustup toolchain install "$(pledge_compile_api toolchain)" ...`, and
//! the startup self-test ([`crate::wasm_builder::startup_self_test`])
//! runs `rustc --version` on the toolchain the builds will use, logs it,
//! and refuses a mismatch in `bwrap` mode (warns in `none` dev mode).

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::build_sandbox::{HOST_ENV_ALLOWLIST, SandboxMode, SandboxSettings};
use crate::proc_tree;

/// The rustc release player code is built with.
pub const PINNED_RUSTC_VERSION: &str = "1.99.0";

/// Bound on `rustc --version`.
const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum ToolchainError {
    #[error("could not run `{program} --version`: {detail}")]
    Probe { program: String, detail: String },
    #[error("could not parse rustc version from {0:?}")]
    Unparsable(String),
    #[error(
        "rustc {found} does not match the pinned {PINNED_RUSTC_VERSION}: the boss dialogue quotes \
         rustc {PINNED_RUSTC_VERSION} diagnostics; install it with `rustup toolchain install \
         {PINNED_RUSTC_VERSION} --profile minimal --target wasm32-wasip1`"
    )]
    Mismatch { found: String },
}

/// `"rustc 1.99.0 (b940084d7 2026-09-28)"` -> `"1.99.0"`.
pub fn parse_rustc_version(output: &str) -> Option<&str> {
    let mut words = output.split_whitespace();
    if words.next()? != "rustc" {
        return None;
    }
    let v = words.next()?;
    v.chars().next().filter(char::is_ascii_digit).map(|_| v)
}

/// Compare a reported version with the pin.
pub fn check_version(found: &str) -> Result<(), ToolchainError> {
    if found == PINNED_RUSTC_VERSION {
        Ok(())
    } else {
        Err(ToolchainError::Mismatch {
            found: found.to_string(),
        })
    }
}

/// Run `rustc --version` for the toolchain the builds will use: the
/// configured toolchain dir in `bwrap` mode, `rustc` from `PATH` (with the
/// build's allow-listed environment) in `none` mode. No player input is
/// involved. Returns the parsed version string.
pub fn probe_rustc(settings: &SandboxSettings) -> Result<String, ToolchainError> {
    let mut cmd = match (settings.mode, settings.toolchain_dir.as_deref()) {
        (SandboxMode::Bwrap, Some(tc)) => {
            let mut c = Command::new(Path::new(tc).join("bin").join("rustc"));
            c.env_clear();
            c
        }
        _ => {
            let mut c = Command::new("rustc");
            c.env_clear();
            for k in HOST_ENV_ALLOWLIST {
                if let Some(v) = std::env::var_os(k) {
                    c.env(k, v);
                }
            }
            c
        }
    };
    cmd.arg("--version");
    let program = format!("{:?}", cmd.get_program());
    let out = proc_tree::run_captured(cmd, VERSION_PROBE_TIMEOUT, 4096, "rustc --version")
        .map_err(|e| ToolchainError::Probe {
            program: program.clone(),
            detail: e.to_string(),
        })?;
    if !out.status.success() || out.timed_out {
        return Err(ToolchainError::Probe {
            program,
            detail: format!("{:?}: {}", out.status, out.stderr.trim()),
        });
    }
    let line = out.stdout.trim().to_string();
    tracing::info!(rustc = %line, pinned = PINNED_RUSTC_VERSION, "toolchain: rustc version");
    parse_rustc_version(&line)
        .map(str::to_string)
        .ok_or(ToolchainError::Unparsable(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rustc_version_lines() {
        assert_eq!(
            parse_rustc_version("rustc 1.99.0 (b940084d7 2026-09-28)"),
            Some("1.99.0")
        );
        assert_eq!(parse_rustc_version("rustc 1.99.0"), Some("1.99.0"));
        assert_eq!(parse_rustc_version("cargo 1.99.0"), None);
        assert_eq!(parse_rustc_version("rustc"), None);
        assert_eq!(parse_rustc_version("rustc nightly"), None);
    }

    #[test]
    fn only_the_pinned_version_is_accepted() {
        assert!(check_version(PINNED_RUSTC_VERSION).is_ok());
        let err = check_version("1.98.0").unwrap_err();
        assert!(err.to_string().contains("1.98.0"));
        assert!(err.to_string().contains(PINNED_RUSTC_VERSION));
    }
}
