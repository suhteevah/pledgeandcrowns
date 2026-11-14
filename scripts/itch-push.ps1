# Upload the packaged web build to itch.io with butler.
#
#   powershell -ExecutionPolicy Bypass -File scripts/itch-push.ps1 -Target <user>/<game>:html5 -Confirm
#
# Refuses to run without -Confirm: a push publishes a new build to
# players. Requires `butler` on PATH and a prior `butler login` (owner
# steps - this script never installs butler or handles credentials).
# Build the zip first with scripts/web-package.ps1.
#
# NOTE: deliberately a plain script (no [CmdletBinding()]) so -Confirm
# is our own switch, not PowerShell's ShouldProcess common parameter.

param(
    [string]$Target = "",
    [string]$Zip = "dist/pledge-and-crown-web.zip",
    [string]$UserVersion = "",
    [switch]$Confirm
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== pledge & crown - itch.io push ===" -ForegroundColor Cyan

if ($Target -eq "") {
    Write-Host "[FAIL] -Target is required, e.g. -Target someuser/pledge-and-crown:html5" -ForegroundColor Red
    exit 2
}
if ($Target -notmatch '^[^/\s]+/[^:\s]+:[^\s]+$') {
    Write-Host "[FAIL] -Target must look like <user>/<game>:<channel> (got '$Target')" -ForegroundColor Red
    exit 2
}
if (-not (Test-Path $Zip)) {
    Write-Host "[FAIL] $Zip not found - run scripts/web-package.ps1 first." -ForegroundColor Red
    exit 1
}

$butler = Get-Command butler -ErrorAction SilentlyContinue
if (-not $butler) {
    Write-Host "[FAIL] butler not on PATH. Owner step: install butler (itch.io/docs/butler) and run 'butler login'." -ForegroundColor Red
    exit 1
}

$zipMb = (Get-Item $Zip).Length / 1MB
Write-Host ("[info] butler : {0}" -f $butler.Source)
Write-Host ("[info] zip    : {0} ({1:N2} MB)" -f $Zip, $zipMb)
Write-Host ("[info] target : {0}" -f $Target)

if (-not $Confirm) {
    Write-Host "[refused] dry run only. Re-run with -Confirm to actually publish this build." -ForegroundColor Yellow
    exit 3
}

$butlerArgs = @("push", $Zip, $Target)
if ($UserVersion -ne "") { $butlerArgs += @("--userversion", $UserVersion) }
Write-Host "> butler $($butlerArgs -join ' ')"
& $butler.Source @butlerArgs
if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] butler push exit $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "[ok] pushed to $Target" -ForegroundColor Green
