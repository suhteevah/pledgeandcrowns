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
#
# ASCII-only + verbose by project convention.

param(
    [Parameter(Mandatory = $true)]
    [string]$Step,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Rest
)

$ErrorActionPreference = 'Stop'

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
