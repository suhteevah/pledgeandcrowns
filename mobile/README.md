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
| Android **NDK** | **present — r27d `27.3.13750724`** at `G:\AndroidSdk\ndk\27.3.13750724` (installed 2026-06-19 via `install-android-ndk.ps1`); `cmdline-tools\latest` also added. `ANDROID_HOME`/`NDK_HOME` persisted (user scope). |
| Rust android targets | **present** — `aarch64`/`armv7`/`i686`/`x86_64-linux-android` all added |
| **Android APK build** | **WORKS** — `cargo tauri android build` produces an APK end-to-end (verified 2026-06-19). See below. |
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

## The Android bundle — WORKS (verified 2026-06-19)

The Android bundle is the actual point of this wrapper, and it builds
end-to-end. `cargo tauri android build --target aarch64 --apk` produces a
**31 MB `app-universal-release-unsigned.apk`** containing
`lib/arm64-v8a/libapp_lib.so` (the Rust shell with the web bundle embedded by
Tauri) + `AndroidManifest.xml`. The toolchain pieces:

- **SDK** at **`G:\AndroidSdk`** (Matt-managed) — platform-tools, `android-36.1`,
  build-tools 36.1.0/37.0.0, emulator, accepted license. Do **not** install a
  second copy. (Ignore the stale `E:\android\Android SDK` — 2015-era android-22.)
- **NDK** r27d `27.3.13750724` at `G:\AndroidSdk\ndk\27.3.13750724`, plus
  `cmdline-tools\latest`, installed via `mobile/install-android-ndk.ps1`.
- **Env** persisted (user scope): `ANDROID_HOME` / `ANDROID_SDK_ROOT` =
  `G:\AndroidSdk`, `NDK_HOME` / `ANDROID_NDK_HOME` = the NDK path above. JDK 21
  (`C:\Program Files\Java\jdk-21`) drives Gradle.
- **Rust targets**: all four `*-linux-android` triples added.

### Rebuilding / building it (the helper sets env for you)

```
powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 init    # one-time; generates gen/android
powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 build --apk --target aarch64   # arm64 APK
powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 build --apk --aab              # all ABIs + Play bundle
powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 dev                             # install+run on a device/emulator
```

`build-android.ps1` loads `ANDROID_HOME`/`NDK_HOME`/`JAVA_HOME` for its own
process and runs `cargo tauri android <step>` from `src-tauri/`. To set the
env up from scratch on a new box, run `mobile/install-android-ndk.ps1` first.

### Owner-action remaining: signing

`cargo tauri android build` always assembles the **release** variant, and the
APK comes out **unsigned** — it won't install on a device as-is. Two paths,
both yours to own (a signing keystore is a credential — not something this
runbook creates for you):

- **On-device testing:** `...build-android.ps1 dev` runs Gradle's *debug*
  assemble, which auto-signs with the local Android debug keystore and installs
  to a connected device/emulator. No keystore setup needed.
- **Distribution (Play / sideload):** generate a release keystore
  (`keytool -genkeypair ...`), wire it into
  `gen/android/app/build.gradle.kts` (or `key.properties`), and re-run
  `build --aab`/`--apk`. Then upload the signed `.aab` to Play.

Until you sign, the build is **proven** (release-unsigned APK on disk) but not
**installable**; the desktop wrapper (`build-desktop.bat`) remains the runnable
demo target.

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
