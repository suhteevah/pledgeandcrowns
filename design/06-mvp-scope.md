# 06 — MVP Scope (30-Day Vertical Slice)

> The MVP is a free, web-playable demo covering Acts 1–2. It ends with the Borrow Checker boss encounter. Players who clear it get a "more coming" screen with email signup. Everything else in the design doc is post-MVP.

## What's in MVP

### Content
- ✅ Act 1: Hearthstone Village — full
- ✅ Act 2: Borrow Bridge — full, including The Borrow Checker boss
- ✅ "End of demo" screen with signup
- ✅ Title screen and main menu
- ✅ Save/load (single auto-slot, no manual saves yet)
- ✅ Settings (volume, difficulty toggle: Story/Strict)

### Systems
- ✅ Cast Editor with `bevy_egui` + Rust syntax highlighting
- ✅ Compile API (deployed on Hetzner)
- ✅ Wasmtime sandbox (client-side)
- ✅ Compiler error rendering with line highlighting
- ✅ Tracing/logging (verbose by default in dev, info in release)
- ✅ Auto-save on zone transition

### Platforms
- ✅ Web (WASM build deployed to itch.io and a custom subdomain)
- ⬜ Desktop binary (Windows + Mac + Linux) — nice-to-have but not blocking
- ❌ Mobile — post-MVP

## What's deliberately NOT in MVP

- All Acts 3 through 11.
- Town building.
- Party system (single character only in MVP).
- Crafting.
- Cliphy hint system (the imp arrives in v1.0).
- Cloud saves.
- Achievements / leaderboards.
- Audio polish beyond placeholder bgm + 4 sfx.
- Localization (English only).
- Anything that smells like a feature creep request.

## Day-by-day plan

### Week 1 — Foundations

**Day 1.** Trademark check on the four working titles **plus Rust Foundation Trademark Policy review** (see HANDOFF.md — "Cargo & Crowns" is flagged out by RF policy on the "Cargo" mark). Pick winner. Register `.dev` domain. Create GitHub org + repo. Set up CI **locally** — `scripts/ci.ps1` runs `cargo check + clippy + test`, wired to a pre-commit hook and a self-hosted runner. **No GitHub Actions** (banned per global instructions).

**Day 2.** Style bible reference image generation pass. All 10 reference images. Iterate until Matt approves the look.

**Day 3.** Bevy project scaffold. Hello world: window opens, background color is right, pixel font loads, FPS counter shows. `tracing_subscriber` configured. CI green.

**Day 4.** `bevy_egui` + `egui_code_editor` integration. Code editor opens on a key press, shows Rust syntax highlighting, "CAST" button is visible (no-op for now).

**Day 5.** Compile API skeleton on Hetzner. Single endpoint, no caching yet. `cargo build` runs in a tempdir, returns WASM bytes. Deploy script tested. End-to-end: client sends "fn main() { println!(\"hello\"); }", server returns WASM, client logs it. No execution yet.

### Week 2 — The pipeline

**Day 6–7.** Wasmtime client-side sandbox. Day 6: loads WASM, calls function, returns value, fuel + memory caps enforced, basic timeouts. Day 7: adversarial pass — infinite loop, OOM, panic, deep recursion, large allocations — verify all are caught and reported cleanly. (Originally one day; widened — sandbox hardening is not a one-day job.)

**Day 8.** Encounter scaffolding system. Each encounter is a struct that defines: harness wrapping, expected return type, test cases, success criteria. Encounter "act_1_count_logs" exists and is solvable end-to-end.

**Day 9.** First three Act 1 encounters: Firewood, Bell Tower, Chicken Coop. Code editor opens, player solves, world reacts. Manual playtest: it feels good. **Day 9 also covers** sprite rendering: player character moves on a tilemap, 4-directional walk, camera follows. (Compressed — these are independent enough to parallel.)

**Day 10.** Hearthstone Village environment. Tilemap built, NPCs placed, dialogue boxes work (no code editor for these yet, just chatter).

### Week 3 — The bridge

**Day 11.** Bandit Sparring Dummy combat encounter. First combat. Health bars render. Damage numbers float. Death animation.

**Day 12.** Act 1 wrap. Bell Tower at sundown completes Act 1. Transition to Act 2 zone.

**Day 13.** Borrow Bridge environment. Stone tilemap, the Riftwater Gorge backdrop. Borrow Checker NPC sprite placed at center.

**Day 14.** First Move encounter. Player is forced to deal with move semantics. Borrow Checker dialogue system in place.

**Day 15.** Two Travellers encounter. `&` references introduced.

**Day 16.** The Mutable Quill encounter. `&mut` introduced. The "one mutable XOR many immutable" rule has been concretely demonstrated.

**Day 17.** The Lifetime Lantern encounter. Explicit `'a` lifetimes. This is the hardest tutorial puzzle in the demo.

**Day 18.** Slack day for Week 3 overruns. (Almost certain to be needed.)

### Week 4 — Boss, polish, launch

**Day 19.** The Borrow Checker boss encounter, stage 1. Four-stage code puzzle scaffolded. Stage 1 playable.

**Day 20.** Stages 2–4 of boss encounter.

**Day 21.** Boss flow: lose / retry / final reward. Borrowstone item awarded.

**Day 22.** "End of demo" screen. Email capture form. Wire up to ConvertKit or similar. Privacy policy in place.

**Day 23.** Audio pass. Placeholder BGM (free or licensed loop), 4 sfx (UI click, attack hit, level transition, boss-defeated sting).

**Day 24.** Compile API caching. Redis added. Cache hit ratio observable in Grafana.

**Day 25.** Polish day. Easings on UI, screen-shake on hit, particle effects on cast.

**Day 26.** Web build. WASM size optimization (`lol_alloc` — note: **not** `wee_alloc`, which is unmaintained — `--no-default-features` audit, `wasm-opt -Oz`, `--release` with `lto = "fat"` and `codegen-units = 1` in `[profile.release]`). Target: <8MB initial download.

**Day 27.** itch.io page set up. Screenshots taken. Description written. Tags applied. Devlog post drafted.

**Day 28.** Internal playtest with three external testers (Matt's choice — likely Cory at First Choice Plastics, plus two Rust-community contacts). Bug bash.

**Day 29.** Bug fixes from playtest. Final web deploy.

**Day 30.** **Launch day.** Post on r/rust, Hacker News (Show HN), Bluesky, Twitter, Rust gamedev Discord. Reach out to a handful of Rust YouTubers (Jon Gjengset, No Boilerplate, Ryan Levick) with a short personal note and a free key for the eventual paid version.

## Definition of done for MVP

A new player who has never opened the game can:

1. Land on the itch.io page.
2. Click "Run game in browser" and have it load in <15 seconds.
3. See a title screen, hit "New Game."
4. Be guided through Hearthstone Village by Ferris.
5. Open the Cast Editor, write `fn count_logs(pile: i32) -> i32 { pile + 1 }`, and watch their character chop wood.
6. Cross the village to the Borrow Bridge.
7. Be challenged by the Borrow Checker, attempt and fail a few times, learn from real rustc errors shown in the Compiler Output panel, and ultimately succeed.
8. Receive the Borrowstone, see the "End of demo" screen, sign up for the mailing list.

If those eight steps work, MVP is done. **Anything beyond that is v1.0 territory.**

## Risk register

| Risk                                         | Mitigation                                              |
|----------------------------------------------|---------------------------------------------------------|
| Compile API latency feels bad                | Aggressive caching, pre-warm common solutions, optimistic UI ("compiling..." with humor) |
| WASM sandbox security surprise               | Pin wasmtime, follow CVE feed, no `wasi`                |
| Bevy 0.15 release lands mid-build            | Pin 0.14, defer migration                               |
| Art generation produces inconsistent results | Reference image gate, batch review process              |
| Player can't solve Borrow Checker boss       | "Show me the solution" after 3 attempts, with explanation |
| Hetzner box dies on launch day               | Have AWS Lightsail $5 instance ready as cold standby; DNS TTL low |
| 30 days slips                                | Cut scope: drop the desktop binary, keep web only       |

## Post-launch (Days 31–60)

Reserved for: bug fixes from real-world telemetry, the "we read your feedback" devlog, beginning Act 3 work, and starting to monetize through the email list. Detailed v1.0 plan lives in `07-monetization.md`.
