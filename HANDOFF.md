# HANDOFF.md

## Last Updated
2026-04-25 (handoff — TESS authoritative GREEN; art deliverables spec drafted; cleared to register domain + GH org + file 1(b) ITU)

## Project Status
🟡 In progress — design package complete, Bevy 0.18 workspace scaffolded and compiling green through full local CI (fmt/check/clippy/test), palette redesigned around a defensible color-theory structure ("Heraldic Code"). No art assets yet. Working title locked as **Pledge & Crown**.

## What Was Done This Session (2026-04-25 evening)

- **TESS sweep on "Pledge & Crown" — AUTHORITATIVE GREEN.** Matt ran the search in his real Chrome and downloaded the result xlsx (broad sweep, 500-record cap, search term `pledge & crown`). Hard findings: **zero marks anywhere in USPTO contain both "PLEDGE" and "CROWN"**; 5 LIVE PLEDGE-prefix marks in IC 009/041 (none gaming — Port Nexus enterprise software, Indonesian sports foundation, Texas fitness DBA, TN consulting); 52 LIVE CROWN-containing marks in IC 009/041 (23 are bare CROWN — Crown Battery, Crown Audio, Crown Equipment, Crown Financial Ministries, Crown Melbourne casino, Amazon CROWN, etc.; 29 are differently-prefixed compounds). Under 2(d) likelihood-of-confusion analysis the leading distinctive qualifier "PLEDGE &" gives strong commercial-impression separation from bare CROWN and from any other compound. RF Trademark Policy clean (neither word on protected-mark list). **Cleared to:** register `pledgeandcrown.dev`, create GitHub org, file 1(b) ITU in IC 009 + IC 041, commission logo lockup and marketing. Earlier note about "indirect/indexer triangulation" is superseded — authoritative source agrees with indexers. Source xlsx files: `C:\Users\Matt\Downloads\tmSearchResults2026-04-25.xlsx` (50-row narrow) + ` (1).xlsx` (500-row broad).
- **`design/04b-art-deliverables.md` written.** Companion to bible v2.0 + handoff-prompts v1.0 — covers the gaps identified for handing the visual art to a contractor: master asset manifest (~180 deliverables for MVP, prioritized A/B/C, mapped to Acts 1–2 + Borrow Checker), file format spec (PNG-32 / sRGB / single-frame / pivot rules), authoritative animation frame-count table, filename + folder convention promoted out of CLAUDE.md, approval workflow (2-round reference gate, batches of 20, palette-conformance auto-rejection), IP/licensing stance (work-for-hire default + portfolio carve-out), explicit out-of-scope (audio, marketing, 3D).

## Earlier This Session — Initial Audit + Scaffold

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
- **Bevy 0.18 workspace boots and passes full CI.** `game/` has tracing-instrumented main + lib + Camera2d. `compile-api/` is a stub bin that proves the workspace compiles end-to-end. Pinned versions: bevy 0.18, bevy_egui 0.39, bevy_ecs_tilemap 0.18, bevy_kira_audio 0.25, egui_code_editor 0.2.
- **Local CI fully wired.** `scripts/ci.ps1` (fmt + check + clippy -D warnings + test) runs on every commit via `scripts/pre-commit.sh` → `.git/hooks/pre-commit`. Cold full run ~12 min on i9-11900K; warm runs ~5 sec.
- **Palette v2.0 ("Heraldic Code")** locked. 32 colors derived from a named harmonic structure (split-complementary @ 350° anchor + 140°/178° pair, +42° gold accent, +270° magic flicker). Every color has a role name; prompts reference roles, not raw hexes. Sweep across 5 prompt files replaced ~97 color references; zero residual EDG32 hexes.

**Not yet done (expected):**
- ~~Bevy project scaffold.~~ done 2026-04-25 (commit d7fa659).
- Style bible reference image generation (10 images, REF-01..REF-10 in `design/04-art-handoff-prompts.md`).
- Compile-API prototype on Hetzner (currently a stub).
- Vertical slice (Acts 1–2 + Borrow Checker boss).
- Game subsystems: world, party, combat, editor, compile_client, sandbox, town, persistence, audio. Currently only a Camera2d on a blank window.

**Stubbed / pending decisions:**
- Repo directory renamed `J:\cargoandcrowns\` → `J:\pledgeandcrowns\` (plural, not the singular `pledgeandcrown` originally planned — keep this in mind when reading older notes/skills that may still point at the old path). Internal product/crate naming (`Pledge & Crown`, `pledge-and-crown`, `pledge_and_crown`) is unchanged.
- Open questions 2, 3, 4, 5 in HANDOFF.md still need Matt's call (leaderboards, licensing, distribution platform, Temple-in-marketing).

## Blocking Issues

- ~~**Authoritative TESS confirm on "Pledge & Crown"**~~ — done 2026-04-25 evening, GREEN. No remaining trademark blocker.
- **Open design questions 2–5** — leaderboards in MVP, licensing model (recommendation: dual-license), distribution platform priority, Temple-in-marketing. Each is design-impacting; needs Matt's call.
- **No GitHub remote** yet — local-only repo. Org creation pending TESS clearance.

## What's Next

Prioritized for the incoming session:

1. **TESS + RF policy final clearance on "Pledge & Crown"** — required before any public asset, domain registration, or repo rename. ~30 minutes.
2. **Decide remaining open questions** (HANDOFF §"Open questions" 2–5) — leaderboards in MVP, licensing model, distribution platform priority, Temple-in-marketing. Each is design-impacting.
3. ~~**Repo rename**~~ — done 2026-04-25: `J:\cargoandcrowns\` → `J:\pledgeandcrowns\`. Audit any external references (skills, scheduled tasks, wiki entries) that still point at the old path.
4. **Day 1 of critical path** — register `pledgeandcrown.dev` (or alt), create GitHub org. ~~scaffold `scripts/ci.ps1` + pre-commit hook~~ done 2026-04-25 (commits 6838264, ceaed94). Hook is installed in `.git/hooks/pre-commit`; `ci.ps1` is a no-op until `Cargo.toml` exists. PS 5.1 gotcha logged: scripts must be ASCII-only (no em-dashes) because PS 5.1 reads UTF-8-without-BOM as ANSI.
5. **Day 2 — style bible reference image gen** — 10 images per `design/04-art-handoff-prompts.md`. Do not bulk-gen until the 10 are approved. Prompts already use the new Heraldic Code palette by role name — no further prompt edits needed before gen.
6. ~~**Day 3 onward** — Bevy scaffold~~ — done 2026-04-25 (Bevy 0.18 scaffolded, not 0.14 as the doc anticipated).

## Notes for Next Session

- **Read this file first.** That's a global Matt rule and applies here.
- **The 8 audit findings are reflected in the docs** — do not re-diagnose them. If anything in `05-tech-architecture.md` §2 (compile-time RCE defense) feels under-specified, that's a real concern; if it feels redundant, that's the audit being thorough.
- **Pledge & Crown is the title.** Internal-only until TESS clears. Don't push it to any public surface yet.
- **The Borrow Checker still exists** — as the Act 2 boss. Don't rename the boss along with the game.
- **Bevy is now pinned to 0.18.** Doc text in `05-tech-architecture.md` was written against 0.14 and references some old API patterns (e.g. window resolution as floats — Bevy 0.18 only accepts u32). Treat the doc's code samples as intent, not literal. The workspace `Cargo.toml` is the source of truth for versions.
- **PS 5.1 ASCII gotcha** (logged from CI scaffolding): Windows PowerShell 5.1 reads UTF-8-without-BOM as ANSI/Windows-1252, which garbles em-dashes and breaks parsing of `.ps1` files. All scripts in `scripts/` are ASCII-only. Don't add em-dashes or smart quotes to .ps1 files.
- **Pre-commit hook is real CI.** It runs cargo fmt/check/clippy/test on every commit and blocks bad commits. To bypass for a WIP, use `git commit --no-verify` — but per global rules, don't unless you have to.
- **Commit messages with embedded double-quotes** must be passed via `git commit -F <file>` on Windows (PowerShell heredocs don't escape `"` cleanly through to git's argv parser). Single-quoted arg names like `"Heraldic Code"` in a commit message will be misparsed.
- **Wraith MCP wired into this project** as of 2026-04-25 in `~/.claude.json` → `projects['J:/pledgeandcrowns'].mcpServers['wraith-browser']` (stdio, `J:/wraith-browser/target/release/wraith-browser.exe serve`). **Will not load until next session restart.** Backup of pre-patch config at `~/.claude.json.bak-20260425-180938`. If the `serve` subcommand isn't right, that's where to fix it — `args` array in the JSON.
- **Wraith GPU smoke test in progress on Matt's side.** When that completes and Wraith is verified working, restart this session and the `mcp__wraith*` tools will surface; first task should be the TESS sweep on "Pledge & Crown" (exact phrase + classes 9 and 41, live + dead, plus a Rust Foundation Trademark Policy sanity check).
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
