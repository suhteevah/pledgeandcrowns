// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed runtime configuration for the compile API.
//!
//! Parsed exactly once at startup from environment variables (see
//! [`Config::from_env`]) and validated before the listener binds. Every
//! knob has a safe default so `cargo run -p pledge_compile_api` on a dev
//! box needs no env at all.
//!
//! | Variable                    | Default                                       |
//! |-----------------------------|-----------------------------------------------|
//! | `PNC_BIND`                  | `127.0.0.1:7878`                              |
//! | `PNC_CORS_ORIGINS`          | `http://127.0.0.1:8080,http://localhost:8080` |
//! | `PNC_AUTH_TOKENS`           | *(unset: no auth, loopback bind only)*        |
//! | `PNC_TRUST_PROXY`           | `false`                                       |
//! | `PNC_RATE_LIMIT_PER_MIN`    | `600` (per client IP, all routes)             |
//! | `PNC_REAL_RATE_LIMIT_PER_MIN` | `20` (per client IP, `/compile-real`)       |
//! | `PNC_MAX_CONCURRENT_BUILDS` | `2`                                           |
//! | `PNC_MAX_QUEUED_BUILDS`     | `8`                                           |
//! | `PNC_RUN_MEMORY_MIB`        | `16` (design/05 §2), max `128`                |
//!
//! # Safety invariant
//!
//! With no bearer token configured the server refuses to bind to anything
//! but a loopback address ([`ConfigError::UnauthenticatedPublicBind`]),
//! so it can never be exposed to the network unauthenticated by accident.
//!
//! Secrets (`PNC_AUTH_TOKENS`) never appear in `Debug` output or in error
//! messages.

use std::fmt;
use std::net::SocketAddr;

use axum::http::HeaderValue;

use crate::wasm_builder::{DEFAULT_RUN_MAX_MEMORY_BYTES, MAX_RUN_MAX_MEMORY_BYTES};

/// Env var names, public so tests and docs reference one spelling.
pub mod vars {
    pub const BIND: &str = "PNC_BIND";
    pub const CORS_ORIGINS: &str = "PNC_CORS_ORIGINS";
    pub const AUTH_TOKENS: &str = "PNC_AUTH_TOKENS";
    pub const TRUST_PROXY: &str = "PNC_TRUST_PROXY";
    pub const RATE_LIMIT_PER_MIN: &str = "PNC_RATE_LIMIT_PER_MIN";
    pub const REAL_RATE_LIMIT_PER_MIN: &str = "PNC_REAL_RATE_LIMIT_PER_MIN";
    pub const MAX_CONCURRENT_BUILDS: &str = "PNC_MAX_CONCURRENT_BUILDS";
    pub const MAX_QUEUED_BUILDS: &str = "PNC_MAX_QUEUED_BUILDS";
    pub const RUN_MEMORY_MIB: &str = "PNC_RUN_MEMORY_MIB";
}

/// Default bind address. Loopback: safe with no auth configured.
pub const DEFAULT_BIND: &str = "127.0.0.1:7878";
/// Default CORS allow-list: the local `web-serve` dev server only.
pub const DEFAULT_CORS_ORIGINS: &[&str] = &["http://127.0.0.1:8080", "http://localhost:8080"];
/// design/05 §2: per-IP 600/min.
pub const DEFAULT_RATE_LIMIT_PER_MIN: u32 = 600;
/// Tighter per-IP budget for the expensive real build route.
pub const DEFAULT_REAL_RATE_LIMIT_PER_MIN: u32 = 20;
pub const DEFAULT_MAX_CONCURRENT_BUILDS: usize = 2;
pub const DEFAULT_MAX_QUEUED_BUILDS: usize = 8;
/// Bearer tokens shorter than this are rejected at startup. 32 chars of
/// hex is 128 bits; anything shorter is guessable under the rate limit.
pub const MIN_TOKEN_LEN: usize = 32;
/// Upper bounds so a typo cannot turn a limit into "effectively off".
const MAX_RATE_LIMIT_PER_MIN: u32 = 100_000;
const MAX_CONCURRENT_BUILDS_CAP: usize = 64;
const MAX_QUEUED_BUILDS_CAP: usize = 1_024;

/// Startup configuration errors. Never carries a secret value.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{var}={value:?} is invalid: {reason}")]
    Invalid {
        var: &'static str,
        value: String,
        reason: String,
    },
    #[error(
        "{var} entry #{index} is shorter than {min} characters; generate one with `openssl rand -hex 32`"
    )]
    TokenTooShort {
        var: &'static str,
        index: usize,
        min: usize,
    },
    #[error(
        "refusing to bind {addr}: it is not a loopback address and no bearer token is configured \
         (set {} or bind to 127.0.0.1 / [::1])",
        vars::AUTH_TOKENS
    )]
    UnauthenticatedPublicBind { addr: SocketAddr },
}

/// A bearer token. `Debug` is redacted; the raw bytes are only reachable
/// through [`Secret::expose`], which the auth middleware uses for the
/// constant-time compare.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn expose(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

/// Validated runtime configuration.
#[derive(Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub cors_origins: Vec<HeaderValue>,
    pub auth_tokens: Vec<Secret>,
    /// Honour the right-most `X-Forwarded-For` entry as the client IP.
    /// Only safe when every request arrives through a reverse proxy you
    /// control (bind to loopback behind it).
    pub trust_proxy: bool,
    pub rate_limit_per_min: u32,
    pub real_rate_limit_per_min: u32,
    pub max_concurrent_builds: usize,
    pub max_queued_builds: usize,
    pub run_max_memory_bytes: u64,
}

impl Default for Config {
    /// Dev defaults. Always valid: loopback bind, no auth.
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 7878)),
            cors_origins: DEFAULT_CORS_ORIGINS
                .iter()
                .map(|o| HeaderValue::from_static(o))
                .collect(),
            auth_tokens: Vec::new(),
            trust_proxy: false,
            rate_limit_per_min: DEFAULT_RATE_LIMIT_PER_MIN,
            real_rate_limit_per_min: DEFAULT_REAL_RATE_LIMIT_PER_MIN,
            max_concurrent_builds: DEFAULT_MAX_CONCURRENT_BUILDS,
            max_queued_builds: DEFAULT_MAX_QUEUED_BUILDS,
            run_max_memory_bytes: DEFAULT_RUN_MAX_MEMORY_BYTES,
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let origins: Vec<&str> = self
            .cors_origins
            .iter()
            .map(|o| o.to_str().unwrap_or("<non-ascii>"))
            .collect();
        f.debug_struct("Config")
            .field("bind_addr", &self.bind_addr)
            .field("cors_origins", &origins)
            .field(
                "auth_tokens",
                &format_args!("<redacted; {} configured>", self.auth_tokens.len()),
            )
            .field("trust_proxy", &self.trust_proxy)
            .field("rate_limit_per_min", &self.rate_limit_per_min)
            .field("real_rate_limit_per_min", &self.real_rate_limit_per_min)
            .field("max_concurrent_builds", &self.max_concurrent_builds)
            .field("max_queued_builds", &self.max_queued_builds)
            .field("run_max_memory_bytes", &self.run_max_memory_bytes)
            .finish()
    }
}

impl Config {
    /// Parse from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    /// Parse from an arbitrary lookup (tests pass a map). Empty values are
    /// treated as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |k: &str| {
            lookup(k)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let d = Self::default();

        let bind_addr = match get(vars::BIND) {
            None => d.bind_addr,
            Some(v) => v.parse().map_err(|e| ConfigError::Invalid {
                var: vars::BIND,
                value: v.clone(),
                reason: format!("{e} (expected ip:port)"),
            })?,
        };

        let cors_origins = match get(vars::CORS_ORIGINS) {
            None => d.cors_origins,
            Some(v) => parse_origins(&v)?,
        };

        let auth_tokens = match lookup(vars::AUTH_TOKENS) {
            None => Vec::new(),
            Some(v) => parse_tokens(&v)?,
        };

        let trust_proxy = match get(vars::TRUST_PROXY) {
            None => d.trust_proxy,
            Some(v) => parse_bool(vars::TRUST_PROXY, &v)?,
        };

        let rate_limit_per_min = parse_ranged(
            vars::RATE_LIMIT_PER_MIN,
            get(vars::RATE_LIMIT_PER_MIN),
            d.rate_limit_per_min,
            1,
            MAX_RATE_LIMIT_PER_MIN,
        )?;
        let real_rate_limit_per_min = parse_ranged(
            vars::REAL_RATE_LIMIT_PER_MIN,
            get(vars::REAL_RATE_LIMIT_PER_MIN),
            d.real_rate_limit_per_min,
            1,
            MAX_RATE_LIMIT_PER_MIN,
        )?;
        let max_concurrent_builds = parse_ranged(
            vars::MAX_CONCURRENT_BUILDS,
            get(vars::MAX_CONCURRENT_BUILDS),
            d.max_concurrent_builds,
            1,
            MAX_CONCURRENT_BUILDS_CAP,
        )?;
        let max_queued_builds = parse_ranged(
            vars::MAX_QUEUED_BUILDS,
            get(vars::MAX_QUEUED_BUILDS),
            d.max_queued_builds,
            0,
            MAX_QUEUED_BUILDS_CAP,
        )?;
        let run_memory_mib: u64 = parse_ranged(
            vars::RUN_MEMORY_MIB,
            get(vars::RUN_MEMORY_MIB),
            d.run_max_memory_bytes / (1024 * 1024),
            1,
            MAX_RUN_MAX_MEMORY_BYTES / (1024 * 1024),
        )?;

        let cfg = Self {
            bind_addr,
            cors_origins,
            auth_tokens,
            trust_proxy,
            rate_limit_per_min,
            real_rate_limit_per_min,
            max_concurrent_builds,
            max_queued_builds,
            run_max_memory_bytes: run_memory_mib * 1024 * 1024,
        };
        cfg.validate()?;
        Ok(cfg)
    }

    /// Cross-field invariants. Called by [`Config::from_lookup`]; call it
    /// yourself if you build a `Config` by hand for a public bind.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.auth_tokens.is_empty() && !self.bind_addr.ip().is_loopback() {
            return Err(ConfigError::UnauthenticatedPublicBind {
                addr: self.bind_addr,
            });
        }
        Ok(())
    }

    pub fn auth_enabled(&self) -> bool {
        !self.auth_tokens.is_empty()
    }
}

fn parse_ranged<T>(
    var: &'static str,
    raw: Option<String>,
    default: T,
    min: T,
    max: T,
) -> Result<T, ConfigError>
where
    T: std::str::FromStr + PartialOrd + fmt::Display + Copy,
    T::Err: fmt::Display,
{
    let Some(v) = raw else {
        return Ok(default);
    };
    let n: T = v.parse().map_err(|e: T::Err| ConfigError::Invalid {
        var,
        value: v.clone(),
        reason: e.to_string(),
    })?;
    if n < min || n > max {
        return Err(ConfigError::Invalid {
            var,
            value: v,
            reason: format!("must be between {min} and {max}"),
        });
    }
    Ok(n)
}

fn parse_bool(var: &'static str, v: &str) -> Result<bool, ConfigError> {
    match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(ConfigError::Invalid {
            var,
            value: v.to_string(),
            reason: "expected true/false".to_string(),
        }),
    }
}

/// Comma-separated exact origins (`scheme://host[:port]`, no path, no
/// wildcard). An origin that fails these checks is a startup error, not a
/// silent drop: a CORS allow-list typo should never widen access.
fn parse_origins(v: &str) -> Result<Vec<HeaderValue>, ConfigError> {
    let mut out = Vec::new();
    for raw in v.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let invalid = |reason: &str| ConfigError::Invalid {
            var: vars::CORS_ORIGINS,
            value: raw.to_string(),
            reason: reason.to_string(),
        };
        if raw.contains('*') {
            return Err(invalid("wildcards are not allowed; list exact origins"));
        }
        let Some((scheme, rest)) = raw.split_once("://") else {
            return Err(invalid("expected scheme://host[:port]"));
        };
        if scheme.is_empty() || rest.is_empty() || rest.contains('/') {
            return Err(invalid(
                "expected scheme://host[:port] with no path or trailing slash",
            ));
        }
        let hv = HeaderValue::from_str(raw).map_err(|e| invalid(&e.to_string()))?;
        out.push(hv);
    }
    Ok(out)
}

/// Comma-separated bearer tokens. The value is never echoed in errors.
fn parse_tokens(v: &str) -> Result<Vec<Secret>, ConfigError> {
    let mut out = Vec::new();
    for (index, raw) in v
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        if raw.len() < MIN_TOKEN_LEN {
            return Err(ConfigError::TokenTooShort {
                var: vars::AUTH_TOKENS,
                index,
                min: MIN_TOKEN_LEN,
            });
        }
        out.push(Secret::new(raw));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cfg(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Config::from_lookup(|k| map.get(k).cloned())
    }

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn defaults_are_valid_and_match_design() {
        let c = cfg(&[]).expect("defaults must validate");
        assert_eq!(c.bind_addr.to_string(), DEFAULT_BIND);
        assert_eq!(c.rate_limit_per_min, 600);
        assert_eq!(c.real_rate_limit_per_min, 20);
        assert_eq!(c.max_concurrent_builds, 2);
        assert_eq!(c.run_max_memory_bytes, 16 * 1024 * 1024);
        assert!(!c.auth_enabled());
        assert!(!c.trust_proxy);
    }

    #[test]
    fn memory_cap_cannot_exceed_previous_128_mib() {
        assert!(cfg(&[(vars::RUN_MEMORY_MIB, "128")]).is_ok());
        assert!(cfg(&[(vars::RUN_MEMORY_MIB, "129")]).is_err());
        assert!(cfg(&[(vars::RUN_MEMORY_MIB, "0")]).is_err());
    }

    #[test]
    fn zero_rate_limit_and_zero_concurrency_rejected() {
        assert!(cfg(&[(vars::RATE_LIMIT_PER_MIN, "0")]).is_err());
        assert!(cfg(&[(vars::MAX_CONCURRENT_BUILDS, "0")]).is_err());
        assert!(cfg(&[(vars::MAX_QUEUED_BUILDS, "0")]).is_ok());
    }

    #[test]
    fn cors_rejects_wildcards_and_paths() {
        assert!(cfg(&[(vars::CORS_ORIGINS, "*")]).is_err());
        assert!(cfg(&[(vars::CORS_ORIGINS, "https://a.example/")]).is_err());
        assert!(cfg(&[(vars::CORS_ORIGINS, "a.example")]).is_err());
        let c = cfg(&[(
            vars::CORS_ORIGINS,
            "https://play.example, http://tauri.localhost",
        )])
        .unwrap();
        assert_eq!(c.cors_origins.len(), 2);
    }

    #[test]
    fn short_tokens_rejected_without_echoing_them() {
        let err = cfg(&[(vars::AUTH_TOKENS, "hunter2")]).unwrap_err();
        assert!(!err.to_string().contains("hunter2"));
    }

    #[test]
    fn debug_output_redacts_tokens() {
        let c = cfg(&[(vars::AUTH_TOKENS, TOKEN)]).unwrap();
        let dbg = format!("{c:?}");
        assert!(!dbg.contains(TOKEN), "token leaked into Debug: {dbg}");
        assert!(dbg.contains("redacted"));
    }

    #[test]
    fn bad_bool_rejected() {
        assert!(cfg(&[(vars::TRUST_PROXY, "maybe")]).is_err());
        assert!(cfg(&[(vars::TRUST_PROXY, "1")]).unwrap().trust_proxy);
    }
}
