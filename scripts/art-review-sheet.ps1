# art-review-sheet.ps1
# Composites the live NPC sprites (game/assets/sprites/npc/*.png) into one
# labeled contact sheet for Matt's art review. NOT a sprite renderer -- it only
# tiles already-rendered PNGs (render-refs remains the only sprite pipeline).
# Native .NET System.Drawing; no ImageMagick, no Python. ASCII-only.

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$repo    = Split-Path $PSScriptRoot -Parent
$npcDir  = Join-Path $repo 'game\assets\sprites\npc'
$outPng  = Join-Path $repo 'scratch\art-review-sheet.png'
New-Item -ItemType Directory -Force -Path (Split-Path $outPng) | Out-Null

# Curriculum order (REF order) so the sheet reads Act 1 -> Act 8.
$order = @(
  'ferris','borrow_checker',
  'smith','cartographer','trait_mage','bellringer','cooper',
  'oracle','herald','twin','tinker','heraldic_sage',
  'forgewright','linguist','pilgrim','drillmaster','reckoner',
  'quartermaster','auditor','chronicler','alchemist',
  'guildmaster','recruiter','locksmith','porter','surveyor','armorer',
  'vexis','wandwright','conjurer','familiar','lanternkeeper','loremaster',
  'barkeep','bouncer','interpreter','mixologist','tabkeeper','cellarer',
  'keymaster','sifter','smelter','tallywright','riveter','bondsmith',
  'dockmaster','lighthousekeeper','signaler','tidewatch','harbormaster','tideforecaster',
  'vaultwright','sharekeeper','warden','swapwarden','strongbox','ghostkeeper'
)

# Sprites flagged "rougher / pending" in 04b batch notes -> highlight border.
$rough = @('quartermaster','auditor','chronicler','alchemist','locksmith','porter','armorer','lanternkeeper','loremaster')

$cell    = 192       # sprite draws into a 192x192 box (nearest-neighbor upscale)
$labelH  = 26
$pad     = 12
$cols    = 8
$cellW   = $cell + $pad
$cellH   = $cell + $labelH + $pad

$items = @()
foreach ($slug in $order) {
  $f = Join-Path $npcDir "$($slug)_idle_0.png"
  if (Test-Path $f) { $items += [pscustomobject]@{ Slug = $slug; Path = $f } }
  else { Write-Host "MISSING: $f" }
}
$rows = [math]::Ceiling($items.Count / $cols)

$W = $cols * $cellW + $pad
$H = $rows * $cellH + $pad + 40
$bmp = New-Object System.Drawing.Bitmap($W, $H)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g.PixelOffsetMode   = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
$g.SmoothingMode     = [System.Drawing.Drawing2D.SmoothingMode]::None
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit

$bg = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(28, 26, 34))
$g.FillRectangle($bg, 0, 0, $W, $H)

$titleFont = New-Object System.Drawing.Font('Consolas', 16, [System.Drawing.FontStyle]::Bold)
$labelFont = New-Object System.Drawing.Font('Consolas', 11)
$white = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$amber = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(240, 200, 120))
$roughPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(220, 90, 90), 3)
$cellPen  = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(70, 66, 80), 1)

$g.DrawString("Pledge & Crown -- NPC art review ($($items.Count) sprites, curriculum order; red border = flagged rough)", $titleFont, $white, 14, 10)

for ($i = 0; $i -lt $items.Count; $i++) {
  $it  = $items[$i]
  $col = $i % $cols
  $row = [math]::Floor($i / $cols)
  $x = $pad + $col * $cellW
  $y = 40 + $pad + $row * $cellH

  $img = [System.Drawing.Image]::FromFile($it.Path)
  $scale = [math]::Floor($cell / [math]::Max($img.Width, $img.Height))
  $dw = $img.Width * $scale
  $dh = $img.Height * $scale
  $ox = $x + [int](($cell - $dw) / 2)
  $oy = $y + [int](($cell - $dh) / 2)
  $g.DrawImage($img, $ox, $oy, $dw, $dh)
  $img.Dispose()

  $pen = if ($rough -contains $it.Slug) { $roughPen } else { $cellPen }
  $g.DrawRectangle($pen, $x, $y, $cell, $cell)

  $brush = if ($rough -contains $it.Slug) { $amber } else { $white }
  $fmt = New-Object System.Drawing.StringFormat
  $fmt.Alignment = [System.Drawing.StringAlignment]::Center
  $g.DrawString($it.Slug, $labelFont, $brush, [single]($x + $cell/2), [single]($y + $cell + 4), $fmt)
}

$g.Dispose()
$bmp.Save($outPng, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Host "Wrote $outPng ($W x $H), $($items.Count) sprites"
