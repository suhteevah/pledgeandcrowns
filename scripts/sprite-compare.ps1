# sprite-compare.ps1 - upscaled montage of specific rendered NPC sprites for
# side-by-side review. Args: pairs of "slug" (looks up game/assets/sprites/npc).
# Native .NET System.Drawing, nearest-neighbor upscale. ASCII-only.
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Slugs)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$repo = Split-Path $PSScriptRoot -Parent
$npc = Join-Path $repo 'game\assets\sprites\npc'
$out = Join-Path $repo 'scratch\sprite-compare.png'
New-Item -ItemType Directory -Force -Path (Split-Path $out) | Out-Null

$scale = 9
$cell = 32 * $scale          # 288
$labelH = 26
$pad = 16
$cols = $Slugs.Count
$W = $cols * ($cell + $pad) + $pad
$H = $cell + $labelH + 2 * $pad + 30

$bmp = New-Object System.Drawing.Bitmap($W, $H)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
$g.FillRectangle((New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(28,26,34))), 0, 0, $W, $H)
$titleFont = New-Object System.Drawing.Font('Consolas', 13, [System.Drawing.FontStyle]::Bold)
$labelFont = New-Object System.Drawing.Font('Consolas', 12)
$white = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$g.DrawString("Re-rendered from approved claude.ai/design grids (9x)", $titleFont, $white, 14, 8)

for ($i = 0; $i -lt $Slugs.Count; $i++) {
  $slug = $Slugs[$i]
  $f = Join-Path $npc "$($slug)_idle_0.png"
  $x = $pad + $i * ($cell + $pad)
  $y = 40 + $pad
  if (Test-Path $f) {
    $img = [System.Drawing.Image]::FromFile($f)
    $g.DrawImage($img, $x, $y, $cell, $cell)
    $img.Dispose()
  }
  $fmt = New-Object System.Drawing.StringFormat; $fmt.Alignment = [System.Drawing.StringAlignment]::Center
  $g.DrawString($slug, $labelFont, $white, [single]($x + $cell/2), [single]($y + $cell + 4), $fmt)
}
$g.Dispose()
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
Write-Host "Wrote $out ($W x $H)"
