# WASM web build pipeline.
# Requires: rustup target add wasm32-unknown-unknown; cargo install wasm-bindgen-cli.
# Optional: wasm-opt (binaryen) on PATH, or WASM_OPT=<path to wasm-opt.exe>.
#
#   powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1                 # the DEMO (default)
#   powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1 -Full           # the full game
#   powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1 -CargoProfile release
#   powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1 -SkipWasmOpt
#
# The free web / itch / mobile build IS the demo (product decision,
# 2026-11-15): `-Features` defaults to `demo` (24 missions, then the
# end-of-demo panel). Building the whole game takes an explicit -Full.
# A feature list without `demo` and without -Full is refused.
#
# Output: web/pkg/ (wasm-bindgen glue + .wasm), web/assets/ (copy of
# game/assets) and web/pkg/build-stamp.json, which records the flavour
# (demo/full), features, commit, date, profile and .wasm hash. Packaging
# and shipping scripts read that stamp and refuse a full build unless
# they too are given -Full (see scripts/build-stamp-lib.ps1).
# web/index.html is the hand-written loader page and is checked in; this
# script never overwrites it.
#
# Server URL: the web build reads window.PLEDGE_CONFIG from the page (none
# set = offline mode, pattern-checked passes count). The build-time
# variable PLEDGE_BAKED_API_URL only applies to NATIVE release builds
# (`$env:PLEDGE_BAKED_API_URL = "https://..."; cargo build --release -p
# pledge_and_crown --features demo`); see compile_client/config.rs.

param(
    # Cargo profile. `wasm-release` (root Cargo.toml) is size-tuned:
    # opt-level "s", fat LTO, 1 CGU, strip, panic=abort.
    [string]$CargoProfile = "wasm-release",
    # Cargo features, comma- or space-separated. Default: the demo.
    [string]$Features = "demo",
    # Build the full game (drops the default `demo` feature).
    [switch]$Full,
    [switch]$SkipWasmOpt
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot
. (Join-Path $PSScriptRoot "build-stamp-lib.ps1")

# --- which game ----------------------------------------------------------
$featureList = @($Features -split '[,\s]+' | Where-Object { $_ -ne "" })
if ($Full) {
    if ($PSBoundParameters.ContainsKey("Features") -and ($featureList -contains "demo")) {
        Write-Host "[FAIL] -Full contradicts -Features '$Features' (it contains demo)." -ForegroundColor Red
        exit 2
    }
    $featureList = @($featureList | Where-Object { $_ -ne "demo" })
} elseif (-not ($featureList -contains "demo")) {
    Write-Host "[FAIL] -Features '$Features' has no 'demo': that is the FULL game. Pass -Full to build it on purpose." -ForegroundColor Red
    exit 2
}
$flavour = if ($featureList -contains "demo") { "demo" } else { "full" }
$featureArg = ($featureList -join ",")
Write-Host ("[info] building the {0} (features: '{1}')" -f $(if ($flavour -eq "demo") { "DEMO" } else { "FULL GAME" }), $featureArg) -ForegroundColor $(if ($flavour -eq "demo") { "Green" } else { "Magenta" })

function Get-GzipSize {
    param([string]$Path)
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $ms = New-Object System.IO.MemoryStream
    $gz = New-Object System.IO.Compression.GZipStream($ms, [System.IO.Compression.CompressionLevel]::Optimal)
    $gz.Write($bytes, 0, $bytes.Length)
    $gz.Close()
    $len = $ms.ToArray().Length
    $ms.Dispose()
    return $len
}

function Write-SizeLine {
    param([string]$Label, [string]$Path)
    $raw = (Get-Item $Path).Length
    $gz = Get-GzipSize -Path $Path
    Write-Host ("[size] {0,-22} raw {1,8:N2} MB   gzip {2,8:N2} MB" -f $Label, ($raw / 1MB), ($gz / 1MB))
}

Write-Host "=== pledge & crown - web build (profile: $CargoProfile, $flavour) ===" -ForegroundColor Cyan

$buildStart = Get-Date
$cargoArgs = @("build", "-p", "pledge_and_crown", "--target", "wasm32-unknown-unknown", "--profile", $CargoProfile)
if ($featureArg -ne "") { $cargoArgs += @("--features", $featureArg) }
Write-Host "> cargo $($cargoArgs -join ' ')"
& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host ("[ok] cargo build finished in {0:N0}s" -f ((Get-Date) - $buildStart).TotalSeconds) -ForegroundColor Green

# Cargo target dir comes from `.cargo/config.toml` (or CARGO_TARGET_DIR).
# Resolve it via `cargo metadata` so this script doesn't drift from the config.
$targetDir = (cargo metadata --no-deps --format-version 1 |
              ConvertFrom-Json).target_directory
# Cargo puts the built-in `release` profile in `release/`, custom
# profiles in a directory named after the profile.
$profileDir = $CargoProfile
if ($CargoProfile -eq "dev") { $profileDir = "debug" }
$wasmIn = Join-Path $targetDir "wasm32-unknown-unknown/$profileDir/pledge_and_crown.wasm"
if (-not (Test-Path $wasmIn)) {
    Write-Host "[FAIL] wasm artifact not found at $wasmIn" -ForegroundColor Red
    exit 1
}
Write-SizeLine -Label "cargo output" -Path $wasmIn

$pkgOut = "web/pkg"
if (Test-Path $pkgOut) { Remove-Item -Recurse -Force $pkgOut }
New-Item -ItemType Directory -Force -Path $pkgOut | Out-Null
wasm-bindgen --out-dir $pkgOut --target web --no-typescript $wasmIn
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$wasmOut = Join-Path $pkgOut "pledge_and_crown_bg.wasm"
Write-SizeLine -Label "after wasm-bindgen" -Path $wasmOut

# --- wasm-opt (binaryen) -------------------------------------------------
# Never downloaded by this script: installing binaryen is an owner step
# (e.g. `cargo install wasm-opt`, or a binaryen release on PATH).
if ($SkipWasmOpt) {
    Write-Host "[skip] wasm-opt pass skipped (-SkipWasmOpt)." -ForegroundColor Yellow
} else {
    $wasmOpt = $null
    if ($env:WASM_OPT -and (Test-Path $env:WASM_OPT)) {
        $wasmOpt = $env:WASM_OPT
    } else {
        $cmd = Get-Command wasm-opt -ErrorAction SilentlyContinue
        if ($cmd) { $wasmOpt = $cmd.Source }
    }
    if (-not $wasmOpt) {
        Write-Host "[WARN] wasm-opt not found (PATH / WASM_OPT) - shipping un-optimised wasm. Owner step: install binaryen." -ForegroundColor Yellow
    } else {
        Write-Host "[info] wasm-opt: $wasmOpt"
        $optOut = Join-Path $pkgOut "pledge_and_crown_bg.opt.wasm"
        # Feature flags match what rustc's wasm32-unknown-unknown target
        # emits by default (Rust 1.82+); without them binaryen rejects
        # the module during validation.
        & $wasmOpt -Oz `
            --enable-bulk-memory --enable-nontrapping-float-to-int `
            --enable-sign-ext --enable-mutable-globals `
            --enable-reference-types --enable-multivalue `
            -o $optOut $wasmOut
        if ($LASTEXITCODE -ne 0 -or -not (Test-Path $optOut)) {
            Write-Host "[WARN] wasm-opt failed (exit $LASTEXITCODE) - keeping un-optimised wasm." -ForegroundColor Yellow
            if (Test-Path $optOut) { Remove-Item -Force $optOut }
        } else {
            Move-Item -Force $optOut $wasmOut
            Write-SizeLine -Label "after wasm-opt -Oz" -Path $wasmOut
        }
    }
}

# --- assets ---------------------------------------------------------------
if (Test-Path "game/assets") {
    # Copy-Item -Recurse into an EXISTING directory nests the source as
    # web/assets/assets/... instead of refreshing web/assets/ in place.
    # That silently serves STALE assets (and strands new ones, e.g.
    # newly-baked audio) on every rebuild after the first. Clear the
    # destination first so each build copies fresh.
    if (Test-Path "web/assets") { Remove-Item -Recurse -Force "web/assets" }
    Copy-Item -Recurse -Force "game/assets" "web/assets"
    # .wav masters (left behind by `audio-to-ogg.ps1 -KeepWav`) are not
    # referenced by the game and kira is built without its wav decoder.
    $strayWav = @(Get-ChildItem -Path "web/assets" -Recurse -Filter *.wav)
    foreach ($w in $strayWav) { Remove-Item -Force $w.FullName }
    if ($strayWav.Count -gt 0) {
        Write-Host "[warn] dropped $($strayWav.Count) .wav file(s) from web/assets - run scripts/audio-to-ogg.ps1" -ForegroundColor Yellow
    }
    $assetBytes = (Get-ChildItem -Path "web/assets" -Recurse -File | Measure-Object -Property Length -Sum).Sum
    Write-Host ("[ok] refreshed web/assets from game/assets ({0:N2} MB)" -f ($assetBytes / 1MB)) -ForegroundColor Green
}

if (-not (Test-Path "web/index.html")) {
    Write-Host "[FAIL] web/index.html (the loader page) is missing - restore it from git." -ForegroundColor Red
    exit 1
}

$stampPath = Write-BuildStamp -WebDir (Resolve-Path "web").Path -Flavour $flavour -Features $featureArg -CargoProfile $CargoProfile
Write-Host "[ok] wrote $stampPath" -ForegroundColor Green
Show-BuildStamp -Stamp (Read-BuildStamp -WebDir (Resolve-Path "web").Path)

Write-Host "[ok] web build at web/. Serve with: cargo run -p web-serve --release" -ForegroundColor Green
