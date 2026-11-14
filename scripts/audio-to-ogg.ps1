# Convert game/assets/audio/*.wav to Ogg Vorbis (quality 5) via ffmpeg.
#
# Why: bevy_kira_audio is built with its default feature set, which is
# "ogg" only - kira's "wav" decoder is NOT compiled in, so .wav assets
# never decode. Ogg is also ~10x smaller, which matters for the web build.
#
# Run after scripts/synthwave-gen.ps1 (which bakes .wav masters):
#   powershell -ExecutionPolicy Bypass -File scripts/audio-to-ogg.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/audio-to-ogg.ps1 -KeepWav
#
# Requires ffmpeg on PATH with the libvorbis encoder. No Python.

param(
    [int]$Quality = 5,
    [switch]$KeepWav
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== audio-to-ogg (vorbis q$Quality) ===" -ForegroundColor Cyan

$ffmpeg = Get-Command ffmpeg -ErrorAction SilentlyContinue
if (-not $ffmpeg) {
    Write-Host "[FAIL] ffmpeg not on PATH. Owner step: install ffmpeg (e.g. winget install Gyan.FFmpeg)." -ForegroundColor Red
    exit 1
}
Write-Host "[info] ffmpeg: $($ffmpeg.Source)"

$encoders = & ffmpeg -hide_banner -encoders 2>$null | Out-String
if ($encoders -notmatch "libvorbis") {
    Write-Host "[FAIL] this ffmpeg build has no libvorbis encoder." -ForegroundColor Red
    exit 1
}

$audioDir = "game/assets/audio"
$wavs = @(Get-ChildItem -Path $audioDir -Filter *.wav -ErrorAction SilentlyContinue)
if ($wavs.Count -eq 0) {
    Write-Host "[skip] no .wav files in $audioDir - nothing to convert." -ForegroundColor Yellow
    exit 0
}

$totalIn = 0
$totalOut = 0
foreach ($wav in $wavs) {
    $ogg = [System.IO.Path]::ChangeExtension($wav.FullName, ".ogg")
    Write-Host "[convert] $($wav.Name) -> $([System.IO.Path]::GetFileName($ogg))"
    & ffmpeg -hide_banner -loglevel error -y -i $wav.FullName -c:a libvorbis -q:a $Quality $ogg
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[FAIL] ffmpeg exit $LASTEXITCODE on $($wav.Name)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    $inLen = $wav.Length
    $outLen = (Get-Item $ogg).Length
    $totalIn += $inLen
    $totalOut += $outLen
    Write-Host ("          {0:N0} B -> {1:N0} B" -f $inLen, $outLen)
    if (-not $KeepWav) {
        Remove-Item -Force $wav.FullName
    }
}

Write-Host ("[ok] audio total {0:N2} MB -> {1:N2} MB" -f ($totalIn / 1MB), ($totalOut / 1MB)) -ForegroundColor Green
if ($KeepWav) {
    Write-Host "[info] -KeepWav set: .wav masters left in place (they are copied into web/assets by web-build.ps1)." -ForegroundColor Yellow
}
