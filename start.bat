@echo off
set PATH=%USERPROFILE%\.cargo\bin;%USERPROFILE%\.rustup\bin;%PATH%
rustup default stable-x86_64-pc-windows-msvc >nul 2>&1
cd /d "D:\pc tracker\apps\desktop"
npx tauri dev
