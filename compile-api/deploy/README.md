# compile-api deployment

Files in this directory:

| File | Purpose |
|---|---|
| `compile-api.service` | Hardened systemd unit |
| `env.example` | Every `PNC_*` setting, with its default |
| `verify-sandbox.sh` | Operator check: proves the build sandbox blocks file reads |
| `README.md` | These steps |

Target: a Linux VPS (Debian 12 or Ubuntu 24.04, x86_64) with systemd 252
or newer. Topology:
`internet --TLS--> Caddy (same host) --> 127.0.0.1:7878`. The unit denies
all IP traffic except loopback, so the server and its cargo children
cannot reach the internet.

Nothing here has been run against a real host yet. Treat the first
install as a verification pass: the server's startup self-test (step 5)
and `verify-sandbox.sh` (step 9) are the proof. **Do not expose the
service until step 9 passes.**

## Why the build sandbox is mandatory

rustc evaluates `include_str!`, `include_bytes!` and `#[path]` with the
filesystem view of the process running it. Without a sandbox, player
source can read any file the service user can (the env file with the
bearer tokens, `/proc/<server pid>/environ`) and get it back in the wasm
output or in compiler diagnostics. With `PNC_BUILD_SANDBOX=bwrap` each
build runs in bubblewrap with only `/usr`, the toolchain and its own job
dir visible, a fresh `/proc` in a new PID namespace, no network, a cleared
environment, and per-build rlimits from `prlimit`. A token-level deny-list
in the server (`source_policy.rs`) is a second layer only.

## 1. Packages and service user

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git caddy bubblewrap util-linux
sudo useradd --system --home-dir /var/lib/pnc-compile --shell /usr/sbin/nologin pnc-compile
```

`bubblewrap` provides `/usr/bin/bwrap`; `util-linux` provides
`/usr/bin/prlimit` (normally already installed).

Unprivileged user namespaces must be available to bwrap:

- Debian 12: enabled by default (`sysctl kernel.unprivileged_userns_clone`
  should print `1`).
- Ubuntu 24.04: AppArmor restricts them for unconfined programs; the
  `bubblewrap` package ships the `bwrap-userns-restrict` profile that
  allows `/usr/bin/bwrap`. Check with `sudo aa-status | grep bwrap`.

Quick manual check (as any user):
`bwrap --unshare-all --ro-bind /usr /usr --symlink usr/lib64 /lib64 --symlink usr/lib /lib --proc /proc --dev /dev /usr/bin/true && echo ok`

## 2. Rust toolchain (root-owned, read-only to the service)

```bash
sudo mkdir -p /opt/rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sudo env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=/opt/rust/cargo \
  sh /tmp/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable
sudo env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=/opt/rust/cargo \
  /opt/rust/cargo/bin/rustup target add wasm32-wasip1
sudo chmod -R a+rX /opt/rust
ls /opt/rust/rustup/toolchains/    # note the exact toolchain dir name
```

`wasm32-wasip1` is required. Without it every `/compile-real` call fails
with "can't find crate for `std`". `PNC_TOOLCHAIN_DIR` must point at the
toolchain dir itself (it contains `bin/cargo` and `bin/rustc`), e.g.
`/opt/rust/rustup/toolchains/stable-x86_64-unknown-linux-gnu`. The sandbox
runs that `cargo` directly, not the rustup proxy.

## 3. Build and install the binary

On the VPS, from a checkout of the repo:

```bash
git clone <repo-url> ~/pledgeandcrown && cd ~/pledgeandcrown
env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=$HOME/.cargo \
  /opt/rust/cargo/bin/cargo build --release -p pledge_compile_api
sudo install -m 0755 target/release/pledge_compile_api /usr/local/bin/pledge_compile_api
sudo install -D -m 0644 compile-api/deploy/README.md /opt/pnc-compile/README.md
sudo install -D -m 0755 compile-api/deploy/verify-sandbox.sh /opt/pnc-compile/verify-sandbox.sh
```

## 4. Configuration

The env file holds the bearer tokens. systemd reads it as root before
dropping to `pnc-compile`, so it is **root-only**: the service user (and
therefore anything a player build could ever run as) cannot read it.

```bash
sudo install -d -m 0700 -o root -g root /etc/pnc-compile
sudo install -m 0600 -o root -g root compile-api/deploy/env.example /etc/pnc-compile/env
openssl rand -hex 32        # put this in PNC_AUTH_TOKENS
openssl rand -hex 32        # optional: PNC_METRICS_TOKENS (different value)
sudoedit /etc/pnc-compile/env
# verify: must print "600 root root" and the service user must be refused
stat -c '%a %U %G' /etc/pnc-compile/env
sudo -u pnc-compile cat /etc/pnc-compile/env && echo "BAD: readable" || echo "ok: not readable"
```

Set at least:

- `PNC_AUTH_TOKENS=<the hex string>`
- `PNC_TRUST_PROXY=true`, because Caddy is the only client
- `PNC_BUILD_SANDBOX=bwrap` and `PNC_TOOLCHAIN_DIR=<toolchain dir from step 2>`
- `PNC_CORS_ORIGINS=` your web origin(s), plus `tauri://localhost,http://tauri.localhost`
  if the mobile app calls the API

Leave `PNC_BIND=127.0.0.1:7878`. The server refuses to start (config error
logged, non-zero exit) if it is bound publicly without a token, if
`PNC_TRUST_PROXY` is combined with a public bind, or on Linux if
`PNC_BUILD_SANDBOX=none` is combined with a public bind or with
`PNC_TRUST_PROXY=true`.

## 5. systemd unit

```bash
sudo install -m 0644 compile-api/deploy/compile-api.service /etc/systemd/system/compile-api.service
sudo systemd-analyze verify /etc/systemd/system/compile-api.service
sudo systemctl daemon-reload
sudo systemctl enable --now compile-api
sudo systemctl status compile-api
journalctl -u compile-api -n 80
systemd-analyze security compile-api
```

In the journal look for, in order: the redacted `config:` line,
`sandbox probe ok` for prlimit and bwrap, `sandbox probe: isolation checks
passed`, `build self-test passed`, then `listening`. With
`PNC_BUILD_SANDBOX=bwrap` the server **exits** if any probe fails, and the
log names the failing step and bwrap's stderr:

- `Creating new namespace failed` / `Operation not permitted` on user
  namespaces: see step 1 (sysctl / AppArmor), and that the unit's
  `RestrictNamespaces=` line is intact.
- `Can't mount proc`: something re-enabled `ProtectProc=`, `ProcSubset=`,
  `ProtectKernelTunables=` or `ProtectKernelLogs=` (see the unit comment).
- `SIGSYS` / `EPERM` on another syscall: temporarily comment out the
  `SystemCallFilter=~...` line, retest, and narrow it down.
- `loopback: Failed RTM_NEWADDR`: `AF_NETLINK` missing from
  `RestrictAddressFamilies=`.
- the build self-test fails with an allocation error: raise
  `PNC_BUILD_ADDRESS_SPACE_MIB` (max 1408).

## 6. Reverse proxy (TLS)

`/etc/caddy/Caddyfile`:

```caddyfile
api.example.com {
    # /metrics is for the local Prometheus only.
    respond /metrics 404
    reverse_proxy 127.0.0.1:7878
}
```

Caddy sets `X-Forwarded-For` to the real client address, and the
server reads the right-most entry. Then run `sudo systemctl reload caddy`.

## 7. Prometheus scrape (optional)

Scrape from the same host, directly (not through Caddy). Without
`PNC_METRICS_TOKENS`, `/metrics` only answers direct loopback clients;
with it, send that token (player tokens are refused on `/metrics`).

```yaml
scrape_configs:
  - job_name: pnc-compile
    static_configs:
      - targets: ["127.0.0.1:7878"]
    # if PNC_METRICS_TOKENS is set:
    # authorization: { type: Bearer, credentials: <a PNC_METRICS_TOKENS value> }
```

The series are `pnc_http_requests_total{route,status}`,
`pnc_rejections_total{reason=auth|rate|queue|body|source_policy|input}`,
`pnc_rate_limit_fail_open_total`, `pnc_rate_limit_tracked_clients{limiter}`,
`pnc_rate_limit_evictions_total{limiter}`, `pnc_builds_in_flight`,
`pnc_builds_queued`, `pnc_build_duration_seconds` and
`pnc_run_duration_seconds`.

## 8. Smoke test

```bash
TOKEN=<token>
curl -s http://127.0.0.1:7878/health
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:7878/compile   # expect 401
curl -s -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"source":"fn main(){ println!(\"hi\"); }","encounter_id":"intro_let_binding"}' \
  http://127.0.0.1:7878/compile-real
# expect {"ok":true,"stdout":"[real] hi\n",...}
curl -s http://127.0.0.1:7878/metrics | head
```

## 9. Prove the sandbox (required before going public)

```bash
sudo /opt/pnc-compile/verify-sandbox.sh
```

It plants a random canary in the env file, restarts the service with the
second-layer deny-list temporarily off, submits probe programs that try to
read the env file, the server's `/proc/<pid>/environ` and `/etc` at
compile time, and fails if the canary or the bearer token appears in any
response. It restores the env file and restarts the service on exit
(including on failure). It must print `PASS`.

## Updating

Repeat step 3, then run `sudo systemctl restart compile-api` and re-run
step 9. To rotate tokens, add the new token next to the old one in
`PNC_AUTH_TOKENS`, restart, move the clients over, remove the old token,
and restart again.
