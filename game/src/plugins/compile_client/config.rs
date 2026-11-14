// SPDX-License-Identifier: MIT
//! Typed configuration for the game's compile-API client.
//!
//! Parsed once at startup into the [`CompileApiConfig`] resource.
//!
//! # Sources
//!
//! **Native** (environment variables):
//!
//! | Variable                  | Meaning                                              |
//! |---------------------------|------------------------------------------------------|
//! | `PLEDGE_API_URL`          | Base URL, `http://` or `https://`. Default `http://127.0.0.1:7878`. `off` / `offline` / `none` = offline demo mode. |
//! | `PLEDGE_API_TOKEN`        | Optional bearer token sent as `Authorization: Bearer`. |
//! | `PLEDGE_REAL_COMPILE`     | `1`/`true`/`yes`/`on` routes to `/compile-real`.      |
//! | `PLEDGE_API_TIMEOUT_SECS` | Optional request timeout, 1..=300 seconds.           |
//!
//! **Web** (wasm32): the page may define
//! `window.PLEDGE_CONFIG = { apiUrl, apiToken, realCompile }` before the
//! game loads (see `web/index.html`). No object, or no `apiUrl`, means
//! offline demo mode. URL query parameters are deliberately never read:
//! a link must not be able to inject a token or redirect a player's code
//! to someone else's server.
//!
//! A token shipped to a browser is public (anyone can read it out of the
//! page), so on the web it is an abuse speed-bump for the server's rate
//! limiter, not a secret.
//!
//! # Targets
//!
//! - [`ApiTarget::Remote`]: send code to the server. Only server verdicts
//!   count as clears; if the server cannot be reached the offline stub
//!   grader gives feedback that does not count.
//! - [`ApiTarget::Offline`]: no server configured on purpose (web demo,
//!   `PLEDGE_API_URL=off`). The stub grader is the grader and its passes
//!   count.
//! - [`ApiTarget::Misconfigured`]: a server was asked for but the config
//!   is invalid. Treated like an unreachable server (stub feedback, no
//!   clears) so a typo can never silently turn grading into substring
//!   matching that saves progress.

use std::fmt;
use std::time::Duration;

use bevy::prelude::Resource;

/// Env var names, public so tests and docs share one spelling.
pub mod vars {
    pub const API_URL: &str = "PLEDGE_API_URL";
    pub const API_TOKEN: &str = "PLEDGE_API_TOKEN";
    pub const REAL_COMPILE: &str = "PLEDGE_REAL_COMPILE";
    pub const TIMEOUT_SECS: &str = "PLEDGE_API_TIMEOUT_SECS";
}

/// Native default: the local dev server.
pub const DEFAULT_API_URL: &str = "http://127.0.0.1:7878";
/// Pattern grader answers in ~100 ms; 10 s covers a slow link.
pub const DEFAULT_PATTERN_TIMEOUT: Duration = Duration::from_secs(10);
/// Real builds take 1-3 s warm but may wait in the server's build queue.
pub const DEFAULT_REAL_TIMEOUT: Duration = Duration::from_secs(60);
/// Upper bound for an explicit timeout.
pub const MAX_TIMEOUT_SECS: u64 = 300;

/// Words that select offline demo mode when used as the API URL.
const OFFLINE_WORDS: &[&str] = &["off", "offline", "none"];

/// Why a configuration was rejected. Never carries the token value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("API URL {url:?} is invalid: {reason}")]
    InvalidUrl { url: String, reason: &'static str },
    #[error("API token is invalid: {0}")]
    InvalidToken(&'static str),
    #[error("timeout {value:?} is invalid: expected whole seconds 1..={MAX_TIMEOUT_SECS}")]
    InvalidTimeout { value: String },
}

/// Which server route grades the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    /// `/compile`: fast pattern grader.
    Pattern,
    /// `/compile-real`: real build + sandboxed run.
    Real,
}

impl CompileMode {
    pub fn path(self) -> &'static str {
        match self {
            Self::Pattern => "/compile",
            Self::Real => "/compile-real",
        }
    }

    fn default_timeout(self) -> Duration {
        match self {
            Self::Pattern => DEFAULT_PATTERN_TIMEOUT,
            Self::Real => DEFAULT_REAL_TIMEOUT,
        }
    }
}

/// A bearer token. `Debug` and `Display` are redacted; the raw value is
/// only reachable through [`ApiToken::expose`].
#[derive(Clone, PartialEq, Eq)]
pub struct ApiToken(String);

impl ApiToken {
    /// Validate and wrap a token: non-empty, visible ASCII only (it goes
    /// into an HTTP header verbatim).
    pub fn new(raw: &str) -> Result<Self, ConfigError> {
        let t = raw.trim();
        if t.is_empty() {
            return Err(ConfigError::InvalidToken("empty"));
        }
        if !t.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(ConfigError::InvalidToken(
                "must be visible ASCII with no spaces",
            ));
        }
        Ok(Self(t.to_string()))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl fmt::Display for ApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// A validated server to talk to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiEndpoint {
    /// Scheme + host (+ optional path prefix), no trailing slash.
    pub base_url: String,
    pub token: Option<ApiToken>,
    pub mode: CompileMode,
    pub timeout: Duration,
}

impl ApiEndpoint {
    /// Full URL of the compile route for this mode.
    pub fn compile_url(&self) -> String {
        format!("{}{}", self.base_url, self.mode.path())
    }
}

/// Where compile requests go. See the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiTarget {
    Remote(ApiEndpoint),
    Offline,
    Misconfigured(ConfigError),
}

/// The client's configuration resource.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CompileApiConfig {
    pub target: ApiTarget,
}

impl CompileApiConfig {
    /// Offline demo mode: stub grader, passes count.
    pub fn offline() -> Self {
        Self {
            target: ApiTarget::Offline,
        }
    }

    /// True when stub-grader passes count as clears (no server on purpose).
    pub fn stub_is_canonical(&self) -> bool {
        matches!(self.target, ApiTarget::Offline)
    }

    /// Load from the platform's source (env on native, `window.PLEDGE_CONFIG`
    /// on web). Errors become [`ApiTarget::Misconfigured`] and are logged.
    pub fn load() -> Self {
        let raw = RawConfig::from_platform();
        let cfg = Self::from_raw(&raw, platform_default_url());
        if let ApiTarget::Misconfigured(e) = &cfg.target {
            tracing::error!(
                "compile-api client config invalid ({e}); code will get offline \
                 feedback only and passes will not be saved"
            );
        }
        cfg
    }

    /// Build from raw string inputs. `default_url` applies when no URL is
    /// given (native: localhost; web: `None` = offline).
    pub fn from_raw(raw: &RawConfig, default_url: Option<&str>) -> Self {
        let target = match parse_target(raw, default_url) {
            Ok(t) => t,
            Err(e) => ApiTarget::Misconfigured(e),
        };
        Self { target }
    }

    /// Human-readable summary for the startup log. Never contains the token.
    pub fn summary(&self) -> String {
        match &self.target {
            ApiTarget::Remote(ep) => format!(
                "remote url={} mode={:?} timeout={}s token={}",
                ep.compile_url(),
                ep.mode,
                ep.timeout.as_secs(),
                if ep.token.is_some() {
                    "<redacted>"
                } else {
                    "none"
                }
            ),
            ApiTarget::Offline => {
                "offline demo mode (stub grader is canonical; passes count)".to_string()
            }
            ApiTarget::Misconfigured(e) => format!("misconfigured ({e}); offline feedback only"),
        }
    }
}

/// Unvalidated inputs, from env or the page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawConfig {
    pub api_url: Option<String>,
    pub api_token: Option<String>,
    pub real_compile: Option<String>,
    pub timeout_secs: Option<String>,
}

impl RawConfig {
    /// Read from a key lookup using the native env var names. Empty and
    /// whitespace-only values count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let get = |k: &str| lookup(k).filter(|v| !v.trim().is_empty());
        Self {
            api_url: get(vars::API_URL),
            api_token: get(vars::API_TOKEN),
            real_compile: get(vars::REAL_COMPILE),
            timeout_secs: get(vars::TIMEOUT_SECS),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn from_platform() -> Self {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    #[cfg(target_arch = "wasm32")]
    fn from_platform() -> Self {
        read_page_config()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_default_url() -> Option<&'static str> {
    Some(DEFAULT_API_URL)
}

#[cfg(target_arch = "wasm32")]
fn platform_default_url() -> Option<&'static str> {
    None
}

/// `window.PLEDGE_CONFIG = { apiUrl, apiToken, realCompile }`. Missing
/// object or fields read as unset. Only the global object is consulted,
/// never `location.search`.
#[cfg(target_arch = "wasm32")]
fn read_page_config() -> RawConfig {
    use wasm_bindgen::JsValue;

    let Some(window) = web_sys::window() else {
        tracing::warn!("compile-api config: no window object; offline mode");
        return RawConfig::default();
    };
    let cfg = match js_sys::Reflect::get(&window, &JsValue::from_str("PLEDGE_CONFIG")) {
        Ok(v) if v.is_object() => v,
        Ok(_) => {
            tracing::info!("compile-api config: window.PLEDGE_CONFIG not set; offline mode");
            return RawConfig::default();
        }
        Err(_) => {
            tracing::warn!("compile-api config: reading window.PLEDGE_CONFIG threw; offline mode");
            return RawConfig::default();
        }
    };
    let field = |name: &str| -> Option<String> {
        let v = js_sys::Reflect::get(&cfg, &JsValue::from_str(name)).ok()?;
        if let Some(s) = v.as_string() {
            Some(s).filter(|s| !s.trim().is_empty())
        } else if let Some(b) = v.as_bool() {
            Some(b.to_string())
        } else {
            v.as_f64().map(|n| n.to_string())
        }
    };
    RawConfig {
        api_url: field("apiUrl"),
        api_token: field("apiToken"),
        real_compile: field("realCompile"),
        timeout_secs: field("timeoutSecs"),
    }
}

fn is_truthy(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn parse_target(raw: &RawConfig, default_url: Option<&str>) -> Result<ApiTarget, ConfigError> {
    let url = match raw.api_url.as_deref().or(default_url) {
        None => return Ok(ApiTarget::Offline),
        Some(u) if OFFLINE_WORDS.contains(&u.trim().to_ascii_lowercase().as_str()) => {
            return Ok(ApiTarget::Offline);
        }
        Some(u) => u,
    };
    let base_url = validate_base_url(url)?;
    let token = raw.api_token.as_deref().map(ApiToken::new).transpose()?;
    let mode = if raw.real_compile.as_deref().is_some_and(is_truthy) {
        CompileMode::Real
    } else {
        CompileMode::Pattern
    };
    let timeout = match raw.timeout_secs.as_deref() {
        None => mode.default_timeout(),
        Some(v) => parse_timeout(v)?,
    };
    Ok(ApiTarget::Remote(ApiEndpoint {
        base_url,
        token,
        mode,
        timeout,
    }))
}

fn parse_timeout(v: &str) -> Result<Duration, ConfigError> {
    match v.trim().parse::<u64>() {
        Ok(n) if (1..=MAX_TIMEOUT_SECS).contains(&n) => Ok(Duration::from_secs(n)),
        _ => Err(ConfigError::InvalidTimeout {
            value: v.to_string(),
        }),
    }
}

/// Accept `http://host[:port][/prefix]` or `https://...`; return it with
/// any trailing slash removed.
pub fn validate_base_url(raw: &str) -> Result<String, ConfigError> {
    let url = raw.trim();
    let bad = |reason| ConfigError::InvalidUrl {
        url: url.to_string(),
        reason,
    };
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(bad("contains whitespace or control characters"));
    }
    let lower = url.to_ascii_lowercase();
    let rest = if lower.starts_with("https://") {
        &url["https://".len()..]
    } else if lower.starts_with("http://") {
        &url["http://".len()..]
    } else {
        return Err(bad("must start with http:// or https://"));
    };
    if url.contains('?') || url.contains('#') {
        return Err(bad("must not contain a query string or fragment"));
    }
    let authority = rest.split('/').next().unwrap_or("");
    if authority.contains('@') {
        return Err(bad("must not embed credentials; use the token setting"));
    }
    // `[v6]:port` keeps the brackets; `host:port` drops the port.
    let host = if authority.starts_with('[') {
        authority.split_once(']').map_or("", |(h, _)| h)
    } else {
        authority.split(':').next().unwrap_or("")
    };
    if host.is_empty() || host == "[" {
        return Err(bad("missing host"));
    }
    Ok(url.trim_end_matches('/').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn raw(pairs: &[(&str, &str)]) -> RawConfig {
        let m: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        RawConfig::from_lookup(|k| m.get(k).cloned())
    }

    fn remote(cfg: &CompileApiConfig) -> &ApiEndpoint {
        match &cfg.target {
            ApiTarget::Remote(ep) => ep,
            other => panic!("expected remote, got {other:?}"),
        }
    }

    #[test]
    fn native_default_is_localhost_pattern() {
        let cfg = CompileApiConfig::from_raw(&raw(&[]), Some(DEFAULT_API_URL));
        let ep = remote(&cfg);
        assert_eq!(ep.compile_url(), "http://127.0.0.1:7878/compile");
        assert_eq!(ep.token, None);
        assert_eq!(ep.timeout, DEFAULT_PATTERN_TIMEOUT);
        assert!(!cfg.stub_is_canonical());
    }

    #[test]
    fn web_default_is_offline() {
        let cfg = CompileApiConfig::from_raw(&raw(&[]), None);
        assert_eq!(cfg.target, ApiTarget::Offline);
        assert!(cfg.stub_is_canonical());
    }

    #[test]
    fn offline_words_select_offline_mode() {
        for w in ["off", "OFFLINE", " none "] {
            let cfg =
                CompileApiConfig::from_raw(&raw(&[(vars::API_URL, w)]), Some(DEFAULT_API_URL));
            assert_eq!(cfg.target, ApiTarget::Offline, "{w}");
        }
    }

    #[test]
    fn full_config_parses() {
        let cfg = CompileApiConfig::from_raw(
            &raw(&[
                (vars::API_URL, "https://api.example.com/pnc/"),
                (vars::API_TOKEN, "  abc123  "),
                (vars::REAL_COMPILE, "TRUE"),
                (vars::TIMEOUT_SECS, "45"),
            ]),
            None,
        );
        let ep = remote(&cfg);
        assert_eq!(ep.compile_url(), "https://api.example.com/pnc/compile-real");
        assert_eq!(ep.token.as_ref().map(ApiToken::expose), Some("abc123"));
        assert_eq!(ep.mode, CompileMode::Real);
        assert_eq!(ep.timeout, Duration::from_secs(45));
    }

    #[test]
    fn real_mode_gets_longer_default_timeout() {
        let cfg =
            CompileApiConfig::from_raw(&raw(&[(vars::REAL_COMPILE, "1")]), Some(DEFAULT_API_URL));
        assert_eq!(remote(&cfg).timeout, DEFAULT_REAL_TIMEOUT);
        let cfg =
            CompileApiConfig::from_raw(&raw(&[(vars::REAL_COMPILE, "0")]), Some(DEFAULT_API_URL));
        assert_eq!(remote(&cfg).mode, CompileMode::Pattern);
    }

    #[test]
    fn url_validation() {
        for ok in [
            "http://127.0.0.1:7878",
            "https://api.example.com",
            "HTTPS://Example.com/",
            "http://[::1]:7878",
            "http://localhost",
        ] {
            assert!(validate_base_url(ok).is_ok(), "{ok}");
        }
        for bad in [
            "ftp://example.com",
            "example.com",
            "javascript:alert(1)",
            "http://",
            "http://:7878",
            "http://exa mple.com",
            "http://example.com/?x=1",
            "http://example.com/#frag",
            "http://user:pw@example.com",
            "file:///etc/passwd",
        ] {
            assert!(validate_base_url(bad).is_err(), "{bad}");
        }
        assert_eq!(
            validate_base_url("http://h:1/api///").as_deref(),
            Ok("http://h:1/api")
        );
    }

    #[test]
    fn invalid_inputs_become_misconfigured_not_offline() {
        for pairs in [
            vec![(vars::API_URL, "ftp://x")],
            vec![(vars::API_TOKEN, "has space")],
            vec![(vars::TIMEOUT_SECS, "0")],
            vec![(vars::TIMEOUT_SECS, "9999")],
            vec![(vars::TIMEOUT_SECS, "soon")],
        ] {
            let cfg = CompileApiConfig::from_raw(&raw(&pairs), Some(DEFAULT_API_URL));
            assert!(
                matches!(cfg.target, ApiTarget::Misconfigured(_)),
                "{pairs:?} -> {cfg:?}"
            );
            assert!(
                !cfg.stub_is_canonical(),
                "misconfig must not save stub passes"
            );
        }
    }

    #[test]
    fn token_is_redacted_everywhere() {
        let secret = "s3cr3t-token-value";
        let cfg =
            CompileApiConfig::from_raw(&raw(&[(vars::API_TOKEN, secret)]), Some(DEFAULT_API_URL));
        assert!(!format!("{cfg:?}").contains(secret));
        assert!(!cfg.summary().contains(secret));
        assert!(cfg.summary().contains("<redacted>"));
        let tok = ApiToken::new(secret).expect("valid token");
        assert_eq!(tok.to_string(), "<redacted>");
        // An invalid token's error does not echo it.
        let err = ApiToken::new("bad token here").expect_err("space rejected");
        assert!(!err.to_string().contains("bad token"));
    }

    #[test]
    fn empty_values_are_unset() {
        let cfg = CompileApiConfig::from_raw(
            &raw(&[(vars::API_URL, "   "), (vars::API_TOKEN, "")]),
            Some(DEFAULT_API_URL),
        );
        let ep = remote(&cfg);
        assert_eq!(ep.base_url, DEFAULT_API_URL);
        assert_eq!(ep.token, None);
    }
}
