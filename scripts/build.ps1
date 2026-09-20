<#
.SYNOPSIS
  Verify the Cinder Electron launcher (typecheck + tests).
.DESCRIPTION
  Runs `npm run typecheck` as warn-only (failures do not stop the build),
  then runs `npm test` which must pass.
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts/build.ps1
#>
$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $Root
try {
  Write-Host '=== Stage 1: typecheck (warn-only) ==='
  try {
    npm run typecheck
    if ($LASTEXITCODE -ne 0) { throw "typecheck exited with code $LASTEXITCODE" }
  }
  catch {
    Write-Warning "typecheck failed (warn-only, continuing): $_"
  }

  Write-Host '=== Stage 2: tests (must pass) ==='
  npm test
  if ($LASTEXITCODE -ne 0) { throw "npm test failed with exit code $LASTEXITCODE" }

  Write-Host '=== Build checks passed ==='
}
finally {
  Pop-Location
}
