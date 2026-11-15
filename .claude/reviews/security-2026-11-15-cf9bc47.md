# Fourth security review — compile-api (021ceb3..cf9bc47, UnsafeLateNonce)

*Reviewer: security-reviewer agent, 2026-11-15. Recorded by the orchestrator
from the agent's report. Read-only; the reviewer built and ran nothing. Tags:
[code] read in source at cf9bc47, [inf] inference.*

## Verdict

cf9bc47 does **not** change the third review's deploy verdict (safe to deploy
publicly on a host like the tested one once its three fixes are done; not yet
on Debian/Ubuntu). **Safe to merge.** No P0, no P1.

It adds one server-side surface, the host function `take_late_nonce`, which
checks out clean. Taking `#![forbid(unsafe_code)]` off for opted missions
gives the server nothing new to withstand: the Plain layout (every
Stdout/RunOnly mission and every unknown encounter id) has never had the
forbid (`wasm_builder.rs:94`, `:504-507`), so any API client could already
send `unsafe` player code, and the third review's wasmtime and bwrap
assumptions already cover it. The nonce never reaches argv, env, disk or logs.

**Do not register an UnsafeLateNonce mission until the Linux list below has
passed.**

## Questions checked

1. **Host import: clean [code].** Fixed 35-byte write through wasmtime's
   bounds-checked `Memory::write`; wrong `len` returns -2 before memory is
   touched; out of range returns -3; nothing is read from the guest. One-shot:
   the slot is consumed before any argument check, later calls get -1. Missing
   or non-memory `memory` export returns -3; shared memory, memory64 and
   multi-memory are refused at validation (`wasm_runner.rs:366-390`,
   `MAX_MEMORIES = 1`). No re-entrancy (it calls nothing in the guest), no
   allocation, no blocking, no panic path. One Store per run, one run per
   child process.
2. **Absent for every non-opted run: clean [code].** Linked only under
   `if let Some(name) = import.as_deref()`; `late` comes only from
   `grade_harness` with `HarnessMode::UnsafeLateNonce`, which comes only from
   the static mission table. `HarnessMode` does not derive `Deserialize`;
   every constructor in `data.rs` hard-codes `ForbidUnsafe`; no `Default`.
   No API or IPC path selects the mode or the nonce. `RunHeader.late_nonce`
   is optional, `deny_unknown_fields` on both structs, shape validated on
   both sides.
3. **What player `unsafe` reaches on the server: nothing new [code][inf].**
   `#[link(wasm_import_module)]` is still refused (`source_policy.rs:224`,
   including under `cfg_attr`). Bare `extern` blocks import from `env`, which
   is not linked. wasi-libc symbols land on the same WASIp1 host functions
   the third review already assumed reachable. Start sections need linker
   flags the player cannot pass. bwrap spec, argv and env unchanged.
4. **Secret handling: clean [code].** `LateNonce` `Debug` redacts both
   fields; error strings are fixed text; the nonce travels only on the
   child's stdin pipe; the checker file holds only the import name; raw guest
   stdout is never logged.
5. **IPC / `RunHeader`: clean [code].** Parent and child are the same binary;
   skew fails closed. Header cap (4 KiB) still checked before parsing; stdin
   capped. API/builder protocol untouched.
6. **Test gaps:** see findings.
7. **`cargo deny check licenses`: noise for this deploy [code][inf].**
   `deny.toml` has no `[licenses]` section, so with no allow list every
   license is denied. The documented gate is
   `cargo deny check advisories bans sources`. Lockfile unchanged. A license
   policy is a separate compliance task.

## Findings

- **P2-1 [code]:** the Linux bwrap test
  `late_nonce_crosses_into_the_run_sandbox_once_and_unlogged`
  (`tests/act9_unsafe_harness.rs:221-257`) had never been compiled or run at
  review time, and no test grades a late-nonce mission through bwrap build
  plus bwrap run. Risk [inf]: it pairs the test binary's mapped shared
  objects with the server exe; if they link different libraries it fails
  (closed).
- **P2-2 [code]:** no test for a module that exports no `memory` (-3 path) or
  for `ptr = 0xFFFFFFFF`. Correct by reading; add the two cases.
- **P2-3 [inf]:** when Act 9 is registered, turn
  `every_registered_mission_keeps_forbid_unsafe` into an explicit allow-list
  of opted mission ids rather than deleting it.
- **P2-4 [code], pre-existing, hardening only:** a repeated import call logs
  a `warn!` each time (unreachable by the player). The child's stderr is
  captured at 2 MiB (`run_sandbox.rs:309`), not the 64 KiB that
  `RUN_LOG_CAP`'s doc comment implies; only the 16 KiB relay reaches the
  server log.

## Cheating and correctness (not part of the security verdict)

- Documented and accepted: a player in this mode can forge a pass by
  rewriting the checker's stored result strings
  (`known_limit_heap_patch_forges_a_pass`). `no_mangle` interposition
  (allowed for `extern_c`), overwriting a function pointer, or corrupting the
  stdout buffer during the late print are assumed to work too. Same trust
  class as C-2.
- ForbidUnsafe behaviour change: a tagged line after `<done>`, or a second
  `<done>`, now fails a run that used to pass. Stricter and correct.

## Must run on Linux before any deploy that registers an UnsafeLateNonce mission

1. `cargo test --test act9_unsafe_harness -- --ignored late_nonce_crosses_into_the_run_sandbox_once_and_unlogged`
   as the builder user, on the deploy host.
2. One end-to-end grade of at least the `librarian_index` and `extern_c`
   fixtures with `SandboxMode::Bwrap` (bwrap build and bwrap run): the spec
   solution passes and one wrong variant fails.
3. After step 2, at `RUST_LOG=debug`:
   `journalctl -u <api> -u <builder> | grep -E 'PNC[0-9a-f]{32}|nonce_[0-9a-f]{32}'`
   returns nothing.
4. The full non-ignored suite on Linux, including
   `run_child_links_the_import_only_when_the_header_carries_a_nonce`.
5. `verify-sandbox.sh`, both passes.
6. `cargo deny check advisories bans sources` and `cargo audit`.
