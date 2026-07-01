# qa-shots.ps1 -- full-page screenshot of the deck, sliced into per-slide PNGs
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$pitch  = "J:\pledgeandcrowns\pitch"
$html   = Join-Path $pitch "deck.html"
$qa     = Join-Path $pitch "qa"
New-Item -ItemType Directory -Force -Path $qa | Out-Null
$full   = Join-Path $qa "deck_full.png"

$chrome = "C:\Program Files\Google\Chrome\Application\chrome.exe"
$url = "file:///" + ($html -replace '\\','/')
Remove-Item $full -ErrorAction SilentlyContinue
$cargs = @("--headless=new","--disable-gpu","--no-sandbox","--hide-scrollbars",
           "--force-color-profile=srgb","--window-size=1280,9360",
           "--virtual-time-budget=6000","--screenshot=$full",$url)
$p = Start-Process -FilePath $chrome -ArgumentList $cargs -NoNewWindow -Wait -PassThru
if (-not (Test-Path $full)) { throw "screenshot failed (exit $($p.ExitCode))" }

$img = [System.Drawing.Image]::FromFile($full)
Write-Host ("full shot: {0}x{1}" -f $img.Width, $img.Height)
$slideH = 720
$n = [Math]::Floor($img.Height / $slideH)
for ($i = 0; $i -lt $n; $i++) {
    $rect = New-Object System.Drawing.Rectangle(0, ($i*$slideH), [Math]::Min(1280,$img.Width), $slideH)
    $bmp  = New-Object System.Drawing.Bitmap($rect.Width, $rect.Height)
    $g    = [System.Drawing.Graphics]::FromImage($bmp)
    $g.DrawImage($img, (New-Object System.Drawing.Rectangle(0,0,$rect.Width,$rect.Height)), $rect, [System.Drawing.GraphicsUnit]::Pixel)
    $out = Join-Path $qa ("slide_{0:D2}.png" -f ($i+1))
    $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
}
$img.Dispose()
Write-Host ("sliced {0} slides into {1}" -f $n, $qa)
