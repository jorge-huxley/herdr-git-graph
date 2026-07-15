# fetch-or-build.ps1 — herdr [[build]] step for herdr-git-graph (Windows).
$ErrorActionPreference = 'Stop'

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = if ($env:GG_REPO_ROOT) { $env:GG_REPO_ROOT } else { Split-Path -Parent $ScriptDir }
$CargoToml = if ($env:GG_CARGO_TOML) { $env:GG_CARGO_TOML } else { Join-Path $RepoRoot 'Cargo.toml' }
$Out = if ($env:GG_OUT) { $env:GG_OUT } else { Join-Path $RepoRoot 'target\release\herdr-git-graph.exe' }
$BaseUrl = if ($env:GG_BASE_URL) { $env:GG_BASE_URL } else { 'https://github.com/jorge-huxley/herdr-git-graph/releases/download' }

function Build-FromSource {
  $cargo = Get-Command cargo -ErrorAction SilentlyContinue
  if (-not $cargo) {
    Write-Error "herdr-git-graph needs Rust to build, but cargo was not found. Install from https://rustup.rs"
  }
  Push-Location $RepoRoot
  try {
    & cargo build --release
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  } finally {
    Pop-Location
  }
  exit 0
}

function Fallback([string]$Reason) {
  Write-Host "herdr-git-graph: $Reason — building from source instead." -ForegroundColor Yellow
  Build-FromSource
}

$arch = $env:PROCESSOR_ARCHITECTURE
$triple = switch ($arch) {
  'AMD64' { 'x86_64-pc-windows-msvc' }
  default { $null }
}
if (-not $triple) { Fallback "no prebuilt binary for Windows/$arch" }

$versionLine = Select-String -Path $CargoToml -Pattern '^version\s*=' | Select-Object -First 1
if (-not $versionLine) { Fallback "could not read version from Cargo.toml" }
$version = ($versionLine.Line -replace '^version\s*=\s*"([^"]+)".*', '$1').Trim('"')
if (-not $version) { Fallback "could not parse version from Cargo.toml" }

$asset = "herdr-git-graph-$triple"
$tmpdir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_.FullName }
$tmpbin = Join-Path $tmpdir $asset
$tmpsums = Join-Path $tmpdir 'SHA256SUMS'
$binUrl = "$BaseUrl/v$version/$asset"
$sumsUrl = "$BaseUrl/v$version/SHA256SUMS"

try {
  Invoke-WebRequest -Uri $binUrl -OutFile $tmpbin -UseBasicParsing
} catch {
  Fallback "prebuilt binary not available for v$version ($asset)"
}

try {
  Invoke-WebRequest -Uri $sumsUrl -OutFile $tmpsums -UseBasicParsing
} catch {
  Fallback "checksums not available for v$version"
}

$expected = (Select-String -Path $tmpsums -Pattern "^[0-9a-f]{64} [ *]$asset\$" | Select-Object -First 1).Line
if (-not $expected) { Fallback "no checksum listed for $asset" }
$expectedHash = ($expected -split '\s+')[0]

$hash = (Get-FileHash -Path $tmpbin -Algorithm SHA256).Hash.ToLower()
if ($hash -ne $expectedHash) {
  Fallback "checksum mismatch for $asset"
}

$outDir = Split-Path -Parent $Out
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Copy-Item -Force $tmpbin $Out
Write-Host "herdr-git-graph: installed prebuilt v$version ($triple), verified SHA-256."
exit 0
