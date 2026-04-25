# Pledge & Crown — Local CI
# Runs: cargo fmt --check, cargo check, clippy (warnings as errors), tests.
# Per global rules: GitHub Actions is BANNED. This is the only CI.
# Invoke: powershell -ExecutionPolicy Bypass -File scripts/ci.ps1

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== Pledge & Crown CI ===" -ForegroundColor Cyan
Write-Host "repo: $repoRoot"

if (-not (Test-Path "Cargo.toml")) {
    Write-Host "[skip] No Cargo.toml yet — Rust scaffold not in place. CI is a no-op." -ForegroundColor Yellow
    exit 0
}

function Run-Step {
    param([string]$Name, [string]$Cmd)
    Write-Host ""
    Write-Host "--- $Name ---" -ForegroundColor Cyan
    Write-Host "> $Cmd"
    Invoke-Expression $Cmd
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[FAIL] $Name (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "[ok] $Name" -ForegroundColor Green
}

Run-Step "fmt"    "cargo fmt --all -- --check"
Run-Step "check"  "cargo check --workspace --all-targets"
Run-Step "clippy" "cargo clippy --workspace --all-targets -- -D warnings"
Run-Step "test"   "cargo test --workspace"

Write-Host ""
Write-Host "=== CI passed ===" -ForegroundColor Green
