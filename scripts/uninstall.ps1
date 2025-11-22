# Requires: Windows PowerShell 5.1+ or PowerShell 7+
# Purpose: Uninstall lst from a user-local bin and optionally remove it from PATH.

[CmdletBinding()]
param(
  [string]$InstallDir = "$env:LOCALAPPDATA\lst\bin",
  [switch]$RemoveFromPath,
  [switch]$Force,
  [switch]$Purge
)

$ErrorActionPreference = 'Stop'

function Write-Info { param([string]$Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Ok   { param([string]$Message) Write-Host $Message -ForegroundColor Green }
function Write-Warn { param([string]$Message) Write-Warning $Message }

function Remove-FromUserPath {
  param([Parameter(Mandatory)][string]$Dir)
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  if (-not $userPath) { return }
  $parts = ($userPath -split ';') | Where-Object { $_ -ne '' }
  $newParts = @()
  foreach ($p in $parts) {
    if ($p.TrimEnd('\') -ieq $Dir.TrimEnd('\')) { continue }
    $newParts += $p
  }
  $newPath = ($newParts -join ';').Trim(';')
  if ($newPath -ne $userPath) {
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    # Update current session too
    $envParts = ($env:Path -split ';') | Where-Object { $_ -ne '' }
    $envParts = $envParts | Where-Object { $_.TrimEnd('\\') -ine $Dir.TrimEnd('\\') }
    $env:Path = ($envParts -join ';').Trim(';')
    Write-Info "Removed from user PATH: $Dir"
  }
}

try {
  $exePath = Join-Path $InstallDir 'lst.exe'
  if (-not (Test-Path -LiteralPath $exePath)) {
    Write-Warn "lst.exe not found at: $exePath"
  } else {
    if (-not $Force) {
      $ans = Read-Host "Remove $exePath ? (y/N)"
      if ($ans -notin @('y','Y')) { throw "Uninstall aborted by user." }
    }

    try {
      Remove-Item -LiteralPath $exePath -Force
      Write-Ok "Removed: $exePath"
    } catch {
      throw "Failed to remove $exePath. Make sure it's not in use and try again with PowerShell as the same user. Details: $($_.Exception.Message)"
    }
  }

  if ($RemoveFromPath) {
    Remove-FromUserPath -Dir $InstallDir
  }

  if ($Purge) {
    # Attempt to remove empty install dir and parent lst folder if empty
    foreach ($dir in @($InstallDir, (Split-Path $InstallDir -Parent))) {
      if ($dir -and (Test-Path -LiteralPath $dir)) {
        try {
          $items = Get-ChildItem -LiteralPath $dir -Force -ErrorAction SilentlyContinue
          if (-not $items) { Remove-Item -LiteralPath $dir -Force; Write-Info "Removed empty folder: $dir" }
        } catch {}
      }
    }
  }

  Write-Ok "Uninstall complete. Open a new terminal to refresh PATH if changed."
}
catch {
  Write-Error ($_.Exception.Message)
  exit 1
}
