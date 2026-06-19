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
| Android SDK | **present at `G:\AndroidSdk`** (Matt-managed) — platform-tools, `android-36.1`, build-tools 36.1.0/37.0.0, emulator, license accepted |
| Android **NDK** | **MISSING** — no `G:\AndroidSdk\ndk\`, no `cmdline-tools`/`sdkmanager`. **This blocks the Android bundle** (Tauri needs the NDK to cross-compile Rust). `ANDROID_HOME`/`NDK_HOME` also unset. See finishing steps below. |
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

## The Android bundle: base SDK at `G:\AndroidSdk`, NDK still missing

The Android bundle is the actual point of this wrapper. The base SDK is
**present** at **`G:\AndroidSdk`** (Matt-managed; verified 2026-06-19, last
written 2026-05-22) — do **not** auto-install a second copy. What it has:
`platform-tools\` (adb), `platforms\android-36.1`, `build-tools\{36.1.0,37.0.0}`,
`emulator\`, and an accepted `licenses\android-sdk-license`.

What it does **not** have yet — and Tauri's Android build requires both:
- **`ndk\<version>\`** — Tauri cross-compiles Rust to the Android ABIs with
  the NDK's clang+linker. This is the real blocker; without it
  `cargo tauri android build` fails.
- **`cmdline-tools\latest\`** (`sdkmanager`) — the CLI used to install the
  NDK and accept its license.

(Ignore the stale `E:\android\Android SDK` — it's a 2015-era android-22 SDK.)

### Finishing it

1. Install the NDK + cmdline-tools into the existing SDK (pick ONE):
   - **Android Studio** (already installed) → Settings → *Languages &
     Frameworks → Android SDK* → set the SDK path to `G:\AndroidSdk` →
     *SDK Tools* tab → check **NDK (Side by Side)** + **Android SDK
     Command-line Tools** → Apply.
   - **CLI**: drop the Google `commandlinetools-win-*.zip` into
     `G:\AndroidSdk\cmdline-tools\latest\`, then
     `sdkmanager --sdk_root=G:\AndroidSdk "ndk;<version>" "platform-tools"`
     and accept the NDK license.
2. Set environment to the existing SDK (NOT `%LOCALAPPDATA%`):
   - `ANDROID_HOME` = `G:\AndroidSdk`
   - `NDK_HOME` = `G:\AndroidSdk\ndk\<version>`  (the version you installed)
   ```
   setx ANDROID_HOME "G:\AndroidSdk"
   setx NDK_HOME "G:\AndroidSdk\ndk\<version>"
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

Until the NDK is installed and the env is pointed at `G:\AndroidSdk`, the
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
