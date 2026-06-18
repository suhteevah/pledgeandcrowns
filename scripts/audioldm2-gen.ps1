# SPDX-License-Identifier: MIT
# Pledge & Crown diegetic SFX generator - Windows wrapper.
#
# Creates/uses a venv at tools/audioldm2-gen/.venv, installs torch+CUDA
# wheels against the pytorch index, installs the rest from PyPI, then
# runs tools/audioldm2-gen/generate.py with whatever args you pass.
#
# Sibling of scripts/synthwave-gen.ps1. Same shape; AudioLDM2 instead of
# Stable Audio Open. Generates short diegetic SFX (door creaks, coin
# clinks, sword unsheathe) that the music-biased SAO does poorly.
#
# Examples:
#   powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1 --dry-run
#   powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1 --only door_creak
#   powershell -ExecutionPolicy Bypass -File scripts/audioldm2-gen.ps1 --force

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$PassThru = @()
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ToolDir  = Join-Path $RepoRoot "tools\audioldm2-gen"
$VenvDir  = Join-Path $ToolDir ".venv"
$Reqs     = Join-Path $ToolDir "requirements.txt"
$Driver   = Join-Path $ToolDir "generate.py"

Write-Host "=== pledge & crown - audioldm2 sfx gen ==="
Write-Host "repo:   $RepoRoot"
Write-Host "venv:   $VenvDir"

# 1. Locate a usable Python (3.10+, but 3.11 is the sweet spot for torch wheels).
$PythonExe = $null
foreach ($candidate in @("py -3.11", "py -3.10", "python")) {
    try {
        $ver = & cmd /c "$candidate --version 2>&1"
        if ($LASTEXITCODE -eq 0 -and $ver -match "Python 3\.(1[0-3])") {
            $PythonExe = $candidate
            Write-Host "[ok] python: $candidate -> $ver"
            break
        }
    } catch { }
}
if (-not $PythonExe) {
    Write-Error "no Python 3.10-3.13 found; install one and retry"
    exit 1
}

# 2. Create venv if missing.
if (-not (Test-Path $VenvDir)) {
    Write-Host "[..] creating venv"
    & cmd /c "$PythonExe -m venv `"$VenvDir`""
    if ($LASTEXITCODE -ne 0) { Write-Error "venv create failed"; exit 1 }
}

$VenvPy  = Join-Path $VenvDir "Scripts\python.exe"
$VenvPip = Join-Path $VenvDir "Scripts\pip.exe"
if (-not (Test-Path $VenvPy)) {
    Write-Error "venv python not at $VenvPy after create"
    exit 1
}

# 3. Install / upgrade pip + wheel before anything else (avoids old-pip
#    "could not resolve" footguns on torch index).
$Marker = Join-Path $VenvDir ".audioldm2-deps-installed"
if (-not (Test-Path $Marker) -or $env:AUDIOLDM2_FORCE_DEPS) {
    Write-Host "[..] upgrading pip + wheel"
    & $VenvPy -m pip install --upgrade pip wheel setuptools
    if ($LASTEXITCODE -ne 0) { Write-Error "pip upgrade failed"; exit 1 }

    # Torch + CUDA 12.1 wheel from the pytorch index. cu121 works on
    # CUDA 12.1+ drivers; both 30-series and Pascal (P100) accept it.
    Write-Host "[..] installing torch (cu121, ~2.5 GB - go get coffee)"
    & $VenvPy -m pip install torch --index-url https://download.pytorch.org/whl/cu121
    if ($LASTEXITCODE -ne 0) { Write-Error "torch install failed"; exit 1 }

    Write-Host "[..] installing diffusers + transformers + audio deps"
    & $VenvPy -m pip install -r $Reqs
    if ($LASTEXITCODE -ne 0) { Write-Error "requirements install failed"; exit 1 }

    Set-Content -Path $Marker -Value (Get-Date -Format o) -Encoding utf8
    Write-Host "[ok] deps installed; marker at $Marker"
} else {
    Write-Host "[ok] deps already installed (delete $Marker or set AUDIOLDM2_FORCE_DEPS=1 to reinstall)"
}

# 4. Forward HF auth into the subprocess. AudioLDM2's cvssp repos are
#    not gated today, but the weights are CC-BY-NC and a custom --model
#    may point at a gated mirror. The Python side prints a long
#    actionable error if access fails; here we just make sure HF_TOKEN
#    is plumbed through if present.
if ($env:HF_TOKEN) {
    Write-Host "[ok] HF_TOKEN found in env (length=$($env:HF_TOKEN.Length))"
} else {
    Write-Host "[..] no HF_TOKEN env var; relying on cached `huggingface-cli login` (if you've done it)"
}

# 5. Run the generator with whatever args were passed through.
Write-Host "[..] generating sfx"
& $VenvPy $Driver @PassThru
$rc = $LASTEXITCODE
if ($rc -ne 0) {
    Write-Error "generate.py exited with $rc"
    exit $rc
}
Write-Host "[ok] audioldm2 sfx gen complete"
