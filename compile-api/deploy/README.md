# compile-api deployment runbook

| File | Purpose |
|---|---|
| `pnc-compile-api.service` | HTTP API unit. Holds the tokens, full hardening, never builds player code |
| `pnc-builder.service` | Builder unit. No secrets, the settings bubblewrap needs, builds AND runs player code |
| `env.example` | API settings, including the tokens -> `/etc/pnc-compile/env` (0600 root:root) |
| `builder.env.example` | Builder settings, nothing secret -> `/etc/pnc-builder/builder.env` (0644) |
| `verify-sandbox.sh` | Operator proof: positive-evidence sandbox probes, config refusals, exact HTTP checks |

**Status: nothing in this directory has been run on a Linux host yet.** The
first install is a verification pass. Both services run fail-closed
self-tests at startup, and `verify-sandbox.sh` must print `PASS`. **Do not
expose the service until step 9 passes.**

## How it fits together

```
internet --TLS--> Caddy --> 127.0.0.1:7878  pnc-compile-api.service  (user pnc-api, reads /etc/pnc-compile/env)
Prometheus -------------> 127.0.0.1:9878  same process, /metrics only, never proxied
pnc-compile-api --unix socket /run/pnc-builder/builder.sock--> pnc-builder.service (user pnc-builder, no secrets)
pnc-builder --per request--> `pledge_compile_api job` --> prlimit -> bwrap -> cargo/rustc ; then wasmtime run
```

- rustc evaluates `include_str!`, `include_bytes!` and `#[path]` with the
  filesystem view of the process running it, so player builds run in
  bubblewrap and see only `/usr`, the toolchain and their own job dir: no
  `/etc`, no `/proc`, no network, a cleared environment, per-build rlimits.
- The builder holds no secret, so a rustc or wasmtime escape lands in a
  process with nothing worth stealing. The API holds the tokens and keeps
  full systemd hardening because it never builds or runs player code.
- Each job is its own process, killed with its whole process tree at a
  50 s deadline. If a job's processes cannot be killed, the builder exits
  (status 70) and systemd kills its whole cgroup and restarts it.
- The token-level source deny-list is a second layer, applied by the API
  and again by the builder.

## 1. Packages and kernel prerequisites

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git caddy bubblewrap util-linux
```

`bubblewrap` provides `/usr/bin/bwrap`, `util-linux` provides
`/usr/bin/prlimit`. Unprivileged user namespaces must work for bwrap:

- Debian 12: `sysctl kernel.unprivileged_userns_clone` must print `1`.
- Ubuntu 24.04: `sysctl kernel.apparmor_restrict_unprivileged_userns`
  prints `1` (restricted) by default. bwrap then needs its AppArmor
  profile: `sudo aa-status | grep -i bwrap` must list it (the
  `bwrap-userns-restrict` profile from the `apparmor` package). If it is
  missing, load it (`sudo apparmor_parser -r /etc/apparmor.d/bwrap-userns-restrict`)
  rather than setting the sysctl to `0` host-wide.

Manual check, as the builder user once it exists (step 2):

```bash
sudo -u pnc-builder bwrap --unshare-all --die-with-parent --ro-bind /usr /usr \
  --symlink usr/lib64 /lib64 --symlink usr/lib /lib --dev /dev /usr/bin/true && echo ok
```

Expected: `ok`. `setting up uid map: Permission denied` means AppArmor;
`Creating new namespace failed` means the sysctl.

## 2. Users and groups

```bash
sudo groupadd --system pnc-ipc
sudo useradd --system --no-create-home --shell /usr/sbin/nologin pnc-api
sudo useradd --system --no-create-home --shell /usr/sbin/nologin pnc-builder
```

The units do the rest: the builder runs with primary group `pnc-ipc`, so
`/run/pnc-builder` is `pnc-builder:pnc-ipc 0750` and the socket `0660`; the
API unit gets `SupplementaryGroups=pnc-ipc`. Nobody else can connect.

## 3. Build and install the binary

```bash
sudo mkdir -p /opt/rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sudo env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=/opt/rust/cargo \
  sh /tmp/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable
git clone <repo-url> ~/pledgeandcrown && cd ~/pledgeandcrown
env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=$HOME/.cargo \
  /opt/rust/cargo/bin/cargo build --release --locked -p pledge_compile_api
sudo install -m 0755 target/release/pledge_compile_api /usr/local/bin/pledge_compile_api
sudo install -D -m 0644 compile-api/deploy/README.md /opt/pnc-compile/README.md
sudo install -D -m 0755 compile-api/deploy/verify-sandbox.sh /opt/pnc-compile/verify-sandbox.sh
```

## 4. The pinned player toolchain

The boss dialogue quotes real rustc diagnostics, so player code is built
with exactly one rustc. The version lives in one place,
`src/toolchain.rs` (`PINNED_RUSTC_VERSION`), and the binary prints it:

```bash
TC="$(pledge_compile_api toolchain)"      # e.g. 1.99.0
sudo env RUSTUP_HOME=/opt/rust/rustup CARGO_HOME=/opt/rust/cargo \
  /opt/rust/cargo/bin/rustup toolchain install "$TC" --profile minimal --target wasm32-wasip1
sudo chmod -R a+rX /opt/rust
ls -d /opt/rust/rustup/toolchains/"$TC"-*     # this is PNC_TOOLCHAIN_DIR
```

The builder runs `<PNC_TOOLCHAIN_DIR>/bin/rustc --version` at startup and
refuses to start on any other version. `wasm32-wasip1` is required.

## 5. Configuration files

```bash
# API: the ONLY file with secrets. root-only; systemd reads it as root.
sudo install -d -m 0700 -o root -g root /etc/pnc-compile
sudo install -m 0600 -o root -g root compile-api/deploy/env.example /etc/pnc-compile/env
openssl rand -hex 32        # -> PNC_AUTH_TOKENS
openssl rand -hex 32        # -> PNC_METRICS_TOKENS (optional, different value)
sudoedit /etc/pnc-compile/env
stat -c '%a %U %G' /etc/pnc-compile/env          # must print: 600 root root
sudo -u pnc-api cat /etc/pnc-compile/env && echo BAD || echo "ok: pnc-api cannot read it"
sudo -u pnc-builder cat /etc/pnc-compile/env && echo BAD || echo "ok: pnc-builder cannot read it"

# Builder: nothing secret (it refuses to start if a token variable is set).
sudo install -d -m 0755 -o root -g root /etc/pnc-builder
sudo install -m 0644 -o root -g root compile-api/deploy/builder.env.example /etc/pnc-builder/builder.env
sudoedit /etc/pnc-builder/builder.env            # set PNC_TOOLCHAIN_DIR from step 4
```

In `/etc/pnc-compile/env` set at least `PNC_AUTH_TOKENS`,
`PNC_TRUST_PROXY=true` (Caddy is the only client) and `PNC_CORS_ORIGINS`.
Leave `PNC_BIND=127.0.0.1:7878` and `PNC_METRICS_BIND=127.0.0.1:9878`. The
unit sets `PNC_BUILDER_SOCKET`; with it unset the API refuses to start
whenever a token is configured.

## 6. Install and check both units

```bash
sudo install -m 0644 compile-api/deploy/pnc-builder.service /etc/systemd/system/
sudo install -m 0644 compile-api/deploy/pnc-compile-api.service /etc/systemd/system/
sudo systemd-analyze verify /etc/systemd/system/pnc-builder.service /etc/systemd/system/pnc-compile-api.service
```

The builder's syscall filter is an explicit allow composition
(`@system-service @mount`) whose deny line subtracts no group bwrap needs
(the unit comment lists each syscall and its group). Confirm on this host:

```bash
systemd-analyze syscall-filter @system-service | grep -wE 'capset|chroot|setgroups|setuid|setresuid|unshare|clone3|prctl|prlimit64'
systemd-analyze syscall-filter @mount | grep -wE 'mount|umount2|pivot_root'
```

Expected: every listed name appears. Then:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now pnc-builder
journalctl -u pnc-builder -n 80 --no-pager
sudo systemctl enable --now pnc-compile-api
journalctl -u pnc-compile-api -n 40 --no-pager
systemd-analyze security pnc-compile-api.service
systemd-analyze security pnc-builder.service
```

`systemd-analyze security` should rate the API unit clearly better than the
builder (the builder needs user namespaces, `@mount` and netlink; the API
needs none of them).

## 7. What the journal must show

Builder, in this order:

1. `builder: config BuilderConfig { ... }`
2. `toolchain: rustc version rustc=rustc 1.99.0 (...)` and
   `toolchain: matches the pinned version`
3. `sandbox probe ok: prlimit ...` and `sandbox probe ok: bubblewrap ...`
4. `sandbox probe: isolation checks passed`
5. `build self-test passed (job process: build + run)`
6. `builder: listening socket=/run/pnc-builder/builder.sock`

API: `config: Config { ... auth_tokens: <redacted; N configured> ... }`,
`executor: builder service (this process never builds or runs player
code)`, `listening on http://127.0.0.1:7878`, `metrics listening on
http://127.0.0.1:9878/metrics (loopback only)`.

If the builder exits instead, the log names the failing step:

| Symptom | Cause / fix |
|---|---|
| `Creating new namespace failed`, `setting up uid map: Permission denied` | step 1 (sysctl / AppArmor), or `RestrictNamespaces=` edited |
| `Can't read /proc/sys/kernel/overflowuid` | someone added `ProcSubset=pid` to the builder unit; remove it |
| `Can't mount proc` | `PNC_BWRAP_PROC=fresh` with `ProtectProc=`/`ProtectKernelTunables=`/`ProtectKernelLogs=` still on; remove those three, or go back to `PNC_BWRAP_PROC=none` |
| self-test build fails mentioning `/proc` | this rustc needs procfs: set `PNC_BWRAP_PROC=fresh` and drop the three settings above |
| `SIGSYS` / `Operation not permitted` on another syscall | find it with `journalctl -k`, check the group with `systemd-analyze syscall-filter`, add that one syscall to the allow line |
| `loopback: Failed RTM_NEWADDR` | `AF_NETLINK` missing from `RestrictAddressFamilies=` |
| `does not match the pinned 1.99.0` | step 4: wrong toolchain or `PNC_TOOLCHAIN_DIR` |
| allocation error in the self-test | raise `PNC_BUILD_ADDRESS_SPACE_MIB` (max 1408) |

Optional hardening to try once it works (each fails closed): add
`ProcSubset=pid` to the builder (expected to break bwrap, see the unit
comment); add `NoExecPaths=/tmp` to the builder.

## 8. Reverse proxy and metrics exposure

`/etc/caddy/Caddyfile`:

```caddyfile
api.example.com {
    # /metrics is not served on 7878 at all; this is belt and braces.
    respond /metrics 404
    reverse_proxy 127.0.0.1:7878
}
```

Never proxy `127.0.0.1:9878`. `sudo systemctl reload caddy`, then:

```bash
ss -ltnp | grep -E ':7878|:9878'      # both bound to 127.0.0.1 only
curl -s -o /dev/null -w '%{http_code}\n' https://api.example.com/metrics      # 404
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:7878/metrics         # 404
curl -s http://127.0.0.1:9878/metrics | head -3   # (with -H "Authorization: Bearer <metrics token>" if set)
```

Prometheus scrapes `127.0.0.1:9878` directly (add
`authorization: { type: Bearer, credentials: <PNC_METRICS_TOKENS value> }`
if set). Series: `pnc_http_requests_total{route,status}`,
`pnc_rejections_total{reason=auth|rate|queue|body|source_policy|input|timeout}`,
`pnc_rate_limit_fail_open_total`, `pnc_rate_limit_tracked_clients{limiter}`,
`pnc_rate_limit_evictions_total{limiter}`, `pnc_builds_in_flight`,
`pnc_builds_queued`, `pnc_build_duration_seconds`, `pnc_run_duration_seconds`.

## 9. Prove it: verify-sandbox.sh (required before going public)

```bash
sudo /opt/pnc-compile/verify-sandbox.sh
```

It prints one `ok` line per check and ends with `PASS (<n> checks held, 0
failed)`; anything else exits non-zero. It needs no restart and never
touches the production deny-list: probes run through
`pledge_compile_api sandbox-verify` as `pnc-builder` with the builder's own
env file. What it proves is listed at the top of the script; in short:
rustc inside the sandbox reports `couldn't read \`<path>\`: No such file
or directory` for the env files, `/etc/passwd`, the builder socket and the
API's `/proc` entry (all of which exist on the host), while an in-sandbox
file is readable through the same mechanism.

## 10. Negative test: the API must refuse unsandboxed builds with tokens

`verify-sandbox.sh` already runs this; to see it under the real unit's
environment:

```bash
sudo systemd-run --wait --pipe -p User=pnc-api -p EnvironmentFile=/etc/pnc-compile/env \
  -E PNC_BUILDER_SOCKET= -E PNC_BUILD_SANDBOX=none /usr/local/bin/pledge_compile_api serve; echo "exit=$?"
```

Expected: a non-zero exit and `refusing to start: invalid configuration:
refusing to build player code inside the token-holding server process`.
(`PNC_BUILD_SANDBOX=none` is refused whenever a token is set, the bind is
public or `PNC_TRUST_PROXY` is on; the only override,
`PNC_DEV_UNSAFE_LOCAL_BUILDS=true`, is itself refused on a non-loopback
bind.)

## 11. Dependency audit

Not run by the server. On the build box, from the repo root:

```bash
cargo install --locked cargo-audit cargo-deny    # once
cargo audit                                       # RustSec advisories for Cargo.lock
cargo deny init                                   # once, then commit deny.toml
cargo deny check advisories bans sources
```

Treat any advisory against `wasmtime`, `wasmtime-wasi`, `hyper`, `axum`
or `tokio` as a deploy blocker.

## Updating

Repeat step 3, then `sudo systemctl restart pnc-builder pnc-compile-api`
and re-run step 9. If `PINNED_RUSTC_VERSION` changed, repeat step 4 and
update `PNC_TOOLCHAIN_DIR` first. To rotate tokens, add the new token next
to the old one in `PNC_AUTH_TOKENS`, restart the API, move the clients
over, remove the old token and restart again.

## Local development (not a server)

`cargo run -p pledge_compile_api` on Windows/macOS builds in-process with
`PNC_BUILD_SANDBOX=none` (the default there). On a Linux dev box the
default is `bwrap`: set `PNC_TOOLCHAIN_DIR`, or `PNC_BUILD_SANDBOX=none`
for a loopback-only, token-less dev server.
