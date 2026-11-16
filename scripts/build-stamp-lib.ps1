# Shared helpers for the web build stamp. Dot-source it:
#
#   . (Join-Path $PSScriptRoot "build-stamp-lib.ps1")
#
# scripts/web-build.ps1 writes web/pkg/build-stamp.json after every build.
# It records which game was built (flavour "demo" = the free demo, the
# `demo` cargo feature, 24 missions; "full" = the whole game), the cargo
# features, the commit, the date, the cargo profile and the SHA-256 of the
# final .wasm. The stamp lives INSIDE web/pkg/, so it is deleted together
# with the build it describes and cannot outlive it.
#
# Everything that packages or ships the web build (web-package.ps1,
# itch-push.ps1, mobile/build-android.ps1, and the Tauri shell's build.rs)
# reads the stamp, prints it, and refuses a full build unless the caller
# says -Full (PNC_ALLOW_FULL_BUILD=1 for the Tauri build). A missing stamp,
# or a .wasm whose hash does not match it, is refused too: nobody can tell
# which game that is.
#
# ASCII-only (PowerShell 5.1 reads BOM-less files as ANSI).

$script:StampFileName = "build-stamp.json"
$script:WasmFileName = "pledge_and_crown_bg.wasm"

function Get-BuildStampPath {
    param([Parameter(Mandatory = $true)][string]$WebDir)
    return (Join-Path (Join-Path $WebDir "pkg") $script:StampFileName)
}

function Get-Sha256Hex {
    param([Parameter(Mandatory = $true)][System.IO.Stream]$Stream)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = $sha.ComputeHash($Stream)
    } finally {
        $sha.Dispose()
    }
    return (($bytes | ForEach-Object { $_.ToString("x2") }) -join "")
}

function Get-FileSha256Hex {
    param([Parameter(Mandatory = $true)][string]$Path)
    $fs = [System.IO.File]::OpenRead($Path)
    try {
        return (Get-Sha256Hex -Stream $fs)
    } finally {
        $fs.Dispose()
    }
}

function Write-BuildStamp {
    param(
        [Parameter(Mandatory = $true)][string]$WebDir,
        [Parameter(Mandatory = $true)][ValidateSet("demo", "full")][string]$Flavour,
        [string]$Features = "",
        [Parameter(Mandatory = $true)][string]$CargoProfile
    )
    $wasm = Join-Path (Join-Path $WebDir "pkg") $script:WasmFileName
    if (-not (Test-Path $wasm)) { throw "cannot stamp: $wasm is missing" }
    $commit = (& git rev-parse --short=12 HEAD 2>$null)
    if ($LASTEXITCODE -ne 0 -or -not $commit) { $commit = "unknown" }
    $dirtyOut = (& git status --porcelain --untracked-files=no 2>$null)
    $dirty = if ($dirtyOut) { "true" } else { "false" }
    $date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    $hash = Get-FileSha256Hex -Path $wasm
    # Written by hand (not ConvertTo-Json) so the layout is fixed and the
    # Tauri build.rs can read it without a JSON crate.
    $lines = @(
        "{",
        ('  "flavour": "{0}",' -f $Flavour),
        ('  "features": "{0}",' -f $Features),
        ('  "commit": "{0}",' -f $commit),
        ('  "dirty": {0},' -f $dirty),
        ('  "date": "{0}",' -f $date),
        ('  "profile": "{0}",' -f $CargoProfile),
        ('  "wasm_sha256": "{0}"' -f $hash),
        "}"
    )
    $path = Get-BuildStampPath -WebDir $WebDir
    $text = ($lines -join "`n") + "`n"
    [System.IO.File]::WriteAllText($path, $text, (New-Object System.Text.UTF8Encoding($false)))
    return $path
}

function ConvertFrom-BuildStampText {
    param([Parameter(Mandatory = $true)][string]$Text)
    try {
        $s = $Text | ConvertFrom-Json
    } catch {
        return $null
    }
    if (-not $s.flavour -or ($s.flavour -ne "demo" -and $s.flavour -ne "full")) { return $null }
    return $s
}

function Read-BuildStamp {
    param([Parameter(Mandatory = $true)][string]$WebDir)
    $path = Get-BuildStampPath -WebDir $WebDir
    if (-not (Test-Path $path)) { return $null }
    return (ConvertFrom-BuildStampText -Text ([System.IO.File]::ReadAllText($path)))
}

# Reads pkg/build-stamp.json out of a packaged zip and adds a
# `zip_wasm_sha256` property with the hash of the zip's own .wasm entry.
function Read-BuildStampFromZip {
    param([Parameter(Mandatory = $true)][string]$ZipPath)
    Add-Type -AssemblyName System.IO.Compression
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path $ZipPath).Path)
    try {
        $entry = $zip.GetEntry("pkg/" + $script:StampFileName)
        if (-not $entry) { return $null }
        $reader = New-Object System.IO.StreamReader($entry.Open())
        try { $text = $reader.ReadToEnd() } finally { $reader.Dispose() }
        $stamp = ConvertFrom-BuildStampText -Text $text
        if (-not $stamp) { return $null }
        $wasmEntry = $zip.GetEntry("pkg/" + $script:WasmFileName)
        $zipHash = ""
        if ($wasmEntry) {
            $ws = $wasmEntry.Open()
            try { $zipHash = Get-Sha256Hex -Stream $ws } finally { $ws.Dispose() }
        }
        $stamp | Add-Member -NotePropertyName zip_wasm_sha256 -NotePropertyValue $zipHash
        return $stamp
    } finally {
        $zip.Dispose()
    }
}

function Show-BuildStamp {
    param($Stamp, [string]$Source = "")
    if (-not $Stamp) {
        Write-Host "[stamp] none found $Source" -ForegroundColor Yellow
        return
    }
    $color = if ($Stamp.flavour -eq "demo") { "Green" } else { "Magenta" }
    $label = if ($Stamp.flavour -eq "demo") { "DEMO (free, 24 missions)" } else { "FULL GAME (all missions)" }
    Write-Host ("[stamp] {0} {1}" -f $label, $Source) -ForegroundColor $color
    Write-Host ("[stamp]   features : '{0}'" -f $Stamp.features)
    Write-Host ("[stamp]   commit   : {0}{1}" -f $Stamp.commit, $(if ($Stamp.dirty) { " (uncommitted changes)" } else { "" }))
    Write-Host ("[stamp]   date     : {0}" -f $Stamp.date)
    Write-Host ("[stamp]   profile  : {0}" -f $Stamp.profile)
}

# Returns $true when the build may be packaged/shipped. Prints why not.
# $ActualWasmSha256: hash of the .wasm actually about to ship.
function Test-BuildAllowed {
    param(
        $Stamp,
        [Parameter(Mandatory = $true)][bool]$Full,
        [string]$ActualWasmSha256 = ""
    )
    if (-not $Stamp) {
        Write-Host "[refused] no valid build stamp: cannot tell whether this is the demo or the full game. Rebuild with scripts/web-build.ps1." -ForegroundColor Red
        return $false
    }
    if ($ActualWasmSha256 -and ($ActualWasmSha256 -ne $Stamp.wasm_sha256)) {
        Write-Host "[refused] the .wasm does not match its build stamp (rebuilt or replaced outside web-build.ps1). Rebuild with scripts/web-build.ps1." -ForegroundColor Red
        return $false
    }
    if ($Stamp.flavour -eq "full" -and -not $Full) {
        Write-Host "[refused] this is the FULL game. The free web/itch/mobile build is the demo. Pass -Full only if you really mean to ship the full game." -ForegroundColor Red
        return $false
    }
    if ($Stamp.flavour -eq "demo" -and $Full) {
        Write-Host "[refused] -Full was passed but this is the demo build. Rebuild with scripts/web-build.ps1 -Full." -ForegroundColor Red
        return $false
    }
    return $true
}
