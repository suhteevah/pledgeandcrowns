# install-android-ndk.ps1
# Bootstraps the Android command-line tools + NDK into the EXISTING SDK at
# G:\AndroidSdk so the Tauri Android bundle can cross-compile Rust.
#
# Idempotent: skips the cmdline-tools download if sdkmanager is already there;
# sdkmanager itself skips the NDK download if that exact version is installed.
#
# Run:  powershell -ExecutionPolicy Bypass -File mobile/install-android-ndk.ps1
#
# ASCII-only by project convention. Verbose by project convention.

$ErrorActionPreference = 'Stop'

$Sdk        = 'G:\AndroidSdk'
$Jdk        = 'C:\Program Files\Java\jdk-21'
$CmdlineUrl = 'https://dl.google.com/android/repository/commandlinetools-win-15641748_latest.zip'
$NdkPkg     = 'ndk;27.3.13750724'   # r27 LTS, latest patch

Write-Host "=== Android NDK bootstrap into $Sdk ==="

if (-not (Test-Path $Sdk))  { throw "SDK root not found: $Sdk" }
if (-not (Test-Path $Jdk))  { throw "JDK not found: $Jdk (sdkmanager needs JDK 17+)" }

# sdkmanager needs JAVA_HOME pointing at the JDK ROOT (not \bin) for THIS process.
$env:JAVA_HOME = $Jdk
$env:PATH      = "$Jdk\bin;$env:PATH"
Write-Host "JAVA_HOME = $env:JAVA_HOME"

$SdkManager = Join-Path $Sdk 'cmdline-tools\latest\bin\sdkmanager.bat'

# --- 1. cmdline-tools (sdkmanager) -----------------------------------------
if (Test-Path $SdkManager) {
    Write-Host "[skip] cmdline-tools already present: $SdkManager"
} else {
    Write-Host "[1/3] Downloading cmdline-tools ..."
    $zip = Join-Path $env:TEMP 'android-cmdline-tools.zip'
    & curl.exe -L --fail -o $zip $CmdlineUrl
    if ($LASTEXITCODE -ne 0) { throw "cmdline-tools download failed (curl exit $LASTEXITCODE)" }

    $tmp = Join-Path $env:TEMP 'android-cmdline-extract'
    if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
    Write-Host "[1/3] Extracting ..."
    Expand-Archive -Path $zip -DestinationPath $tmp -Force

    # The zip unpacks to a top-level 'cmdline-tools' dir; the SDK layout needs
    # its contents under <sdk>\cmdline-tools\latest\.
    $dest    = Join-Path $Sdk 'cmdline-tools\latest'
    $destDir = Split-Path $dest
    New-Item -ItemType Directory -Force -Path $destDir | Out-Null
    if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
    Move-Item (Join-Path $tmp 'cmdline-tools') $dest

    Remove-Item $zip -Force -ErrorAction SilentlyContinue
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "[1/3] cmdline-tools installed at $dest"
}

# Sanity: sdkmanager must launch under JDK 21 before we pull ~1GB of NDK.
Write-Host "[check] sdkmanager --version"
& $SdkManager --sdk_root=$Sdk --version
if ($LASTEXITCODE -ne 0) { throw "sdkmanager failed to launch (exit $LASTEXITCODE)" }

# --- 2. Licenses (auto-accept on Matt's behalf, per pipeline policy) --------
Write-Host "[2/3] Accepting SDK licenses ..."
$yes = ("y`r`n" * 50)
$yes | & $SdkManager --sdk_root=$Sdk --licenses
# (non-fatal: license prompt may already be fully accepted)

# --- 3. NDK ----------------------------------------------------------------
Write-Host "[3/3] Installing $NdkPkg (this is the big download) ..."
$yes | & $SdkManager --sdk_root=$Sdk $NdkPkg
if ($LASTEXITCODE -ne 0) { throw "NDK install failed (exit $LASTEXITCODE)" }

# --- Report ----------------------------------------------------------------
$ndkVer  = ($NdkPkg -split ';')[1]
$ndkPath = Join-Path $Sdk "ndk\$ndkVer"
Write-Host ""
Write-Host "=== DONE ==="
if (Test-Path $ndkPath) {
    Write-Host "NDK installed: $ndkPath"
    Write-Host "Next: setx ANDROID_HOME `"$Sdk`"; setx NDK_HOME `"$ndkPath`""
} else {
    Write-Host "WARNING: expected NDK dir not found at $ndkPath -- check sdkmanager output above."
}
