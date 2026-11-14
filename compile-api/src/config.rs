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
//! | `PNC_METRICS_TOKENS`        | *(unset: `/metrics` is loopback-only)*        |
//! | `PNC_METRICS_BIND`          | `127.0.0.1:9878` (loopback only, never proxied) |
//! | `PNC_BUILDER_SOCKET`        | *(unset: build in this process, dev only)*    |
//! | `PNC_BUILD_SANDBOX`         | `bwrap` on Linux, `none` elsewhere            |
//! | `PNC_TOOLCHAIN_DIR`         | *(required for `bwrap`)*                      |
//! | `PNC_BWRAP_PATH`            | `/usr/bin/bwrap`                              |
//! | `PNC_PRLIMIT_PATH`          | `/usr/bin/prlimit`                            |
//! | `PNC_BWRAP_PROC`            | `none` (`fresh`: `--proc /proc`)              |
//! | `PNC_BUILD_ADDRESS_SPACE_MIB` | `1280` (512..=1408, below unit MemoryMax)   |
//! | `PNC_BUILD_ROOT`            | `<temp>/pledge-wasm-builder`                  |
//! | `PNC_RATE_LIMIT_MAX_CLIENTS`| `100000`                                      |
//! | `PNC_SOURCE_DENYLIST`       | `true`                                        |
//! | `PNC_HEADER_READ_TIMEOUT_SECS` | `10` (1..=60)                              |
//! | `PNC_BODY_READ_TIMEOUT_SECS` | `15` (1..=120)                               |
//! | `PNC_REQUEST_DEADLINE_SECS` | `120` (70..=600)                              |
//! | `PNC_ALLOW_TRUST_PROXY_PUBLIC_BIND` | `false`                               |
//! | `PNC_DEV_UNSAFE_LOCAL_BUILDS` | `false` (dev override; loopback bind only)  |
//!
//! # Safety invariants (enforced by [`Config::validate`])
//!
//! "Exposed" below means: any bearer token configured, OR a non-loopback
//! bind, OR `PNC_TRUST_PROXY` on.
//!
//! - With no bearer token configured the server refuses to bind to
//!   anything but a loopback address
//!   ([`ConfigError::UnauthenticatedPublicBind`]).
//! - `PNC_TRUST_PROXY` with a non-loopback bind is refused unless
//!   `PNC_ALLOW_TRUST_PROXY_PUBLIC_BIND=true`: anyone reaching the socket
//!   directly could spoof `X-Forwarded-For` and dodge the rate limit.
//! - `PNC_DEV_UNSAFE_LOCAL_BUILDS=true` is refused on a non-loopback bind,
//!   full stop ([`ConfigError::DevOverrideOnPublicBind`]).
//! - `/metrics` is served on its own listener, `PNC_METRICS_BIND`, which
//!   must be loopback ([`ConfigError::MetricsPublicBind`]); the reverse
//!   proxy only ever forwards to `PNC_BIND`.
//! - Exposed and building in this process (no `PNC_BUILDER_SOCKET`):
//!   - `PNC_BUILD_SANDBOX=none` is refused
//!     ([`ConfigError::UnsandboxedExposure`]) on every platform;
//!   - on Linux, building at all in the token-holding process is refused
//!     ([`ConfigError::InProcessExposure`]): production runs player code
//!     in the separate builder service (`pnc-builder.service`);
//!
//!   each unless `PNC_DEV_UNSAFE_LOCAL_BUILDS=true` (loopback only).
//! - Exposed with `PNC_SOURCE_DENYLIST=false` is refused unless the dev
//!   override is set ([`ConfigError::DenylistOffExposed`]).
//! - `bwrap` is Linux-only and needs `PNC_TOOLCHAIN_DIR`; the builder
//!   executor is Linux-only and needs an absolute socket path.
//! - The removed `PNC_ALLOW_UNSANDBOXED_BUILDS` is refused if set to true,
//!   so an old env file cannot silently mean something else.
//!
//! Secrets (`PNC_AUTH_TOKENS`) never appear in `Debug` output or in error
//! messages.

use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::http::HeaderValue;

use crate::build_sandbox::{
    BuildRlimits, BwrapProc, DEFAULT_BUILD_ADDRESS_SPACE_MIB, MAX_BUILD_ADDRESS_SPACE_MIB,
    MIN_BUILD_ADDRESS_SPACE_MIB, SandboxMode, SandboxSettings, validate_toolchain_dir,
};
use crate::rate_limit::MAX_TRACKED_CLIENTS;
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
    pub const METRICS_TOKENS: &str = "PNC_METRICS_TOKENS";
    pub const BUILD_SANDBOX: &str = "PNC_BUILD_SANDBOX";
    pub const TOOLCHAIN_DIR: &str = "PNC_TOOLCHAIN_DIR";
    pub const BWRAP_PATH: &str = "PNC_BWRAP_PATH";
    pub const PRLIMIT_PATH: &str = "PNC_PRLIMIT_PATH";
    pub const BUILD_ADDRESS_SPACE_MIB: &str = "PNC_BUILD_ADDRESS_SPACE_MIB";
    pub const BUILD_ROOT: &str = "PNC_BUILD_ROOT";
    pub const RATE_LIMIT_MAX_CLIENTS: &str = "PNC_RATE_LIMIT_MAX_CLIENTS";
    pub const SOURCE_DENYLIST: &str = "PNC_SOURCE_DENYLIST";
    pub const ALLOW_TRUST_PROXY_PUBLIC_BIND: &str = "PNC_ALLOW_TRUST_PROXY_PUBLIC_BIND";
    /// Removed (replaced by [`DEV_UNSAFE_LOCAL_BUILDS`]); refused if true.
    pub const ALLOW_UNSANDBOXED_BUILDS: &str = "PNC_ALLOW_UNSANDBOXED_BUILDS";
    pub const DEV_UNSAFE_LOCAL_BUILDS: &str = "PNC_DEV_UNSAFE_LOCAL_BUILDS";
    pub const METRICS_BIND: &str = "PNC_METRICS_BIND";
    pub const BUILDER_SOCKET: &str = "PNC_BUILDER_SOCKET";
    pub const BUILDER_MAX_JOBS: &str = "PNC_BUILDER_MAX_JOBS";
    pub const BWRAP_PROC: &str = "PNC_BWRAP_PROC";
    pub const HEADER_READ_TIMEOUT_SECS: &str = "PNC_HEADER_READ_TIMEOUT_SECS";
    pub const BODY_READ_TIMEOUT_SECS: &str = "PNC_BODY_READ_TIMEOUT_SECS";
    pub const REQUEST_DEADLINE_SECS: &str = "PNC_REQUEST_DEADLINE_SECS";
}

/// Default build root dir name under the system temp dir.
pub const DEFAULT_BUILD_ROOT_NAME: &str = "pledge-wasm-builder";

/// Default bind address. Loopback: safe with no auth configured.
pub const DEFAULT_BIND: &str = "127.0.0.1:7878";
/// Default `/metrics` listener. Loopback only, never proxied.
pub const DEFAULT_METRICS_BIND: &str = "127.0.0.1:9878";
/// Default builder socket (the builder unit's `RuntimeDirectory`).
pub const DEFAULT_BUILDER_SOCKET: &str = "/run/pnc-builder/builder.sock";
/// Time allowed for a client to send the request line + headers.
pub const DEFAULT_HEADER_READ_TIMEOUT: Duration = Duration::from_secs(10);
/// Time allowed for a client to send the whole (<= 68 KiB) body.
pub const DEFAULT_BODY_READ_TIMEOUT: Duration = Duration::from_secs(15);
/// Total time for one request, queue wait and build included. Must exceed
/// the builder reply deadline (`ipc::REPLY_DEADLINE`, 65 s).
pub const DEFAULT_REQUEST_DEADLINE: Duration = Duration::from_secs(120);
const MIN_REQUEST_DEADLINE_SECS: u64 = 70;
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
const MAX_RATE_LIMIT_CLIENTS_CAP: usize = 1_000_000;

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
    #[error(
        "refusing to bind {addr} with {} on: clients reaching the socket directly could spoof \
         X-Forwarded-For (bind to loopback behind your proxy, or set {}=true if a firewall \
         guarantees only the proxy can connect)",
        vars::TRUST_PROXY,
        vars::ALLOW_TRUST_PROXY_PUBLIC_BIND
    )]
    TrustProxyPublicBind { addr: SocketAddr },
    #[error(
        "refusing to start with {}=none: {reason}. Unsandboxed builds let player source read \
         any file the service user can (env file, /proc). Use the builder service with \
         {}=bwrap (local dev only: {}=true on a loopback bind)",
        vars::BUILD_SANDBOX,
        vars::BUILD_SANDBOX,
        vars::DEV_UNSAFE_LOCAL_BUILDS
    )]
    UnsandboxedExposure { reason: String },
    #[error(
        "refusing to build player code inside the token-holding server process: {reason}. Run \
         the builder service and set {} (local dev only: {}=true on a loopback bind)",
        vars::BUILDER_SOCKET,
        vars::DEV_UNSAFE_LOCAL_BUILDS
    )]
    InProcessExposure { reason: String },
    #[error(
        "refusing {}=true with non-loopback bind {addr}: the dev override is loopback-only",
        vars::DEV_UNSAFE_LOCAL_BUILDS
    )]
    DevOverrideOnPublicBind { addr: SocketAddr },
    #[error(
        "refusing {}={addr}: the metrics listener must be loopback (it is never proxied)",
        vars::METRICS_BIND
    )]
    MetricsPublicBind { addr: SocketAddr },
    #[error("{}={addr} collides with {}", vars::METRICS_BIND, vars::BIND)]
    MetricsBindConflict { addr: SocketAddr },
    #[error(
        "refusing {}=false on an exposed deployment ({reason}); it is only for local dev",
        vars::SOURCE_DENYLIST
    )]
    DenylistOffExposed { reason: String },
    #[error("{var} was removed; {hint}")]
    RemovedVariable {
        var: &'static str,
        hint: &'static str,
    },
    #[error("{}=bwrap is only supported on Linux", vars::BUILD_SANDBOX)]
    SandboxUnsupported,
    #[error(
        "{} (the builder service) is only supported on Linux",
        vars::BUILDER_SOCKET
    )]
    BuilderUnsupported,
    #[error(
        "refusing to start the builder with {var} set: the builder must hold no secrets (keep \
         tokens in the API service's env file only)"
    )]
    BuilderHoldsSecrets { var: &'static str },
}

/// Where `/compile-real` builds and runs player code.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Executor {
    /// In this process (local dev and tests). Refused when exposed on
    /// Linux; see the module docs.
    #[default]
    InProcess,
    /// One `<exe> job` child process per request, killed at its deadline
    /// (programmatic only: tests and local dev).
    Subprocess { exe: PathBuf },
    /// The builder service over its Unix socket (production).
    Builder { socket: PathBuf },
}

impl Executor {
    /// True when player code runs under this process's own unit/user.
    pub fn is_local(&self) -> bool {
        !matches!(self, Self::Builder { .. })
    }
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
    /// Tokens accepted on `/metrics` only. Player tokens are NOT accepted
    /// there. Empty: `/metrics` answers only direct loopback clients.
    pub metrics_tokens: Vec<Secret>,
    /// Build sandbox (see [`crate::build_sandbox`]).
    pub sandbox: SandboxSettings,
    /// Parent dir of per-request job dirs.
    pub build_root: PathBuf,
    /// Distinct clients tracked per rate limiter before pressure handling.
    pub rate_limit_max_clients: usize,
    /// Second-layer source deny-list ([`crate::source_policy`]). Can only
    /// be turned off on an unexposed (local dev) server.
    pub source_denylist: bool,
    pub allow_trust_proxy_public_bind: bool,
    /// `PNC_DEV_UNSAFE_LOCAL_BUILDS`: allow unsandboxed / in-process
    /// builds on an exposed-looking config. Loopback bind only.
    pub dev_unsafe_local_builds: bool,
    /// Where `/compile-real` work runs.
    pub executor: Executor,
    /// `/metrics` listener (loopback only).
    pub metrics_bind: SocketAddr,
    /// Request line + headers must arrive within this.
    pub header_read_timeout: Duration,
    /// The whole body must arrive within this.
    pub body_read_timeout: Duration,
    /// Total time for one request (queue wait + build included).
    pub request_deadline: Duration,
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
            metrics_tokens: Vec::new(),
            sandbox: SandboxSettings::default(),
            build_root: crate::build_root::default_root(DEFAULT_BUILD_ROOT_NAME),
            rate_limit_max_clients: MAX_TRACKED_CLIENTS,
            source_denylist: true,
            allow_trust_proxy_public_bind: false,
            dev_unsafe_local_builds: false,
            executor: Executor::InProcess,
            metrics_bind: SocketAddr::from(([127, 0, 0, 1], 9878)),
            header_read_timeout: DEFAULT_HEADER_READ_TIMEOUT,
            body_read_timeout: DEFAULT_BODY_READ_TIMEOUT,
            request_deadline: DEFAULT_REQUEST_DEADLINE,
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
            .field(
                "metrics_tokens",
                &format_args!("<redacted; {} configured>", self.metrics_tokens.len()),
            )
            .field("sandbox", &self.sandbox)
            .field("build_root", &self.build_root)
            .field("rate_limit_max_clients", &self.rate_limit_max_clients)
            .field("source_denylist", &self.source_denylist)
            .field(
                "allow_trust_proxy_public_bind",
                &self.allow_trust_proxy_public_bind,
            )
            .field("dev_unsafe_local_builds", &self.dev_unsafe_local_builds)
            .field("executor", &self.executor)
            .field("metrics_bind", &self.metrics_bind)
            .field("header_read_timeout", &self.header_read_timeout)
            .field("body_read_timeout", &self.body_read_timeout)
            .field("request_deadline", &self.request_deadline)
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

        let metrics_tokens = match lookup(vars::METRICS_TOKENS) {
            None => Vec::new(),
            Some(v) => parse_tokens_for(vars::METRICS_TOKENS, &v)?,
        };

        let sandbox = parse_sandbox(&get)?;
        let build_root = get(vars::BUILD_ROOT)
            .map(PathBuf::from)
            .unwrap_or(d.build_root);
        let rate_limit_max_clients = parse_ranged(
            vars::RATE_LIMIT_MAX_CLIENTS,
            get(vars::RATE_LIMIT_MAX_CLIENTS),
            d.rate_limit_max_clients,
            1,
            MAX_RATE_LIMIT_CLIENTS_CAP,
        )?;
        let source_denylist = match get(vars::SOURCE_DENYLIST) {
            None => d.source_denylist,
            Some(v) => parse_bool(vars::SOURCE_DENYLIST, &v)?,
        };
        let allow_trust_proxy_public_bind = match get(vars::ALLOW_TRUST_PROXY_PUBLIC_BIND) {
            None => false,
            Some(v) => parse_bool(vars::ALLOW_TRUST_PROXY_PUBLIC_BIND, &v)?,
        };
        reject_removed_override(&get)?;
        let dev_unsafe_local_builds = parse_dev_override(&get)?;
        let executor = match get(vars::BUILDER_SOCKET) {
            None => Executor::InProcess,
            Some(v) => Executor::Builder {
                socket: parse_socket_path(&v)?,
            },
        };
        let metrics_bind = match get(vars::METRICS_BIND) {
            None => d.metrics_bind,
            Some(v) => v.parse().map_err(|e| ConfigError::Invalid {
                var: vars::METRICS_BIND,
                value: v.clone(),
                reason: format!("{e} (expected ip:port)"),
            })?,
        };
        let secs = |var: &'static str, default: Duration, min: u64, max: u64| {
            parse_ranged(var, get(var), default.as_secs(), min, max).map(Duration::from_secs)
        };
        let header_read_timeout = secs(
            vars::HEADER_READ_TIMEOUT_SECS,
            DEFAULT_HEADER_READ_TIMEOUT,
            1,
            60,
        )?;
        let body_read_timeout = secs(
            vars::BODY_READ_TIMEOUT_SECS,
            DEFAULT_BODY_READ_TIMEOUT,
            1,
            120,
        )?;
        let request_deadline = secs(
            vars::REQUEST_DEADLINE_SECS,
            DEFAULT_REQUEST_DEADLINE,
            MIN_REQUEST_DEADLINE_SECS,
            600,
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
            metrics_tokens,
            sandbox,
            build_root,
            rate_limit_max_clients,
            source_denylist,
            allow_trust_proxy_public_bind,
            dev_unsafe_local_builds,
            executor,
            metrics_bind,
            header_read_timeout,
            body_read_timeout,
            request_deadline,
        };
        cfg.validate()?;
        Ok(cfg)
    }

    /// Why this config counts as exposed (see the module docs), or `None`.
    pub fn exposure(&self) -> Option<String> {
        let mut why = Vec::new();
        if !self.auth_tokens.is_empty() {
            why.push("bearer tokens are configured".to_string());
        }
        if !self.bind_addr.ip().is_loopback() {
            why.push(format!("bind address {} is not loopback", self.bind_addr));
        }
        if self.trust_proxy {
            why.push(format!("{} is on (proxied deployment)", vars::TRUST_PROXY));
        }
        (!why.is_empty()).then(|| why.join(", "))
    }

    /// Cross-field invariants. Called by [`Config::from_lookup`]; call it
    /// yourself if you build a `Config` by hand for a public bind.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.validate_for(cfg!(target_os = "linux"))
    }

    /// [`Config::validate`] with the platform made explicit, so the Linux
    /// rules are unit-tested on every platform.
    pub fn validate_for(&self, linux: bool) -> Result<(), ConfigError> {
        let loopback = self.bind_addr.ip().is_loopback();
        if self.auth_tokens.is_empty() && !loopback {
            return Err(ConfigError::UnauthenticatedPublicBind {
                addr: self.bind_addr,
            });
        }
        if self.trust_proxy && !loopback && !self.allow_trust_proxy_public_bind {
            return Err(ConfigError::TrustProxyPublicBind {
                addr: self.bind_addr,
            });
        }
        if self.dev_unsafe_local_builds && !loopback {
            return Err(ConfigError::DevOverrideOnPublicBind {
                addr: self.bind_addr,
            });
        }
        if !self.metrics_bind.ip().is_loopback() {
            return Err(ConfigError::MetricsPublicBind {
                addr: self.metrics_bind,
            });
        }
        if self.metrics_bind == self.bind_addr {
            return Err(ConfigError::MetricsBindConflict {
                addr: self.metrics_bind,
            });
        }
        let exposure = self.exposure();
        let refused_exposure = exposure.filter(|_| !self.dev_unsafe_local_builds);
        if !self.source_denylist
            && let Some(reason) = refused_exposure.clone()
        {
            return Err(ConfigError::DenylistOffExposed { reason });
        }
        match &self.executor {
            Executor::Builder { socket } => {
                if !linux {
                    return Err(ConfigError::BuilderUnsupported);
                }
                check_socket_path(socket)?;
                // The API's own sandbox settings are unused: the builder
                // validates its own (see `builder::BuilderConfig`).
            }
            Executor::InProcess | Executor::Subprocess { .. } => {
                if linux && let Some(reason) = refused_exposure.clone() {
                    return Err(ConfigError::InProcessExposure { reason });
                }
                match self.sandbox.mode {
                    SandboxMode::Bwrap => check_bwrap(&self.sandbox, linux)?,
                    SandboxMode::None => {
                        if let Some(reason) = refused_exposure {
                            return Err(ConfigError::UnsandboxedExposure { reason });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn auth_enabled(&self) -> bool {
        !self.auth_tokens.is_empty()
    }
}

/// `bwrap` needs Linux and a toolchain dir.
pub(crate) fn check_bwrap(sandbox: &SandboxSettings, linux: bool) -> Result<(), ConfigError> {
    if !linux {
        return Err(ConfigError::SandboxUnsupported);
    }
    if sandbox.toolchain_dir.is_none() {
        return Err(ConfigError::Invalid {
            var: vars::TOOLCHAIN_DIR,
            value: String::new(),
            reason: format!(
                "required when {}=bwrap (the default on Linux; for local dev set {}=none)",
                vars::BUILD_SANDBOX,
                vars::BUILD_SANDBOX
            ),
        });
    }
    Ok(())
}

fn check_socket_path(p: &Path) -> Result<(), ConfigError> {
    let s = p.to_string_lossy();
    if !s.starts_with('/') || s.contains("/../") || s.ends_with("/..") || s.len() > 100 {
        return Err(ConfigError::Invalid {
            var: vars::BUILDER_SOCKET,
            value: s.into_owned(),
            reason: "must be an absolute Unix path without `..`, at most 100 bytes".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn parse_socket_path(v: &str) -> Result<PathBuf, ConfigError> {
    let p = PathBuf::from(v);
    check_socket_path(&p)?;
    Ok(p)
}

/// `PNC_ALLOW_UNSANDBOXED_BUILDS` was removed: refuse it when truthy so an
/// old env file cannot keep meaning "unsandboxed is fine".
pub(crate) fn reject_removed_override(
    get: &impl Fn(&str) -> Option<String>,
) -> Result<(), ConfigError> {
    if let Some(v) = get(vars::ALLOW_UNSANDBOXED_BUILDS)
        && parse_bool(vars::ALLOW_UNSANDBOXED_BUILDS, &v)?
    {
        return Err(ConfigError::RemovedVariable {
            var: vars::ALLOW_UNSANDBOXED_BUILDS,
            hint: "production builds run in the builder service; for local dev on a loopback \
                   bind use PNC_DEV_UNSAFE_LOCAL_BUILDS=true",
        });
    }
    Ok(())
}

pub(crate) fn parse_dev_override(
    get: &impl Fn(&str) -> Option<String>,
) -> Result<bool, ConfigError> {
    match get(vars::DEV_UNSAFE_LOCAL_BUILDS) {
        None => Ok(false),
        Some(v) => parse_bool(vars::DEV_UNSAFE_LOCAL_BUILDS, &v),
    }
}

/// The build-sandbox settings, shared by the API config and the builder.
/// `PNC_BUILD_SANDBOX` unset means [`SandboxMode::platform_default`].
pub(crate) fn parse_sandbox(
    get: &impl Fn(&str) -> Option<String>,
) -> Result<SandboxSettings, ConfigError> {
    let d = SandboxSettings::default();
    let mode = match get(vars::BUILD_SANDBOX) {
        None => SandboxMode::platform_default(),
        Some(v) => v.parse().map_err(|reason| ConfigError::Invalid {
            var: vars::BUILD_SANDBOX,
            value: v.clone(),
            reason,
        })?,
    };
    let toolchain_dir = match get(vars::TOOLCHAIN_DIR) {
        None => None,
        Some(v) => {
            let p = PathBuf::from(&v);
            validate_toolchain_dir(&p).map_err(|reason| ConfigError::Invalid {
                var: vars::TOOLCHAIN_DIR,
                value: v.clone(),
                reason,
            })?;
            Some(p)
        }
    };
    let address_space_mib: u64 = parse_ranged(
        vars::BUILD_ADDRESS_SPACE_MIB,
        get(vars::BUILD_ADDRESS_SPACE_MIB),
        DEFAULT_BUILD_ADDRESS_SPACE_MIB,
        MIN_BUILD_ADDRESS_SPACE_MIB,
        MAX_BUILD_ADDRESS_SPACE_MIB,
    )?;
    let proc_mode = match get(vars::BWRAP_PROC) {
        None => BwrapProc::default(),
        Some(v) => v.parse().map_err(|reason| ConfigError::Invalid {
            var: vars::BWRAP_PROC,
            value: v.clone(),
            reason,
        })?,
    };
    Ok(SandboxSettings {
        mode,
        toolchain_dir,
        bwrap_path: get(vars::BWRAP_PATH)
            .map(PathBuf::from)
            .unwrap_or(d.bwrap_path),
        prlimit_path: get(vars::PRLIMIT_PATH)
            .map(PathBuf::from)
            .unwrap_or(d.prlimit_path),
        rlimits: BuildRlimits {
            address_space_bytes: address_space_mib * 1024 * 1024,
            ..BuildRlimits::default()
        },
        proc_mode,
    })
}

pub(crate) fn parse_ranged<T>(
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

pub(crate) fn parse_bool(var: &'static str, v: &str) -> Result<bool, ConfigError> {
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
    parse_tokens_for(vars::AUTH_TOKENS, v)
}

fn parse_tokens_for(var: &'static str, v: &str) -> Result<Vec<Secret>, ConfigError> {
    let mut out = Vec::new();
    for (index, raw) in v
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        if raw.len() < MIN_TOKEN_LEN {
            return Err(ConfigError::TokenTooShort {
                var,
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

    /// Parse `pairs` as a local dev box would: `PNC_BUILD_SANDBOX=none`
    /// unless the test sets it (the env default is `bwrap` on Linux).
    fn cfg(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let mut map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        map.entry(vars::BUILD_SANDBOX.to_string())
            .or_insert_with(|| "none".to_string());
        Config::from_lookup(|k| map.get(k).cloned())
    }

    fn raw_cfg(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Config::from_lookup(|k| map.get(k).cloned())
    }

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn env_default_sandbox_is_bwrap_on_linux() {
        // No PNC_BUILD_SANDBOX at all: Linux gets bwrap (and so needs a
        // toolchain dir, a loud startup error), other platforms none.
        let r = raw_cfg(&[]);
        if cfg!(target_os = "linux") {
            assert!(
                matches!(&r, Err(ConfigError::Invalid { var, .. }) if *var == vars::TOOLCHAIN_DIR),
                "{r:?}"
            );
        } else {
            assert_eq!(r.unwrap().sandbox.mode, SandboxMode::None);
        }
    }

    #[test]
    fn defaults_are_valid_and_match_design() {
        let c = cfg(&[]).expect("defaults must validate");
        assert_eq!(c.metrics_bind.to_string(), DEFAULT_METRICS_BIND);
        assert_eq!(c.executor, Executor::InProcess);
        assert_eq!(c.request_deadline, DEFAULT_REQUEST_DEADLINE);
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
        let c = cfg(&[
            (vars::AUTH_TOKENS, TOKEN),
            (vars::DEV_UNSAFE_LOCAL_BUILDS, "true"),
        ])
        .unwrap();
        let dbg = format!("{c:?}");
        assert!(!dbg.contains(TOKEN), "token leaked into Debug: {dbg}");
        assert!(dbg.contains("redacted"));
    }

    const TC: &str = "/opt/rust/rustup/toolchains/stable-x86_64-unknown-linux-gnu";

    /// Build a config by hand and validate it for an explicit platform, so
    /// the Linux-only rules are exercised on every dev box.
    fn validate_on(linux: bool, edit: impl FnOnce(&mut Config)) -> Result<(), ConfigError> {
        let mut c = Config::default();
        edit(&mut c);
        c.validate_for(linux)
    }

    fn public_with_token(c: &mut Config) {
        c.bind_addr = "0.0.0.0:7878".parse().unwrap();
        c.auth_tokens = vec![Secret::new(TOKEN)];
    }

    fn bwrap(c: &mut Config) {
        c.sandbox.mode = SandboxMode::Bwrap;
        c.sandbox.toolchain_dir = Some(PathBuf::from(TC));
    }

    fn builder(c: &mut Config) {
        c.executor = Executor::Builder {
            socket: PathBuf::from(DEFAULT_BUILDER_SOCKET),
        };
    }

    fn with_tokens(c: &mut Config) {
        c.auth_tokens = vec![Secret::new(TOKEN)];
    }

    fn with_proxy(c: &mut Config) {
        c.trust_proxy = true;
    }

    fn with_tokens_and_proxy(c: &mut Config) {
        with_tokens(c);
        with_proxy(c);
    }

    type Exposure = (&'static str, fn(&mut Config));

    /// The "exposed" triggers, each applied to an otherwise-dev config.
    fn exposures() -> [Exposure; 4] {
        [
            ("tokens", with_tokens),
            ("public bind + token", public_with_token),
            ("trust proxy", with_proxy),
            ("tokens + trust proxy", with_tokens_and_proxy),
        ]
    }

    /// Re-review P1-B: `none` is refused whenever the config is exposed,
    /// on both platforms, without the dev override.
    #[test]
    fn unsandboxed_in_process_refused_whenever_exposed() {
        for linux in [true, false] {
            for (name, expose) in exposures() {
                let r = validate_on(linux, expose);
                let expected_variant = if linux {
                    matches!(r, Err(ConfigError::InProcessExposure { .. }))
                } else {
                    matches!(r, Err(ConfigError::UnsandboxedExposure { .. }))
                };
                assert!(expected_variant, "linux={linux} {name}: {r:?}");
            }
            // Plain loopback, no token, no proxy: local dev stays allowed.
            assert!(validate_on(linux, |_| {}).is_ok(), "linux={linux}");
        }
    }

    /// The dev override unlocks loopback exposure shapes (tokens, trust
    /// proxy) and is refused outright on a non-loopback bind.
    #[test]
    fn dev_override_is_loopback_only() {
        for linux in [true, false] {
            for (name, expose) in exposures() {
                let r = validate_on(linux, |c| {
                    expose(c);
                    c.dev_unsafe_local_builds = true;
                });
                if name.starts_with("public") {
                    assert!(
                        matches!(r, Err(ConfigError::DevOverrideOnPublicBind { .. })),
                        "linux={linux} {name}: {r:?}"
                    );
                } else {
                    assert!(r.is_ok(), "linux={linux} {name}: {r:?}");
                }
            }
            // Override on a public bind is refused even with a builder.
            let r = validate_on(linux, |c| {
                public_with_token(c);
                builder(c);
                c.dev_unsafe_local_builds = true;
            });
            assert!(
                matches!(r, Err(ConfigError::DevOverrideOnPublicBind { .. })),
                "{r:?}"
            );
        }
    }

    /// Linux production shape: exposed + builder executor. Building in the
    /// token-holding process is refused even with bwrap.
    #[test]
    fn exposed_linux_requires_the_builder() {
        for (name, expose) in exposures() {
            let with_bwrap = validate_on(true, |c| {
                expose(c);
                bwrap(c);
            });
            assert!(
                matches!(with_bwrap, Err(ConfigError::InProcessExposure { .. })),
                "{name}: {with_bwrap:?}"
            );
            let subprocess = validate_on(true, |c| {
                expose(c);
                bwrap(c);
                c.executor = Executor::Subprocess {
                    exe: PathBuf::from("/usr/local/bin/pledge_compile_api"),
                };
            });
            assert!(
                matches!(subprocess, Err(ConfigError::InProcessExposure { .. })),
                "{name}: {subprocess:?}"
            );
            // With the builder the API's own sandbox mode is irrelevant.
            let ok = validate_on(true, |c| {
                expose(c);
                builder(c);
            });
            assert!(ok.is_ok(), "{name}: {ok:?}");
            assert!(matches!(
                validate_on(false, |c| {
                    expose(c);
                    builder(c);
                }),
                Err(ConfigError::BuilderUnsupported)
            ));
        }
        assert!(matches!(
            validate_on(true, |c| {
                c.executor = Executor::Builder {
                    socket: PathBuf::from("relative.sock"),
                };
            }),
            Err(ConfigError::Invalid { .. })
        ));
    }

    #[test]
    fn env_parsing_of_the_p1b_rules() {
        // Exposed via env with the dev-box sandbox (none).
        assert!(cfg(&[(vars::AUTH_TOKENS, TOKEN)]).is_err());
        assert!(cfg(&[(vars::TRUST_PROXY, "true")]).is_err());
        assert!(
            cfg(&[
                (vars::AUTH_TOKENS, TOKEN),
                (vars::DEV_UNSAFE_LOCAL_BUILDS, "true")
            ])
            .is_ok()
        );
        assert!(matches!(
            cfg(&[
                (vars::BIND, "0.0.0.0:7878"),
                (vars::AUTH_TOKENS, TOKEN),
                (vars::DEV_UNSAFE_LOCAL_BUILDS, "true")
            ]),
            Err(ConfigError::DevOverrideOnPublicBind { .. })
        ));
        // The removed variable is refused when true, ignored when false.
        assert!(matches!(
            cfg(&[(vars::ALLOW_UNSANDBOXED_BUILDS, "true")]),
            Err(ConfigError::RemovedVariable { .. })
        ));
        assert!(cfg(&[(vars::ALLOW_UNSANDBOXED_BUILDS, "false")]).is_ok());
        // Builder socket via env.
        let c = cfg(&[(vars::BUILDER_SOCKET, DEFAULT_BUILDER_SOCKET)]);
        if cfg!(target_os = "linux") {
            assert!(matches!(c.unwrap().executor, Executor::Builder { .. }));
        } else {
            assert!(matches!(c, Err(ConfigError::BuilderUnsupported)));
        }
    }

    #[test]
    fn denylist_off_only_when_unexposed() {
        assert!(validate_on(true, |c| c.source_denylist = false).is_ok());
        for linux in [true, false] {
            let r = validate_on(linux, |c| {
                c.source_denylist = false;
                with_tokens(c);
                builder(c);
            });
            assert!(
                matches!(r, Err(ConfigError::DenylistOffExposed { .. })),
                "{r:?}"
            );
        }
    }

    #[test]
    fn metrics_listener_must_be_loopback_and_distinct() {
        assert!(matches!(
            cfg(&[(vars::METRICS_BIND, "0.0.0.0:9878")]),
            Err(ConfigError::MetricsPublicBind { .. })
        ));
        assert!(matches!(
            cfg(&[(vars::METRICS_BIND, DEFAULT_BIND)]),
            Err(ConfigError::MetricsBindConflict { .. })
        ));
        assert!(cfg(&[(vars::METRICS_BIND, "[::1]:9900")]).is_ok());
    }

    #[test]
    fn timeouts_parse_within_bounds() {
        let c = cfg(&[
            (vars::HEADER_READ_TIMEOUT_SECS, "5"),
            (vars::BODY_READ_TIMEOUT_SECS, "7"),
            (vars::REQUEST_DEADLINE_SECS, "90"),
        ])
        .unwrap();
        assert_eq!(c.header_read_timeout, Duration::from_secs(5));
        assert_eq!(c.body_read_timeout, Duration::from_secs(7));
        assert_eq!(c.request_deadline, Duration::from_secs(90));
        assert!(cfg(&[(vars::HEADER_READ_TIMEOUT_SECS, "0")]).is_err());
        assert!(cfg(&[(vars::BODY_READ_TIMEOUT_SECS, "0")]).is_err());
        // The deadline cannot undercut a build (65 s builder reply budget).
        assert!(cfg(&[(vars::REQUEST_DEADLINE_SECS, "30")]).is_err());
    }

    #[test]
    fn bwrap_needs_linux_and_toolchain() {
        assert!(matches!(
            validate_on(false, bwrap),
            Err(ConfigError::SandboxUnsupported)
        ));
        assert!(validate_on(true, bwrap).is_ok());
        assert!(
            validate_on(true, |c| c.sandbox.mode = SandboxMode::Bwrap).is_err(),
            "bwrap without a toolchain dir"
        );
        assert!(cfg(&[(vars::TOOLCHAIN_DIR, "/etc/rust")]).is_err());
        assert!(cfg(&[(vars::BUILD_SANDBOX, "docker")]).is_err());
        assert!(cfg(&[(vars::BWRAP_PROC, "host")]).is_err());
    }

    #[test]
    fn trust_proxy_on_public_bind_needs_explicit_override() {
        let base = |c: &mut Config| {
            public_with_token(c);
            builder(c);
            c.trust_proxy = true;
        };
        assert!(matches!(
            validate_on(true, base),
            Err(ConfigError::TrustProxyPublicBind { .. })
        ));
        assert!(
            validate_on(true, |c| {
                base(c);
                c.allow_trust_proxy_public_bind = true;
            })
            .is_ok()
        );
    }

    #[test]
    fn metrics_tokens_and_build_cap_parse() {
        let c = cfg(&[(vars::METRICS_TOKENS, TOKEN)]).unwrap();
        assert_eq!(c.metrics_tokens.len(), 1);
        assert!(!format!("{c:?}").contains(TOKEN));
        assert!(cfg(&[(vars::METRICS_TOKENS, "short")]).is_err());
        assert!(cfg(&[(vars::BUILD_ADDRESS_SPACE_MIB, "1408")]).is_ok());
        assert!(cfg(&[(vars::BUILD_ADDRESS_SPACE_MIB, "1536")]).is_err());
        assert!(cfg(&[(vars::RATE_LIMIT_MAX_CLIENTS, "0")]).is_err());
        assert!(
            !cfg(&[(vars::SOURCE_DENYLIST, "false")])
                .unwrap()
                .source_denylist
        );
        assert!(cfg(&[]).unwrap().source_denylist);
    }

    #[test]
    fn bad_bool_rejected() {
        assert!(cfg(&[(vars::TRUST_PROXY, "maybe")]).is_err());
        // (The dev override keeps this parse test valid: TRUST_PROXY with
        // an in-process unsandboxed build is otherwise refused.)
        assert!(
            cfg(&[
                (vars::TRUST_PROXY, "1"),
                (vars::DEV_UNSAFE_LOCAL_BUILDS, "1")
            ])
            .unwrap()
            .trust_proxy
        );
    }
}
