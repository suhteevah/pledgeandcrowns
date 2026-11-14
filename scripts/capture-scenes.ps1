# capture-scenes.ps1 -- NON-INTRUSIVE scripted capture of game scenes.
#
# Launches the game in PNC_SHOTS mode. The game drives itself through every
# scripted scene (title, world vantages, dialogue, editor, menus, y-sort and
# collision proofs), screenshots each one with Bevy's own screenshot API,
# writes a DONE marker listing what it captured, and exits by itself.
#
# This launcher ONLY sets env vars, starts the process, and waits for DONE.
# No focus stealing, no key injection, no window manipulation: the game
# window opens UNFOCUSED (lib.rs) and nothing here touches input. If the
# timeout passes it stops ONLY the process ID it started.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/capture-scenes.ps1 `
#       -Exe G:\cargo-target\...\debug\pledge_and_crown.exe -OutDir C:\tmp\shots
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [int]$TimeoutSec = 300
)
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$gameDir = Join-Path $repoRoot "game"
if (-not (Test-Path $Exe)) { Write-Host "[FAIL] exe not found: $Exe" -ForegroundColor Red; exit 2 }
if (-not (Test-Path (Join-Path $gameDir "assets"))) {
    Write-Host "[FAIL] no assets dir under $gameDir" -ForegroundColor Red; exit 2
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$OutDir = (Resolve-Path $OutDir).Path

$done = Join-Path $OutDir "DONE"
$stdoutLog = Join-Path $OutDir "game-stdout.log"
$stderrLog = Join-Path $OutDir "game-stderr.log"
Remove-Item $done -ErrorAction SilentlyContinue
Get-ChildItem -Path $OutDir -Filter "*.png" -ErrorAction SilentlyContinue | Remove-Item -ErrorAction SilentlyContinue

# Child inherits these. BEVY_ASSET_ROOT must be the game/ dir of THIS checkout
# or every sprite renders gray (see HANDOFF.md).
$env:BEVY_ASSET_ROOT = $gameDir
$env:PNC_SHOTS = "1"
$env:PNC_SHOTS_DIR = $OutDir
if (-not $env:RUST_LOG) { $env:RUST_LOG = "warn,pledge_and_crown=info" }

Write-Host "Launching game in scripted-capture mode (unfocused, self-driving)..."
Write-Host "  exe:    $Exe"
Write-Host "  assets: $gameDir"
Write-Host "  out:    $OutDir"
$proc = Start-Process -FilePath $Exe -PassThru -WorkingDirectory $gameDir `
    -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog
$startedPid = $proc.Id
Write-Host "  pid:    $startedPid"

$sw = [System.Diagnostics.Stopwatch]::StartNew()
$timedOut = $false
while ($true) {
    if (Test-Path $done) { break }
    if ($proc.HasExited) { break }
    if ($sw.Elapsed.TotalSeconds -gt $TimeoutSec) { $timedOut = $true; break }
    Start-Sleep -Milliseconds 500
}

if (Test-Path $done) {
    # The game exits by itself right after writing DONE; give it a moment.
    if (-not $proc.WaitForExit(15000)) { $timedOut = $true }
}
if ($timedOut -and -not $proc.HasExited) {
    Write-Host "[WARN] timeout after $TimeoutSec s; stopping pid $startedPid (the process this script started)" -ForegroundColor Yellow
    Stop-Process -Id $startedPid -Force -ErrorAction SilentlyContinue
}
$elapsed = [math]::Round($sw.Elapsed.TotalSeconds, 1)

Write-Host ""
Write-Host "--- summary ($elapsed s) ---" -ForegroundColor Cyan
$status = 0
if (Test-Path $done) {
    Get-Content $done | ForEach-Object { Write-Host "  $_" }
    if (Select-String -Path $done -Pattern "^FAIL" -Quiet) { $status = 1 }
} else {
    Write-Host "[FAIL] no DONE marker; the game did not finish its scene list" -ForegroundColor Red
    $status = 1
}
$pngs = @(Get-ChildItem -Path $OutDir -Filter "*.png" -ErrorAction SilentlyContinue)
Write-Host "  png files in out dir: $($pngs.Count)"

Write-Host ""
Write-Host "--- WARN/ERROR lines from the game log ---" -ForegroundColor Cyan
$hits = @()
foreach ($log in @($stdoutLog, $stderrLog)) {
    if (Test-Path $log) {
        $hits += @(Select-String -Path $log -Pattern "WARN|ERROR|panicked" | ForEach-Object { $_.Line })
    }
}
if ($hits.Count -eq 0) { Write-Host "  (none)" } else { $hits | ForEach-Object { Write-Host "  $_" } }
exit $status
