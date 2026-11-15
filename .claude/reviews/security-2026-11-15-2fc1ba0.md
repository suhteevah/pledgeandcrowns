# Third security review — compile-api (9cda580..2fc1ba0)

*Reviewer: security-reviewer agent, 2026-11-15. Recorded by the orchestrator
from the agent's report. Read-only; the reviewer ran nothing. Tags: [code] read
in source, [ev] backed by recorded command output, [inf] inference.*

## Verdict

**(b) Safe to deploy publicly on a host like the one tested (openSUSE Leap
Micro, systemd 257) after three small fixes:**

1. Correct `deploy/README.md:34-36` (see S-1) and add a check that the host
   has no world-readable secrets.
2. Make `verify-sandbox.sh` assert the SHIPPED values of the key builder unit
   properties, not just that the running unit matches itself.
3. Re-run `cargo deny check` and `verify-sandbox.sh` (both passes) on the
   deploy box.

**Debian/Ubuntu: not yet.** Every divergence found fails closed. Deploy only
after the builder self-test passes under the installed unit AND
`verify-sandbox.sh` prints PASS on that host.

## Status of earlier findings

| # | Status |
|---|---|
| P0-1a build sandbox | Closed, verified on Linux (11-14 file-read evidence, F3; 11-15 section 4, PASS 46) |
| P0-1b deny-list | Closed in code (`source_policy.rs:159-190`); unit tests only, acceptable because the sandbox is the control |
| P0-1c env file | Closed, verified (11-14 negative tests; verify-sandbox section 1) |
| P1-2 process tree | Verified (11-14 F2). Gap: normal-exit path `proc_tree.rs:279-285` does not scan nested groups; `setsid` is caught only by bwrap's PID namespace |
| P1-3 OOM | Verified (11-15 section 4, real cgroup OOM kill, 0 restarts) |
| P1-4 rate limiter | Closed in code; never flood-tested on Linux, logic is platform-independent |
| P1-5 wasm in server | Closed for the tokens; partial overall, see S-1 |
| P1-A filter blocks bwrap | Verified |
| P1-B unsandboxed behind proxy | Verified |
| P1-C abandoned compiles | Closed structurally; the Module::new timeout was tested only with a zero budget |
| P1-D verify script | Verified; remaining false-pass cases below |
| P2s | Closed in code; build-root owner check ran as root, weakening that evidence |

## New security findings

- **S-1 (P2) [code][inf]: wasmtime runs outside bwrap.** The run phase is in
  the job process (`wasm_builder.rs:351-358`, `jobs.rs:159-179`) with the
  builder unit's host view. A wasmtime escape could read any world-readable
  host file and return it in `stdout` (up to 256 KiB); `setsid` + double-fork
  out of the per-job kill and re-bind the builder socket (the live-socket
  check runs only at startup, `builder.rs:395-412`), then see and forge every
  job; and reach filesystem Unix sockets (D-Bus, journald). Tokens stay safe.
  Needs a wasmtime 0-day. Fix: wrap the job process in bwrap, or add
  `InaccessiblePaths=`/`TemporaryFileSystem=` to the builder unit. The README
  claim "nothing worth stealing" is wrong as written.
- **S-2 (P2) [code]:** `UnixStream::connect` (`ipc.rs:509`) has no timeout; a
  stopped builder with a leftover socket pins API slots.
- **S-3 (P2) [code]:** setup failures return `{e:#}` in a 500
  (`lib.rs:837-843`), leaking internal paths.

Checked clean: the IPC (connect ACL, length caps before allocation, unknown
fields rejected, reply caps, no settings from the API; a malicious builder can
only forge grades or claim busy); wasmtime 50 configuration (allow-list covers
future defaults, limiter attached, 4 MiB cap before parsing, layered
timeouts); the request path; the API unit's reach. RUSTSEC-2026-0182 and -0322
were availability-only, and the server has never been deployed, so there is no
past exposure.

## Cheating and correctness (not part of the security verdict)

- **C-1 (P1):** `unsafe` is allowed in player code and the harness nonce is a
  `const` (`expectations.rs:478`) in the guest's data segment. A player can
  scan memory for it, print forged tagged results and exit. Passes every
  harness mission. Fix: `#![forbid(unsafe_code)]` in `HARNESS_ENTRY_SOURCE`.
  Act 9's `unsafe` missions will need a different harness design. Residual:
  safe-code soundness exploits can still read memory in the same instance.
- **C-2 (P2):** answers can be hardcoded since failure messages reveal fixed
  inputs and expected values; unpinned return types accept a player-written
  `Debug`; `uses` is a substring check that dead code satisfies.

## Operations

- **SELinux enforcing:** untested. Could differ on `deny_execmem` (blocks the
  JIT), a confined label if the binary is installed elsewhere, or
  user-namespace policy; all fail closed at the self-test. Never fix a failure
  with `setenforce 0`.
- **Ubuntu 24.04:** whether the bwrap AppArmor profile attaches under
  `NoNewPrivileges=yes` is unverified [inf]. Do not relax the userns
  restriction host-wide.
- **Debian 12:** systemd 252 groups syscalls differently; mismatches fail
  closed.
- **verify-sandbox.sh false-pass cases:** the `/proc/$API_PID/environ` probe
  in the unit pass is hidden by `ProtectProc` even with no bwrap (`:255-257`);
  unit settings are compared with the running builder, not the shipped values
  (`:170-198`); the run phase is never probed.
