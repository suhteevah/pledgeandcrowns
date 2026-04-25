# HANDOFF.md

## Last Updated
2026-04-25 (repo rename + Day 1 local CI scaffold)

## Project Status
🟡 In progress — design package complete and audited; no code or art yet. Working title locked as **Pledge & Crown**.

## What Was Done This Session

- **First-read design audit** — surfaced 8 issues that would have invalidated downstream work; all fixed in-doc:
  1. "Cargo & Crowns" disqualified by Rust Foundation Trademark Policy on the "Cargo" mark.
  2. CI plan (GitHub Actions) contradicted Matt's banned-on-account global rule.
  3. Compile-API spec did not address proc-macro / `[dependencies]` RCE — only `build.rs` and `[build-dependencies]`.
  4. AGPL-3.0 doesn't fit a binary-distributed game (network clause).
  5. Bevy 0.14 pin likely stale at session date.
  6. `wee_alloc` (Day 26 prescription) is unmaintained.
  7. Day 6 wasmtime sandbox + adversarial testing in one day was unrealistic.
  8. Compile-API latency story relied only on Redis source-hash cache; cold-miss `cargo build` on a CX22 is 5–30s.

- **Files updated:**
  - `HANDOFF.md` — RF policy gate, supply-chain RCE risk, dual-license recommendation, fallback-coverage clarified.
  - `CLAUDE.md` — Bevy version note, Hard Rule 2 extended (manifest-control), Hard Rule 6 (naming gate), Hard Rule 7 (no GH Actions).
  - `design/06-mvp-scope.md` — Day 1 swaps GH Actions for `scripts/ci.ps1`+pre-commit; Day 6 widened to Day 6–7; Day 26 swaps `wee_alloc` for `lol_alloc`.
  - `design/05-tech-architecture.md` — Bevy version preamble; three-layer cache (Redis + sccache + pre-warmed `target/` + reference-solution seeding); compile-time RCE defense rewritten as primary control (server-owned `Cargo.toml`, vendored deps, `--offline`, single player file).
  - `design/03-art-style-bible.md` — EDG32 license/credit note added.
  - `design/00-vision.md`, `README.md`, `CLAUDE.md` — title swapped to **Pledge & Crown** across all internal references; layout examples renamed `pledge-and-crown/`; crate name `pledge_and_crown`.

- **Naming decision:** working title locked as **Pledge & Crown** (medieval *pledge* = collateral on a loan, etymologically the Rust borrow-checker concept). The Borrow Checker remains the Act 2 boss's name. "Cargo & Crowns", "Rust & Ruin", and "Ferris's Trial" all rejected on RF Trademark Policy considerations.

## Current State

**Working:**
- Design package coherent and self-consistent across all 9 design docs + 4 prompt files.
- All 8 audit flags reflected in the docs that own them (no orphan TODOs).
- Title selected, etymology documented in CLAUDE.md and 00-vision.md.

**Not yet done (expected):**
- Bevy project scaffold.
- Style bible reference image generation (10 images).
- Compile-API prototype on Hetzner.
- Vertical slice (Acts 1–2 + Borrow Checker boss).

**Stubbed / pending decisions:**
- Repo directory renamed `J:\cargoandcrowns\` → `J:\pledgeandcrowns\` (plural, not the singular `pledgeandcrown` originally planned — keep this in mind when reading older notes/skills that may still point at the old path). Internal product/crate naming (`Pledge & Crown`, `pledge-and-crown`, `pledge_and_crown`) is unchanged.
- Open questions 2, 3, 4, 5 in HANDOFF.md still need Matt's call (leaderboards, licensing, distribution platform, Temple-in-marketing).

## Blocking Issues

None. All blockers from the audit are resolved in-doc. Code-side blockers are unknown until scaffolding starts.

## What's Next

Prioritized for the incoming session:

1. **TESS + RF policy final clearance on "Pledge & Crown"** — required before any public asset, domain registration, or repo rename. ~30 minutes.
2. **Decide remaining open questions** (HANDOFF §"Open questions" 2–5) — leaderboards in MVP, licensing model, distribution platform priority, Temple-in-marketing. Each is design-impacting.
3. ~~**Repo rename**~~ — done 2026-04-25: `J:\cargoandcrowns\` → `J:\pledgeandcrowns\`. Audit any external references (skills, scheduled tasks, wiki entries) that still point at the old path.
4. **Day 1 of critical path** — register `pledgeandcrown.dev` (or alt), create GitHub org. ~~scaffold `scripts/ci.ps1` + pre-commit hook~~ done 2026-04-25 (commits 6838264, ceaed94). Hook is installed in `.git/hooks/pre-commit`; `ci.ps1` is a no-op until `Cargo.toml` exists. PS 5.1 gotcha logged: scripts must be ASCII-only (no em-dashes) because PS 5.1 reads UTF-8-without-BOM as ANSI.
5. **Day 2 — style bible reference image gen** — 10 images per `design/04-art-handoff-prompts.md`. Do not bulk-gen until the 10 are approved.
6. **Day 3 onward** — Bevy scaffold per `design/06-mvp-scope.md`. Verify current stable Bevy at scaffold time; pin and document.

## Notes for Next Session

- **Read this file first.** That's a global Matt rule and applies here.
- **The 8 audit findings are reflected in the docs** — do not re-diagnose them. If anything in `05-tech-architecture.md` §2 (compile-time RCE defense) feels under-specified, that's a real concern; if it feels redundant, that's the audit being thorough.
- **Pledge & Crown is the title.** Internal-only until TESS clears. Don't push it to any public surface yet.
- **The Borrow Checker still exists** — as the Act 2 boss. Don't rename the boss along with the game.
- **Bevy version is intentionally unpinned** in CLAUDE.md and `05-tech-architecture.md`. Doc text was written against 0.14; verify current stable at scaffold time and pin then.
- **Repo is now a git repo** as of 2026-04-25. `main` branch, two commits. No remote yet — GitHub org creation is still pending.
- **Compile API design hard rule:** player input never touches `Cargo.toml`. Server owns the manifest. If a future session designs a "user crates" or "player imports" feature, that's a v2+ conversation — not MVP.
- **GitHub Actions is banned on Matt's account** (global rule). Local CI only. The design docs now reflect this; do not re-introduce GH Actions.

## Critical path to MVP (30 days)

1. **Day 1–2: Trademark + naming.** Working title locked: **Pledge & Crown**. Action: USPTO TESS + final RF policy sanity check, register `pledgeandcrown.dev`, create GitHub org + repo, scaffold local CI (`scripts/ci.ps1` + pre-commit + optional self-hosted runner — **no GitHub Actions**).
2. **Day 3–5: Style bible reference pass.** Use `design/04-art-handoff-prompts.md` to generate the 10 reference images. Lock the look. Do not proceed to bulk asset generation until references are approved.
3. **Day 6–10: Bevy scaffold + server compile API.** Project boots, shows a town tile, opens a `bevy_egui` code editor, ships player code to the compile server, runs WASM in sandbox, returns result. Verbose tracing throughout.
4. **Day 11–18: Act 1 implementation.** Variables, mutability, basic functions. Tutorial village. First two enemies.
5. **Day 19–25: Act 2 implementation.** Ownership, references, lifetimes. Bridge zone. The Borrow Checker boss.
6. **Day 26–28: Polish, juice, sound, web build.**
7. **Day 29: Deploy demo to itch.io as free web build. Email capture.**
8. **Day 30: Launch.** r/rust, Hacker News, Bluesky, Twitter, Rust gamedev Discord.

## Open questions for Matt

1. ~~**Naming.**~~ Resolved 2026-04-25: **Pledge & Crown**.
2. **Leaderboards in MVP?** Single-player only, or async leaderboards from day one? Adds backend dependency, boosts retention.
3. **Licensing.** AGPL-3.0 doesn't fit binary-distributed games. Recommendation: **dual-license** — AGPL+CLA for compile-API server, MIT or proprietary for game binary + assets. Matches Wraith pattern.
4. **Distribution: itch.io demo + Steam paid, or itch.io paid + Steam later?** itch.io ships faster, no review gate.
5. **HolyC Temple in MVP marketing?** Day-one tease frames the long roadmap, or save for surprise reveal?

## Known risks

- **Server-side compile API is a single point of failure.** Mitigations: three-layer cache (Redis source-hash + `sccache` + per-encounter pre-warmed `target/`), pre-compiled stub fallback covering **all 8 MVP encounters**, AWS Lightsail $5 cold-standby with low DNS TTL.
- **Compile-API supply-chain RCE risk.** Hard-locked in `05-tech-architecture.md` §2: server owns `Cargo.toml`, vendored deps, `cargo --offline`, single player file. Container hardening is defense-in-depth, not the primary control.
- **Bevy version churn.** Pin at scaffold time, document upgrade path, budget a week per major bump.
- **Art consistency drift.** Reference-image gate, batch-of-20 review, style bible amendments per pass.
- **Curriculum scope creep.** Doc fixes the curriculum. Resist mid-MVP additions.

## Where to look next

- Curriculum → `design/01-curriculum.md`
- Art → `design/03-art-style-bible.md` and `prompts/`
- Tech → `design/05-tech-architecture.md`
- Compile-API security → `design/05-tech-architecture.md` §2 (compile-time RCE defense)
- "What should I build today" → this file, the critical path above
