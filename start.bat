@echo off
set PATH=C:\msys64\mingw64\bin;%USERPROFILE%\.cargo\bin;%PATH%
cd /d "D:\pc tracker\apps\desktop"
npx tauri dev
