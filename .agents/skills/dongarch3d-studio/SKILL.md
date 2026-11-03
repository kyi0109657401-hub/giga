---
name: dongarch3d-studio
description: >-
  Build, test, capture native window screenshots, request Gemini Argon code reviews via ego-browser,
  and develop features for STRATA / DongArch3D Studio (primary Rust workspace `strata-arch3d/` with
  8 crates including `strata-studio`, plus frozen read-only C++ reference `studio/` and `karch/`).
  Use this skill whenever modifying `strata-arch3d/` crates, running verification gates, capturing
  screenshots, requesting Gemini Argon reviews, or updating the desktop shortcut binary.
---

# STRATA · DongArch3D Studio Development & Verification Skill (Antigravity CLI)

## 1. Primary Target: `strata-arch3d/` (Rust Workspace)

- **Working Directory**: `strata-arch3d/` (or the isolated git worktree's `strata-arch3d/` subdirectory)
- **Toolchain**: Rust `1.91.1` (`rustc`, `cargo`, `rustfmt`, `clippy` pinned via `strata-arch3d/rust-toolchain.toml` and installed in `~/.cargo/bin`)
- **Workspace Lints**: `#![forbid(unsafe_code)]`, `clippy::all = "warn"`, `unwrap_used = "warn"`
- **Profiles (`strata-arch3d/Cargo.toml`)**:
  - `dev`: `opt-level = 1`, `dev.package."*" = { opt-level = 2 }` (fast incremental compilation + 8.3x faster unit tests)
  - `release`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`
  - **Important**: Because `release` sets `panic = "abort"`, never run `cargo test --release` (enforced by `.agents/hooks.json`). Always run tests in the default `dev` profile.

## 2. Executable Helper Scripts (`scripts/`)

1. **One-Click Full Verification Gate (`verify-gate.ps1`)**:
   Runs `cargo fmt-check` → `cargo verify-v1` → `cargo test --workspace` → `--verify-all` (V1~V8) → `--capture-window`:
   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .agents\skills\dongarch3d-studio\scripts\verify-gate.ps1
   ```
2. **Desktop Shortcut Binary Sync (`sync-desktop-shortcut.ps1`)**:
   Builds `strata-studio` in release mode, copies `target\release\strata-studio.exe` to `%LOCALAPPDATA%\Programs\STRATA\strata-studio.exe`, and verifies SHA256 match:
   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .agents\skills\dongarch3d-studio\scripts\sync-desktop-shortcut.ps1
   ```
3. **`ego-browser` Gemini Argon Automated Review Request (`ego-gemini-review.mjs`)**:
   Sends a review prompt for the pushed commit/branch to the connected Google Gemini (`Gemini Argon`) tab in `ego-browser`:
   ```powershell
   $scriptUri = (Resolve-Path ".agents\skills\dongarch3d-studio\scripts\ego-gemini-review.mjs").Path -replace '\\','/'
   $env:STRATA_REVIEW_PROMPT = "저장소 https://github.com/kyi0109657401-hub/giga 의 브랜치/커밋 변경사항을 리뷰해 주세요: ..."
   ego-browser nodejs -e "await import('file:///$scriptUri')"
   ```

## 3. Standard Verification & Deployment Workflow (`strata-arch3d/`)

When running commands manually inside `strata-arch3d/`:

1. **Format Check (`rustfmt`)**:
   ```powershell
   cargo fmt-check          # alias for: cargo fmt --all -- --check (use `cargo format` to apply)
   ```
2. **Strict Clippy Gate (0 warnings allowed)**:
   ```powershell
   cargo verify-v1          # alias for: cargo clippy --workspace --all-targets -- -D warnings
   ```
3. **Full Workspace Test Suite**:
   ```powershell
   cargo test --workspace   # or `cargo verify-all` for --nocapture
   ```
4. **Real Native Window Framebuffer Capture (`strata-studio.exe`)**:
   ```powershell
   cargo run --release -p strata-studio -- --capture-window <output.png>
   cargo run --release -p strata-studio -- --tab 1 --capture-window <tab1.png>
   cargo run --release -p strata-studio -- --open <mesh_path> --save-project <out.darch>
   ```
5. **Mandatory Commit → GitHub Actions → `ego-browser` Gemini Argon Review Loop**:
   - At the end of every development step:
     1. `git commit` and `git push origin <branch>` to trigger GitHub Actions (`strata-rust-ci.yml`).
     2. Use `ego-browser` (`scripts/ego-gemini-review.mjs`) where Gemini Argon is connected to GitHub and waiting for review; request a review of the pushed commit/branch.
     3. Inspect the Gemini review feedback, verify each point against the actual code and spec, and implement fixes for valid findings.
6. **Post-Merge Desktop Shortcut Binary Update**:
   Run `scripts/sync-desktop-shortcut.ps1` after merging to `main`.

## 4. Rust Workspace Crate Map (`strata-arch3d/crates/`)

- **`strata-core`**: Core geometry/topology (`TriMesh` with UV/texture `MeshTextureRgba`), Stackless LBVH (`bvh.rs`), design tokens (`strata_tokens.rs`), 3-part structured error (`StrataError` in `error.rs`), and domain types (`types.rs`).
- **`strata-io`**: `.obj` + `.mtl`/`.jpg` 4K bilinear texture loader (`obj_loader.rs`), ASCII/Binary `.ply` & `.stl` (`ply_stl.rs`), `.ap2` bidirectional transform/cut-plane codec (`ap2_codec.rs`), and `.darch` project serialization (`darch_codec.rs`).
- **`strata-orient`**: 6-class artifact orientation, `Structure-from-Sherds++` 3-region fracture segmentation (`fracture_seg.rs`), Pratt+Taubin+Huber circle fitting (`circle_fit.rs`), Huber IRLS axis & rills solver (`axis_solver.rs`), 0°–360° rim defect scan (`defect_scan.rs`), bead perforation ODR (`bead_perforation.rs`), and BAISER refitting (`refit_baiser.rs`).
- **`strata-measure`**: Precision archaeological metrics (`metrics.rs`: height, restored diameter ±σ, survival ratio, thickness, bead borehole calipers to `0.01 mm`, volume, estimated weight).
- **`strata-section`**: Arbitrary/fixed plane slicing (`plane_slicer.rs`), occluded inner-wall dashed interpolation (`occluded_repair.rs`), and `kurbo` cubic Bézier fitting (`bezier_fit.rs`).
- **`strata-shade`**: Precision Rubbing (`rubbing_engine.rs`: normal differential, MSII multi-scale curvature, DoG + Sigmoid ink transfer, orthographic multi-view raster), 5 differential feature lines (`feature_lines.rs`), 360° cylinder/cone/sphere unrolling (`unroll.rs`), and roof-tile hemp cloth FFT (`hemp_fft.rs`).
- **`strata-export`**: 8-mode & 10-template CHS report plate composer (`plate_composer.rs`), Adobe Illustrator layered SVG (`svg_illustrator.rs`), `ret/` package exporter (`ret_exporter.rs`), 1:1 tiled PDF print (`pdf_tiled_print.rs`), and AutoCAD DXF R12 + dimension tables (`dxf_table_export.rs`).
- **`strata-studio`**: Native `eframe`/`egui` desktop studio (`main.rs`, `layout_r1.rs`, `theme.rs`, `view3d_pane/*`, `sheet_pane/*`, `rubbing_ddg_pipeline.rs`, `dialogs.rs`, `harness_runner.rs`).

## 5. Core Rules & Invariants

- **Terminology**: Always use the Korean standard term **'정밀 탁본'** and English identifier **`rubbing` / `Rubbing`**.
- **Branch Isolation**: Implement each feature in its own dedicated branch (`feat/<topic>-<slug>`) before merging upon user approval.
- **Legacy C++ Reference (`studio/` and `GigaMesh/gui/src/dongarch/karch/`)**:
  - The C++20 Qt6 `studio/` folder and `karch/` static library are **frozen read-only references** used only for comparing historical algorithms or file format edge cases. Do not modify C++ files unless the user explicitly asks for C++ changes.
