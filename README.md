# STRATA · DongArch3D — 고고학 유물 3D 스캔 → 1:1 실측 도판 스튜디오 (Rust v2.0)

Artec 등 3D 스캐너로 얻은 토기·기와·석기·옥류·금속 유물 메시(`PLY`/`OBJ`/`STL` + `MTL`/`JPG`/`AP2`, 단위 `mm`)를 읽어 정위치(바닥 수평·회전 중심축)를 잡고, 단면선·외곽선·**정밀 탁본(`Rubbing`)**·치수를 추출하여 **1:1 mm 실측 도판**(`SVG`/`DXF`/`PDF`/`ret/`)으로 내보내는 Windows 네이티브 데스크톱 프로그램입니다.

- **주력 워크스페이스**: [`strata-arch3d/`](strata-arch3d/) (Rust `1.91.1`, `eframe`/`egui`/`wgpu`, `#![forbid(unsafe_code)]`, 8개 크레이트)
- **개발 및 에이전트 규칙**: [`AGENTS.md`](AGENTS.md) · [`GEMINI.md`](GEMINI.md) · [`CLAUDE.md`](CLAUDE.md) · UI 정본 [`docs/ui/UI_RULES.md`](docs/ui/UI_RULES.md)
- **원격 저장소**: `https://github.com/kyi0109657401-hub/giga.git`

---

## 1. 저장소 구성

| 폴더 | 내용 | 상태 |
|---|---|---|
| **`strata-arch3d/`** | **주력 앱 STRATA · 동국유물실측 v2.0** (Rust 1.91.1 8개 크레이트: `strata-core`, `strata-io`, `strata-orient`, `strata-measure`, `strata-section`, `strata-shade`, `strata-export`, `strata-studio`) | **주력 운영 (단위 테스트 121종 + V1~V8 하네스 통과)** |
| `.agents/` | Antigravity CLI(`agy`) 전용 스킬(`dongarch3d-studio`), 훅(`hooks.json`), 검증/배포/리뷰 자동화 스크립트 | 운영 중 |
| `docs/ui/` | 「조용한 유물 도판」 UI 규칙 정본(`UI_RULES.md` R1~R8) | 정본 |
| `studio/` | 초기 프로토타입 DongArch3D Studio (Qt 6 C++20) | 동결 (읽기 전용 레퍼런스) |
| `GigaMesh/` | K-Arch C++20 기하 엔진(`gui/src/dongarch/karch/`) 및 GigaMesh 프레임워크(Qt 5, GPLv3) | 동결 (읽기 전용 레퍼런스) |

---

## 2. 빌드 · 검증 · 실행 (`strata-arch3d/`)

필수 환경: Windows 11, Rust `1.91.1` (`rustfmt`, `clippy` 포함 — `strata-arch3d/rust-toolchain.toml` 자동 인식).

### 빠른 검증 및 실행 (Cargo 별칭)

```powershell
Set-Location strata-arch3d

cargo fmt-check          # 포맷 검사 (cargo fmt --all -- --check, 적용은 cargo format)
cargo verify-v1          # Clippy 엄격 검사 (cargo clippy --workspace --all-targets -- -D warnings)
cargo test --workspace   # 전체 단위·통합 테스트 (dev 프로파일)
cargo studio             # 데스크톱 스튜디오 실행 (cargo run --release -p strata-studio --)
```

> **주의**: `strata-arch3d/Cargo.toml`의 `[profile.release]`에는 `panic = "abort"`가 설정되어 있으므로 `cargo test --release`는 실행하지 마세요. 테스트는 기본 `dev` 프로파일(`cargo test --workspace`)로 실행합니다.

### 원클릭 통합 검증 및 바탕화면 바로가기 갱신 스크립트

```powershell
# 1) fmt-check + clippy(-D warnings) + cargo test + V1~V8 하네스 + 실제 창 프레임버퍼 캡처
powershell -NoProfile -ExecutionPolicy Bypass -File .agents\skills\dongarch3d-studio\scripts\verify-gate.ps1

# 2) 릴리즈 빌드 후 바탕화면 바로가기 고정 경로(%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe) 동기화
powershell -NoProfile -ExecutionPolicy Bypass -File .agents\skills\dongarch3d-studio\scripts\sync-desktop-shortcut.ps1
```

---

## 3. 단계별 개발 및 `ego-browser` Gemini Argon 리뷰 워크플로

1. 기능별 브랜치(`feat/<주제>-<슬러그>`)에서 TDD로 구현 및 `verify-gate.ps1` 통과.
2. `git commit` 및 `git push origin <branch>`로 GitHub Actions([`.github/workflows/strata-rust-ci.yml`](.github/workflows/strata-rust-ci.yml)) 실행.
3. `ego-browser`(Ego Lite)에 연결된 **Gemini Argon**에게 커밋/브랜치 리뷰를 요청하고 피드백을 검증·반영.
4. 사용자 승인 후 병합 및 `sync-desktop-shortcut.ps1`로 바탕화면 실행 파일 갱신.
