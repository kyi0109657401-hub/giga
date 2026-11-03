# factory/project-profile.md — 에이전트가 가장 먼저 읽는 프로젝트 프로필

갱신: 2026-11-03. 주력 앱 본체가 `strata-arch3d/` (Rust 1.91.1 워크스페이스, 8개 크레이트)로 전환 완료됨을 반영.

## 한 줄 정의
Artec 등 3D 스캐너의 유물 메시(`PLY`/`OBJ`/`STL` + `MTL`/`JPG`/`AP2`, 단위 `mm`) → 정위치(바닥 수평·회전 중심축) → 단면선·외곽선·정밀 탁본·치수 → 1:1 mm 실측 도판(`SVG`/`DXF`/`PDF`/`ret/`, Illustrator 레이어 분리). 초보자가 4단계 워크플로 탭(가져오기 → 정위치·접합 → 단면·정밀 탁본 → 도판·내보내기)으로 끝내는 Windows 네이티브 데스크톱 앱.

## 스택
- **주력 앱 본체 (`strata-arch3d/`, Rust 2.0)**:
  - 언어/툴체인: Rust `1.91.1` (`#![forbid(unsafe_code)]`, `rust-toolchain.toml` 고정)
  - 8개 크레이트 워크스페이스:
    - `strata-core`: `TriMesh`, `StacklessLbvh`, 디자인 토큰(`strata_tokens`), `StrataError`
    - `strata-io`: `.obj`+`.mtl`/`.jpg` 4K 바이리니어 텍스처 로더, `.ply`/`.stl`, `.ap2` 양방향 코덱, `.darch` 프로젝트 저장/로드
    - `strata-orient`: 6분류 유물 정위치, `Structure-from-Sherds++` 3영역 분할, `Pratt+Taubin+Huber` 원호 피팅, `Huber IRLS` 회전축, 360° 구연 결손 스캔, 구슬 천공 ODR, `BAISER` 접합
    - `strata-measure`: 기고·복원구경(±σ)·잔존율·기벽두께·구슬 천공경(`0.01 mm`)·용적·추정중량 정밀 계측
    - `strata-section`: 수직/수평/임의 보조단면(`A-A'`) 슬라이싱, 미스캔 내벽 파선 보정, `kurbo` 3차 베지어 피팅
    - `strata-shade`: 정밀 탁본(`rubbing_engine.rs`: MSII 다중스케일 곡률 + DoG + Sigmoid 먹 농도), 5종 미분 특징선, 360° 원통/원추/구면 전개, 기와 포목흔 2D FFT
    - `strata-export`: 8모드·10템플릿 고고학 도판 조판기(`plate_composer.rs`), Illustrator 레이어 SVG, 1:1 분할 인쇄 PDF, AutoCAD DXF R12, `ret/` 폴더 패키지
    - `strata-studio`: `eframe`/`egui`/`wgpu` 네이티브 데스크톱 스튜디오
- **읽기 전용 레퍼런스 (C++20 / Qt)**:
  - `studio/` (Qt 6 Widgets 초기 프로토타입) 및 `GigaMesh/gui/src/dongarch/karch/` (C++20 K-Arch 기하 엔진): Rust `strata-arch3d/`로 100% 이식 완료되어 동결(Read-Only).
  - `GigaMesh/` (Qt 5.15, GPLv3): 업스트림 알고리즘 참조·CLI용. `GigaMesh/core`·`external` 수정 금지.

## 검증된 명령 (`strata-arch3d/`, 2026-11-03, 이 PC)
| 대상 | 명령 | 결과 |
|---|---|---|
| 통합 검증 게이트 | `powershell -NoProfile -ExecutionPolicy Bypass -File .agents/skills/dongarch3d-studio/scripts/verify-gate.ps1` | `fmt-check` + `clippy (-D warnings)` + `cargo test` (121종) + `V1~V8` 하네스 + 실제 창 캡처 통과 |
| 바탕화면 배포 동기화 | `powershell -NoProfile -ExecutionPolicy Bypass -File .agents/skills/dongarch3d-studio/scripts/sync-desktop-shortcut.ps1` | `target\release\strata-studio.exe` → `%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe` 복사 및 SHA256 일치 |
| CI 로컬 재현 | `powershell -NoProfile -ExecutionPolicy Bypass -File strata-arch3d/scripts/ci.ps1` | `.github/workflows/strata-rust-ci.yml` 동일 5단계 게이트 통과 |

주의: `strata-arch3d/Cargo.toml`의 `[profile.release]`에 `panic = "abort"`가 설정되어 있으므로 **`cargo test --release`는 실행하지 않는다**(기본 `dev` 프로파일로 실행).

## 규약
- 단위 `mm`(`f64`), 수직축/회전축 `+Z`, 도판 기본 배치는 **외면 왼쪽 · 단면 오른쪽**(`R` 키로 좌/우 반전).
- SVG는 `width/height` mm · viewBox 1:1 (`1mm = 5px` 4코너 앵커 지원) · `<g id>` 레이어 · 선 두께 mm, DXF R12 `$INSUNITS=4` 11레이어.
- 용어: 영문 축약어 금지어를 쓰지 않고 반드시 **'정밀 탁본' (`rubbing` / `Rubbing`)**으로 표기.
- UI 정본: `docs/ui/UI_RULES.md`(R1~R8) 및 `strata-arch3d/docs/TODO_TDD_PLAN.md`. 새 문자열 한국어 100%, 색상은 `strata-core::strata_tokens`.
- 단계 완료 후 필수 루프: `git commit` → `git push` (GitHub Actions) → `ego-browser` 내 연결된 **Gemini Argon** 리뷰 요청 → 타당성 대조 검증 후 수정.
