# Security review — compile-api hardening (6f53ab3..83f5daf)

*Reviewer: security-reviewer agent, 2026-11-14. Recorded by the orchestrator
from the agent's report. Read-only review; items marked (inferred) were not run.*

**Verdict: not safe for a public deploy.**

## P0

1. **Player source can make rustc read any file the service user can read.**
   `wasm_builder.rs:312-364`. rustc runs on the host with the server's
   filesystem view. Compile-time file inclusion with an absolute path puts the
   contents into the wasm and so into `stdout`; `#[path]` modules leak through
   `stderr`. The deploy README installs the env file `0640 root:pnc-compile`,
   so bearer tokens are readable. Breaks design/05 section 2 (blast radius =
   one job's tempdir). Fix: run cargo in its own mount namespace (bwrap /
   nsjail) that sees only the read-only toolchain and the sandbox dir.
   Stopgap: env file `0600 root:root`.

## P1

2. **Watchdog kills cargo, not rustc.** `wasm_builder.rs:400,413`. No process
   group or job object, so rustc outlives the timeout, outside the semaphore
   (inferred). Unbounded const eval leaks one rustc per request. Fix: process
   group + group kill on Linux, Job Object on Windows, per-build rlimits.
3. **One request can stop the unit for good (inferred).** rustc grows to the
   unit `MemoryMax`, OOM kill, default `OOMPolicy=stop`, restart loop hits the
   start limit. Fix: `OOMPolicy=continue`, per-build memory caps, raise
   `StartLimit*`.
4. **Distinct-client flood locks everyone out.** `rate_limit.rs:93-111`. 100k
   /64 keys fill the table; new clients are refused on every route including
   `/health`, no token needed. Fix: evict oldest, aggregate /48 bucket, fail
   open on the global limiter.
5. **Player wasm runs inside the server process.** `lib.rs:480`. A wasmtime
   bug is code execution with tokens in memory; the epoch timeout does not
   cover `Module::new`. Fix with (1): one sandboxed worker for build and run.

## P2

- No wasmtime table/instance limits (`wasm_runner.rs:174`).
- `TRUST_PROXY` with a public bind only warns (`main.rs:33`); should refuse.
- `encounter_id` uncapped and logged raw (`lib.rs:349`): cap ~64 bytes.
- `/tmp/pledge-wasm-builder` created with no owner/mode check
  (`wasm_builder.rs:261`); pre-creation race without `PrivateTmp`.
- Sandbox dirs leak on SIGKILL/OOM; no startup sweep; the unit's tmpfs comment
  is wrong (`PrivateTmp` uses host `/tmp` storage).
- `/metrics` shares the player token, so it is effectively public.
- Chunked oversized body: client may see a reset instead of 413 (cosmetic).
- Token-length timing leak: not a real issue.

## Checked, no issue

Auth coverage of matched routes; X-Forwarded-For (right-most entry); CORS
(exact origins, no credentials); token redaction in logs; `env_clear` build
env; `--offline`; server-owned manifest with no build.rs/proc-macro path;
WASI stdio only with fuel, epoch and memory limits; `/compile` pattern grader
is linear over at most 64 KiB.
