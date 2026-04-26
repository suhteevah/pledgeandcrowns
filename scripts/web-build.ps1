# WASM web build pipeline.
# Requires: rustup target add wasm32-unknown-unknown; cargo install wasm-bindgen-cli.
# powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== pledge & crown - web build ===" -ForegroundColor Cyan

cargo build -p pledge_and_crown --target wasm32-unknown-unknown --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$wasmIn = "target/wasm32-unknown-unknown/release/pledge_and_crown.wasm"
$pkgOut = "web/pkg"

New-Item -ItemType Directory -Force -Path $pkgOut | Out-Null
wasm-bindgen --out-dir $pkgOut --target web $wasmIn
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (Test-Path "game/assets") {
    Copy-Item -Recurse -Force "game/assets" "web/assets"
}

Write-Host "[ok] web build at web/. Serve with: python -m http.server -d web" -ForegroundColor Green
