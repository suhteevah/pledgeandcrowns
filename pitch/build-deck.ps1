# build-deck.ps1  --  assemble the Pledge & Crown pitch deck (HTML -> PDF)
# ASCII-only. Embeds REAL game assets as base64, renders via headless Chrome.
$ErrorActionPreference = "Stop"

$root     = "J:\pledgeandcrowns"
$pitch    = Join-Path $root "pitch"
$assets   = Join-Path $root "game\assets\sprites"
$npcDir   = Join-Path $assets "npc"
$template = Join-Path $pitch "deck.template.html"
$outHtml  = Join-Path $pitch "deck.html"
$outPdf   = Join-Path $pitch "Pledge-and-Crown-Pitch.pdf"

function To-DataUri([string]$path) {
    $bytes = [IO.File]::ReadAllBytes($path)
    $b64   = [Convert]::ToBase64String($bytes)
    return "data:image/png;base64,$b64"
}

function Title-Case([string]$slug) {
    ($slug -split '_' | ForEach-Object {
        if ($_.Length -gt 0) { $_.Substring(0,1).ToUpper() + $_.Substring(1) } else { $_ }
    }) -join ' '
}

Write-Host "Encoding hero assets..."
$shots = Join-Path $pitch "world-shots"
$tplToken = @{
    '@@title@@'        = To-DataUri (Join-Path $assets "title.png")
    '@@world_marker@@' = To-DataUri (Join-Path $shots "shot-ferris-marker.png")
    '@@world_south@@'  = To-DataUri (Join-Path $shots "shot-south.png")
    '@@borrow@@'       = To-DataUri (Join-Path $npcDir "borrow_checker_idle_0.png")
}

Write-Host "Building cast grid..."
# Curated lead order so recognizable NPCs land in row 1; rest alphabetical.
$lead = @('ferris','borrow_checker','trait_mage','oracle','guildmaster',
          'quartermaster','smith','herald','barkeep','alchemist','armorer','loremaster')

$allFiles = Get-ChildItem $npcDir -Filter *_idle_0.png | Sort-Object Name
$bySlug = @{}
foreach ($f in $allFiles) {
    $slug = $f.Name -replace '_idle_0\.png$',''
    $bySlug[$slug] = $f.FullName
}

$ordered = @()
foreach ($s in $lead) { if ($bySlug.ContainsKey($s)) { $ordered += $s } }
foreach ($s in ($bySlug.Keys | Sort-Object)) { if ($lead -notcontains $s) { $ordered += $s } }

$cells = New-Object System.Text.StringBuilder
foreach ($slug in $ordered) {
    $uri  = To-DataUri $bySlug[$slug]
    $name = Title-Case $slug
    [void]$cells.AppendLine("    <figure><img class=""pixel"" src=""$uri""><figcaption>$name</figcaption></figure>")
}
Write-Host ("  cast cells: {0}" -f $ordered.Count)

Write-Host "Injecting into template..."
$html = [IO.File]::ReadAllText($template)
foreach ($k in $tplToken.Keys) { $html = $html.Replace($k, $tplToken[$k]) }
$html = $html.Replace('@@CAST_GRID@@', $cells.ToString())
[IO.File]::WriteAllText($outHtml, $html, (New-Object System.Text.UTF8Encoding($false)))
Write-Host ("  wrote {0} ({1:N0} bytes)" -f $outHtml, (Get-Item $outHtml).Length)

Write-Host "Rendering PDF via headless Chrome..."
$chrome = "C:\Program Files\Google\Chrome\Application\chrome.exe"
if (-not (Test-Path $chrome)) { $chrome = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" }
$url = "file:///" + ($outHtml -replace '\\','/')
Remove-Item $outPdf -ErrorAction SilentlyContinue
$cargs = @(
    "--headless=new","--disable-gpu","--no-sandbox","--no-pdf-header-footer",
    "--force-color-profile=srgb","--run-all-compositor-stages-before-draw",
    "--virtual-time-budget=6000","--print-to-pdf=$outPdf",$url
)
$p = Start-Process -FilePath $chrome -ArgumentList $cargs -NoNewWindow -Wait -PassThru
if ((Test-Path $outPdf) -and (Get-Item $outPdf).Length -gt 0) {
    Write-Host ("OK -> {0} ({1:N0} bytes, exit {2})" -f $outPdf, (Get-Item $outPdf).Length, $p.ExitCode)
} else {
    throw "PDF render failed (exit $($p.ExitCode))"
}
