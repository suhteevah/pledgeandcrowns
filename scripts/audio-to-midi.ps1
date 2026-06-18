# SPDX-License-Identifier: MIT
# Pledge & Crown audio-to-MIDI transcriber - Windows wrapper.
#
# Creates/uses a venv at tools/audio-to-midi/.venv, installs torch +
# torchaudio (cu121 wheels from the PyTorch index), installs the rest
# from PyPI, then runs tools/audio-to-midi/transcribe.py with whatever
# args you pass.
#
# ASCII-only by design (PowerShell 5.1 + non-ASCII is a known gotcha).
#
# Examples:
#   powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --only village
#   powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --all --force
#   powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --only village --no-stems
#   powershell -ExecutionPolicy Bypass -File scripts/audio-to-midi.ps1 --dry-run

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$PassThru = @()
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ToolDir  = Join-Path $RepoRoot "tools\audio-to-midi"
$VenvDir  = Join-Path $ToolDir ".venv"
$Reqs     = Join-Path $ToolDir "requirements.txt"
$Driver   = Join-Path $ToolDir "transcribe.py"

Write-Host "=== pledge & crown - audio-to-midi ==="
Write-Host "repo:   $RepoRoot"
Write-Host "venv:   $VenvDir"

# 1. Locate a usable Python. 3.11 is the sweet spot for torch/ML wheels;
#    3.10 is the fallback. Do NOT use 3.12+/3.13/3.14 - ML wheels lag.
$PythonExe = $null
foreach ($candidate in @("py -3.11", "py -3.10")) {
    try {
        $ver = & cmd /c "$candidate --version 2>&1"
        if ($LASTEXITCODE -eq 0 -and $ver -match "Python 3\.(10|11)") {
            $PythonExe = $candidate
            Write-Host "[ok] python: $candidate -> $ver"
            break
        }
    } catch { }
}
if (-not $PythonExe) {
    Write-Error "no Python 3.10 or 3.11 found (py -3.11 / py -3.10). ML wheels need one of those; install it and retry."
    exit 1
}

# 2. Create venv if missing.
if (-not (Test-Path $VenvDir)) {
    Write-Host "[..] creating venv"
    & cmd /c "$PythonExe -m venv `"$VenvDir`""
    if ($LASTEXITCODE -ne 0) { Write-Error "venv create failed"; exit 1 }
}

$VenvPy = Join-Path $VenvDir "Scripts\python.exe"
if (-not (Test-Path $VenvPy)) {
    Write-Error "venv python not at $VenvPy after create"
    exit 1
}

# 3. Install deps once (marker-gated). Torch first from the cu121 index,
#    then everything else from PyPI - same ordering as synthwave-gen.
$Marker = Join-Path $VenvDir ".audio-to-midi-deps-installed"
if (-not (Test-Path $Marker) -or $env:A2M_FORCE_DEPS) {
    # Upgrade pip + wheel, but pin setuptools < 81: Basic Pitch / mir_eval
    # still import the legacy `pkg_resources` module, which setuptools
    # removed in v81. requirements.txt re-pins this too as a backstop.
    Write-Host "[..] upgrading pip + wheel (+ setuptools<81 for pkg_resources)"
    & $VenvPy -m pip install --upgrade pip wheel "setuptools<81"
    if ($LASTEXITCODE -ne 0) { Write-Error "pip upgrade failed"; exit 1 }

    # Torch + torchaudio (cu121). Demucs needs torchaudio; the cu121
    # wheels work on both 30-series and Pascal. CPU-only boxes still get
    # a working torch from this index (it includes a CPU runtime).
    Write-Host "[..] installing torch + torchaudio (cu121, ~2.5 GB - go get coffee)"
    & $VenvPy -m pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu121
    if ($LASTEXITCODE -ne 0) { Write-Error "torch/torchaudio install failed"; exit 1 }

    Write-Host "[..] installing demucs + basic-pitch + midi/audio deps"
    & $VenvPy -m pip install -r $Reqs
    if ($LASTEXITCODE -ne 0) { Write-Error "requirements install failed"; exit 1 }

    Set-Content -Path $Marker -Value (Get-Date -Format o) -Encoding ascii
    Write-Host "[ok] deps installed; marker at $Marker"
} else {
    Write-Host "[ok] deps already installed (delete $Marker or set A2M_FORCE_DEPS=1 to reinstall)"
}

# 4. Run the transcriber with whatever args were passed through.
Write-Host "[..] transcribing"
& $VenvPy $Driver @PassThru
$rc = $LASTEXITCODE
if ($rc -ne 0) {
    Write-Error "transcribe.py exited with $rc"
    exit $rc
}
Write-Host "[ok] audio-to-midi complete"
