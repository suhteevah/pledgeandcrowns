# Linux verification: compile-api hardening pass (2026-11-15)

*Base: `finish/integration` @ 9cea928. Executed on `cnc-server` (root,
owner-approved, same host as 2026-11-14: openSUSE Leap Micro 6.2, kernel
6.12, systemd 257, bubblewrap 0.11, SELinux targeted/permissive, 56 cores).
Everything ran under `/var/tmp/pnc` (own `RUSTUP_HOME`/`CARGO_HOME`,
toolchain 1.99.0 + `wasm32-wasip1`), builds `nice -n 19 ionice -c3`, at most
24 jobs. Same deviations as 2026-11-14: ports 17878/19878, toolchain copied
to `/opt/pnc/toolchains/...` (PrivateTmp hides `/var/tmp`), `systemctl
start` only. No token appears in this file.*

## 1. Dependency advisories

`cargo audit` 0.22.1 before the change: **42 vulnerabilities + 23 warnings**
over the whole lockfile. Server runtime tree =
`cargo tree -p pledge_compile_api -e normal` (246 packages before, 190 after).

### In the server tree before (all fixed now)

| Advisory | Crate @ before | Reachable here? (stdio-only WASI, no preopens/sockets/components/GC/threads, one engine per run) | Fixed in |
|---|---|---|---|
| RUSTSEC-2026-0222 type indices mixed between engines | wasmtime 44.0.1 | No: one `Engine` per run, module compiled on it | 47.0.3 (have 50.0.1) |
| RUSTSEC-2026-0269 fs sandbox escape (trailing slash) | wasmtime 44.0.1 | No: no preopened dirs | 47.0.4 |
| RUSTSEC-2026-0316 record lifting beyond hostcall fuel | wasmtime 44.0.1 | No: component model only | 49.0.1 |
| RUSTSEC-2026-0327 async-lifted callback count | wasmtime 44.0.1 | No: component-model-async only | 49.0.2 |
| RUSTSEC-2026-0359 stack overflow dropping recursive types | wasmtime 44.0.1 | No: rec groups need GC/function-references, refused at validation | 49.0.3 |
| RUSTSEC-2026-0362 async-entrypoint requirement | wasmtime 44.0.1 | No in 44 (sync store only); now async store, fixed version | 49.0.3 |
| RUSTSEC-2026-0363 component handle tables | wasmtime 44.0.1 | No: components never instantiated | 49.0.3 |
| RUSTSEC-2026-0379 Cranelift aarch64 wrong jump | wasmtime 44.0.1 | No on x86_64 (would be on an aarch64 host) | 49.0.4 |
| RUSTSEC-2026-0424 borrowed resources between components | wasmtime 44.0.1 | No: components | 50.0.1 |
| RUSTSEC-2026-0149 path_open(TRUNCATE) bypasses FilePerms | wasmtime-wasi 44.0.1 | No: no preopens | 44.0.2 |
| RUSTSEC-2026-0182 leak in p1 `fd_renumber` | wasmtime-wasi 44.0.1 | Possibly (guest can renumber stdio fds); impact bounded to one job process that exits after the run (inferred from the title; advisory text not available offline) | 44.0.3 |
| RUSTSEC-2026-0188 links/renames bypass FilePerms | wasmtime-wasi 44.0.1 | No: no preopens | 46.0.1 |
| RUSTSEC-2026-0314 panic via fs datetime overflow | wasmtime-wasi 44.0.1 | No: needs a file descriptor; only stdio exists | 49.0.1 |
| RUSTSEC-2026-0321 preview-0 `poll_oneoff` circumvents fuel | wasmtime-wasi 44.0.1 | No: p0 not linked (same bug class as the run-timeout hole below, which was p1) | 49.0.2 |
| RUSTSEC-2026-0322 host memory when guests lack stdio | wasmtime-wasi 44.0.1 | Unclear (stdout/stderr are set, stdin is the closed default); inferred from the title | 49.0.2 |
| RUSTSEC-2026-0323 `fd_readdir` uninit padding | wasmtime-wasi 44.0.1 | No: no directory fds | 49.0.2 |
| RUSTSEC-2026-0352 rename symlinks escape | wasmtime-wasi 44.0.1 | No: no preopens | 49.0.3 |
| RUSTSEC-2026-0482 path/symlink trailing slash escape | cap-primitives 3.4.5 | No: no preopens. Crate gone (wasmtime-wasi 50 dropped cap-std) | removed |
| RUSTSEC-2026-0204 `fmt::Pointer` for `Atomic` | crossbeam-epoch 0.9.18 | No (via rayon). Crate gone: `parallel-compilation` not compiled in | removed |
| RUSTSEC-2026-0391/0398/0403/0410 + 11 unsound | zstd 0.13.3, zstd-safe 7.2.4, zstd-sys 2.0.16 | No: wasmtime module cache never configured. Crates gone: `cache` feature off | removed |
| RUSTSEC-2026-0190 (unsound) `downcast_mut` | anyhow 1.0.102 | No: only `downcast_ref` used | 1.0.104 |
| RUSTSEC-2026-0386 (unsound) `AsyncFd::new` | tokio 1.52.1 | No: no `AsyncFd` | 1.54.1 |
| RUSTSEC-2026-0486 (unsound) xattr | rustix 1.1.4 | No: no xattr calls | 1.1.6 |
| yanked | chacha20 0.10.0 (via wasmtime-wasi rand) | n/a | 0.10.2 |

**After: 0 advisories in the server tree.** `cargo deny check advisories
bans sources` (cargo-deny 0.19.8, new `deny.toml` scoped to the server's
runtime tree, nothing ignored, yanked = deny): `advisories ok, bans ok,
sources ok`. `cargo audit` after: 19 vulnerabilities + 9 warnings, all in
the game's tree or dev-only: quick-xml 0.39.2 (2), quinn-proto 0.11.14 (7),
quinn-udp 0.5.14, rtrb 0.3.3, rustls 0.23.40 (4), rustls-webpki 0.103.13,
ruzstd 0.8.2, webbrowser 1.2.1, webpki-roots 1.0.7; warnings bincode,
paste, ttf-parser (unmaintained), event-listener, memmap2 (unsound),
yanked async-executor, clipboard-win, spin. rustls/webpki/quinn reach the
compile-api only as reqwest dev-dependencies (tests). Not chased.

### wasmtime 44.0.1 -> 50.0.1

The changelog is not shipped in the crate; the API was re-read from the
registry source (`wasmtime-50.0.1/src/config.rs`, `runtime/store.rs`,
`runtime/limits.rs`, `wasmtime-wasi-50.0.1/src/{ctx,p1}.rs`). Changes that
mattered: `Config::async_support` is deprecated and a no-op (stores are
sync or async per call); the default proposal set is now all of
WebAssembly 3.0 plus wide-arithmetic (exceptions, extended-const and wide
arithmetic would have been silently ON with the old deny-list);
`wasm_reference_types`/`wasm_threads`/`parallel_compilation` exist only
with their cargo features; `wasmtime::Error` is its own type (anyhow via
the `anyhow` feature). Built with `default-features = false`.

| Setting | How now | Pinned by |
|---|---|---|
| Fuel | `consume_fuel(true)`, `set_fuel` | `runs_out_of_fuel_on_long_compute` |
| Epoch interruption | `epoch_interruption(true)`, deadline 1, `epoch_deadline_trap()` | `times_out_on_infinite_loop`, `busy_loop_with_unlimited_fuel_stops_at_the_run_timeout` |
| Proposals | explicit allow-list (`wasm_features(all, false)` then baseline only) | `allow_list_is_exactly_the_baseline`, `every_proposal_outside_the_allow_list_is_refused` (threads x2, GC, func-refs, exceptions, tail calls, memory64, multi-memory, extended-const, SIMD, wide arithmetic, custom page sizes, component), `accepts_baseline_bulk_memory_feature` |
| GC/threads/cache/rayon compiled out | Cargo features | `wasmtime_is_built_without_default_features` (manifest + lockfile deps) |
| Memory/table/instance limits | `StoreLimitsBuilder` + `store.limiter` | `linear_memory_growth_is_capped...`, `table_growth_is_capped...`, `table_count_and_initial_size_are_capped...`, `second_memory_or_instance_is_refused` |
| WASI = stdio only | `WasiCtxBuilder`: closed stdin, memory pipes, tcp/udp/name lookup off, `allow_blocking_current_thread(false)` | `guest_sees_no_args_no_env_no_preopens_and_no_sockets`, `stdin_is_closed_so_reads_return_eof_at_once` |
| Module size cap | 4 MiB before parse | `rejects_oversized_module_before_compiling` |
| Compile budget | 10 s helper thread | `module_compile_timeout_fails_the_run` |
| CoW images off | `memory_init_cow(false)` | (startup self-test under the builder seccomp filter) |

## 2. Run timeout

Root cause: epoch checks live in guest code; wasmtime-wasi's synchronous
p1 `poll_oneoff` (what `thread::sleep` becomes) blocked in the host for
the requested time and fuel does not burn in the host. Fix
(`wasm_runner.rs`): async store + `call_async` in a current-thread tokio
runtime wrapped in `tokio::time::timeout`, plus the epoch trap for guest
code, plus a backstop (`timeout + 2 s`) that lets the job process answer
even if both failed. Verdict: `runtime_error`, stderr starts `timeout: your
program was still running after the 5 s time limit...`. The 50 s
job-deadline verdict now says `runtime_error` too when the job had reached
its run phase (stderr marker), `compile_error` only if it was still
building. Host functions: only a clock wait in `poll_oneoff` can block
(table in `wasm_runner.rs` module docs).

Through the real units (`http-probes.sh timeout`, API -> builder -> job):
```
sleep 3600 s          -> 200   5.4s {"ok": false, "stderr": "timeout: your program was still running after the 5 s time limit and was stopped. ...", "grade": "runtime_error"}
sleep Duration::MAX   -> 200   5.4s {"ok": false, "stderr": "timeout: ...", "grade": "runtime_error"}
busy loop             -> 200   0.6s {"ok": false, "stderr": "fuel exhausted (likely infinite loop)", "grade": "runtime_error"}
stdin read            -> 200   0.4s {"ok": true, "stdout": "[real] read 0 bytes ...", "grade": "ran_not_checked"}
pnc_builds_in_flight 0 / pnc_builds_queued 0; builder procs: 1; NRestarts=0
4 x sleep concurrently (2 slots): 5.4s, 5.4s, 10.8s, 10.8s, all runtime_error; afterwards in_flight 0, queued 0
builder journal: job done ... timed_out=false elapsed_ms=5386 / 5354 / 5401 / 5401 / 5389 / 5403
```
(One of the four concurrent bodies was lost by a race in the probe
script's temp-file naming; its status line was 200 at 10.8 s.) Before:
50.0 s and `compile_error`. `run_timeout.rs --ignored` on cnc: 6 passed
(sleep job elapsed 5.63 s / run 5.33 s, never-ready 5.46 s, busy loop
0.73 s, slot back to 2/2).

## 3. Flaky tests

Wall-clock assumptions found and fixed: `hardening.rs` (fixed sleeps
racing a stub build in the queue-full, queued-wait and request-deadline
tests -> gated stub build + wait for the slot state; header/body timeout
upper bounds 4 s -> 10 s with the reason); `builder_ipc.rs` (500 ms
request/reply deadlines in every test -> 20 s except the one test about
the deadline; the drip test made discriminating: 1 byte/s vs a 500 ms
deadline); `job_isolation.rs` (immediate "no build process left" and
5 s helper wait: Windows `taskkill` is unconfirmed and teardown is
asynchronous, so Windows polls up to 30 s; Linux stays immediate because
proc_tree confirms death); `proc_tree.rs` (6 s watchdog racing two
test-binary launches -> 10 s; Windows grandchild grace 5 s -> 30 s).
Rate-limit tests use 30-60 s refill windows or injected time: fine.
Stale (not timing): `grading_real.rs` and `http_real.rs` used `vec_iter`
as an unchecked mission; 6642ca4 pinned its output (`60`), so they failed
on cnc before any change of mine. Fixed to use a no-check id.

10x on cnc while 4 cargo builds x 6 jobs (24 jobs, `rm -rf` target and
rebuild in a loop; a single build of this crate graph only kept 1-3 rustc
busy, so the load was split into four) ran in parallel, load average 22-29, 17-24 rustc
processes: each run = full default suite + `run_timeout` and
`job_isolation` `--ignored`.
```
load: 24 rustc processes, loadavg 18.66 8.54 5.15
run 1: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=24 loadavg=22.66
run 2: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=24 loadavg=24.69
run 3: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=24 loadavg=24.86
run 4: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=24 loadavg=25.61
run 5: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=24 loadavg=26.12
run 6: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=19 loadavg=26.09
run 7: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=17 loadavg=29.33
run 8: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=22 loadavg=28.20
run 9: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=22 loadavg=28.52
run 10: default exit=0 ignored exit=0 passed=367 failed=0 rustc_now=24 loadavg=29.80
PASSED 10/10 runs
```

## 4. Loose ends

**cgroup OOM path** (real units). With the shipped `MemoryMax=1536M` the
macro bomb is stopped by RLIMIT_AS (`memory allocation of 100663320 bytes
failed`, 6.4 s, `compile_error`, memory.peak 641 MiB, `oom_kill 0`), as on
2026-11-14. To exercise the cgroup OOM killer (the rlimit-absent case) the
builder's MemoryMax was TEMPORARILY lowered to 400M (`systemctl
set-property --runtime`, tightening only, restored and the runtime
drop-in removed afterwards: `memory.max=1610612736`, no drop-ins) and two
bombs sent at once:
```
macro bomb #1 -> 200 1.2s {"ok": false, "stderr": "error: could not compile `player_submission` ... process didn't exit successfully ...", "grade": "compile_error"}
macro bomb #2 -> 200 1.2s (same)
kernel: Memory cgroup out of memory: Killed process 464763 (rustc) ... oom_memcg=/system.slice/pnc-builder.service
kernel: Memory cgroup out of memory: Killed process 464759 (rustc) ...
memory.events: oom 4 oom_kill 2; builder: active, NRestarts=0, procs 1; in_flight 0, queued 0
hello after   -> 200 0.4s {"ok": true, "stdout": "[real] still serving ...", "grade": "ran_not_checked"}
```
`OOMPolicy=continue` holds: the kernel kills rustc, the builder and its
other job survive, each client gets a clean graded error, slots return.

**verify-sandbox.sh unit pass**: new `PNC_VERIFY_PROBES=runuser|unit|both`
(default both). The `[unit]` pass runs every probe in a transient
`systemd-run` service built from the installed builder unit's `[Service]`
settings, after checking 38 security properties equal the running
builder's. Run on the real units: `PASS (46 checks held, 0 failed)`
(`[runuser]` 9 probes, `unit pass: probe service has the running builder's
settings (38 security properties identical)`, `[unit]` 9 probes, each with
rustc's exact `couldn't read ...: No such file or directory`, positive
control read `/build/Cargo.toml` in both passes). A pre-existing script
bug was fixed on the way: the probe counter was incremented inside
`$(...)`, so every probe reused `probe-1.*` and failure messages named the
wrong stderr file.

**README**: the three Linux-only fixes now have their own subsection;
run-timeout, unit pass, dependency-audit scope and SELinux/AppArmor
sections added. `systemd-analyze security`: builder 1.5 OK, API 1.2 OK
(unchanged).

## 5. SELinux

Permissive host; mode not changed. Both services ran as
`system_u:system_r:unconfined_service_t:s0`. `ausearch` printed `<no
matches>` for every query on this host (even record types present in the
log), so `/var/log/audit/audit.log` was grepped directly: 2196 AVCs from
other workloads (containers: postgres, n8n, ...) during the session,
**0 for pledge_compile_api / bwrap / prlimit / cargo / rustc / rust-lld**.
Four AVCs named our files, all from the first version of the new unit
pass: `pid=1 comm="systemd" ... { write } path="/tmp/pnc-verify.*/probe-1.err"
scontext=init_t tcontext=user_tmp_t permissive=1` (systemd-run `--pipe`
handing PID 1 a file as the probe's stderr). Enforcing mode would deny
it; the script now passes pipes and the rerun logged 0. The toolchain
copied with `cp -a` from `/var/tmp` kept `user_tmp_t`: README now says to
`restorecon` it. Enforcing mode itself remains unverified, as does
Debian/Ubuntu (AppArmor userns restriction).

## 6. Tests

- cnc (root, `PNC_BUILD_SANDBOX=none` as the tests default), final code:
  `cargo test -p pledge_compile_api --no-fail-fast`: **360 passed, 0
  failed, 38 ignored**; `-- --ignored`: **38 passed, 0 failed** (adds
  `run_timeout` 6). `cargo clippy -p pledge_compile_api --all-targets --
  -D warnings`: clean.
- Windows (3 jobs): `cargo test -p pledge_compile_api`: 359 passed, 0
  failed, 38 ignored (one unit test is `cfg(unix)`); `-- --ignored`: 38
  passed. `cargo fmt --all -- --check`: clean. Full `scripts/ci.ps1` ran in
  the pre-commit hook.

## Teardown

`/var/tmp/pnc/teardown.sh --all` (written before any user or unit was
created), then:
```
id pnc-api / pnc-builder -> no such user; getent group pnc-ipc pnc-api pnc-builder -> none
ls /etc/systemd/system/pnc-* /etc/pnc-compile /etc/pnc-builder /run/pnc-builder /usr/local/bin/pledge_compile_api
   /opt/pnc /opt/pnc-compile /var/tmp/pnc /run/systemd/system.control/pnc-* /tmp/pnc-* /tmp/pledge-* -> all absent
systemctl list-units --all 'pnc*' -> 0; unit files with pnc -> 0
ss :17878/:19878 -> 0; pledge_compile_api processes -> 0; rustc -> 0
/etc/passwd 30 lines, /etc/group 54 (as before); /etc/subuid, /etc/subgid md5 unchanged
find /etc /opt /usr/local /tmp /var/tmp /run /var/lib/systemd -uid 458/459 -gid 457..459 -> nothing
/var 87 % (as before; peak 89 % during the stress run)
```
The builder's MemoryMax runtime change was reverted before teardown. No
reboot, no package change, no sysctl/SELinux change; the host's other
services, containers and firewall were not touched. **The host is clean.**
