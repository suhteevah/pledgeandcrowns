# Nearest-neighbor upscale batch-5 PNGs x8 for visual inspection. ASCII-only.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$srcDir = "J:\pledgeandcrowns\design\art\refs\png"
$outDir = "J:\pledgeandcrowns\scratch\batch5-preview"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$scale = 8

foreach ($n in 30..35) {
    $src = Join-Path $srcDir "REF$n.png"
    $img = [System.Drawing.Image]::FromFile($src)
    $w = $img.Width * $scale
    $h = $img.Height * $scale
    $bmp = New-Object System.Drawing.Bitmap($w, $h)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $g.DrawImage($img, 0, 0, $w, $h)
    $dst = Join-Path $outDir "REF${n}_x8.png"
    $bmp.Save($dst, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose(); $img.Dispose()
    Write-Output "wrote $dst"
}
