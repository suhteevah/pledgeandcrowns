# Verify batch-5 ref grids: every row exactly 32 chars, only valid palette chars.
# ASCII-only. Run from repo root.
$ErrorActionPreference = 'Stop'

$valid = 'KYOWRPUZNGLCADTIMJFEHQXBSV*%<>!#. '.ToCharArray()
$validSet = @{}
foreach ($c in $valid) { $validSet[$c] = $true }

$refs = 30..35
$anyBad = $false

foreach ($n in $refs) {
    $glob = Get-ChildItem "design\art\refs\ref-$n-*.jsx"
    $path = $glob.FullName
    $lines = Get-Content $path
    $inGrid = $false
    $rowNum = 0
    foreach ($line in $lines) {
        if ($line -match "const REF${n}_GRID = \[") { $inGrid = $true; continue }
        if ($inGrid -and $line -match '^\];') { $inGrid = $false; continue }
        if ($inGrid) {
            $m = [regex]::Match($line, "'([^']*)'")
            if (-not $m.Success) { continue }
            $row = $m.Groups[1].Value
            $len = $row.Length
            $bad = @()
            foreach ($ch in $row.ToCharArray()) {
                if (-not $validSet.ContainsKey($ch)) { $bad += $ch }
            }
            if ($len -ne 32 -or $bad.Count -gt 0) {
                $anyBad = $true
                $badStr = ($bad | Select-Object -Unique) -join ''
                Write-Output "REF$n row $rowNum LEN=$len BADCHARS=[$badStr]  >>$row<<"
            }
            $rowNum++
        }
    }
    Write-Output "REF$n : checked $rowNum rows"
}

if ($anyBad) { Write-Output "RESULT: ERRORS FOUND" } else { Write-Output "RESULT: ALL GRIDS OK (32x32, valid palette)" }
