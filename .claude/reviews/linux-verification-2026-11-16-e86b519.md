# Linux verification: compile-api at e86b519 (pre-handoff)

*Run by a verification agent on cnc-server, 2026-11-16 (01:54 to 02:30
PST). Base: `finish/integration` @ e86b519, copied with `git archive` (no
`.git`, no `target/`). Host: openSUSE Leap Micro 6.2, kernel
6.12.0-160000.40-default, systemd 257.13, bubblewrap 0.11.0, SELinux
permissive (not changed), 56 cores. Toolchain installed fresh under
`/var/tmp/pnc` (own `RUSTUP_HOME`/`CARGO_HOME`): **rustc 1.99.0
(b940084d7 2026-09-28)**, cargo 1.99.0 (5f94df478 2026-08-27), target
`wasm32-wasip1`. All test steps ran on that rustc. Builds ran under
`nice -n 19 ionice -c3` with `CARGO_BUILD_JOBS=16`, `RUST_TEST_THREADS=16`,
and debuginfo off (env only, no source change). Logs are in the session
scratchpad (`scratchpad/cnc/logs-e86b519/`), not in the repo. No token
appears in this file. No source file was edited.*

## Result

Everything that was run passed, including `verify-sandbox.sh` (both
passes) and end-to-end grading of the registered Act 9/10 missions through
the real binary under bwrap. One defect was found, and it is **not a
security issue**: the player build's `rust-lld` crashes now and then on a
host with many cores (section 4).

| # | What | Result |
|---|---|---|
| 1 | `cargo test -p pledge_compile_api --no-fail-fast` | pass: **410 passed, 0 failed, 45 ignored** |
| 2 | same `-- --ignored` (the compile-api part of `scripts/ci-slow.ps1`) | pass: **45 of 45 passed** |
| 3 | `--test act9_unsafe_harness linux_bwrap -- --include-ignored --nocapture` | pass: `late_nonce_crosses_into_the_run_sandbox_once_and_unlogged ... ok` |
| 4 | `--test expectations_proof print_canonical_transcripts -- --ignored --nocapture` | pass; `prealloc` / `compiler_alloc` print `allocations: 1` (no sandbox, root) |
| 5 | Temporary deploy of the shipped units + `verify-sandbox.sh` | **PASS (110 checks held, 0 failed)** |
| 6 | 23 end-to-end grades via HTTP -> API -> builder -> job -> bwrap build + `run-wasm` | all 23 as expected (table below) |
| 7 | Journal grep under the installed units | 0 nonce matches |
| 8 | `cargo deny check advisories bans sources` (cargo-deny 0.20.2, on cnc) | `advisories ok, bans ok, sources ok` |
| 9 | `cargo audit` (0.23.0, on cnc) | 19 vulnerabilities + 8 allowed warnings, all in the game's tree or dev-only (the same crate set as 2026-11-15); none in the server's runtime tree (item 8) |
| - | game `solutions --include-ignored` | **not run** (section 5) |

Per-suite counts, default run / `--ignored` run (passed, failed 0
everywhere):

| Suite | default (passed / ignored) | `--ignored` passed |
|---|---|---|
| lib unit tests | 279 / 0 | 0 |
| act9_unsafe_harness | 9 / 4 | 4 (incl. `linux_bwrap::late_nonce_...`, `act9_fixtures_discriminate_and_resist_attacks`, both `known_limit_*`) |
| builder_ipc | 7 / 0 | - |
| cargo_grader | 1 / 5 | 5 |
| deploy_files | 9 / 0 | - |
| expectations_proof | 1 / 2 | 2 (`every_expectation_discriminates` 43.8 s, `print_canonical_transcripts`) |
| grader_snapshot | 1 / 0 | - |
| grading_real | 0 / 11 | 11 |
| hardening | 32 / 0 | - |
| http | 24 / 0 | - |
| http_real | 1 / 5 | 5 |
| job_isolation | 2 / 1 | 1 (`runaway_rustc_is_killed_with_its_job`) |
| proc_tree | 5 / 0 | - |
| run_timeout | 0 / 6 | 6 |
| third_review | 11 / 2 | 2 (`linux_bwrap::module_runs_inside_the_run_sandbox`, `run_sandbox_hides_the_host`) |
| wasm_builder | 2 / 9 | 9 |
| wasm_runner | 26 / 0 | - |

`expectations_proof` checks the toolchain version first
(`assert_pinned_toolchain`) and passed, so it ran on 1.99.0. Totals went
from 406 / 45 (11-15c) to 410 / 45: four more default tests, the same
ignored set. Nothing flaked or needed a retry in the recorded run.

### Line endings: the first pass was thrown away

With `core.autocrlf=true` on the dev box, `git archive` writes CRLF line
endings (the blobs are LF). The first pass ran on a CRLF tree, and
`verify-sandbox.sh` then failed with `env: 'bash\r'`. That pass was
discarded. The tree was copied again with `git -c core.autocrlf=false
archive` (0 CRLF files), and `cargo clean -p pledge_compile_api` was run
for both profiles, because `git archive` stamps files with the commit time
and cargo would otherwise have reused the CRLF build. Then everything
above was run again from scratch. Every number in this record comes from
the LF tree. (The CRLF pass gave the same test counts and the same
grades, apart from one lld crash, section 4.) **Note for later runs:
always copy with `-c core.autocrlf=false`.**

## Temporary deploy (how item 5-7 were run)

The brief allowed this only with transient units and service users that
teardown removes completely. Done as follows:

- `/var/tmp/pnc/teardown.sh` was written and the baseline recorded before
  anything else was created. The deploy script refused to run if any
  target path, user or group already existed (none did).
- Units: the shipped `pnc-builder.service` and `pnc-compile-api.service`
  were copied **byte-identical** (`cmp`) to `/run/systemd/system/`. These
  are runtime units that do not survive a reboot (`UnitFileState:
  disabled`). Started with `systemctl start` only, never `enable`.
- Users and groups `pnc-ipc`, `pnc-api`, `pnc-builder` were created with
  `useradd --system`, as in README step 2.
- Fixed paths that the shipped units require:
  `/usr/local/bin/pledge_compile_api` (the release build of e86b519),
  `/opt/pnc/toolchains/1.99.0-x86_64-unknown-linux-gnu` (a `cp -a` of the
  toolchain), `/opt/pnc-compile/{README.md,verify-sandbox.sh}`,
  `/etc/pnc-compile/env` (0600) and `/etc/pnc-builder/builder.env`.
  `builder.env` is identical to `builder.env.example`; it already has
  `RUST_LOG=pledge_compile_api=debug`. `env` differs from `env.example`
  only in `PNC_BIND=127.0.0.1:17878`, `PNC_METRICS_BIND=127.0.0.1:19878`
  and a random `PNC_AUTH_TOKENS`.
- Builder journal at start: `toolchain: matches the pinned version
  rustc=1.99.0`, `build self-test passed (job process: build + run)
  sandbox=bwrap`, `builder: listening`. The API journal shows `executor:
  builder service`, and the API listened on 127.0.0.1:17878/19878 only.

## Review Linux lists, item by item

### Fifth review (`security-2026-11-15-51e1745.md`)

1. **`late_nonce_crosses_into_the_run_sandbox_once_and_unlogged` as the
   builder user: partly passed.** The test passed at e86b519 as **root**
   with bwrap (items 2 and 3). A libtest binary cannot be run as
   `pnc-builder` inside the unit. What the test guards (late nonce
   delivered into the run sandbox, never logged) was shown as the builder
   user under the real unit by items 6 and 7: four late-nonce missions
   graded `passed`, 39 `late_nonce=true` log lines, 0 nonce patterns.
2. **End-to-end bwrap grades of the registered `librarian_index` and
   `extern_c` through the real binary: passed** (item 6).
3. **Journal grep under the installed units at `RUST_LOG=debug`: passed.**
   `journalctl _SYSTEMD_INVOCATION_ID=<inv> | grep -cE
   'PNC[0-9a-f]{32}|nonce_[0-9a-f]{32}'` gives builder **0 of 930 lines**
   and API **0 of 98 lines**. Only this run's invocations were read.
4. **Full suite on Linux, default and `--ignored`: passed** (410/45, then
   45/45).
5. **`verify-sandbox.sh`, both passes: passed** on this host under the
   temporary deploy: `PASS (110 checks held, 0 failed)`, with `builder unit
   has its shipped values (44 lines in
   /run/systemd/system/pnc-builder.service and in systemctl show; 3
   properties unset)`, `unit pass: probe service has the running builder's
   settings (43 security properties identical)`, and the run-sandbox
   filesystem being exactly the 5 files (binary, ld-linux, libc, libm,
   libgcc_s) in both passes. 11-15b had 111 checks. The difference is
   probably the metrics-token branch (no `PNC_METRICS_TOKENS` here, so only
   the `200` check runs, not `401` + `200`). This was not confirmed line by
   line. The negative "loosened unit" test of 11-15b was not repeated.
6. **`cargo deny` + `cargo audit`: passed / as before** (items 8 and 9).

### Sixth review (`security-2026-11-15-96b55ac.md`)

1. **Everything still open on the fifth list:** see above.
2. **Allocation counts under bwrap as the builder user with the deploy
   toolchain: passed.** These came through the real unit (item 6), using
   the `actual` / stdout from the API:

   | Case | Grade | Output |
   |---|---|---|
   | prealloc canonical | passed | `allocations: 1` |
   | prealloc ALT_0 | passed | `allocations: 1` |
   | prealloc WRONG_0 | wrong_output | `allocations: 8` |
   | compiler_alloc canonical | passed | `allocations: 1` |
   | compiler_alloc ALT_0 | passed | `allocations: 1` |
   | compiler_alloc WRONG_0 | wrong_output | `allocations: 2` |
   | compiler_alloc WRONG_1 | wrong_output | `allocations: 13` |
   | compiler_alloc WRONG_2 | wrong_output | `allocations: 1` (wrong join format, not an allocator variant: `Ferris the Borrow Checker ...`) |

   `expectations_proof` itself runs without a sandbox (its
   `BuildSettings::default()` is `SandboxMode::None`), so it is not a bwrap
   check. Its full table (every solution, alternative and wrong variant,
   all missions) passed as root on 1.99.0 (item 2). Only the rows above
   were checked under bwrap.
3. **`solutions --include-ignored`: NOT RUN** (section 5).
4. **Registered `prealloc`, `compiler_alloc`, `no_std` end to end under
   bwrap: passed.** Solution passes and wrong variants fail for each (item
   6).
5. **`rustc -V`: recorded.** Deploy toolchain on cnc: `rustc 1.99.0
   (b940084d7 2026-09-28)`, the same build string as the pin's doc example
   in `toolchain.rs`. The dev box's `rustc -V` was not collected this run
   (no local commands, per the brief).
6. **`cargo deny` + `cargo audit`:** see the fifth list, item 6.

### End-to-end table (item 6)

23 requests, all `200`, about 0.45 to 0.57 s each:

| Mission | Canonical | Alternative | Wrong variants |
|---|---|---|---|
| librarian_index (late nonce) | passed | ALT_0 passed | 0, 1, 2: wrong_output |
| extern_c (late nonce) | passed | - | 0, 1, 2: wrong_output |
| librarian_seal (late nonce) | passed | - | 0: wrong_output |
| send_sync (late nonce) | passed | - | 0: wrong_output |
| prealloc | passed | ALT_0 passed | 0: wrong_output |
| compiler_alloc | passed | ALT_0 passed | 0, 1, 2: wrong_output |
| no_std | passed | - | 0: wrong_output |

The sources came verbatim from the game's `solutions/data.rs` and
`compile-api/tests/act9_programs` / `act10_programs` at e86b519.

## 4. Finding: intermittent `rust-lld` crash on a 56-core host (correctness, not security)

In the discarded CRLF pass, one of 23 grades (`librarian_index` WRONG_0)
came back `compile_error`:

```
error: the mission checker could not use your code. It needs: `fn fetch(shelf: &[u32], index: usize) -> Option<u32>` and a `fn main`
error: linking with `rust-lld` failed: exit status: 1
  = note: terminate called after throwing an instance of 'std::system_error'
            what():  Resource temporarily unavailable
          rust-lld: error: failed to write output '/build/target/wasm32-wasip1/release/deps/player_submission-....wasm': No such file or directory
```

Over another 32 sequential grades it did not happen again. The pids cgroup
was not the cause (`pids.peak` 72 of 256, `pids.events max 0`), and
neither was memory (`memory.events` all 0).

To test the cause, a copy of the player crate (codegen-units 16,
wasm32-wasip1 release) was built 40 times under the build sandbox's
address-space limit, `prlimit --as=1342177280` (1280 MiB):

```
AS=1280MiB default-threads:       lld EAGAIN in 6/40
AS=1280MiB lld --threads=4:       lld EAGAIN in 0/40
AS=1280MiB MALLOC_ARENA_MAX=2:    lld EAGAIN in 0/40
```

Diagnosis: rust-lld starts one thread per core (56 here). Thread stacks
plus glibc per-thread malloc arenas sometimes push past `RLIMIT_AS`, and
`pthread_create` then fails with EAGAIN. **This is a real robustness
defect, not a property of this environment**, but it depends on the
host's core count. On a small VPS (2 to 4 cores) it is unlikely. On a
large host, a correct solution can now and then get `compile_error`. The
message then also wrongly blames the player's code ("the mission checker
could not use your code"): the expectations layer treats any link failure
in harness mode as a missing signature. No source was changed. Possible
fixes for the owner: pass `-C link-arg=--threads=<n>` (or set
`MALLOC_ARENA_MAX`) for player builds, and/or tell a linker crash apart
from a missing signature in that message.

## 5. Not run, and why

- **game `solutions --include-ignored`** (sixth list, item 3): the game
  crate needs `alsa-sys` / `libudev` (present in Cargo.lock), and the host
  has neither dev package (`pkg-config --exists alsa` and `libudev` both
  fail). Installing packages is forbidden on this host. **It remains
  open.** It needs a Linux box with those headers, or the deploy host.
- **Running the late-nonce test binary as the builder user** (fifth list,
  item 1): not possible as such (see above). The behaviour was covered
  end to end.
- **The 11-15b negative test of `verify-sandbox.sh`** (deliberately
  loosening the unit): not repeated. The script and units are unchanged
  since then.
- **SELinux enforcing, Debian/Ubuntu (AppArmor):** still untested (as in
  11-15b).
- **Dev box `rustc -V` and Windows-side runs:** not collected (no local
  builds or tests, per the brief).

## Teardown

`teardown.sh` stops and removes every `pnc*` unit, kills leftover
processes, runs `userdel`/`groupdel`, and removes the paths listed above;
`--all` also removes `/var/tmp/pnc`. It ran once after the CRLF pass, once
after `verify-sandbox.sh`, and once with `--all` at the end. A first
version's process check used `pgrep -x pledge_compile_api`, which cannot
match (the name is longer than 15 characters). It was corrected to `pgrep
-f` before the later runs. Final proof:

```
users:  id: 'pnc-api': no such user | id: 'pnc-builder': no such user
groups: pnc-ipc:none pnc-api:none pnc-builder:none
units loaded: 0  unit files: 0
unit paths:   0          (/etc/systemd/system, /run/systemd/system, /run/systemd/transient)
absent:  /etc/pnc-compile /etc/pnc-builder /run/pnc-builder /usr/local/bin/pledge_compile_api
         /opt/pnc /opt/pnc-compile /tmp/pledge-wasm-builder /var/tmp/pnc
tmp:    0 /tmp/pnc-* /tmp/pledge-* private-tmp entries
ports:  0 listeners on 17878/19878
procs:  pledge_compile_api=0 cargo/rustc-under-pnc=0 bwrap=0
passwd: f879c50393ff3eb1 lines=36    (baseline: f879c50393ff3eb1 lines=36)
group:  4b49e085181c0425 lines=60    (baseline: 4b49e085181c0425 lines=60)
disk:   33G free, 97%                (baseline: 33G, 97%)
```

There was no reboot, no package change and no sysctl or SELinux change.
No existing service, container or firewall rule was touched. `systemctl
daemon-reload` was run (needed to load and unload the runtime units).

Two things went beyond "read only what we created":

- `systemd-analyze verify` on the two units printed three warnings that
  quote lines from other units on the host (`nvidia-persistenced`,
  `libsweep`, `brain-feed-ingest`). Nothing was changed and the content
  was not used further. Future runs should skip `systemd-analyze verify`
  on this host.
- The journal entries of this run's own units remain in the system journal
  (it cannot be edited selectively).

**The host is clean.**
