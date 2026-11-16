# build-android.ps1
# Sets the Android SDK/NDK/JDK env for THIS process and runs a Tauri android
# step against the existing SDK at G:\AndroidSdk. The rest of the repo keeps
# its default (GNU) host toolchain; the android targets link via the NDK clang
# that `cargo tauri android` wires up from NDK_HOME.
#
#   powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 init
#   powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 build         # all ABIs, release
#   powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 build --debug
#   powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 dev           # needs a device/emulator
#   powershell -ExecutionPolicy Bypass -File mobile/build-android.ps1 build -Full   # bundle a FULL web build on purpose
#
# The app bundles ../web (the wasm web build). The free mobile build is the
# DEMO: for `build` and `dev` this script reads web/pkg/build-stamp.json
# (written by scripts/web-build.ps1), prints it, and refuses a full build
# unless -Full is passed (it then sets PNC_ALLOW_FULL_BUILD=1, which the
# shell's build.rs gate also requires). Build the web side first:
#   powershell -ExecutionPolicy Bypass -File scripts/web-build.ps1          # demo
#
# ASCII-only + verbose by project convention.

param(
    [Parameter(Mandatory = $true)]
    [string]$Step,
    # Allow bundling a full-game web build.
    [switch]$Full,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Rest
)

$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot '..\scripts\build-stamp-lib.ps1')
if ($Step -eq 'build' -or $Step -eq 'dev') {
    $webDir = (Resolve-Path (Join-Path $PSScriptRoot '..\web')).Path
    $stamp = Read-BuildStamp -WebDir $webDir
    Show-BuildStamp -Stamp $stamp -Source '(web/pkg)'
    $wasm = Join-Path $webDir 'pkg\pledge_and_crown_bg.wasm'
    $wasmHash = if (Test-Path $wasm) { Get-FileSha256Hex -Path $wasm } else { 'missing' }
    if (-not (Test-BuildAllowed -Stamp $stamp -Full ([bool]$Full) -ActualWasmSha256 $wasmHash)) {
        exit 4
    }
    if ($Full) { $env:PNC_ALLOW_FULL_BUILD = '1' } else { Remove-Item Env:PNC_ALLOW_FULL_BUILD -ErrorAction SilentlyContinue }
}

$Sdk    = 'G:\AndroidSdk'
$NdkVer = '27.3.13750724'
$Ndk    = Join-Path $Sdk "ndk\$NdkVer"
$Jdk    = 'C:\Program Files\Java\jdk-21'

foreach ($p in @($Sdk, $Ndk, $Jdk)) {
    if (-not (Test-Path $p)) { throw "Required path missing: $p" }
}

$env:ANDROID_HOME     = $Sdk
$env:ANDROID_SDK_ROOT = $Sdk
$env:NDK_HOME         = $Ndk
$env:ANDROID_NDK_HOME = $Ndk
$env:JAVA_HOME        = $Jdk
$env:PATH             = "$Jdk\bin;$env:PATH"

Write-Host "ANDROID_HOME = $env:ANDROID_HOME"
Write-Host "NDK_HOME     = $env:NDK_HOME"
Write-Host "JAVA_HOME    = $env:JAVA_HOME"

$proj = Join-Path $PSScriptRoot 'src-tauri'
Set-Location $proj
Write-Host "cwd = $proj"
Write-Host "RUN: cargo tauri android $Step $($Rest -join ' ')"

& cargo tauri android $Step @Rest
exit $LASTEXITCODE
