@echo off
set "APP=%~dp0src-tauri\target\release\desktop-pet.exe"
if exist "%APP%" (
  start "" "%APP%"
  exit /b 0
)
cd /d "%~dp0"
start "小团子桌宠" cmd /c "npm install && npm run tauri dev"
