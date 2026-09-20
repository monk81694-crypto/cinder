@echo off
rem Cinder Launcher - double-click to play. No terminal needed.
title Cinder Launcher
cd /d "%~dp0"

where npm >nul 2>nul
if errorlevel 1 (
  echo [Cinder] Node.js / npm not found.
  echo [Cinder] Install Node.js LTS from https://nodejs.org/ , then double-click this again.
  pause
  exit /b 1
)

if not exist "node_modules\electron\dist\electron.exe" (
  echo [Cinder] First run - installing dependencies, this takes a minute...
  call npm install
  if errorlevel 1 (
    echo [Cinder] npm install failed. See above.
    pause
    exit /b 1
  )
)

call npm start
if errorlevel 1 (
  echo.
  echo [Cinder] The launcher exited with an error. See the lines above.
  pause
)
