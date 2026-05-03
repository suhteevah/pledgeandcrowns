# Pledge and Crown - Local CI
# Runs: cargo fmt --check, cargo check, clippy (warnings as errors), tests.
# GitHub Actions is BANNED per global rule. This is the only CI.
# Invoke: powershell -ExecutionPolicy Bypass -File scripts/ci.ps1

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== Pledge and Crown CI ===" -ForegroundColor Cyan
Write-Host "repo: $repoRoot"

# Worktree isolation. The shared `.cargo/config.toml` points
# `target-dir` at G:/cargo-target/pledgeandcrown so every checkout
# would otherwise stomp on the same compiled test binaries — bit us
# this session when a worktree's edited stub_grader.rs got linked
# into a binary that main's `cargo test` then ran against, causing
# the byte-parity contract test to fail on consistent main sources.
# Fix: in a worktree (`.git` is a file pointer, not a dir), pin
# CARGO_TARGET_DIR to a per-worktree subdir so they can't collide.
$gitMeta = Get-Item .git -ErrorAction SilentlyContinue
if ($gitMeta -and -not $gitMeta.PSIsContainer) {
    $wtName = Split-Path -Leaf $repoRoot
    $env:CARGO_TARGET_DIR = "G:/cargo-target/pledgeandcrown-wt-$wtName"
    Write-Host "[worktree] CARGO_TARGET_DIR -> $env:CARGO_TARGET_DIR" -ForegroundColor Yellow
}

if (-not (Test-Path "Cargo.toml")) {
    Write-Host "[skip] No Cargo.toml yet - Rust scaffold not in place. CI is a no-op." -ForegroundColor Yellow
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
