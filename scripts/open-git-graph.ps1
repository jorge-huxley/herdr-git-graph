# open-git-graph.ps1 — Windows launcher for split-pane git graph.
$ErrorActionPreference = 'Continue'

$HerdrBin = if ($env:HERDR_BIN_PATH) { $env:HERDR_BIN_PATH } else { 'herdr' }

function Strip-Verbatim([string]$p) {
  if ($p -and $p.StartsWith('\\?\')) { return $p.Substring(4) }
  return $p
}

$PluginRoot = Strip-Verbatim (Split-Path -Parent $PSScriptRoot)
$ViewerBin = Join-Path $PluginRoot 'target\release\herdr-git-graph.exe'
$PaneLabel = 'Git Graph'

function Get-UserCwd {
  try {
    $focused = (& $HerdrBin pane list | ConvertFrom-Json).result.panes |
      Where-Object { $_.focused } | Select-Object -First 1
    if ($focused -and $focused.cwd) { return Strip-Verbatim $focused.cwd }
  } catch {}
  return $PluginRoot
}

function Get-PaneId([string]$json) {
  return ([regex]'"pane_id":"([^"]+)"').Match($json).Groups[1].Value
}

function Open-Pane {
  $cwd = Get-UserCwd
  $out = (& $HerdrBin pane split --direction right --cwd $cwd --focus | Out-String)
  $np = Get-PaneId $out
  if ($np) {
    & $HerdrBin pane run $np "& \`"$ViewerBin\`""
    & $HerdrBin pane rename $np $PaneLabel *> $null
  }
  exit 0
}

$Decision = 'OPEN'
if (Test-Path $ViewerBin) {
  $panes = & $HerdrBin pane list 2>$null
  if ($LASTEXITCODE -ne 0) { $panes = $null }
  if ($panes) {
    $panesText = ($panes -join "`n")
    $Decision = ($panesText | & $ViewerBin --launch-decision 2>$null)
    if ($LASTEXITCODE -ne 0 -or -not $Decision) { $Decision = 'OPEN' }
  }
}

if ($Decision -like 'FOCUS *') {
  $PaneId = $Decision.Substring(6)
  & $HerdrBin pane zoom $PaneId --on *> $null
  & $HerdrBin pane zoom $PaneId --off
  exit $LASTEXITCODE
} elseif ($Decision -like 'CLOSE *') {
  $PaneId = $Decision.Substring(6)
  & $HerdrBin pane close $PaneId
  exit $LASTEXITCODE
} else {
  Open-Pane
}
