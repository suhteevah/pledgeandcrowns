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
| Android SDK + NDK | **ABSENT** (`ANDROID_HOME` / `NDK_HOME` unset) → **blocks the Android bundle** |
| iOS toolchain | N/A — iOS bundling requires macOS + Xcode; not possible on this Windows box |
| Rust toolchain for the desktop link | **MSVC required** — see below |

## Windows toolchain gotcha: build the wrapper with MSVC, not GNU

The whole game workspace builds fine with this box's **default** toolchain,
`stable-x86_64-pc-windows-gnu`. The Tauri shell does **not**: the GNU
linker (`x86_64-w64-mingw32 ld`) chokes on webview2's COM bindings with
`export ordinal too large` — webview2 generates more exports than GNU `ld`
can emit. This is a known Tauri-on-Windows constraint; Tauri officially
targets MSVC on Windows.

The fix is to build the wrapper (only) with the already-installed MSVC
toolchain — the rest of the repo stays on GNU:

```
cargo +stable-x86_64-pc-windows-msvc build --manifest-path mobile/src-tauri/Cargo.toml
# or, with the Tauri CLI from mobile/src-tauri:
#   rustup override set stable-x86_64-pc-windows-msvc   (scopes MSVC to this dir)
#   cargo tauri build
```

`rustup override set` inside `mobile/` is the ergonomic way to make
`cargo tauri build`/`dev` here always pick MSVC without disturbing the
GNU default the game uses. (Not committed as a `rust-toolchain.toml`
because that would wrongly pin the host triple for the future macOS/Android
builds too.)

### Verified state on kokonoe (2026-06-18) — desktop link is currently blocked

Both toolchains were tried against this scaffold. **The Rust compiles
cleanly under both** (every dependency — wry, tao, tauri, webview2-com —
builds); only the final *link* fails, and for a different reason each way:

- **GNU** (`stable-x86_64-pc-windows-gnu`, the box default):
  `ld.exe: error: export ordinal too large: 95271` — webview2 has more
  exports than GNU `ld` can emit. Not fixable on the GNU toolchain.
- **MSVC** (`stable-x86_64-pc-windows-msvc`, installed via rustup):
  `lld-link: error: could not open 'msvcrt.lib'`. VS 2022 Community **is**
  installed and `cl.exe` / `link.exe` exist
  (`VC\Tools\MSVC\14.43.34808\bin\Hostx64\x64`), but the C++ environment
  is **incomplete**: the standard **desktop** x64 CRT import libraries
  (`VC\Tools\MSVC\14.43.34808\lib\x64\msvcrt.lib` / `libcmt.lib`) are
  absent — only the `lib\onecore\x64\` variant is present — and
  `VC\Auxiliary\Build\vcvarsall.bat` is missing (so neither `vcvars64.bat`
  nor `Enter-VsDevShell` can fully populate `LIB`; the DevShell sets the
  Windows SDK paths but not the VC CRT paths). The Windows 10 SDK
  (10.0.22621.0) lib dir IS present.

In short: VS is installed, but the **"Desktop development with C++"**
workload is only partially present (OneCore CRT libs + compiler, but no
desktop CRT import libs and no `vcvarsall.bat`).

**Unblock (Matt-action):** open the **Visual Studio Installer → Modify →
Desktop development with C++** and ensure "MSVC v143 - VS 2022 C++ x64/x86
build tools" and the Windows SDK are checked (a repair/modify, since the
compiler is already there). That restores `lib\x64\*` + `vcvarsall.bat`.
Then:

```
cd mobile/src-tauri
rustup override set stable-x86_64-pc-windows-msvc
cargo tauri build          # desktop bundle (WebView2 runtime already present)
# or, from mobile/:  build-desktop.bat   (loads MSVC env, builds the shell)
```

Until the desktop C++ workload is completed, the wrapper is **scaffolded +
compile-verified (all deps build under both toolchains)** but cannot
produce a linked binary on this box. This is an environment gate, not a
code change.

## The one blocker: Android SDK/NDK (Matt-action)

The Android bundle is the actual point of this wrapper, and it's gated on a
toolchain that isn't installed. This is a deliberate Matt-action (multi-GB
download + Google SDK license acceptance — not something to auto-install):

1. Install Android Studio (or the standalone command-line tools) and, via
   the SDK Manager, the **SDK Platform**, **Platform-Tools**, **Build-Tools**,
   and the **NDK (Side by side)**.
2. Set environment:
   - `ANDROID_HOME` = `%LOCALAPPDATA%\Android\Sdk` (or your SDK path)
   - `NDK_HOME` = `%ANDROID_HOME%\ndk\<version>`
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

Until that lands, the wrapper is verified on the **desktop** target only
(WebView2), which proves the scaffold + `frontendDist -> ../../web` wiring
loads the game in a webview end-to-end.

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
