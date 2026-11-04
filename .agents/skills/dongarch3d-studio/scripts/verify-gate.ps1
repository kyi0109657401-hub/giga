<#
.SYNOPSIS
  STRATA · DongArch3D Studio 통합 검증 게이트 실행 스크립트 (Antigravity CLI 표준)

.DESCRIPTION
  `strata-arch3d/` 워크스페이스에서 다음 5단계를 순서대로 검증한다:
    1) cargo fmt-check (rustfmt 포맷 검사)
    2) cargo verify-v1 (clippy --workspace --all-targets -- -D warnings, 경고 0건)
    3) cargo test --workspace (dev 프로파일 전체 단위·통합 테스트)
    4) cargo run --release -p strata-studio -- --verify-all (V1~V8 하네스 검증)
    5) cargo run --release -p strata-studio -- --capture-window <CapturePath> (실제 네이티브 창 프레임버퍼 캡처)

.PARAMETER CapturePath
  실제 실행 창 캡처 이미지를 저장할 경로 (기본값: `target/verify_window_capture.png`).

.PARAMETER Tab
  캡처 시 초기 워크플로 탭 번호 (1..4, 기본 1).

.PARAMETER SkipCapture
  네이티브 창 프레임버퍼 캡처 단계를 건너뛸지 여부.
#>
[CmdletBinding()]
param(
    [string]$CapturePath = "target/verify_window_capture.png",
    [int]$Tab = 1,
    [switch]$SkipCapture
)

$ErrorActionPreference = "Stop"

# 저장소 루트 및 strata-arch3d 경로 탐색
$scriptDir = $PSScriptRoot
$repoRoot = (Resolve-Path (Join-Path $scriptDir "..\..\..\..")).Path
$wsDir = Join-Path $repoRoot "strata-arch3d"

if (-not (Test-Path (Join-Path $wsDir "Cargo.toml"))) {
    throw "strata-arch3d/Cargo.toml not found at: $wsDir"
}

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
    $env:PATH = "$cargoBin;$env:PATH"
}

Push-Location $wsDir
try {
    function Invoke-Step {
        param([string]$Title, [string[]]$CargoArgs)
        Write-Host ""
        Write-Host "=== $Title ===" -ForegroundColor Cyan
        $sw = [Diagnostics.Stopwatch]::StartNew()
        & cargo @CargoArgs
        $code = $LASTEXITCODE
        $sw.Stop()
        if ($code -ne 0) {
            Write-Host ("FAIL: {0} (exit={1}, {2:N1}s)" -f $Title, $code, $sw.Elapsed.TotalSeconds) -ForegroundColor Red
            exit $code
        }
        Write-Host ("PASS: {0} ({1:N1}s)" -f $Title, $sw.Elapsed.TotalSeconds) -ForegroundColor Green
    }

    Invoke-Step -Title "[1/6] cargo fmt-check" -CargoArgs @("fmt", "--all", "--", "--check")
    Invoke-Step -Title "[2/6] cargo verify-v1 (clippy -D warnings)" -CargoArgs @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")
    Invoke-Step -Title "[3/6] cargo test --workspace (dev profile)" -CargoArgs @("test", "--workspace")
    Invoke-Step -Title "[4/6] V1~V8 headless verification harness" -CargoArgs @("run", "--release", "-p", "strata-studio", "--", "--verify-all")
    Invoke-Step -Title "[5/6] SelfAuditor 14-gate autonomous audit (A1~A5, D1~D5, P1~P4)" -CargoArgs @("run", "--release", "-p", "strata-studio", "--", "--self-audit")

    if (-not $SkipCapture) {
        $resolvedCap = if ([System.IO.Path]::IsPathRooted($CapturePath)) {
            $CapturePath
        } else {
            Join-Path $wsDir $CapturePath
        }
        $parentDir = Split-Path $resolvedCap -Parent
        if ($parentDir -and -not (Test-Path $parentDir)) {
            New-Item -ItemType Directory -Force -Path $parentDir | Out-Null
        }
        if (Test-Path $resolvedCap) {
            Remove-Item $resolvedCap -Force
        }
        Invoke-Step -Title "[6/6] Native window swapchain capture (--capture-window)" -CargoArgs @(
            "run", "--release", "-p", "strata-studio", "--", "--tab", "$Tab", "--capture-window", $resolvedCap
        )
        if (-not (Test-Path $resolvedCap)) {
            Write-Host "FAIL: 캡처 이미지 파일이 생성되지 않았습니다: $resolvedCap" -ForegroundColor Red
            exit 1
        }
        $len = (Get-Item $resolvedCap).Length
        if ($len -lt 5000) {
            Write-Host "FAIL: 캡처 이미지 크기가 비정상적으로 작습니다 ($len bytes): $resolvedCap" -ForegroundColor Red
            exit 1
        }
        Write-Host ("PASS: 실제 창 캡처 완료 ({0:N0} bytes) -> {1}" -f $len, $resolvedCap) -ForegroundColor Green
    }
}
finally {
    Pop-Location
}
