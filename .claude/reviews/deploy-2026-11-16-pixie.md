# Deploy: compile-api at 8b23315 on the Contabo VPS ("pixie")

*First real deployment, 2026-11-16, 15:30 to 16:05 PST. Run by a deploy
agent over `ssh pixie` (user `pixiedust`, sudo). Source: `8b23315`
(= `origin/main`), copied with `git -c core.autocrlf=false archive` (0 CRLF
files). Scope of this run: installed, running on loopback only, verified.
**Nothing is exposed to the internet**: no Caddy change, no firewall change,
no DNS. No token and no host IP appear in this file. No repository file
was edited and nothing was committed.*

## Result

**Deployed and verified. Both units are running and enabled, bound to
127.0.0.1 only.**

| # | Check | Result |
|---|---|---|
| 1 | `cargo test -p pledge_compile_api --no-fail-fast` (on the host, rustc 1.99.0) | **416 passed, 0 failed, 45 ignored** |
| 2 | same, `-- --ignored` | **45 of 45 passed** (incl. the three `linux_bwrap` tests, run as a non-root user) |
| 3 | Builder startup self-test under the shipped unit | toolchain matches 1.99.0, isolation checks passed, build self-test passed |
| 4 | `verify-sandbox.sh`, both passes, installed copy (repo-identical) | 110 held, **1 failed: a systemd 255 formatting mismatch** (`IPAddressDeny` printed in the other order; see below) |
| 4b | same script with only that one alternate string accepted | **PASS (111 checks held, 0 failed)** |
| 5 | End-to-end over loopback HTTP, 22 requests | **22 of 22 as expected** |
| 6 | `#[link_name]` forged-crash input | HTTP 500 in plain and late-nonce layouts; `compile_error` in ForbidUnsafe; never a pass; **0 forged journal entries** |
| 7 | Nonce grep, our units' journal, `RUST_LOG=pledge_compile_api=debug` | **0** (builder 1724 lines incl. 51 `late_nonce=true`; API 216 lines) |
| 8 | `systemd-analyze security` | `pnc-compile-api` **1.2 OK**, `pnc-builder` **1.5 OK** |
| 9 | Listeners | `127.0.0.1:17878` (API), `127.0.0.1:19878` (metrics) only |
| 10 | AppArmor denials from our profiles during the whole run | 1, from a deliberate test (nested userns from a bwrap child, denied as designed) |
| 11 | Other services | same unit states and listening ports as the baseline, apart from ours |

No unit directive had to be changed or loosened. The shipped unit files
are installed byte-identical (`cmp`). All site-specific values live in the
env files.

## Host

Ubuntu 24.04.5, kernel 6.8.0-147-generic, systemd 255 (255.4-1ubuntu8.18),
AppArmor enabled with `kernel.apparmor_restrict_unprivileged_userns=1`
(unchanged), 6 cores, 11.7 GiB RAM. A live production machine with
existing services; listening ports before this run: 22, 80, 443, 8080,
8443, 3849, 25565, 22000, and on loopback 2019, 4910, 8384, 8444, 18080,
18096, 19090 (plus resolver and tailnet listeners).

## Every host change

### Package

`bubblewrap 0.9.0-1ubuntu0.3` (`apt-get install --no-install-recommends`).
`apt-get -s` first: 1 new package, 0 upgraded, 0 removed, nothing else
pulled in. `apt-get update` was not needed. dpkg package count 799 -> 800.
Side effects of the apt run, both checked: `needrestart` printed its usual
"outdated binaries" notice and restarted nothing (no running service
entered the active state in that window); the static, D-Bus-activated
`packagekit.service` was started by apt's hook (it was not loaded before)
and has since exited on idle by itself. Not touched by us.

The package ships **no** AppArmor profile on 24.04 (`dpkg -L bubblewrap`
has none; `/etc/apparmor.d` had no bwrap profile before or after the
install). The README's `bwrap-userns-restrict` comes with apparmor >= 4.1,
not with 24.04's 4.0.1.

### AppArmor profile: `/etc/apparmor.d/pnc-bwrap` (new file, 0644 root)

Without it, unprivileged bwrap fails on this host (checked as
`pnc-builder` before loading: `bwrap: loopback: Failed RTM_NEWADDR:
Operation not permitted`: an unconfined process that creates a user
namespace is moved into Ubuntu's `unprivileged_userns` profile, which
denies every capability, so bwrap cannot set the namespace up).

Choice: the stacked "restrict" form (Ubuntu's own `bwrap-userns-restrict`
design, written for 4.0.1's parser in the same shape as 24.04's shipped
`unprivileged_userns` profile), not the simpler documented
`flags=(unconfined) { userns, }` form (as in 24.04's `linux-sandbox`
profile). The simple form would let every program bwrap starts, and their
descendants, create further capable user namespaces, which is exactly what
the Ubuntu restriction exists to stop. With the stacked form only bwrap's
own setup code gets capabilities, and only inside the namespace it creates.

Loaded with `apparmor_parser -r /etc/apparmor.d/pnc-bwrap` (that file
only). `aa-status`: 118 -> 120 profiles loaded, 24 -> 26 in enforce mode,
complain mode 4 -> 4 (unchanged), sysctl unchanged, no other profile
touched. Full text:

```
# Pledge & Crown compile API (added 2026-11-16; removed by /opt/pnc/UNINSTALL.sh).
#
# Ubuntu 24.04 sets kernel.apparmor_restrict_unprivileged_userns=1: an
# unconfined process that creates a user namespace is moved to the
# `unprivileged_userns` profile, which denies every capability, so
# unprivileged bubblewrap cannot map uids or mount (pnc-builder.service
# needs it). This profile lets /usr/bin/bwrap itself create user
# namespaces and use capabilities INSIDE them (it never has host
# capabilities: it is not setuid), while every program bwrap starts is
# stacked with pnc-unpriv-bwrap, which denies all capabilities, so
# bwrap cannot be used to hand arbitrary code a capable namespace.
# Modelled on Ubuntu's `bwrap-userns-restrict` (apparmor >= 4.1, not
# shipped on 24.04). If the apparmor package ever ships that profile,
# delete this file (both attach to /usr/bin/bwrap).
#
# Affects /usr/bin/bwrap only, for every user on the host.

abi <abi/4.0>,

include <tunables/global>

profile pnc-bwrap /usr/bin/bwrap flags=(attach_disconnected,mediate_deleted) {
  allow capability,
  allow file rwlkm /{**,},
  allow network,
  allow unix,
  allow ptrace,
  allow signal,
  allow mqueue,
  allow io_uring,
  allow userns,
  allow mount,
  allow umount,
  allow pivot_root,
  allow dbus,

  # Children are stacked with pnc-unpriv-bwrap: both profiles apply, so
  # they never get bwrap's capabilities.
  allow pix /** -> &pnc-unpriv-bwrap,
}

profile pnc-unpriv-bwrap flags=(attach_disconnected,mediate_deleted) {
  allow file rwlkm /{**,},
  allow network,
  allow unix,
  allow ptrace,
  allow signal,
  allow mqueue,
  allow io_uring,
  allow userns,
  allow mount,
  allow umount,
  allow pivot_root,
  allow dbus,

  # Same shape as Ubuntu 24.04's own `unprivileged_userns` profile: what
  # a child execs stays stacked with this profile, so no descendant of
  # bwrap ever gets a capability.
  allow pix /** -> &pnc-unpriv-bwrap,

  audit deny capability,
}
```

**What it allows.** It attaches only to the path `/usr/bin/bwrap`. For
that program: create user namespaces and use capabilities inside them
(bwrap is not setuid, so it never holds host capabilities; AppArmor can
only restrict, never grant, so `allow capability` adds nothing in the host
namespace). For bwrap it is no looser than "unconfined" was, apart from
being allowed past the userns restriction. Every program bwrap execs, and
everything those exec, runs stacked as `pnc-bwrap//&pnc-unpriv-bwrap`
(seen in `/proc/self/attr/current`), where every capability use is denied
and logged. This is tighter than before for those programs.

Checked on the host:
- bwrap as `pnc-builder` works (`/usr/bin/true` -> ok); the child's label is
  `pnc-bwrap//&pnc-unpriv-bwrap (enforce)` and its `CapEff/CapPrm/CapBnd`
  are all 0.
- With `--cap-add ALL` the child shows the capability bits, but using them
  fails: `mount -t tmpfs` inside is refused, and `unshare -Ur` (nested
  userns) from the child is refused, with the kernel log `apparmor="DENIED"
  operation="capable" profile="pnc-unpriv-bwrap" comm="unshare"
  capname="sys_admin"`.
- An unrelated program is still restricted: `unshare -Ur` as `pnc-builder`
  outside bwrap still fails (`write failed /proc/self/uid_map: Operation
  not permitted`).
- It attaches under the builder unit's `NoNewPrivileges=yes` (the kernel
  allows a transition from unconfined under no_new_privs, and stacking is
  a subset). The builder's self-test and both verify passes ran with the
  shipped unit.

**Host-wide effect (for the owner).** **Any local user** on this host can
now run `/usr/bin/bwrap` to create user namespaces; before this change no
unprivileged program could get a usable (capable) user namespace. The
code they run inside bwrap still cannot use capabilities in any namespace,
so the restriction is not bypassed for arbitrary code. The remaining
exposure is bwrap's own setup code: mounts (tmpfs, bind, overlay),
`pivot_root` and netlink loopback setup inside its own namespaces. That
reaches kernel paths that the Ubuntu restriction otherwise keeps from
unprivileged users. It is the same trade-off as Ubuntu 24.10 and later make
by default with `bwrap-userns-restrict`. Docker containers are not
affected: `/usr/bin/bwrap` is a host path and containers run under their
own profile. To undo: `sudo apparmor_parser -R /etc/apparmor.d/pnc-bwrap
&& sudo rm /etc/apparmor.d/pnc-bwrap` (the builder then fails its
self-test, closed).

### Users and groups (README step 2)

`groupadd --system pnc-ipc`, `useradd --system --no-create-home --shell
/usr/sbin/nologin pnc-api` and `pnc-builder` (useradd also created the
groups `pnc-api` and `pnc-builder`). passwd 41 -> 43 lines, group 70 -> 73.
No home directories were created.

### Files and directories

| Path | Owner / mode | What |
|---|---|---|
| `/usr/local/bin/pledge_compile_api` | root 0755 | release build of 8b23315 (sha256 `99b658c4...fd3f8`, 18.4 MB) |
| `/opt/pnc/toolchains/1.99.0-x86_64-unknown-linux-gnu` | root, a+rX | `PNC_TOOLCHAIN_DIR` (a `cp -a` of the rustup toolchain, 697 MB) |
| `/opt/pnc/UNINSTALL.sh` | root 0700 | uninstall script (below) |
| `/opt/pnc-compile/README.md`, `verify-sandbox.sh` | root 0644 / 0755 | as README step 3; the script is identical to the repo's |
| `/etc/pnc-compile/env` | root 0600, dir 0700 | API settings and tokens |
| `/etc/pnc-builder/builder.env` | root 0644 | identical to `builder.env.example` |
| `/etc/systemd/system/pnc-builder.service`, `pnc-compile-api.service` | root 0644 | byte-identical to `compile-api/deploy/` |
| `/etc/systemd/system/multi-user.target.wants/pnc-*.service` | symlinks | from `systemctl enable` |
| `/etc/apparmor.d/pnc-bwrap` | root 0644 | above |
| `/opt/rust/{rustup,cargo}` | root | rustup with exactly one toolchain, 1.99.0 minimal + `wasm32-wasip1` (718 MB) |
| `/opt/pnc-src/` | `pixiedust` | source tree (`repo/`, `COMMIT`), release `target/` (1.1 GB after `cargo clean --profile dev`), `cargo-home/` (crate cache), build logs, the e2e and check scripts |
| `/opt/pnc-deploy/` | root 0700 | baselines (before/after), verify outputs, e2e output, the systemd-255 verify copy |
| `/run/pnc-builder/` | runtime | created by the unit |

Where the source lives: `/opt/pnc-src`, not `/opt/pnc/src`. The builder
unit binds `/opt/pnc` back into its otherwise-empty `/opt`, so anything
under `/opt/pnc` is visible to the builder and job processes. Keeping the
source tree and `target/` out of it leaves the builder's view as the
toolchain plus the uninstall script (root 0700, unreadable to it).

Disk: 2.9 GB used by us in total (`/` free 56.2 GB -> 53.3 GB).

### Ports

`127.0.0.1:17878` (API, `PNC_BIND`) and `127.0.0.1:19878` (metrics,
`PNC_METRICS_BIND`). Both were checked free before use; neither is in the
existing list.

### Configuration (site-specific values only)

`/etc/pnc-compile/env` differs from `env.example` only in:

```
PNC_BIND=127.0.0.1:17878
PNC_METRICS_BIND=127.0.0.1:19878
PNC_AUTH_TOKENS=<64 hex, generated on the host with openssl rand -hex 32>
PNC_METRICS_TOKENS=<64 hex, a different value>
PNC_CORS_ORIGINS=tauri://localhost,http://tauri.localhost
PNC_TRUST_PROXY=true
```

- `PNC_TRUST_PROXY=true` is the README's production setting (Caddy will be
  the only client). It is harmless while only loopback can connect.
- `PNC_CORS_ORIGINS`: the default (`http://127.0.0.1:8080`,
  `http://localhost:8080`) is the local dev server, and on this host port
  8080 belongs to an existing service. It was replaced with the two Tauri
  origins from the example. The web build's origin is added when exposing.
- The tokens were generated on the host and passed only through root
  processes' environment (never in argv, never printed, never copied off
  the host). `verify-sandbox.sh` confirms that the file is `600 root root`
  and that neither service user can read it.

**Log level in use:** builder `RUST_LOG=pledge_compile_api=debug`, API
`RUST_LOG=pledge_compile_api=debug,tower_http=info` (the shipped defaults).

## Toolchain

`rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478
2026-08-27)`: the same build string as the pin's doc example and the cnc
record. `pledge_compile_api toolchain` prints `1.99.0`, and the builder
journal shows `toolchain: matches the pinned version rustc=1.99.0`. The
server binary and the tests were built with this same toolchain (no other
toolchain is installed).

## Build and test (README step 3 + the review lists)

As `pixiedust` in `/opt/pnc-src/repo`, with `nice -n 19 ionice -c3`,
`CARGO_BUILD_JOBS=2`, `RUST_TEST_THREADS=2`, debuginfo off for the
dev/test profile (env only), `CARGO_TARGET_DIR=/opt/pnc-src/target` (the
repo's `.cargo/config.toml` names a Windows `G:` target dir). A watchdog
would have stopped the build if MemAvailable fell under 2 GB. It never
fired: the minimum was **7558 MB**, and the peak 1-minute load was about 3.4.

| Phase | Time | Result |
|---|---|---|
| `cargo build --release --locked -p pledge_compile_api` | 7m23s | ok |
| `cargo test --locked -p pledge_compile_api --no-fail-fast` | 10m52s | 416 passed, 0 failed, 45 ignored |
| same `-- --ignored` | 2m27s | 45 passed, 0 failed |

Per suite, default run (passed / ignored): lib 285/0, act9_unsafe_harness
9/4, builder_ipc 7/0, cargo_grader 1/5, deploy_files 9/0,
expectations_proof 1/2, grader_snapshot 1/0, grading_real 0/11, hardening
32/0, http 24/0, http_real 1/5, job_isolation 2/1, proc_tree 5/0,
run_timeout 0/6, third_review 11/2, wasm_builder 2/9, wasm_runner 26/0.
`--ignored`: act9_unsafe_harness 4, cargo_grader 5, expectations_proof 2,
grading_real 11, http_real 5, job_isolation 1, run_timeout 6, third_review
2, wasm_builder 9. These include
`linux_bwrap::late_nonce_crosses_into_the_run_sandbox_once_and_unlogged`,
`linux_bwrap::run_sandbox_hides_the_host`,
`linux_bwrap::module_runs_inside_the_run_sandbox`,
`runaway_rustc_is_killed_with_its_job` and `every_expectation_discriminates`.
The `linux_bwrap` tests ran as an unprivileged user, so they ran through
the new AppArmor profile, not as root as on cnc. Compared with e86b519
(410/45), there are 6 more default tests and the same ignored set.

Afterwards `cargo clean --profile dev` freed 1.5 GiB. The release build
was kept.

## Install (README steps 4-7)

- Step 1/2: bubblewrap, AppArmor and users, as above. README check command
  as `pnc-builder`: `ok`.
- Step 5b (world-readable secret candidates in `/etc /usr /opt/pnc`, names
  only): 264 hits, all package material (docs, CA certificates, PAM
  configs, `/etc/passwd`, our non-secret `builder.env`). The one
  name-suspicious file under `/etc` is an unmodified package template
  (`dpkg -V` clean). Nothing to fix.
- Step 6 syscall check on systemd 255: no `MISSING` (`chroot`, `mount`,
  `umount2`, `pivot_root` from `@mount`; `setgroups`, `setuid`, `setresuid`
  from `@setuid`; `unshare`, `clone3`, `setns`, `prctl` from `@process`;
  `capset` from `@system-service`; `prlimit64` from `@default`).
  `@system-service` includes `@process`, `@setuid`, `@default` and
  `@resources` on 255 as on 257, and none of the denied groups contains a
  syscall bwrap needs. `systemd-analyze verify` was not run (per the
  brief).
- `daemon-reload`, `start pnc-builder`, then `start pnc-compile-api`.
  `enable` was run only after everything below passed.

Builder journal (current invocation), in order: `builder: config
BuilderConfig { ... mode: Bwrap ... }`; `toolchain: rustc version
rustc=rustc 1.99.0 (b940084d7 2026-09-28)`; `toolchain: matches the pinned
version`; `sandbox probe ok: prlimit from util-linux 2.39.3`; `sandbox
probe ok: bubblewrap 0.9.0`; `sandbox probe: isolation checks passed`;
`build self-test passed (job process: build + run) sandbox=bwrap
build_ms=309`; `builder: listening socket=/run/pnc-builder/builder.sock
max_jobs=2`. The `proc_tree: processes outlived the build leader` WARNs are
the expected noise the README lists.

API journal: `config: Config { bind_addr: 127.0.0.1:17878, ...
auth_tokens: <redacted; 1 configured>, trust_proxy: true ... }`,
`executor: builder service (this process never builds or runs player
code)`, `listening on http://127.0.0.1:17878`, `metrics listening on
http://127.0.0.1:19878/metrics (loopback only)`.

## verify-sandbox.sh

**Run 1, the installed script (identical to the repo's):** `RESULT: FAIL
(1 failed, 110 passed)`. The one failure:

```
FAIL - shipped value: running pnc-builder has IPAddressDeny='::/0 0.0.0.0/0', shipped '0.0.0.0/0 ::/0' (drop-in or edit?)
```

Hand comparison, as README §12 says for a non-257 systemd ("its
`systemctl show` may format some values differently from the script's
`SHIPPED` table ... compare by hand before adjusting the table"):
- The installed unit has `IPAddressDeny=any` (line 163, byte-identical to
  the repo).
- systemd 255 prints that as `::/0 0.0.0.0/0`, and 257 as `0.0.0.0/0 ::/0`.
  It is the same deny-everything set. `PrivateNetwork=yes` is also set, and
  the API unit shows the same `::/0 0.0.0.0/0`.
- Every other `SHIPPED` value, the RestrictNamespaces D-Bus mask
  (`2114060288`) and the `NOT_SET` checks held, because no other FAIL line
  was printed.

So this is a calibration issue in the script on systemd 255, not a
loosened unit.

**Run 2, a copy** (`/opt/pnc-deploy/verify-sandbox.systemd255.sh`; the
installed script was left repo-identical) whose only change accepts
exactly that alternate string for exactly that property:

```
<             if [ "$got" != "$want" ]; then
>             if [ "$got" != "$want" ] && ! { [ "$prop" = IPAddressDeny ] && [ "$want" = "0.0.0.0/0 ::/0" ] && [ "$got" = "::/0 0.0.0.0/0" ]; }; then
```

Result: **`PASS (111 checks held, 0 failed)`**. Key lines: `builder unit
has its shipped values (44 lines in /etc/systemd/system/pnc-builder.service
and in systemctl show; 3 properties unset)`; `unit pass: probe service has
the running builder's settings (43 security properties identical)`; the
four `[unit] control` lines; in both passes, the rustc probes report
`couldn't read ...: No such file or directory` for the env files,
`/etc/passwd`, the socket, `/proc/sys/kernel/overflowuid`, `/proc/self/environ`
(and the API's `/proc/<pid>/environ` in `[runuser]`); `env!` reports `not
defined at compile time`; the run sandbox's whole filesystem is 5 files
(binary, ld-linux, libc, libm, libgcc_s) in both passes; it runs in its own
PID namespace (pid 2) with a cleared environment
(`LD_LIBRARY_PATH,PWD,RUST_LOG`); IP sockets get `af_refused` under the unit
and `network_unreachable` / loopback `refused` under runuser; all four
config refusals hold; HTTP `/health` 200, no token 401, end to end 200
with output, deny-list 400 `source_policy`, `/metrics` on the API listener
404 (also with a proxy header), metrics listener 401 without its token and
200 with it. 111 checks here, against 110 on cnc: this run has
`PNC_METRICS_TOKENS` set, which adds the 401 check.

**Repo fix suggested (not made):** in `verify-sandbox.sh`, compare
`IPAddressDeny` as a set (or accept both orders), and update the comment
that says the table is calibrated on systemd 257 only.

## End to end over loopback HTTP (`POST /compile-real`, real token)

Sources were taken verbatim from the deployed tree: canonical solutions
from `game/src/plugins/editor/solutions/data.rs`, variants from
`compile-api/tests/act9_programs` / `act10_programs`. The requests were
sent one after another, about 3 s apart (under `PNC_REAL_RATE_LIMIT_PER_MIN=20`).
Final complete run (`/opt/pnc-deploy/e2e.out`):

| Request | HTTP | grade | stdout |
|---|---|---|---|
| double_function canonical (ordinary, ForbidUnsafe harness) | 200 | passed | |
| double_function wrong (`n * 3`) | 200 | wrong_output | |
| extern_c canonical (late nonce) | 200 | passed | |
| extern_c WRONG_0 | 200 | wrong_output | |
| send_sync canonical (late nonce) | 200 | passed | |
| send_sync WRONG_0 | 200 | wrong_output | |
| librarian_index canonical (late nonce) | 200 | passed | |
| librarian_index WRONG_0 | 200 | wrong_output | |
| librarian_seal canonical (late nonce) | 200 | passed | |
| librarian_seal WRONG_0 | 200 | wrong_output | |
| prealloc canonical | 200 | passed | `allocations: 1` |
| prealloc ALT_0 | 200 | passed | `allocations: 1` |
| prealloc WRONG_0 | 200 | wrong_output | `allocations: 8` |
| compiler_alloc canonical | 200 | passed | `allocations: 1` |
| compiler_alloc ALT_0 | 200 | passed | `allocations: 1` |
| compiler_alloc WRONG_0 | 200 | wrong_output | `allocations: 2` |
| compiler_alloc WRONG_1 | 200 | wrong_output | `allocations: 13` |
| compiler_alloc WRONG_2 | 200 | wrong_output | `allocations: 1` (wrong join format, as on cnc) |
| `#[link_name]` forge, plain build | **500** | - | generic "internal error ... request id ..." |
| forge, `extern_c` with only the forge (bare) | 200 | compile_error | the checker's items are missing, so rustc stops before linking |
| forge, `extern_c` solution + forge (UnsafeLateNonce, reaches the linker) | **500** | - | generic body |
| forge, `double_function` (ForbidUnsafe) | 200 | compile_error | `unsafe extern` refused by the checker's `forbid(unsafe_code)` |

Each request took 0.3 to 0.8 s (2.0 s for the first late-nonce build after
start). The allocation counts match the cnc record exactly.

Forge input (the seventh review's P1-1/P2-1 shape): an `unsafe extern "C"`
block whose `#[link_name]` holds `\n  terminate called after throwing an
instance of 'std::system_error'\n` followed by a fake tracing line
carrying a marker. In an earlier run, the late-nonce forge without the
mission's items gave `compile_error` (as in the table): that was a test
input problem, not the server, and the corrected input above was added.

**Journal (our units only, `journalctl -u`, every entry since install):**
- `pnc-builder`: 5 entries contain the marker, one per forged request
  that reached the linker. All 5 are the single `ERROR ... wasm_builder:
  the toolchain crashed while building player code kind="cxx_terminate"
  ... stderr_tail=error: linking with \`rust-lld\` failed ...\n  |\n  =
  note: ...` line, with the newlines escaped as text.
- **0** entries carry the marker outside an escaped `stderr_tail=` field.
  **0** entries start with the marker or with the forged `terminate` text.
- `pnc-compile-api`: 0 marker entries. No natural lld crash occurred: 5
  crash lines = 5 forged requests.

**Nonce grep** (`PNC[0-9a-f]{32}|nonce_[0-9a-f]{32}`) at the configured
debug level: builder **0** (1724 lines in the current invocation,
including 51 `late_nonce=true` lines), API **0** (216 lines). The same
holds over the units' whole journal.

## systemd-analyze security (our units only)

```
pnc-compile-api.service: Overall exposure level 1.2 OK
pnc-builder.service:     Overall exposure level 1.5 OK
```

The API is rated better than the builder, as README step 6 expects.

## Baseline vs after

The baseline was recorded before any change (`/opt/pnc-deploy/baseline-before`,
root 0700). The after state was recorded at the end (`baseline-after`).

| Item | Before | After |
|---|---|---|
| `getent passwd` lines / sha256 | 41 / `59c303f0...` | 43 (+`pnc-api`, `pnc-builder`) |
| `getent group` lines | 70 (`36086d0f...`) | 73 (+`pnc-ipc`, `pnc-api`, `pnc-builder`) |
| dpkg packages | 799 | 800 (+bubblewrap) |
| AppArmor loaded / enforce / complain | 118 / 24 / 4 | 120 / 26 / 4 |
| `apparmor_restrict_unprivileged_userns` | 1 | 1 |
| TCP listeners | list above | the same + `127.0.0.1:17878`, `127.0.0.1:19878` |
| UDP listeners | 14 | 14, identical |
| unit files | - | + `pnc-builder.service enabled`, `pnc-compile-api.service enabled` |
| loaded units | 461 | + our two services; no other state change (see note) |
| docker containers (name + status) | - | identical |
| `/` used / free | 46.7 GB / 56.2 GB | 49.6 GB / 53.3 GB |
| MemAvailable | 10386 MB | 10367 MB |
| load (1/5/15) | 0.50 0.36 0.37 | 0.43 1.07 1.55 at the end (the 1-min value is back to normal; 5/15 still include the builds) |

Note on units: the only other differences are a docker network-namespace
mount and a veth device pair that rotated. One existing container is in a
restart loop of its own (restart count about 107,000, which predates this
session), and it restarted during the run. This deploy did not touch it.
It may be worth a look by the owner.

Nothing else was done on the host: no reboot; no service, container,
Caddy, docker, firewall, tailscale or sshd was restarted, stopped or
reconfigured; no sysctl change; no existing file was edited.
`systemctl daemon-reload` was run (needed to load our units). Reads were
limited to our own files, our units' journal, kernel audit lines for our
profiles, and `ss`/`df`/`free`/`systemctl`/`aa-status`-level status.

## Uninstall script (`/opt/pnc/UNINSTALL.sh`, root 0700)

It was written and installed before any change (sha256 `2949ce42...490a7`).
It was **not run**. It was syntax-checked (`bash -n`) and reviewed against
the final list of changes: it covers everything in the tables above,
including the `enable` symlinks (through `disable`) and the
`/opt/pnc-deploy` records. Usage: `sudo /opt/pnc/UNINSTALL.sh --yes`.

```bash
#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Pledge & Crown compile API - complete uninstall for this host.
# Written BEFORE the deploy (2026-11-16). Removes everything the deploy
# created and nothing else:
#   units      pnc-compile-api.service, pnc-builder.service (+ any pnc-verify-* transients)
#   users      pnc-api, pnc-builder        groups  pnc-ipc, pnc-api, pnc-builder
#   apparmor   /etc/apparmor.d/pnc-bwrap (unloaded first) + its cache entry
#   paths      /usr/local/bin/pledge_compile_api /etc/pnc-compile /etc/pnc-builder
#              /opt/pnc /opt/pnc-compile /opt/pnc-src /opt/pnc-deploy /opt/rust /run/pnc-builder
# It does NOT remove the bubblewrap package (see the end) and cannot remove
# journal entries already written by the two units.
#
# Usage: sudo /opt/pnc/UNINSTALL.sh --yes
set -u

[ "$(id -u)" -eq 0 ] || { echo "run as root (sudo)" >&2; exit 1; }
[ "${1:-}" = "--yes" ] || { echo "this removes the Pledge & Crown compile API from this host; re-run with --yes" >&2; exit 2; }

step() { echo "== $*"; }

step "stop and disable our units"
for u in pnc-compile-api.service pnc-builder.service; do
    systemctl disable --now "$u" 2>/dev/null || systemctl stop "$u" 2>/dev/null || true
done
# Transient probe units verify-sandbox.sh may have left behind.
for u in $(systemctl list-units --all --no-legend --plain 'pnc-verify-*' 2>/dev/null | awk '{print $1}'); do
    systemctl stop "$u" 2>/dev/null || true
    systemctl reset-failed "$u" 2>/dev/null || true
done

step "remove unit files"
rm -f /etc/systemd/system/pnc-compile-api.service /etc/systemd/system/pnc-builder.service
rm -rf /etc/systemd/system/pnc-compile-api.service.d /etc/systemd/system/pnc-builder.service.d
systemctl daemon-reload
systemctl reset-failed pnc-compile-api.service pnc-builder.service 2>/dev/null || true

step "kill any leftover processes of the service users"
for user in pnc-api pnc-builder; do
    if id "$user" >/dev/null 2>&1; then
        pkill -KILL -u "$user" 2>/dev/null || true
    fi
done
sleep 1

step "remove users and groups"
for user in pnc-api pnc-builder; do
    if id "$user" >/dev/null 2>&1; then userdel "$user"; fi
done
for group in pnc-ipc pnc-api pnc-builder; do
    if getent group "$group" >/dev/null; then groupdel "$group"; fi
done

step "unload and remove the AppArmor profile"
if [ -f /etc/apparmor.d/pnc-bwrap ]; then
    apparmor_parser -R /etc/apparmor.d/pnc-bwrap 2>/dev/null || true
    rm -f /etc/apparmor.d/pnc-bwrap
fi
# apparmor_parser may have cached the compiled policy.
find /var/cache/apparmor -maxdepth 2 -type f -name 'pnc-bwrap' -delete 2>/dev/null || true

step "remove files and directories"
rm -f /usr/local/bin/pledge_compile_api
rm -rf /etc/pnc-compile /etc/pnc-builder /run/pnc-builder
rm -rf /opt/pnc-compile /opt/pnc-src /opt/pnc-deploy
# The rustup toolchain added by the deploy (RUSTUP_HOME=/opt/rust/rustup,
# CARGO_HOME=/opt/rust/cargo); nothing else on the host used /opt/rust.
rm -rf /opt/rust
rm -f /tmp/pnc-rustup-init.sh
rm -rf /tmp/pnc-verify.* 2>/dev/null || true
# Last, because this script lives in it.
rm -rf /opt/pnc

step "check"
fail=0
for p in /usr/local/bin/pledge_compile_api /etc/pnc-compile /etc/pnc-builder /opt/pnc /opt/pnc-compile \
         /opt/pnc-src /opt/pnc-deploy /opt/rust /run/pnc-builder /etc/apparmor.d/pnc-bwrap \
         /etc/systemd/system/pnc-compile-api.service /etc/systemd/system/pnc-builder.service; do
    if [ -e "$p" ]; then echo "STILL PRESENT: $p"; fail=1; fi
done
for user in pnc-api pnc-builder; do id "$user" >/dev/null 2>&1 && { echo "STILL PRESENT: user $user"; fail=1; }; done
for group in pnc-ipc pnc-api pnc-builder; do getent group "$group" >/dev/null && { echo "STILL PRESENT: group $group"; fail=1; }; done
if grep -q 'pnc-bwrap' /sys/kernel/security/apparmor/profiles 2>/dev/null; then echo "STILL LOADED: AppArmor profile pnc-bwrap"; fail=1; fi
if ss -ltnH | awk '{print $4}' | grep -qE ':(17878|19878)$'; then echo "STILL LISTENING on 17878/19878"; fail=1; fi
[ "$fail" -eq 0 ] && echo "uninstall complete"

cat <<'EOF'

The bubblewrap package was installed by this deploy and is NOT removed
here (other software may start using it). To remove it as well:
    apt-cache rdepends --installed bubblewrap     # must list nothing that you need
    sudo apt-get remove --purge bubblewrap        # do NOT add autoremove
Journal entries of pnc-builder / pnc-compile-api stay until journald
rotates them.
EOF
exit "$fail"
```

## What is NOT done

- **Public exposure:** no Caddy site, no DNS record, no TLS. The service
  answers on loopback only.
- **Rate limits at the proxy:** none. Only the app's own limits are active
  (600/min global, 20/min `/compile-real` per client IP, 2 concurrent
  builds plus 8 queued).
- **Metrics scraping:** nothing scrapes `127.0.0.1:19878`. The metrics
  token exists only in `/etc/pnc-compile/env`.
- **Backups:** none. Nothing here is stateful apart from the tokens; to
  restore, re-run the install and generate new tokens.
- **Log retention:** the host's default journald settings were not changed
  or inspected. At debug level the builder writes about 75 journal lines
  per graded request (1724 lines for this run's traffic). Consider
  `RUST_LOG=pledge_compile_api=info` for the builder once public, or a
  journald size cap; that is the owner's call. Debug was kept here because
  the nonce check is specified at that level.
- **Update procedure:** not automated. Manual steps below.
- **`PNC_CORS_ORIGINS`** does not contain the web build's origin yet.
- **Not verified in this run:**
  - `cargo deny check` / `cargo audit` (README step 11; it needs
    `cargo install`, out of scope);
  - game `solutions --include-ignored` (needs the alsa/udev dev packages;
    package installs beyond bubblewrap were not allowed);
  - the seventh review's item 3 (`cargo build -v` through `sandbox-verify`
    to confirm the exact rustc flags and that `/.cargo` and `/build/.cargo`
    are absent);
  - item 4 (the 56-core condition; this host has 6 cores);
  - the late-nonce test binary run as `pnc-builder` itself (it ran as
    `pixiedust`, unprivileged, with bwrap; under the unit it is covered by
    the e2e and journal checks);
  - `no_std` end to end (not in this run's list);
  - the 11-15b "loosened unit" negative test of `verify-sandbox.sh`;
  - `PrivateUsers=yes` (left off, as shipped);
  - reboot persistence (the units are enabled but the host was not
    rebooted, by rule);
  - a load test.

## Next steps to expose it behind the existing Caddy (instructions only, not done)

1. Pick the hostname and create its DNS record pointing at this host. Caddy
   then gets the certificate itself, over ports 80/443 that it already
   serves.
2. Add `https://<web-origin>` to `PNC_CORS_ORIGINS` in
   `/etc/pnc-compile/env` (`sudoedit`, keep the Tauri origins), then
   `sudo systemctl restart pnc-compile-api` (our unit only).
3. Add a site block to the existing Caddyfile. Do not remove anything
   there; note the port is **17878**, not the README's 7878:
   ```caddyfile
   api.<domain> {
       respond /metrics 404
       reverse_proxy 127.0.0.1:17878
   }
   ```
   Optionally add `request_body { max_size 256KB }`, and proxy-level rate
   limiting if a rate-limit module is available in this Caddy build (stock
   Caddy has none). The app's own per-IP limits work through Caddy because
   `PNC_TRUST_PROXY=true` and Caddy writes the client address into
   `X-Forwarded-For`.
4. `caddy validate --config /etc/caddy/Caddyfile`, then `sudo systemctl
   reload caddy`. Do this only with the owner's agreement, because Caddy
   also fronts existing services.
5. Never proxy `127.0.0.1:19878`. No firewall change is needed: both
   listeners are loopback-only, and Caddy reaches them locally.
6. Re-check:
   - `ss -ltnp | grep -E ':17878|:19878'` (still 127.0.0.1);
   - `curl -s -o /dev/null -w '%{http_code}\n' https://api.<domain>/health`
     must give 200;
   - `.../metrics` must give 404;
   - a `/compile-real` with the token must give 200;
   - `sudo /opt/pnc-deploy/verify-sandbox.systemd255.sh` (or the repo
     script once it is fixed) must print PASS.
7. Give the game client the base URL and the token (`PNC_AUTH_TOKENS`,
   read on the host with sudo; do not paste it into chat or a repo).
8. Optional: have Prometheus scrape `127.0.0.1:19878` with the metrics
   token as a Bearer credential.

**Update procedure (manual):**
1. Copy the new commit with `git -c core.autocrlf=false archive <sha>` into
   `/opt/pnc-src/repo`.
2. As `pixiedust`, run the release build with the same env as
   `/opt/pnc-src/build-and-test.sh`, at nice 19 with 2 jobs.
3. `sudo install -m 0755 .../target/release/pledge_compile_api /usr/local/bin/`.
4. Re-copy the deploy files if they changed (`cmp` first).
5. `sudo systemctl restart pnc-builder pnc-compile-api`.
6. Re-run `verify-sandbox.sh` and `/opt/pnc-src/e2e.py`.
7. If `PINNED_RUSTC_VERSION` changes, install the new toolchain into
   `/opt/rust`, copy it to `/opt/pnc/toolchains/`, and update
   `PNC_TOOLCHAIN_DIR` first.
