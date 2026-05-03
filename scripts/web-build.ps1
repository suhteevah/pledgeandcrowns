# WASM web build pipeline.
# Requires: rustup target add wasm32-unknown-unknown; cargo install wasm-bindgen-cli.
# powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== pledge & crown - web build ===" -ForegroundColor Cyan

cargo build -p pledge_and_crown --target wasm32-unknown-unknown --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Cargo target dir comes from `.cargo/config.toml` (currently G:/cargo-target/...).
# Resolve it via `cargo metadata` so this script doesn't drift from the config.
$targetDir = (cargo metadata --no-deps --format-version 1 |
              ConvertFrom-Json).target_directory
$wasmIn = Join-Path $targetDir "wasm32-unknown-unknown/release/pledge_and_crown.wasm"
if (-not (Test-Path $wasmIn)) {
    Write-Host "[FAIL] wasm artifact not found at $wasmIn" -ForegroundColor Red
    exit 1
}

$pkgOut = "web/pkg"
New-Item -ItemType Directory -Force -Path $pkgOut | Out-Null
wasm-bindgen --out-dir $pkgOut --target web $wasmIn
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (Test-Path "game/assets") {
    Copy-Item -Recurse -Force "game/assets" "web/assets"
}

# Minimal index.html if one doesn't exist yet. Players hit web/index.html.
if (-not (Test-Path "web/index.html")) {
    @'
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Pledge & Crown</title>
  <style>
    html, body { margin: 0; padding: 0; background: #161313; overflow: hidden; }
    canvas { display: block; image-rendering: pixelated; margin: 0 auto; }
  </style>
</head>
<body>
  <canvas id="bevy"></canvas>
  <script type="module">
    import init from "./pkg/pledge_and_crown.js";
    init();
  </script>
</body>
</html>
'@ | Set-Content -Encoding UTF8 web/index.html
    Write-Host "[ok] wrote web/index.html starter" -ForegroundColor Green
}

Write-Host "[ok] web build at web/. Serve with: cargo run -p web-serve --release" -ForegroundColor Green
