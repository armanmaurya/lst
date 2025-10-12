# Requires: Windows PowerShell 5.1+ or PowerShell 7+
# Purpose: Install the latest nva Windows release to a user-local bin and add it to PATH.
#
# Usage (Remote):
#   iwr -useb https://raw.githubusercontent.com/armanmaurya/nva/main/scripts/install.ps1 | iex
#   Or with options:
#   & ([scriptblock]::Create((iwr -useb https://raw.githubusercontent.com/armanmaurya/nva/main/scripts/install.ps1))) -Force
#
# Usage (Local):
#   powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1
#   Or:
#   .\scripts\install.ps1 -Force -InstallDir "C:\custom\path"

[CmdletBinding()]
param(
  [string]$Repo = 'armanmaurya/nva',
  [string]$InstallDir = "$env:LOCALAPPDATA\nva\bin",
  [switch]$Force
)

$ErrorActionPreference = 'Stop'
try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 } catch {}

function Write-Info {
  param([string]$Message)
  Write-Host $Message -ForegroundColor Cyan
}

function Ensure-Dir {
  param([Parameter(Mandatory)][string]$Path)
  if (-not (Test-Path -LiteralPath $Path)) {
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
  }
}

function Add-ToUserPath {
  param([Parameter(Mandatory)][string]$Dir)
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  $paths = ($userPath -split ';') | Where-Object { $_ -and $_.Trim() } | ForEach-Object { $_.Trim() }
  if ($paths -notcontains $Dir) {
    $newPath = @($paths + $Dir) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    if ($env:Path -notmatch [Regex]::Escape($Dir)) { $env:Path = "$env:Path;$Dir" }
    Write-Info "Added to user PATH: $Dir"
  } else {
    Write-Verbose "Path already contains: $Dir"
  }
}

function Get-LatestReleaseAsset {
  param([Parameter(Mandatory)][string]$Repo)
  $uri = "https://api.github.com/repos/$Repo/releases/latest"
  $headers = @{ 'User-Agent' = 'nva-installer'; 'Accept' = 'application/vnd.github+json' }
  Write-Verbose "Fetching latest release JSON: $uri"
  $release = Invoke-RestMethod -Uri $uri -Headers $headers -Method GET
  if (-not $release.assets) { throw "No assets found in the latest release." }

  $assets = @($release.assets)

  # Prefer Windows x86_64 zip/exe assets. Try common patterns first.
  $candidates = $assets | Where-Object { $_.name -match '(?i)(windows|win).*(x86_64|amd64).*(\.zip|\.exe)$' }
  if (-not $candidates) { $candidates = $assets | Where-Object { $_.name -match '(?i)(x86_64|amd64).*(\.zip|\.exe)$' } }
  if (-not $candidates) { $candidates = $assets | Where-Object { $_.name -match '(?i)(windows|win).*(\.zip|\.exe)$' } }
  if (-not $candidates) { $candidates = $assets | Where-Object { $_.name -match '(?i)\.zip$|\.exe$' } }

  $asset = $candidates | Select-Object -First 1
  if (-not $asset) { throw "Unable to find a suitable Windows asset in the latest release." }

  [PSCustomObject]@{
    Version = $release.tag_name
    Name    = $asset.name
    Url     = $asset.browser_download_url
    IsZip   = ($asset.name -match '(?i)\.zip$')
  }
}

function Install-FromZip {
  param(
    [Parameter(Mandatory)][string]$ZipPath,
    [Parameter(Mandatory)][string]$InstallDir
  )
  $extractDir = Join-Path ([IO.Path]::GetDirectoryName($ZipPath)) 'extracted'
  Ensure-Dir $extractDir
  Write-Info "Extracting archive..."
  Expand-Archive -Path $ZipPath -DestinationPath $extractDir -Force
  $exe = Get-ChildItem -Path $extractDir -Recurse -Filter 'nva.exe' | Select-Object -First 1
  if (-not $exe) { throw "nva.exe not found in the extracted archive." }
  Ensure-Dir $InstallDir
  $destExe = Join-Path $InstallDir 'nva.exe'
  Copy-Item -Path $exe.FullName -Destination $destExe -Force
  $destExe
}

function Install-FromExeAsset {
  param(
    [Parameter(Mandatory)][string]$ExePath,
    [Parameter(Mandatory)][string]$InstallDir
  )
  Ensure-Dir $InstallDir
  $destExe = Join-Path $InstallDir 'nva.exe'
  Copy-Item -Path $ExePath -Destination $destExe -Force
  $destExe
}

try {
  $asset = Get-LatestReleaseAsset -Repo $Repo
  Write-Info "Latest release: $($asset.Version)"
  Write-Info "Asset: $($asset.Name)"

  $tmpDir = Join-Path $env:TEMP ("nva-install-" + [Guid]::NewGuid().ToString('N'))
  Ensure-Dir $tmpDir
  $downloadPath = Join-Path $tmpDir $asset.Name

  if ((Test-Path $InstallDir) -and (Test-Path (Join-Path $InstallDir 'nva.exe')) -and -not $Force) {
    Write-Host "nva is already installed at: $(Join-Path $InstallDir 'nva.exe')"
    $choice = Read-Host "Overwrite existing file? (y/N)"
    if ($choice -notin @('y','Y')) { throw "Installation aborted by user." }
  }

  Write-Info "Downloading asset..."
  Invoke-WebRequest -Uri $asset.Url -OutFile $downloadPath -UseBasicParsing

  if ($asset.IsZip) {
    $installedExe = Install-FromZip -ZipPath $downloadPath -InstallDir $InstallDir
  } else {
    $installedExe = Install-FromExeAsset -ExePath $downloadPath -InstallDir $InstallDir
  }

  Add-ToUserPath -Dir $InstallDir
  Write-Host "Installed: $installedExe" -ForegroundColor Green
  Write-Host "Open a new terminal and run: nva --help" -ForegroundColor Green
}
catch {
  Write-Error ($_.Exception.Message)
  exit 1
}
finally {
  if ($tmpDir -and (Test-Path $tmpDir)) { try { Remove-Item -Recurse -Force -Path $tmpDir | Out-Null } catch {} }
}
