# AGENTS.md — STRATA · DongArch3D Antigravity CLI & Multi-Agent Rules

고고학 유물 3D 스캔(`PLY`/`OBJ`/`STL` + `MTL`/`JPG`/`AP2`, 단위 `mm`)을 정위치 → 단면·외곽·정밀 탁본·치수 → 1:1 mm 실측 도판(`SVG`/`DXF`/`PDF`/`ret/`)으로 완성하는 Windows 네이티브 데스크톱 스튜디오.
모든 대화·계획·보고·UI 문자열은 **한국어**를 사용한다.

## 1. 주력 개발 워크스페이스 (`strata-arch3d/`)

- **주력 타깃**: `strata-arch3d/` (Rust `1.91.1`, `eframe`/`egui`/`wgpu`, `#![forbid(unsafe_code)]`, 8개 크레이트 워크스페이스: `strata-core`, `strata-io`, `strata-orient`, `strata-measure`, `strata-section`, `strata-shade`, `strata-export`, `strata-studio`).
- **설계 및 명세 정본**:
  - `docs/ui/UI_RULES.md` (R1~R8 조용한 유물 도판 레이아웃·토큰)
  - `strata-arch3d/docs/DESIGN_SPEC.md`, `strata-arch3d/docs/DEV_PRINCIPLES.md`, `strata-arch3d/docs/IMPLEMENTATION_PLAN.md`, `strata-arch3d/docs/TODO_TDD_PLAN.md`, `strata-arch3d/docs/BUILD_SETUP.md`
- **읽기 전용(Read-Only) C++ 레퍼런스**:
  - `studio/` (Qt 6 C++20 초기 프로토타입), `GigaMesh/gui/src/dongarch/karch/` (C++ K-Arch 기하 엔진), `GigaMesh/` (Qt 5.15 업스트림)는 알고리즘 대조용 읽기 전용 레퍼런스이며 별도 지시 없이는 수정하지 않는다.

## 2. Rust Core Rules & 도판 규약

1. **안전성 및 린트**: `#![forbid(unsafe_code)]` 엄수, 불필요한 `clone()` 지양, 프로덕션 코드 내 `unwrap()`/`expect()` 금지 (`Result<T, StrataError>` 및 `?` 연산자 사용).
2. **좌표·단위·배치**:
   - 내부 단위는 `mm` (`f64`), 수직축·회전축은 `+Z`, 도판 기본 배치는 **외면 왼쪽 · 단면 오른쪽** (`R` 키로 좌/우 반전).
   - SVG: `1 user unit = 1 mm`, Illustrator 호환 최상위 `<g id>` 레이어 분리. DXF: R12 (`$INSUNITS=4`).
3. **용어 표준화**: 금지어 영문 축약어를 절대 쓰지 않고 반드시 한국 고고학 표준 용어인 **'정밀 탁본'**과 영문 식별자 **`rubbing` / `Rubbing`**만 사용한다.

## 3. 필수 검증 게이트 (`strata-arch3d/`)

모든 코드 변경 후 아래 검증을 순서대로 통과해야 한다 (`powershell -NoProfile -ExecutionPolicy Bypass -File .agents/skills/dongarch3d-studio/scripts/verify-gate.ps1`로 일괄 실행 가능):

```powershell
Set-Location strata-arch3d
cargo fmt-check                                      # cargo fmt --all -- --check (적용은 cargo format)
cargo verify-v1                                      # cargo clippy --workspace --all-targets -- -D warnings (경고 0건)
cargo test --workspace                               # 전체 단위·통합 테스트 (반드시 기본 dev 프로파일로 실행; release는 panic="abort"이므로 금지)
cargo run --release -p strata-studio -- --verify-all # V1~V8 헤드리스 하네스 검증
cargo run --release -p strata-studio -- --capture-window <out.png> # 실제 네이티브 창 스왑체인 캡처 검수
```

## 4. 브랜치 · `ego-browser` Gemini Argon 리뷰 · 바탕화면 배포 루프

1. **기능별 브랜치 격리**: 기능 하나당 `feat/<주제>-<슬러그>`(또는 `fix/...`, `chore/...`) 브랜치를 만들어 작업한다.
2. **단계 완료 후 필수 리뷰 루프**:
   - ① `git commit` 및 `git push origin <branch>` (GitHub Actions `strata-rust-ci.yml` 트리거)
   - ② `ego-browser`(Ego Lite) 내 연결된 **Gemini Argon**에게 변경 내역 리뷰 요청 (`.agents/skills/dongarch3d-studio/scripts/ego-gemini-review.mjs` 활용 가능)
   - ③ 리뷰 지적사항을 실제 코드·명세와 대조 검증한 뒤 타당한 결함을 수정하고 재검증
3. **바탕화면 바로가기 고정 바이너리 동기화**:
   - 병합 및 수정 완료 시 `.agents/skills/dongarch3d-studio/scripts/sync-desktop-shortcut.ps1`을 실행하여 `target\release\strata-studio.exe`를 `%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe`로 복사하고 SHA256 해시 일치를 확인한다.
