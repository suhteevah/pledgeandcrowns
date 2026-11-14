// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP hardening tests: bearer auth, loopback-bind refusal, per-IP rate
//! limits, build-queue backpressure, CORS, body limits and `/metrics`.
//!
//! `/compile-real` is exercised with an injected stub grader so the
//! concurrency path (semaphore + spawn_blocking + bounded queue) runs in
//! the default test suite without a wasm toolchain.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use pledge_compile_api::config::{ConfigError, vars};
use pledge_compile_api::wasm_builder::{RealOutcome, RunLimits};
use pledge_compile_api::{
    AppState, CompileResponse, Config, MAX_BODY_BYTES, RealGrader, RejectionBody,
    build_metrics_router, build_router, serve, serve_with,
};
use reqwest::StatusCode;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const TOKEN: &str = "test-token-0123456789abcdef0123456789";

/// Config for an in-process test server (loopback, stub grader). Tokens or
/// trust-proxy make a config "exposed", which refuses in-process builds;
/// the loopback-only dev override is what local dev uses for that shape.
fn config_with(pairs: &[(&str, &str)]) -> Config {
    let mut all = pairs.to_vec();
    if !pairs
        .iter()
        .any(|(k, _)| *k == vars::DEV_UNSAFE_LOCAL_BUILDS)
    {
        all.push((vars::DEV_UNSAFE_LOCAL_BUILDS, "true"));
    }
    try_config(&all).expect("test config must validate")
}

/// Parse as a local dev box: `PNC_BUILD_SANDBOX=none` unless set (the env
/// default is `bwrap` on Linux). No override added.
fn try_config(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
    let mut map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    map.entry(vars::BUILD_SANDBOX.to_string())
        .or_insert_with(|| "none".to_string());
    Config::from_lookup(|k| map.get(k).cloned())
}

/// Stub grader: sleeps `delay`, then reports a clean compile + run.
fn stub_grader(delay: Duration) -> RealGrader {
    Arc::new(move |_src: &str, _limits: &RunLimits| {
        std::thread::sleep(delay);
        Ok(RealOutcome {
            compiled: true,
            ran_ok: true,
            stdout: "stub ran".to_string(),
            stderr: String::new(),
            build_time: delay,
            run_time: Some(Duration::from_millis(1)),
        })
    })
}

async fn spawn(state: AppState) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_router(state);
    tokio::spawn(async move {
        serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    addr
}

async fn spawn_cfg(cfg: Config) -> SocketAddr {
    spawn(AppState::with_real_grader(
        cfg,
        stub_grader(Duration::from_millis(1)),
    ))
    .await
}

/// API listener + the separate metrics listener for the same state.
async fn spawn_with_metrics(cfg: Config) -> (SocketAddr, SocketAddr) {
    let state = AppState::with_real_grader(cfg, stub_grader(Duration::from_millis(1)));
    let metrics = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let maddr = metrics.local_addr().unwrap();
    let mrouter = build_metrics_router(state.clone());
    tokio::spawn(async move {
        serve(metrics, mrouter).await.unwrap();
    });
    (spawn(state).await, maddr)
}

fn compile_body() -> serde_json::Value {
    serde_json::json!({"source": "fn main() { let answer = 42; }", "encounter_id": "intro_let_binding"})
}

async fn post(
    addr: SocketAddr,
    path: &str,
    token: Option<&str>,
    extra: &[(&str, &str)],
) -> reqwest::Response {
    let mut req = reqwest::Client::new()
        .post(format!("http://{addr}{path}"))
        .json(&compile_body());
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    for (k, v) in extra {
        req = req.header(*k, *v);
    }
    req.send().await.expect("request failed")
}

// ---- auth ----------------------------------------------------------------

#[tokio::test]
async fn auth_missing_token_is_401() {
    let addr = spawn_cfg(config_with(&[(vars::AUTH_TOKENS, TOKEN)])).await;
    let resp = post(addr, "/compile", None, &[]).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.headers().get("www-authenticate").unwrap(),
        "Bearer",
        "401 must advertise the Bearer scheme"
    );
    let body: RejectionBody = resp.json().await.unwrap();
    assert!(!body.ok);
    assert_eq!(body.error, "unauthorized");
}

#[tokio::test]
async fn auth_wrong_token_is_401() {
    let addr = spawn_cfg(config_with(&[(vars::AUTH_TOKENS, TOKEN)])).await;
    let wrong = "test-token-0123456789abcdef0123456780";
    for path in ["/compile", "/compile-real"] {
        let resp = post(addr, path, Some(wrong), &[]).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "path {path}");
    }
}

#[tokio::test]
async fn auth_right_token_is_200_and_any_configured_token_works() {
    let second = "second-token-abcdefabcdefabcdefabcdef00";
    let addr = spawn_cfg(config_with(&[(
        vars::AUTH_TOKENS,
        &format!("{TOKEN},{second}"),
    )]))
    .await;
    for t in [TOKEN, second] {
        let resp = post(addr, "/compile", Some(t), &[]).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: CompileResponse = resp.json().await.unwrap();
        assert!(body.ok, "{body:?}");
    }
    let resp = post(addr, "/compile-real", Some(TOKEN), &[]).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_stays_open_with_auth_enabled() {
    let addr = spawn_cfg(config_with(&[(vars::AUTH_TOKENS, TOKEN)])).await;
    let resp = reqwest::get(format!("http://{addr}/health")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn no_token_configured_means_open_on_loopback() {
    let addr = spawn_cfg(config_with(&[])).await;
    let resp = post(addr, "/compile", None, &[]).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---- loopback-bind refusal ------------------------------------------------

#[test]
fn public_bind_without_token_is_refused() {
    for bind in ["0.0.0.0:7878", "203.0.113.5:7878", "[::]:7878"] {
        let err = try_config(&[(vars::BIND, bind)]).expect_err(bind);
        assert!(
            matches!(err, ConfigError::UnauthenticatedPublicBind { .. }),
            "{bind}: {err}"
        );
        assert!(err.to_string().contains("refusing to bind"));
    }
}

#[test]
fn loopback_bind_without_token_and_public_bind_with_token_are_allowed() {
    assert!(try_config(&[(vars::BIND, "127.0.0.1:9000")]).is_ok());
    assert!(try_config(&[(vars::BIND, "[::1]:9000")]).is_ok());
    // A public bind with a token must hand builds to the builder service.
    // Building in this process is refused (unsandboxed everywhere; at all
    // on Linux), and the dev override is refused on a public bind.
    let public = [(vars::BIND, "0.0.0.0:7878"), (vars::AUTH_TOKENS, TOKEN)];
    let r = try_config(&public);
    assert!(
        matches!(
            r,
            Err(ConfigError::UnsandboxedExposure { .. } | ConfigError::InProcessExposure { .. })
        ),
        "{r:?}"
    );
    let bwrap_in_process = [
        (vars::BIND, "0.0.0.0:7878"),
        (vars::AUTH_TOKENS, TOKEN),
        (vars::BUILD_SANDBOX, "bwrap"),
        (
            vars::TOOLCHAIN_DIR,
            "/opt/rust/rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu",
        ),
    ];
    assert!(try_config(&bwrap_in_process).is_err());
    let with_builder = [
        (vars::BIND, "0.0.0.0:7878"),
        (vars::AUTH_TOKENS, TOKEN),
        (vars::BUILDER_SOCKET, "/run/pnc-builder/builder.sock"),
    ];
    let overridden = [
        (vars::BIND, "0.0.0.0:7878"),
        (vars::AUTH_TOKENS, TOKEN),
        (vars::DEV_UNSAFE_LOCAL_BUILDS, "true"),
    ];
    assert!(matches!(
        try_config(&overridden),
        Err(ConfigError::DevOverrideOnPublicBind { .. })
    ));
    if cfg!(target_os = "linux") {
        assert!(try_config(&with_builder).is_ok());
    } else {
        assert!(matches!(
            try_config(&with_builder),
            Err(ConfigError::BuilderUnsupported)
        ));
    }
}

#[test]
fn trust_proxy_with_public_bind_is_refused_without_override() {
    let base = [
        (vars::BIND, "0.0.0.0:7878"),
        (vars::AUTH_TOKENS, TOKEN),
        (vars::TRUST_PROXY, "true"),
    ];
    assert!(matches!(
        try_config(&base),
        Err(ConfigError::TrustProxyPublicBind { .. })
    ));
}

// ---- rate limiting ----------------------------------------------------------

#[tokio::test]
async fn global_rate_limit_returns_429_with_retry_after() {
    let addr = spawn_cfg(config_with(&[(vars::RATE_LIMIT_PER_MIN, "2")])).await;
    for _ in 0..2 {
        assert_eq!(
            post(addr, "/compile", None, &[]).await.status(),
            StatusCode::OK
        );
    }
    let resp = post(addr, "/compile", None, &[]).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry: u64 = resp
        .headers()
        .get("retry-after")
        .expect("429 must carry Retry-After")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=60).contains(&retry), "retry-after={retry}");
    let body: RejectionBody = resp.json().await.unwrap();
    assert_eq!(body.error, "rate_limited");
    assert_eq!(body.retry_after_seconds, Some(retry));
}

#[tokio::test]
async fn compile_real_has_its_own_tighter_limit() {
    let addr = spawn_cfg(config_with(&[(vars::REAL_RATE_LIMIT_PER_MIN, "1")])).await;
    assert_eq!(
        post(addr, "/compile-real", None, &[]).await.status(),
        StatusCode::OK
    );
    assert_eq!(
        post(addr, "/compile-real", None, &[]).await.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    // The pattern route is unaffected by the /compile-real budget.
    assert_eq!(
        post(addr, "/compile", None, &[]).await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn x_forwarded_for_is_ignored_unless_trust_proxy() {
    // Without TRUST_PROXY a spoofed XFF must not mint a fresh bucket.
    let addr = spawn_cfg(config_with(&[(vars::RATE_LIMIT_PER_MIN, "1")])).await;
    let r1 = post(
        addr,
        "/compile",
        None,
        &[("x-forwarded-for", "198.51.100.1")],
    )
    .await;
    let r2 = post(
        addr,
        "/compile",
        None,
        &[("x-forwarded-for", "198.51.100.2")],
    )
    .await;
    assert_eq!(r1.status(), StatusCode::OK);
    assert_eq!(r2.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn x_forwarded_for_rightmost_entry_keys_the_limit_with_trust_proxy() {
    let addr = spawn_cfg(config_with(&[
        (vars::RATE_LIMIT_PER_MIN, "1"),
        (vars::TRUST_PROXY, "true"),
    ]))
    .await;
    let a = post(
        addr,
        "/compile",
        None,
        &[("x-forwarded-for", "198.51.100.1")],
    )
    .await;
    let b = post(
        addr,
        "/compile",
        None,
        &[("x-forwarded-for", "198.51.100.2")],
    )
    .await;
    assert_eq!(a.status(), StatusCode::OK);
    assert_eq!(
        b.status(),
        StatusCode::OK,
        "distinct clients, distinct buckets"
    );
    // A client-prepended entry must not dodge the limit: the proxy-appended
    // right-most entry is what counts.
    let c = post(
        addr,
        "/compile",
        None,
        &[("x-forwarded-for", "10.9.9.9, 198.51.100.1")],
    )
    .await;
    assert_eq!(c.status(), StatusCode::TOO_MANY_REQUESTS);
}

// ---- build queue ----------------------------------------------------------

#[tokio::test]
async fn full_build_queue_returns_503_with_retry_after() {
    let cfg = config_with(&[
        (vars::MAX_CONCURRENT_BUILDS, "1"),
        (vars::MAX_QUEUED_BUILDS, "0"),
    ]);
    let addr = spawn(AppState::with_real_grader(
        cfg,
        stub_grader(Duration::from_millis(800)),
    ))
    .await;

    let first = tokio::spawn(async move { post(addr, "/compile-real", None, &[]).await });
    tokio::time::sleep(Duration::from_millis(200)).await;
    let second = post(addr, "/compile-real", None, &[]).await;

    assert_eq!(second.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(second.headers().get("retry-after").is_some());
    let body: RejectionBody = second.json().await.unwrap();
    assert_eq!(body.error, "queue_full");

    let first = first.await.unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let body: CompileResponse = first.json().await.unwrap();
    assert!(body.ok);
    assert!(body.stdout.starts_with("[real] stub ran"), "{body:?}");
}

#[tokio::test]
async fn queued_build_waits_for_a_slot_instead_of_failing() {
    let cfg = config_with(&[
        (vars::MAX_CONCURRENT_BUILDS, "1"),
        (vars::MAX_QUEUED_BUILDS, "1"),
    ]);
    let addr = spawn(AppState::with_real_grader(
        cfg,
        stub_grader(Duration::from_millis(300)),
    ))
    .await;
    let a = tokio::spawn(async move { post(addr, "/compile-real", None, &[]).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let b = tokio::spawn(async move { post(addr, "/compile-real", None, &[]).await });
    assert_eq!(a.await.unwrap().status(), StatusCode::OK);
    assert_eq!(b.await.unwrap().status(), StatusCode::OK);
}

// ---- CORS ----------------------------------------------------------------

#[tokio::test]
async fn cors_allows_listed_origin_and_denies_others() {
    let addr = spawn_cfg(config_with(&[(vars::CORS_ORIGINS, "https://play.example")])).await;

    let allowed = post(
        addr,
        "/compile",
        None,
        &[("origin", "https://play.example")],
    )
    .await;
    assert_eq!(
        allowed
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap()),
        Some("https://play.example")
    );

    let denied = post(
        addr,
        "/compile",
        None,
        &[("origin", "https://evil.example")],
    )
    .await;
    assert!(
        denied
            .headers()
            .get("access-control-allow-origin")
            .is_none(),
        "unlisted origin must not get an allow-origin header"
    );

    // Preflight for an allowed origin advertises Authorization.
    let preflight = reqwest::Client::new()
        .request(
            reqwest::Method::OPTIONS,
            format!("http://{addr}/compile-real"),
        )
        .header("origin", "https://play.example")
        .header("access-control-request-method", "POST")
        .header(
            "access-control-request-headers",
            "authorization,content-type",
        )
        .send()
        .await
        .unwrap();
    assert!(preflight.status().is_success(), "{}", preflight.status());
    let allow_headers = preflight
        .headers()
        .get("access-control-allow-headers")
        .unwrap()
        .to_str()
        .unwrap()
        .to_lowercase();
    assert!(allow_headers.contains("authorization"), "{allow_headers}");
}

#[tokio::test]
async fn cors_default_is_localhost_dev_only() {
    let addr = spawn_cfg(config_with(&[])).await;
    let ok = post(
        addr,
        "/compile",
        None,
        &[("origin", "http://localhost:8080")],
    )
    .await;
    assert!(ok.headers().get("access-control-allow-origin").is_some());
    let no = post(addr, "/compile", None, &[("origin", "https://example.com")]).await;
    assert!(no.headers().get("access-control-allow-origin").is_none());
}

// ---- body limit ----------------------------------------------------------

#[tokio::test]
async fn oversized_body_rejected_before_buffering() {
    let addr = spawn_cfg(config_with(&[])).await;
    let big = "x".repeat(MAX_BODY_BYTES + 1);
    let body = serde_json::json!({"source": big, "encounter_id": "intro_let_binding"});
    for path in ["/compile", "/compile-real"] {
        let resp = reqwest::Client::new()
            .post(format!("http://{addr}{path}"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE, "{path}");
        let rb: RejectionBody = resp.json().await.unwrap();
        assert_eq!(rb.error, "body_too_large");
        assert!(rb.stderr.contains("exceeds limit"));
    }
}

#[tokio::test]
async fn chunked_oversized_body_is_capped_while_streaming() {
    // No Content-Length: the pre-buffer header check cannot fire, so the
    // streaming cap in `limit_body` has to stop it. Raw HTTP/1.1 over TCP
    // so the request is genuinely chunked.
    use tokio::io::AsyncReadExt;
    let addr = spawn_cfg(config_with(&[])).await;
    let payload = format!(
        "{{\"source\":\"{}\",\"encounter_id\":\"x\"}}",
        "y".repeat(MAX_BODY_BYTES * 2)
    );
    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    const CRLF: &str = "\r\n";
    let head = [
        "POST /compile HTTP/1.1".to_string(),
        format!("Host: {addr}"),
        "Content-Type: application/json".to_string(),
        "Transfer-Encoding: chunked".to_string(),
        "Connection: close".to_string(),
        String::new(),
        String::new(),
    ]
    .join(CRLF);
    let mut stream = stream;
    // Unpaced: send the entire oversized chunked body as fast as possible,
    // THEN read. The server drains (bounded) what is left of the body
    // after the cap before answering 413, so the response is not lost to
    // a TCP reset. (Before the drain fix this test had to pace its upload
    // and read concurrently to see the 413 on Windows.)
    wr_all(&mut stream, head.as_bytes()).await;
    for chunk in payload.as_bytes().chunks(8 * 1024) {
        wr_all(&mut stream, format!("{:x}{CRLF}", chunk.len()).as_bytes()).await;
        wr_all(&mut stream, chunk).await;
        wr_all(&mut stream, CRLF.as_bytes()).await;
    }
    wr_all(&mut stream, format!("0{CRLF}{CRLF}").as_bytes()).await;
    let mut resp = Vec::new();
    let _ = stream.read_to_end(&mut resp).await;
    let text = String::from_utf8_lossy(&resp);
    assert!(
        text.starts_with("HTTP/1.1 413"),
        "expected 413, got: {}",
        text.lines().next().unwrap_or("")
    );
    assert!(text.contains("body_too_large"), "{text}");
}

async fn wr_all(stream: &mut tokio::net::TcpStream, bytes: &[u8]) {
    use tokio::io::AsyncWriteExt;
    stream
        .write_all(bytes)
        .await
        .expect("server must keep reading (draining) the body, not reset the connection");
}

#[tokio::test]
async fn malformed_json_gets_a_json_error() {
    let addr = spawn_cfg(config_with(&[])).await;
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/compile"))
        .header("content-type", "application/json")
        .body("{not json")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_client_error());
    let rb: RejectionBody = resp.json().await.unwrap();
    assert_eq!(rb.error, "bad_request");
}

// ---- metrics ----------------------------------------------------------------

const METRICS_TOKEN: &str = "metrics-token-0123456789abcdef01234567";

#[tokio::test]
async fn metrics_endpoint_exposes_prometheus_text() {
    let (addr, maddr) = spawn_with_metrics(config_with(&[
        (vars::AUTH_TOKENS, TOKEN),
        (vars::METRICS_TOKENS, METRICS_TOKEN),
        (vars::REAL_RATE_LIMIT_PER_MIN, "1"),
    ]))
    .await;
    // Generate one of each interesting event.
    let _ = post(addr, "/compile", Some(TOKEN), &[]).await;
    let _ = post(addr, "/compile", None, &[]).await; // auth rejection
    let _ = post(addr, "/compile-real", Some(TOKEN), &[]).await; // build + run
    let _ = post(addr, "/compile-real", Some(TOKEN), &[]).await; // rate rejection

    let resp = reqwest::Client::new()
        .get(format!("http://{maddr}/metrics"))
        .bearer_auth(METRICS_TOKEN)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("text/plain; version=0.0.4"), "{ct}");
    let text = resp.text().await.unwrap();

    for needle in [
        "# TYPE pnc_http_requests_total counter",
        "pnc_http_requests_total{route=\"/compile\",status=\"200\"} 1",
        "pnc_http_requests_total{route=\"/compile\",status=\"401\"} 1",
        "pnc_http_requests_total{route=\"/compile-real\",status=\"429\"} 1",
        "# TYPE pnc_rejections_total counter",
        "pnc_rejections_total{reason=\"auth\"} 1",
        "pnc_rejections_total{reason=\"rate\"} 1",
        "pnc_rejections_total{reason=\"queue\"} 0",
        "pnc_rejections_total{reason=\"body\"} 0",
        "pnc_rejections_total{reason=\"source_policy\"} 0",
        "pnc_rejections_total{reason=\"input\"} 0",
        "pnc_rate_limit_fail_open_total 0",
        "pnc_rate_limit_tracked_clients{limiter=\"global\"} 1",
        "pnc_rate_limit_evictions_total{limiter=\"compile-real\"} 0",
        "# TYPE pnc_builds_in_flight gauge",
        "pnc_builds_in_flight 0",
        "pnc_builds_queued 0",
        "# TYPE pnc_build_duration_seconds histogram",
        "pnc_build_duration_seconds_count 1",
        "pnc_build_duration_seconds_bucket{le=\"+Inf\"} 1",
        "# TYPE pnc_run_duration_seconds histogram",
        "pnc_run_duration_seconds_count 1",
    ] {
        assert!(text.contains(needle), "missing `{needle}` in:\n{text}");
    }
    // Every non-comment line is `name{labels} value` or `name value`.
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let (_, value) = line.rsplit_once(' ').expect(line);
        assert!(value.parse::<f64>().is_ok(), "bad sample line: {line}");
    }
}

async fn get_metrics(
    addr: SocketAddr,
    token: Option<&str>,
    extra: &[(&str, &str)],
) -> reqwest::Response {
    let mut req = reqwest::Client::new().get(format!("http://{addr}/metrics"));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    for (k, v) in extra {
        req = req.header(*k, *v);
    }
    req.send().await.unwrap()
}

#[tokio::test]
async fn metrics_does_not_accept_the_player_token() {
    let (addr, maddr) = spawn_with_metrics(config_with(&[
        (vars::AUTH_TOKENS, TOKEN),
        (vars::METRICS_TOKENS, METRICS_TOKEN),
    ]))
    .await;
    assert_eq!(
        get_metrics(maddr, Some(TOKEN), &[]).await.status(),
        StatusCode::UNAUTHORIZED,
        "player token must not open /metrics"
    );
    assert_eq!(
        get_metrics(maddr, None, &[]).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_metrics(maddr, Some(METRICS_TOKEN), &[]).await.status(),
        StatusCode::OK
    );
    // And the metrics token does not open the player routes.
    assert_eq!(
        post(addr, "/compile", Some(METRICS_TOKEN), &[])
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

/// Re-review P2: `/metrics` must not be reachable through the reverse
/// proxy. It is not mounted on the API router at all, whatever the
/// token or proxy settings.
#[tokio::test]
async fn metrics_is_not_served_on_the_api_listener() {
    for cfg in [
        config_with(&[]),
        config_with(&[(vars::TRUST_PROXY, "true")]),
        config_with(&[
            (vars::AUTH_TOKENS, TOKEN),
            (vars::METRICS_TOKENS, METRICS_TOKEN),
        ]),
    ] {
        let (addr, maddr) = spawn_with_metrics(cfg).await;
        for token in [None, Some(TOKEN), Some(METRICS_TOKEN)] {
            let resp = get_metrics(addr, token, &[("x-forwarded-for", "203.0.113.9")]).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND, "token {token:?}");
            let resp = get_metrics(addr, token, &[]).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND, "token {token:?}");
        }
        // The metrics listener serves nothing but /metrics.
        let resp = reqwest::get(format!("http://{maddr}/health"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn metrics_without_token_is_direct_loopback_only() {
    // A request that carries the proxy's X-Forwarded-For is refused even
    // on the loopback metrics listener (defence in depth: the proxy
    // never forwards there anyway).
    let (_addr, maddr) = spawn_with_metrics(config_with(&[
        (vars::AUTH_TOKENS, TOKEN),
        (vars::TRUST_PROXY, "true"),
    ]))
    .await;
    assert_eq!(
        get_metrics(maddr, None, &[]).await.status(),
        StatusCode::OK,
        "direct loopback scrape"
    );
    let proxied = get_metrics(maddr, Some(TOKEN), &[("x-forwarded-for", "203.0.113.9")]).await;
    assert_eq!(proxied.status(), StatusCode::FORBIDDEN);
    let body: RejectionBody = proxied.json().await.unwrap();
    assert_eq!(body.error, "forbidden");
}

#[tokio::test]
async fn health_is_never_rate_limited() {
    let addr = spawn_cfg(config_with(&[(vars::RATE_LIMIT_PER_MIN, "1")])).await;
    assert_eq!(
        post(addr, "/compile", None, &[]).await.status(),
        StatusCode::OK
    );
    assert_eq!(
        post(addr, "/compile", None, &[]).await.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    for _ in 0..20 {
        let resp = reqwest::get(format!("http://{addr}/health")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn full_limiter_table_fails_open_for_new_clients_and_counts_it() {
    let (addr, maddr) = spawn_with_metrics(config_with(&[
        (vars::RATE_LIMIT_PER_MIN, "1"),
        (vars::RATE_LIMIT_MAX_CLIENTS, "2"),
        (vars::TRUST_PROXY, "true"),
    ]))
    .await;
    let from = |ip: &'static str| [("x-forwarded-for", ip)];
    for ip in ["198.51.100.1", "198.51.100.2"] {
        assert_eq!(
            post(addr, "/compile", None, &from(ip)).await.status(),
            StatusCode::OK
        );
    }
    // Table is full of drained buckets: a third client is NOT refused
    // (the old behaviour denied every new client, /health included).
    assert_eq!(
        post(addr, "/compile", None, &from("198.51.100.3"))
            .await
            .status(),
        StatusCode::OK
    );
    // Tracked clients are still limited.
    assert_eq!(
        post(addr, "/compile", None, &from("198.51.100.1"))
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        reqwest::get(format!("http://{addr}/health"))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    // The third client above was let through untracked: counted.
    let text = get_metrics(maddr, None, &[]).await.text().await.unwrap();
    let fail_opens: u64 = text
        .lines()
        .find_map(|l| l.strip_prefix("pnc_rate_limit_fail_open_total "))
        .and_then(|v| v.parse().ok())
        .expect("fail-open counter present");
    assert!(fail_opens >= 1, "{text}");
}

#[tokio::test]
async fn deny_listed_source_is_refused_before_build_and_counted() {
    let (addr, maddr) = spawn_with_metrics(config_with(&[])).await;
    let send = |source: &'static str| async move {
        reqwest::Client::new()
            .post(format!("http://{addr}/compile-real"))
            .json(&serde_json::json!({"source": source, "encounter_id": "probe"}))
            .send()
            .await
            .unwrap()
    };
    for hostile in [
        "fn main() { print!(\"{}\", include_str!(\"/etc/hostname\")); }",
        "#[path = \"/etc/hostname\"] mod m; fn main() {}",
        "use std::include_bytes as b; fn main() { b!(\"x\"); }",
        "#![feature(never_type)] fn main() {}",
    ] {
        let resp = send(hostile).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{hostile}");
        let body: RejectionBody = resp.json().await.unwrap();
        assert_eq!(body.error, "source_policy");
        assert!(!body.ok);
        assert!(
            body.stderr.starts_with("this server does not allow"),
            "{}",
            body.stderr
        );
    }
    // Lessons may mention the names in comments and strings: allowed
    // (reaches the stub grader).
    let benign = send(
        "// include_str! reads a file at compile time\n\
         fn main() { println!(\"#[path] and include_bytes! are disabled\"); }",
    )
    .await;
    assert_eq!(benign.status(), StatusCode::OK);
    let text = get_metrics(maddr, None, &[]).await.text().await.unwrap();
    assert!(
        text.contains("pnc_rejections_total{reason=\"source_policy\"} 4"),
        "{text}"
    );
}

#[tokio::test]
async fn deny_list_can_be_disabled_only_explicitly() {
    // Never on an exposed config (tokens), even with the dev override's
    // absence spelled out.
    assert!(matches!(
        try_config(&[(vars::SOURCE_DENYLIST, "false"), (vars::AUTH_TOKENS, TOKEN)]),
        Err(ConfigError::DenylistOffExposed { .. })
    ));
    let addr = spawn_cfg(config_with(&[(vars::SOURCE_DENYLIST, "false")])).await;
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/compile-real"))
        .json(&serde_json::json!({
            "source": "fn main() { include_str!(\"x\"); }",
            "encounter_id": "probe"
        }))
        .send()
        .await
        .unwrap();
    // Reaches the (stub) grader: the server-side check is off.
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn encounter_id_is_capped_and_charset_restricted() {
    let addr = spawn_cfg(config_with(&[])).await;
    let send = |id: String| async move {
        reqwest::Client::new()
            .post(format!("http://{addr}/compile"))
            .json(&serde_json::json!({"source": "fn main() {}", "encounter_id": id}))
            .send()
            .await
            .unwrap()
    };
    for bad in [
        String::new(),
        "a".repeat(65),
        "../etc/passwd".to_string(),
        "id\nINJECTED log line".to_string(),
        "has space".to_string(),
        "ünïcode".to_string(),
    ] {
        let resp = send(bad.clone()).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{bad:?}");
        let body: RejectionBody = resp.json().await.unwrap();
        assert!(body.stderr.contains("invalid encounter_id"), "{body:?}");
    }
    for good in ["intro_let_binding", "act-3_mission-2", &"a".repeat(64)] {
        assert_eq!(
            send(good.to_string()).await.status(),
            StatusCode::OK,
            "{good}"
        );
    }
}

#[tokio::test]
async fn metrics_route_label_never_contains_raw_paths() {
    let (addr, maddr) = spawn_with_metrics(config_with(&[])).await;
    let _ = reqwest::get(format!("http://{addr}/some/random/scanner/path"))
        .await
        .unwrap();
    let text = reqwest::get(format!("http://{maddr}/metrics"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!text.contains("scanner"), "{text}");
    assert!(
        text.contains("route=\"unmatched\",status=\"404\""),
        "{text}"
    );
}

// ---- slow clients (re-review P2) -------------------------------------------

/// Serve `state` with an explicit header-read timeout.
async fn spawn_with_header_timeout(state: AppState, header: Duration) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_router(state);
    tokio::spawn(async move {
        serve_with(listener, app, header).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    addr
}

/// Read until EOF or `limit`; returns (bytes, saw_eof).
async fn read_until_close(stream: &mut tokio::net::TcpStream, limit: Duration) -> (Vec<u8>, bool) {
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    let res = tokio::time::timeout(limit, async {
        loop {
            match stream.read(&mut buf).await {
                Ok(0) | Err(_) => return,
                Ok(n) => out.extend_from_slice(&buf[..n]),
            }
        }
    })
    .await;
    (out, res.is_ok())
}

#[tokio::test]
async fn slow_headers_are_cut_off_by_the_header_read_timeout() {
    let state = AppState::with_real_grader(config_with(&[]), stub_grader(Duration::ZERO));
    let addr = spawn_with_header_timeout(state, Duration::from_secs(1)).await;
    let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
    // Start a request and never finish the header block.
    s.write_all(b"POST /compile HTTP/1.1\r\nHost: x\r\n")
        .await
        .unwrap();
    let started = std::time::Instant::now();
    let (_bytes, closed) = read_until_close(&mut s, Duration::from_secs(5)).await;
    assert!(
        closed,
        "server must drop a connection whose headers never end"
    );
    let waited = started.elapsed();
    assert!(
        waited >= Duration::from_millis(800) && waited < Duration::from_secs(4),
        "closed after {waited:?}"
    );
}

#[tokio::test]
async fn slow_body_upload_gets_408_within_the_body_timeout() {
    let mut cfg = config_with(&[]);
    cfg.body_read_timeout = Duration::from_secs(1);
    let addr = spawn_cfg(cfg).await;
    let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
    let head = "POST /compile HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\n\
                Content-Length: 1000\r\n\r\n{\"source\":\"";
    s.write_all(head.as_bytes()).await.unwrap();
    // ...then stall with 960+ bytes still owed.
    let started = std::time::Instant::now();
    let (bytes, _) = read_until_close(&mut s, Duration::from_secs(5)).await;
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.starts_with("HTTP/1.1 408"), "{text}");
    assert!(text.contains("request_timeout"), "{text}");
    assert!(
        text.to_ascii_lowercase().contains("connection: close"),
        "{text}"
    );
    assert!(started.elapsed() < Duration::from_secs(4));
}

#[tokio::test]
async fn whole_request_deadline_answers_503_but_keeps_the_slot_until_work_ends() {
    let mut cfg = config_with(&[
        (vars::MAX_CONCURRENT_BUILDS, "1"),
        (vars::MAX_QUEUED_BUILDS, "0"),
    ]);
    cfg.request_deadline = Duration::from_millis(500);
    let state = AppState::with_real_grader(cfg, stub_grader(Duration::from_secs(2)));
    let probe = state.clone();
    let addr = spawn(state).await;
    let resp = post(addr, "/compile-real", None, &[]).await;
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body: RejectionBody = resp.json().await.unwrap();
    assert_eq!(body.error, "deadline_exceeded");
    // The stub build is still running: its slot is still held.
    assert_eq!(probe.available_build_slots(), 0);
    tokio::time::sleep(Duration::from_millis(2_000)).await;
    assert_eq!(
        probe.available_build_slots(),
        1,
        "slot back once work ended"
    );
}
