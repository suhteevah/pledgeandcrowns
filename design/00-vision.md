# 00 — Vision

## The pitch in one paragraph

*Pledge & Crown* is a pixel-art RPG where the only way to act on the world is to write real, compilable Rust. Combat is code. Crafting is code. Town-building is code. The Borrow Checker is an NPC who guards a bridge and rejects your code with the exact errors `rustc` would produce. The 10-act campaign covers a complete university-level Rust curriculum — variables to async to unsafe to FFI — and the post-game opens *The Temple*, a HolyC roguelike where Rust's safety rules don't apply and your code can (and will) segfault on purpose. By the time a player finishes the campaign and survives a Temple run, they are a junior-to-mid Rust developer.

## Why this exists

Rust has the steepest learning cliff of any mainstream language. The community knows it. The Rust Foundation knows it. Every "I tried Rust and bounced" blog post lists the same enemies: ownership, lifetimes, the trait system. Existing solutions split into two camps:

1. **Books and tutorials** (the official Book, *Rust in Action*, *Programming Rust*, Rustlings). High quality, low engagement. Most users abandon mid-ownership chapter.
2. **Programming games** (CodeCombat, Mimo, Grasshopper, Zachtronics titles). High engagement, but none teach Rust. CodeCombat does JS/Python/Lua. Mimo's Rust support is a thin tutorial reader. Zachtronics teaches fictional assembly languages.

There is no third option. We are the third option.

## Audience

**Primary:** Adults aged 22–40 who write code professionally or are studying CS, who have heard "Rust is the future" and tried to learn it once. They have an attention budget that ranges from "thirty minutes after work" to "a Saturday afternoon." They have spending power and will pay $20–$40 for a quality educational game.

**Secondary:** University CS students using the game as a supplement to a systems-programming course. Department licensing potential here.

**Tertiary:** Engineering teams onboarding Rust at $50/seat. The "Rust onboarding" market is growing fast as more companies (Cloudflare, Discord, Microsoft, Linux Foundation) bring Rust into production.

**Explicitly not the audience:** Children. CodeCombat owns that. Going head-to-head with K-12 EdTech is a losing fight, and the curriculum we want to teach (lifetimes, unsafe, FFI) isn't appropriate for kids anyway.

## Why now

- Rust adoption has crossed the inflection point. Linux kernel inclusion, Windows kernel inclusion, AWS Firecracker, Cloudflare Workers, and a generation of new browsers (Servo, Wraith) have made Rust unavoidable for serious systems work.
- The Rust Survey 2025 consistently identifies "learning curve" as the top barrier.
- Indie game distribution (itch.io, Steam Direct) is healthier than ever for educational niches.
- Generative art tools have made indie pixel-art production economically viable for a solo studio.
- LLM-assisted code review and explanation make personalized in-game tutoring feasible at scale.

## Why this team can ship it

The owner ships Rust at scale (294,710-line ClaudioOS, 27,000-line Wraith Browser, an MCP fleet). The infrastructure to host the compile-API backend exists (OpenClaw, Hetzner). The mobile wrap path is known (Tauri 2.0). The art-generation pipeline is being defined here. Every gap maps to an existing skill, not a learning curve.

## Success criteria

**MVP (Day 30):** 1,000 demo plays, 200 email signups, 50 organic Reddit/HN comments, 10 critical pieces of feedback used to inform v1.0.

**v1.0 (Day 90):** Acts 1–4 shipped, $5,000 gross sales, 5,000 wishlists on Steam, written up by at least one Rust-focused outsiders (a Reddit thread doesn't count; a blog post or podcast does).

**v1.5 (Day 180):** Acts 1–7 shipped, $25,000 gross, first B2B inquiry from a company asking about team licenses.

**v2.0 (Day 365):** Full 10-act campaign + Temple roguelike, $100,000 gross, sustainable creator economy where Matt can choose to keep building this or roll the proceeds into the next product.

## Non-goals

- We are not building a Rust compiler. The Rust team has that handled.
- We are not building a general programming-education platform. We do Rust, beautifully and deeply, and that's it.
- We are not building a multiplayer MMO. Single-player + async leaderboards is the ceiling for v1.
- We are not making a children's game. The Temple includes profanity (in moderation) and references to memory corruption, segfaults, and the historical context of HolyC. Rated T at minimum.

## Cultural commitments

- The Borrow Checker is treated with respect. Players will hate it. The game lets them hate it productively, then love it, the way real Rust devs do.
- HolyC and TempleOS are tributed for their genuine technical achievements (a one-person OS, a one-person compiler). Terry Davis's later public statements are not endorsed and not referenced. The Temple appendix is technical homage, not personality cult.
- The Rust ecosystem's prevailing values — inclusivity, technical rigor, reliability — show up in our writing, our error messages, and our community presence.
