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

**Real fix (deferred):** give each worktree its own `CARGO_TARGET_DIR`
under the worktree itself, e.g. via a per-worktree `.cargo/config.toml`
override, so worktrees can never clobber each other's compiled tests.
Open work — file an issue or write the override script before the next
parallel-agent batch.
