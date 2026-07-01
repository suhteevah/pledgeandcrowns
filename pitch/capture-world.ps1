# capture-world.ps1 -- NON-INTRUSIVE in-engine capture.
# The game (PNC_SHOTS mode) self-drives through framed vantages and writes PNGs
# via Bevy's own screenshot API. This launcher only sets env, launches, and waits
# for the files. No focus stealing, no key injection, nothing that can land in
# whatever the user is typing. The window opens UNFOCUSED in a corner.
$ErrorActionPreference = "Stop"

$rel = "G:\cargo-target\pledgeandcrown\release\pledge_and_crown.exe"
$dbg = "G:\cargo-target\pledgeandcrown\debug\pledge_and_crown.exe"
$exe = if (Test-Path $rel) { $rel } else { $dbg }
$out = "J:\pledgeandcrowns\pitch\world-shots"
New-Item -ItemType Directory -Force -Path $out | Out-Null

$expected = @("shot-ferris-marker.png","shot-south.png","shot-street.png","shot-meadow.png")
foreach ($f in $expected) { Remove-Item (Join-Path $out $f) -ErrorAction SilentlyContinue }

Get-Process pledge_and_crown -ErrorAction SilentlyContinue | Stop-Process -Force
$env:BEVY_ASSET_ROOT = "J:\pledgeandcrowns\game"
$env:PNC_SHOTS = "1"
$env:PNC_SHOTS_DIR = $out
$env:RUST_LOG = "warn,pledge_and_crown=info"

Write-Host "Launching game in screenshot mode (unfocused, self-driving)..."
$exe_used = $exe
Write-Host "  exe: $exe_used"
$proc = Start-Process -FilePath $exe -PassThru

try {
    $deadline = 45
    $elapsed = 0
    while ($elapsed -lt $deadline) {
        Start-Sleep -Milliseconds 700
        $elapsed += 0.7
        if ($proc.HasExited) { Write-Host "  (process exited on its own)"; break }
        $have = @($expected | Where-Object { Test-Path (Join-Path $out $_) })
        if ($have.Count -eq $expected.Count) { Write-Host "  all $($expected.Count) shots written"; break }
    }
    Start-Sleep -Milliseconds 800  # let the final PNG flush to disk
}
finally {
    Get-Process pledge_and_crown -ErrorAction SilentlyContinue | Stop-Process -Force
}

Write-Host "--- results ---"
foreach ($f in $expected) {
    $p = Join-Path $out $f
    if (Test-Path $p) { "  OK   $f  ($([math]::Round((Get-Item $p).Length/1KB)) KB)" }
    else { "  MISS $f" }
}
