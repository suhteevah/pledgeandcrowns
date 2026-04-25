# 07 — Monetization

## Strategy in one paragraph

Free demo (Acts 1–2) drives top-of-funnel via itch.io and the Rust community. Paid full game ($24.99 single, $14.99 student-discounted, $49.99 deluxe with HolyC Temple early-access) on Steam and itch.io once Acts 1–4 ship. B2B team licensing ($50/seat, minimum 5 seats) for companies onboarding Rust devs as the long-tail revenue. Total addressable market: small but devout. Beat the existing Rust learning resources on engagement, not on price.

## Revenue model: phased

### Phase 0 — MVP (Days 1–30)
**Goal:** validation, not revenue. Prove the concept works.

- Free demo on itch.io and a custom subdomain.
- Email capture for "notify me when v1.0 ships."
- No charge. No premium tier. No Patreon. Don't dilute the launch with money asks.

**Targets:** 1,000 demo plays, 200 email signups, 50 organic engaged comments across r/rust, HN, Bluesky.

### Phase 1 — Early access (Days 31–90)
**Goal:** first dollars. Validate willingness-to-pay.

- Acts 1–4 shipped on itch.io as **paid early access at $14.99**.
- Email list gets a 24-hour head start at $9.99.
- Steam wishlist page goes live concurrently.
- Free demo (Acts 1–2) remains available, with a clear "buy to continue past Act 2" call-to-action.

**Targets:** 350 sales × $14.99 = $5,200 gross. ~$3,800 net after fees.

### Phase 2 — Steam launch (Days 91–180)
**Goal:** mainstream visibility. Steam's discoverability is unmatched.

- Acts 1–7 on Steam at **$24.99**.
- Launch discount of 15% for first 7 days.
- itch.io price aligns ($24.99) but with itch's lower fee (10% vs Steam's 30%) we keep more from itch sales.
- Soundtrack DLC on Steam at $4.99.
- Community Discord active. Modding API teased.

**Targets:** 1,500 sales × $24.99 = $37,500 gross. ~$25,000 net after Steam's 30% and refunds.

### Phase 3 — Full release (Days 181–365)
**Goal:** the game is the business.

- Acts 1–10 + Temple shipped on Steam, itch.io, GOG.
- Standard edition $24.99, Deluxe (with Temple early-access + soundtrack + art book) $39.99.
- Price drops: $19.99 student price (Steam student verification), $14.99 third-world regional pricing.
- B2B licensing program quietly launches: $50/seat, minimum 5 seats, includes a "team progress dashboard" for engineering managers tracking onboarding.

**Targets:** 4,000 retail sales × $26 average = $104,000 gross. ~$70,000 net.

Plus: 200 B2B seats at $50 = $10,000 gross. Net ~$9,500.

**Year 1 total:** ~$110,000 net revenue if all phases land near targets. Conservative case: $40,000. Stretch case: $250,000 if a Rust YouTube creator with 100K+ subscribers does a feature.

## Pricing rationale

- **$24.99 is the indie sweet spot.** Below this, players treat the game as a casual app. Above this, single-purchase resistance grows quickly.
- **Educational positioning supports premium pricing.** People who would balk at $25 for a casual game will pay $25 for "the game that taught me Rust."
- **Free demo lowers the bar to evaluation.** The Acts 1–2 demo is genuinely 2–3 hours of content. People who finish it have invested enough to buy.
- **B2B at $50/seat is below market for technical training.** A typical Pluralsight team plan is $35/user/month. We're a one-time $50 with no subscription. Easy yes for engineering managers.

## Channels and distribution

| Channel                 | Why                                                  | Effort | Revenue share |
|-------------------------|------------------------------------------------------|--------|---------------|
| itch.io                 | Indie default. Loyal audience. Lowest fees.          | Low    | 90% (configurable up to 100%) |
| Steam                   | Discoverability. Reviews. Wishlists.                 | Medium | 70%           |
| GOG (Phase 3)           | DRM-free crowd. Smaller but devoted.                 | Low    | 70%           |
| Direct (custom site)    | Email list buyers. No platform fees.                 | Medium | 100% (less Stripe ~3%) |
| Apple App Store / Play  | Tauri mobile build. v1.5+ only.                      | Medium | 70% / 70%     |
| Educational license     | Universities. Direct sales.                          | High (per deal) | 100% |

**Refund policy.** Match Steam's 2-hour / 14-day. Generous and standard.

## Marketing channels (what costs $0)

1. **Devlog.** A weekly post on a personal site + cross-posted to dev.to and Hashnode. Topics: "I built a Rust syntax highlighter in egui," "How my Borrow Checker NPC works," "Sandboxing player code with wasmtime." Each post is a marketing artifact disguised as a tech article.
2. **r/rust.** Show HN-style post on launch days. Don't spam. Engage with comments thoughtfully.
3. **Hacker News.** Show HN once for MVP, once for early access, once for Steam launch. Three swings, each genuinely newsworthy.
4. **Rust community Discord and Zulip.** Be present, be helpful, mention the game when relevant — never spam.
5. **YouTube.** Ten-minute "behind the scenes" videos on technical aspects. The Rust audience watches devlog content avidly.
6. **Twitter/X and Bluesky.** Daily art/code progress GIFs. Consistency over volume.
7. **Email list.** Monthly update. Never sell, only inform.

## Marketing channels (small budget, post-revenue)

- **Steam keys for content creators.** Free key + a handwritten note for ~25 Rust-focused creators when v1.0 ships.
- **Reddit Ads in r/rust adjacents.** $200 test budget. Measure CTR. Kill or scale.
- **Sponsoring a small Rust podcast episode.** ~$500. Once the game has revenue.

## Risks to revenue

- **Audience too small.** Rust has fewer learners than Python. Counter: B2B market is growing fastest where Rust adoption is happening (infra, embedded, browsers). Lean into B2B as we scale.
- **Sandbox compromise = trust collapse.** A single security incident with player code escaping the sandbox would kill the brand. Counter: defense in depth, bug-bounty program in v1.0.
- **Bevy migration lands at the wrong time.** Counter: pin and budget.
- **An incumbent enters the space.** CodeCombat could add Rust support. Counter: speed matters; first-mover with a fully designed RPG curriculum has a moat that "we added a language tab" doesn't cross.
- **Player code execution is too slow on low-end hardware.** Counter: server-side compile is the same speed regardless of client; the only cost is sandbox runtime.

## Dovetail with Matt's existing portfolio

- **Wraith Browser** can ship the MVP demo as a "showcased experience" — agent-first browser meets agent-friendly game. Cross-promo natural.
- **OpenClaw infrastructure** hosts the Compile API. Re-uses the deployment skill, the monitoring stack, the VPS model.
- **Job Hunter MCP agent** can identify "Rust developer" job listings; the game is collateral the player can list on their resume ("completed Cargo & Crowns, a 30-hour interactive Rust curriculum").
- **ClaudioOS** can ship a port as the included "demo game" (showcases ClaudioOS's Rust binary compat). Free marketing for both products.

## What we will NOT do for revenue

- **No microtransactions.** Not in this game, not for any reason.
- **No NFTs.** Not in this game, not for any reason.
- **No "energy" systems** that gate play behind waiting or paying.
- **No paywalled hints.** Hints are free, always. The point is teaching.
- **No exclusivity deals** that lock the game to one platform.
- **No advertising in-game.** The game is the product.
- **No data sale.** Telemetry is opt-in and aggregate.

## Long-term: the franchise question

If *Cargo & Crowns* succeeds, the natural sequels are:
- *The Async Frontier* (deep tokio/async ecosystem)
- *Embedded* (no_std, microcontrollers, real hardware integration)
- A non-Rust spinoff in the same universe (e.g., a Zig version, a Go version)

These are v3+ conversations. Not committed, just noting that the IP can extend if it earns the right.
