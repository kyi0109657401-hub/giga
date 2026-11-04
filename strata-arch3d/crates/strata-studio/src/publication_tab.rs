use eframe::egui;
use std::path::{Path, PathBuf};
use strata_core::strata_tokens::tokens::{ACTION, CLAY_WASH, EDGE, INK, MUTED, SURFACE, WASH};
use strata_core::ScalePreset;
use strata_export::{
    append_section_hatching, export_plate_to_dxf_r12,
    render_plate_to_illustrator_svg_with_rubbings_and_photos, render_plate_to_pdf_bytes,
    triangulate_polygon_ear_clipping, ComposedPlate2D, PaperSheetSize, PhotoSlot, PlatePathElement,
    PlateRasterKind, PlateRasterRegion, PlateTextCallout, RubbingPngSlot,
    AI_STANDARD_LINE_WEIGHTS_PT_MM,
};
use strata_measure::ArtifactMeasurementReport;

use crate::sheet_pane::{
    classify_plate_path_layer_idx, compute_sheet_layer_centers_mm, draw_dashed_polyline,
    rubbing_provenance_line, snap_to_standard_weight_mm, PaperSheetPreset, SheetPaneState,
    STANDARD_LINE_WEIGHTS_MM,
};

/// 탭 ② (`📄 ② 출판 조판 · 일러스트 내보내기`) 전용 상태.
#[derive(Debug, Clone)]
pub struct PublicationLayoutState {
    /// 출판 용지 규격 (기본값: `A4 세로 210×297mm · 유물 편집판.ai 표준`)
    pub paper_preset: PaperSheetPreset,
    /// 용지 안쪽 판면 프레임 여백 (`mm`, 기본 `14.0mm`)
    pub margin_mm: f64,
    /// 『유물 편집판.ai』 배경 1cm(`10mm`) 방안지 격자(`레이어 1`) 표시 및 내보내기 포함 여부
    pub show_1cm_grid: bool,
    /// 판면 외곽 프레임 테두리(`#141413` `0.3mm`) 표시 여부
    pub show_frame_border: bool,
    /// 출판 축척 (`1/1`, `1/2`, `1/3`, `1/4`, `1/5`, `1/6`, `1/10`, `2/1` 등)
    pub pub_scale: ScalePreset,
    /// 자동 맞춤이 완료된 슬롯 제목 (슬롯 전환 시 자동으로 용지 최적 축척 계산)
    pub auto_fitted_slot_title: String,
    /// 『유물제도선.ai』 상·하·좌·우 규격 연장선(`|`, `—`, `0.15pt`) 표시 여부
    pub show_extension_ticks: bool,
    /// 『유물 편집판.ai』 하단 실사 사진 2장 슬롯 배치 여부
    pub show_photo_slots: bool,
    /// 하단 실사 사진 슬롯 높이 (`mm`, 기본 `36.0mm`)
    pub photo_height_mm: f64,
    /// 사용자가 직접 지정한 사진 1·2 경로 (비어 있으면 `{유물명}_1.jpg`, `_2.jpg` 자동 탐색)
    pub custom_photo_paths: [Option<PathBuf>; 2],
    /// 조판 그룹별 사용자 드래그 오프셋 (`mm`):
    /// `0: 평면도`, `1: 입단면도`, `2: 정밀탁본`, `3: 수평횡단면`, `4: 축척막대`, `5: 실사사진`, `6: 유물번호·메모`
    pub group_offsets_mm: [[f64; 2]; 7],
    /// 현재 선택/드래그 중인 조판 그룹 인덱스
    pub active_group_idx: Option<usize>,
}

impl Default for PublicationLayoutState {
    fn default() -> Self {
        Self {
            paper_preset: PaperSheetPreset::A4Portrait,
            margin_mm: 14.0,
            show_1cm_grid: true,
            show_frame_border: true,
            pub_scale: ScalePreset::Half1To2,
            auto_fitted_slot_title: String::new(),
            show_extension_ticks: true,
            show_photo_slots: true,
            photo_height_mm: 36.0,
            custom_photo_paths: [None, None],
            group_offsets_mm: [[0.0; 2]; 7],
            active_group_idx: None,
        }
    }
}

/// 출판 조판 페이지 합성 결과 (`ComposedPlate2D` + 요소별 용지 `mm` 바운딩 박스 + 용지 수용 여부).
#[derive(Debug, Clone)]
pub struct PublicationPageComposition {
    /// 용지 크기(`canvas_width_mm = paper_w`, `canvas_height_mm = paper_h`)와 출판 축척(`pub_scale`)이 적용된 완전체 도판
    pub page_plate: ComposedPlate2D,
    /// 7대 조판 요소 그룹의 용지 위 바운딩 박스 `[min_x, min_y, max_x, max_y]` (`mm`, 하단 원점 기준)
    pub group_rects_mm: Vec<(&'static str, [f64; 4])>,
    /// 실사 사진 2장 슬롯의 용지 위 바운딩 박스 `[min_x, min_y, max_x, max_y]` (`mm`)
    pub photo_rects_mm: [[f64; 4]; 2],
    /// 발견된 실사 사진 경로 목록
    pub resolved_photo_paths: Vec<PathBuf>,
    /// 모든 조판 요소가 용지 가장자리로부터 최소 `8.0mm` 이상 안쪽에 수용되었는지 여부 (`L2`)
    pub fits_on_paper: bool,
    /// 용지 사방 최소 여백 (`mm`)
    pub min_paper_margin_mm: f64,
}

pub const PUB_SCALE_CHOICES: [ScalePreset; 8] = [
    ScalePreset::Micro2To1,
    ScalePreset::Full1To1,
    ScalePreset::Half1To2,
    ScalePreset::Third1To3,
    ScalePreset::Quarter1To4,
    ScalePreset::Fifth1To5,
    ScalePreset::Sixth1To6,
    ScalePreset::Tenth1To10,
];

/// 『동국문화재축척및방위표.ai』 정본 축척별 막대 사양: `(종이 위 물리 길이 mm, 중앙 라벨, 우측 끝 라벨)`.
pub fn publication_scale_bar_spec(scale: ScalePreset) -> (f64, &'static str, &'static str) {
    match scale {
        ScalePreset::Full1To1 => (50.0, "2.5", "5cm (1/1)"),
        ScalePreset::Half1To2 => (25.0, "2.5", "5cm (1/2)"),
        ScalePreset::Third1To3 => (100.0 / 3.0, "5", "10cm (1/3)"),
        ScalePreset::Quarter1To4 => (50.0, "10", "20cm (1/4)"),
        ScalePreset::Fifth1To5 => (40.0, "10", "20cm (1/5)"),
        ScalePreset::Sixth1To6 => (200.0 / 6.0, "10", "20cm (1/6)"),
        ScalePreset::Tenth1To10 => (40.0, "20", "40cm (1/10)"),
        ScalePreset::Micro2To1 => (40.0, "1", "2cm (2/1)"),
        ScalePreset::Micro3To1 => (30.0, "0.5", "1cm (3/1)"),
        ScalePreset::Micro4To1 => (40.0, "0.5", "1cm (4/1)"),
        ScalePreset::Micro5To1 => (50.0, "0.5", "1cm (5/1)"),
        ScalePreset::Micro10To1 => (50.0, "0.25", "0.5cm (10/1)"),
    }
}

/// 유물 치수와 용지 규격·사진 포함 여부에 맞춰 판면 안에 겹침 없이 들어가는 최적 출판 축척(`ScalePreset`)을 자동 선택한다.
pub fn auto_select_publication_scale(
    report: &ArtifactMeasurementReport,
    paper_preset: PaperSheetPreset,
    margin_mm: f64,
    show_photo_slots: bool,
    photo_height_mm: f64,
) -> ScalePreset {
    let (pw, ph) = paper_preset.dimensions_mm();
    let usable_w = (pw - margin_mm * 2.0 - 16.0).max(60.0);
    let usable_h = (ph
        - margin_mm * 2.0
        - 36.0
        - if show_photo_slots {
            photo_height_mm + 10.0
        } else {
            0.0
        })
    .max(60.0);

    let max_d = report
        .max_belly_diameter_mm
        .max(report.rim_or_outer_diameter_mm)
        .max(20.0);
    let h = report.height_or_length_mm.max(12.0);
    // 좌열(평면·단면) + 우열(탁본·횡단면) 필요 폭: 약 2.35 * max_d, 필요 높이: 약 max_d + h + 28mm
    let req_w_1to1 = max_d * 2.35 + 22.0;
    let req_h_1to1 = max_d + h + 28.0;

    let candidates = [
        ScalePreset::Full1To1,
        ScalePreset::Half1To2,
        ScalePreset::Third1To3,
        ScalePreset::Quarter1To4,
        ScalePreset::Fifth1To5,
        ScalePreset::Sixth1To6,
        ScalePreset::Tenth1To10,
    ];
    for cand in candidates {
        let s = cand.ratio();
        if req_w_1to1 * s <= usable_w && req_h_1to1 * s <= usable_h {
            return cand;
        }
    }
    ScalePreset::Tenth1To10
}

impl PublicationLayoutState {
    pub fn auto_fit_to_paper(&mut self, report: &ArtifactMeasurementReport, title: &str) {
        self.pub_scale = auto_select_publication_scale(
            report,
            self.paper_preset,
            self.margin_mm,
            self.show_photo_slots,
            self.photo_height_mm,
        );
        self.group_offsets_mm = [[0.0; 2]; 7];
        self.auto_fitted_slot_title = title.to_string();
    }
}

fn resolve_companion_photos(
    source_path: Option<&Path>,
    custom: &[Option<PathBuf>; 2],
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for slot in custom.iter().flatten() {
        if slot.is_file() {
            out.push(slot.clone());
        }
    }
    if out.len() < 2 {
        if let Some(src) = source_path {
            if let (Some(dir), Some(stem)) =
                (src.parent(), src.file_stem().and_then(|s| s.to_str()))
            {
                for suffix in ["_1", "_2"] {
                    for ext in ["jpg", "jpeg", "png", "webp"] {
                        let cand = dir.join(format!("{stem}{suffix}.{ext}"));
                        if cand.is_file() && !out.contains(&cand) {
                            out.push(cand);
                            break;
                        }
                    }
                }
            }
        }
    }
    out
}

/// 원본 1:1 실측 도판(`raw_plate`)을 선택한 출판 용지(`paper_preset`)와 출판 축척(`pub_scale`)에 맞춰
/// 『유물 편집판.ai / 유물제도선.ai / 동국문화재축척및방위표.ai』 규격의 단일 출판 페이지(`PublicationPageComposition`)로 조판한다.
pub fn compose_publication_page_plate(
    raw_plate: &ComposedPlate2D,
    report: &ArtifactMeasurementReport,
    source_path: Option<&Path>,
    sheet_state: &SheetPaneState,
    pub_state: &PublicationLayoutState,
) -> PublicationPageComposition {
    let (paper_w, paper_h) = pub_state.paper_preset.dimensions_mm();
    let m = pub_state.margin_mm.clamp(8.0, 30.0);
    let s = pub_state.pub_scale.ratio();

    let inner_x0 = m;
    let inner_x1 = paper_w - m;
    let inner_y0 = m;
    let inner_y1 = paper_h - m;
    let inner_w = (inner_x1 - inner_x0).max(60.0);

    // 1. 최하단(Band 4): 유물번호 & 확인 메모
    let num_off = pub_state.group_offsets_mm[6];
    let cx_page = paper_w * 0.5;
    let memo_y = inner_y0 + 3.2 + num_off[1];
    let num_y = inner_y0 + 8.6 + num_off[1];

    // 2. 하단(Band 3): 실사 사진 2장 슬롯
    let photo_off = pub_state.group_offsets_mm[5];
    let photo_h = if pub_state.show_photo_slots {
        pub_state.photo_height_mm.clamp(20.0, 62.0)
    } else {
        0.0
    };
    let photo_y0 = inner_y0 + 14.5 + photo_off[1];
    let photo_y1 = photo_y0 + photo_h;
    let photo_w = ((inner_w - 16.0) * 0.5).clamp(32.0, 110.0);
    let photo_rects_mm = [
        [
            cx_page - 4.0 - photo_w + photo_off[0],
            photo_y0,
            cx_page - 4.0 + photo_off[0],
            photo_y1,
        ],
        [
            cx_page + 4.0 + photo_off[0],
            photo_y0,
            cx_page + 4.0 + photo_w + photo_off[0],
            photo_y1,
        ],
    ];

    // 3. 중단(Band 2): 동국문화재축척및방위표.ai 규격 축척 막대
    let scale_off = pub_state.group_offsets_mm[4];
    let scale_base_y = (if pub_state.show_photo_slots {
        photo_y1 + 6.5
    } else {
        inner_y0 + 15.5
    }) + scale_off[1];
    let (bar_len_mm, mid_lbl, end_lbl) = publication_scale_bar_spec(pub_state.pub_scale);
    let bar_cx = cx_page + scale_off[0];
    let bar_x0 = bar_cx - bar_len_mm * 0.5;
    let bar_x1 = bar_cx + bar_len_mm * 0.5;

    // 4. 상단(Band 1): 실측 도면 4대 뷰 (① 평면도, ② 입단면도, ③ 정밀탁본, ④ 수평횡단면)
    let draw_y0 = scale_base_y + 11.5;
    let draw_y1 = inner_y1 - 5.0;
    let draw_h = (draw_y1 - draw_y0).max(50.0);

    let max_d_scaled = (report
        .max_belly_diameter_mm
        .max(report.rim_or_outer_diameter_mm)
        .max(20.0)
        * s)
        .min(inner_w * 0.42);
    let half_d_s = max_d_scaled * 0.5;
    let h_scaled = (report.height_or_length_mm.max(12.0) * s).min(draw_h * 0.42);

    // 정투영 중심축 스냅 잠금(Snap Lock): 평면도(group 0)의 X축은 입단면도(group 1)의 X축과 항상 일치!
    let col_left_x = inner_x0 + inner_w * 0.27 + pub_state.group_offsets_mm[1][0];
    let col_right_x = inner_x0 + inner_w * 0.74 + pub_state.group_offsets_mm[2][0];

    let elev_cy = draw_y0 + h_scaled * 0.5 + 8.0 + pub_state.group_offsets_mm[1][1];
    let plan_cy = (draw_y1 - half_d_s - 7.5 + pub_state.group_offsets_mm[0][1])
        .max(elev_cy + h_scaled * 0.5 + half_d_s + 14.0);
    let rub_cy =
        (draw_y1 - half_d_s - 7.5 + pub_state.group_offsets_mm[2][1]).max(draw_y0 + draw_h * 0.62);
    let hsec_cy = (draw_y0 + half_d_s * 0.72 + 8.0 + pub_state.group_offsets_mm[3][1])
        .min(rub_cy - half_d_s * 1.65 - 10.0);

    let raw_centers = compute_sheet_layer_centers_mm(raw_plate);
    let target_centers = [
        [col_left_x, plan_cy],
        [col_left_x, elev_cy],
        [col_right_x, rub_cy],
        [col_right_x + pub_state.group_offsets_mm[3][0], hsec_cy],
        [col_right_x, elev_cy],
        [col_left_x, elev_cy],
    ];

    let rgb_hex =
        |rgb: [u8; 3]| -> String { format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]) };
    let sec_hex = rgb_hex(sheet_state.line_style.section_color_rgb);
    let out_hex = rgb_hex(sheet_state.line_style.outer_color_rgb);
    let aux_hex = rgb_hex(sheet_state.line_style.aux_color_rgb);
    let dim_hex = rgb_hex(sheet_state.line_style.dim_color_rgb);

    let mut paths = Vec::new();
    let mut callouts = Vec::new();

    // 판면 외곽 테두리
    if pub_state.show_frame_border {
        paths.push(PlatePathElement {
            layer_id: String::from("판면_테두리"),
            points_mm: vec![
                [inner_x0, inner_y0],
                [inner_x1, inner_y0],
                [inner_x1, inner_y1],
                [inner_x0, inner_y1],
            ],
            closed: true,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.25,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }

    for elem in &raw_plate.paths {
        // 기존 raw_plate의 축척 막대/해칭/연장선은 출판 페이지에서 출판 축척 기준으로 새로 생성한다.
        if elem.layer_id.starts_with("치수_축척")
            || elem.layer_id == "축척자_5cm"
            || elem.layer_id == "단면_해칭"
            || elem.layer_id == "연장선_규격"
        {
            continue;
        }
        let v_idx = classify_plate_path_layer_idx(&elem.layer_id);
        let vtf = sheet_state.view_transforms[v_idx];
        if !vtf.visible {
            continue;
        }
        let src_c = raw_centers[v_idx];
        let dst_c = target_centers[v_idx];
        let mapped_pts: Vec<[f64; 2]> = elem
            .points_mm
            .iter()
            .map(|&pt| {
                let p_rot = vtf.transform_point_around(pt, src_c);
                [
                    dst_c[0] + (p_rot[0] - src_c[0]) * s,
                    dst_c[1] + (p_rot[1] - src_c[1]) * s,
                ]
            })
            .collect();

        let mut ne = elem.clone();
        ne.points_mm = mapped_pts;
        if elem.layer_id.contains("단면_실선") {
            ne.stroke_width_mm = f64::from(snap_to_standard_weight_mm(
                sheet_state.line_style.section_width_mm,
            ));
            ne.stroke_hex.clone_from(&sec_hex);
            ne.fill_hex = if sheet_state.line_style.fill_section_solid {
                Some(sec_hex.clone())
            } else {
                None
            };
        } else if elem.layer_id.contains("외곽선")
            || elem.layer_id.contains("구경")
            || elem.layer_id.contains("저경")
        {
            ne.stroke_width_mm = f64::from(snap_to_standard_weight_mm(
                sheet_state.line_style.outer_width_mm,
            ));
            ne.stroke_hex.clone_from(&out_hex);
        } else if elem.layer_id.contains("횡단면")
            || elem.layer_id.contains("중심")
            || elem.layer_id.contains("십자선")
        {
            ne.stroke_width_mm = f64::from(snap_to_standard_weight_mm(
                sheet_state.line_style.aux_width_mm,
            ));
            ne.stroke_hex.clone_from(&aux_hex);
            if elem.layer_id.contains("횡단면_단면") {
                ne.fill_hex = if sheet_state.line_style.fill_section_solid {
                    Some(sec_hex.clone())
                } else {
                    None
                };
            }
        }
        paths.push(ne);
    }

    // 유물제도선.ai 상·하·좌·우 연장선(|, —, 0.15pt = 0.053mm) 생성
    let mut elev_min_x = f64::INFINITY;
    let mut elev_max_x = f64::NEG_INFINITY;
    let mut elev_min_y = f64::INFINITY;
    let mut elev_max_y = f64::NEG_INFINITY;
    for p in paths
        .iter()
        .filter(|p| p.layer_id == "단면_실선" || p.layer_id == "외곽선_실선")
        .flat_map(|p| p.points_mm.iter())
    {
        elev_min_x = elev_min_x.min(p[0]);
        elev_max_x = elev_max_x.max(p[0]);
        elev_min_y = elev_min_y.min(p[1]);
        elev_max_y = elev_max_y.max(p[1]);
    }
    if pub_state.show_extension_ticks
        && elev_min_x.is_finite()
        && elev_max_x.is_finite()
        && elev_min_y.is_finite()
        && elev_max_y.is_finite()
    {
        let gap = 1.4_f64;
        let len = 2.8_f64;
        let ticks = [
            [
                [elev_min_x - gap - len, elev_max_y],
                [elev_min_x - gap, elev_max_y],
            ],
            [
                [elev_max_x + gap, elev_max_y],
                [elev_max_x + gap + len, elev_max_y],
            ],
            [
                [elev_min_x - gap - len, elev_min_y],
                [elev_min_x - gap, elev_min_y],
            ],
            [
                [elev_max_x + gap, elev_min_y],
                [elev_max_x + gap + len, elev_min_y],
            ],
            [
                [col_left_x, elev_max_y + gap],
                [col_left_x, elev_max_y + gap + len],
            ],
            [
                [col_left_x, elev_min_y - gap - len],
                [col_left_x, elev_min_y - gap],
            ],
        ];
        for seg in ticks {
            paths.push(PlatePathElement {
                layer_id: String::from("연장선_규격"),
                points_mm: vec![seg[0], seg[1]],
                closed: false,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.053,
                dash_pattern_mm: None,
                fill_hex: None,
            });
        }
    }

    // 동국문화재축척및방위표.ai 정본 축척 막대 생성 (라벨 < 눈금 < 기선 < 바디 순서 보장: L4 & L5)
    let body_h = 0.9_f64;
    let body_gap = 0.6_f64;
    let baseline_y = scale_base_y + body_h + body_gap;
    paths.push(PlatePathElement {
        layer_id: String::from("치수_축척바디"),
        points_mm: vec![
            [bar_x0, scale_base_y],
            [bar_x1, scale_base_y],
            [bar_x1, scale_base_y + body_h],
            [bar_x0, scale_base_y + body_h],
        ],
        closed: true,
        stroke_hex: String::from("#141413"),
        stroke_width_mm: 0.0,
        dash_pattern_mm: None,
        fill_hex: Some(String::from("#141413")),
    });
    paths.push(PlatePathElement {
        layer_id: String::from("치수_축척막대"),
        points_mm: vec![[bar_x0, baseline_y], [bar_x1, baseline_y]],
        closed: false,
        stroke_hex: String::from("#141413"),
        stroke_width_mm: 0.071,
        dash_pattern_mm: None,
        fill_hex: None,
    });
    for i in 0..=10 {
        let tx = bar_x0 + bar_len_mm * (i as f64 / 10.0);
        let th = if i == 0 || i == 10 {
            2.1
        } else if i == 5 {
            1.6
        } else {
            1.0
        };
        paths.push(PlatePathElement {
            layer_id: String::from("치수_축척눈금"),
            points_mm: vec![[tx, baseline_y], [tx, baseline_y + th]],
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.071,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }
    let lbl_y = baseline_y + 3.4;
    callouts.push(PlateTextCallout {
        layer_id: String::from("치수_축척라벨"),
        pos_mm: [bar_x0 - 0.8, lbl_y],
        text: String::from("0"),
        font_size_pt: 7.5,
        color_hex: String::from("#141413"),
    });
    callouts.push(PlateTextCallout {
        layer_id: String::from("치수_축척라벨"),
        pos_mm: [bar_cx - 2.2, lbl_y],
        text: mid_lbl.to_string(),
        font_size_pt: 7.5,
        color_hex: String::from("#141413"),
    });
    callouts.push(PlateTextCallout {
        layer_id: String::from("치수_축척라벨"),
        pos_mm: [bar_x1 - 2.0, lbl_y],
        text: end_lbl.to_string(),
        font_size_pt: 7.5,
        color_hex: String::from("#141413"),
    });

    // 뷰 캡션 및 유물번호·메모 배치
    let sc_lbl = pub_state.pub_scale.label();
    callouts.push(PlateTextCallout {
        layer_id: String::from("평면도_라벨"),
        pos_mm: [col_left_x - 14.0, plan_cy + half_d_s + 2.6],
        text: format!("{sc_lbl} 평면도"),
        font_size_pt: 8.0,
        color_hex: dim_hex.clone(),
    });
    callouts.push(PlateTextCallout {
        layer_id: String::from("탁본_라벨"),
        pos_mm: [col_right_x - 14.0, rub_cy + half_d_s + 2.6],
        text: format!("{sc_lbl} 정밀 탁본"),
        font_size_pt: 8.0,
        color_hex: dim_hex.clone(),
    });
    callouts.push(PlateTextCallout {
        layer_id: String::from("횡단면_라벨"),
        pos_mm: [
            col_right_x - 14.0 + pub_state.group_offsets_mm[3][0],
            hsec_cy + half_d_s * 0.78 + 2.4,
        ],
        text: format!("{sc_lbl} 수평 횡단면"),
        font_size_pt: 8.0,
        color_hex: String::from("#B5573A"),
    });

    let default_num = source_path
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or(&raw_plate.title);
    let num_text = if sheet_state.custom_artifact_number.trim().is_empty() {
        default_num.to_string()
    } else {
        sheet_state.custom_artifact_number.trim().to_string()
    };
    callouts.push(PlateTextCallout {
        layer_id: String::from("유물번호"),
        pos_mm: [
            cx_page - (num_text.chars().count() as f64) * 1.1 + num_off[0],
            num_y,
        ],
        text: num_text,
        font_size_pt: 9.5,
        color_hex: String::from("#141413"),
    });
    if !sheet_state.custom_artifact_memo.trim().is_empty() {
        let memo = sheet_state.custom_artifact_memo.trim().to_string();
        callouts.push(PlateTextCallout {
            layer_id: String::from("유물캡션"),
            pos_mm: [
                cx_page - (memo.chars().count() as f64) * 0.9 + num_off[0],
                memo_y,
            ],
            text: memo,
            font_size_pt: 8.0,
            color_hex: dim_hex,
        });
    }

    // 정밀 탁본 래스터 영역 (출판 축척 적용)
    let raster_regions = vec![PlateRasterRegion {
        kind: PlateRasterKind::BottomPlanRubbing,
        rect_mm: [
            col_right_x - half_d_s,
            rub_cy - half_d_s,
            col_right_x + half_d_s,
            rub_cy + half_d_s,
        ],
        clip_half_sign: None,
        caption: String::from("정밀 탁본"),
    }];

    let mut page_plate = ComposedPlate2D {
        mode: raw_plate.mode,
        title: raw_plate.title.clone(),
        scale_preset: pub_state.pub_scale,
        canvas_width_mm: paper_w,
        canvas_height_mm: paper_h,
        flip_exterior_right: raw_plate.flip_exterior_right,
        paths,
        callouts,
        raster_regions,
    };

    if sheet_state.line_style.show_hatch_lines && !sheet_state.line_style.fill_section_solid {
        append_section_hatching(&mut page_plate, 1.0);
    }

    // 7대 조판 요소 그룹의 바운딩 박스 산출 (L2 여백 및 L3 충돌 감사용)
    let mut group_rects_mm: Vec<(&'static str, [f64; 4])> = vec![
        (
            "① 평면도",
            [
                col_left_x - half_d_s,
                plan_cy - half_d_s,
                col_left_x + half_d_s,
                plan_cy + half_d_s + 4.0,
            ],
        ),
        (
            "② 단면·입면도",
            [
                col_left_x - half_d_s,
                elev_cy - h_scaled * 0.5,
                col_left_x + half_d_s,
                elev_cy + h_scaled * 0.5,
            ],
        ),
        (
            "③ 정밀 탁본",
            [
                col_right_x - half_d_s,
                rub_cy - half_d_s,
                col_right_x + half_d_s,
                rub_cy + half_d_s + 4.0,
            ],
        ),
        (
            "④ 수평 횡단면",
            [
                col_right_x - half_d_s * 0.78 + pub_state.group_offsets_mm[3][0],
                hsec_cy - half_d_s * 0.78,
                col_right_x + half_d_s * 0.78 + pub_state.group_offsets_mm[3][0],
                hsec_cy + half_d_s * 0.78 + 3.5,
            ],
        ),
        (
            "⑤ 축척 막대",
            [bar_x0 - 2.0, scale_base_y, bar_x1 + 8.0, lbl_y + 2.5],
        ),
        (
            "⑦ 유물번호·메모",
            [
                cx_page - 24.0 + num_off[0],
                memo_y - 1.5,
                cx_page + 24.0 + num_off[0],
                num_y + 3.0,
            ],
        ),
    ];
    if pub_state.show_photo_slots {
        group_rects_mm.push(("⑥ 실사 사진 1", photo_rects_mm[0]));
        group_rects_mm.push(("⑥ 실사 사진 2", photo_rects_mm[1]));
    }

    let mut min_margin = f64::INFINITY;
    for &(_, [x0, y0, x1, y1]) in &group_rects_mm {
        let m_left = x0.min(x1);
        let m_right = paper_w - x0.max(x1);
        let m_bot = y0.min(y1);
        let m_top = paper_h - y0.max(y1);
        min_margin = min_margin.min(m_left).min(m_right).min(m_bot).min(m_top);
    }
    let fits_on_paper = min_margin >= 8.0;
    let resolved_photo_paths = resolve_companion_photos(source_path, &pub_state.custom_photo_paths);

    PublicationPageComposition {
        page_plate,
        group_rects_mm,
        photo_rects_mm,
        resolved_photo_paths,
        fits_on_paper,
        min_paper_margin_mm: if min_margin.is_finite() {
            min_margin
        } else {
            m
        },
    }
}

/// 출판 조판 페이지(`PublicationPageComposition`)를 Adobe Illustrator 호환 SVG 문자열로 직렬화한다.
pub fn export_publication_page_to_svg(
    comp: &PublicationPageComposition,
    sheet_state: &SheetPaneState,
    show_grid: bool,
) -> String {
    let provenance = rubbing_provenance_line(&sheet_state.rubbing_params);
    let mut photo_slots = Vec::new();
    for (i, path) in comp.resolved_photo_paths.iter().take(2).enumerate() {
        photo_slots.push(PhotoSlot {
            rect_mm: comp.photo_rects_mm[i],
            path: path.as_path(),
        });
    }
    let mut rubbing_slots = Vec::new();
    if let Some(reg) = comp.page_plate.raster_regions.first() {
        let png_bytes = sheet_state
            .cached_bottom_img
            .as_ref()
            .or(sheet_state.cached_front_img.as_ref())
            .and_then(|img| {
                let mut buf = std::io::Cursor::new(Vec::new());
                image::DynamicImage::ImageRgba8(img.clone())
                    .write_to(&mut buf, image::ImageFormat::Png)
                    .ok()
                    .map(|()| buf.into_inner())
            })
            .unwrap_or_else(|| {
                vec![
                    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
                ]
            });
        rubbing_slots.push(RubbingPngSlot {
            id: String::from("정밀탁본_래스터"),
            rect_mm: reg.rect_mm,
            png_bytes,
        });
    }
    render_plate_to_illustrator_svg_with_rubbings_and_photos(
        &comp.page_plate,
        &provenance,
        show_grid,
        &photo_slots,
        &rubbing_slots,
    )
}

/// 탭 ② (`📄 ② 출판 조판 · 일러스트 내보내기`) 전체 화면 렌더링 (`좌측 74% 출판 용지 아트보드` + `우측 26% 조판·축척·내보내기 제어 독`).
#[allow(clippy::too_many_arguments)]
pub fn render_publication_tab_egui(
    ui: &mut egui::Ui,
    raw_plate: &ComposedPlate2D,
    report: &ArtifactMeasurementReport,
    mesh: &strata_core::TriMesh,
    source_path: Option<&Path>,
    sheet_state: &mut SheetPaneState,
    pub_state: &mut PublicationLayoutState,
    status_message: &mut String,
) -> Option<&'static str> {
    let mut requested_action = None;
    if pub_state.auto_fitted_slot_title != raw_plate.title {
        pub_state.auto_fit_to_paper(report, &raw_plate.title);
    }

    // 정밀 탁본 텍스처가 아직 준비되지 않았다면 백그라운드 캐시 생성
    if sheet_state.cached_bottom_tex.is_none() {
        let bottom_img = crate::sheet_pane::render_orthographic_rubbing_rgba_with_params(
            mesh,
            PlateRasterKind::BottomPlanRubbing,
            0.0,
            &sheet_state.rubbing_params,
            480,
            480,
        );
        let cimg = egui::ColorImage::from_rgba_unmultiplied(
            [bottom_img.width() as usize, bottom_img.height() as usize],
            bottom_img.as_raw(),
        );
        sheet_state.cached_bottom_tex = Some(ui.ctx().load_texture(
            "pub_tab_rubbing_bottom",
            cimg,
            egui::TextureOptions::LINEAR,
        ));
        sheet_state.cached_bottom_img = Some(bottom_img);
    }

    let comp =
        compose_publication_page_plate(raw_plate, report, source_path, sheet_state, pub_state);

    let full_rect = ui.available_rect_before_wrap();
    let dock_w = 336.0_f32.min(full_rect.width() * 0.30);
    let artboard_rect = egui::Rect::from_min_max(
        full_rect.min,
        egui::pos2(
            (full_rect.right() - dock_w - 1.0).max(full_rect.left() + 200.0),
            full_rect.bottom(),
        ),
    );
    let dock_rect = egui::Rect::from_min_max(
        egui::pos2(artboard_rect.right() + 1.0, full_rect.top()),
        full_rect.max,
    );

    let ink = egui::Color32::from_rgb(INK.rgb[0], INK.rgb[1], INK.rgb[2]);
    let muted = egui::Color32::from_rgb(MUTED.rgb[0], MUTED.rgb[1], MUTED.rgb[2]);
    let terra = egui::Color32::from_rgb(ACTION.rgb[0], ACTION.rgb[1], ACTION.rgb[2]);
    let surface = egui::Color32::from_rgb(SURFACE.rgb[0], SURFACE.rgb[1], SURFACE.rgb[2]);
    let wash = egui::Color32::from_rgb(WASH.rgb[0], WASH.rgb[1], WASH.rgb[2]);
    let clay_wash = egui::Color32::from_rgb(CLAY_WASH.rgb[0], CLAY_WASH.rgb[1], CLAY_WASH.rgb[2]);
    let border = egui::Color32::from_rgb(EDGE.rgb[0], EDGE.rgb[1], EDGE.rgb[2]);

    // =========================================================================
    // 1. 좌측 74%: WYSIWYG 출판 용지 아트보드 캔버스
    // =========================================================================
    let painter = ui.painter_at(artboard_rect);
    painter.rect_filled(artboard_rect, 0.0, egui::Color32::from_rgb(228, 225, 216));

    let (pw_mm, ph_mm) = pub_state.paper_preset.dimensions_mm();
    let avail_w = (artboard_rect.width() - 48.0).max(120.0);
    let avail_h = (artboard_rect.height() - 56.0).max(120.0);
    let px_per_mm = (avail_w / (pw_mm as f32))
        .min(avail_h / (ph_mm as f32))
        .clamp(0.8, 8.0);
    let paper_w_px = (pw_mm as f32) * px_per_mm;
    let paper_h_px = (ph_mm as f32) * px_per_mm;
    let paper_rect = egui::Rect::from_center_size(
        egui::pos2(artboard_rect.center().x, artboard_rect.center().y + 8.0),
        egui::vec2(paper_w_px, paper_h_px),
    );

    // 용지 그림자 + 순백색 종이(#FFFFFF)
    painter.rect_filled(
        paper_rect.translate(egui::vec2(4.0, 5.0)),
        2.0,
        egui::Color32::from_rgba_unmultiplied(20, 20, 19, 38),
    );
    painter.rect_filled(paper_rect, 1.5, egui::Color32::WHITE);
    painter.rect_stroke(
        paper_rect,
        1.5,
        egui::Stroke::new(1.2, egui::Color32::from_rgb(150, 145, 136)),
        egui::StrokeKind::Outside,
    );

    let to_screen = |pt_mm: [f64; 2]| -> egui::Pos2 {
        egui::pos2(
            paper_rect.left() + (pt_mm[0] as f32) * px_per_mm,
            paper_rect.bottom() - (pt_mm[1] as f32) * px_per_mm,
        )
    };

    // 1cm(10mm) 및 5mm 방안지 격자 (레이어 1)
    if pub_state.show_1cm_grid {
        let mut x_mm = 5.0_f64;
        while x_mm < pw_mm {
            let sx = paper_rect.left() + (x_mm as f32) * px_per_mm;
            let is_cm = ((x_mm.round() as i32) % 10) == 0;
            painter.line_segment(
                [
                    egui::pos2(sx, paper_rect.top()),
                    egui::pos2(sx, paper_rect.bottom()),
                ],
                egui::Stroke::new(
                    if is_cm { 0.7 } else { 0.4 },
                    if is_cm {
                        egui::Color32::from_rgb(214, 208, 194)
                    } else {
                        egui::Color32::from_rgb(236, 232, 222)
                    },
                ),
            );
            x_mm += 5.0;
        }
        let mut y_mm = 5.0_f64;
        while y_mm < ph_mm {
            let sy = paper_rect.bottom() - (y_mm as f32) * px_per_mm;
            let is_cm = ((y_mm.round() as i32) % 10) == 0;
            painter.line_segment(
                [
                    egui::pos2(paper_rect.left(), sy),
                    egui::pos2(paper_rect.right(), sy),
                ],
                egui::Stroke::new(
                    if is_cm { 0.7 } else { 0.4 },
                    if is_cm {
                        egui::Color32::from_rgb(214, 208, 194)
                    } else {
                        egui::Color32::from_rgb(236, 232, 222)
                    },
                ),
            );
            y_mm += 5.0;
        }
    }

    // 정밀 탁본 래스터 렌더링
    if let (Some(reg), Some(tex)) = (
        comp.page_plate.raster_regions.first(),
        sheet_state
            .cached_bottom_tex
            .as_ref()
            .or(sheet_state.cached_front_tex.as_ref()),
    ) {
        let p0 = to_screen([reg.rect_mm[0], reg.rect_mm[3]]);
        let p1 = to_screen([reg.rect_mm[2], reg.rect_mm[1]]);
        let r_rect = egui::Rect::from_two_pos(p0, p1);
        painter.image(
            tex.id(),
            r_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    // 하단 실사 사진 2장 슬롯 렌더링
    if pub_state.show_photo_slots {
        for (i, rect_mm) in comp.photo_rects_mm.iter().enumerate() {
            let p0 = to_screen([rect_mm[0], rect_mm[3]]);
            let p1 = to_screen([rect_mm[2], rect_mm[1]]);
            let prect = egui::Rect::from_two_pos(p0, p1);
            painter.rect_filled(prect, 2.0, egui::Color32::from_rgb(246, 244, 238));
            painter.rect_stroke(
                prect,
                2.0,
                egui::Stroke::new(0.9, egui::Color32::from_rgb(175, 170, 160)),
                egui::StrokeKind::Inside,
            );
            let photo_lbl = if let Some(p) = comp.resolved_photo_paths.get(i) {
                format!(
                    "📷 실사 사진 {} ({})",
                    i + 1,
                    p.file_name().and_then(|s| s.to_str()).unwrap_or("자동연동")
                )
            } else {
                format!("📷 유물 실사 사진 슬롯 {} (유물 편집판.ai)", i + 1)
            };
            painter.text(
                prect.center(),
                egui::Align2::CENTER_CENTER,
                photo_lbl,
                egui::FontId::proportional(9.5),
                muted,
            );
        }
    }

    // 모든 벡터 경로(평면·단면·해칭·연장선·축척자) 렌더링
    for elem in &comp.page_plate.paths {
        if elem.points_mm.len() < 2 {
            continue;
        }
        let pts_px: Vec<egui::Pos2> = elem.points_mm.iter().map(|&pt| to_screen(pt)).collect();
        if elem.closed {
            if let Some(ref fill_hex) = elem.fill_hex {
                let fc = if fill_hex == "#141413" {
                    ink
                } else {
                    egui::Color32::from_rgb(20, 20, 19)
                };
                for tri in triangulate_polygon_ear_clipping(&elem.points_mm) {
                    if tri[0] < pts_px.len() && tri[1] < pts_px.len() && tri[2] < pts_px.len() {
                        painter.add(egui::Shape::convex_polygon(
                            vec![pts_px[tri[0]], pts_px[tri[1]], pts_px[tri[2]]],
                            fc,
                            egui::Stroke::NONE,
                        ));
                    }
                }
            }
        }
        if elem.stroke_width_mm > 0.0 {
            let w_px = ((elem.stroke_width_mm as f32) * px_per_mm).clamp(0.7, 3.5);
            let sc = if elem.stroke_hex == "#D92B2B" {
                terra
            } else if elem.stroke_hex == "#5E5D59" {
                muted
            } else {
                ink
            };
            if let Some((on_mm, off_mm)) = elem.dash_pattern_mm {
                draw_dashed_polyline(
                    &painter,
                    &pts_px,
                    w_px,
                    sc,
                    ((on_mm as f32) * px_per_mm).max(3.0),
                    ((off_mm as f32) * px_per_mm).max(2.0),
                );
            } else {
                for seg in pts_px.windows(2) {
                    painter.line_segment([seg[0], seg[1]], egui::Stroke::new(w_px, sc));
                }
                if elem.closed && pts_px.len() >= 3 {
                    if let (Some(&f), Some(&l)) = (pts_px.first(), pts_px.last()) {
                        painter.line_segment([l, f], egui::Stroke::new(w_px, sc));
                    }
                }
            }
        }
    }

    // 텍스트 콜아웃(축척자 라벨, 뷰 캡션, 유물번호, 확인 메모) 렌더링
    for c in &comp.page_plate.callouts {
        let pos = to_screen(c.pos_mm);
        let col = if c.color_hex == "#B5573A" { terra } else { ink };
        painter.text(
            pos,
            egui::Align2::LEFT_BOTTOM,
            &c.text,
            egui::FontId::proportional((c.font_size_pt as f32).clamp(8.0, 13.0)),
            col,
        );
    }

    // 상단 아트보드 상태 배지 (용지 규격 · 축척 · 판면 수용 상태)
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(artboard_rect.left() + 14.0, artboard_rect.top() + 8.0),
        egui::vec2((artboard_rect.width() - 28.0).min(640.0), 24.0),
    );
    painter.rect_filled(
        badge_rect,
        4.0,
        if comp.fits_on_paper {
            egui::Color32::from_rgba_unmultiplied(250, 249, 245, 238)
        } else {
            egui::Color32::from_rgba_unmultiplied(254, 235, 232, 245)
        },
    );
    painter.rect_stroke(
        badge_rect,
        4.0,
        egui::Stroke::new(1.0, if comp.fits_on_paper { border } else { terra }),
        egui::StrokeKind::Inside,
    );
    let badge_msg = if comp.fits_on_paper {
        format!(
            "✓ 출판 조판 아트보드: {} ({:.0}×{:.0}mm) · 출판 축척 {} · 최소 여백 {:.1}mm (일러스트 내보내기 준비 완료)",
            pub_state.paper_preset.label_kr(),
            pw_mm,
            ph_mm,
            pub_state.pub_scale.label(),
            comp.min_paper_margin_mm
        )
    } else {
        format!(
            "▲ 용지 판면 초과 경고 (여백 {:.1}mm) — 우측 [⚡ 용지에 맞게 최적 축척·위치 자동 정렬]을 누르세요",
            comp.min_paper_margin_mm
        )
    };
    painter.text(
        egui::pos2(badge_rect.left() + 10.0, badge_rect.center().y),
        egui::Align2::LEFT_CENTER,
        badge_msg,
        egui::FontId::proportional(10.5),
        if comp.fits_on_paper { ink } else { terra },
    );

    // =========================================================================
    // 2. 우측 26% (336px): 출판 조판 · 축척 · 일러스트 내보내기 제어 독
    // =========================================================================
    painter.rect_filled(dock_rect, 0.0, surface);
    painter.line_segment(
        [dock_rect.left_top(), dock_rect.left_bottom()],
        egui::Stroke::new(1.0, border),
    );

    let mut dock_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(dock_rect.shrink2(egui::vec2(10.0, 8.0)))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    egui::ScrollArea::vertical()
        .id_salt("pub_tab_right_dock")
        .show(&mut dock_ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("📄 출판 조판 · 내보내기 스튜디오")
                        .size(12.5)
                        .strong()
                        .color(ink),
                );
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("← ① 1:1 실측 작업대")
                                .size(10.0)
                                .color(terra),
                        )
                        .fill(clay_wash)
                        .stroke(egui::Stroke::new(1.0, terra))
                        .corner_radius(3.5),
                    )
                    .clicked()
                {
                    requested_action = Some("GOTO_TAB1");
                }
            });
            ui.separator();

            // 섹션 1: 용지 규격 & 판면 설정
            ui.label(
                egui::RichText::new("1. 출판 용지 규격 (『유물 편집판.ai』 표준)")
                    .size(10.8)
                    .strong()
                    .color(ink),
            );
            ui.horizontal_wrapped(|ui| {
                for preset in [
                    PaperSheetPreset::A4Portrait,
                    PaperSheetPreset::A4Landscape,
                    PaperSheetPreset::A3Landscape,
                    PaperSheetPreset::B4Portrait,
                    PaperSheetPreset::B5Portrait,
                ] {
                    let sel = pub_state.paper_preset == preset;
                    let (w, h) = preset.dimensions_mm();
                    let lbl = format!("{} ({:.0}×{:.0})", preset.label_kr(), w, h);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(lbl).size(9.8).color(if sel {
                                egui::Color32::WHITE
                            } else {
                                ink
                            }))
                            .fill(if sel { ink } else { wash })
                            .corner_radius(3.0),
                        )
                        .clicked()
                    {
                        pub_state.paper_preset = preset;
                        pub_state.auto_fit_to_paper(report, &raw_plate.title);
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("판면 여백(mm):").size(9.8).color(muted));
                ui.add(
                    egui::DragValue::new(&mut pub_state.margin_mm)
                        .speed(0.5)
                        .range(10.0..=25.0)
                        .fixed_decimals(1),
                );
                ui.checkbox(&mut pub_state.show_1cm_grid, "1cm 방안격자");
                ui.checkbox(&mut pub_state.show_frame_border, "판면테두리");
            });
            ui.separator();

            // 섹션 2: 출판 축척 & 용지 자동 맞춤
            ui.label(
                egui::RichText::new("2. 출판 축척(Scale) & 용지 자동 맞춤")
                    .size(10.8)
                    .strong()
                    .color(ink),
            );
            ui.horizontal_wrapped(|ui| {
                for sc in PUB_SCALE_CHOICES {
                    let sel = pub_state.pub_scale == sc;
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(sc.label())
                                    .size(10.2)
                                    .strong()
                                    .color(if sel { egui::Color32::WHITE } else { ink }),
                            )
                            .fill(if sel { terra } else { wash })
                            .corner_radius(3.0),
                        )
                        .clicked()
                    {
                        pub_state.pub_scale = sc;
                    }
                }
            });
            if ui
                .add_sized(
                    [ui.available_width(), 24.0],
                    egui::Button::new(
                        egui::RichText::new("⚡ 용지에 맞게 최적 축척·위치 자동 정렬")
                            .size(10.5)
                            .strong()
                            .color(terra),
                    )
                    .fill(clay_wash)
                    .stroke(egui::Stroke::new(1.0, terra))
                    .corner_radius(4.0),
                )
                .clicked()
            {
                pub_state.auto_fit_to_paper(report, &raw_plate.title);
                *status_message = format!(
                    "용지({}) 최적 축척({}) 자동 조판 완료",
                    pub_state.paper_preset.label_kr(),
                    pub_state.pub_scale.label()
                );
            }
            ui.separator();

            // 섹션 3: 유물제도선.ai 7단계 표준 선굵기 & 단면 채움
            ui.label(
                egui::RichText::new("3. 『유물제도선.ai』 7단계 표준 선굵기 & 단면 표현")
                    .size(10.8)
                    .strong()
                    .color(ink),
            );
            if ui
                .add_sized(
                    [ui.available_width(), 22.0],
                    egui::Button::new(
                        egui::RichText::new(
                            "✓ 유물제도선.ai 표준 선굵기 일괄 적용 (0.8 / 0.5 / 0.3 / 0.15pt)",
                        )
                        .size(9.6)
                        .color(ink),
                    )
                    .fill(wash)
                    .stroke(egui::Stroke::new(1.0, border))
                    .corner_radius(3.5),
                )
                .clicked()
            {
                sheet_state.line_style.section_width_mm = 0.80;
                sheet_state.line_style.outer_width_mm = 0.50;
                sheet_state.line_style.aux_width_mm = 0.20;
                sheet_state.line_style.dim_width_mm = 0.15;
                let _ = (STANDARD_LINE_WEIGHTS_MM, AI_STANDARD_LINE_WEIGHTS_PT_MM);
                *status_message =
                    String::from("『유물제도선.ai』 7단계 표준 선 굵기 위계 일괄 적용 완료");
            }
            ui.horizontal(|ui| {
                for (mode_lbl, solid, hatch) in [
                    ("▨ 45° 빗금(0.15pt)", false, true),
                    ("■ 먹물채움", true, false),
                    ("□ 외곽선만", false, false),
                ] {
                    let active = sheet_state.line_style.fill_section_solid == solid
                        && sheet_state.line_style.show_hatch_lines == hatch;
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(mode_lbl).size(9.4).color(if active {
                                    egui::Color32::WHITE
                                } else {
                                    ink
                                }),
                            )
                            .fill(if active { ink } else { wash })
                            .corner_radius(3.0),
                        )
                        .clicked()
                    {
                        sheet_state.line_style.fill_section_solid = solid;
                        sheet_state.line_style.show_hatch_lines = hatch;
                    }
                }
            });
            ui.checkbox(
                &mut pub_state.show_extension_ticks,
                "상·하·좌·우 연장선(|, —, 0.15pt) 표시",
            );
            ui.separator();

            // 섹션 4: 유물번호 · 확인 메모 · 실사 사진 슬롯 편집
            ui.label(
                egui::RichText::new("4. 유물번호 · 실측 메모 · 실사 사진 슬롯")
                    .size(10.8)
                    .strong()
                    .color(ink),
            );
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("유물번호:").size(9.6).color(muted));
                ui.add(
                    egui::TextEdit::singleline(&mut sheet_state.custom_artifact_number)
                        .hint_text("예: 000115 / 7지구 9호-7")
                        .desired_width(190.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("확인메모:").size(9.6).color(muted));
                ui.add(
                    egui::TextEdit::singleline(&mut sheet_state.custom_artifact_memo)
                        .hint_text("예: 유물재질, 결실부분 확인요함")
                        .desired_width(190.0),
                );
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut pub_state.show_photo_slots, "실사 사진 2장 배치");
                if pub_state.show_photo_slots {
                    ui.label(egui::RichText::new("높이:").size(9.4).color(muted));
                    ui.add(
                        egui::DragValue::new(&mut pub_state.photo_height_mm)
                            .speed(0.5)
                            .range(22.0..=58.0)
                            .suffix("mm"),
                    );
                }
            });
            ui.separator();

            // 섹션 5: 최종 내보내기 실행 (조판된 용지·축척 그대로 출력!)
            ui.label(
                egui::RichText::new("5. 조판 완료 도판 내보내기 (용지·축척 100% 일치)")
                    .size(10.8)
                    .strong()
                    .color(ink),
            );
            if ui
                .add_sized(
                    [ui.available_width(), 30.0],
                    egui::Button::new(
                        egui::RichText::new("📤 Adobe Illustrator (.svg / .ai) 저장")
                            .size(11.2)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(terra)
                    .corner_radius(4.5),
                )
                .clicked()
            {
                let out_path = rfd::FileDialog::new()
                    .add_filter("Adobe Illustrator 도판 (.svg, .ai)", &["svg", "ai"])
                    .set_file_name("strata_publication_plate.svg")
                    .save_file()
                    .unwrap_or_else(|| std::env::temp_dir().join("strata_publication_plate.svg"));
                let is_ai = out_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("ai"));
                if is_ai {
                    let pdf_bytes = render_plate_to_pdf_bytes(
                        &comp.page_plate,
                        PaperSheetSize::A4Portrait,
                        pub_state.margin_mm,
                    );
                    if std::fs::write(&out_path, &pdf_bytes).is_ok() {
                        *status_message = format!(
                            "Illustrator 네이티브(.ai / PDF-1.5) 조판 저장 완료: {}",
                            out_path.display()
                        );
                    }
                } else {
                    let svg =
                        export_publication_page_to_svg(&comp, sheet_state, pub_state.show_1cm_grid);
                    if std::fs::write(&out_path, svg.as_bytes()).is_ok() {
                        *status_message = format!(
                            "Illustrator 10레이어·3차베지어(.svg) 조판 저장 완료: {}",
                            out_path.display()
                        );
                    }
                }
            }
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("📄 보고서 PDF 저장")
                                .size(10.0)
                                .color(ink),
                        )
                        .fill(wash)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(3.5),
                    )
                    .clicked()
                {
                    let out_path = rfd::FileDialog::new()
                        .add_filter("출판 조판 PDF (.pdf)", &["pdf"])
                        .set_file_name("strata_publication_plate.pdf")
                        .save_file()
                        .unwrap_or_else(|| {
                            std::env::temp_dir().join("strata_publication_plate.pdf")
                        });
                    let pdf_bytes = render_plate_to_pdf_bytes(
                        &comp.page_plate,
                        PaperSheetSize::A4Portrait,
                        pub_state.margin_mm,
                    );
                    if std::fs::write(&out_path, &pdf_bytes).is_ok() {
                        *status_message =
                            format!("출판 조판 PDF 저장 완료: {}", out_path.display());
                    }
                }
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("📐 AutoCAD DXF 저장")
                                .size(10.0)
                                .color(ink),
                        )
                        .fill(wash)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(3.5),
                    )
                    .clicked()
                {
                    let out_path = rfd::FileDialog::new()
                        .add_filter("AutoCAD R12 (.dxf)", &["dxf"])
                        .set_file_name("strata_publication_plate.dxf")
                        .save_file()
                        .unwrap_or_else(|| {
                            std::env::temp_dir().join("strata_publication_plate.dxf")
                        });
                    let dxf = export_plate_to_dxf_r12(&comp.page_plate);
                    if std::fs::write(&out_path, dxf.as_bytes()).is_ok() {
                        *status_message =
                            format!("출판 조판 DXF 저장 완료: {}", out_path.display());
                    }
                }
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("📦 ret/ 폴더").size(10.0).color(ink),
                        )
                        .fill(wash)
                        .stroke(egui::Stroke::new(1.0, border))
                        .corner_radius(3.5),
                    )
                    .clicked()
                {
                    requested_action = Some("EXP_RET");
                }
            });
        });

    requested_action
}
