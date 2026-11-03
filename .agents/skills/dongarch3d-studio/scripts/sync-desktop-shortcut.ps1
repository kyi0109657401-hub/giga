<#
.SYNOPSIS
  STRATA 데스크톱 바로가기 고정 바이너리(`%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe`) 빌드 및 동기화 스크립트

.DESCRIPTION
  `strata-arch3d`에서 `cargo build --release -p strata-studio`를 실행한 뒤,
  산출물을 `%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe`에 복사하고 SHA256 해시 일치를 검증한다.
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$scriptDir = $PSScriptRoot
$repoRoot = (Resolve-Path (Join-Path $scriptDir "..\..\..\..")).Path
$wsDir = Join-Path $repoRoot "strata-arch3d"
$srcExe = Join-Path $wsDir "target\release\strata-studio.exe"
$dstDir = Join-Path $env:LOCALAPPDATA "Programs\STRATA"
$dstExe = Join-Path $dstDir "strata-studio.exe"

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
    $env:PATH = "$cargoBin;$env:PATH"
}

Push-Location $wsDir
try {
    if (-not $SkipBuild) {
        Write-Host "=== Building strata-studio (release) ===" -ForegroundColor Cyan
        & cargo build --release -p strata-studio
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build --release -p strata-studio failed with exit code $LASTEXITCODE"
        }
    }

    if (-not (Test-Path $srcExe)) {
        throw "Release binary not found at: $srcExe"
    }

    if (-not (Test-Path $dstDir)) {
        New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
    }

    Copy-Item -Path $srcExe -Destination $dstExe -Force

    $srcHash = (Get-FileHash -Path $srcExe -Algorithm SHA256).Hash
    $dstHash = (Get-FileHash -Path $dstExe -Algorithm SHA256).Hash
    $sizeBytes = (Get-Item $dstExe).Length

    if ($srcHash -ne $dstHash) {
        throw "SHA256 mismatch after copy! src=$srcHash dst=$dstHash"
    }

    Write-Host ("OK: Synced strata-studio.exe ({0:N0} bytes, SHA256={1})" -f $sizeBytes, $dstHash) -ForegroundColor Green
    Write-Host "    Source: $srcExe"
    Write-Host "    Target: $dstExe"
}
finally {
    Pop-Location
}
