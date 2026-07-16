@echo off
set PATH=%USERPROFILE%\.cargo\bin;%USERPROFILE%\.rustup\bin;%PATH%
cd /d "D:\pc tracker\apps\desktop"
npx tauri dev
