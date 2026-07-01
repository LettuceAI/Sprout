@echo off
setlocal enabledelayedexpansion

set "DIR=%~dp0"
set "DEST=%LOCALAPPDATA%\Programs\sprout"

if not exist "%DIR%sprout.exe" (
  echo error: sprout.exe not found next to this script 1>&2
  exit /b 1
)

if not exist "%DEST%" mkdir "%DEST%"
copy /y "%DIR%sprout.exe" "%DEST%\sprout.exe" >nul
echo Installed sprout to %DEST%\sprout.exe

echo %PATH% | find /i "%DEST%" >nul
if errorlevel 1 (
  echo Add %DEST% to your PATH to run 'sprout' directly.
)

echo Start it with:            "%DEST%\sprout.exe"
echo Start it in background:   start "" /b "%DEST%\sprout.exe"
endlocal
