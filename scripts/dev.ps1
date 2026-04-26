# Run the game locally with verbose tracing.
# powershell -ExecutionPolicy Bypass -File scripts/dev.ps1

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$env:RUST_LOG = "pledge_and_crown=debug,bevy=info,wgpu=warn,naga=warn"
$env:RUST_BACKTRACE = "1"

Write-Host "=== pledge & crown - dev run ===" -ForegroundColor Cyan
Write-Host "RUST_LOG = $($env:RUST_LOG)"

cargo run -p pledge_and_crown --features dev
