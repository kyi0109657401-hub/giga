@echo off
REM STRATA · DongArch3D v2.0 (Rust Workspace) - Release Build & Desktop Shortcut Sync
REM Copyright (C) 2025-2026 Dongguk University Cultural Heritage Research Institute

set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..

powershell -NoProfile -ExecutionPolicy Bypass -File "%PROJECT_ROOT%\.agents\skills\dongarch3d-studio\scripts\sync-desktop-shortcut.ps1" %*
exit /b %ERRORLEVEL%
