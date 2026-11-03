@echo off
REM STRATA · DongArch3D v2.0 (Rust Workspace) - Run All Verification Gates
REM Copyright (C) 2025-2026 Dongguk University Cultural Heritage Research Institute

set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..

powershell -NoProfile -ExecutionPolicy Bypass -File "%PROJECT_ROOT%\.agents\skills\dongarch3d-studio\scripts\verify-gate.ps1" %*
exit /b %ERRORLEVEL%
