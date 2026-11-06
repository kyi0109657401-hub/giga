# PDF·OBJ 코퍼스 구축 및 4대 도판·정밀탁본 격차 해소 구현 계획 (Implementation Plan)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** PDF·OBJ 참조 코퍼스 매니페스트(Stage 1)와 발굴조사보고서 파일럿 6책 페이지 트리아지(Stage 2)를 구축하고, 조사에서 확인된 `strata-arch3d` 앱의 4대 격차(PDF 축척·래스터·메타데이터 조판, 『유물제도선.ai』 9단계 선폭·세피아 먹색 옵션, 정밀 탁본 고립 점묘 잡음 억제, 16K 텍스처 박스 필터 다운샘플링 및 `vn=0` 법선 검증)를 TDD로 구현(Stage 3)한 뒤 전체 검증 게이트 및 바탕화면·포터블 배포본을 갱신(Stage 4)한다.

**Architecture:**
1. 읽기 전용 Python 분석 도구(`strata-arch3d/tools/reference/`)가 `I:\` 원본을 변경 없이 스캔해 결정론적 정렬(`sort_keys=True`, 경로 오름차순)이 적용된 `manifest.json`, `sha256.csv`, `pilot6_triage.json`, `pilot6_summary.md`를 `strata-arch3d/docs/reference/`에 생성하고 접촉 시트는 `D:\strata-reference\pilot6\`에 저장한다.
2. Rust 워크스페이스(`strata-export`, `strata-shade`, `strata-studio`, `strata-io`)에서 각 격차 항목마다 실패하는 단위 테스트를 먼저 추가(Red)하고 최소 구현으로 통과(Green)시킨다.

**Tech Stack:** Rust 1.91.1 (`strata-arch3d`), Python 3 (`PyMuPDF`, `Pillow`, `numpy`, `opencv-python`)

**Spec:** `strata-arch3d/docs/SEQUENTIAL_EXECUTION_SPEC.md`

## Global Constraints
- `#![forbid(unsafe_code)]`, 프로덕션 코드 내 `unwrap()`/`expect()` 금지 (`Result<T, StrataError>` 및 `?` 연산자 사용).
- 내부 단위 `mm` (`f64`), 수직·회전축 `+Z`, 한국어 표준 용어 **'정밀 탁본' (`rubbing` / `Rubbing`)** 사용.
- 기존 기본 먹색 `#141413` 유지, 참조 도판 세피아 먹색 `#231816`은 선택 옵션으로 추가.
- `I:\` 원본 파일은 100% 읽기 전용으로만 접근.

## Review Focus
1. `I:\` 드라이브의 한글·공백 경로(`I:\디지털 탁본 참고 자료_(주)캐럿펀트`, `I:\새 폴더 (11)`) 처리 시 UTF-8 인코딩 및 결정론적 오름차순 정렬 보장.
2. PDF 출력(`render_plate_to_pdf_bytes`) 시 `ScalePreset`(`1:1`, `1:2`, `1:3`, `1:4`) 비율이 벡터 경로 좌표와 선 굵기, 검증 막대에 일관되게 반영되는지 확인.
3. PDF에 정밀 탁본 PNG 래스터(`RubbingPngSlot`) 임베드 시 PDF-1.4 표준 `/XObject /Subtype /Image` 스트림 구조가 유효한지 확인.
4. 정밀 탁본 점묘 잡음 억제 시 문양의 실제 경계 에지(Edge)는 뭉개지지 않고 평탄면의 고립 점(Isolated speckle)만 제거되는지 확인.
5. 16K(`16384×16384`) 텍스처를 4K(`4096×4096`)로 축소할 때 박스 평균 필터가 경계 오버플로 없이 동작하는지 확인.

---

### Task 1: Stage 1 — 참조 코퍼스 읽기 전용 인벤토리·매니페스트 생성 (`tools/reference/build_reference_manifest.py`)

**Files:**
- Create: `strata-arch3d/tools/reference/build_reference_manifest.py`
- Create: `strata-arch3d/docs/reference/manifest.json`
- Create: `strata-arch3d/docs/reference/sha256.csv`
- Create: `strata-arch3d/docs/reference/duplicate_report.md`

- [ ] **Step 1: `build_reference_manifest.py` 작성 (결정론적 정렬 `sort_keys=True`, 읽기 전용 스캔)**
  - `I:\디지털 탁본 참고 자료_(주)캐럿펀트` (PDF 9종), `I:\새 폴더 (11)` (OBJ 11세트 + AI 5종 + `strata_plate_1to1.pdf` + `034/ret/`)의 파일 크기, SHA-256, PDF 쪽수/판형/이미지 객체 수, OBJ 정점/삼각형/`vt`/`vn`/bbox(mm), 텍스처 JPEG SOF 해상도 추출.
  - 중복 이름 후보 2쌍(`7/7.obj` vs `9/7.obj`, `8/8.obj` vs `10/8.obj`)의 해시 및 형상 지문 비교 리포트 생성.
- [ ] **Step 2: 스크립트 실행 및 산출물 검증**
  - Run: `py -3 strata-arch3d/tools/reference/build_reference_manifest.py`
  - Expected: Exit code 0, `manifest.json`, `sha256.csv`, `duplicate_report.md` 생성 확인.

---

### Task 2: Stage 2 — 발굴조사보고서 파일럿 6책 페이지 트리아지 및 접촉 시트 생성 (`tools/reference/run_pilot6_triage.py`)

**Files:**
- Create: `strata-arch3d/tools/reference/run_pilot6_triage.py`
- Create: `strata-arch3d/docs/reference/pilot6_triage.json`
- Create: `strata-arch3d/docs/reference/pilot6_summary.md`
- Output (Git 외부): `D:\strata-reference\pilot6\` (파일럿 6책 접촉 시트 PNG 및 대표 유물 도면 크롭 PNG)

- [ ] **Step 1: `run_pilot6_triage.py` 작성 (PyMuPDF + OpenCV 선화/레이아웃 휴리스틱)**
  - 파일럿 6책(`매호동2.pdf`, `청도 김전리2.pdf`, `제45책 의성 대리리 7호분(2).pdf`, `36. 제주 외도일동38-1번지.pdf`, `남원 향교동 유적 Ⅰ권.pdf`, `영주 한국문화테마파크 조성부지내 유적 보고서.pdf`)을 대상으로 텍스트 캡션(`도면 N.`)과 이미지/선화 밀도(백색 배경 대비 흑백 세선 비율, 채도 분포로 원색 사진 도판 배제)를 결합해 유물 도면 후보 페이지를 분류.
  - 보고서별 접촉 시트(`D:\strata-reference\pilot6\<id>_contact_sheet.png`)와 대표 크롭을 저장하고, 통계 요약을 `strata-arch3d/docs/reference/pilot6_summary.md`에 기록.
- [ ] **Step 2: 스크립트 실행 및 산출물 검증**
  - Run: `py -3 strata-arch3d/tools/reference/run_pilot6_triage.py`
  - Expected: Exit code 0, 6책 전체 페이지 분류 결과 및 접촉 시트 생성 확인.

---

### Task 3: Stage 3-A & 3-B — `strata-export` 『유물제도선.ai』 9단계 선폭·세피아 먹색 옵션 및 PDF 축척·래스터·메타데이터 조판 (TDD)

**Files:**
- Modify: `strata-arch3d/crates/strata-export/src/svg_illustrator.rs`
- Modify: `strata-arch3d/crates/strata-export/src/pdf_tiled_print.rs`
- Modify: `strata-arch3d/crates/strata-export/src/lib.rs`

- [ ] **Step 1: 먼저 실패하는 단위 테스트 작성 (`svg_illustrator.rs`, `pdf_tiled_print.rs`)**
  - `test_ai_standard_9_line_weights_and_sepia_ink_option`: `AI_STANDARD_9_LINE_WEIGHTS_PT_MM`가 `0.10, 0.12, 0.15, 0.20, 0.30, 0.40, 0.50, 0.80, 1.00 pt` 9단계를 모두 포함하고, 기본 먹색(`#141413`) 유지 및 세피아 먹색(`#231816`) 변환 옵션(`render_plate_to_illustrator_svg_with_ink_color`)이 정확히 동작하는지 검증.
  - `test_pdf_respects_scale_preset_and_embeds_rubbing_and_metadata`: `render_plate_to_pdf_bytes_with_options`가 `ScalePreset::Half` (`1:2`), `ScalePreset::Third` (`1:3`)일 때 경로 좌표를 축척 비율에 맞게 스케일링하고, 유물 ID/축척 텍스트 및 정밀 탁본 래스터(`/Subtype /Image`) XObject를 PDF에 포함하는지 검증.
- [ ] **Step 2: 테스트 실패(Red) 확인**
  - Run: `cargo test -p strata-export`
- [ ] **Step 3: 최소 구현 작성 (Green)**
  - `svg_illustrator.rs`: `AI_STANDARD_9_LINE_WEIGHTS_PT_MM` 상수(`0.10~1.00 pt`), `REFERENCE_SEPIA_INK_HEX = "#231816"`, `DEFAULT_INK_HEX = "#141413"`, 선택적 먹색 교체 렌더러 구현.
  - `pdf_tiled_print.rs`: `render_plate_to_pdf_bytes` 내 `plate.scale_preset.ratio()` 좌표 스케일링 반영 및 `PdfPlateMetadata` / 정밀 탁본 그레이스케일·RGB 래스터 `/XObject` 임베딩 지원(`render_plate_to_pdf_bytes_with_options`).
- [ ] **Step 4: 테스트 통과 확인**
  - Run: `cargo test -p strata-export`

---

### Task 4: Stage 3-C — `strata-shade` & `strata-studio` 정밀 탁본 고립 점묘 잡음(Isolated Speckle) 억제 (TDD)

**Files:**
- Modify: `strata-arch3d/crates/strata-shade/src/rubbing_engine.rs`
- Modify: `strata-arch3d/crates/strata-studio/src/rubbing_ddg_pipeline.rs`

- [ ] **Step 1: 먼저 실패하는 단위 테스트 작성**
  - `test_suppress_isolated_rubbing_speckles_preserves_continuous_strokes`: 1~2px 고립 먹점(점묘 잡음)은 제거(배경 청결도 ≥ 98%)하면서 연결된 선각·능선 먹선은 90% 이상 보존하는 `suppress_isolated_ink_speckles` 필터 단위 테스트 작성.
- [ ] **Step 2: 테스트 실패(Red) 확인**
  - Run: `cargo test -p strata-shade -p strata-studio`
- [ ] **Step 3: 최소 구현 작성 (Green)**
  - `rubbing_engine.rs` 및 `rubbing_ddg_pipeline.rs`에 3×3 이웃 연결성 기반 고립 점묘(Isolated speckle) 억제 패스를 추가하여 연속된 능선·문양 에지는 보존하고 평탄면의 고립 잡음만 정화.
- [ ] **Step 4: 테스트 통과 확인**
  - Run: `cargo test -p strata-shade -p strata-studio`

---

### Task 5: Stage 3-D — `strata-io` 16K 텍스처 박스 평균 다운샘플링 및 `vn=0` 법선 자동 생성 검증 (TDD)

**Files:**
- Modify: `strata-arch3d/crates/strata-io/src/obj_loader.rs`

- [ ] **Step 1: 먼저 실패하는 단위 테스트 작성**
  - `test_downsample_rgba_box_filter_reduces_aliasing_and_vn_zero_normals_valid`: 고해상도 텍스처 축소 시 단일 픽셀 점 샘플링 대신 2×2 영역 평균(Box filter)이 적용되는지와 `vn` 라인이 0개인 OBJ 로드 시 모든 정점 법선이 유한 단위벡터(`|n| ≈ 1.0`)로 생성되는지 검증.
- [ ] **Step 2: 테스트 실패(Red) 확인**
  - Run: `cargo test -p strata-io`
- [ ] **Step 3: 최소 구현 작성 (Green)**
  - `obj_loader.rs`의 `load_and_downsample_texture` 내부에 `downsample_rgba_box_filter` 헬퍼를 분리·적용.
- [ ] **Step 4: 테스트 통과 확인**
  - Run: `cargo test -p strata-io`

---

### Task 6: Stage 4 — 필수 검증 게이트 · 바탕화면 바로가기 및 포터블 패키지 최신화

**Files:**
- Verify: `strata-arch3d/` 전체 크레이트
- Update: `%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe`, 바탕화면 포터블 폴더 및 `.zip`

- [ ] **Step 1: 필수 검증 게이트 일괄 실행**
  - Run: `powershell -NoProfile -ExecutionPolicy Bypass -File .agents\skills\dongarch3d-studio\scripts\verify-gate.ps1`
- [ ] **Step 2: 바탕화면 바로가기 바이너리 및 포터블 패키지 동기화**
  - Run: `powershell -NoProfile -ExecutionPolicy Bypass -File .agents\skills\dongarch3d-studio\scripts\sync-desktop-shortcut.ps1`
  - Update Desktop portable folder & `.zip`.
