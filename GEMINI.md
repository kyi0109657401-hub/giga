# STRATA · DongArch3D Rules (`gemini-4-argon` / DSH)

## 1. 주력 개발 대상: `strata-arch3d/` (Rust 1.91.1 워크스페이스)
1. 현재 모든 신규 기능 개발·버그 수정·UI 개선은 **`strata-arch3d/`** (8개 크레이트: `strata-core`, `strata-io`, `strata-orient`, `strata-measure`, `strata-section`, `strata-shade`, `strata-export`, `strata-studio`)에서 수행한다.
2. 저장소 내 C++ 폴더(`studio/`, `GigaMesh/`)는 초기 프로토타입 및 알고리즘 참조용 **읽기 전용(Read-Only) 레퍼런스**이며, 별도 사용자 지시 없이는 수정하지 않는다.

## 2. Rust Core Rules
1. `#![forbid(unsafe_code)]`를 엄수하고 불필요한 `clone()`을 지양한다.
2. 프로덕션 코드에서 `unwrap()`/`expect()` 사용을 금지하고 `Result<T, StrataError>`와 `?` 연산자로 복구 가능한 에러 처리를 수행한다.
3. 모든 변경 후 `strata-arch3d/`에서 아래 검증 게이트를 통과해야 한다:
   - `cargo fmt-check` (`cargo fmt --all -- --check`)
   - `cargo verify-v1` (`cargo clippy --workspace --all-targets -- -D warnings`, 경고 0건)
   - `cargo test --workspace` (주의: `release` 프로파일은 `panic = "abort"`이므로 테스트는 반드시 기본 `dev` 프로파일로 실행)
   - `cargo run --release -p strata-studio -- --capture-window <out.png>` (실제 실행 창 프레임버퍼 캡처 검수)

## 3. 브랜치 · 리뷰 · 배포 · 도판 규칙
1. **기능별 브랜치 격리**: 기능 하나당 `feat/<주제>-<슬러그>` 브랜치 하나를 만들어 작업한다.
2. **단계 완료 후 필수 리뷰 루프 (Commit → Push & GitHub Actions → `ego-browser` Gemini Argon 리뷰 → 검증 후 수정)**:
   - 단계가 끝나면 반드시 ① `git commit` 및 `git push`(GitHub Actions 실행 포함)를 수행한 뒤, ② `ego-browser`(Ego Lite 브라우저) 안에서 GitHub에 연결되어 대기 중인 제미나이 아르곤(Gemini Argon)에게 리뷰를 요청하고, ③ 그 리뷰 내용이 맞는지 코드·명세와 대조 확인한 뒤 타당한 지적사항을 수정한다.
3. **바탕화면 바로가기 고정 바이너리 갱신**: 병합 완료 시마다 `cargo build --release -p strata-studio` 후 `%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe`로 복사하여 바탕화면 바로가기가 항상 최신 빌드를 실행하도록 유지한다.
4. **도판 및 용어 규약**:
   - 내부 단위는 `mm`(`f64`), 수직축·회전축은 `+Z`, 토기 도판 기본 배치는 **외면 왼쪽 · 단면 오른쪽**(`R` 키 반전).
   - 영문 축약어 금지어를 쓰지 않고 반드시 **'정밀 탁본' (`rubbing` / `Rubbing`)**으로 표기한다.
