# Build the web version and zip it for itch.io (HTML5 upload).
#
#   powershell -ExecutionPolicy Bypass -File scripts/web-package.ps1                  # build + zip the DEMO
#   powershell -ExecutionPolicy Bypass -File scripts/web-package.ps1 -SkipBuild       # zip what is in web/ (must be the demo)
#   powershell -ExecutionPolicy Bypass -File scripts/web-package.ps1 -Full            # build + zip the FULL game
#   powershell -ExecutionPolicy Bypass -File scripts/web-package.ps1 -SkipBuild -Full # zip a full build already in web/
#
# Output: dist/pledge-and-crown-web-<flavour>.zip (flavour = demo or full)
# with index.html at the ZIP ROOT (itch requirement), plus pkg/ (including
# pkg/build-stamp.json) and assets/.
#
# The free itch build is the demo. This script reads web/pkg/build-stamp.json
# (written by web-build.ps1), prints it, and refuses to zip a full build
# unless -Full is passed, a demo build when -Full is passed, a build with
# no stamp, or a .wasm that does not match its stamp.
#
# Entry names are written with forward slashes on purpose: PS 5.1's
# Compress-Archive writes backslashes, which itch's Linux-side extractor
# treats as literal filename characters (assets\audio\title.ogg -> 404).

param(
    [switch]$SkipBuild,
    [string]$CargoProfile = "wasm-release",
    # Package the full game instead of the demo.
    [switch]$Full
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot
. (Join-Path $PSScriptRoot "build-stamp-lib.ps1")

Write-Host "=== pledge & crown - web package (itch.io) ===" -ForegroundColor Cyan

if (-not $SkipBuild) {
    $buildArgs = @("-ExecutionPolicy", "Bypass", "-File", (Join-Path $PSScriptRoot "web-build.ps1"), "-CargoProfile", $CargoProfile)
    if ($Full) { $buildArgs += "-Full" }
    & powershell @buildArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[FAIL] web-build.ps1 exit $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

$webDir = (Resolve-Path "web").Path
foreach ($req in @("index.html", "pkg/pledge_and_crown.js", "pkg/pledge_and_crown_bg.wasm")) {
    if (-not (Test-Path (Join-Path $webDir $req))) {
        Write-Host "[FAIL] web/$req missing - run scripts/web-build.ps1 first." -ForegroundColor Red
        exit 1
    }
}

$stamp = Read-BuildStamp -WebDir $webDir
Show-BuildStamp -Stamp $stamp -Source "(web/pkg)"
$wasmHash = Get-FileSha256Hex -Path (Join-Path $webDir "pkg/pledge_and_crown_bg.wasm")
if (-not (Test-BuildAllowed -Stamp $stamp -Full ([bool]$Full) -ActualWasmSha256 $wasmHash)) {
    exit 4
}

# Only ship what the page needs: the loader, the wasm-bindgen pkg and assets.
$files = @()
$files += Get-Item (Join-Path $webDir "index.html")
$files += Get-ChildItem -Path (Join-Path $webDir "pkg") -Recurse -File
if (Test-Path (Join-Path $webDir "assets")) {
    $files += Get-ChildItem -Path (Join-Path $webDir "assets") -Recurse -File
}

New-Item -ItemType Directory -Force -Path "dist" | Out-Null
$zipPath = Join-Path (Resolve-Path "dist").Path ("pledge-and-crown-web-{0}.zip" -f $stamp.flavour)
if (Test-Path $zipPath) { Remove-Item -Force $zipPath }

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$maxFile = 0
$maxFileName = ""
$extracted = 0
$zip = [System.IO.Compression.ZipFile]::Open($zipPath, [System.IO.Compression.ZipArchiveMode]::Create)
try {
    foreach ($f in $files) {
        $rel = $f.FullName.Substring($webDir.Length).TrimStart('\', '/') -replace '\\', '/'
        [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
            $zip, $f.FullName, $rel, [System.IO.Compression.CompressionLevel]::Optimal) | Out-Null
        $extracted += $f.Length
        if ($f.Length -gt $maxFile) { $maxFile = $f.Length; $maxFileName = $rel }
    }
} finally {
    $zip.Dispose()
}

$zipSize = (Get-Item $zipPath).Length
$count = $files.Count

# itch.io HTML5 limits (itch.io/docs/creators/html5): zip < 1 GB,
# extracted total <= 500 MB, <= 1000 files, each file <= 200 MB.
$okZip = $zipSize -lt 1GB
$okExtracted = $extracted -le 500MB
$okCount = $count -le 1000
$okFile = $maxFile -le 200MB

Write-Host ""
Write-Host "[ok] wrote $zipPath" -ForegroundColor Green
Write-Host ("  zip size        {0,9:N2} MB   (itch limit 1024 MB)  {1}" -f ($zipSize / 1MB), $(if ($okZip) { "ok" } else { "OVER" }))
Write-Host ("  extracted total {0,9:N2} MB   (itch limit  500 MB)  {1}" -f ($extracted / 1MB), $(if ($okExtracted) { "ok" } else { "OVER" }))
Write-Host ("  file count      {0,9}      (itch limit  1000)     {1}" -f $count, $(if ($okCount) { "ok" } else { "OVER" }))
Write-Host ("  largest file    {0,9:N2} MB   (itch limit  200 MB)  {1}  [{2}]" -f ($maxFile / 1MB), $(if ($okFile) { "ok" } else { "OVER" }), $maxFileName)

if (-not ($okZip -and $okExtracted -and $okCount -and $okFile)) {
    Write-Host "[FAIL] package exceeds an itch.io HTML5 limit." -ForegroundColor Red
    exit 1
}
if ($stamp.flavour -eq "full") {
    Write-Host "Upload (FULL game): scripts/itch-push.ps1 -Target <user>/<game>:<channel> -Full -Confirm" -ForegroundColor Magenta
} else {
    Write-Host "Upload (demo): scripts/itch-push.ps1 -Target <user>/<game>:html5 -Confirm" -ForegroundColor Cyan
}
