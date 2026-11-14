# compile-api deployment

Files in this directory:

| File | Purpose |
|---|---|
| `compile-api.service` | Hardened systemd unit |
| `env.example` | Every `PNC_*` setting, with its default |
| `README.md` | These steps |

Target: a Linux VPS (Debian 12 or Ubuntu 24.04, x86_64) with systemd 252
or newer. Topology:
`internet --TLS--> Caddy (same host) --> 127.0.0.1:7878`. The unit denies
all IP traffic except loopback, so the server and its cargo children
cannot reach the internet.

Nothing here has been run against a real host yet. Treat the first
install as a verification pass and use the checks in step 8.

## 1. Packages and service user

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git caddy
sudo useradd --system --home-dir /var/lib/pnc-compile --shell /usr/sbin/nologin pnc-compile
```

## 2. Rust toolchain (root-owned, read-only to the service)

```bash
sudo mkdir -p /opt/rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sudo env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=/opt/rust/cargo \
  sh /tmp/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable
sudo env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=/opt/rust/cargo \
  /opt/rust/cargo/bin/rustup target add wasm32-wasip1
sudo chmod -R a+rX /opt/rust
```

`wasm32-wasip1` is required. Without it every `/compile-real` call fails
with "can't find crate for `std`".

## 3. Build and install the binary

On the VPS, from a checkout of the repo:

```bash
git clone <repo-url> ~/pledgeandcrown && cd ~/pledgeandcrown
env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=$HOME/.cargo \
  /opt/rust/cargo/bin/cargo build --release -p pledge_compile_api
sudo install -m 0755 target/release/pledge_compile_api /usr/local/bin/pledge_compile_api
sudo install -D -m 0644 compile-api/deploy/README.md /opt/pnc-compile/README.md
```

## 4. Configuration

```bash
sudo install -d -m 0750 -o root -g pnc-compile /etc/pnc-compile
sudo install -m 0640 -o root -g pnc-compile compile-api/deploy/env.example /etc/pnc-compile/env
openssl rand -hex 32        # put this in PNC_AUTH_TOKENS
sudoedit /etc/pnc-compile/env
```

Set at least:

- `PNC_AUTH_TOKENS=<the hex string>`
- `PNC_TRUST_PROXY=true`, because Caddy is the only client
- `PNC_CORS_ORIGINS=` your web origin(s), plus `tauri://localhost,http://tauri.localhost`
  if the mobile app calls the API

Leave `PNC_BIND=127.0.0.1:7878`. The server will not start if it is bound
to a public address with no token, and a startup config error is logged
and exits non-zero.

## 5. systemd unit

```bash
sudo install -m 0644 compile-api/deploy/compile-api.service /etc/systemd/system/compile-api.service
sudo systemd-analyze verify /etc/systemd/system/compile-api.service
sudo systemctl daemon-reload
sudo systemctl enable --now compile-api
sudo systemctl status compile-api
journalctl -u compile-api -n 50     # look for the redacted "config:" line
systemd-analyze security compile-api
```

## 6. Reverse proxy (TLS)

`/etc/caddy/Caddyfile`:

```caddyfile
api.example.com {
    reverse_proxy 127.0.0.1:7878
}
```

Caddy sets `X-Forwarded-For` to the real client address, and the
server reads the right-most entry. Then run `sudo systemctl reload caddy`.

## 7. Prometheus scrape (optional)

```yaml
scrape_configs:
  - job_name: pnc-compile
    scheme: https
    authorization:
      type: Bearer
      credentials: <a token from PNC_AUTH_TOKENS>
    static_configs:
      - targets: ["api.example.com"]
```

The series are `pnc_http_requests_total{route,status}`,
`pnc_rejections_total{reason=auth|rate|queue|body}`,
`pnc_builds_in_flight`, `pnc_builds_queued`,
`pnc_build_duration_seconds` and `pnc_run_duration_seconds`.

## 8. Smoke test

```bash
TOKEN=<token>
curl -s http://127.0.0.1:7878/health
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:7878/compile   # expect 401
curl -s -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"source":"fn main(){ println!(\"hi\"); }","encounter_id":"intro_let_binding"}' \
  http://127.0.0.1:7878/compile-real
# expect {"ok":true,"stdout":"[real] hi\n",...}
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7878/metrics | head
```

If `/compile-real` returns `ok:false` with a cargo or permission error,
check these against the hardening first:

- `CARGO_HOME` (`/var/lib/pnc-compile/cargo`) must be writable. The unit's
  `StateDirectory` creates it.
- If it fails with `SIGSYS` or `EPERM`, a syscall is being filtered.
  Comment out the `SystemCallFilter=~...` line, retest, and narrow it down.
- Do not enable `MemoryDenyWriteExecute`. It breaks the wasmtime JIT.

## Updating

Repeat step 3, then run `sudo systemctl restart compile-api`. To rotate
tokens, add the new token next to the old one in `PNC_AUTH_TOKENS`,
restart, move the clients over, remove the old token, and restart again.
