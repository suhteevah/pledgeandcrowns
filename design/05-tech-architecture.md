# 05 — Tech Architecture

## Stack at a glance

```
┌──────────────────────────────────────────────────────────────┐
│  Client (Bevy 0.14+ ECS)                                     │
│  ├─ rendering: bevy default 2D                               │
│  ├─ UI: bevy_egui + egui_code_editor (Rust syntax)           │
│  ├─ assets: pixel-art sprites + tilemaps via bevy_ecs_tilemap│
│  ├─ audio: bevy_kira_audio                                   │
│  ├─ persistence: serde + bincode → save files                │
│  └─ player code execution: wasmtime sandbox                  │
└──────────────────────────────────────────────────────────────┘
                            │
                            │ HTTPS (player Rust source → WASM)
                            ▼
┌──────────────────────────────────────────────────────────────┐
│  Compile API (Axum)                                          │
│  ├─ POST /compile  { source: String } → { wasm: bytes, ... } │
│  ├─ runs cargo build --target wasm32-unknown-unknown         │
│  ├─ resource limits: time, memory, output size               │
│  ├─ caching: SHA-256 of source → cached WASM (Redis)         │
│  └─ tracing → Prometheus → Grafana                           │
└──────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│  Hetzner CX22 VPS ($7/mo)                                    │
│  ├─ Ubuntu 24.04 LTS                                         │
│  ├─ Caddy reverse proxy (auto-TLS)                           │
│  ├─ Tailscale internal mesh                                  │
│  └─ Provisioned via openclaw-deploy skill                    │
└──────────────────────────────────────────────────────────────┘
```

> **Bevy version pin.** This doc was authored against Bevy 0.14. As of 2026-04-25, verify the current stable Bevy release before scaffolding (Bevy ships ~quarterly) and pin to that. Re-check compatibility for `bevy_egui`, `egui_code_editor`, `bevy_ecs_tilemap`, and `bevy_kira_audio` against whichever Bevy version you pin — these four moving in lockstep with Bevy is a recurring tax.

## 1. Why Bevy

- **Eats own dog food.** A Rust-teaching game written in Rust with the leading Rust game engine is the marketing story. Saves us from writing it in Unity-and-then-explaining-why.
- **ECS architecture is itself teachable.** Optional bonus content: a "Behind the Scenes" section explains how the game uses ECS, which is itself a Rust pattern.
- **Mature 2D pipeline.** `bevy_ecs_tilemap` for tilemaps, `bevy_egui` for UI, `bevy_kira_audio` for sound.
- **WASM target works.** Bevy compiles cleanly to `wasm32-unknown-unknown`, allowing a free web demo with no install friction.
- **Active community.** When (not if) we hit issues, the Bevy Discord is responsive.

**Risks accepted:** Bevy churns. We pin a version, document the upgrade path, and budget a week per major bump.

## 2. Why a server-side compile API for v1

The hardest engineering question in this project is "how do we run player Rust code." Options considered:

| Option                        | Pros                              | Cons                                         | Verdict |
|-------------------------------|-----------------------------------|----------------------------------------------|---------|
| Embed `rustc` in client       | Offline-capable, no backend       | rustc is huge, build times, distribution hell | No      |
| Server-side compile           | Simple, fast iteration, cached    | Requires backend, single point of failure    | **Yes for v1** |
| Use a Rust-like scripting lang| No compile step                   | Defeats the purpose — players need real Rust | No      |
| WASM-bundled `rustc` (rustc.wasm)| Offline, real Rust              | ~80MB download, immature                     | Maybe v2 |
| Pre-compiled solution stubs   | Trivial, fast                     | Players can't write novel solutions          | Fallback only |

**Decision:** Server-side compile API for v1. Aggressive caching makes it fast. Pre-compiled stubs serve as offline fallback for tutorial levels. v2 can investigate WASM-bundled `rustc` once it matures.

### Compile API spec

```
POST /compile
Authorization: Bearer <session-token>
Content-Type: application/json

{
  "source": "fn cast() -> i32 { 42 }",
  "harness": "act_1_chicken_coop_v3",
  "client_id": "uuid-of-player-save"
}

Response 200:
{
  "wasm": "<base64-encoded WASM bytes>",
  "compile_time_ms": 1247,
  "warnings": [...],
  "cached": false
}

Response 400 (compile error):
{
  "error": "compile_failed",
  "stderr": "<full rustc stderr>",
  "spans": [{ "line": 2, "col": 18, "level": "error", "msg": "..." }]
}

Response 429 (rate limited):
{
  "error": "rate_limited",
  "retry_after_seconds": 12
}
```

**Caching.** Three layers, all required to hit the latency target:

1. **Redis source-hash cache.** SHA-256 of `(source + harness + rustc_version)` → cached WASM bytes. Cache hits return in <50ms.
2. **`sccache`** in front of every `cargo build` invocation, backed by local disk on the VPS. Catches "near-miss" compiles where only the player's function body changed.
3. **Per-encounter pre-warmed `target/`.** Each encounter ships with its `target/` already populated by compiling the reference solution at deploy time. Cold-miss compile only rebuilds the player's single source file against an already-built sysroot + dep graph. Without this step, cold misses on a CX22 are 5–30s and the "spell cast" UX falls apart.

At deploy time, every reference solution is also pre-compiled and the resulting WASM bytes are seeded into Redis. First-touch player attempts that happen to match the reference solution return instantly.

**Resource limits.** Each compile job: 30-second wall clock, 512MB memory, output size cap 2MB.

**Rate limiting.** Per-client-id: 60 compiles/min. Per-IP: 600/min. Above limit, 429 response.

**Compile-time RCE defense — the primary control.** Server-side `cargo build` on player-controlled source is a code-execution surface: `proc-macro` crates and `[build-dependencies]` execute arbitrary code at compile time. A container alone is not sufficient — once a malicious proc-macro runs, network egress + privilege escalation are downstream problems.

The primary defense is to **never let player input touch `Cargo.toml`**:

- Each encounter directory on the server is a fixed, pre-built `cargo` project. Server owns `Cargo.toml`, `Cargo.lock`, and any `build.rs`. Dependencies are vendored (`cargo vendor`) at deploy time.
- The compile request includes only the player's **single source file** (e.g. `src/player.rs`). The server writes that file into the encounter project and runs `cargo build --offline --target wasm32-unknown-unknown`.
- `--offline` enforces the vendored set; no registry fetches are possible from a compile job.
- The vendored dep set is curated per encounter. Acts 1–2 use `std` only. Later acts add specific crates (`serde`, `tokio`, etc.) only after each is reviewed.
- `proc-macro` deps are still allowed (they're necessary for `serde`/`tokio`), but the set is fixed and audited — players cannot introduce new ones.

**Container hardening (defense in depth).** `cargo build` runs as a non-root user in a container with no network, ephemeral filesystem, dropped capabilities, seccomp profile, read-only rootfs except for the build dir. If a known-good proc-macro is somehow exploited, the blast radius is one compile job's tempdir.

### Wasmtime client-side sandbox

Returned WASM is loaded with strict limits:

- **Memory cap:** 16MB (overridable per encounter).
- **Fuel cap:** 100M instructions (configurable; bosses get more).
- **No system access:** `wasi_snapshot_preview1` is not provided. The harness exposes only the specific imports the encounter needs (e.g., `read_enemy_stats()`, `deal_damage()`).
- **Async cancellation:** if execution exceeds wall clock, kill the instance and report timeout.

## 3. Why Tauri 2.0 for mobile

Matt's `tauri-frontend` skill applies. Tauri 2.0 ships official iOS and Android support. The web build of our Bevy game runs in a Tauri WebView and gets native packaging. We pay one engineering cost (build the web version well) and get desktop, mobile, and web essentially for free.

The compile API is a network dependency on mobile. Offline mobile play requires the pre-compiled stubs fallback for tutorial levels (acceptable — most mobile play is wifi-adjacent anyway).

## 4. Project structure

```
pledge-and-crown/
├── game/                                 # the main Bevy game
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                       # entry point, sets up tracing
│   │   ├── plugins/
│   │   │   ├── mod.rs
│   │   │   ├── world.rs                  # tilemap, camera, world-state
│   │   │   ├── party.rs                  # PartyMember struct, stats
│   │   │   ├── combat.rs                 # turn loop, damage calc
│   │   │   ├── editor.rs                 # bevy_egui code editor
│   │   │   ├── compile_client.rs         # talks to Compile API
│   │   │   ├── sandbox.rs                # wasmtime instance management
│   │   │   ├── town.rs                   # post-Act-6 town economy
│   │   │   ├── persistence.rs            # save/load
│   │   │   └── audio.rs                  # bgm + sfx
│   │   ├── encounters/
│   │   │   ├── act_01/                   # one file per encounter
│   │   │   ├── act_02/
│   │   │   └── ...
│   │   ├── content/                      # static content tables
│   │   │   ├── enemies.rs
│   │   │   ├── items.rs
│   │   │   └── dialogue.rs
│   │   └── lib.rs
│   ├── assets/
│   │   ├── sprites/
│   │   ├── tilemaps/
│   │   ├── audio/
│   │   └── fonts/
│   └── tests/
│       └── integration_*.rs
├── compile-api/                          # the Axum service
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── routes/
│       ├── compile.rs
│       └── cache.rs
├── mobile/                               # Tauri 2.0 wrapper
│   ├── src-tauri/
│   └── tauri.conf.json
├── web/                                  # WASM build outputs
├── scripts/                              # PowerShell / bash helpers
│   ├── dev.ps1                           # cargo run with tracing
│   ├── web-build.ps1                     # WASM + wasm-bindgen pipeline
│   ├── deploy-compile-api.sh             # Hetzner deploy
│   └── gen-asset-batch.ps1               # batch image-gen helper
├── design/                               # this directory
└── prompts/                              # art generation prompts
```

## 5. Persistence format

Save files are bincode-serialized `Save` structs:

```rust
#[derive(Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    pub created_at: SystemTime,
    pub last_played_at: SystemTime,
    pub player: PartyMember,
    pub party: Vec<PartyMember>,
    pub world: World,
    pub current_act: ActId,
    pub completed_encounters: HashSet<EncounterId>,
    pub spellbook_source: String,    // the actual .rs file the player has built
    pub town: Option<Town>,          // None until Act 6
    pub temple_runs: Vec<TempleRun>, // post-Act-10
    pub stats: PlayerStats,
}
```

Schema migrations live in `persistence.rs` and bump `Save::version` on each format change. v1 saves are upgradable to v2 etc., never downgradable.

## 6. Telemetry & logging

- **Client-side:** `tracing` with a debug-default filter. In release builds, info-default. Logs go to a rolling file in the OS save dir; web build logs to console.
- **Server-side (Compile API):** `tracing` → OTLP → Prometheus. Grafana dashboard provisioned via Matt's `grafana-dashboard` skill. Key metrics: compile request rate, cache hit ratio, p50/p99 compile time, error rate by encounter ID.
- **No PII collected.** Players are identified by an opaque save UUID, never email/name. The optional opt-in telemetry collects only "encounter X took N attempts to pass" — never source code.

## 7. Build & deploy scripts (sketch)

`scripts/dev.ps1`:
```powershell
# Run the game locally with verbose tracing.
$env:RUST_LOG = "pledge_and_crown=debug,bevy=info"
$env:RUST_BACKTRACE = "1"
cd game
cargo run --features dev
```

`scripts/web-build.ps1`:
```powershell
# Full WASM web build pipeline.
$env:RUST_LOG = "info"
cd game
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir ../web/pkg --target web `
  ../target/wasm32-unknown-unknown/release/pledge_and_crown.wasm
Copy-Item -Recurse assets ../web/assets -Force
Write-Host "Web build at ../web/. Serve with: python -m http.server"
```

`scripts/deploy-compile-api.sh`:
```bash
#!/usr/bin/env bash
# Deploy compile API to Hetzner. Requires env: HETZNER_HOST, SSH_KEY.
set -euo pipefail
echo "Building release binary..."
cd compile-api
cargo build --release --target x86_64-unknown-linux-musl
echo "Uploading..."
scp -i "$SSH_KEY" target/x86_64-unknown-linux-musl/release/compile-api \
    "deploy@${HETZNER_HOST}:/opt/compile-api/compile-api.new"
echo "Rotating..."
ssh -i "$SSH_KEY" "deploy@${HETZNER_HOST}" \
  "sudo systemctl stop compile-api && \
   mv /opt/compile-api/compile-api.new /opt/compile-api/compile-api && \
   sudo systemctl start compile-api && \
   sudo systemctl status compile-api --no-pager"
echo "Deployed."
```

## 8. Testing strategy

- **Unit tests** on every encounter's harness (the wrapper around player code) — make sure the test cases are correct.
- **Integration tests** that simulate a player solving each encounter end-to-end. Reference solutions live in `tests/reference_solutions/act_X/encounter_Y.rs` and must compile and pass.
- **Snapshot tests** (insta) for compiler error formatting and dialogue rendering.
- **Property tests** (proptest) for the town economy simulation — invariants like "no resource ever goes negative" must hold under random inputs.

## 9. Open architectural questions

1. **Anti-cheat for leaderboards?** If we ship leaderboards, players will submit hand-crafted WASM that lies about its score. Either accept that the leaderboard is for vibes only, or have leaderboard-eligible runs require server-side re-execution. Decision deferred.
2. **Multiplayer in v2?** A "co-op puzzle" mode where two players collaborate on a single encounter is genuinely cool but a major architectural change. v2 question.
3. **Modding API?** Letting community contribute encounters is an enormous content lever. Requires designing a stable encounter-definition schema. Pencil in for v1.5.
