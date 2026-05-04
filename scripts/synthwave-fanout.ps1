# SPDX-License-Identifier: MIT
# Multi-GPU fanout dispatcher for synthwave-gen.
#
# Reuses the venv that scripts/synthwave-gen.ps1 sets up. If the venv
# doesn't exist yet, defer to that script first - this one assumes
# torch + diffusers are already installed.
#
# Examples:
#   powershell -ExecutionPolicy Bypass -File scripts/synthwave-fanout.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/synthwave-fanout.ps1 --dry-run
#   powershell -ExecutionPolicy Bypass -File scripts/synthwave-fanout.ps1 --devices 0,1,2 --force

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$PassThru = @()
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$VenvDir  = Join-Path $RepoRoot "tools\synthwave-gen\.venv"
$VenvPy   = Join-Path $VenvDir "Scripts\python.exe"
$Driver   = Join-Path $RepoRoot "tools\synthwave-gen\fanout.py"

Write-Host "=== pledge & crown - synthwave gen (multi-gpu fanout) ==="

if (-not (Test-Path $VenvPy)) {
    Write-Error "venv not found at $VenvDir. Run scripts/synthwave-gen.ps1 once first to set it up."
    exit 1
}

# Pass --python so the spawned subprocesses inherit the same venv.
& $VenvPy $Driver --python $VenvPy @PassThru
$rc = $LASTEXITCODE
if ($rc -ne 0) {
    Write-Error "fanout.py exited with $rc"
    exit $rc
}
Write-Host "[ok] fanout complete"
