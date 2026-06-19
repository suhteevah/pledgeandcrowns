# Pledge & Crown — mobile / Tauri 2.0 wrapper

This directory holds the **Tauri 2.0** shell that wraps the wasm web build
(`web/`) into a native app. It exists so the same WebGL2 build that runs in
a browser can ship as an Android (and, on a Mac, iOS) bundle without a
second renderer. Desktop targets should keep using the **native Bevy
binary** (`cargo run -p pledge_and_crown`) — a webview wrapper is strictly
worse than native wgpu there; Tauri's job here is *mobile*.

## How it fits together

```
game/  --wasm32-unknown-unknown + wasm-bindgen-->  web/   (pkg/ + assets/ + index.html)
                                                     |
                                          frontendDist points here
                                                     v
mobile/src-tauri/  --tauri-->  Android .apk / .aab   (or desktop .exe via WebView2)
```

The wrapper serves **prebuilt static assets** — there is no dev server and
no `beforeBuildCommand` that invokes a JS bundler. The build order is:

1. `powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1`
   (produces `web/pkg/*.wasm` + `web/assets/` + `web/index.html`)
2. `cargo tauri build` from `mobile/src-tauri/` (bundles `../../web`)

## Toolchain status on kokonoe (2026-06-18)

| Need | Status |
|------|--------|
| `cargo tauri` (tauri-cli) | **2.10.0 — present** |
| node / npm | **24.13 / 11.8 — present** (only needed for `cargo tauri icon` / optional tooling) |
| WebView2 runtime (desktop) | **present** (Edge WebView 148/149) → desktop wrapper builds + runs |
| Android SDK + NDK | **installing at `G:\android`** (Matt-managed; was mid-update 2026-06-19). `ANDROID_HOME` / `NDK_HOME` still unset → set them to the `G:\android` paths once the update settles, then the Android bundle unblocks |
| iOS toolchain | N/A — iOS bundling requires macOS + Xcode; not possible on this Windows box |
| Rust toolchain for the desktop link | **MSVC required** — see below |

## Windows toolchain gotcha: build the wrapper with MSVC, not GNU

The whole game workspace builds fine with this box's **default** toolchain,
`stable-x86_64-pc-windows-gnu`. The Tauri shell does **not**: the GNU
linker (`x86_64-w64-mingw32 ld`) chokes on webview2's COM bindings with
`export ordinal too large` — webview2 generates more exports than GNU `ld`
can emit. This is a known Tauri-on-Windows constraint; Tauri officially
targets MSVC on Windows.

Build the wrapper (only) with the MSVC toolchain — the rest of the repo
stays on GNU. The one catch is *which* MSVC install: see the box note
below. The ergonomic entry point is the helper:

```
mobile\build-desktop.bat                     REM loads the right MSVC env, builds the shell
mobile\build-desktop.bat --release           REM optimized
```

`rustup override set stable-x86_64-pc-windows-msvc` inside `mobile/` also
works (scopes MSVC to this dir without disturbing the GNU default the game
uses), as long as the MSVC env (`LIB`/`PATH`) is loaded first. Not pinned
via `rust-toolchain.toml` because that would wrongly fix the host triple
for the future macOS/Android builds too.

### Verified state on kokonoe (2026-06-18) — desktop build WORKS

The wrapper **builds and links to a runnable `app.exe`** (37 MB PE32+) via
the **VS BuildTools** MSVC environment. Confirmed end-to-end:
`Finished dev profile ... in 1m 38s`.

Getting there surfaced a real toolchain trap worth recording:

- **GNU** (`stable-x86_64-pc-windows-gnu`, the box default) cannot link the
  shell: `ld.exe: error: export ordinal too large` — webview2 emits more
  exports than GNU `ld` handles. Use MSVC for the wrapper.
- **The box has *two* VS 2022 installs, and only one is complete.** VS
  **Community** has an incomplete C++ workload — OneCore CRT libs only
  (`lib\onecore\x64`), no desktop `lib\x64\msvcrt.lib`/`libcmt.lib`, and no
  `vcvarsall.bat`. VS **BuildTools** is complete (full desktop CRT + a real
  `vcvarsall.bat`). `vswhere -latest` returns Community (the broken one);
  **`vswhere -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64`
  returns BuildTools** (the working one). `build-desktop.bat` queries by
  that component, which is why it links where a naive `-latest` does not.

So nothing needs installing for the **desktop** build — it works today via
`build-desktop.bat`. (If you ever see `could not open 'msvcrt.lib'`, you're
on the Community install; point the env at BuildTools.)

## The Android bundle: SDK lives at `G:\android` (Matt-managed)

The Android bundle is the actual point of this wrapper. The SDK/NDK is
**not absent** — Matt keeps it at **`G:\android`** (it was mid-update on
2026-06-19, so the dir may not be fully populated until that settles). Do
**not** auto-install a second copy; point the env at the existing one.

1. Confirm the SDK is settled (update finished): `G:\android` should hold
   `platform-tools\`, `platforms\`, `build-tools\`, and `ndk\<version>\`.
2. Set environment to the existing SDK (NOT `%LOCALAPPDATA%`):
   - `ANDROID_HOME` = `G:\android`
   - `NDK_HOME` = `G:\android\ndk\<version>`  (pick the installed version)
   ```
   setx ANDROID_HOME "G:\android"
   setx NDK_HOME "G:\android\ndk\<version>"
   ```
   (new shell required for `setx` to take effect)
3. Add the Rust Android targets:
   ```
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```
4. Initialize + build:
   ```
   cd mobile/src-tauri
   cargo tauri android init
   cargo tauri android build        # or: cargo tauri android dev   (needs a device/emulator)
   ```

Until the env is pointed at `G:\android` and the targets are added, the
wrapper is verified on the **desktop** target only (WebView2), which proves
the scaffold + `frontendDist -> ../../web` wiring loads the game in a
webview end-to-end.

## Verifying the desktop wrapper (works today)

```
powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1   # build web/ first
cd mobile/src-tauri
cargo tauri build --no-bundle      # compiles the shell; or `cargo tauri dev` to launch it
```

## Notes

- `mobile/src-tauri/Cargo.toml` declares its own empty `[workspace]` so it
  is NOT pulled into the root cargo workspace (`game` / `compile-api` / the
  two `tools/*` crates). Keeping it standalone avoids dependency-resolution
  coupling and keeps the root `cargo build --workspace` fast.
- The wasm payload is large (~40 MB `*_bg.wasm` + ~20 MB of baked audio).
  For an Android release, enable wasm-opt in `scripts/web-build.ps1` and
  consider trimming/compressing the audio beds before bundling.
- Per CLAUDE.md the Tauri path is owned by Matt's `tauri-frontend` skill;
  this README is the runbook, not a replacement for it.
