# CLAUDE.md — STRATA · DongArch3D 작업 규칙 (2026-11-03 갱신)

고고학 유물 3D 스캔(PLY/OBJ/STL/WRL + MTL/JPG/AP2, mm)을 정위치 → 단면·외곽·정밀 탁본·치수 → 1:1 mm 실측 도판(SVG/DXF/PDF/`ret/`)으로 만드는 Windows 데스크톱 앱. 모든 문서·대화·보고는 **한국어**. 코드 식별자·커밋 제목은 영문 허용.

## 1. 정본과 아키텍처 구분 (바꾸려면 먼저 사용자 승인)

- **현재 주력 앱 본체 (Rust 2.0)**: `strata-arch3d/` (Rust 1.91.1, `eframe`/`egui`/`wgpu`, `#![forbid(unsafe_code)]`, 8개 크레이트 워크스페이스: `strata-core`, `strata-io`, `strata-orient`, `strata-measure`, `strata-section`, `strata-shade`, `strata-export`, `strata-studio`).
  - 설계·구현 정본: `strata-arch3d/docs/DESIGN_SPEC.md`, `strata-arch3d/docs/DEV_PRINCIPLES.md`, `strata-arch3d/docs/IMPLEMENTATION_PLAN.md`, `strata-arch3d/docs/TODO_TDD_PLAN.md`, `strata-arch3d/docs/BUILD_SETUP.md`.
- **구형 C++ 참조용 코드 (동결 · 읽기 전용)**:
  - `studio/` (Qt 6 C++20 초기 프로토타입) 및 `GigaMesh/gui/src/dongarch/karch/` (C++ K-Arch 기하 엔진): Rust `strata-arch3d`로 기능 이식이 완료되었으며, 수식·알고리즘 대조용 **읽기 전용 레퍼런스**다. 사용자가 명시적으로 지시하지 않는 한 수정하지 않는다.
  - `GigaMesh/` (Qt 5.15, GPLv3): 업스트림 알고리즘 참조·CLI용이며 `GigaMesh/core/`, `GigaMesh/external/`은 절대 수정하지 않는다.
- **UI 정본**: `docs/ui/UI_RULES.md`(R1~R8) + `strata-arch3d/docs/TODO_TDD_PLAN.md`(사용자 피드백 반영 최신 UI/도판 명세). 구형 시안(`docs/legacy/`)은 참조하지 않는다.
- **토기 도판 좌/우(D5, 2026-10-17 확정)**: 외면 왼쪽 · 단면 오른쪽(국내 보고서 표준), `R` 로 좌/우 반전. 근거: `factory/project-profile.md` D5.
- **라이선스 및 보안**: Artec SDK·Studio 바이너리·번역·분석 결과·대용량 스캔 데이터(`*.ply/*.obj/*.stl` — `GigaMesh/testdata/` 소형 파일 제외)는 절대 커밋하지 않는다.

## 2. 좌표·단위·용어 규약 (코드와 문서 모두 이 규약으로)

- 내부 단위 **mm(f64/double)**, 수직축 **+Z**, 회전축 **+Z**, 도면 좌표 `(x, z)` → SVG 출력 시 y 반전.
- SVG: `width/height="…mm"` + viewBox 1 user unit = 1 mm, 선 두께 mm, 최상위 `<g id>` 레이어. DXF: R12, `$INSUNITS=4`(mm), 레이어 11종(`01_AXIS_CENTER` … `11_SCALE_BAR`).
- 단위 자동 환산은 **모호하면 사용자에게 묻는다**(조용히 ×10/×1000 금지). 추정값(내벽 미스캔 파선 추정 등)은 실측값과 항상 구분해 표기한다.
- **용어 표준화**: 금지어 영문 축약어를 쓰지 않고 반드시 한국 고고학 표준 용어인 **'정밀 탁본'**과 영문 식별자 **`rubbing` / `Rubbing`**만 사용한다.

## 3. 검증된 명령 (`strata-arch3d/` Rust 워크스페이스 기준)

작업 디렉터리 `strata-arch3d/`(또는 해당 기능 worktree의 `strata-arch3d/`)에서 실행한다:

```powershell
cargo fmt-check                                      # = cargo fmt --all -- --check (포맷 검사, 적용은 cargo format)
cargo verify-v1                                      # = cargo clippy --workspace --all-targets -- -D warnings (경고 0건 필수)
cargo test --workspace                               # 전체 단위·통합 테스트 (dev 프로파일, 의존성 opt-level=2 적용)
cargo run --release -p strata-studio -- --capture-window <out.png>          # 실제 네이티브 창 스왑체인 캡처 검증
cargo run --release -p strata-studio -- --open <mesh> --save-project <out.darch>  # 저장 경로 자동 검증 (EXIT=0)
```

- `Cargo.toml`의 `[profile.release]`에 `panic = "abort"`가 설정되어 있으므로 **`cargo test --release`는 실행하지 않는다**(테스트는 기본 `dev` 프로파일로 실행).
- (참고: 레거시 C++ 검증이 필요한 경우에만 `powershell -NoProfile -ExecutionPolicy Bypass -File factory/scripts/verify.ps1` 사용)

## 4. 절대 규칙

1. `GigaMesh/core/`, `GigaMesh/external/`는 수정하지 않는다. 구형 C++ `studio/` 역시 별도 지시 없이는 수정하지 않으며, 모든 신규 개발·수정은 `strata-arch3d/`에서 수행한다.
2. 새 UI 문자열은 100 % 한국어, 약어 노출 금지. Rust 코드는 `#![forbid(unsafe_code)]`를 유지하고 프로덕션 코드에서 `unwrap()`/`expect()` 대신 `Result<T, StrataError>`를 사용한다.
3. 구현은 항상 **별도 기능 브랜치 + worktree**에서 수행한다. `main`·`factory/setup`에 직접 커밋하지 않는다. 동시 구현 작업은 1개.
4. **`main` 브랜치 병합은 반드시 사용자 승인 후에만 수행한다.** AI 검토 통과는 최종 병합 승인이 아니다. 단, **격리된 기능 브랜치(`feat/*`, `fix/*`, `chore/*`)의 원격 푸시(`git push origin <branch>`)는 단계별 GitHub Actions 및 `ego-browser` Gemini Argon 리뷰 루프(10항)를 위해 수행**하며, `main` 병합은 사용자 승인을 기다린다.
5. 완료 증거는 마커 파일이나 가짜 오프스크린 드로잉이 아니라 **`cargo fmt-check` · `cargo clippy` · `cargo test` 통과 로그 및 실제 실행 창(`strata-studio.exe`) 캡처**다. 실패는 실패라고 쓴다.
6. 검토자 역할(code/correctness/security/ui-reviewer)은 제품 코드를 수정하지 않는다. 구현은 builder만.
7. 5 MB 초과 바이너리, 스캔 데이터(`*.ply/*.obj/*.stl` — `GigaMesh/testdata/`의 5개 소형 파일 제외), 독점 자료는 커밋하지 않는다.
8. 메인 체크아웃 `D:\gigamesh`의 미커밋 변경과 `feat/dongarch-studio` 브랜치는 건드리지 않는다.
9. **기능 추가는 반드시 기능별 브랜치를 새로 만들어 그 위에서만 한다**(2026-11-01 사용자 지시). 브랜치 이름은 `feat/<주제>-<짧은 영문 슬러그>`(예: `feat/hsec-fixed-view`), 작업 단위는 **기능 하나 = 브랜치 하나 = 커밋 묶음 하나**. 리팩터링·문서·설정 변경도 각각 `refactor/…`, `docs/…`, `chore/…` 브랜치에서 하는 것을 권장한다.
10. **단계 완료 시 필수 워크플로: Git 커밋 → Push & GitHub Actions → `ego-browser` 내 Gemini Argon 리뷰 → 타당성 확인 후 수정**(2026-11-03 사용자 지시):
    - 매 단계(또는 기능 단위) 구현 및 로컬 검증(`cargo fmt-check` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · 실제 창 캡처)이 끝나면:
      ① **Git 커밋 및 원격 푸시(`git commit` & `git push`) + GitHub Actions 실행**까지 수행한다.
      ② **`ego-browser`(Ego Lite 브라우저) 안에 GitHub과 연결되어 대기 중인 제미나이 아르곤(Gemini Argon)에게 리뷰를 요청**한다.
      ③ **제미나이의 리뷰 결과를 읽고 실제 코드·명세 기준 맞는지(타당한 지적사항인지) 검증**한 뒤, 타당한 항목은 즉시 수정(커밋·푸시·재검증)에 들어간다.
11. **수정 완료(병합) 때마다 바탕화면 바로가기를 최신 빌드로 갱신**(2026-11-02 사용자 지시, 사용자가 직접 확인). 순서: `cargo build --release -p strata-studio` → `%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe`에 복사 → STRATA 바로가기가 고정 폴더를 가리키는지 확인. worktree 산출물을 직접 가리키지 않는다.

## 5. 커밋 규칙

- 제목: `<범주>(<영역>): <한국어 요약>` (범주 `feat`/`fix`/`refactor`/`docs`/`chore`/`test`/`ci`), 본문 한국어로 이유·검증 결과 기록.
- 커밋 메시지·PR 본문에 비밀값·절대경로 비밀을 넣지 않는다.

## 6. 참고 위치

- Rust 설계 및 빌드 문서: `strata-arch3d/docs/BUILD_SETUP.md`, `strata-arch3d/docs/DEV_PRINCIPLES.md`, `strata-arch3d/docs/IMPLEMENTATION_PLAN.md`, `strata-arch3d/docs/TODO_TDD_PLAN.md`.
- 조사 보고서(2026-10-17): 프로젝트 라이브러리 `DongArch3D_1단계_조사보고서_및_준비계획*.md`.
- 업스트림 GigaMesh 클론: `C:\Users\kyi25\OneDrive\Dokumen\GigaMesh`(gitlab fcgl 23d8cf4, testdata 26개).
- Artec 매뉴얼: `C:\Program Files\Artec\Artec Studio 20 Trial\Documentation\Artec_Studio_Manual_EN.pdf`(내보내기 9.5, 단위 13.1.10).
- 하위 `GigaMesh/CLAUDE.md`는 GigaMesh 원본 빌드 참고용이며 이 파일과 충돌하면 **이 파일이 우선**한다.
