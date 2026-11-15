# Linux verification — compile-api at cf9bc47 (UnsafeLateNonce)

*Run by a verification agent on cnc-server, 2026-11-15. Recorded by the
orchestrator from the agent's report. Logs were copied to the session
scratchpad (`scratchpad/cnc/logs/`), not into the repo.*

## Result

Everything that was run passed. **`deploy/verify-sandbox.sh` was NOT run**
this time (see below).

| # | What | Result |
|---|---|---|
| 1 | `cargo test -p pledge_compile_api --lib` | pass: 278 passed |
| 2 | `--test act9_unsafe_harness -- --include-ignored` | pass: 10 passed, incl. `linux_bwrap::late_nonce_crosses_into_the_run_sandbox_once_and_unlogged` (first time compiled) |
| 3 | `--test third_review -- --include-ignored` | pass: 13 passed |
| 4 | `--test expectations_proof -- --include-ignored` | pass: 3 passed |
| 5 | full suite, then `-- --ignored` | pass: 406 passed, 45 ignored; then 45 of 45 ignored passed (previous run: 391 / 41) |
| 6 | `verify-sandbox.sh` | **not run** |
| 7 | late-nonce fixture through the real bwrap build and run sandboxes | pass |
| 8 | `cargo deny check` (run on the Windows box; not on the host) | advisories ok, bans ok, sources ok; licenses FAILED (no `[licenses]` section in `deny.toml`; not a regression) |

Nothing failed, flaked or needed a retry. Both `known_limit_*` tests assert
their documented outcomes (heap patch forges a pass; global allocator is
caught statically).

## Item 6: why verify-sandbox.sh was not run

The script checks a deployed system (service users, units in
`/etc/systemd/system`, env files, the binary in `/usr/local/bin`, listeners).
The orchestrator's brief for this run forbade anything outside
`/var/tmp/pnc`, which was stricter than the 11-15b run that did deploy
transient units. The script, the unit files and the run-sandbox spec are
unchanged since that run's PASS (111 checks). It remains the standing gate on
the real deploy host.

## Item 7: end to end through bwrap

No existing test or switch does this (default settings use no sandbox; a
libtest binary cannot be re-executed as `run-wasm`; no registered mission
uses the mode). The agent added one throwaway example, on the host copy only
and since deleted, that calls the public `expectations::grade_check` with
`SandboxMode::Bwrap` on a verbatim copy of the `librarian_index` fixture and
hands off to `run_sandbox::run_wasm_main()` when invoked as `run-wasm`, as
`main.rs` does. Difference from production: the run sandbox binds that
example binary rather than `pledge_compile_api` (same library code).

- solution: `Passed` ("`unsafe` allowed, results keyed by the server only
  after your code finished")
- wrong (checker-only difference): `WrongOutput`
- wrong (too strict): `WrongOutput`

The debug log shows the real `prlimit` + `bwrap --unshare-all` argv for both
the build and the run, the import linked and the nonce delivered once, and 0
occurrences of the nonce pattern in 82 log lines.

This closes security finding P2-1 of
`security-2026-11-15-cf9bc47.md` for this host, except that step 3 of that
review's Linux list (journal grep under the installed units) needs a real
deploy.

## Host

- Before: root fs 96% (46G free), load about 28 from the owner's own
  inference jobs, SELinux permissive. Neither `/opt/pnc` nor `/var/tmp/pnc`
  existed, so the toolchain (rustc 1.99.0) was installed fresh under
  `/var/tmp/pnc` and dependencies fetched once online, then all builds ran
  `--offline --locked`.
- After teardown: no pnc units, users, processes, sockets, mounts, listeners
  or directories; disk back to 46G free. No reboot, package, sysctl or
  SELinux change; no existing service touched; no file read that the run did
  not create.
