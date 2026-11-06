# STRATA · 동국유물실측 순차 진행 요청 정리 및 명세서

- **작성일**: 2026-11-06
- **기준 브랜치 / 커밋**: `fix/texture-stretch-and-plan-alignment` (`5cc7b86`)
- **근거 문서**: `strata-arch3d/docs/REFERENCE_CORPUS_PLAN.md`, `strata-arch3d/docs/ARTIFACT_DRAWING_PLAN.md`, `strata-arch3d/docs/DESIGN_SPEC.md`

---

## 1. 요청 정리 (무엇을 왜 하는지 / 하지 않을 것)

### 1.1 무엇을 왜 하는가
PDF·OBJ 참고자료(`I:\디지털 탁본 참고 자료_(주)캐럿펀트`, `I:\새 폴더 (11)`) 및 동국문화재연구원 발굴조사보고서 77책(`I:\report\동국참여보고서`) 조사에서 밝혀진 **앱 출력 격차 해소**와 **참조 코퍼스·파일럿 데이터 구축**을 다음 4단계로 순차 진행한다.

1. **Stage 1 — 참조 코퍼스 읽기 전용 인벤토리·매니페스트 구축 (`REFERENCE_CORPUS_PLAN.md` Phase 1)**
   - PDF 9종 + OBJ 11세트 + AI 템플릿 5종의 SHA-256 해시, 정점/삼각형/UV/법선(`vn=0` 여부)/바운딩 박스, 텍스처 해상도, 중복 파일 2쌍(`7/7.obj` vs `9/7.obj`, `8/8.obj` vs `10/8.obj`) 판정 결과를 `strata-arch3d/docs/reference/`에 생성한다.
2. **Stage 2 — 발굴조사보고서 파일럿 6책 페이지 트리아지 및 유물 도면 표본 추출 (`ARTIFACT_DRAWING_PLAN.md` Phase 1 파일럿)**
   - 파일럿 6책(`매호동2.pdf`, `청도 김전리2.pdf`, `제45책 의성 대리리 7호분(2).pdf`, `36. 제주 외도일동38-1번지.pdf`, `남원 향교동 유적 Ⅰ권.pdf`, `영주 한국문화테마파크 조성부지내 유적 보고서.pdf`)에서 유물 도면 쪽을 분류하고 표본 크롭·접촉 시트·요약 리포트를 생성한다.
3. **Stage 3 — `strata-arch3d` 앱 4대 핵심 격차 해소 구현 (TDD)**
   - **3-A (`strata-export` PDF 다중 뷰·축척·래스터 조판)**: `render_plate_to_pdf_bytes`에서 도판 경로에 `plate.scale_preset.ratio()`(`1:1`, `1:2`, `1:3`, `1:4`)를 정확히 적용하고, 정밀 탁본/실사 래스터(`PDF Image XObject`) 및 유물 ID·축척·도면 번호 메타데이터 출력을 지원한다.
   - **3-B (`strata-export` 『유물제도선.ai』 9단계 선폭 계단 완비)**: `AI_STANDARD_LINE_WEIGHTS_PT_MM`에 조사에서 확인된 `1.00 pt`(`0.353 mm`)와 `0.10 pt`(`0.035 mm`)를 포함한 9단계 선폭 규격을 지원하고 참조 세피아 먹색(`#231816`) 프리셋 옵션을 제공한다.
   - **3-C (`strata-studio` / `strata-shade` 정밀 탁본 고립 점묘 잡음 억제)**: 평탄면·배경 경계의 고립 점(Spot/Stipple outlier) 밀도를 낮춰 참조 도판 수준의 배경 청결도(≥ 98%)를 확보한다.
   - **3-D (`strata-io` 16K 텍스처 박스 필터 다운샘플링 및 `vn=0` 법선 생성 검증)**: 16K(`16384²`) 텍스처를 4K로 축소할 때 앨리어싱을 줄이는 박스 평균 샘플링을 적용하고, `vn=0` OBJ 로드 시 자동 생성된 정점 법선의 단위벡터 무결성을 보장한다.
4. **Stage 4 — 전체 검증 게이트 통과 · 바탕화면 바로가기 및 포터블 패키지 재갱신**
   - `verify-gate.ps1` 통과 후 `sync-desktop-shortcut.ps1` 및 바탕화면 포터블 폴더·ZIP을 최신 바이너리로 갱신한다.

### 1.2 하지 않을 것 (Out of Scope)
- `I:\` 드라이브의 원본 PDF·OBJ·AI 파일은 절대 이동·수정·삭제하지 않는다 (100% Read-Only).
- `I:\pdf자료` (6,717개, 350 GB) 전체 스캔 및 대규모 ML 모델 학습은 1차 순차 진행에서 제외한다.
- 읽기 전용 C++ 레퍼런스 폴더(`studio/`, `GigaMesh/`)는 수정하지 않는다.
- 대용량 원본 이미지/OBJ를 Git에 커밋하지 않는다 (크롭·대용량 파생물은 `D:\strata-reference\`에 분리 저장하고, Git에는 매니페스트·소형 리포트만 반영).

---

## 2. 완료 기준 및 검증 방법

| 단계 | 완료 기준 | 확인 방법 (실행 명령 / 테스트) |
|---|---|---|
| Stage 1 | PDF 9종·OBJ 11세트·AI 5종 매니페스트(`manifest.json`, `sha256.csv`) 및 중복 2쌍 판정 리포트 생성 | `py -3 tools/reference/build_reference_manifest.py` (종료 코드 `0`) |
| Stage 2 | 파일럿 6책 페이지 트리아지 결과표(`pilot6_triage.json`) 및 접촉 시트(`D:\strata-reference\pilot6\`) 생성 | `py -3 tools/reference/run_pilot6_triage.py` (종료 코드 `0`) |
| Stage 3-A | PDF 출력 시 `scale_preset`(`1:1`, `1:2`, `1:3`) 비율 반영 및 정밀 탁본 래스터 XObject 포함 단위 테스트 통과 | `cargo test -p strata-export` (종료 코드 `0`) |
| Stage 3-B | `유물제도선.ai` 9단계 선폭(`0.10~1.00 pt`) 및 세피아 먹색(`#231816`) 옵션 단위 테스트 통과 | `cargo test -p strata-export` (종료 코드 `0`) |
| Stage 3-C | 정밀 탁본 고립 점묘 잡음 억제 및 배경 청결도 단위 테스트 통과 | `cargo test -p strata-shade -p strata-studio` (종료 코드 `0`) |
| Stage 3-D | `vn=0` OBJ 법선 생성 및 텍스처 다운샘플링 단위 테스트 통과 | `cargo test -p strata-io` (종료 코드 `0`) |
| Stage 4 | 포맷·Clippy(경고 0)·전체 테스트·`--verify-all`·창 캡처·바탕화면 바로가기 및 포터블 갱신 완료 | `powershell -NoProfile -ExecutionPolicy Bypass -File .agents\skills\dongarch3d-studio\scripts\verify-gate.ps1` (종료 코드 `0`) |

---

## 3. 바꿀 파일과 바꾸지 않을 파일

### 3.1 생성·수정할 파일
- `strata-arch3d/docs/SEQUENTIAL_EXECUTION_SPEC.md` (본 명세서)
- `strata-arch3d/docs/REFERENCE_CORPUS_PLAN.md`, `strata-arch3d/docs/ARTIFACT_DRAWING_PLAN.md`
- `strata-arch3d/docs/reference/manifest.json`, `strata-arch3d/docs/reference/sha256.csv`, `strata-arch3d/docs/reference/pilot6_summary.md`
- `strata-arch3d/tools/reference/build_reference_manifest.py`, `strata-arch3d/tools/reference/run_pilot6_triage.py`
- `strata-arch3d/crates/strata-export/src/pdf_tiled_print.rs`
- `strata-arch3d/crates/strata-export/src/svg_illustrator.rs`
- `strata-arch3d/crates/strata-shade/src/rubbing_engine.rs`
- `strata-arch3d/crates/strata-studio/src/rubbing_ddg_pipeline.rs`
- `strata-arch3d/crates/strata-io/src/obj_loader.rs`

### 3.2 바꾸지 않을 파일
- `I:\` 드라이브 내 모든 원본 파일 (읽기 전용)
- `studio/`, `GigaMesh/` (읽기 전용 C++ 레퍼런스)
