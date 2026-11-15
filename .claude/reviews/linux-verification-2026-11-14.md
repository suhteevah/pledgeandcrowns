# Linux verification: compile-api sandbox on a real host (2026-11-14)

*Base: `finish/integration` @ 7084912. Executed by a verification agent on
`cnc-server` (root, owner-approved), following `compile-api/deploy/README.md`
steps 1-7, 9, 10 and the loopback half of 8. Logs: `scratch/cnc/` in the
worktree (not committed). No token appears in this file or in the logs.*

**Verdict.** The two-unit design works on Linux once five defects found
here are fixed (all fixed in this commit, re-verified on the host with the
final binary): bwrap starts under the shipped builder unit, the
compile-time file-read P0 is closed on Linux by independent evidence, the
process tree dies at every deadline, slots return to full, and every
negative config test refuses. Still open (not fixed, see the end): a
blocking WASI call (`thread::sleep`) is not stopped by the 5 s run timeout,
only by the 50 s job deadline; the cgroup OOM path was not exercised; not
yet run on Debian/Ubuntu.

## Host facts

| | |
|---|---|
| OS | openSUSE Leap Micro 6.2 (immutable `/usr`; `/usr/local`, `/opt`, `/etc`, `/var` writable) |
| Kernel | 6.12.0-160000.40-default x86_64, 56 cores, 362 GB RAM |
| systemd | 257.13+suse.131 (`systemd-257.13-160000.4.1`) |
| bubblewrap / prlimit | 0.11.0 (not setuid, `-rwxr-xr-x root`) / util-linux 2.41.1 |
| SELinux | enabled, targeted policy, **permissive** (config file: permissive). Services run as `unconfined_service_t` |
| userns | `user.max_user_namespaces = 1481340`; no `kernel.unprivileged_userns_clone` (not Debian), no AppArmor |
| cgroups | v2 (`cgroup2fs`), controllers `cpuset cpu io memory hugetlb pids rdma misc`; unit limits are enforced (`memory.max=1610612736`, `pids.max=256`, `cpu.max=200000 100000` on the builder) |
| `/tmp` | **tmpfs** (so PrivateTmp job dirs are RAM, not disk) |
| Disk | `/var` 87 % before, 88 % peak (target dir), 87 % after |

## Deviations from the runbook

1. **Ports 17878 / 19878** instead of 7878 / 9878: `0.0.0.0:7878` is already
   bound by a podman `conmon` on this host. Set via the env file only.
2. **Source transfer**: `git archive` piped over ssh (no git credentials).
   The first extract had CRLF line endings (Windows checkout,
   `core.autocrlf=true`); everything was re-extracted with
   `git -c core.autocrlf=false archive` and the fixes applied as a
   `git apply` patch. All installed units/env files/scripts were LF in the
   final run (`file` reports `ASCII text`). Not a repo bug (blobs are LF);
   a `git clone` on the server is unaffected.
3. **Toolchain**: dedicated rustup with `RUSTUP_HOME/CARGO_HOME` under
   `/var/tmp/pnc` (toolchain 1.99.0 + `wasm32-wasip1`). The builder unit has
   `PrivateTmp=yes`, which privatises `/var/tmp` too, so the pinned
   toolchain was `cp -a`'d to `/opt/pnc/toolchains/1.99.0-x86_64-unknown-linux-gnu`
   (`PNC_TOOLCHAIN_DIR`). Now documented in README step 4 and the unit.
4. **Binary**: `/usr/local/bin` is writable on Leap Micro, so the runbook path
   was used unchanged (ExecStart unmodified). README/verify script were run
   from the source tree, not installed to `/opt/pnc-compile`.
5. **`systemctl start`, not `enable --now`**: nothing would come back after a
   reboot of this production node.
6. `PNC_TRUST_PROXY=true` as the runbook says (no Caddy; requests carried a
   distinct `X-Forwarded-For` like a proxy would, which also kept the
   20/min real-build limit from interfering). `PNC_METRICS_TOKENS` set.
7. Release build: `nice -n 19 ionice -c3`, `CARGO_BUILD_JOBS=24`. Cold
   release build 5 m 28 s.

## Defects found and fixed (all in this commit)

| # | Finding | Fix |
|---|---|---|
| F1 (P0 for deploy) | **Builder self-test failed: rustc cannot start without `/proc`.** `rustc: error while loading shared libraries: librustc_driver-999a121de2f042be.so` (exit 127). rustc's RUNPATH is `$ORIGIN/../lib`; glibc expands `$ORIGIN` for the main program via `/proc/self/exe`, which the no-procfs sandbox removes. Reproduced by hand: same bwrap argv + `rustc -vV` fails; `--proc /proc` or `LD_LIBRARY_PATH=<tc>/lib` both fix it. | `build_sandbox.rs`: sandbox env sets `LD_LIBRARY_PATH=<toolchain>/lib` (read-only toolchain dir). Keeps the no-procfs strategy and all three proc protections; argv unit test extended. |
| F2 (P1) | **Job deadline did not kill cargo/rustc with `PNC_BUILD_SANDBOX=none`.** The job process runs cargo through `run_captured`, which gives cargo a NEW process group; `kill(-job_pgid)` missed it. On Linux `job_isolation` failed 2/2 ("build processes survived") and 8 cargo + 8 rustc orphans (PPID 1) were left running (killed by hand). In bwrap mode the sandbox is also in its own group (seen: job pgid 4062136, bwrap/cargo/rustc pgid 4062140); there bwrap `--die-with-parent` + the PID namespace were the only thing saving it (verified: SIGKILL of the job group alone -> whole sandbox gone within 1 s). Windows never showed it (`taskkill /T` walks parent PIDs). | `proc_tree.rs` (Linux, timeout path): SIGSTOP the child's group, collect the process groups of all descendants from `/proc/*/stat` parent links, SIGKILL them all, confirm each empty before returning (else `Survivors`). After the fix: `job_isolation` 3/3 incl. the ignored runaway-rustc test, no leaked processes. |
| F3 (P1, defence in depth) | **`/usr/local` was visible inside the sandbox.** The sandbox binds all of `/usr`; on this host `/usr/local` holds operator scripts, 56 world-readable files with token/secret/password assignments (contents not read). Proof: `include_bytes!("/usr/local/bin/pledge_compile_api")` compiled (21 576 592 bytes). | `build_sandbox.rs`: `--tmpfs /usr/local` right after the `/usr` bind (a toolchain under it is bound back on top). After: `couldn't read \`/usr/local/bin/pledge_compile_api\`: No such file or directory`. Unit test checks the order. |
| F4 (P2) | **`verify-sandbox.sh` failed on rustc 1.99** (the `#[path]` probe expected the old "file not found for module" wording; 1.99 says "couldn't find file"). Its target, `/etc/pnc-compile/env`, also proved nothing: rustc only stats module files and EACCES on the 0700 dir reads as "not found" even without a sandbox. | Probe now targets the builder env file (host-readable by `pnc-builder`, with a host control), accepts both wordings, still names the path. Host contrast: the same `#[path]` on the host leaks the file through diagnostics (`--> /etc/pnc-builder/builder.env:6:35 ... # run its probes with exactly ...`); in the sandbox it is not found. |
| F5 (P3) | **A second `builder` silently stole the live socket.** A hand-started builder (same uid) removed and re-bound `/run/pnc-builder/builder.sock`; when it exited the API's socket path pointed nowhere (had to restart the real builder). | `builder.rs`: refuse to replace a socket that accepts a connection. Verified: "is a live socket (another builder is listening); refusing to replace it". |

Docs/units: README step 6 syscall-filter check rewritten (see P1-A below),
toolchain location rule, new troubleshooting rows (F1 symptom, the benign
"processes outlived the build leader" warning), systemd-run `EnvironmentFile`
precedence note, status line; builder unit: `NoExecPaths=/tmp` added
(verified), comments updated with what was verified, `prlimit64` is in
`@default`. `tests/deploy_files.rs` also requires the new rustc wording.

## Checks (final binary unless noted)

**Manual bwrap check (step 1)** as `pnc-builder`: `ok`.

**Step 6** `systemd-analyze verify` on both units: no findings for the pnc
units (warnings printed only for unrelated host units).

**Syscall groups** (`systemd-analyze syscall-filter <group>`, expanded):
```
capset: @system-service   chroot: @mount   setgroups/setuid/setresuid: @setuid
unshare/clone3/setns/prctl: @process   prlimit64: @default   mount/umount2/pivot_root: @mount
```
All are in the builder's allow set (`@system-service` includes `@process
@setuid @default`), none in its deny line. The README's original grep
printed only `@setuid` and `capset` because `syscall-filter @system-service`
lists nested groups unexpanded; fixed in the README.

**Step 7 journal**, builder, in order (one run, trimmed):
```
builder: config BuilderConfig { socket: "/run/pnc-builder/builder.sock", ... mode: Bwrap, toolchain_dir: Some("/opt/pnc/toolchains/1.99.0-x86_64-unknown-linux-gnu") ...
toolchain: rustc version rustc=rustc 1.99.0 (b940084d7 2026-09-28) pinned="1.99.0"
toolchain: matches the pinned version rustc=1.99.0
sandbox probe ok: prlimit from util-linux 2.41.1
sandbox probe ok: bubblewrap 0.11.0
sandbox probe: isolation checks passed
build self-test passed (job process: build + run) sandbox=bwrap build_ms=275
builder: listening socket=/run/pnc-builder/builder.sock max_jobs=2
```
Matches the runbook line by line (plus `build_root: created`, `swept 0
stale job dir(s)`, and a WARN `processes outlived the build leader` on
every sandboxed run: bwrap returns the command's status before its
in-sandbox init exits; the sweep kills that init. Benign, documented).
API: `config: Config { bind_addr: 127.0.0.1:17878, ... auth_tokens: <redacted; 1 configured> ...}`,
`executor: builder service (this process never builds or runs player code)`,
`listening on http://127.0.0.1:17878`, `metrics listening on http://127.0.0.1:19878/metrics (loopback only)`.
Sockets: both listeners on `127.0.0.1` only; `/run/pnc-builder` is
`pnc-builder:pnc-ipc 0750`, socket `srw-rw---- pnc-builder pnc-ipc`; API
process `pnc-api` with supplementary `pnc-ipc`.

**`systemd-analyze security`**: `pnc-compile-api.service` **1.2 OK**,
`pnc-builder.service` **1.5 OK** (builder's open items: user/mnt/pid/net/uts/ipc/cgroup
namespaces, `@mount`/`@privileged`/`@resources` reachable, AF_NETLINK, no
MemoryDenyWriteExecute, no ProcSubset, no PrivateUsers; all by design).

**verify-sandbox.sh**: first run `RESULT: FAIL (1 failed, 35 passed)` (F4);
after the fix `PASS (36 checks held, 0 failed)`, rerun on the final binary:
`PASS (36 checks held, 0 failed)`. I believe its PASS now, because every
file probe requires rustc's "No such file or directory" (EACCES gives
"Permission denied"), there is a positive control, and I reproduced each
claim independently below. Its limits: probes run via `sandbox-verify`
under `runuser`, i.e. the same bwrap argv but NOT inside the systemd unit;
the unit path is covered by the self-test and the HTTP checks.

### Independent file-read evidence (`sandbox-verify`, deny-list off, as `pnc-builder`)

Host side: `/etc/passwd` `-rw-r--r-- root`, `/var/tmp/pnc/LOG.md` `-rw-r--r--`, all exist.
```
include_str!("/etc/pnc-compile/env")  -> compiled=False  error: couldn't read `/etc/pnc-compile/env`: No such file or directory (os error 2)
include_str!("/etc/passwd")           -> compiled=False  error: couldn't read `/etc/passwd`: No such file or directory (os error 2)
include_str!("/var/tmp/pnc/LOG.md")   -> compiled=False  error: couldn't read `/var/tmp/pnc/LOG.md`: No such file or directory (os error 2)
include_str!("/proc/self/status")     -> compiled=False  error: couldn't read `/proc/self/status`: No such file or directory (os error 2)
include_bytes!("/usr/local/bin/pledge_compile_api") -> before F3: compiled=True "21576592 bytes"; after: couldn't read ...: No such file or directory
include_str!("/build/Cargo.toml")     -> compiled=True ran_ok=True, printed the server manifest ([package] name = "player_submission" ...)
include_bytes!("/usr/lib64/libc.so.6")-> compiled=True "libc bytes=2358720" (same mechanism reads files that ARE in the sandbox)
env!("PATH"/"HOME"/"RUSTC")           -> PATH=<tc>/bin:/usr/bin:/bin HOME=/build RUSTC=<tc>/bin/rustc
env!("USER")                          -> environment variable `USER` not defined at compile time
```

### Inside a job (exact builder bwrap argv, inner command replaced by a shell)

```
/ : bin->usr/bin build dev lib->usr/lib lib64->usr/lib64 opt tmp usr      (no /etc, /proc, /var, /home, /root, /sys)
/proc: absent   /etc: absent   /usr/local: empty   /dev: core fd full null ptmx pts random shm stderr stdin stdout tty urandom zero
id: uid=458 gid=457 (mapped builder uid)
env: CARGO_HOME CARGO_INCREMENTAL CARGO_TARGET_DIR CARGO_TERM_COLOR HOME LD_LIBRARY_PATH PATH RUSTC TMPDIR (+ shell's PWD SHLVL _)
ulimit: -v 1310720 kB, -t 90 s, -f 65536 blocks, -n 1024, -c 0
ip: only "1: lo <LOOPBACK,UP>" 127.0.0.1/8 ::1/128
curl https://1.1.1.1 -> Failed to connect ... Could not connect to server;  getent hosts example.com -> no resolution
/usr, /build/src: Read-only file system
```
The unit adds `PrivateNetwork=yes`, `IPAddressDeny=any` and
`RestrictAddressFamilies=AF_UNIX AF_NETLINK` on top.

### HTTP through the real API (loopback, real token)

```
intro_let_binding canonical  -> 200 0.5s {"ok":true,"stdout":"[real] 42\n-- checked: does what the mission asks","grade":"passed","behaviour_checked":true}
wrong output (41)            -> 200 0.5s {"ok":false,"grade":"wrong_output","expected":"a line reading `42`","actual":"41"}
compile error (i32 = &str)   -> 200 0.2s {"ok":false,"stderr":"error[E0308]: mismatched types ...","grade":"compile_error"}
include_str!("/etc/pnc-compile/env") -> 400 {"error":"source_policy","stderr":"this server does not allow reading files at compile time (compile-time file inclusion (`include_str`) is not allowed)"}
no token -> 401;  /health -> 200
/metrics on :17878 -> 404 (also with the metrics token);  :19878 no token -> 401, player token -> 401, metrics token -> 200
```

### Timeouts, process-tree kill, slots (after F2)

Fixtures: `slowcompile` (const-eval loop with `#![allow(long_running_const_eval)]`,
memory-flat), the tests' macro bomb, `loop {}`, `thread::sleep(3600 s)`.
```
slowcompile -> 200 in 30.0s "cargo build timed out after 30s" (inner build watchdog)
macro bomb  -> 200 in 6.4s  "memory allocation of 100663320 bytes failed" (RLIMIT_AS; rustc VmPeak 1279 MiB)
loop {}     -> 200 in 2.1s  "fuel exhausted (likely infinite loop)", grade runtime_error
sleep 3600  -> 200 in 50.0s "build and run exceeded the 50s job budget and were stopped" (job deadline; see open items)
after each: pgrep -u pnc-builder -> only the builder;  pnc_builds_in_flight 0, pnc_builds_queued 0
```
6 × slowcompile concurrently (2 slots, 8 queue): answers at 30/30/60/60/90/90 s,
peak 11 builder-user processes (2 jobs, 2 cargo, 2 rustc, 4 bwrap), afterwards
only the builder, in_flight 0 / queued 0, next request passed in 0.4 s.
6 × sleep concurrently: 50/50/100/100 s graded, the last two got
`503 deadline_exceeded` at 120 s (request deadline) while their jobs, already
started, kept their slots until killed at 150 s (correct accounting: the next
request waited 29 s for a slot). Builder log: `job hit its deadline; process
tree killed ... elapsed_ms=50007`, `free_slots` back to 2. Final binary: 2 sleep +
2 slowcompile concurrently, same result, nothing left. Kernel log: 0
seccomp/SIGSYS lines for the whole session; builder `NRestarts=0`.

### Negative config tests (systemd transient units, `PrivateNetwork=yes`)

Note: under systemd `EnvironmentFile=` overrides `-E` for the same key; the
first round's bind/trust-proxy cases were masked by the env file and were
redone without it (now in the README).
```
runbook step 10 (tokens + PNC_BUILD_SANDBOX=none, socket unset) -> exit 1 "refusing to start: invalid configuration: refusing to build player code inside the token-holding server process: bearer tokens are configured, PNC_TRUST_PROXY is on ..."
tokens, loopback, no TRUST_PROXY, sandbox none, no socket (P1-B) -> exit 1 same refusal ("bearer tokens are configured")
builder + PNC_AUTH_TOKENS / + PNC_METRICS_TOKENS               -> exit 1 "refusing to start the builder with PNC_*_TOKENS set: the builder must hold no secrets"
builder + PNC_BUILD_SANDBOX=none                                -> exit 1 "refusing to start with PNC_BUILD_SANDBOX=none: the builder is the production build service"
builder + PNC_SOURCE_DENYLIST=false                             -> exit 1 "the builder always applies the deny-list"
sandbox-verify + sandbox none                                   -> exit 2 "sandbox-verify only runs with PNC_BUILD_SANDBOX=bwrap"
TRUST_PROXY=true on 0.0.0.0:17879                               -> exit 1 "refusing to bind 0.0.0.0:17879 with PNC_TRUST_PROXY on ..."
  ... with PNC_ALLOW_TRUST_PROXY_PUBLIC_BIND=true               -> started (documented override; inside a private netns, stopped after 20 s)
DEV_UNSAFE_LOCAL_BUILDS=true on 0.0.0.0                         -> exit 1 "the dev override is loopback-only"
PNC_METRICS_BIND=0.0.0.0:19879                                  -> exit 1 "the metrics listener must be loopback"
PNC_SOURCE_DENYLIST=false with tokens                           -> exit 1 "refusing PNC_SOURCE_DENYLIST=false on an exposed deployment"
public bind, no tokens                                          -> exit 1 "refusing to bind 0.0.0.0:17879: it is not a loopback address and no bearer token is configured"
second builder on the live socket (after F5)                    -> exit 1 "is a live socket (another builder is listening); refusing to replace it"
```

### Second review's open questions

- **P1-A (does the builder's filter let bwrap start under systemd 257?)**
  Yes. With the shipped unit unchanged, `isolation checks passed` on every
  start and real builds succeed; no SIGSYS, no seccomp audit records.
- **ProtectProc=invisible / ProtectKernelTunables / ProtectKernelLogs on the
  builder**: they stay on; all checks above ran with them. With
  `PNC_BWRAP_PROC=fresh` and them on, the self-test fails closed:
  `bwrap: Can't mount proc on /newroot/proc: Operation not permitted`.
- **`ProcSubset=pid`**: breaks bwrap, fail-closed:
  `bwrap: Can't read /proc/sys/kernel/overflowuid: No such file or directory`.
  Stays off. `NoExecPaths=/tmp`: works (self-test + HTTP pass), adopted.
  `PrivateUsers=yes`: self-test passed here (nested userns), not adopted
  until checked on Ubuntu/AppArmor.
- **Does rustc 1.99 build without /proc?** Only with F1: the loader needs
  the lib dir named explicitly; sysroot detection works (`rustc --print
  sysroot` -> the toolchain dir).
- **Memory** (5 ms `/proc` polling, hello world, in the sandbox): rustc
  VmPeak 451 MiB / VmHWM 100 MiB, cargo 230 / 18, rust-lld 215 / 48. The
  1280 MiB RLIMIT_AS holds with ~830 MiB headroom; even the 512 MiB minimum
  builds hello world (rustc VmPeak 457 MiB, tight). The job process
  (wasmtime) reserves ~5 GiB of address space but is outside the rlimit by
  design (RSS 17 MiB). Builder cgroup `memory.peak` 652 MiB after a macro
  bomb, 987 MiB after two concurrent ones (both stopped by RLIMIT_AS,
  `oom_kill 0`).
- **Timing**: first self-test after a fresh install 923 ms (toolchain just
  copied, so page cache was not truly cold; dropping caches would be a
  host-wide sysctl write, not done); warm builds: self-test 226-304 ms,
  `sandbox-verify` hello build+run 0.41-0.47 s wall (10 runs), HTTP
  end-to-end 0.4-0.5 s.

## Tests on Linux (as root, final code)

- `cargo test -p pledge_compile_api --no-fail-fast`: **341 passed, 0 failed,
  32 ignored** (before F2: `job_isolation` 2 failed; cargo then skipped the
  remaining targets).
- `cargo test -p pledge_compile_api -- --ignored`: **32 passed, 0 failed**
  (cargo_grader 5, expectations_proof 2 (~150 real builds), grading_real 10,
  http_real 5, job_isolation 1, wasm_builder 9). 47 s.
- These run with `PNC_BUILD_SANDBOX=none` (the tests' default); the cfg(unix)
  paths covered: process-group kill (`proc_tree`, `job_isolation`), build
  root owner/mode/symlink refusal, `O_NOFOLLOW` artifact open, sandbox argv
  (pure). Running as root means the owner check compares against uid 0.
  No automated test runs the bwrap path; it is covered by the builder's
  startup self-test, `verify-sandbox.sh` and the probes above.

## SELinux

Permissive on this host, so nothing could be blocked. `ausearch -m
AVC,USER_AVC,SECCOMP -ts today` filtered for pnc/bwrap/pledge/rustc/cargo/prlimit:
0 records. Both services ran as `system_u:system_r:unconfined_service_t:s0`.
**Unverified: behaviour under enforcing mode** (and under any confined
domain).

## Still open / unverified

1. **Blocking WASI calls ignore the 5 s run timeout** (P2, not fixed: needs
   a wasm_runner design change). `std::thread::sleep(3600 s)` blocks in a
   host call where epoch interruption cannot reach; only the 50 s job
   deadline stops it, so one cheap request holds a build slot 10x longer
   than the run budget suggests. The deadline verdict is also labelled
   `grade: compile_error` although the program compiled and ran (P3).
2. cgroup OOM path (`OOMPolicy=continue`) not exercised: RLIMIT_AS always
   fired first, even with two concurrent memory bombs.
3. Not run on Debian 12 / Ubuntu 24.04 (AppArmor userns restriction), nor
   under SELinux enforcing. No Caddy/TLS step (loopback only by rule).
4. `verify-sandbox.sh` probes run outside the unit (see above).
5. Cosmetic: job-process logs carry ANSI colour codes into the journal;
   the macro-bomb verdict returns rustc's raw allocation backtrace.
6. F2's descendant scan needs `/proc` visibility of the builder's own
   processes (true under `ProtectProc=invisible`, same uid); a process that
   calls `setsid` and re-parents before the scan is still only caught by
   bwrap's PID namespace (no player code runs during a dependency-free
   build).

## Teardown

`/var/tmp/pnc/teardown.sh --all` (written before any user/unit was created)
then checked:
```
id pnc-api / pnc-builder -> no such user;  getent group pnc-ipc pnc-api pnc-builder -> none
ls /etc/systemd/system/pnc-* /etc/pnc-compile /etc/pnc-builder /run/pnc-builder /usr/local/bin/pledge_compile_api /opt/pnc /var/tmp/pnc -> all "No such file or directory"
systemctl list-units --all 'pnc*' -> 0;  ls /etc/systemd/system | grep -c pnc -> 0 (daemon-reload done)
ss -ltnp :17878/:19878 -> none;  pgrep pledge_compile_api -> none
/etc/passwd 30 lines, /etc/group 54 (as before); /etc/subuid md5 unchanged
find /etc /opt /usr/local /tmp /var/tmp /run /var/lib/systemd -uid/-gid 457..459 -> 0
```
Two leftovers found by that check and removed: `/tmp/pledge-wasm-builder`
and `/tmp/pnc-neg-root` (created by the transient negative-test units,
which had no PrivateTmp). `/var` back to 87 %. The host's other services,
containers, firewall, Caddy and SSH config were not touched.

**The teardown left the host clean.**
