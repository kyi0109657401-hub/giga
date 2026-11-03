use eframe::egui;
use image::RgbaImage;
use strata_core::{ArtifactCategory, ScalePreset, TriMesh};
use strata_export::PlateRasterKind;
use strata_measure::compute_artifact_measurements;

use super::config::*;
use super::render::{
    preview_stroke_color_for_hex, resolve_rim_label_width_mm, PAPER_BORDER_RED_RGB,
    SHEET_GRID_5CM_RGB,
};
use super::rubbing::*;
use super::state::*;
use super::widgets::*;

fn make_patterned_bowl_mesh() -> TriMesh {
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    let nu = 48usize;
    let nv = 28usize;
    for iv in 0..=nv {
        let v_frac = iv as f64 / nv as f64;
        let z = v_frac * 55.0;
        let groove = if (iv % 4) == 2 { -1.35 } else { 0.35 };
        let r = 26.0 + 28.0 * v_frac.powf(0.8) + groove;
        for iu in 0..=nu {
            let theta = (iu as f64 / nu as f64) * std::f64::consts::TAU;
            let stamp = if (iu % 6) == 0 && iv > 6 && iv < 22 {
                -1.15
            } else {
                0.0
            };
            let rr = r + stamp;
            positions.push([rr * theta.cos(), rr * theta.sin(), z]);
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
    TriMesh::from_positions_and_triangles("patterned_bowl", positions, triangles).expect("ok")
}

#[test]
fn test_orthographic_rubbing_raster_produces_visible_ink_within_silhouette() {
    let mesh = make_patterned_bowl_mesh();
    for kind in [
        PlateRasterKind::FrontExteriorRubbing,
        PlateRasterKind::BackInteriorRubbing,
        PlateRasterKind::BottomPlanRubbing,
    ] {
        let img = render_orthographic_rubbing_rgba(
            &mesh,
            kind,
            0.0,
            RUBBING_STANDARD_INK_PCT,
            RUBBING_STANDARD_RES_PX,
            RUBBING_STANDARD_RES_PX,
        );
        let mut opaque_px = 0usize;
        let mut ink_shaded_px = 0usize;
        for p in img.pixels() {
            if p[3] > 128 {
                opaque_px += 1;
                if p[0] < 215 {
                    ink_shaded_px += 1;
                }
            }
        }
        assert!(opaque_px > 2_000);
        assert!(ink_shaded_px > 400);
    }
}

#[test]
fn test_rubbing_params_hash_tracks_every_field() {
    let base = AcademicRubbingParams::default();
    let h0 = rubbing_params_hash(&base);
    assert_eq!(
        h0,
        rubbing_params_hash(&base.clone()),
        "해시가 결정론적이어야 함"
    );

    let mut ink = base.clone();
    ink.ink_density_pct = 79.0;
    let mut light = base.clone();
    light.light_azimuth_deg = 316.0;
    let mut style = base.clone();
    style.style = AcademicRubbingStyle::DryGraphite;
    let mut unroll = base.clone();
    unroll.unroll_mode = Some(strata_shade::UnrollMode::Cylindrical);

    for (name, h) in [
        ("먹물농도", rubbing_params_hash(&ink)),
        ("조명방위", rubbing_params_hash(&light)),
        ("표현기법", rubbing_params_hash(&style)),
        ("전개모드", rubbing_params_hash(&unroll)),
    ] {
        assert_ne!(h, h0, "{} 변경이 해시에 반영되지 않음", name);
    }
}

#[test]
fn test_rubbing_provenance_line_records_standard_values() {
    let line = rubbing_provenance_line(&AcademicRubbingParams::default());
    for token in [
        "L-Std 315/45/1.00",
        "MSII 1.45",
        "Ink 78",
        "전개=평면",
        "투영=직교",
    ] {
        assert!(
            line.contains(token),
            "출처 문자열에 '{}' 없음: {}",
            token,
            line
        );
    }
    let unrolled = AcademicRubbingParams {
        unroll_mode: Some(strata_shade::UnrollMode::Cylindrical),
        ..Default::default()
    };
    let u_line = rubbing_provenance_line(&unrolled);
    assert!(u_line.contains("전개=원통"));
    assert!(u_line.contains("투영=원통전개"));
}

#[test]
fn test_rubbing_render_is_bit_identical() {
    let mesh = make_patterned_bowl_mesh();
    let params = AcademicRubbingParams::default();
    let a = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::BottomPlanRubbing,
        0.0,
        &params,
        160,
        160,
    );
    let b = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::BottomPlanRubbing,
        0.0,
        &params,
        160,
        160,
    );
    assert_eq!(
        a.as_raw(),
        b.as_raw(),
        "같은 입력인데 렌더 결과가 다름(비결정적)"
    );
}

#[test]
fn test_line_weights_snap_to_house_standard() {
    // 「유물제도선.ai」 규격: 0.12/0.15/0.2/0.3/0.4/0.5/0.8
    assert_eq!(STANDARD_LINE_WEIGHTS_MM.len(), 7);
    assert_eq!(snap_to_standard_weight_mm(0.35), 0.30);
    assert_eq!(snap_to_standard_weight_mm(0.26), 0.30);
    assert_eq!(snap_to_standard_weight_mm(0.22), 0.20);
    assert_eq!(snap_to_standard_weight_mm(0.44), 0.40);
    assert_eq!(snap_to_standard_weight_mm(0.9), 0.80);
    // 기본값 자체가 규격값이어야 한다.
    let d = PlateLineStyleConfig::default();
    for w in [
        d.section_width_mm,
        d.outer_width_mm,
        d.aux_width_mm,
        d.dim_width_mm,
    ] {
        assert!(
            STANDARD_LINE_WEIGHTS_MM.contains(&w),
            "기본 선굵기 {} 가 규격값이 아님",
            w
        );
    }
}

#[test]
fn test_fit_scale_preset_keeps_large_plate_on_paper() {
    // 실물 스캔(Φ223.8, 높이 71.2)의 1:1 도판 캔버스는 A3(420×297)를 넘는다.
    let (c_w, c_h) = (653.0, 381.0);
    assert_eq!(
        fit_scale_preset(c_w, c_h, 420.0, 297.0, 12.0),
        ScalePreset::Half1To2
    );
    // 작은 도판은 1:1 유지.
    assert_eq!(
        fit_scale_preset(240.0, 260.0, 420.0, 297.0, 12.0),
        ScalePreset::Full1To1
    );
    // 아주 큰 유물은 1:3 까지 내려간다.
    assert_eq!(
        fit_scale_preset(900.0, 700.0, 420.0, 297.0, 12.0),
        ScalePreset::Third1To3
    );
    // 어떤 축척으로도 안 들어가면 최소 축척(1:4)을 돌려준다.
    assert_eq!(
        fit_scale_preset(2000.0, 1500.0, 420.0, 297.0, 12.0),
        ScalePreset::Quarter1To4
    );
}

#[test]
fn test_parse_plate_mode_cli() {
    assert_eq!(parse_plate_mode("auto"), None);
    assert_eq!(
        parse_plate_mode("1"),
        Some(strata_export::PlateLayoutMode::ALL[0])
    );
    assert_eq!(
        parse_plate_mode("8"),
        Some(strata_export::PlateLayoutMode::ALL[7])
    );
    assert_eq!(parse_plate_mode("9"), None);
    assert_eq!(parse_plate_mode("0"), None);
    assert_eq!(parse_plate_mode("bogus"), None);
}

#[test]
fn test_parse_unroll_mode_cli() {
    assert_eq!(
        parse_unroll_mode("cylindrical"),
        Some(strata_shade::UnrollMode::Cylindrical)
    );
    assert_eq!(
        parse_unroll_mode("conical"),
        Some(strata_shade::UnrollMode::Conical)
    );
    assert_eq!(
        parse_unroll_mode("spherical"),
        Some(strata_shade::UnrollMode::Spherical)
    );
    assert_eq!(parse_unroll_mode("none"), None);
    assert_eq!(parse_unroll_mode("bogus"), None);
}

#[test]
fn test_unrolled_texture_keeps_physical_aspect_ratio() {
    // 원주 364.4mm × 높이 64.0mm 전개 띠: 텍셀 종횡비가 물리 비율과 같아야 1:1 계측이 성립한다.
    let (w, h) = unrolled_texture_dims([364.4, 64.0], 520);
    assert_eq!(w, 520);
    let expected_h = 520.0 * (64.0 / 364.4);
    assert!(
        ((h as f64) - expected_h).abs() < 2.0,
        "전개 텍스처 높이 {}px 는 물리 비율 {:.1}px 와 어긋남",
        h,
        expected_h
    );
}

#[test]
fn test_seam_spanning_triangle_is_culled_in_unroll() {
    // 원주 364mm 전개도에서 봉합선(±π)을 가로지르는 삼각형은 전체 폭 줄무늬를 만든다.
    let width = 364.0;
    assert!(is_seam_spanning_triangle([-182.0, 0.0, 182.0], width));
    // 정상 삼각형은 컬링하지 않는다.
    assert!(!is_seam_spanning_triangle([-100.0, 0.0, 100.0], width));
    assert!(!is_seam_spanning_triangle([10.0, 12.0, 15.0], width));
    // 폭이 0 이면 판정 불가 → 컬링하지 않는다.
    assert!(!is_seam_spanning_triangle([-1.0, 0.0, 1.0], 0.0));
    // B6: 조각 메쉬의 부분 extent가 아니라 전개 전체 폭으로 판정해야 한다.
    // 전체 폭 364mm 기준으로 92mm 삼각형은 정상 → 컬링 금지.
    assert!(!is_seam_spanning_triangle([-46.0, 0.0, 46.0], width));
    // 같은 삼각형도 조각 extent(100mm)로 재면 오판하므로 전체 폭을 써야 한다.
    assert!(is_seam_spanning_triangle([-46.0, 0.0, 46.0], 100.0));
}

#[test]
fn test_unrolled_cylindrical_rubbing_differs_from_plan_and_has_ink() {
    let mesh = make_patterned_bowl_mesh();
    let planar = AcademicRubbingParams::default();
    let unrolled = AcademicRubbingParams {
        unroll_mode: Some(strata_shade::UnrollMode::Cylindrical),
        ..Default::default()
    };

    let img_plan = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::BottomPlanRubbing,
        0.0,
        &planar,
        200,
        200,
    );
    let img_unroll = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::BottomPlanRubbing,
        0.0,
        &unrolled,
        200,
        200,
    );

    let opaque = |img: &RgbaImage| img.pixels().filter(|p| p[3] > 128).count();
    assert!(
        opaque(&img_unroll) > 1_000,
        "원통 전개 탁본이 비어 있음: {} px",
        opaque(&img_unroll)
    );
    assert_ne!(
        img_plan.as_raw(),
        img_unroll.as_raw(),
        "전개 모드를 켜도 평면 탁본과 동일함"
    );
}

#[test]
fn test_action_bar_shows_for_single_view_only() {
    let visible = [0usize, 1, 2, 3];
    // 선택된 레이어가 있으면 그것 하나만.
    assert_eq!(active_action_bar_layer(Some(2), Some(3), &visible), Some(2));
    // 선택이 없으면 호버된 하나만.
    assert_eq!(active_action_bar_layer(None, Some(3), &visible), Some(3));
    // 둘 다 없으면 아무것도 그리지 않는다(도면 가림 방지).
    assert_eq!(active_action_bar_layer(None, None, &visible), None);
    // 숨겨진 레이어가 선택돼 있으면 무시하고 호버로 되돌아간다.
    assert_eq!(active_action_bar_layer(Some(4), Some(1), &visible), Some(1));
}

#[test]
fn test_chs_plate_title_block_and_no_overlapping_aux_views() {
    let mesh = make_patterned_bowl_mesh();
    let report = compute_artifact_measurements(&mesh, ArtifactCategory::CompleteVessel);
    let (left_c, center_t, center_s, right_c) =
        build_chs_plate_title_block("7지구 9호-7 (완형 토기)", "① 완형 토기 표준 도판", &report);
    assert!(left_c.contains("유적명"));
    assert!(center_t.contains("圖版"));
    assert!(center_s.contains("Plate."));
    assert!(right_c.contains("축척"));
}

#[test]
fn test_tdd5_full_bleed_archaeological_millimeter_graph_paper_grid() {
    let sheet_rect = egui::Rect::from_min_size(egui::pos2(400.0, 34.0), egui::vec2(1040.0, 866.0));
    let spec = compute_full_bleed_graph_paper_spec(sheet_rect, 210.0, 140.0, 1.0, egui::Vec2::ZERO);
    assert_eq!(spec.paper_rect, sheet_rect);
    assert!(spec.lines_10mm_x.len() >= 12);
    assert!(spec.lines_10mm_y.len() >= 10);
    assert!(spec.lines_5mm_x.len() >= 10);
    assert!(!spec.cm_labels_top.is_empty());
    assert!(!spec.cm_labels_left.is_empty());
}

#[test]
fn test_tdd6_academic_msii_dog_sigmoid_digital_rubbing_contrast() {
    let mesh = make_patterned_bowl_mesh();
    let params = AcademicRubbingParams {
        ink_density_pct: RUBBING_STANDARD_INK_PCT,
        msii_detail_boost: 1.45,
        sigmoid_contrast: 8.5,
        contact_threshold: -0.12,
        style: AcademicRubbingStyle::WetHanjiInk,
        ..Default::default()
    };
    let img = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::FrontExteriorRubbing,
        0.0,
        &params,
        RUBBING_STANDARD_RES_PX,
        RUBBING_STANDARD_RES_PX,
    );
    let mut deep_ink_ridges = 0usize;
    let mut uninked_paper_grooves = 0usize;
    for p in img.pixels() {
        if p[3] > 200 {
            if p[0] <= 75 {
                deep_ink_ridges += 1;
            } else if p[0] >= 220 {
                uninked_paper_grooves += 1;
            }
        }
    }
    assert!(deep_ink_ridges > 300);
    assert!(uninked_paper_grooves > 150);
}

#[test]
fn test_tdd7_drag_roi_partial_rubbing_window_and_density_control() {
    let mesh = make_patterned_bowl_mesh();
    let mut state = SheetPaneState::default();
    assert!(state.rubbing_roi_mm.is_none());

    state.set_rubbing_roi_mm(Some([20.0, 15.0, 65.0, 50.0]));
    assert_eq!(state.rubbing_roi_mm, Some([20.0, 15.0, 65.0, 50.0]));

    let src = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::FrontExteriorRubbing,
        0.0,
        &state.rubbing_params,
        RUBBING_STANDARD_RES_PX,
        RUBBING_STANDARD_RES_PX,
    );
    let full_cimg =
        rgba_to_color_image_clipped_with_roi(&src, [10.0, 10.0, 110.0, 80.0], Some(-1), None);
    let roi_cimg = rgba_to_color_image_clipped_with_roi(
        &src,
        [10.0, 10.0, 110.0, 80.0],
        Some(-1),
        state.rubbing_roi_mm,
    );
    let full_opaque = full_cimg.pixels.iter().filter(|c| c.a() > 100).count();
    let roi_opaque = roi_cimg.pixels.iter().filter(|c| c.a() > 100).count();
    assert!(roi_opaque > 50 && roi_opaque < full_opaque);
}

#[test]
fn test_tdd15_horizontal_section_above_and_outline_section_with_mm_color_controls() {
    // 1. Image 11: 단면 기본값은 검은 채움이 아닌 외곽선(fill_section_solid == false)이어야 함
    let state = SheetPaneState::default();
    assert!(
        !state.line_style.fill_section_solid,
        "cross-section must default to outline-only (no solid black fill)"
    );
    assert!((state.line_style.section_width_mm - 0.40).abs() < 1e-5);

    // 2. "겹치잖아.? 수평횡단면은 아래에만하라고":
    // 완형 토기 도판에서 수평 횡단면(횡단면_외벽윤곽선)이 평면도와 겹치지 않고 우측 하단(vessel_top_y 아래쪽, vessel_right_x 오른쪽)에만 배치되어야 함!
    let mesh = make_patterned_bowl_mesh();
    let m = compute_artifact_measurements(&mesh, ArtifactCategory::CompleteVessel);
    let vsec = strata_section::slice_mesh_with_fixed_plane(
        &mesh,
        None,
        strata_section::CutPlaneOrientation::VerticalLongitudinal,
        0.0,
        0.0,
        0.25,
    );
    let hsec = strata_section::slice_mesh_with_fixed_plane(
        &mesh,
        None,
        strata_section::CutPlaneOrientation::HorizontalTransverse,
        0.0,
        20.0,
        0.25,
    );
    let plate = strata_export::compose_archaeological_plate(&strata_export::PlateComposeInput {
        title: "테스트 완형 토기",
        measurements: &m,
        vertical_section: &vsec,
        horizontal_sections: &[hsec],
        occluded_spans: &[],
        sherd_subtype: None,
        sherd_strategy: None,
        handle_attachment: None,
        bead_subtype: None,
        bead_drill_type: None,
        wado_cut_ratio: None,
        hemp_density_label: None,
        flip_exterior_right: false,
    });
    let vessel_right_x = plate.raster_regions[0].rect_mm[2];
    let vessel_top_y = plate.raster_regions[0].rect_mm[3];
    let hsec_ring = plate
        .paths
        .iter()
        .find(|p| p.layer_id == "횡단면_외벽윤곽선")
        .expect("horizontal section ring must exist");
    let hsec_mean_x: f64 =
        hsec_ring.points_mm.iter().map(|p| p[0]).sum::<f64>() / (hsec_ring.points_mm.len() as f64);
    let hsec_mean_y: f64 =
        hsec_ring.points_mm.iter().map(|p| p[1]).sum::<f64>() / (hsec_ring.points_mm.len() as f64);
    assert!(
        hsec_mean_x > vessel_right_x && hsec_mean_y < vessel_top_y,
        "horizontal cross-section ({hsec_mean_x:.1}, {hsec_mean_y:.1}mm) must be positioned ONLY in the bottom-right column"
    );
}

#[test]
fn test_tdd16_roi_1to1_rubbing_with_under_image_wheels_and_tight_dimensions() {
    let mut params = AcademicRubbingParams::default();
    let init_ink = params.ink_density_pct;
    let init_contrast = params.sigmoid_contrast;
    let init_sharp = params.msii_detail_boost;
    let init_thresh = params.contact_threshold;

    // Image 14: 탁본 이미지 바로 밑 4종 미세조정 휠 스텝 동작 검증
    params.apply_wheel_step(RubbingFineWheelKind::InkDensity, 5);
    params.apply_wheel_step(RubbingFineWheelKind::SigmoidContrast, 4);
    params.apply_wheel_step(RubbingFineWheelKind::MsiiRidgeSharpness, 6);
    params.apply_wheel_step(RubbingFineWheelKind::PaperContactThreshold, -3);

    assert!((params.ink_density_pct - (init_ink + 5.0)).abs() < 1e-6);
    assert!((params.sigmoid_contrast - (init_contrast + 0.8)).abs() < 1e-4);
    assert!((params.msii_detail_boost - (init_sharp + 0.30)).abs() < 1e-4);
    assert!((params.contact_threshold - (init_thresh - 0.03)).abs() < 1e-4);
}

#[test]
fn test_tdd20_paper_sheet_5cm_grid_and_non_overlapping_line_widgets() {
    // 1. "방안지 여백은 무한대로 배경을 하고 용지만 붉은선으로 표시하게 하자":
    // 방안선이 용지 경계(paper_bounds_rect) 바깥 여백까지 무한대(sheet_rect 전체)로 확장되고, 용지 경계 사각형이 정확히 산출됨을 검증!
    // (내부 적색 제거: 5cm 격자선은 중립 회색 — 용지 경계 적색만 유지.)
    let sheet_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 760.0));
    let spec = compute_paper_sheet_graph_spec(
        sheet_rect,
        PaperSheetPreset::A3Landscape,
        1.0,
        egui::Vec2::ZERO,
    );
    assert!(
        spec.paper_bounds_rect.width() < sheet_rect.width()
            && spec.paper_bounds_rect.height() < sheet_rect.height(),
        "paper bounds must sit inside the viewport as the red-bordered publication sheet"
    );
    assert!(
        !spec.lines_50mm_grid_x.is_empty() && !spec.lines_50mm_grid_y.is_empty(),
        "must generate 5cm (50mm) thin neutral grid lines"
    );
    assert!(
        spec.lines_10mm_x
            .iter()
            .any(|&gx| gx < spec.paper_bounds_rect.left() - 2.0)
            && spec
                .lines_10mm_x
                .iter()
                .any(|&gx| gx > spec.paper_bounds_rect.right() + 2.0),
        "infinite graph paper grid lines must extend into the margin beyond paper_bounds_rect"
    );

    // 2. Image 2: 모든 선 조정 위젯이 유물이 움직이면 함께 이동하고 줌인/줌아웃 시 절대 중복(겹침)되지 않아야 함!
    for zoom in [0.55_f32, 1.0_f32, 2.2_f32] {
        let rects = compute_non_overlapping_line_widget_rects(
            egui::pos2(480.0, 420.0),
            zoom * 2.5,
            [-62.0, 0.0, 62.0, 63.0],
            [0.0, 118.0, 26.0],
            sheet_rect,
        );
        assert_eq!(
            rects.len(),
            4,
            "exact 4 line widgets (단면선, 외곽선, 횡단면, 치수선)"
        );
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                assert!(
                    !rects[i].intersects(rects[j]),
                    "zoom={zoom}: widget {i} {:?} must NEVER overlap widget {j} {:?}",
                    rects[i],
                    rects[j]
                );
            }
        }
    }
}

#[test]
fn test_aux_line_default_is_neutral_gray_not_terra_red() {
    // task-14: 보조선(횡단면·중심·십자선) 기본값은 중립 회색이어야 한다.
    // (용도 기준 식별 — 사용자가 색상 편집기로 바꾸는 것은 허용.)
    let style = PlateLineStyleConfig::default();
    let [r, g, b] = style.aux_color_rgb;
    assert!(
        (r as i16 - g as i16).abs() < 12 && (g as i16 - b as i16).abs() < 12,
        "보조선 기본값이 중립 회색이 아님: [{r},{g},{b}]"
    );
    assert_ne!(
        style.aux_color_rgb,
        [181, 87, 58],
        "보조선 기본값이 테라코타 적색이면 안 됨"
    );
}

#[test]
fn test_red_rules_keep_section_and_paper_but_not_interior() {
    // task-14 색상 계약: 단면선 빨강(#FF0000)과 용지 경계 적색은 유지,
    // 내부(5cm 방안선·보조선 기본값)는 중립 회색이어야 한다.
    use strata_core::strata_tokens::tokens::PROFILE_RED;
    assert_eq!(
        PROFILE_RED.rgb,
        [255, 0, 0],
        "단면선 빨강 정본이 바뀌면 안 됨"
    );
    assert!(
        PAPER_BORDER_RED_RGB[0] > PAPER_BORDER_RED_RGB[1] + 100,
        "용지 경계는 적색을 유지해야 함: {:?}",
        PAPER_BORDER_RED_RGB
    );
    let [r, g, b] = SHEET_GRID_5CM_RGB;
    assert!(
        (r as i16 - g as i16).abs() < 12 && (g as i16 - b as i16).abs() < 12,
        "5cm 방안선이 중립 회색이 아님: [{r},{g},{b}]"
    );
    assert!(
        r < 150,
        "5cm 방안선에 적색이 남아 있으면 안 됨: [{r},{g},{b}]"
    );
}

#[test]
fn test_preview_stroke_color_decision_paths() {
    // task-20 ③: 상수값이 아니라 실제 색 결정 경로를 검증한다.
    // 서로 다른 감시색으로 각 분기를 식별한다.
    use egui::Color32;
    let aux = Color32::from_rgb(1, 2, 3);
    let profile = Color32::from_rgb(4, 5, 6);
    let select = Color32::from_rgb(7, 8, 9);
    let muted = Color32::from_rgb(10, 11, 12);
    let fallback = Color32::from_rgb(13, 14, 15);
    let decide = |hex| preview_stroke_color_for_hex(hex, aux, profile, select, muted, fallback);
    assert_eq!(decide("#B5573A"), aux, "테라코타는 보조색으로 가야 함");
    assert_eq!(decide("#FF0000"), profile, "단면 빨강은 보존되어야 함");
    assert_eq!(decide("#2A78D6"), select);
    assert_eq!(decide("#5E5D59"), muted);
    assert_eq!(decide("#123456"), fallback, "미지정 색은 폴백으로 가야 함");
    assert_eq!(decide(""), fallback);
}

#[test]
fn test_tdd21_all8_feedback_illustrator_vector_plan_measured_hsec_and_individual_flips() {
    // 1. Image 0 & Image 1: 기본값에서 좌측 평단면은 일러스트용 순수 벡터 선화(show_elevation_rubbing_overlay == false)이고,
    //    우측 상단에 1:1 정밀 탁본(BottomPlanRubbing)이 배치됨을 검증
    let state = SheetPaneState::default();
    assert!(
        !state.show_elevation_rubbing_overlay,
        "left column (1:1 plan + 1:1 section) must default to pure Illustrator vector line art"
    );

    let mesh = make_patterned_bowl_mesh();
    let m = compute_artifact_measurements(&mesh, ArtifactCategory::CompleteVessel);
    let vsec = strata_section::slice_mesh_with_fixed_plane(
        &mesh,
        None,
        strata_section::CutPlaneOrientation::VerticalLongitudinal,
        0.0,
        0.0,
        0.25,
    );
    let hsec = strata_section::slice_mesh_with_fixed_plane(
        &mesh,
        None,
        strata_section::CutPlaneOrientation::HorizontalTransverse,
        0.0,
        24.0,
        0.25,
    );
    let plate = strata_export::compose_archaeological_plate(&strata_export::PlateComposeInput {
        title: "테스트 완형 토기",
        measurements: &m,
        vertical_section: &vsec,
        horizontal_sections: &[hsec],
        occluded_spans: &[],
        sherd_subtype: None,
        sherd_strategy: None,
        handle_attachment: None,
        bead_subtype: None,
        bead_drill_type: None,
        wado_cut_ratio: None,
        hemp_density_label: None,
        flip_exterior_right: false,
    });
    assert!(
        plate
            .paths
            .iter()
            .any(|p| p.layer_id == "평면도_구연내경선"),
        "1:1 plan view must contain Illustrator concentric vector lines"
    );
    assert!(
        plate.paths.iter().any(|p| p.layer_id == "횡단면_단면_실선"),
        "1:1 horizontal cross-section must contain measured cross-section polygon"
    );
    assert!(
        plate.raster_regions.len() >= 2
            && plate.raster_regions[1].rect_mm[0] > plate.raster_regions[0].rect_mm[2],
        "1:1 precision rubbing must be placed in the right column at 1:1 scale"
    );

    // 2. Image 7: 개별 도면 뷰(0~3)의 90° 회전 · 180° 뒤집기 · 앞으로 연속 뒤집기 동작 검증
    let mut tf = IndividualViewTransform2D::default();
    let center = [100.0, 100.0];
    let p = [120.0, 110.0];
    tf.rotate_cw_90();
    let p_rot90 = tf.transform_point_around(p, center);
    assert!((p_rot90[0] - 110.0).abs() < 1e-6 && (p_rot90[1] - 80.0).abs() < 1e-6);

    let mut tf_flip = IndividualViewTransform2D::default();
    tf_flip.flip_forward_continuous_90();
    assert_eq!(tf_flip.flip_forward_steps, 1);
    tf_flip.flip_forward_continuous_90();
    assert_eq!(tf_flip.flip_forward_steps, 2);
    let p_flip180 = tf_flip.transform_point_around(p, center);
    assert!((p_flip180[0] - 120.0).abs() < 1e-6 && (p_flip180[1] - 90.0).abs() < 1e-6);

    // 3. Image 6: 3D 유물 90° 앞으로뒤집기 · 180° 상하뒤집기 검증
    let mut v3d = crate::view3d_pane::View3DPaneState::default();
    v3d.flip_artifact_forward_90();
    assert!((v3d.tilt_deg - 90.0).abs() < 1e-5);
    v3d.flip_artifact_forward_90();
    assert!((v3d.tilt_deg.abs() - 180.0).abs() < 1e-5);
    v3d.flip_artifact_upside_down_180();
    assert!(v3d.tilt_deg.abs() < 1e-5);
}

#[test]
fn test_tdd22_measured_plan_dashed_fractures_light_control_and_6layer_delete() {
    // 1. Image 0 ("실제 모습이 평면도에 나와야하지않나? 깨진부분은 점선아닌가?"):
    // 구연부 일부가 결손(깨진)된 메쉬에서 extract_topdown_plan_slice_2d가 실제 윤곽과 결손 점선(평면도_결손복원_점선)을 생성함을 검증!
    let mut mesh = make_patterned_bowl_mesh();
    for pos in &mut mesh.positions {
        let ang_deg = pos[1].atan2(pos[0]).to_degrees().rem_euclid(360.0);
        if (25.0..=55.0).contains(&ang_deg) && pos[2] > 42.0 {
            pos[2] = 36.0; // 구연부 결손(깨진 부분) 모사
            pos[0] *= 0.86;
            pos[1] *= 0.86;
        }
    }
    let mut v3d = crate::view3d_pane::View3DPaneState::default();
    let top_plan = crate::view3d_pane::extract_topdown_plan_slice_2d(&mesh, &v3d);
    assert!(
        top_plan.polylines_2d.len() >= 3,
        "top-down plan slice must extract outer/inner rim, foot, and broken sector polylines"
    );
    let m = compute_artifact_measurements(&mesh, ArtifactCategory::CompleteVessel);
    let vsec = crate::view3d_pane::slice_transformed_mesh_world_locked(&mesh, &v3d, true);
    let hsec = crate::view3d_pane::slice_transformed_mesh_world_locked(&mesh, &v3d, false);
    v3d.custom_section_line_mm = Some([[-42.0, -18.0], [42.0, 22.0]]);
    let custom_sec = crate::view3d_pane::slice_custom_line_section_2d(&mesh, &v3d)
        .expect("custom section A-A' must be extracted");

    let plate = strata_export::compose_archaeological_plate(&strata_export::PlateComposeInput {
        title: "결손 구연 완형 토기",
        measurements: &m,
        vertical_section: &vsec,
        horizontal_sections: &[hsec, top_plan, custom_sec],
        occluded_spans: &[],
        sherd_subtype: None,
        sherd_strategy: None,
        handle_attachment: None,
        bead_subtype: None,
        bead_drill_type: None,
        wado_cut_ratio: None,
        hemp_density_label: None,
        flip_exterior_right: false,
    });
    assert!(
        plate
            .paths
            .iter()
            .any(|p| p.layer_id == "평면도_결손복원_점선" && p.dash_pattern_mm.is_some()),
        "chipped/broken rim segment in 1:1 plan view must be rendered as a dashed reconstruction line"
    );
    assert!(
        plate
            .paths
            .iter()
            .any(|p| p.layer_id.starts_with("추가단면_")),
        "custom section A-A' must be placed in its dedicated layer"
    );

    // 2. Image 2 ("광원을 조정할수있는 작은창"): 광원 방위각/고도각/세기 벡터 정규화 검증
    v3d.light_azimuth_deg = 225.0;
    v3d.light_elevation_deg = 35.0;
    v3d.light_intensity = 1.35;
    let lvec = v3d.compute_light_direction_vector();
    let llen = (lvec[0] * lvec[0] + lvec[1] * lvec[1] + lvec[2] * lvec[2]).sqrt();
    assert!((llen - 1.0).abs() < 1e-4);

    // 3. 전 레이어 관리 ("레이어 모든것에는 방향조정 90도씩 delete키로 지우기 추가단면선 생성시 이것도 레이어에생성 삭제가능하고 키고끌수있게")
    let mut sheet = SheetPaneState {
        selected_layer_idx: Some(4), // ⑤ 추가 단면 A-A' 선택
        ..SheetPaneState::default()
    };
    sheet.rotate_selected_layer_cw_90();
    assert_eq!(sheet.view_transforms[4].rot_90_steps, 1);
    let deleted = sheet.delete_selected_layer();
    assert_eq!(deleted, Some(4));
    assert!(!sheet.view_transforms[4].visible);
    assert!(
        sheet.request_clear_custom_section,
        "deleting layer 4 (추가 단면 A-A') must also signal clearing custom_section_line_mm"
    );
}

#[test]
fn test_dark_offscreen_keeps_plate_output_pixels_identical() {
    // 철칙: 화면 크롬만 다크, 실측 도판 픽셀은 라이트와 동일해야 한다.
    use crate::sheet_pane::{
        render_full_workspace_offscreen_png, render_full_workspace_offscreen_png_dark,
    };
    use strata_export::{ComposedPlate2D, PlateLayoutMode};

    let plate = ComposedPlate2D {
        mode: PlateLayoutMode::CompleteVesselStandard,
        title: String::from("다크 불변 검증판"),
        scale_preset: ScalePreset::Full1To1,
        canvas_width_mm: 120.0,
        canvas_height_mm: 90.0,
        flip_exterior_right: false,
        paths: vec![
            strata_export::PlatePathElement {
                layer_id: String::from("단면_실선"),
                points_mm: vec![[10.0, 10.0], [110.0, 80.0]],
                closed: false,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.4,
                dash_pattern_mm: None,
                fill_hex: None,
            },
            strata_export::PlatePathElement {
                layer_id: String::from("횡단면_단면_실선"),
                points_mm: vec![[20.0, 20.0], [60.0, 20.0], [40.0, 60.0]],
                closed: true,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.3,
                dash_pattern_mm: None,
                fill_hex: Some(String::from("#141413")),
            },
        ],
        callouts: vec![],
        raster_regions: vec![],
    };
    let mesh = make_patterned_bowl_mesh();
    let light = render_full_workspace_offscreen_png(&plate, &mesh, false, false);
    let dark = render_full_workspace_offscreen_png_dark(&plate, &mesh, false, false);
    assert_eq!((light.width(), light.height()), (1440, 900));
    assert_eq!((dark.width(), dark.height()), (1440, 900));

    // 시트 영역(헤더 제외)에서 배경 제외 그려진 픽셀 집합이 양쪽 동일해야 한다.
    const LIGHT_BG: [u8; 3] = [250, 249, 245];
    let mut drawn_light = Vec::new();
    let mut drawn_dark = Vec::new();
    for y in 198..870u32 {
        for x in 518..1440u32 {
            let pl = light.get_pixel(x, y).0;
            let pd = dark.get_pixel(x, y).0;
            if pl[0] != LIGHT_BG[0] || pl[1] != LIGHT_BG[1] || pl[2] != LIGHT_BG[2] {
                drawn_light.push((x, y, pl));
            }
            if pd != [255, 255, 255, 255] {
                drawn_dark.push((x, y, pd));
            }
        }
    }
    assert!(!drawn_light.is_empty(), "시트에 그려진 도판이 없음");
    assert_eq!(
        drawn_light, drawn_dark,
        "다크 모드에서 실측 도판 픽셀이 바뀜 (출력물 불변 위반)"
    );

    // 화면 크롬(상단 바)은 실제로 바뀌어야 한다.
    assert_ne!(
        light.get_pixel(720, 10),
        dark.get_pixel(720, 10),
        "다크 모드에서 상단 바가 바뀌지 않음"
    );
    // 다크 바탕은 어둡고, 다크 용지는 흰색이어야 한다.
    let db = dark.get_pixel(720, 10).0;
    let dark_luma = (u16::from(db[0]) + u16::from(db[1]) + u16::from(db[2])) / 3;
    assert!(dark_luma < 80, "다크 상단 바가 어둡지 않음: {db:?}");
    assert_eq!(
        dark.get_pixel(1000, 500).0,
        [255, 255, 255, 255],
        "다크 모드 용지가 흰색이 아님"
    );
}

#[test]
fn test_msii_radii_scale_with_render_extent() {
    // B-통일 고정점: 기준 180px에서 기존 고정값과 동일, 비율 유지.
    assert_eq!(msii_radii_for_extent(180), [1, 2, 4, 8]);
    assert_eq!(msii_radii_for_extent(90), [1, 1, 2, 4]);
    assert_eq!(msii_radii_for_extent(360), [2, 4, 8, 16]);
    assert_eq!(msii_radii_for_extent(48), [1, 1, 1, 2]);
    assert_eq!(msii_radii_for_extent(240), [1, 3, 5, 11]);
}

#[test]
fn test_rubbing_is_deterministic_across_repeated_renders() {
    // B-통일 고정점: 사용자 세팅 없이 같은 입력이면 바이트 동일 탁본.
    let mesh = make_patterned_bowl_mesh();
    let params = AcademicRubbingParams::default();
    assert_eq!(params.ink_density_pct, RUBBING_STANDARD_INK_PCT);
    let a = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::FrontExteriorRubbing,
        0.0,
        &params,
        RUBBING_STANDARD_RES_PX,
        RUBBING_STANDARD_RES_PX,
    );
    let b = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::FrontExteriorRubbing,
        0.0,
        &params,
        RUBBING_STANDARD_RES_PX,
        RUBBING_STANDARD_RES_PX,
    );
    assert_eq!(
        a.as_raw(),
        b.as_raw(),
        "같은 입력의 탁본이 다르면 일관성 위반"
    );
    assert!(a.pixels().any(|p| p[3] > 32), "탁본에 잉크가 있어야 함");
}

#[test]
fn test_ddg_cache_key_distinguishes_same_size_meshes() {
    // B2 고정점: 이름·개수가 같아도 내용이 다르면 다른 캐시 항목이어야 한다.
    let mut mesh_a = make_patterned_bowl_mesh();
    mesh_a.name = String::from("cache_probe_same_name");
    let mut mesh_b = mesh_a.clone();
    mesh_b.positions[10][2] += 7.5;
    mesh_b.bbox.max[2] += 7.5;
    assert_eq!(mesh_a.positions.len(), mesh_b.positions.len());
    assert_eq!(mesh_content_hash(&mesh_a), mesh_content_hash(&mesh_a));
    assert_ne!(
        mesh_content_hash(&mesh_a),
        mesh_content_hash(&mesh_b),
        "내용이 다르면 해시가 달라야 함"
    );
    let ca = get_or_build_ddg_mesh(&mesh_a);
    let cb = get_or_build_ddg_mesh(&mesh_b);
    assert!(
        !std::sync::Arc::ptr_eq(&ca, &cb),
        "다른 내용의 메쉬가 같은 DDG 캐시를 타면 안 됨"
    );
}

#[test]
fn test_resolve_rim_label_width_mm_always_uses_report() {
    // 추가분(치수 단일원천): 리드 재현 케이스 — 상단바 Φ125.2 vs 그려진 폭 109.4.
    // 그려진 폭이 12% 좁아도 라벨은 리포트값을 써야 한다.
    assert!((resolve_rim_label_width_mm(125.2, 109.4) - 125.2).abs() < 1e-9);
    assert!((resolve_rim_label_width_mm(125.2, 140.0) - 125.2).abs() < 1e-9);
    assert!((resolve_rim_label_width_mm(0.0, 109.4) - 109.4).abs() < 1e-9);
    assert!((resolve_rim_label_width_mm(f64::NAN, 109.4) - 109.4).abs() < 1e-9);
}

#[test]
fn test_light_hash_quantization_matches_view3d_cache() {
    // B3 고정점: 0.4° 광원 이동은 해시 동일, 0.6° 이동은 해시 상이해야 한다 (0.5° 양자화).
    let hash_at = |az: f32| {
        let p = AcademicRubbingParams {
            light_azimuth_deg: az,
            ..AcademicRubbingParams::default()
        };
        rubbing_params_hash(&p)
    };
    assert_eq!(hash_at(315.0), hash_at(315.2));
    assert_ne!(hash_at(315.0), hash_at(315.6));
}

#[test]
fn test_plate_gauge_lines_use_report_values_across_flips() {
    // 추가분: 상태바·도판(구경선/저경선)·치수선이 뒤집기 각도와 무관하게 리포트값과 일치해야 한다.
    use strata_export::{compose_archaeological_plate, PlateComposeInput, PlateLayoutMode};
    let mesh = make_patterned_bowl_mesh();
    let m = compute_artifact_measurements(&mesh, ArtifactCategory::CompleteVessel);
    assert!(m.rim_or_outer_diameter_mm > 1.0);
    let mut poses = Vec::new();
    for tilt in [0.0, 45.0, 90.0, 180.0, -90.0] {
        poses.push((tilt, 0.0, 0.0));
    }
    poses.push((0.0, 90.0, 0.0));
    poses.push((0.0, 0.0, 90.0));
    for (tilt, yaw, roll) in poses {
        let v3d = crate::view3d_pane::View3DPaneState {
            tilt_deg: tilt,
            active_section_angle_deg: yaw,
            roll_deg: roll,
            ..Default::default()
        };
        let vsec = crate::view3d_pane::slice_transformed_mesh_world_locked(&mesh, &v3d, true);
        let plate = compose_archaeological_plate(&PlateComposeInput {
            title: "뒤집기 정합",
            measurements: &m,
            vertical_section: &vsec,
            horizontal_sections: &[],
            occluded_spans: &[],
            sherd_subtype: None,
            sherd_strategy: None,
            handle_attachment: None,
            bead_subtype: None,
            bead_drill_type: None,
            wado_cut_ratio: None,
            hemp_density_label: None,
            flip_exterior_right: false,
        });
        assert_eq!(plate.mode, PlateLayoutMode::CompleteVesselStandard);
        let gauge_width = |layer: &str| {
            let xs: Vec<f64> = plate
                .paths
                .iter()
                .filter(|p| p.layer_id == layer)
                .flat_map(|p| p.points_mm.iter().map(|q| q[0]))
                .collect();
            assert!(
                !xs.is_empty(),
                "tilt={tilt} yaw={yaw} roll={roll}: {layer} 없음"
            );
            xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                - xs.iter().cloned().fold(f64::INFINITY, f64::min)
        };
        assert!(
            (gauge_width("구경_기준선") - m.rim_or_outer_diameter_mm).abs() < 1e-6,
            "tilt={tilt} yaw={yaw} roll={roll}: 구경선이 리포트와 다름"
        );
        assert!(
            (gauge_width("저경_기준선") - m.base_diameter_or_width_mm).abs() < 1e-6,
            "tilt={tilt} yaw={yaw} roll={roll}: 저경선이 리포트와 다름"
        );
    }
}

#[test]
fn test_msii_mix_weights_sum_to_one() {
    // B8: 명명된 상수의 불변식 — 가중합이 발산하지 않아야 한다.
    // (합이 1이 아니면 탁본 명암이 전체적으로 밝아지거나 어두워진다.)
    let sum: f32 = MSII_MIX_WEIGHTS.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-6,
        "MSII 혼합 가중치 합이 1이 아님: {sum}"
    );
}

#[test]
fn test_relief_percentile_95_scale_clamps_and_selects() {
    // ① Rubbing.cpp:357-359 이식: |D| 95백분위 + [0.06, 1.5] 클램프.
    let mut uniform = vec![0.5f32; 100];
    assert!((relief_percentile_95_scale(&mut uniform) - 0.5).abs() < 1e-6);
    let mut ramp: Vec<f32> = (0..100).map(|i| i as f32 * 0.01).collect();
    let s = relief_percentile_95_scale(&mut ramp);
    assert!((s - 0.95).abs() < 1e-4, "95번째 백분위여야 함: {s}");
    let mut huge: Vec<f32> = (0..100).map(|i| i as f32).collect();
    assert!((relief_percentile_95_scale(&mut huge) - 1.5).abs() < 1e-6);
    let mut tiny = vec![0.001f32; 50];
    assert!((relief_percentile_95_scale(&mut tiny) - 0.06).abs() < 1e-6);
    let mut empty: Vec<f32> = Vec::new();
    assert!((relief_percentile_95_scale(&mut empty) - 1.0).abs() < 1e-6);
}

fn make_flat_bump_mesh() -> TriMesh {
    // 넓은 양각(지름 60mm·높이 2mm 완만 돔) + 평탄 바탕 — 백화 재현용.
    let n = 61usize;
    let mut positions = Vec::with_capacity(n * n);
    for iy in 0..n {
        for ix in 0..n {
            let x = -60.0 + 120.0 * (ix as f64) / ((n - 1) as f64);
            let y = -60.0 + 120.0 * (iy as f64) / ((n - 1) as f64);
            let r = (x * x + y * y).sqrt();
            let z = if r < 30.0 {
                2.0 * (r / 30.0 * std::f64::consts::FRAC_PI_2).cos()
            } else {
                0.0
            };
            positions.push([x, y, z]);
        }
    }
    let mut triangles = Vec::with_capacity(2 * (n - 1) * (n - 1));
    for iy in 0..(n - 1) {
        for ix in 0..(n - 1) {
            let a = (iy * n + ix) as u32;
            let b = a + 1;
            let c = a + n as u32;
            let d = c + 1;
            triangles.push([a, b, d]);
            triangles.push([a, d, c]);
        }
    }
    TriMesh::from_positions_and_triangles("flat_bump", positions, triangles).expect("valid")
}

fn mean_ink_in_disk(img: &RgbaImage, cx: u32, cy: u32, r: u32) -> f64 {
    // u32 덧셈 오버플로 방지: 상한 클램프와 함께 saturating_add를 쓴다.
    let mut sum = 0.0f64;
    let mut cnt = 0u64;
    for y in cy.saturating_sub(r)..=cy.saturating_add(r).min(img.height() - 1) {
        for x in cx.saturating_sub(r)..=cx.saturating_add(r).min(img.width() - 1) {
            let dx = x as i64 - cx as i64;
            let dy = y as i64 - cy as i64;
            if dx * dx + dy * dy > (r as i64) * (r as i64) {
                continue;
            }
            let p = img.get_pixel(x, y);
            if p.0[3] == 0 {
                continue;
            }
            sum += f64::from(p.0[0]);
            cnt += 1;
        }
    }
    assert!(cnt > 0, "디스크에 칠해진 픽셀이 없음");
    sum / cnt as f64
}

#[test]
fn test_broad_relief_does_not_wash_out_to_white() {
    // ① 백화: 넓은 양각 윗면이 순백으로 지워지면 안 된다.
    // 돔 정상부·평탄 바탕 모두 먹을 머금어야 한다 (순백 255 대비 확연히 어둡게).
    let mesh = make_flat_bump_mesh();
    let params = AcademicRubbingParams::default();
    let img = render_orthographic_rubbing_rgba_with_params(
        &mesh,
        PlateRasterKind::BottomPlanRubbing,
        0.0,
        &params,
        200,
        200,
    );
    // 메쉬 120mm → 200px: 중심(100,100)=돔 정상, (150,50)=평탄 바탕(돔·가장자리 중간).
    // 구 코드(고정 이득+pad 0.44)에서는 둘 다 230 이상 순백으로 지워진다.
    let top = mean_ink_in_disk(&img, 100, 100, 12);
    let flat = mean_ink_in_disk(&img, 150, 50, 8);
    assert!(top < 180.0, "돔 정상부가 순백으로 지워짐: {top}");
    assert!(flat < 190.0, "평탄 바탕이 순백으로 지워짐: {flat}");
}

#[test]
fn test_msii_normalization_adapts_to_relief_scale() {
    // ① 자동정규화: 문양 깊이가 1/20로 줄어도 명암 대비가 유지되어야 한다.
    // 고정 이득이면 완만 문양이 하얗게 날아간다.
    let mesh = make_patterned_bowl_mesh();
    let mean_z: f64 =
        mesh.positions.iter().map(|p| p[2]).sum::<f64>() / mesh.positions.len() as f64;
    let flat_pos: Vec<[f64; 3]> = mesh
        .positions
        .iter()
        .map(|p| [p[0], p[1], mean_z + (p[2] - mean_z) * 0.05])
        .collect();
    let flat_mesh = TriMesh::from_positions_and_triangles(
        "patterned_bowl_flat",
        flat_pos,
        mesh.triangles.clone(),
    )
    .expect("valid");
    let params = AcademicRubbingParams::default();
    let contrast_of = |m: &TriMesh| {
        let img = render_orthographic_rubbing_rgba_with_params(
            m,
            PlateRasterKind::BottomPlanRubbing,
            0.0,
            &params,
            200,
            200,
        );
        let vals: Vec<f64> = img
            .pixels()
            .filter(|p| p.0[3] != 0)
            .map(|p| f64::from(p.0[0]))
            .collect();
        assert!(!vals.is_empty(), "칠해진 픽셀이 없음");
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
        var.sqrt()
    };
    let c_full = contrast_of(&mesh);
    let c_flat = contrast_of(&flat_mesh);
    assert!(
        c_flat >= c_full * 0.7,
        "완만 문양 대비가 유지되어야 함: full={c_full:.1} flat={c_flat:.1}"
    );
}

fn mean_ink_in_annulus(img: &RgbaImage, cx: u32, cy: u32, r0: u32, r1: u32) -> f64 {
    // u32 덧셈 오버플로 방지: 상한 클램프와 함께 saturating_add를 쓴다.
    let mut sum = 0.0f64;
    let mut cnt = 0u64;
    let r1sq = (r1 as i64) * (r1 as i64);
    let r0sq = (r0 as i64) * (r0 as i64);
    for y in cy.saturating_sub(r1)..=cy.saturating_add(r1).min(img.height() - 1) {
        for x in cx.saturating_sub(r1)..=cx.saturating_add(r1).min(img.width() - 1) {
            let dx = x as i64 - cx as i64;
            let dy = y as i64 - cy as i64;
            let d2 = dx * dx + dy * dy;
            if d2 < r0sq || d2 > r1sq {
                continue;
            }
            let p = img.get_pixel(x, y);
            if p.0[3] == 0 {
                continue;
            }
            sum += f64::from(p.0[0]);
            cnt += 1;
        }
    }
    assert!(cnt > 0, "고리 영역에 칠해진 픽셀이 없음");
    sum / cnt as f64
}

#[test]
fn test_convex_and_concave_relief_keep_their_sign() {
    // ③ 부호 소실: 양각(돔) 사면은 먹을 머금고, 음각(크레이터) 사면은 한지로 남아야 한다.
    // H 부호를 지우면(abs*화면부호) 둘의 명암 순서가 뒤바뀌거나 같아진다.
    let bump = make_flat_bump_mesh();
    let mut crater_pos = bump.positions.clone();
    for p in &mut crater_pos {
        p[2] = -p[2];
    }
    let crater =
        TriMesh::from_positions_and_triangles("flat_crater", crater_pos, bump.triangles.clone())
            .expect("valid");
    let params = AcademicRubbingParams::default();
    let render = |m: &TriMesh| {
        render_orthographic_rubbing_rgba_with_params(
            m,
            PlateRasterKind::BottomPlanRubbing,
            0.0,
            &params,
            200,
            200,
        )
    };
    // 돔 사면 고리(r 8..20px) vs 크레이터 사면 고리: 양각이 확연히 어두워야 한다.
    let bump_ring = mean_ink_in_annulus(&render(&bump), 100, 100, 8, 20);
    let crater_ring = mean_ink_in_annulus(&render(&crater), 100, 100, 8, 20);
    assert!(
        bump_ring + 15.0 < crater_ring,
        "양각 사면이 음각 사면보다 어두워야 함: bump={bump_ring:.1} crater={crater_ring:.1}"
    );
}

#[test]
fn test_rotated_90_vessel_plan_and_section_fidelity_and_ui_layout_fixes() {
    use strata_export::{compose_archaeological_plate, PlateComposeInput};
    let mesh = make_patterned_bowl_mesh();
    let m = compute_artifact_measurements(&mesh, ArtifactCategory::CompleteVessel);

    // ⑥ 광원 조정 팝업 기본 비활성화 검증
    let mut v3d = crate::view3d_pane::View3DPaneState::default();
    assert!(
        !v3d.show_light_window,
        "초기 실행 시 광원 조정 팝업은 닫혀 있어야 함"
    );

    // 90° 앞으로 뒤집기 회전 적용
    v3d.flip_artifact_forward_90();
    assert!((v3d.tilt_deg - 90.0).abs() < 1e-5);

    let vsec = crate::view3d_pane::slice_transformed_mesh_world_locked(&mesh, &v3d, true);
    let hsec = crate::view3d_pane::slice_transformed_mesh_world_locked(&mesh, &v3d, false);
    let top_plan = crate::view3d_pane::extract_topdown_plan_slice_2d(&mesh, &v3d);
    let h_sections = vec![hsec, top_plan];

    let plate = compose_archaeological_plate(&PlateComposeInput {
        title: "90도회전_1대1_정합검증",
        measurements: &m,
        vertical_section: &vsec,
        horizontal_sections: &h_sections,
        occluded_spans: &[],
        sherd_subtype: None,
        sherd_strategy: None,
        handle_attachment: None,
        bead_subtype: None,
        bead_drill_type: None,
        wado_cut_ratio: None,
        hemp_density_label: None,
        flip_exterior_right: false,
    });

    // 1. 90° 회전 시 수직 단면도가 기존 유물 높이(55mm * 1.15 = 63.25mm 클램프)로 잘리지 않고
    //    중간 동체부 원형 절단 직경(~80.8mm) 전체 높이를 1:1로 유지하는지 검증
    let elev_reg = &plate.raster_regions[0];
    let elev_h = (elev_reg.rect_mm[3] - elev_reg.rect_mm[1]).abs();
    assert!(
        elev_h > 75.0,
        "90° 회전 단면 높이가 1:1 실측 절단 직경(>75mm, 무클램프)을 반영해야 함 (실제: {elev_h:.1}mm)"
    );

    // 2. 평면도 아래 불필요한 내부 원(`평면도_굽외경선`, `평면도_내저원각선`) 및 회전 시 가짜 외곽 점선 원(`평면도_결손복원_점선`)이 없어야 함
    assert!(
        !plate.paths.iter().any(|p| p.layer_id == "평면도_굽외경선"),
        "불필요한 평면도_굽외경선이 생성되면 안 됨"
    );
    assert!(
        !plate
            .paths
            .iter()
            .any(|p| p.layer_id == "평면도_내저원각선"),
        "불필요한 평면도_내저원각선이 생성되면 안 됨"
    );
    assert!(
        !plate
            .paths
            .iter()
            .any(|p| p.layer_id == "평면도_결손복원_점선"),
        "90° 회전 시 가짜 결손복원 점선 원이 생성되면 안 됨"
    );

    // ⑦ 외저면_십자선이 레이어 0(① 1:1 평면도)으로 분류되는지 검증
    assert_eq!(classify_plate_path_layer_idx("외저면_십자선"), 0);

    // ② 선 두께 4번째 카드가 우상단 레이어 패널(238px)과 겹치지 않는지 검증
    let viewport = egui::Rect::from_min_size(egui::pos2(400.0, 40.0), egui::vec2(1000.0, 800.0));
    let cards = compute_non_overlapping_line_widget_rects(
        egui::pos2(900.0, 400.0),
        3.0,
        [20.0, 20.0, 140.0, 100.0],
        [0.0, 0.0, 0.0],
        viewport,
    );
    let layer_panel_left = viewport.right() - 246.0;
    assert!(
        cards[3].right() <= layer_panel_left,
        "4번째 선 카드 우측 끝({})이 도면 레이어 패널 좌측({})을 침범하면 안 됨",
        cards[3].right(),
        layer_panel_left
    );
}
