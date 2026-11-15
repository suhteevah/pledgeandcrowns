# Linux verification: third security review fixes (2026-11-15b)

*Base: `finish/integration` @ f8b279c plus this commit. Executed on
`cnc-server` (root, owner-approved; the same host as 2026-11-14/15:
openSUSE Leap Micro 6.2, kernel 6.12.0-160000.40, systemd 257.13, bubblewrap
0.11.0, SELinux targeted/permissive, 56 cores). Everything under
`/var/tmp/pnc` (own `RUSTUP_HOME`/`CARGO_HOME`, toolchain 1.99.0 +
`wasm32-wasip1`), builds `nice -n 19 ionice -c3`, at most 24 jobs, `/var`
87 % before, 89 % peak. Same deviations as before: ports 17878/19878,
toolchain copied to `/opt/pnc/toolchains/1.99.0-x86_64-unknown-linux-gnu`
(now also the README's required location), `systemctl start` only. Units
and env files installed UNCHANGED from `compile-api/deploy/` except the
two ports. No token appears in this file. No file on the host that this
run did not create was read.*

## 1. Run sandbox (S-1)

Design chosen: **(b) sibling sandbox**. The `job` process stays the
supervisor with the host view (no player code in it) and, after the build
sandbox has exited, runs the module in a second bwrap sandbox:
`prlimit --cpu=30 --fsize=0 --nofile=64 --core=0 -- bwrap --unshare-all
--die-with-parent --cap-drop ALL --clearenv --ro-bind <libs> ... --ro-bind
/usr/local/bin/pledge_compile_api /pnc/pledge_compile_api --chdir /
--setenv LD_LIBRARY_PATH /usr/lib64 -- /usr/lib64/ld-linux-x86-64.so.2
/pnc/pledge_compile_api run-wasm`. Pipes only (module + limits in, JSON
verdict out). No nesting of user namespaces is needed, so it works under
the shipped unit unchanged: the evidence below is the builder's own
startup self-test (a module built AND run through the run sandbox under
the real unit) and the `[unit]` probe pass. Nested bwrap (option a) was
not needed and not measured.

Builder journal, first start under the shipped unit (trimmed):
```
sandbox probe: isolation checks passed
jobs: starting job process exe=/usr/local/bin/pledge_compile_api encounter=pnc_self_test deadline=50s
  proc_tree: this process is a child subreaper
  proc_tree: spawned label="cargo build" ... finished code=Some(0) elapsed=200.9ms
  run_sandbox: run image RunImage { exe: "/usr/local/bin/pledge_compile_api", loader: Some("/usr/lib64/ld-linux-x86-64.so.2"), libs: ["/usr/lib64/libc.so.6", "/usr/lib64/libm.so.6", "/usr/lib64/libgcc_s.so.1"] }
  run_sandbox: starting run child argv=["/usr/bin/prlimit", "--cpu=30", "--fsize=0", "--nofile=64", "--core=0", "--", "/usr/bin/bwrap", "--unshare-all", "--die-with-parent", "--cap-drop", "ALL", ...
  run-wasm: running module bytes=62609 h=RunHeader { timeout_ms: 5000, fuel: 2000000000, max_memory_bytes: 16777216, wasm_bytes: 62609 }
  wasm_runner: _start returned Ok
  proc_tree: finished label="wasm run" code=Some(0) elapsed=143.4ms
build self-test passed (job process: build + run) sandbox=bwrap build_ms=201
builder: listening socket=/run/pnc-builder/builder.sock max_jobs=2
```
JIT: works in the run sandbox with the unit's settings (no
`MemoryDenyWriteExecute`, anonymous executable mappings are not affected by
`NoExecPaths=/tmp` or bwrap's mounts; bwrap adds no seccomp filter).

What the builder process itself sees after the new unit settings
(`nsenter -t <builder pid> -m ls -A`, names only):
```
/usr/local      bin            /usr/local/bin  pledge_compile_api
/opt            pnc            /run            pnc-builder systemd   (find: /run/systemd/incoming, empty)
/var/lib /var/log /srv /home /root   (empty / inaccessible)
```

`cargo test --test third_review linux_bwrap -- --ignored` (root, bwrap):
`module_runs_inside_the_run_sandbox ok`, `run_sandbox_hides_the_host ok`
(probe line `{"check":"tree","subject":"/","result":"/pnc/pledge_compile_api,/usr/lib64/ld-linux-x86-64.so.2,/usr/lib64/libc.so.6,/usr/lib64/libgcc_s.so.1,/usr/lib64/libm.so.6"}`).
First run of that test found two wrong expectations of mine, both fixed:
`/usr` itself exists inside (bwrap creates the parents of the bound
libraries; probe `/usr/bin` instead, plus the full file listing) and bwrap
sets `PWD` for `--chdir`.

## 2. verify-sandbox.sh on the real units

First run of the new script: `RESULT: FAIL (5 failed, 108 passed)`: three
SHIPPED show values needed calibrating to systemd 257's formatting
(`SystemCallErrorNumber` shows `1`, `RestrictAddressFamilies` shows
`AF_NETLINK AF_UNIX`; `RestrictNamespaces` shows `no` for "all but time",
the same word as an unrestricted unit, so the script now reads the raw
D-Bus mask, `2114060288` = 0x7E020000), and in the `[unit]` pass the
probe's IP sockets are refused outright by the unit's
`RestrictAddressFamilies=` (EAFNOSUPPORT, now reported as `af_refused`)
rather than reaching bwrap's empty netns. After those fixes:
```
PASS (111 checks held, 0 failed)
ok - builder unit has its shipped values (44 lines in /etc/systemd/system/pnc-builder.service and in systemctl show; 3 properties unset)
ok - unit pass: probe service has the running builder's settings (43 security properties identical)
ok - [unit] control: /proc/sys/kernel/overflowuid IS visible to the unit without bwrap
ok - [unit] control: /run/pnc-builder/builder.sock IS visible to the unit without bwrap
ok - [unit] control: /usr/local/bin/pledge_compile_api IS visible to the unit without bwrap
ok - [unit] control: /opt/pnc/toolchains/1.99.0-x86_64-unknown-linux-gnu IS visible to the unit without bwrap
ok - [unit] probe '/proc/sys/kernel/overflowuid (not hidden by ProtectProc)': rustc in the sandbox: couldn't read `/proc/sys/kernel/overflowuid`: No such file or directory
ok - [runuser] probe 'API process environ': rustc in the sandbox: couldn't read `/proc/<api pid>/environ`: No such file or directory
ok - [runuser|unit] run phase: positive control: the probe received its request
ok - [runuser|unit] run phase: positive control: its own binary is visible at /pnc/pledge_compile_api
ok - [runuser] run phase: positive control: loopback connect is refused (networking works at all)
ok - [runuser] run phase: no network: an outside address is unreachable            (network_unreachable)
ok - [unit] run phase: IP sockets refused by the unit (loopback / outside)          (af_refused)
ok - [runuser|unit] run phase: <each of 16 host paths> is not in the run sandbox
     (/etc/pnc-compile/env /etc/pnc-builder/builder.env /etc /etc/passwd /run /run/pnc-builder/builder.sock
      /proc/self /proc/sys/kernel/overflowuid /usr/local/bin/pledge_compile_api /usr/local /usr/bin/bwrap
      /opt/pnc/toolchains/... /tmp /var /home /root; each exists on the host)
ok - [runuser|unit] run phase: the builder socket cannot be connected to
ok - [runuser|unit] run phase: runs as the builder uid (458); rlimit cpu 30/30, nofile 64/64, fsize 0/0, core 0/0
ok - [runuser|unit] run phase: own PID namespace (probe is pid 2)
ok - [runuser|unit] run phase: cleared environment (LD_LIBRARY_PATH,PWD,RUST_LOG)
ok - [runuser|unit] run phase: the sandbox's whole filesystem is the binary and its libraries (5 files: /pnc/pledge_compile_api,
     /usr/lib64/ld-linux-x86-64.so.2,/usr/lib64/libc.so.6,/usr/lib64/libgcc_s.so.1,/usr/lib64/libm.so.6)
ok - sandbox-verify-run with a token: refused (must hold no secrets)
ok - end-to-end build ran and returned the program's output
```
(every build-sandbox probe of 2026-11-15 still holds in both passes.)

### Deliberate loosening (negative test), then restore

```
== A: unit file edited: ProtectSystem=strict -> ProtectSystem=full   (builder: active)
FAIL - shipped value: 'ProtectSystem=strict' is not in /etc/systemd/system/pnc-builder.service (edited or loosened?)
FAIL - shipped value: running pnc-builder has ProtectSystem='full', shipped 'strict' (drop-in or edit?)
RESULT: FAIL (2 failed, 110 passed)
== B: drop-in: ProtectProc=default, NoNewPrivileges=no                (builder: active)
FAIL - shipped value: running pnc-builder has ProtectProc='default', shipped 'invisible' (drop-in or edit?)
FAIL - shipped value: running pnc-builder has NoNewPrivileges='no', shipped 'yes' (drop-in or edit?)
RESULT: FAIL (2 failed, 110 passed)
== restored (cmp with the shipped unit: identical; no drop-ins)
PASS (111 checks held, 0 failed)
```
The previous script would have passed both A and B: its unit pass only
compared the probe service with the (equally loosened) running builder.

## 3. End to end through the HTTP API (real units)

```
pass (stdout mission)      -> 200 0.38s {"ok":true,"stdout":"[real] 42\n-- checked: does what the mission asks","grade":"passed"}
wrong output               -> 200 0.35s {"ok":false,"grade":"wrong_output","expected":"a line reading `42`","actual":"41"}
compile error              -> 200 0.14s {"ok":false,"stderr":"error[E0308]: mismatched types\n --> src/main.rs:2:26 ...","grade":"compile_error"}
pass (harness mission)     -> 200 0.37s {"ok":true,"grade":"passed","note":"checked by calling your code (5 checks)"}
busy loop (fuel)           -> 200 0.69s {"ok":false,"stderr":"fuel exhausted (likely infinite loop)","grade":"runtime_error"}
sleep 3600 s (run timeout) -> 200 5.38s {"ok":false,"stderr":"timeout: your program was still running after the 5 s time limit ...","grade":"runtime_error"}
unsafe in harness mission  -> 200 0.16s {"ok":false,"stderr":"error: this mission is checked by calling your code, and its checker does not allow `unsafe` code. ...\nerror: usage of an `unsafe` block\n --> src/main.rs:2:77 ..."}
build timeout (30 s)       -> 200 30.0s {"ok":false,"stderr":"cargo build timed out after 30s ..."}
after EVERY request above: builder-user processes 1 (the builder), builder cgroup procs 1
```
cgroup OOM (MemoryMax lowered at runtime to 400M, tightening only, two
macro bombs at once): both `200 1.07s compile_error`; kernel `Memory cgroup
out of memory: Killed process ... (rustc) ... oom_memcg=/system.slice/pnc-builder.service`
x2; `memory.events oom 3 oom_kill 2`; builder procs 1, cgroup procs 1,
NRestarts 0; MemoryMax restored (`memory.max=1610612736`, runtime drop-in
removed); next request `200 0.38s passed`.

## 4. Socket takeover attempt

An "escaped job": a process running as `pnc-builder`, moved into the
builder's cgroup, unlinks `/run/pnc-builder/builder.sock` and binds its own:
```
== before      MainPID=1014709 NRestarts=0 state=active sock_ino=1290107
impostor bound 1290753
ERROR builder: socket watch: /run/pnc-builder/builder.sock was replaced by another socket
ERROR builder: ... exiting so systemd kills this unit's cgroup and restarts the builder on a clean socket
pnc-builder.service: Main process exited, code=exited, status=71/OSERR
== 4 s later   MainPID=0 state=activating; impostor dead (killed with the cgroup)
pnc-builder.service: Scheduled restart job, restart counter is at 1.
sandbox probe: isolation checks passed / build self-test passed / builder: listening
== 12 s later  MainPID=1021912 NRestarts=1 state=active sock_ino=1290793
hello -> 200 0.36s passed; builder-user procs 1
```
(Inside the run and build sandboxes `/run` does not exist at all: see
section 2.)

## 5. Tests on cnc (root, `PNC_BUILD_SANDBOX=none` as the tests default)

- `cargo test -p pledge_compile_api --no-fail-fast`: **391 passed, 0
  failed, 41 ignored** (final code). One earlier run had 1 failure, a race
  in a NEW test of mine: a "stale socket" made by dropping a
  `UnixListener` can still accept while a child forked by a concurrent
  test holds the fd until its exec; the test now binds without `listen()`.
  Then `third_review` + `proc_tree` 5x in a row: all green.
- `cargo test -p pledge_compile_api --no-fail-fast -- --ignored`: **41
  passed, 0 failed** (includes `expectations_proof::every_expectation_discriminates`,
  i.e. every mission's canonical solution still passes under
  `#![forbid(unsafe_code)]` and every wrong variant still fails;
  `grading_real::harness_mission_refuses_an_unsafe_memory_scan_at_compile_time`;
  `job_isolation::runaway_rustc_is_killed_with_its_job`; `run_timeout` x6;
  `third_review::linux_bwrap` x2).

`cargo clippy -p pledge_compile_api --all-targets -- -D warnings`: clean.

Windows dev box (3 jobs, Git Bash): `cargo fmt -- --check` clean;
`cargo test -p pledge_compile_api`: **383 passed, 0 failed, 39 ignored**
(the Linux-only tests are cfg'd out); `-- --ignored`: **39 passed, 0
failed**; `powershell -ExecutionPolicy Bypass -File scripts/ci.ps1`: fmt,
check, clippy, test, check-demo, check-wasm all ok.
New Linux-only tests that passed: `proc_tree::clean_exit_leaves_no_setsid_descendant_behind`,
`clean_exit_sweeps_the_whole_escaped_subtree`; `third_review::unix_socket::*`
(no listener 503 in < 1 s and slot back; full backlog -> `Timeout("connect")`
after 300 ms; accepts-and-never-replies -> 503 within the 10 s ack deadline);
`builder::tests::socket_watch_fires_when_the_socket_is_replaced_or_removed`.

## 6. Other

- `systemd-analyze security`: builder 1.5 OK, API 1.2 OK (unchanged).
- Journal ANSI: 0 escape bytes in the current invocation of either unit
  (`journalctl _SYSTEMD_INVOCATION_ID=... | grep -c $'\x1b'` = 0 / 0;
  older invocations from 2026-11-14/15 in the same journal still have them).
- SELinux: both services `system_u:system_r:unconfined_service_t:s0`
  (`ps -eZ`), host permissive, mode not changed. The audit log was not read
  this time (not a file this run created); enforcing mode stays unverified.
- `cargo deny check advisories bans sources` (cargo-deny 0.19.8, on the
  Windows dev box): `advisories ok, bans ok, sources ok`. rustix gained only
  the `net` feature (no new crate; Cargo.lock unchanged).

## Teardown

A last `verify-sandbox.sh` after the OOM and takeover runs: `PASS (111
checks held, 0 failed)`. Then `/var/tmp/pnc/teardown.sh` (written before
any user or unit was created; its first version also matched itself with
`pkill -f /var/tmp/pnc/` and ended the ssh session mid-way, so the pattern
was narrowed to `/var/tmp/pnc/target/` and it was run again), then
`teardown.sh --all`, then:
```
users:   id: 'pnc-api': no such user | id: 'pnc-builder': no such user
groups:  pnc-ipc:none pnc-api:none pnc-builder:none
units:   list-units pnc* = 0; unit files = 0; /etc/systemd/system/*pnc* = 0; runtime drop-ins = 0
absent:  /etc/pnc-compile /etc/pnc-builder /run/pnc-builder /usr/local/bin/pledge_compile_api /opt/pnc /opt/pnc-compile /var/tmp/pnc /tmp/pledge-wasm-builder
tmp:     0 /tmp/pnc-* /tmp/pledge-* entries
ports:   0 listeners on 17878/19878
procs:   pledge_compile_api=0 rustc=0 processes of pnc users=0
passwd:  435f0887... lines=30   (baseline identical)
group:   4b75742c... lines=54   (baseline identical)
subuid / subgid: 9007d5cd... (baseline identical)
owned:   0 files owned by uid 458/459 or gid 457-459 under /etc /opt /usr/local /tmp /var/tmp /run /var/lib/systemd
disk:    /var 87 % (as before)
```
No reboot, no package change, no sysctl or SELinux change; the cgroup
MemoryMax change was runtime-only and reverted before teardown; the
host's other services, containers and firewall were not touched. **The
host is clean.**

## Still open

- SELinux enforcing and any confined domain: untested (fail closed:
  `deny_execmem` would stop the run sandbox's JIT, userns/mount denials
  would stop bwrap; both show in the startup self-test).
- Debian 12 / Ubuntu 24.04: untested (AppArmor bwrap profile under
  `NoNewPrivileges=yes`; systemd 252 syscall grouping; `systemctl show`
  formatting vs the SHIPPED table, which fails closed).
- A wasmtime escape still has the run child's own CPU (30 s rlimit, 19 s
  deadline), memory up to the unit's `MemoryMax` (shared) and tasks up to
  `TasksMax`; going further needs a kernel bug or a bug in the job
  process's strict JSON decoding of the child's reply.
