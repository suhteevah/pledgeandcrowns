# Pledge and Crown - slow CI (the #[ignore]d suites).
#
#   powershell -ExecutionPolicy Bypass -File scripts/ci-slow.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/ci-slow.ps1 -Toolchain 1.99.0
#   powershell -ExecutionPolicy Bypass -File scripts/ci-slow.ps1 -TestThreads 4
#
# scripts/ci.ps1 (the pre-commit gate) runs the fast tests only. The
# evidence for hard rule 3 (every compiler claim an NPC makes is what rustc
# really says) and for "the server accepts every correct solution" lives in
# #[ignore]d tests that run rustc / real cargo builds hundreds of times.
# ci.ps1 runs the quickest of them (mission_text --ignored, ~1 min, the
# rustc claim checks). This script runs every one of them:
#
#   game:        mission_text (starters, pinned outputs, hint claims, trial /
#                Act 9 / Act 10 / other-mission claims), solutions (native,
#                wasm32-wasip1, real server sandbox), contract (cargo check
#                per solution, corpus programs)
#   compile-api: every ignored test in the crate (expectations_proof,
#                grading_real, http_real, wasm_builder, run_timeout,
#                job_isolation, cargo_grader, third_review,
#                act9_unsafe_harness, and the ignored unit tests)
#
# It first checks that rustc is the compile-API's pinned release
# (PINNED_RUSTC_VERSION in compile-api/src/toolchain.rs) and stops if not:
# NPC claims checked on another compiler prove nothing. The tests assert
# this too; checking here fails in seconds instead of after a build.
#
# RELEASE RULE: a release tag requires a green run of this script on the
# tagged commit. Each run writes a record to logs/ci-slow-<sha>.txt; file
# the green record with the release.
#
# Needs: the wasm32-wasip1 target. One act9_unsafe_harness case needs
# Linux + bwrap (run on cnc for the full set).
# ASCII-only (PowerShell 5.1).

param(
    # Run cargo/rustc through `+<toolchain>` (e.g. 1.99.0) instead of the
    # default toolchain.
    [string]$Toolchain = "",
    # Limit test threads (each ignored test spawns rustc/cargo itself).
    [int]$TestThreads = 0
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== Pledge and Crown slow CI (ignored suites) ===" -ForegroundColor Cyan

# Same worktree isolation as ci.ps1.
$gitMeta = Get-Item .git -Force -ErrorAction SilentlyContinue
if ($gitMeta -and -not $gitMeta.PSIsContainer) {
    $wtName = Split-Path -Leaf $repoRoot
    $env:CARGO_TARGET_DIR = "G:/cargo-target/pledgeandcrown-wt-$wtName"
    Write-Host "[worktree] CARGO_TARGET_DIR -> $env:CARGO_TARGET_DIR" -ForegroundColor Yellow
}

$tc = @()
if ($Toolchain -ne "") { $tc = @("+$Toolchain") }

# --- pinned toolchain check ------------------------------------------------
$toolchainSrc = Get-Content -Raw "compile-api/src/toolchain.rs"
$m = [regex]::Match($toolchainSrc, 'pub const PINNED_RUSTC_VERSION: &str = "([^"]+)";')
if (-not $m.Success) {
    Write-Host "[FAIL] could not read PINNED_RUSTC_VERSION from compile-api/src/toolchain.rs" -ForegroundColor Red
    exit 1
}
$pinned = $m.Groups[1].Value
$rustcLine = (& rustc @tc --version)
if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] rustc $($tc -join ' ') --version failed" -ForegroundColor Red
    exit 1
}
$found = ($rustcLine -split '\s+')[1]
Write-Host "[info] pinned rustc $pinned; using: $rustcLine"
if ($found -ne $pinned) {
    Write-Host "[FAIL] rustc $found is not the pinned $pinned. Install it (rustup toolchain install $pinned --profile minimal --target wasm32-wasip1) and re-run with -Toolchain $pinned." -ForegroundColor Red
    exit 1
}

$sha = (& git rev-parse --short=12 HEAD)
$dirty = [bool](& git status --porcelain --untracked-files=no)
$record = @()
$record += "ci-slow record"
$record += "commit : $sha$(if ($dirty) { ' (uncommitted changes)' } else { '' })"
$record += "date   : $((Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ'))"
$record += "rustc  : $rustcLine"

$threadArgs = @()
if ($TestThreads -gt 0) { $threadArgs = @("--test-threads=$TestThreads") }

$steps = @(
    @{ Name = "game slow suites"; Args = @("test", "-p", "pledge_and_crown", "--test", "mission_text", "--test", "solutions", "--test", "contract", "--", "--ignored") },
    @{ Name = "compile-api slow suites"; Args = @("test", "-p", "pledge_compile_api", "--", "--ignored") }
)

$failed = $false
$total = Get-Date
foreach ($s in $steps) {
    $cargoArgs = $tc + $s.Args + $threadArgs
    Write-Host ""
    Write-Host "--- $($s.Name) ---" -ForegroundColor Cyan
    Write-Host "> cargo $($cargoArgs -join ' ')"
    $t = Get-Date
    & cargo @cargoArgs
    $code = $LASTEXITCODE
    $secs = ((Get-Date) - $t).TotalSeconds
    $line = "{0,-26} {1,-4} {2,7:N0}s" -f $s.Name, $(if ($code -eq 0) { "ok" } else { "FAIL" }), $secs
    $record += $line
    if ($code -ne 0) {
        Write-Host "[FAIL] $($s.Name) (exit $code)" -ForegroundColor Red
        $failed = $true
    } else {
        Write-Host "[ok] $($s.Name) in $([int]$secs)s" -ForegroundColor Green
    }
}
$record += ("total {0:N0}s, result {1}" -f ((Get-Date) - $total).TotalSeconds, $(if ($failed) { "FAILED" } else { "GREEN" }))

New-Item -ItemType Directory -Force -Path "logs" | Out-Null
$recordPath = "logs/ci-slow-$sha.txt"
[System.IO.File]::WriteAllText((Join-Path $repoRoot $recordPath), (($record -join "`n") + "`n"), (New-Object System.Text.UTF8Encoding($false)))
Write-Host ""
$record | ForEach-Object { Write-Host $_ }
Write-Host "[info] record written to $recordPath"
if ($failed) {
    Write-Host "=== slow CI FAILED ===" -ForegroundColor Red
    exit 1
}
Write-Host "=== slow CI GREEN ===" -ForegroundColor Green
