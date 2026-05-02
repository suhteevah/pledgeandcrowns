# Licensing

Pledge & Crown is dual-licensed.

| Component | License | File |
|-----------|---------|------|
| `game/` crate (game binary, gameplay code, plugins, assets) | MIT | [LICENSE-MIT](LICENSE-MIT) |
| `compile-api/` crate (server-side compile-to-WASM API) | AGPL-3.0-or-later | [LICENSE-AGPL](LICENSE-AGPL) |

## Why dual-license

- **MIT on the game.** Steam's distribution agreement restricts copyleft on game binaries, and players who buy the game on Steam, itch.io, GOG, or Apple/Google should not have their devices subject to a viral license. MIT keeps the binary distributable everywhere.
- **AGPL on the compile-API server.** The compile-API is the moat. AGPL prevents a competitor from running our server-side curriculum infrastructure as a hosted service without contributing improvements back. This matches the licensing pattern used by [Wraith Browser](https://github.com/suhteevah/wraith-browser).

## What the AGPL means for players

The AGPL only governs the **server**, not the player's submitted code. Anything a player writes in the in-game editor and ships to the compile API remains theirs under whatever license they choose. The AGPL applies to the server source code, not to user-generated content the server processes.

## Contributing

By submitting a pull request, you agree that your contribution is licensed under both MIT and AGPL-3.0-or-later, matching the dual-license structure of the repository. A formal Contributor License Agreement (CLA) will be added before the project accepts external contributions; until then, all contributions come from Ridge Cell Repair LLC.

## Third-party dependencies

See `Cargo.lock` for the full transitive dependency graph and `cargo about` (run on demand) for the per-dependency license report. We avoid GPL-2-only and SSPL dependencies in `game/` to preserve MIT compatibility.
