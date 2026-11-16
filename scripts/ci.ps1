# Pledge and Crown - Local CI
# Runs: cargo fmt --check, cargo check, clippy (warnings as errors), tests,
# the demo-feature tests, the rustc claim checks (hard rule 3), and a
# wasm32 `cargo check` of the game crate (if the target is installed).
# The remaining #[ignore]d suites run in scripts/ci-slow.ps1 (release gate).
# GitHub Actions is BANNED per global rule. This is the only CI.
# Invoke: powershell -ExecutionPolicy Bypass -File scripts/ci.ps1

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== Pledge and Crown CI ===" -ForegroundColor Cyan
Write-Host "repo: $repoRoot"

# Worktree isolation. The shared `.cargo/config.toml` points
# `target-dir` at G:/cargo-target/pledgeandcrown so every checkout
# would otherwise stomp on the same compiled test binaries - bit us
# this session when a worktree's edited stub_grader.rs got linked
# into a binary that main's `cargo test` then ran against, causing
# the byte-parity contract test to fail on consistent main sources.
# Fix: in a worktree (`.git` is a file pointer, not a dir), pin
# CARGO_TARGET_DIR to a per-worktree subdir so they can't collide.
# -Force: worktree `.git` files are often created Hidden, and Get-Item
# without -Force silently returns nothing for hidden items (isolation
# then never engaged and worktrees raced on the shared target dir).
$gitMeta = Get-Item .git -Force -ErrorAction SilentlyContinue
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
# The demo build (Acts 1-2 only) is a shipping configuration; keep it compiling.
Run-Step "check-demo" "cargo check -p pledge_and_crown --features demo --all-targets"
# ...and run its tests: the demo branches of the gating tests only execute
# under the feature. Warm: seconds; after a game-crate change it rebuilds
# the game once more with `demo` (~2.5 min at CARGO_BUILD_JOBS=3).
Run-Step "test-demo" "cargo test -p pledge_and_crown --features demo --test mission_text --test registry"
# Hard rule 3: compile and run every NPC compiler claim, starter and
# pinned output against rustc (~1 min). The tests first assert rustc is
# the server's pinned release (game/tests/common/mod.rs) and fail with the
# fix if not. The rest of the slow suites (solutions, contract,
# compile-api) are in scripts/ci-slow.ps1, which a release tag requires.
Run-Step "rustc-claims" "cargo test --workspace --test mission_text -- --ignored --test-threads=4"

# Web-only cfg(target_arch = "wasm32") code is never compiled by the
# native steps above, so it rots silently. Check the game crate for the
# web target too. Skipped (loudly) if the target is not installed.
$installedTargets = @(rustup target list --installed 2>$null)
if ($installedTargets -contains "wasm32-unknown-unknown") {
    Run-Step "check-wasm" "cargo check -p pledge_and_crown --target wasm32-unknown-unknown"
} else {
    Write-Host ""
    Write-Host "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" -ForegroundColor Yellow
    Write-Host "[WARN] wasm32-unknown-unknown target NOT installed - web cfg code" -ForegroundColor Yellow
    Write-Host "       was NOT checked. Fix: rustup target add wasm32-unknown-unknown" -ForegroundColor Yellow
    Write-Host "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== CI passed ===" -ForegroundColor Green
