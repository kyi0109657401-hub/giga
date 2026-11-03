# strata-arch3d/AGENTS.md — STRATA Rust 1.91.1 Workspace Rules

이 디렉터리(`strata-arch3d/`)는 STRATA · 동국유물실측 v2.0의 주력 Rust 워크스페이스다.

## 1. 8개 크레이트 아키텍처 및 책임

| 크레이트 | 경로 | 핵심 역할 |
|---|---|---|
| `strata-core` | `crates/strata-core` | `TriMesh`(정점·법선·UV·4K 텍스처), `CompressedCornerTable`, `StacklessLbvh`, 디자인 토큰(`strata_tokens.rs`), 3단 구조화 에러(`StrataError`) |
| `strata-io` | `crates/strata-io` | `.obj`+`.mtl`/`.jpg` 바이리니어 텍스처 로더, ASCII/Binary `.ply`·`.stl`, `.ap2` 변환행렬/절단면 코덱, `.darch` 프로젝트 직렬화 |
| `strata-orient` | `crates/strata-orient` | 유물 6분류 정위치, `Structure-from-Sherds++` 3영역 분할, `Pratt+Taubin+Huber` 원호 피팅, `Huber IRLS` 회전축, 360° 결손 스캔, 구슬 천공 ODR, `BAISER` 접합 |
| `strata-measure` | `crates/strata-measure` | 기고·복원구경(±σ)·잔존율·기벽두께·구슬 천공경(`0.01 mm`)·용적·추정중량 정밀 계측 |
| `strata-section` | `crates/strata-section` | 고정 절단틀 수직/수평/임의 보조단면(`A-A'`) 슬라이싱, 미스캔 내벽 파선 보정, `kurbo` 3차 베지어 피팅 |
| `strata-shade` | `crates/strata-shade` | 정밀 탁본(`rubbing_engine.rs`: MSII 다중스케일 곡률 + DoG + Sigmoid 먹 농도), 5종 미분 특징선, 360° 원통/원추/구면 전개, 기와 포목흔 2D FFT |
| `strata-export` | `crates/strata-export` | 8모드·10템플릿 고고학 도판 조판기(`plate_composer.rs`), Illustrator 레이어 SVG, 1:1 분할 인쇄 PDF, AutoCAD DXF R12, `ret/` 폴더 패키지 |
| `strata-studio` | `crates/strata-studio` | `eframe`/`egui`/`wgpu` 데스크톱 앱 (`main.rs`, `layout_r1.rs`, `view3d_pane/*`, `sheet_pane/*`, `dialogs.rs`, `harness_runner.rs`) |

## 2. 빌드 프로파일 및 명령 주의사항

- **`[profile.release]`는 `panic = "abort"`를 사용**하므로 **`cargo test --release`를 절대 실행하지 않는다** (Rust 테스트 하네스는 unwind 필요).
- 단위·통합 테스트는 항상 기본 `dev` 프로파일(`cargo test --workspace`)로 실행한다 (`[profile.dev.package."*"] opt-level = 2`가 적용되어 고속 실행됨).
- Cargo 별칭(`.cargo/config.toml`):
  - `cargo format` : `cargo fmt --all`
  - `cargo fmt-check` : `cargo fmt --all -- --check`
  - `cargo verify-v1` : `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo verify-all` : `cargo test --workspace -- --nocapture`
  - `cargo studio` : `cargo run --release -p strata-studio --`

## 3. UI 및 도판 수정 시 필수 확인 사항

1. **실제 실행 창 캡처 검증**:
   - UI(`view3d_pane`, `sheet_pane`, `layout_r1`, `dialogs`) 또는 도판 조판(`plate_composer`)을 수정한 경우 반드시 `cargo run --release -p strata-studio -- --capture-window <path.png>`로 실제 창을 캡처하고 텍스트·위젯·치수선·축척자·도면 간 겹침이나 잘림이 없는지 눈으로 확인한다.
2. **단일 원천(Single Source of Truth) 치수 원칙**:
   - 도판 치수선과 상단 상태바의 구경·기고·두께 수치는 항상 `ArtifactMeasurementReport` 실측값을 단일 원천으로 표시한다.
3. **내부 적색선 배제 원칙**:
   - 화면 내 절단 기준선(`#FF0000`)과 조판 용지 외곽 경계선을 제외한 보조 격자·5cm 방안선·회전 바늘은 중립 회색 토큰을 유지하며, 내보내기(SVG/PDF/DXF) 산출물과 미리보기 색상을 혼동하지 않는다.
