use image::RgbaImage;
use strata_core::strata_tokens::tokens::SURFACE;
use strata_core::TriMesh;

use super::*;

fn make_test_bowl_mesh() -> TriMesh {
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    let nu = 36usize;
    let nv = 16usize;
    for iv in 0..=nv {
        let z = (iv as f64 / nv as f64) * 60.0;
        let r = 28.0 + 32.0 * (iv as f64 / nv as f64).powf(0.75);
        for iu in 0..=nu {
            let theta = (iu as f64 / nu as f64) * std::f64::consts::TAU;
            positions.push([r * theta.cos(), r * theta.sin(), z]);
        }
    }
    let stride = (nu + 1) as u32;
    for iv in 0..(nv as u32) {
        for iu in 0..(nu as u32) {
            let a = iv * stride + iu;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            triangles.push([a, b, d]);
            triangles.push([a, d, c]);
        }
    }
    TriMesh::from_positions_and_triangles("test_bowl", positions, triangles).expect("valid")
}

#[test]
fn test_zbuffer_3d_rasterizer_produces_opaque_shaded_and_textured_pixels() {
    let mesh = make_test_bowl_mesh();
    for mode in [
        View3DShaderMode::PhotoTexture,
        View3DShaderMode::SolidClay,
        View3DShaderMode::RealTimeRubbing,
    ] {
        let state = View3DPaneState {
            shader_mode: mode,
            cutaway_open: false,
            ..Default::default()
        };
        let img = render_mesh_zbuffer_rgba(&mesh, None, &state, 240, 240);
        let shaded_count = img
            .pixels()
            .filter(|px| {
                let [r, g, b, _] = px.0;
                (i32::from(r) - i32::from(SURFACE.rgb[0])).abs() > 25
                    || (i32::from(g) - i32::from(SURFACE.rgb[1])).abs() > 25
                    || (i32::from(b) - i32::from(SURFACE.rgb[2])).abs() > 25
            })
            .count();
        assert!(
            shaded_count > 2_500,
            "mode={mode:?} expected >2500 shaded Z-buffer pixels, got {shaded_count}"
        );
    }
}

#[test]
fn test_view3d_cutaway_and_world_locked_plane_intersection() {
    let mesh = make_test_bowl_mesh();
    let v_segs = extract_3d_plane_intersection_segments(&mesh, 45.0, 0.0, 0.0, true);
    let h_segs = extract_3d_plane_intersection_segments(&mesh, 45.0, 0.0, 18.0, false);
    assert!(
        !v_segs.is_empty(),
        "vertical Y=0 plane intersection must produce 3D segments"
    );
    assert!(
        !h_segs.is_empty(),
        "horizontal Z=18mm plane intersection must produce 3D segments"
    );
}

#[test]
fn test_flat_artifact_under_2mm_height_initializes_top_cut_without_panic() {
    // 1.63mm 두께의 편평 유물(석기·소편) — 예전에는 `.clamp(2.0, height_mm)` 가
    // `min > max` 로 패닉해 창이 뜨자마자 죽었다.
    let positions = vec![
        [0.0, 0.0, 0.0],
        [40.0, 0.0, 0.0],
        [40.0, 30.0, 1.63],
        [0.0, 30.0, 1.63],
    ];
    let triangles = vec![[0_u32, 1, 2], [0, 2, 3]];
    let mesh = TriMesh::from_positions_and_triangles("flat_sherd", positions, triangles)
        .expect("valid flat mesh");
    assert!((mesh.bbox.extents()[2] - 1.63).abs() < 1e-9);

    let mut state = View3DPaneState::default();
    state.initialize_top_cut_for_mesh(&mesh);
    assert!(
        state.top_cut_z_mm >= 0.0 && state.top_cut_z_mm <= 1.63 + 1e-9,
        "수평횡단면 기본 높이가 유물 높이 범위를 벗어남: {}",
        state.top_cut_z_mm
    );
    // 같은 메쉬를 다시 열면 초기화를 반복하지 않는다.
    state.top_cut_z_mm = 0.2;
    state.initialize_top_cut_for_mesh(&mesh);
    assert!((state.top_cut_z_mm - 0.2).abs() < 1e-12);

    // 경계가 뒤집힌 클램프도 패닉 없이 순서를 보정해야 한다.
    assert!((clamp_ordered(-5.0, 2.0, 1.63) - 1.63).abs() < 1e-12);
    assert!((clamp_ordered(5.0, 2.0, 1.63) - 2.0).abs() < 1e-12);
    assert!((clamp_ordered(0.4, 0.5, 1.63) - 0.5).abs() < 1e-12);
}

#[test]
fn test_transformed_horizontal_section_offset_is_relative_to_artifact_base() {
    // 유물이 월드 원점에서 100mm 떨어져 있어도(스캔 좌표는 제각각) 절단면 오프셋은
    // 유물 바닥 기준 `top_cut_z_mm` 그대로여야 한다. 예전에는 `bbox.min[2]` 를 더해
    // 우측 도판 라벨이 `Z=9.0mm`(클램프된 상한)처럼 어긋났다.
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    let nu = 24usize;
    let nv = 12usize;
    for iv in 0..=nv {
        let z = 100.0 + (iv as f64 / nv as f64) * 60.0;
        let r = 30.0 + 25.0 * (iv as f64 / nv as f64).powf(0.8);
        for iu in 0..=nu {
            let theta = (iu as f64 / nu as f64) * std::f64::consts::TAU;
            positions.push([r * theta.cos(), r * theta.sin(), z]);
        }
    }
    let stride = (nu + 1) as u32;
    for iv in 0..(nv as u32) {
        for iu in 0..(nu as u32) {
            let a = iv * stride + iu;
            triangles.push([a, a + 1, a + stride + 1]);
            triangles.push([a, a + stride + 1, a + stride]);
        }
    }
    let mesh = TriMesh::from_positions_and_triangles("offset_bowl", positions, triangles)
        .expect("valid offset mesh");
    assert!((mesh.bbox.min[2] - 100.0).abs() < 1e-9);

    let state = View3DPaneState {
        top_cut_z_mm: 12.5,
        ..Default::default()
    };
    let hsec = slice_transformed_mesh_world_locked(&mesh, &state, false);
    assert!(
        !hsec.polylines_2d.is_empty(),
        "수평 횡단면 폴리라인이 비어 있음"
    );
    assert!(
        (hsec.offset_mm - 12.5).abs() < 1e-9,
        "절단면 오프셋은 바닥 기준이어야 함(기대 12.5, 실제 {})",
        hsec.offset_mm
    );
}

#[test]
fn test_tdd1_view3d_dirty_cache_skips_redundant_rerender_and_supports_hd_resolution() {
    let mesh = make_test_bowl_mesh();
    let mut state = View3DPaneState::default();

    let (w1, h1, recomputed1) = state.ensure_zbuffer_cached(&mesh, None, 560, 560, 1.0);
    assert!(recomputed1, "first frame must recompute Z-buffer");
    assert_eq!(
        (w1, h1),
        (560, 560),
        "must support HD 560x560 resolution without 360px clamp"
    );

    let (w2, h2, recomputed2) = state.ensure_zbuffer_cached(&mesh, None, 560, 560, 1.0);
    assert!(
        !recomputed2,
        "identical state must hit dirty cache with 0 recomputation"
    );
    assert_eq!((w2, h2), (560, 560));

    let mut state = View3DPaneState {
        active_section_angle_deg: 45.0,
        ..state
    };
    let (_, _, recomputed3) = state.ensure_zbuffer_cached(&mesh, None, 560, 560, 1.0);
    assert!(
        recomputed3,
        "changed rotation angle must trigger Z-buffer recomputation"
    );
}

#[test]
fn test_tdd4_hand_dial_spherical_rotation_updates_pitch_yaw_roll() {
    let mut state = View3DPaneState::default();
    let initial_psi = state.active_section_angle_deg;
    let initial_pitch = state.pitch_deg;

    state.apply_hand_dial_drag(40.0, -30.0, false);
    assert!(
        (state.active_section_angle_deg - initial_psi).abs() > 10.0,
        "horizontal dial drag must rotate artifact ψ"
    );
    assert!(
        (state.pitch_deg - initial_pitch).abs() > 5.0,
        "vertical dial drag must tilt/rotate vertical viewing angle"
    );

    state.apply_hand_dial_drag(20.0, -15.0, true);
    assert!(
        state.roll_deg.abs() > 0.5,
        "fine axis dial horizontal drag must adjust roll_deg"
    );
    assert!(
        state.tilt_deg.abs() > 0.5,
        "fine axis dial vertical drag must adjust tilt_deg"
    );

    state.reset_hand_orientation();
    assert_eq!(state.active_section_angle_deg, 0.0);
    assert_eq!(state.tilt_deg, 0.0);
    assert_eq!(state.roll_deg, 0.0);
}

#[test]
fn test_tdd8_isotropic_projection_never_stretches_vertically_on_drag_release() {
    let (dw, dh) = compute_isotropic_render_dimensions_hd(420.0, 680.0, true, 1.0);
    let (rw, rh) = compute_isotropic_render_dimensions_hd(420.0, 680.0, false, 1.0);
    let target_aspect = 420.0 / 680.0;
    let drag_aspect = (dw as f64) / (dh as f64);
    let release_aspect = (rw as f64) / (rh as f64);
    assert!(
        (drag_aspect - target_aspect).abs() < 0.005,
        "drag aspect {drag_aspect} must match target {target_aspect}"
    );
    assert!(
        (release_aspect - target_aspect).abs() < 0.005,
        "release aspect {release_aspect} must match target {target_aspect}"
    );
}

#[test]
fn test_tdd9_four_cardinal_fine_dials_step_exact_001mm_and_01deg() {
    let mut state = View3DPaneState::default();
    for _ in 0..100 {
        state.apply_cardinal_fine_dial(CardinalDialDir::Up, 1);
    }
    assert!((state.cut_y_mm - 1.00).abs() < 1e-9);
    for _ in 0..35 {
        state.apply_cardinal_fine_dial(CardinalDialDir::Down, 1);
    }
    assert!((state.cut_y_mm - 0.65).abs() < 1e-9);
    for _ in 0..15 {
        state.apply_cardinal_fine_dial(CardinalDialDir::Right, 1);
    }
    assert!((state.active_section_angle_deg - 1.5).abs() < 1e-9);
    for _ in 0..5 {
        state.apply_cardinal_fine_dial(CardinalDialDir::Left, 1);
    }
    assert!((state.active_section_angle_deg - 1.0).abs() < 1e-9);
}

#[test]
fn test_shader_chip_modes_expose_all_five_backend_modes() {
    let modes = SHADER_CHIP_MODES;
    assert_eq!(
        modes.len(),
        5,
        "3D 셰이더 칩은 백엔드 5종을 모두 노출해야 한다(현재 {}개)",
        modes.len()
    );
    for m in [
        View3DShaderMode::PhotoTexture,
        View3DShaderMode::SolidClay,
        View3DShaderMode::RealTimeRubbing,
        View3DShaderMode::Segmentation3Way,
        View3DShaderMode::XRayInner,
    ] {
        assert!(modes.contains(&m), "{:?} 칩이 노출되지 않음", m);
    }
}

#[test]
fn test_tdd11_concentric_120pct_topdown_and_zero_lag_frame_budget() {
    let mesh = make_test_bowl_mesh();
    let state = View3DPaneState::default();
    assert_eq!(state.yaw_deg, 0.0);
    assert_eq!(state.pitch_deg, 0.0);

    let artifact_r = 62.5_f64;
    let grid_r = compute_concentric_grid_radius_mm(artifact_r);
    assert!((grid_r - artifact_r * 1.20).abs() < 1e-9);

    let t0 = std::time::Instant::now();
    let top_img = render_topdown_concentric_plan_rgba(&mesh, &state, 280, 220);
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let non_bg = top_img
        .pixels()
        .filter(|px| {
            (i32::from(px[0]) - i32::from(SURFACE.rgb[0])).abs() > 12
                || (i32::from(px[1]) - i32::from(SURFACE.rgb[1])).abs() > 12
        })
        .count();
    assert!(non_bg > 1_500);
    assert!(elapsed_ms < 40.0, "took {elapsed_ms:.2}ms");
}

#[test]
fn test_tdd13_five_wheel_cardinal_and_inner_level_exact_section_sync() {
    let mesh = make_test_bowl_mesh();
    let mut state = View3DPaneState::default();

    // 1. Image 1: 상단 가로 휠 -> 유물 시계·반시계 회전 (0.1° 단위)
    state.apply_precision_wheel(PrecisionWheelKind::TopTurnCwCcw, 25);
    assert!((state.active_section_angle_deg - 2.5).abs() < 1e-9);
    assert!(PrecisionWheelKind::TopTurnCwCcw
        .description_label_kr()
        .contains("시계"));

    // 2. Image 2: 하단 가로 휠 -> 유물·단면선 앞뒤 미세조정 (0.01mm 단위)
    state.apply_precision_wheel(PrecisionWheelKind::BottomSectionFrontBack, -18);
    assert!((state.cut_y_mm - (-0.18)).abs() < 1e-9);

    // 3. Image 3: 우측 세로 휠 -> 유물 좌우 기울기 미세조정 (0.1° 단위)
    state.apply_precision_wheel(PrecisionWheelKind::RightTiltLeftRight, 12);
    assert!((state.roll_deg - 1.2).abs() < 1e-9);

    // 4. Image 4: 좌측 세로 휠 -> 올리면(+steps) 위로 회전(+tilt_deg), 내리면(-steps) 아래로 회전(-tilt_deg)
    state.apply_precision_wheel(PrecisionWheelKind::LeftPitchUpDown, 15);
    assert!((state.tilt_deg - 1.5).abs() < 1e-9);
    state.apply_precision_wheel(PrecisionWheelKind::LeftPitchUpDown, -5);
    assert!((state.tilt_deg - 1.0).abs() < 1e-9);

    // 5. <USER_REQUEST>: 내부 제5 수평정합 휠 (0.05° 단위) 및 3D-2D 단면 100% 일치 검증
    state.apply_precision_wheel(PrecisionWheelKind::InnerHorizontalLevel, 10);
    assert!((state.inner_level_deg - 0.50).abs() < 1e-9);

    let v_segs_3d = extract_3d_plane_intersection_segments_full(&mesh, &state, true);
    let slice_2d = slice_transformed_mesh_world_locked(&mesh, &state, true);
    assert_eq!(v_segs_3d.len(), slice_2d.segments_2d.len());
    assert!(!slice_2d.polylines_2d.is_empty());

    // 3D 단면 선분의 X 범위와 2D 슬라이서 폴리라인의 X 범위가 0.01mm 이내로 100% 일치해야 함!
    let min_x_3d = v_segs_3d
        .iter()
        .flat_map(|s| [s[0][0], s[1][0]])
        .fold(f64::INFINITY, f64::min);
    let max_x_3d = v_segs_3d
        .iter()
        .flat_map(|s| [s[0][0], s[1][0]])
        .fold(f64::NEG_INFINITY, f64::max);
    let min_x_2d = slice_2d
        .polylines_2d
        .iter()
        .flatten()
        .map(|p| p[0])
        .fold(f64::INFINITY, f64::min);
    let max_x_2d = slice_2d
        .polylines_2d
        .iter()
        .flatten()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (min_x_3d - min_x_2d).abs() < 1e-5 && (max_x_3d - max_x_2d).abs() < 1e-5,
        "3D cut X [{min_x_3d:.3}, {max_x_3d:.3}] must match 2D slice X [{min_x_2d:.3}, {max_x_2d:.3}]"
    );
}

#[test]
fn test_tdd14_topdown_fixed_center_section_and_vertical_dashed_link() {
    let mesh = make_test_bowl_mesh();
    let state = View3DPaneState {
        artifact_offset_x_mm: 12.5,
        cut_y_mm: -4.2,
        ..Default::default()
    };

    // 유물을 좌우(+12.5mm) 이동시켰을 때 3D 단면 교차선도 정확히 +12.5mm 이동하여 상하 뷰 점선 연결 위치가 일치해야 함
    let segs = extract_3d_plane_intersection_segments_full(&mesh, &state, true);
    let mean_x: f64 =
        segs.iter().map(|s| 0.5 * (s[0][0] + s[1][0])).sum::<f64>() / (segs.len().max(1) as f64);
    assert!(
        (mean_x - 12.5).abs() < 1.5,
        "translated artifact section center X must shift by +12.5mm, got {mean_x:.2}"
    );

    // 상단 직상방 동심원 평면도와 하단 수평 정면 입면도의 스케일(px/mm)이 1:1 동일해야 하며,
    // 좌·우 절단점의 화면 X 좌표가 기울어지지 않고 100% 수직(90°)으로 정렬되어야 함 (v6_vessel_roi_1to1 Image 0)
    let ext = mesh.bbox.extents();
    let grid_r_mm = compute_concentric_grid_radius_mm(0.5 * ext[0].max(ext[1]));
    let top_scale = compute_shared_ortho_scale(388.0, grid_r_mm, ext[2], state.zoom);
    let bot_scale = compute_shared_ortho_scale(388.0, grid_r_mm, ext[2], state.zoom);
    assert!((top_scale - bot_scale).abs() < 1e-12);
}

#[test]
fn test_tdd17_inner_wheel_controls_horizontal_section_z_and_updates_plate() {
    let mesh = make_test_bowl_mesh();
    let mut state = View3DPaneState::default();
    let init_z = state.top_cut_z_mm;
    let init_tilt = state.tilt_deg;
    let init_roll = state.roll_deg;

    // Image 0: 안쪽 제5휠을 움직이면 유물은 고정된 채 수평횡단면 선(top_cut_z_mm)만 아래위로 이동하고 수평횡단면 도면이 갱신되어야 함
    state.apply_precision_wheel(PrecisionWheelKind::InnerHorizSectionZ, 25);
    assert!(
        (state.top_cut_z_mm - (init_z + 2.5)).abs() < 1e-6,
        "InnerHorizSectionZ +25 steps must move top_cut_z_mm by +2.5mm, got {}",
        state.top_cut_z_mm
    );
    assert_eq!(state.tilt_deg, init_tilt, "artifact tilt must stay fixed");
    assert_eq!(state.roll_deg, init_roll, "artifact roll must stay fixed");

    let h_slice_low = slice_transformed_mesh_world_locked(&mesh, &state, false);
    state.apply_precision_wheel(PrecisionWheelKind::InnerHorizSectionZ, 120);
    let h_slice_high = slice_transformed_mesh_world_locked(&mesh, &state, false);
    assert!(!h_slice_low.polylines_2d.is_empty());
    assert!(!h_slice_high.polylines_2d.is_empty());
    assert!(
        (h_slice_high.offset_mm - h_slice_low.offset_mm - 12.0).abs() < 1e-5,
        "horizontal section Z offset must advance by +12.0mm"
    );
}

#[test]
fn test_tdd18_arrow_keys_nudge_artifact_and_horizontal_section() {
    let mut state = View3DPaneState::default();
    state.apply_arrow_key_nudge(-1, 0, false, false, 60.0);
    assert!((state.artifact_offset_x_mm - (-0.10)).abs() < 1e-6);

    state.apply_arrow_key_nudge(0, 1, true, false, 60.0);
    assert!((state.cut_y_mm - (-1.00)).abs() < 1e-6);

    let z0 = state.top_cut_z_mm;
    state.apply_arrow_key_nudge(0, 1, false, true, 60.0);
    assert!((state.top_cut_z_mm - (z0 + 0.20)).abs() < 1e-6);
}

#[test]
fn test_tdd19_custom_section_line_single_endpoint_move_and_2d_profile() {
    let mesh = make_test_bowl_mesh();
    let mut state = View3DPaneState::default();
    state.add_default_custom_section_line(55.0);
    let line0 = state
        .custom_section_line_mm
        .expect("custom section line must exist");
    let p0_before = line0[0];
    let p1_before = line0[1];

    // Image 5: 끝지점(0번 끝점)을 누르고 이동하면 반대쪽(1번 끝점)은 고정되고 한쪽만 이동해야 함!
    state.move_custom_section_endpoint(0, [-38.0, 22.0]);
    let line1 = state.custom_section_line_mm.expect("custom line 1");
    assert_eq!(line1[0], [-38.0, 22.0], "dragged endpoint 0 must move");
    assert_eq!(
        line1[1], p1_before,
        "other endpoint 1 must remain 100% fixed"
    );

    // 반대로 1번 끝점을 이동하면 0번 끝점은 고정되고 1번 끝점만 이동해야 함!
    state.move_custom_section_endpoint(1, [42.0, -16.0]);
    let line2 = state.custom_section_line_mm.expect("custom line 2");
    assert_eq!(line2[0], [-38.0, 22.0], "endpoint 0 must remain 100% fixed");
    assert_eq!(line2[1], [42.0, -16.0], "dragged endpoint 1 must move");
    let _ = p0_before;

    // 그어진 사용자 단면선에 대한 2D 단면 폴리라인이 추출되어야 함
    let custom_slice = slice_custom_line_section_2d(&mesh, &state).expect("custom slice");
    assert!(
        !custom_slice.polylines_2d.is_empty(),
        "custom section line across vessel must produce 2D cross-section polylines"
    );
}

#[test]
fn test_dark_mode_invalidates_zbuffer_cache_and_darkens_viewport_bg() {
    // Gemini 리뷰 a6ecaaa ③: 다크 전환 시 래스터 바탕이 바뀌어야 하고 캐시가 무효화되어야 한다.
    let mesh = make_test_bowl_mesh();
    let mut state = View3DPaneState::default();
    let (_, _, recomputed1) = state.ensure_zbuffer_cached(&mesh, None, 120, 120, 1.0);
    assert!(recomputed1, "첫 렌더링은 재계산되어야 함");
    let (_, _, recomputed2) = state.ensure_zbuffer_cached(&mesh, None, 120, 120, 1.0);
    assert!(!recomputed2, "동일 상태는 캐시 적중해야 함");
    state.dark_mode = true;
    let (_, _, recomputed3) = state.ensure_zbuffer_cached(&mesh, None, 120, 120, 1.0);
    assert!(recomputed3, "다크 전환 시 캐시가 무효화되어야 함");
    let img = state.cached_zbuffer_rgba.as_ref().expect("cached image");
    // 사용자 피드백 ①: 밤 작업대보다 한 단계 밝은 뷰포트 배경이어야 한다.
    let dark_bg_px = img
        .pixels()
        .filter(|p| {
            p.0[0] == crate::theme::dark::VIEWPORT_BG[0]
                && p.0[1] == crate::theme::dark::VIEWPORT_BG[1]
                && p.0[2] == crate::theme::dark::VIEWPORT_BG[2]
        })
        .count();
    assert!(
        dark_bg_px > 1_000,
        "다크 래스터에 한 단계 밝은 뷰포트 배경이 있어야 함: {dark_bg_px}"
    );
}

#[test]
fn test_standard_light_direction_matches_legacy_formula_and_rubbing_frames() {
    // B1 고정점: 표준광 315/45가 뷰 파이프라인 양쪽에서 같은 방향이어야 한다.
    let wl = standard_light_direction(315.0, 45.0);
    let az = 315.0f64.to_radians();
    let el = 45.0f64.to_radians();
    assert!((wl[0] - el.cos() * az.sin()).abs() < 1e-12);
    assert!((wl[1] + el.cos() * az.cos()).abs() < 1e-12);
    assert!((wl[2] - el.sin()).abs() < 1e-12);
    let len = (wl[0] * wl[0] + wl[1] * wl[1] + wl[2] * wl[2]).sqrt();
    assert!((len - 1.0).abs() < 1e-12, "단위벡터여야 함");
    // 뷰별 프레임 매핑(탁본 법선과 동일 규칙)은 내적을 보존해야 한다.
    for n in [[0.3, 0.5, 0.8], [-0.2, 0.9, 0.1], [0.0, 0.0, 1.0]] {
        let base = n[0] * wl[0] + n[1] * wl[1] + n[2] * wl[2];
        let front = n[0] * wl[0] + (-n[1]) * (-wl[1]) + n[2] * wl[2];
        let back = (-n[0]) * (-wl[0]) + n[1] * wl[1] + n[2] * wl[2];
        let plan = n[0] * wl[0] + n[2] * wl[2] + n[1] * wl[1];
        assert!((base - front).abs() < 1e-12, "정면 매핑 내적 보존 실패");
        assert!((base - back).abs() < 1e-12, "배면 매핑 내적 보존 실패");
        assert!((base - plan).abs() < 1e-12, "평면도 매핑 내적 보존 실패");
    }
}

#[test]
fn test_hd_render_dimensions_scale_with_pixels_per_point() {
    // B5 고정점: HiDPI에서 ensure 경로가 물리 픽셀 기준으로 타겟을 키워야 흐림이 없다.
    // (헬퍼 직접 호출이 아닌 실제 렌더 경로로 검증한다.)
    let mesh = make_test_bowl_mesh();
    let mut state = View3DPaneState::default();
    let (w1, h1, recomputed1) = state.ensure_zbuffer_cached(&mesh, None, 420, 680, 1.0);
    assert!(recomputed1, "첫 렌더링은 재계산되어야 함");
    assert_eq!((w1, h1), (420, 680));
    // dpr 전환 → dims가 바뀌므로 캐시 무효화 + 2배 타겟.
    let (w2, h2, recomputed2) = state.ensure_zbuffer_cached(&mesh, None, 420, 680, 2.0);
    assert!(recomputed2, "dpr 변경 시 재계산되어야 함");
    assert_eq!((w2, h2), (w1 * 2, h1 * 2), "dpr=2.0은 2배여야 함");
    // 종횡비는 dpr과 무관하게 보존되어야 한다.
    let target = 420.0 / 680.0;
    assert!(((w2 as f64) / (h2 as f64) - target).abs() < 1e-9);
    // 동일 dpr 반복 호출은 캐시 적중.
    let (_, _, recomputed3) = state.ensure_zbuffer_cached(&mesh, None, 420, 680, 2.0);
    assert!(!recomputed3, "동일 dpr는 캐시 적중해야 함");
}

#[test]
fn test_light_quantization_matches_rubbing_pipeline() {
    // B3 고정점: 0.4° 광원 이동은 양쪽 캐시 모두 적중, 0.6° 이동은 양쪽 모두 무효화되어야 한다.
    // (한쪽만 성기면 같은 조작에 3D와 탁본이 어긋난다.)
    let key_at = |az: f32| {
        let state = View3DPaneState {
            light_azimuth_deg: az,
            ..Default::default()
        };
        state.build_cache_key("quant_probe", 200, 200)
    };
    assert_eq!(key_at(315.0), key_at(315.2));
    assert_ne!(key_at(315.0), key_at(315.6));
}

#[test]
fn test_top_cut_z_defaults_to_half_height_in_world_coords() {
    // B4 문서화 고정: top_cut_z_mm는 월드 절대 높이. 기본값=기고 50%, 절단은 빗나가지 않아야 한다.
    let mesh = make_test_bowl_mesh();
    let mut state = View3DPaneState::default();
    state.initialize_top_cut_for_mesh(&mesh);
    assert!(
        (state.top_cut_z_mm - 30.0).abs() < 1e-9,
        "기본 절단면은 기고 절반이어야 함: {}",
        state.top_cut_z_mm
    );
    let hsec = slice_transformed_mesh_world_locked(&mesh, &state, false);
    assert!(
        hsec.polylines_2d.iter().any(|p| p.len() >= 2),
        "기본 높이에서 수평횡단면이 비어 있으면 안 됨"
    );
}

fn section_drawn_half_width(mesh: &TriMesh, tilt_deg: f64, psi_deg: f64) -> f64 {
    let state = View3DPaneState {
        tilt_deg,
        active_section_angle_deg: psi_deg,
        ..Default::default()
    };
    let vsec = slice_transformed_mesh_world_locked(mesh, &state, true);
    vsec.polylines_2d
        .iter()
        .flatten()
        .map(|p| p[0].abs())
        .fold(0.0f64, f64::max)
}

#[test]
fn test_section_width_stays_equal_to_report_across_poses() {
    // 추가분(치수 단일원천): 뒤집기 후 단면 폭이 실측과 어긋나면 그리기 버그다.
    // 원형 대칭 유물은 뒤집기(180°)·방위각 회전에서 단면 반폭이 일정해야 한다.
    // (기울이기 25~90°는 절단면이 비스듬해져 기하학적으로 좁아지므로 본 단언에서 제외 —
    //  치수 표기는 리포트값 단일원천으로 고정한다.)
    let mesh = make_test_bowl_mesh();
    let w0 = section_drawn_half_width(&mesh, 0.0, 0.0);
    assert!(w0 > 1.0, "기준 단면 반폭이 비정상: {w0}");
    for tilt in [180.0, -180.0, 360.0] {
        let w = section_drawn_half_width(&mesh, tilt, 0.0);
        assert!(
            (w - w0).abs() < 1.0,
            "tilt={tilt}에서 단면 반폭이 어긋남: {w} vs {w0}"
        );
    }
    for psi in [45.0, 90.0, 135.0, 180.0, 270.0] {
        let w = section_drawn_half_width(&mesh, 0.0, psi);
        assert!(
            (w - w0).abs() < 1.0,
            "psi={psi}에서 단면 반폭이 어긋남: {w} vs {w0}"
        );
    }
}

#[test]
fn test_viewport_grid_has_no_red_in_either_theme() {
    // 사용자 피드백 ②: 격자 적색(156,74,47) 제거 → 중립 회색.
    // 단면선 빨강(PROFILE_RED)은 기능선이라 유지되므로 여기서는 검사하지 않는다.
    let mesh = make_test_bowl_mesh();
    for dark in [false, true] {
        let mut state = View3DPaneState {
            dark_mode: dark,
            ..Default::default()
        };
        let (_, _, recomputed) = state.ensure_topdown_cached(&mesh, 200, 200, 1.0);
        assert!(recomputed);
        let img = state.cached_topdown_rgba.as_ref().expect("cached image");
        let red_px = img
            .pixels()
            .filter(|p| p.0[0] == 156 && p.0[1] == 74 && p.0[2] == 47)
            .count();
        assert_eq!(red_px, 0, "dark={dark}: 격자에 적색이 남아 있음");
        let gray_px = img
            .pixels()
            .filter(|p| {
                p.0[0] == VIEWPORT_GRID_GRAY[0]
                    && p.0[1] == VIEWPORT_GRID_GRAY[1]
                    && p.0[2] == VIEWPORT_GRID_GRAY[2]
            })
            .count();
        assert!(
            gray_px > 100,
            "dark={dark}: 중립 회색 격자가 그려져야 함: {gray_px}"
        );
    }
}

#[test]
fn test_xray_renders_back_to_front_deterministically() {
    // B7: X-Ray는 z버퍼 없이 덧씌우므로 뒤→앞 순서가 고정되어야 한다.
    // 메쉬 저장 순서와 무관하게 같은 그림이 나와야 한다.
    let mesh = make_test_bowl_mesh();
    let n = mesh.positions.len();
    let mut world = vec![[0.0f64; 3]; n];
    // 0번 삼각형(정점 0, 1, stride+1 = 0, 1, 38)을 가장 멀리 둔다.
    world[0] = [0.0, 0.0, 100.0];
    world[1] = [1.0, 0.0, 100.0];
    world[38] = [0.0, 1.0, 100.0];
    let order = xray_back_to_front_order(&mesh, &world);
    assert_eq!(
        order.len(),
        mesh.triangles.len(),
        "전 삼각형이 순서에 있어야 함"
    );
    assert_eq!(order[0], 0, "가장 먼 삼각형이 먼저 그려져야 함");
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        (0..mesh.triangles.len()).collect::<Vec<_>>(),
        "순서는 삼각형 인덱스의 순열이어야 함"
    );
    // 같은 입력은 항상 같은 순서 (결정적).
    assert_eq!(order, xray_back_to_front_order(&mesh, &world));

    // 실제 X-Ray 렌더도 두 번 하면 바이트 동일해야 한다.
    let state = View3DPaneState {
        shader_mode: View3DShaderMode::XRayInner,
        ..Default::default()
    };
    let a = render_mesh_zbuffer_rgba(&mesh, None, &state, 160, 160);
    let b = render_mesh_zbuffer_rgba(&mesh, None, &state, 160, 160);
    assert_eq!(a.as_raw(), b.as_raw(), "X-Ray 렌더가 비결정적이면 안 됨");
}

#[test]
fn test_dark_viewport_bg_three_candidates() {
    // task-12 A/B 선정용 3종 프리셋 검증 (동일 장면 캡처용).
    let mesh = make_test_bowl_mesh();
    let mut img = RgbaImage::new(200, 200);
    // A: 웜 차콜 그라데이션 — 중심 #45423C, 가장자리 #23211D.
    fill_dark_viewport_bg(&mut img, DarkViewportBg::WarmCharcoalGradient, &mesh);
    assert_eq!(img.get_pixel(100, 100).0[0..3], [0x45, 0x42, 0x3C]);
    assert_eq!(img.get_pixel(0, 0).0[0..3], [0x23, 0x21, 0x1D]);
    // B: 플랫 #38352F.
    fill_dark_viewport_bg(&mut img, DarkViewportBg::FlatTaupe, &mesh);
    assert!(
        img.pixels().all(|p| p.0[0..3] == [0x38, 0x35, 0x2F]),
        "B안은 전역 플랫이어야 함"
    );
    // C: 적응형 — 어두운 유물 → 밝은 배경, 밝은 유물 → 어두운 배경.
    let mut dark_mesh = make_test_bowl_mesh();
    dark_mesh.colors_rgb = vec![[40, 30, 25]; dark_mesh.positions.len()];
    fill_dark_viewport_bg(&mut img, DarkViewportBg::AdaptiveContrast, &dark_mesh);
    assert_eq!(img.get_pixel(100, 100).0[0..3], [0x38, 0x35, 0x2F]);
    let mut bright_mesh = make_test_bowl_mesh();
    bright_mesh.colors_rgb = vec![[235, 228, 218]; bright_mesh.positions.len()];
    fill_dark_viewport_bg(&mut img, DarkViewportBg::AdaptiveContrast, &bright_mesh);
    assert_eq!(
        img.get_pixel(100, 100).0[0..3],
        crate::theme::dark::VIEWPORT_BG
    );
    // 선택자 기본값 = 현행.
    assert_eq!(
        DarkViewportBg::from_selector("zzz"),
        DarkViewportBg::Current
    );
    assert_eq!(
        DarkViewportBg::from_selector("A"),
        DarkViewportBg::WarmCharcoalGradient
    );
}
