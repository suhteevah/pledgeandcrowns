# Installs the pre-commit hook into .git/hooks/.
# Run once after `git init` (or after fresh clone).
# powershell -ExecutionPolicy Bypass -File scripts/install-hooks.ps1

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not (Test-Path ".git")) {
    Write-Host "[fail] not a git repo. run 'git init' first." -ForegroundColor Red
    exit 1
}

$hookPath = ".git/hooks/pre-commit"
$hookSrc  = "scripts/pre-commit.sh"

if (-not (Test-Path $hookSrc)) {
    Write-Host "[fail] missing $hookSrc" -ForegroundColor Red
    exit 1
}

Copy-Item -Force $hookSrc $hookPath
# Git for Windows respects shebang; mark executable bit via git update-index for visibility.
git update-index --chmod=+x $hookSrc 2>$null

Write-Host "[ok] pre-commit hook installed at $hookPath" -ForegroundColor Green
