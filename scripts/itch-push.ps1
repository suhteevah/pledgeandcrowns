# Upload the packaged web build to itch.io with butler.
#
#   powershell -ExecutionPolicy Bypass -File scripts/itch-push.ps1 -Target <user>/<game>:html5 -Confirm
#   powershell -ExecutionPolicy Bypass -File scripts/itch-push.ps1 -Target <user>/<game>:<channel> -Full -Confirm
#
# Refuses to run without -Confirm: a push publishes a new build to
# players. Requires `butler` on PATH and a prior `butler login` (owner
# steps - this script never installs butler or handles credentials).
# Build the zip first with scripts/web-package.ps1.
#
# The free itch build is the DEMO. By default this pushes
# dist/pledge-and-crown-web-demo.zip. It reads pkg/build-stamp.json from
# inside the zip, prints it, and refuses: a zip with no stamp, a zip whose
# .wasm does not match its stamp, a FULL build unless -Full is passed, and
# a demo build when -Full is passed. These checks run before butler is
# even looked for.
#
# NOTE: deliberately a plain script (no [CmdletBinding()]) so -Confirm
# is our own switch, not PowerShell's ShouldProcess common parameter.

param(
    [string]$Target = "",
    # Default: dist/pledge-and-crown-web-demo.zip (-full.zip with -Full).
    [string]$Zip = "",
    [string]$UserVersion = "",
    # Push the full game. Without it a full build is refused.
    [switch]$Full,
    [switch]$Confirm
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot
. (Join-Path $PSScriptRoot "build-stamp-lib.ps1")

if ($Zip -eq "") {
    $Zip = if ($Full) { "dist/pledge-and-crown-web-full.zip" } else { "dist/pledge-and-crown-web-demo.zip" }
}

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

$stamp = Read-BuildStampFromZip -ZipPath $Zip
Show-BuildStamp -Stamp $stamp -Source "(inside $Zip)"
$zipWasmHash = if ($stamp) { $stamp.zip_wasm_sha256 } else { "" }
if ($stamp -and -not $zipWasmHash) {
    Write-Host "[refused] $Zip has no pkg/pledge_and_crown_bg.wasm." -ForegroundColor Red
    exit 4
}
if (-not (Test-BuildAllowed -Stamp $stamp -Full ([bool]$Full) -ActualWasmSha256 $zipWasmHash)) {
    exit 4
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
