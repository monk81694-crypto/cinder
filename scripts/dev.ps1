<#
.SYNOPSIS
  Start the Cinder Electron launcher for local development.
.DESCRIPTION
  Installs npm dependencies with `npm install` if node_modules is missing,
  then starts the app with `npm start`.
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts/dev.ps1
#>
$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $Root
try {
  if (-not (Test-Path -LiteralPath (Join-Path $Root 'node_modules'))) {
    Write-Host '[dev] node_modules missing - running npm install...'
    npm install
  }
  Write-Host '[dev] starting app (npm start)...'
  npm start
}
finally {
  Pop-Location
}
