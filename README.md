# Pledge & Crown
*A Rust-teaching RPG with a HolyC roguelike endgame.*

**Working title:** Pledge & Crown. *Pledge* is the medieval term for an item left as collateral on a loan — etymologically the same concept the Rust borrow checker enforces. Title selected after the prior working title ("Cargo & Crowns") was flagged out by Rust Foundation Trademark Policy. **Still required before public marketing:** USPTO TESS search on "Pledge & Crown" + RF policy re-check (the term "Pledge" is generic; "Crown" is generic; the conjunction should be clear, but verify).

**Tagline (working):** "The only RPG where the compiler is a boss."

---

## What this is

A pixel-art RPG that teaches a complete university-level Rust curriculum across 10 main acts, with an unlockable HolyC roguelike post-game (Act 11, *The Temple*). Combat, crafting, and town-building are all driven by the player writing real, compilable Rust. Bad code = damage taken. Elegant code = critical hits. The Borrow Checker is a literal NPC.

**Target audience:** CS students, working developers transitioning to Rust, hobbyists who bounced off Rustlings. Adult-skewed (CodeCombat is 62% under-18 — that's the gap).

**Market position:** No direct competitor exists for "RPG that teaches production Rust." CodeCombat does JS/Python/Lua. Mimo and Grasshopper are tutorial apps, not games. Zachtronics games (Shenzhen I/O, Exapunks) teach fictional assembly. The "Hands-On Rust" book teaches Rust by *building* a roguelike — we're the inverse.

---

## Repo layout

```
pledge-and-crown/
├── README.md                       <- you are here
├── CLAUDE.md                       <- working context for Claude Code
├── HANDOFF.md                      <- current status & next actions
├── design/
│   ├── 00-vision.md                <- concept, audience, market gap
│   ├── 01-curriculum.md            <- Acts 1-10, full Rust mapping (the heart)
│   ├── 02-mechanics.md             <- combat, building, progression systems
│   ├── 03-art-style-bible.md       <- locked visual constraints
│   ├── 04-art-handoff-prompts.md   <- copy-paste prompts for image generation
│   ├── 05-tech-architecture.md     <- Bevy + WASM + Tauri stack details
│   ├── 06-mvp-scope.md             <- 30-day vertical slice plan
│   ├── 07-monetization.md          <- revenue path
│   └── 08-temple-appendix.md       <- Act 11 HolyC roguelike spec
└── prompts/
    ├── characters.md               <- per-character generation prompts
    ├── tilesets.md                 <- environment/tile prompts
    ├── ui.md                       <- HUD, menus, code editor frame
    └── bosses.md                   <- The Borrow Checker, etc.
```

---

## Read order

If you only read three docs: **00-vision → 01-curriculum → 06-mvp-scope**.

If you're starting on art today: **03-art-style-bible → 04-art-handoff-prompts → prompts/characters.md**.

If you're starting on code today: **05-tech-architecture → 06-mvp-scope → CLAUDE.md**.

---

## Quick-start commands (when scaffolded)

These don't exist yet. The `06-mvp-scope.md` doc enumerates exactly what to scaffold first. When the Bevy project is initialized, populate this section:

```powershell
# Windows dev (Matt's primary)
cd pledge-and-crown/game
cargo run --features dev
```

```powershell
# Web build. The free web / itch / mobile build IS the demo (24 missions):
powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1           # demo (default)
powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1 -Full     # full game, on purpose
# Each build writes web/pkg/build-stamp.json (flavour, features, commit,
# date, .wasm hash). web-package.ps1, itch-push.ps1 and the Tauri shell
# read it and refuse a full build unless given -Full
# (PNC_ALLOW_FULL_BUILD=1 for the Tauri build).
powershell -ExecutionPolicy Bypass -File scripts/web-package.ps1         # dist/pledge-and-crown-web-demo.zip
```

```powershell
# Native release build. A release exe never defaults to the local dev
# server: it uses PLEDGE_API_URL at run time, else the URL baked in at
# build time from PLEDGE_BAKED_API_URL, else offline mode (pattern-checked
# passes count, labelled as such). Debug builds default to 127.0.0.1:7878.
$env:PLEDGE_BAKED_API_URL = "https://<compile-api host>"   # optional
cargo build --release -p pledge_and_crown --features demo
# Logs: %APPDATA%\pledge-and-crown\pledge-and-crown.log (no console window
# in a Windows release build).
```

```powershell
# Mobile wrapper (Tauri)
cd pledge-and-crown/mobile
cargo tauri android dev
```

---

## Status

**Phase:** Design complete, no code yet.
**Next action:** See `HANDOFF.md`.

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

---

## Support This Project

If you find this project useful, consider buying me a coffee! Your support helps me keep building and sharing open-source tools.

[![Donate via PayPal](https://img.shields.io/badge/Donate-PayPal-blue.svg?logo=paypal)](https://www.paypal.me/baal_hosting)

**PayPal:** [baal_hosting@live.com](https://paypal.me/baal_hosting)

Every donation, no matter how small, is greatly appreciated and motivates continued development. Thank you!
