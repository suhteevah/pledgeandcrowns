# Pledge & Crown audit gaps

Each entry is a bug that slipped past the audit harness and the test that now guards against it.

## 2026-05-02 — Stub-vs-server flavor wording drift (incident `190fdda`)

The `stub_covers_every_mission_in_the_registry` test only verified that
`stub_verdict` returned `Some` for every registered mission — it didn't
compare wording to the server. After Agent C's Act-2 batch, the
client-side stub had the right *arms* but slightly different *strings*
than the server, so a player offline saw different flavor than online.
Now covered by `game/tests/contract.rs::server_and_stub_flavor_agree_byte_for_byte`,
which boots both graders against every canonical solution and asserts
ok flag + stdout/stderr (after stripping `[stub] `) match exactly.

## 2026-05-02 — Stale test binaries in shared cargo target dir

Not a code bug; a workflow trap. The repo's `.cargo/config.toml` points
the build dir to `G:/cargo-target/pledgeandcrown`. When parallel-agent
worktrees all share that target dir, one worktree's compiled test
binaries can be picked up by `cargo test` invocations from another
worktree — including the main checkout. Symptom: a test asserts
"server == stub" but the in-memory stub strings come from a different
worktree's compiled `.exe`, so the test fails on main even though main
sources are consistent.

**Workaround until properly fixed:** before merging a worktree branch,
clean the affected test binaries:

```powershell
Get-ChildItem G:/cargo-target/pledgeandcrown/debug/deps/contract-*.exe `
              G:/cargo-target/pledgeandcrown/debug/deps/stub_grader-*.exe `
              -EA SilentlyContinue | Remove-Item -Force
Get-ChildItem G:/cargo-target/pledgeandcrown/debug/.fingerprint/pledge_compile_api-* `
              G:/cargo-target/pledgeandcrown/debug/.fingerprint/pledge_and_crown-* `
              -EA SilentlyContinue | Remove-Item -Recurse -Force
```

Or just `cargo clean -p pledge_compile_api -p pledge_and_crown` (slower —
~20 GB rebuild).

**Real fix (landed 2026-05-02):** `scripts/ci.ps1` now detects worktree
checkouts (`.git` is a file pointer rather than a directory) and pins
`CARGO_TARGET_DIR` to `G:/cargo-target/pledgeandcrown-wt-<basename>`
for the duration of the script. Main keeps the shared default. Pre-commit
hooks invoke ci.ps1, so any agent commit gets isolation for free. Caveat:
agents that run raw `cargo test` outside the script don't get isolation —
they should set the env var themselves or invoke `scripts/ci.ps1`.

## 2026-06-18 — wasm_runner disabled baseline features rustc actually emits

The harness didn't just miss this bug — it *asserted* it. `wasm_runner`'s
wasmtime `Config` disabled `bulk_memory` and `reference_types` as
"defence in depth," with a comment claiming "the student curriculum
compiles plain Rust to wasm32-wasip1 and exercises none of these
proposals." That premise is false: it's not the student snippet, it's the
**std** the linker pulls in — current rustc lowers `memcpy`/`memset` to
`memory.copy`/`memory.fill` (bulk-memory) and emits the reference-types
table encoding. Both are WebAssembly 2.0 baseline. With them disabled,
wasmtime's decoder mis-parses and rejects EVERY real Rust module with
`Invalid input WebAssembly code ... zero byte expected`.

It was never caught because nothing built a *real* rustc wasm and ran it
through `wasm_runner` — the wasm-exec wiring was deferred, and the
existing `wasm_runner` tests synthesized hand-written WAT (which used none
of these features). Worse, `rejects_disabled_bulk_memory_feature` actively
asserted that a `memory.copy` module must be **rejected** — encoding the
bug as a guarantee. The first end-to-end test that compiled `fn main() {
println!("hello"); }` and ran it surfaced it immediately.

**Fix (landed 2026-06-18):** `wasm_runner::run_wasm` now enables
`bulk_memory` + `reference_types` (baseline, required by rustc output) and
keeps the genuinely-exotic / concurrency-bearing proposals off (SIMD,
relaxed-SIMD, multi-memory, memory64, tail-call, threads, gc,
function-references). The deny-list regression guard was re-pointed from
bulk-memory (now correctly allowed) to SIMD:
`compile-api/tests/wasm_runner.rs::accepts_baseline_bulk_memory_feature`
proves a real `memory.copy` module runs, and `rejects_disabled_simd_feature`
proves a `v128` module is still refused. The honest end-to-end guard is
`compile-api/tests/wasm_builder.rs` (`#[ignore]`d, runs real
`cargo build --target wasm32-wasip1` then executes the artifact).

**Lesson:** a "defence in depth" deny-list is only safe if you actually
exercise the real inputs against it. A synthetic-WAT test asserting a
feature is denied is worthless if no real toolchain output ever flows the
same path — it just freezes a wrong assumption into a green check.
