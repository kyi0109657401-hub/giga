use serde::{Deserialize, Serialize};
use strata_core::{
    ArtifactCategory, BeadDrillType, BeadSubtype, HandleAttachmentKind, ScalePreset,
    SherdPlateStrategy, SherdSubtype,
};
use strata_measure::ArtifactMeasurementReport;
use strata_section::{OccludedDashedSpan, PlaneSliceResult};

/// 8대 고고학 유물군별 표준 보고서 도판 배치 모드 (`DESIGN_SPEC.md` 1.3 & 3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlateLayoutMode {
    /// ① 완형 토기·도자기 (D5: 좌측 외면 입면+탁본 / 우측 수직 회전단면 + R 반전 + 저부·내저면 탁본 + 다단 횡단면)
    CompleteVesselStandard,
    /// ② 구연·저부 편조각 복원 도판 ($\Delta\theta \ge 45^\circ$ 실선 복원 / $20^\circ\sim 45^\circ$ 추정 파선 복원)
    RimOrBaseSherdReconstructed,
    /// ③ 동체부·미세 편조각 3단 병렬 도판 (`[① 외면+탁본 ── ② 수직 회전단면 ── ③ 내면+탁본]`, 억지 지름 추정 금지)
    BodyOrMicroSherdThreeColumn,
    /// ④ 파수·주구·투창 특수편 도판 (접합면/삽입촉 파선 + 파수 횡단면 보조 배치)
    HandleOrSpoutSherd,
    /// ⑤ 구슬·옥류 확대 도판 (`2:1~10:1` 미소 축척, 상면 3중 원 + 정면 + 종단면 `Ⅰa/Ⅰb/Ⅰb'/Ⅱb` 천공 파선)
    BeadJadeMicroPlate,
    /// ⑥ 곡옥(CommaJade) C자형 중심 곡선 도판 (배면/복면 + 두부 천공 단면 + 동체 횡단면)
    CommaJadeCurvedPlate,
    /// ⑦ 기와·막새 도판 (내면 포목흔 탁본 + 외면 타날문 탁본 + 와도 분할선 화살표 + 접합기법 파선 + 1㎠ 포목흔 밀도 박스)
    RoofTileAndEavesPlate,
    /// ⑧ 석기 3단계 선굵기 박리도 / 부식 금속기 3층 분리 단면 도판 (원심 코어 실선 + 부식 팽창층 파선 + 목질흔 결선)
    LithicAndCorrodedMetalPlate,
}

impl PlateLayoutMode {
    /// 8대 도판 모드 전체 목록(UI 선택 목록의 단일 출처).
    pub const ALL: [PlateLayoutMode; 8] = [
        PlateLayoutMode::CompleteVesselStandard,
        PlateLayoutMode::RimOrBaseSherdReconstructed,
        PlateLayoutMode::BodyOrMicroSherdThreeColumn,
        PlateLayoutMode::HandleOrSpoutSherd,
        PlateLayoutMode::BeadJadeMicroPlate,
        PlateLayoutMode::CommaJadeCurvedPlate,
        PlateLayoutMode::RoofTileAndEavesPlate,
        PlateLayoutMode::LithicAndCorrodedMetalPlate,
    ];

    /// 드롭다운·칩에 쓰는 짧은 한글 라벨.
    pub fn short_label_kr(self) -> &'static str {
        match self {
            Self::CompleteVesselStandard => "① 완형 표준",
            Self::RimOrBaseSherdReconstructed => "② 구연·저부 복원",
            Self::BodyOrMicroSherdThreeColumn => "③ 동체 3단 병렬",
            Self::HandleOrSpoutSherd => "④ 파수·주구",
            Self::BeadJadeMicroPlate => "⑤ 구슬·옥류",
            Self::CommaJadeCurvedPlate => "⑥ 곡옥",
            Self::RoofTileAndEavesPlate => "⑦ 기와·막새",
            Self::LithicAndCorrodedMetalPlate => "⑧ 석기·금속기",
        }
    }

    pub fn select_for_artifact(
        category: ArtifactCategory,
        sherd_subtype: Option<SherdSubtype>,
        sherd_strategy: Option<SherdPlateStrategy>,
        bead_subtype: Option<BeadSubtype>,
    ) -> Self {
        match category {
            ArtifactCategory::CompleteVessel => Self::CompleteVesselStandard,
            ArtifactCategory::Potsherd => match sherd_subtype {
                Some(SherdSubtype::HandleSpoutSherd) => Self::HandleOrSpoutSherd,
                Some(SherdSubtype::BodySherd) => Self::BodyOrMicroSherdThreeColumn,
                _ => match sherd_strategy {
                    Some(SherdPlateStrategy::ThreeColumnParallel) => {
                        Self::BodyOrMicroSherdThreeColumn
                    }
                    _ => Self::RimOrBaseSherdReconstructed,
                },
            },
            ArtifactCategory::BeadJade => {
                if bead_subtype == Some(BeadSubtype::CommaJade) {
                    Self::CommaJadeCurvedPlate
                } else {
                    Self::BeadJadeMicroPlate
                }
            }
            ArtifactCategory::RoofTile => Self::RoofTileAndEavesPlate,
            ArtifactCategory::Lithic | ArtifactCategory::CorrodedMetal => {
                Self::LithicAndCorrodedMetalPlate
            }
        }
    }

    pub fn label_kr(self) -> &'static str {
        match self {
            Self::CompleteVesselStandard => "① 완형 토기·도자기 표준 도판 (좌 외면·우 단면)",
            Self::RimOrBaseSherdReconstructed => "② 구연·저부 편조각 회전 복원 도판",
            Self::BodyOrMicroSherdThreeColumn => "③ 동체·미세편 3단 병렬 도판 (외면·단면·내면)",
            Self::HandleOrSpoutSherd => "④ 파수·주구·특수편 접합 도판",
            Self::BeadJadeMicroPlate => "⑤ 구슬·옥류 확대 도판 (상면 3원 + 천공 종단면)",
            Self::CommaJadeCurvedPlate => "⑥ 곡옥 C자형 중심축 + 두부 천공 도판",
            Self::RoofTileAndEavesPlate => "⑦ 기와·막새 도판 (와도 분할선 + 포목흔 밀도)",
            Self::LithicAndCorrodedMetalPlate => "⑧ 석기 3단 선굵기 / 금속기 3층 부식 분리 도판",
        }
    }
}

/// 도판 용지 위에 합성될 다중 뷰 정밀 탁본(`Rubbing`) 래스터 영상의 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlateRasterKind {
    /// 정면 외면 정밀 탁본 (완형 토기는 외면 반쪽 클리핑, 편조각·기와·구슬은 전체 외면)
    FrontExteriorRubbing,
    /// 내면 정밀 탁본 (3단 병렬 편조각·기와의 내면 물레속선/포목흔)
    BackInteriorRubbing,
    /// 저면·외저면 원형 정밀 탁본 (완형 토기 외저면 인화문·굽 탁본)
    BottomPlanRubbing,
}

/// 도판 용지 위 정밀 탁본 래스터 배치 영역 (`mm` 좌표계).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlateRasterRegion {
    pub kind: PlateRasterKind,
    /// `[min_x_mm, min_y_mm, max_x_mm, max_y_mm]`
    pub rect_mm: [f64; 4],
    /// 반단면 클리핑: `Some(-1)`이면 좌측 반쪽만, `Some(1)`이면 우측 반쪽만, `None`이면 전체 표시
    pub clip_half_sign: Option<i8>,
    pub caption: String,
}

/// 도판 내 개별 벡터 선분 또는 폴리라인 요소.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatePathElement {
    pub layer_id: String,
    /// mm 단위 2D 좌표열 (`1 mm = 5 px` 변환 전 실측 mm 좌표)
    pub points_mm: Vec<[f64; 2]>,
    pub closed: bool,
    pub stroke_hex: String,
    pub stroke_width_mm: f64,
    pub dash_pattern_mm: Option<(f64, f64)>,
    pub fill_hex: Option<String>,
}

/// 도판 내 텍스트 주석 및 치수 콜아웃 요소 (하드코딩 금지: 오직 실측치에서만 생성).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlateTextCallout {
    pub layer_id: String,
    pub pos_mm: [f64; 2],
    pub text: String,
    pub font_size_pt: f64,
    pub color_hex: String,
}

/// 합성 완료된 2D 고고학 실측 도판 문서 (`SVG / PDF / DXF / SheetPane` 공용).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComposedPlate2D {
    pub mode: PlateLayoutMode,
    pub title: String,
    pub scale_preset: ScalePreset,
    /// 실측 도판 전체 폭 (mm)
    pub canvas_width_mm: f64,
    /// 실측 도판 전체 높이 (mm)
    pub canvas_height_mm: f64,
    /// `R` 반전 여부 (`false`: 좌 외면 / 우 단면, `true`: 좌 단면 / 우 외면)
    pub flip_exterior_right: bool,
    pub paths: Vec<PlatePathElement>,
    pub callouts: Vec<PlateTextCallout>,
    /// 도판 용지 위에 합성할 정밀 탁본(`Rubbing`) 다중 뷰 래스터 영역 목록
    #[serde(default)]
    pub raster_regions: Vec<PlateRasterRegion>,
}

/// 도판 합성에 필요한 입력 컨텍스트.
#[derive(Debug, Clone)]
pub struct PlateComposeInput<'a> {
    pub title: &'a str,
    pub measurements: &'a ArtifactMeasurementReport,
    pub vertical_section: &'a PlaneSliceResult,
    pub horizontal_sections: &'a [PlaneSliceResult],
    pub occluded_spans: &'a [OccludedDashedSpan],
    pub sherd_subtype: Option<SherdSubtype>,
    pub sherd_strategy: Option<SherdPlateStrategy>,
    pub handle_attachment: Option<HandleAttachmentKind>,
    pub bead_subtype: Option<BeadSubtype>,
    pub bead_drill_type: Option<BeadDrillType>,
    pub wado_cut_ratio: Option<f64>,
    pub hemp_density_label: Option<String>,
    pub flip_exterior_right: bool,
}

/// 비볼록(Concave) 2D 단면 다각형을 Ear-Clipping 알고리즘으로 삼각분할한다.
/// - 반환값: 입력 `poly` 정점 인덱스 삼조 `[i0, i1, i2]` 목록.
pub fn triangulate_polygon_ear_clipping(poly: &[[f64; 2]]) -> Vec<[usize; 3]> {
    if poly.len() < 3 {
        return Vec::new();
    }

    // 1. 중복·초근접 연속 정점(< 0.12mm) 제거 (원본 인덱스 보존)
    let mut clean_indices: Vec<usize> = Vec::with_capacity(poly.len());
    for i in 0..poly.len() {
        if let Some(&last_idx) = clean_indices.last() {
            let p = poly[last_idx];
            let q = poly[i];
            if (p[0] - q[0]).hypot(p[1] - q[1]) < 0.12 {
                continue;
            }
        }
        clean_indices.push(i);
    }
    if clean_indices.len() >= 2 {
        let first = poly[clean_indices[0]];
        let last = poly[clean_indices[clean_indices.len() - 1]];
        if (first[0] - last[0]).hypot(first[1] - last[1]) < 0.12 {
            clean_indices.pop();
        }
    }
    let n = clean_indices.len();
    if n < 3 {
        return Vec::new();
    }

    // 2. Shoelace 부호 면적으로 회전 방향(CCW > 0, CW < 0) 판별
    let mut signed_area2 = 0.0f64;
    for i in 0..n {
        let p = poly[clean_indices[i]];
        let q = poly[clean_indices[(i + 1) % n]];
        signed_area2 += p[0] * q[1] - q[0] * p[1];
    }
    let mut indices: Vec<usize> = if signed_area2 >= 0.0 {
        clean_indices
    } else {
        clean_indices.into_iter().rev().collect()
    };

    let mut tris = Vec::with_capacity(n.saturating_sub(2));
    let mut guard = n * 4 + 16;

    while indices.len() > 3 && guard > 0 {
        guard -= 1;
        let m = indices.len();
        let mut ear_found = false;
        let mut best_convex_idx = None;
        let mut best_convex_cross = 0.0f64;

        for idx_i in 0..m {
            let prev = indices[(idx_i + m - 1) % m];
            let curr = indices[idx_i];
            let next = indices[(idx_i + 1) % m];

            let a = poly[prev];
            let b = poly[curr];
            let c = poly[next];

            let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            if cross <= 1e-8 {
                continue;
            }
            if cross > best_convex_cross {
                best_convex_cross = cross;
                best_convex_idx = Some(idx_i);
            }

            let mut contains_other = false;
            for &other in &indices {
                if other == prev || other == curr || other == next {
                    continue;
                }
                if point_in_triangle_2d(poly[other], a, b, c) {
                    contains_other = true;
                    break;
                }
            }

            if !contains_other {
                tris.push([prev, curr, next]);
                indices.remove(idx_i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            let pick = best_convex_idx.unwrap_or(1);
            let prev = indices[(pick + m - 1) % m];
            let curr = indices[pick];
            let next = indices[(pick + 1) % m];
            tris.push([prev, curr, next]);
            indices.remove(pick);
        }
    }

    if indices.len() == 3 {
        tris.push([indices[0], indices[1], indices[2]]);
    }
    tris
}

#[inline]
fn point_in_triangle_2d(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = (p[0] - b[0]) * (a[1] - b[1]) - (a[0] - b[0]) * (p[1] - b[1]);
    let d2 = (p[0] - c[0]) * (b[1] - c[1]) - (b[0] - c[0]) * (p[1] - c[1]);
    let d3 = (p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1]);
    let has_neg = (d1 < -1e-9) || (d2 < -1e-9) || (d3 < -1e-9);
    let has_pos = (d1 > 1e-9) || (d2 > 1e-9) || (d3 > 1e-9);
    !(has_neg && has_pos)
}

/// 폴리라인(폐합 루프 또는 열린 곡선)을 `x = 0.0` 반평면(`keep_positive_x`)으로 절단하되,
/// 중공(빈 공간)을 가로지르는 가짜 직선 현(chord)을 만들지 않고 반평면 내부의 연속 체인들만 추출한다.
fn clip_polyline_half_plane_x_chains(
    poly: &[[f64; 2]],
    is_closed_loop: bool,
    keep_positive_x: bool,
) -> Vec<Vec<[f64; 2]>> {
    if poly.len() < 2 {
        return Vec::new();
    }
    let inside = |p: [f64; 2]| -> bool {
        if keep_positive_x {
            p[0] >= -1e-5
        } else {
            p[0] <= 1e-5
        }
    };
    let n = poly.len();
    let edge_count = if is_closed_loop { n } else { n - 1 };
    let mut chains: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut cur_chain: Vec<[f64; 2]> = Vec::new();

    if inside(poly[0]) {
        cur_chain.push(poly[0]);
    }

    for i in 0..edge_count {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let seg_len = (b[0] - a[0]).hypot(b[1] - a[1]);
        // 체이닝 단절 구간(개구부 양끝 등 24mm 초과 점프)은 실제 벽면 선분이 아니므로 분리한다.
        if seg_len > 24.0 {
            if cur_chain.len() >= 2 {
                chains.push(std::mem::take(&mut cur_chain));
            } else {
                cur_chain.clear();
            }
            if inside(b) {
                cur_chain.push(b);
            }
            continue;
        }
        let a_in = inside(a);
        let b_in = inside(b);
        match (a_in, b_in) {
            (true, true) => {
                if cur_chain.is_empty() {
                    cur_chain.push(a);
                }
                cur_chain.push(b);
            }
            (true, false) => {
                if cur_chain.is_empty() {
                    cur_chain.push(a);
                }
                let dx = b[0] - a[0];
                if dx.abs() > 1e-12 {
                    let t = (-a[0] / dx).clamp(0.0, 1.0);
                    let iy = a[1] + t * (b[1] - a[1]);
                    cur_chain.push([0.0, iy]);
                }
                if cur_chain.len() >= 2 {
                    chains.push(std::mem::take(&mut cur_chain));
                } else {
                    cur_chain.clear();
                }
            }
            (false, true) => {
                if cur_chain.len() >= 2 {
                    chains.push(std::mem::take(&mut cur_chain));
                } else {
                    cur_chain.clear();
                }
                let dx = b[0] - a[0];
                if dx.abs() > 1e-12 {
                    let t = (-a[0] / dx).clamp(0.0, 1.0);
                    let iy = a[1] + t * (b[1] - a[1]);
                    cur_chain.push([0.0, iy]);
                }
                cur_chain.push(b);
            }
            (false, false) => {}
        }
    }
    if cur_chain.len() >= 2 {
        chains.push(cur_chain);
    }
    if is_closed_loop && chains.len() >= 2 {
        let first_pt = chains[0][0];
        let last_pt = *chains.last().and_then(|c| c.last()).unwrap_or(&first_pt);
        if (first_pt[0] - last_pt[0]).hypot(first_pt[1] - last_pt[1]) <= 1.0 {
            let mut head = chains.remove(0);
            if let Some(tail) = chains.last_mut() {
                tail.extend(head.drain(1..));
            }
        }
    }
    chains
}

/// 단일 외피 체인(`chain`)을 중심 `(0.0, center_z)` 방향으로 `wall_t`만큼 안쪽 오프셋하여
/// 중공 내부를 침범하지 않는 실제 기벽 두께(`wall_t`)의 폐합 단면 다각형을 생성한다.
fn offset_chain_inward_to_closed_wall(
    chain: &[[f64; 2]],
    center_z: f64,
    wall_t: f64,
) -> Vec<[f64; 2]> {
    if chain.len() < 2 {
        return chain.to_vec();
    }
    let mut out = Vec::with_capacity(chain.len() * 2);
    out.extend_from_slice(chain);
    for pt in chain.iter().rev() {
        let dx = pt[0];
        let dz = pt[1] - center_z;
        let len = dx.hypot(dz);
        if len > 1e-6 {
            let scale = ((len - wall_t).max(0.0)) / len;
            let ix = if pt[0].abs() <= 0.25 { 0.0 } else { dx * scale };
            out.push([ix, center_z + dz * scale]);
        } else {
            out.push(*pt);
        }
    }
    out
}

/// 외면 반평면(`x * ext_sign >= 0`) 체인들로부터 내벽선을 제거하고,
/// 임의의 높이 스캔라인(`z`)과 정확히 1번만 교차하는 **단일 최외곽 외벽 실루엣(Outer Contour)** 1줄만 추출한다 (`A1`).
fn extract_single_outer_silhouette_contour(
    ext_chains: &[Vec<[f64; 2]>],
    height: f64,
    keep_positive_x: bool,
) -> Vec<[f64; 2]> {
    let ext_sign = if keep_positive_x { -1.0 } else { 1.0 };
    let mut segs: Vec<[[f64; 2]; 2]> = Vec::new();
    for chain in ext_chains {
        for w in chain.windows(2) {
            if (w[0][0] - w[1][0]).hypot(w[0][1] - w[1][1]) <= 24.0 {
                segs.push([w[0], w[1]]);
            }
        }
    }
    if segs.is_empty() {
        return Vec::new();
    }

    const N_Z: usize = 64;
    let mut raw_samples: Vec<(f64, Option<f64>)> = Vec::with_capacity(N_Z + 1);
    for k in 0..=N_Z {
        let z = (k as f64 / N_Z as f64) * height;
        let mut best_mag = None::<f64>;
        for &[a, b] in &segs {
            let z_lo = a[1].min(b[1]);
            let z_hi = a[1].max(b[1]);
            if z >= z_lo - 1e-4 && z <= z_hi + 1e-4 {
                let dz = b[1] - a[1];
                let x = if dz.abs() > 1e-9 {
                    let t = ((z - a[1]) / dz).clamp(0.0, 1.0);
                    a[0] + t * (b[0] - a[0])
                } else if (a[0] * ext_sign) >= (b[0] * ext_sign) {
                    a[0]
                } else {
                    b[0]
                };
                let mag = x * ext_sign;
                if mag >= -1e-4 {
                    best_mag = Some(best_mag.map_or(mag.max(0.0), |cur| cur.max(mag.max(0.0))));
                }
            }
        }
        raw_samples.push((z, best_mag));
    }

    // 빈 구간이 있으면 가장 가까운 유효 스캔라인 값으로 선형 보간하여 Z 단조성을 100% 보장한다.
    let mut filled: Vec<[f64; 2]> = Vec::with_capacity(N_Z + 1);
    for i in 0..raw_samples.len() {
        let z = raw_samples[i].0;
        let mag = if let Some(m) = raw_samples[i].1 {
            m
        } else {
            let prev = (0..i).rev().find_map(|p| raw_samples[p].1.map(|m| (p, m)));
            let next = ((i + 1)..raw_samples.len()).find_map(|n| raw_samples[n].1.map(|m| (n, m)));
            match (prev, next) {
                (Some((pi, pm)), Some((ni, nm))) => {
                    let t = (i - pi) as f64 / (ni - pi).max(1) as f64;
                    pm + t * (nm - pm)
                }
                (Some((_, pm)), None) => pm,
                (None, Some((_, nm))) => nm,
                (None, None) => continue,
            }
        };
        filled.push([mag * ext_sign, z]);
    }
    filled
}

/// 외벽 실루엣 프로파일(`outer_silhouette`)의 실제 2차 미분 곡률 변곡점($\Delta^2 x$ 부호 교차·극값)에서만
/// 입면 구조선 높이(`z`)를 추출한다 (`A4`: `[0.93, 0.32, 0.09]` 고정 비율 하드코딩 금지).
fn detect_outer_curvature_inflection_points(
    outer_silhouette: &[[f64; 2]],
    height: f64,
) -> Vec<[f64; 2]> {
    if outer_silhouette.len() < 7 {
        return Vec::new();
    }
    let mut candidates: Vec<(f64, [f64; 2])> = Vec::new();
    for i in 2..(outer_silhouette.len() - 2) {
        let p_prev = outer_silhouette[i - 2];
        let p_cur = outer_silhouette[i];
        let p_next = outer_silhouette[i + 2];
        if p_cur[1] < height * 0.12 || p_cur[1] > height * 0.88 || p_cur[0].abs() < 3.0 {
            continue;
        }
        let d1 = p_cur[0].abs() - p_prev[0].abs();
        let d2 = p_next[0].abs() - p_cur[0].abs();
        let curv = (d2 - d1).abs();
        let is_inflection_or_corner = (d1 * d2 <= 0.0 && curv > 0.25) || curv > 0.65;
        if is_inflection_or_corner {
            candidates.push((curv, p_cur));
        }
    }
    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut picked: Vec<[f64; 2]> = Vec::new();
    for (_, pt) in candidates {
        if picked
            .iter()
            .all(|existing| (existing[1] - pt[1]).abs() >= height * 0.22)
        {
            picked.push(pt);
            if picked.len() >= 2 {
                break;
            }
        }
    }
    picked
}

type MeasuredHorizSectionContours = (Vec<[f64; 2]>, Vec<[f64; 2]>, Vec<[f64; 2]>, f64);

/// 수평 절단면(`PlaneSliceResult`)의 실제 3D 메쉬 교차 점군(`polylines_2d` + `segments_2d`)으로부터
/// 72방위각(5° 간격) 실제 외벽 윤곽선(`outer_contour`), 실제 내벽 윤곽선(`inner_contour`),
/// 우측 반단면 폐합 폴리곤(`right_half_poly`), 그리고 최대 반경(`max_r`)을 추출한다 (`Image 2`).
fn extract_measured_horizontal_section_contours(
    hsec: &PlaneSliceResult,
    wall_t: f64,
    fallback_r: f64,
) -> MeasuredHorizSectionContours {
    const N_BINS: usize = 72;
    let mut all_pts: Vec<[f64; 2]> = hsec.polylines_2d.iter().flatten().copied().collect();
    for seg in &hsec.segments_2d {
        all_pts.push(seg[0]);
        all_pts.push(seg[1]);
    }

    if all_pts.len() < 6 {
        let ro = fallback_r.max(10.0);
        let ri = (ro - wall_t).max(6.0);
        let mut outer = Vec::with_capacity(N_BINS);
        let mut inner = Vec::with_capacity(N_BINS);
        for k in 0..N_BINS {
            let a = -std::f64::consts::PI + (k as f64) * std::f64::consts::TAU / (N_BINS as f64);
            outer.push([ro * a.cos(), ro * a.sin()]);
            inner.push([ri * a.cos(), ri * a.sin()]);
        }
        let mut rpoly = Vec::with_capacity(38);
        for step in 0..=18 {
            let a = -std::f64::consts::FRAC_PI_2 + (step as f64) * std::f64::consts::PI / 18.0;
            rpoly.push([ro * a.cos(), ro * a.sin()]);
        }
        for step in (0..=18).rev() {
            let a = -std::f64::consts::FRAC_PI_2 + (step as f64) * std::f64::consts::PI / 18.0;
            rpoly.push([ri * a.cos(), ri * a.sin()]);
        }
        return (outer, inner, rpoly, ro);
    }

    let mut bin_max_r = [0.0_f64; N_BINS];
    let mut bin_min_r = [f64::INFINITY; N_BINS];
    let mut bin_has = [false; N_BINS];

    for p in &all_pts {
        let r = p[0].hypot(p[1]);
        if r < 1.0 || !r.is_finite() {
            continue;
        }
        let ang = p[1].atan2(p[0]); // [-pi, pi]
        let norm = ((ang + std::f64::consts::PI) / std::f64::consts::TAU).clamp(0.0, 0.999_999);
        let bi = (norm * (N_BINS as f64)) as usize;
        bin_has[bi] = true;
        if r > bin_max_r[bi] {
            bin_max_r[bi] = r;
        }
        if r < bin_min_r[bi] {
            bin_min_r[bi] = r;
        }
    }

    // 인접 빈(±3 섹터) 보간으로 미세 삼각형 틈새 연결 및 실제 비대칭·타원 형상 100% 보존
    let mut smooth_out_r = [0.0_f64; N_BINS];
    let mut smooth_in_r = [0.0_f64; N_BINS];
    let mut smooth_valid = [false; N_BINS];
    let mut global_max_r = 10.0_f64;

    for k in 0..N_BINS {
        let mut sum_o = 0.0;
        let mut sum_i = 0.0;
        let mut w_sum = 0.0;
        for d in -2isize..=2isize {
            let nk = ((k as isize + d).rem_euclid(N_BINS as isize)) as usize;
            if bin_has[nk] {
                let w = match d.abs() {
                    0 => 1.0,
                    1 => 0.65,
                    _ => 0.35,
                };
                let ro = bin_max_r[nk];
                let ri = if (ro - bin_min_r[nk]) >= 1.2 {
                    bin_min_r[nk].max((ro - wall_t * 1.8).max(2.0))
                } else {
                    (ro - wall_t).max(2.0)
                };
                sum_o += ro * w;
                sum_i += ri * w;
                w_sum += w;
            }
        }
        if w_sum > 0.0 {
            let ro = sum_o / w_sum;
            let ri = (sum_i / w_sum).clamp(1.5, (ro - 1.2).max(1.5));
            smooth_out_r[k] = ro;
            smooth_in_r[k] = ri;
            smooth_valid[k] = true;
            if ro > global_max_r {
                global_max_r = ro;
            }
        }
    }

    let mut outer_pts = Vec::with_capacity(N_BINS);
    let mut inner_pts = Vec::with_capacity(N_BINS);
    let mut right_outer = Vec::new();
    let mut right_inner = Vec::new();

    for k in 0..N_BINS {
        if !smooth_valid[k] {
            continue;
        }
        let a =
            -std::f64::consts::PI + ((k as f64) + 0.5) * std::f64::consts::TAU / (N_BINS as f64);
        let (sin_a, cos_a) = a.sin_cos();
        let po = [smooth_out_r[k] * cos_a, smooth_out_r[k] * sin_a];
        let pi = [smooth_in_r[k] * cos_a, smooth_in_r[k] * sin_a];
        outer_pts.push(po);
        inner_pts.push(pi);
        if cos_a >= -0.02 {
            right_outer.push(po);
            right_inner.push(pi);
        }
    }

    let mut right_half_poly = Vec::with_capacity(right_outer.len() + right_inner.len());
    right_half_poly.extend_from_slice(&right_outer);
    for pt in right_inner.into_iter().rev() {
        right_half_poly.push(pt);
    }

    (outer_pts, inner_pts, right_half_poly, global_max_r)
}

/// `Image 0 ("그리고 평면 아닌가? 실제 모습이 평면도에 나와야하지않나? 깨진부분은 점선아닌가?")`:
/// 실제 3D 메쉬에서 추출된 직상방 평면 윤곽(`top_plan_opt`) 또는 수평 단면·결손 정보로부터
/// 좌상단 `1:1 평면도`의 **실제 비대칭·유기적 외곽/내경 실선(`평면도_외곽윤곽선`, `평면도_구연내경선`)**,
/// **깨진 부분(구연 결손부)의 추정 복원 점선(`평면도_결손복원_점선`)**, 그리고 **실제 파손 경계 실선(`평면도_파손경계_실선`)**을 제도한다.
/// 불필요한 하단/내부 원(`평면도_굽외경선`, `평면도_내저원각선`)과 회전 시 가짜 외곽 점선 원을 제거하여 좌측 3D 직상방 뷰와 1:1 일치시킨다.
#[allow(clippy::too_many_arguments)]
fn append_measured_top_plan_view_paths(
    paths: &mut Vec<PlatePathElement>,
    cx_plan: f64,
    cy_plan: f64,
    plan_r: f64,
    _base_r: f64,
    wall_t: f64,
    _is_upside_down: bool,
    top_plan_opt: Option<&PlaneSliceResult>,
    hsec_opt: Option<&PlaneSliceResult>,
) {
    const N_SECTORS: usize = 72;

    // 1. 중심 십자 기준선 (외저면_십자선)
    paths.push(PlatePathElement {
        layer_id: String::from("외저면_십자선"),
        points_mm: vec![
            [cx_plan - plan_r * 1.06, cy_plan],
            [cx_plan + plan_r * 1.06, cy_plan],
        ],
        closed: false,
        stroke_hex: String::from("#5E5D59"),
        stroke_width_mm: 0.20,
        dash_pattern_mm: None,
        fill_hex: None,
    });
    paths.push(PlatePathElement {
        layer_id: String::from("외저면_십자선"),
        points_mm: vec![
            [cx_plan, cy_plan - plan_r * 1.06],
            [cx_plan, cy_plan + plan_r * 1.06],
        ],
        closed: false,
        stroke_hex: String::from("#5E5D59"),
        stroke_width_mm: 0.20,
        dash_pattern_mm: None,
        fill_hex: None,
    });

    // 2. 실제 3D 직상방 평면 윤곽 점군 확보 (72 섹터)
    let mut outer_ring: Vec<[f64; 2]> = Vec::with_capacity(N_SECTORS);
    let mut inner_ring: Vec<[f64; 2]> = Vec::with_capacity(N_SECTORS);
    let mut is_broken = [false; N_SECTORS];
    let mut recon_outer_r = plan_r;
    let mut recon_inner_r = (plan_r - wall_t).max(5.0);

    if let Some(tp) = top_plan_opt {
        if let Some(p0) = tp.polylines_2d.first() {
            if p0.len() == N_SECTORS {
                outer_ring.clone_from(p0);
            }
        }
        if let Some(p1) = tp.polylines_2d.get(1) {
            if p1.len() == N_SECTORS {
                inner_ring.clone_from(p1);
            }
        }
        for seg in &tp.segments_2d {
            let k0 = (seg[0][0].round() as isize).rem_euclid(N_SECTORS as isize) as usize;
            let k1 = (seg[0][1].round() as isize).rem_euclid(N_SECTORS as isize) as usize;
            if seg[1][0] > 4.0 {
                recon_outer_r = seg[1][0];
            }
            if seg[1][1] > 3.0 {
                recon_inner_r = seg[1][1];
            }
            let mut cur = k0;
            for _ in 0..=N_SECTORS {
                is_broken[cur] = true;
                if cur == k1 {
                    break;
                }
                cur = (cur + 1) % N_SECTORS;
            }
        }
    }

    // top_plan_opt가 없을 때(단위 테스트 등): hsec_opt의 실제 절단 윤곽 또는 정원형 윤곽으로 구성
    if outer_ring.len() != N_SECTORS {
        outer_ring.clear();
        inner_ring.clear();
        if let Some(hsec) = hsec_opt {
            let (ho, hi, _, hmax_r) =
                extract_measured_horizontal_section_contours(hsec, wall_t, plan_r);
            let scale_r = if hmax_r > 4.0 { plan_r / hmax_r } else { 1.0 };
            if ho.len() == N_SECTORS && hi.len() == N_SECTORS {
                for k in 0..N_SECTORS {
                    outer_ring.push([ho[k][0] * scale_r, ho[k][1] * scale_r]);
                    inner_ring.push([hi[k][0] * scale_r, hi[k][1] * scale_r]);
                }
            }
        }
        if outer_ring.len() != N_SECTORS {
            outer_ring.clear();
            inner_ring.clear();
            for k in 0..N_SECTORS {
                let a = -std::f64::consts::PI
                    + ((k as f64) + 0.5) * std::f64::consts::TAU / (N_SECTORS as f64);
                let (s, c) = a.sin_cos();
                let ro = plan_r;
                let ri = (plan_r - wall_t).max(4.0);
                outer_ring.push([ro * c, ro * s]);
                inner_ring.push([ri * c, ri * s]);
            }
        }
    }

    // 결손 구간(is_broken)이 전혀 없는 경우(온전한 유물 또는 90° 회전된 유물):
    // 가짜 결손 점선 원을 그리지 않고 실제 3D 직상방 외곽·내경 실선 루프만 1:1로 깔끔하게 제도한다!
    // A5: 평면도 외곽원의 중심 X좌표(cx_plan)와 좌우 외경 폭(2 * plan_r)이 정면 입·단면도의 중심축(cx) 및 구경 기준선과 0.01mm 이내로 일치하도록 정규화한다.
    if !is_broken.iter().any(|&b| b) {
        let min_ox = outer_ring
            .iter()
            .map(|p| p[0])
            .fold(f64::INFINITY, f64::min);
        let max_ox = outer_ring
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max);
        let min_oy = outer_ring
            .iter()
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min);
        let max_oy = outer_ring
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max);
        let mid_ox = if min_ox.is_finite() && max_ox.is_finite() {
            0.5 * (min_ox + max_ox)
        } else {
            0.0
        };
        let mid_oy = if min_oy.is_finite() && max_oy.is_finite() {
            0.5 * (min_oy + max_oy)
        } else {
            0.0
        };
        let half_span_ox = 0.5 * (max_ox - min_ox);
        let scale_ox = if !inner_ring.is_empty() && half_span_ox > 2.0 {
            plan_r / half_span_ox
        } else {
            1.0
        };
        let out_pts: Vec<[f64; 2]> = outer_ring
            .iter()
            .map(|p| {
                [
                    cx_plan + (p[0] - mid_ox) * scale_ox,
                    cy_plan + (p[1] - mid_oy),
                ]
            })
            .collect();
        paths.push(PlatePathElement {
            layer_id: String::from("평면도_외곽윤곽선"),
            points_mm: out_pts,
            closed: true,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.35,
            dash_pattern_mm: None,
            fill_hex: None,
        });
        if !inner_ring.is_empty() {
            let in_pts: Vec<[f64; 2]> = inner_ring
                .iter()
                .map(|p| {
                    [
                        cx_plan + (p[0] - mid_ox) * scale_ox,
                        cy_plan + (p[1] - mid_oy),
                    ]
                })
                .collect();
            paths.push(PlatePathElement {
                layer_id: String::from("평면도_구연내경선"),
                points_mm: in_pts,
                closed: true,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.28,
                dash_pattern_mm: None,
                fill_hex: None,
            });
        }
        return;
    }

    // 3. 국소 결손부가 존재하는 정위치 토기의 경우: 온전한 구간(실선)과 깨진 구간(점선 + 파손 경계 실선) 분리 제도
    let mut cur_solid_out: Vec<[f64; 2]> = Vec::new();
    let mut cur_solid_in: Vec<[f64; 2]> = Vec::new();
    let mut cur_dash_out: Vec<[f64; 2]> = Vec::new();
    let mut cur_dash_in: Vec<[f64; 2]> = Vec::new();
    let mut cur_fracture: Vec<[f64; 2]> = Vec::new();

    for step in 0..=N_SECTORS {
        let k = step % N_SECTORS;
        let a =
            -std::f64::consts::PI + ((k as f64) + 0.5) * std::f64::consts::TAU / (N_SECTORS as f64);
        let (s, c) = a.sin_cos();
        let po = [cx_plan + outer_ring[k][0], cy_plan + outer_ring[k][1]];
        let pi = [cx_plan + inner_ring[k][0], cy_plan + inner_ring[k][1]];
        let recon_po = [cx_plan + recon_outer_r * c, cy_plan + recon_outer_r * s];
        let recon_pi = [cx_plan + recon_inner_r * c, cy_plan + recon_inner_r * s];

        if !is_broken[k] {
            if !cur_dash_out.is_empty() {
                cur_dash_out.push(po);
                cur_dash_in.push(pi);
                cur_fracture.push(po);
                if cur_dash_out.len() >= 2 {
                    paths.push(PlatePathElement {
                        layer_id: String::from("평면도_결손복원_점선"),
                        points_mm: std::mem::take(&mut cur_dash_out),
                        closed: false,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: 0.42,
                        dash_pattern_mm: Some((2.2, 1.4)),
                        fill_hex: None,
                    });
                } else {
                    cur_dash_out.clear();
                }
                if cur_dash_in.len() >= 2 {
                    paths.push(PlatePathElement {
                        layer_id: String::from("평면도_결손복원_점선"),
                        points_mm: std::mem::take(&mut cur_dash_in),
                        closed: false,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: 0.34,
                        dash_pattern_mm: Some((2.0, 1.3)),
                        fill_hex: None,
                    });
                } else {
                    cur_dash_in.clear();
                }
                if cur_fracture.len() >= 2 {
                    paths.push(PlatePathElement {
                        layer_id: String::from("평면도_파손경계_실선"),
                        points_mm: std::mem::take(&mut cur_fracture),
                        closed: false,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: 0.28,
                        dash_pattern_mm: None,
                        fill_hex: None,
                    });
                } else {
                    cur_fracture.clear();
                }
            }
            cur_solid_out.push(po);
            cur_solid_in.push(pi);
        } else {
            if !cur_solid_out.is_empty() {
                let last_po = *cur_solid_out.last().unwrap_or(&po);
                let last_pi = *cur_solid_in.last().unwrap_or(&pi);
                if cur_solid_out.len() >= 2 {
                    paths.push(PlatePathElement {
                        layer_id: String::from("평면도_외곽윤곽선"),
                        points_mm: std::mem::take(&mut cur_solid_out),
                        closed: false,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: 0.35,
                        dash_pattern_mm: None,
                        fill_hex: None,
                    });
                } else {
                    cur_solid_out.clear();
                }
                if cur_solid_in.len() >= 2 {
                    paths.push(PlatePathElement {
                        layer_id: String::from("평면도_구연내경선"),
                        points_mm: std::mem::take(&mut cur_solid_in),
                        closed: false,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: 0.28,
                        dash_pattern_mm: None,
                        fill_hex: None,
                    });
                } else {
                    cur_solid_in.clear();
                }
                cur_dash_out.push(last_po);
                cur_dash_in.push(last_pi);
                cur_fracture.push(last_po);
            }
            cur_dash_out.push(recon_po);
            cur_dash_in.push(recon_pi);
            cur_fracture.push(po);
        }
    }

    if cur_solid_out.len() >= 2 {
        paths.push(PlatePathElement {
            layer_id: String::from("평면도_외곽윤곽선"),
            points_mm: cur_solid_out,
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.35,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }
    if cur_solid_in.len() >= 2 {
        paths.push(PlatePathElement {
            layer_id: String::from("평면도_구연내경선"),
            points_mm: cur_solid_in,
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.28,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }
    if cur_dash_out.len() >= 2 {
        paths.push(PlatePathElement {
            layer_id: String::from("평면도_결손복원_점선"),
            points_mm: cur_dash_out,
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.32,
            dash_pattern_mm: Some((2.0, 1.3)),
            fill_hex: None,
        });
    }
    if cur_dash_in.len() >= 2 {
        paths.push(PlatePathElement {
            layer_id: String::from("평면도_결손복원_점선"),
            points_mm: cur_dash_in,
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.26,
            dash_pattern_mm: Some((1.8, 1.2)),
            fill_hex: None,
        });
    }
    if cur_fracture.len() >= 2 {
        paths.push(PlatePathElement {
            layer_id: String::from("평면도_파손경계_실선"),
            points_mm: cur_fracture,
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.28,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }
}

/// 닫힌 폴리곤 내부에 45° 사선 해칭 선분을 생성한다(스캔라인 even-odd).
/// 「유물제도선」 규격처럼 단면을 빗금으로 표시할 때 쓴다.
pub fn hatch_polygon_45(poly: &[[f64; 2]], gap_mm: f64) -> Vec<[[f64; 2]; 2]> {
    if poly.len() < 3 || gap_mm <= 1e-6 {
        return Vec::new();
    }
    let inv = std::f64::consts::FRAC_1_SQRT_2;
    // 45° 회전 좌표계로 옮겨 수평 스캔라인으로 처리한다.
    let rot = |p: [f64; 2]| -> [f64; 2] { [(p[0] - p[1]) * inv, (p[0] + p[1]) * inv] };
    let unrot = |p: [f64; 2]| -> [f64; 2] { [(p[0] + p[1]) * inv, (p[1] - p[0]) * inv] };
    let rp: Vec<[f64; 2]> = poly.iter().map(|&p| rot(p)).collect();
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    for p in &rp {
        min_v = min_v.min(p[1]);
        max_v = max_v.max(p[1]);
    }
    let mut out = Vec::new();
    let mut v = min_v + gap_mm * 0.5;
    while v < max_v {
        let mut xs: Vec<f64> = Vec::new();
        for i in 0..rp.len() {
            let a = rp[i];
            let b = rp[(i + 1) % rp.len()];
            if (a[1] <= v && b[1] > v) || (b[1] <= v && a[1] > v) {
                let t = (v - a[1]) / (b[1] - a[1]);
                xs.push(a[0] + t * (b[0] - a[0]));
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut i = 0;
        while i + 1 < xs.len() {
            let (x0, x1) = (xs[i], xs[i + 1]);
            if x1 - x0 > 1e-6 {
                out.push([unrot([x0, v]), unrot([x1, v])]);
            }
            i += 2;
        }
        v += gap_mm;
    }
    out
}

/// 판 안의 닫힌 단면 폴리곤에 해칭 레이어를 추가한다(`간격 기본 1.2mm`).
pub fn append_section_hatching(plate: &mut ComposedPlate2D, gap_mm: f64) {
    let mut added = Vec::new();
    for elem in &plate.paths {
        // 세로 단면 절단면만 해칭한다(수평 횡단면·평면도 윤곽선은 제외).
        let is_section = (elem.layer_id.contains("단면") || elem.layer_id.contains("내벽"))
            && !elem.layer_id.contains("횡단면");
        if !is_section || !elem.closed || elem.points_mm.len() < 3 {
            continue;
        }
        for seg in hatch_polygon_45(&elem.points_mm, gap_mm) {
            added.push(PlatePathElement {
                layer_id: String::from("단면_해칭"),
                points_mm: vec![seg[0], seg[1]],
                closed: false,
                stroke_hex: String::from("#5E5D59"),
                stroke_width_mm: 0.12,
                dash_pattern_mm: None,
                fill_hex: None,
            });
        }
    }
    plate.paths.extend(added);
}

/// 판 하단 정면 입·단면도 중심축(`cx`) 아래에 **유물번호 + 캡션** 블록을 넣는다(「유물 편집판.ai」 템플릿 규격 · `D1` 우하단 수평 횡단면과 비충돌).
pub fn append_artifact_label_block(plate: &mut ComposedPlate2D, number: &str, caption: &str) {
    let cx = plate
        .paths
        .iter()
        .find(|p| p.layer_id == "중심_기준선" && !p.points_mm.is_empty())
        .map(|p| p.points_mm[0][0])
        .or_else(|| {
            plate
                .raster_regions
                .first()
                .map(|r| 0.5 * (r.rect_mm[0] + r.rect_mm[2]))
        })
        .unwrap_or(plate.canvas_width_mm * 0.5);
    let label_y = 8.5_f64;
    if !number.trim().is_empty() {
        plate.callouts.push(PlateTextCallout {
            layer_id: String::from("유물번호"),
            pos_mm: [cx - 12.0, label_y],
            text: number.trim().to_string(),
            font_size_pt: 10.5,
            color_hex: String::from("#141413"),
        });
    }
    if !caption.trim().is_empty() {
        plate.callouts.push(PlateTextCallout {
            layer_id: String::from("유물캡션"),
            pos_mm: [cx - 20.0, (label_y - 4.0).max(4.0)],
            text: caption.trim().to_string(),
            font_size_pt: 8.5,
            color_hex: String::from("#141413"),
        });
    }
}

/// 유구(Pit/Feature) 도면 전용 방위표(북화살)를 우상단 여백에 추가한다 (`A4`: 이동식 출토 유물 도판에서는 기본 호출하지 않음).
pub fn append_site_north_arrow(plate: &mut ComposedPlate2D) {
    let margin = 18.0;
    let anx = plate.canvas_width_mm - margin - 2.5;
    let an_y0 = plate.canvas_height_mm - margin - 10.6;
    let apex = [anx, an_y0 + NORTH_ARROW_HEIGHT_MM];
    let inner = [anx, an_y0 + NORTH_ARROW_INNER_Y_MM];
    plate.paths.push(PlatePathElement {
        layer_id: String::from("방위_북화살"),
        points_mm: vec![[anx - NORTH_ARROW_HALF_WIDTH_MM, an_y0], apex, inner],
        closed: true,
        stroke_hex: String::from("#141413"),
        stroke_width_mm: 0.11,
        dash_pattern_mm: None,
        fill_hex: Some(String::from("#FFFFFF")),
    });
    plate.paths.push(PlatePathElement {
        layer_id: String::from("방위_북화살"),
        points_mm: vec![[anx + NORTH_ARROW_HALF_WIDTH_MM, an_y0], apex, inner],
        closed: true,
        stroke_hex: String::from("#141413"),
        stroke_width_mm: 0.11,
        dash_pattern_mm: None,
        fill_hex: Some(String::from("#141413")),
    });
    let n_rel: [[f64; 2]; 9] = [
        [-0.399, 0.536],
        [-0.404, 1.860],
        [-0.730, 1.860],
        [-0.730, 0.0],
        [-0.342, 0.0],
        [0.406, 1.324],
        [0.406, 0.0],
        [0.730, 0.0],
        [0.730, 1.860],
    ];
    let n_y0 = an_y0 + NORTH_ARROW_HEIGHT_MM + 0.6;
    let n_pts: Vec<[f64; 2]> = n_rel.iter().map(|p| [anx + p[0], n_y0 + p[1]]).collect();
    plate.paths.push(PlatePathElement {
        layer_id: String::from("방위_북화살"),
        points_mm: n_pts,
        closed: true,
        stroke_hex: String::from("#141413"),
        stroke_width_mm: 0.06,
        dash_pattern_mm: None,
        fill_hex: Some(String::from("#141413")),
    });
}

/// 실측치(`ArtifactMeasurementReport`)와 단면 윤곽(`PlaneSliceResult`)으로부터 8대 유물군 맞춤형 2D 도판을 합성한다.
/// 실측 축척 막대 바디 높이 (mm) — 「동국문화재 축척 및 방위표.ai」 실측 3.314pt = 1.17mm.
pub const SCALE_BAR_HEIGHT_MM: f64 = 1.17;
/// 축척 막대 기선(베이스라인)이 바디 상단에서 떨어진 간격 (mm) — 실측 2.906pt = 1.03mm.
pub const SCALE_BAR_BASELINE_GAP_MM: f64 = 1.03;
/// 축척 막대 10등분 눈금 높이 (mm, 기선 기준 상방) — E:\ 3차 실측 확정.
/// 민눈금 1.17mm · 양끝 2.39mm · 중앙 2.69mm.
/// (구 주석의 "민눈금 3.315pt"는 오기 — 3.315pt는 바디 높이 1.17mm이다.)
pub const SCALE_BAR_TICK_MINOR_MM: f64 = 1.17;
pub const SCALE_BAR_TICK_END_MM: f64 = 2.39;
pub const SCALE_BAR_TICK_MID_MM: f64 = 2.69;
/// 방위표(북화살) 연꼴 밑변 반폭·높이·중앙 꺾임점 (mm) — 「동국문화재 축척 및 방위표.ai」 실측
/// (밑변 11.34pt = 4.0mm, 높이 22.2pt = 7.83mm, 꺾임 3.93pt = 1.39mm).
pub const NORTH_ARROW_HALF_WIDTH_MM: f64 = 2.0;
pub const NORTH_ARROW_HEIGHT_MM: f64 = 7.83;
pub const NORTH_ARROW_INNER_Y_MM: f64 = 1.39;
/// 목질흔 나뭇결 세선 굵기 (mm) — 「목질흔.ai」 실측 0.15 (STANDARD_LINE_WEIGHTS_MM 규격값).
pub const WOOD_GRAIN_LINE_WIDTH_MM: f64 = 0.15;
/// 목질흔 결선 획수 — 「목질흔.ai」 실측 띠(수평 결선 2개 밴드) 재현용.
pub const WOOD_GRAIN_STROKE_COUNT: usize = 16;

/// 축척 막대 중간 라벨(절반값, 단위 없음) — 샘플 표기("20cm(1/6)" 막대의 "10")와 같은 규칙.
/// 막대 길이 50mm 이상이면 cm 단위, 그 미만이면 mm 단위로 절반값을 표기한다.
pub fn scale_bar_mid_label(bar_len_mm: f64) -> String {
    let half = if bar_len_mm >= 50.0 {
        bar_len_mm / 20.0
    } else {
        bar_len_mm / 2.0
    };
    let s = format!("{half:.1}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn compose_archaeological_plate(input: &PlateComposeInput<'_>) -> ComposedPlate2D {
    compose_archaeological_plate_inner(input, None)
}

/// 사용자가 지정한 도판 모드로 합성한다(`PlateLayoutMode` 8종 UI 노출용).
pub fn compose_archaeological_plate_with_mode(
    input: &PlateComposeInput<'_>,
    mode: PlateLayoutMode,
) -> ComposedPlate2D {
    compose_archaeological_plate_inner(input, Some(mode))
}

fn compose_archaeological_plate_inner(
    input: &PlateComposeInput<'_>,
    mode_override: Option<PlateLayoutMode>,
) -> ComposedPlate2D {
    let m = input.measurements;
    let mode = mode_override.unwrap_or_else(|| {
        PlateLayoutMode::select_for_artifact(
            m.category,
            input.sherd_subtype,
            input.sherd_strategy,
            input.bead_subtype,
        )
    });

    let max_d = if m.category == ArtifactCategory::Potsherd
        && mode == PlateLayoutMode::BodyOrMicroSherdThreeColumn
    {
        m.max_belly_diameter_mm.max(20.0)
    } else {
        m.max_belly_diameter_mm
            .max(m.rim_or_outer_diameter_mm)
            .max(20.0)
    };
    let margin = 18.0;

    // 수직 단면 폴리라인의 실제 Z 최소·최대값을 구하여 회전 시에도 절대 잘리지(clamp) 않고 1:1 그대로 표현되도록 산출
    let z_min_slice = input
        .vertical_section
        .polylines_2d
        .iter()
        .flatten()
        .map(|p| p[1])
        .fold(f64::INFINITY, f64::min);
    let z_max_slice = input
        .vertical_section
        .polylines_2d
        .iter()
        .flatten()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let nominal_h = m.height_or_length_mm.max(10.0);
    let slice_span_z = if z_min_slice.is_finite() && z_max_slice.is_finite() {
        (z_max_slice - z_min_slice).max(0.0)
    } else {
        0.0
    };
    let z_offset = if z_min_slice.is_finite() {
        z_min_slice
    } else {
        0.0
    };
    let height = if slice_span_z > nominal_h * 1.15 {
        slice_span_z
    } else {
        nominal_h
    };
    let z_scale = if slice_span_z > 2.0 && slice_span_z <= nominal_h * 1.15 {
        height / slice_span_z
    } else {
        1.0
    };

    let has_custom_sec = input
        .horizontal_sections
        .iter()
        .any(|s| s.orientation == strata_section::CutPlaneOrientation::VerticalLongitudinal);

    let (mut canvas_width_mm, canvas_height_mm) = match mode {
        PlateLayoutMode::CompleteVesselStandard | PlateLayoutMode::RimOrBaseSherdReconstructed => (
            (max_d * 2.58 + margin * 2.0).max(236.0),
            (height + max_d + 80.0).max(246.0),
        ),
        PlateLayoutMode::BodyOrMicroSherdThreeColumn => (
            (max_d * 2.58 + margin * 2.0).max(236.0),
            (height + max_d + 80.0).max(246.0),
        ),
        PlateLayoutMode::BeadJadeMicroPlate | PlateLayoutMode::CommaJadeCurvedPlate => (
            (max_d * 3.6 + margin * 2.2).max(72.0),
            (height * 3.3 + margin * 2.6).max(78.0),
        ),
        _ => (
            (max_d * 2.45 + margin * 2.2).max(172.0),
            (height + max_d * 1.15 + margin * 3.6).max(188.0),
        ),
    };
    if has_custom_sec {
        canvas_width_mm += max_d * 1.05 + 42.0;
    }

    let mut paths = Vec::new();
    let mut callouts = Vec::new();
    let mut raster_regions = Vec::new();

    // 완형 토기 및 편조각 도판은 좌측 열(제1열)에 평·단면도를 두고 우측 열(제2열)에 1:1 정밀 탁본 및 수평 횡단면을 충분한 간격으로 배치
    let cx = match mode {
        PlateLayoutMode::CompleteVesselStandard
        | PlateLayoutMode::RimOrBaseSherdReconstructed
        | PlateLayoutMode::BodyOrMicroSherdThreeColumn => margin + max_d * 0.56 + 6.0,
        _ => canvas_width_mm * 0.32,
    };
    let base_y = 35.0;
    let top_y = base_y + height;

    // 실제 단면 폴리라인 상단/하단 폭을 측정하여 유물을 회전·뒤집었을 때 여부를 판별
    let mut measured_top_half_x = 0.0_f64;
    let mut measured_bot_half_x = 0.0_f64;
    let mut measured_max_abs_x = 0.0_f64;
    for pt in input.vertical_section.polylines_2d.iter().flatten() {
        let rel_z = pt[1] - z_offset;
        measured_max_abs_x = measured_max_abs_x.max(pt[0].abs());
        if rel_z >= height * 0.82 {
            measured_top_half_x = measured_top_half_x.max(pt[0].abs());
        }
        if rel_z <= height * 0.18 {
            measured_bot_half_x = measured_bot_half_x.max(pt[0].abs());
        }
    }
    let is_upside_down =
        measured_top_half_x > 4.0 && measured_bot_half_x > measured_top_half_x * 1.15;
    let is_rotated_ring_section = slice_span_z > m.height_or_length_mm.max(10.0) * 1.18
        || (measured_max_abs_x > 10.0
            && measured_top_half_x < measured_max_abs_x * 0.62
            && measured_bot_half_x < measured_max_abs_x * 0.62);

    // 1. 중심 회전축 / 기준선 제도 (`#141413` 먹색 0.25mm)
    if mode != PlateLayoutMode::BodyOrMicroSherdThreeColumn {
        paths.push(PlatePathElement {
            layer_id: String::from("중심_기준선"),
            points_mm: vec![[cx, base_y - 2.0], [cx, top_y + 2.0]],
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.25,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }

    // 2. 구경선 / 저경선 (수평 브리지 연결선 — 리포트 단일원천 폭 유지)
    let is_estimated_dashed = matches!(
        input.sherd_strategy,
        Some(SherdPlateStrategy::EstimatedDashedReconstruction)
    );
    if m.rim_or_outer_diameter_mm > 0.0 && mode != PlateLayoutMode::BodyOrMicroSherdThreeColumn {
        let half_top = m.rim_or_outer_diameter_mm * 0.5;
        paths.push(PlatePathElement {
            layer_id: if is_estimated_dashed {
                String::from("추정복원_파선")
            } else {
                String::from("구경_기준선")
            },
            points_mm: vec![[cx - half_top, top_y], [cx + half_top, top_y]],
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.32,
            dash_pattern_mm: if is_estimated_dashed {
                Some((2.0, 1.5))
            } else {
                None
            },
            fill_hex: None,
        });
    }

    if m.base_diameter_or_width_mm > 0.0 && mode != PlateLayoutMode::BodyOrMicroSherdThreeColumn {
        let half_bot = m.base_diameter_or_width_mm * 0.5;
        paths.push(PlatePathElement {
            layer_id: String::from("저경_기준선"),
            points_mm: vec![[cx - half_bot, base_y], [cx + half_bot, base_y]],
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.32,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }

    // 3. 수직 단면 `#141413` 폴리곤 및 반대편 외면 입면선 배치 (모든 회전 각도·모든 유물 1:1 무왜곡 보존)
    let section_sign = if input.flip_exterior_right { -1.0 } else { 1.0 };
    let keep_positive_x = section_sign > 0.0;
    let wall_t = m.mean_wall_thickness_mm.clamp(2.2, 12.0);

    let fallback_polylines: Vec<Vec<[f64; 2]>> = if input
        .vertical_section
        .polylines_2d
        .iter()
        .all(|p| p.len() < 2)
    {
        let r_base = (m.base_diameter_or_width_mm * 0.5).max(max_d * 0.32);
        let r_belly = (m.max_belly_diameter_mm * 0.5).max(r_base + 4.0);
        let r_rim = (m.rim_or_outer_diameter_mm * 0.5).max(r_base + 2.0);
        vec![vec![
            [0.0, 0.0],
            [r_base, 0.0],
            [r_belly, height * 0.55],
            [r_rim, height],
            [(r_rim - wall_t).max(0.0), height],
            [(r_belly - wall_t).max(0.0), height * 0.55],
            [(r_base - wall_t).max(0.0), wall_t],
            [0.0, wall_t],
        ]]
    } else {
        Vec::new()
    };
    let active_polylines = if fallback_polylines.is_empty() {
        &input.vertical_section.polylines_2d
    } else {
        &fallback_polylines
    };

    let norm_polys: Vec<Vec<[f64; 2]>> = active_polylines
        .iter()
        .filter(|p| p.len() >= 2)
        .map(|poly| {
            poly.iter()
                .map(|pt| [pt[0], ((pt[1] - z_offset) * z_scale).clamp(0.0, height)])
                .collect()
        })
        .collect();

    let all_min_x = norm_polys
        .iter()
        .flatten()
        .map(|p| p[0])
        .fold(f64::INFINITY, f64::min);
    let all_max_x = norm_polys
        .iter()
        .flatten()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max);
    let global_spans_both = all_min_x < -2.0 && all_max_x > 2.0;

    // 90° 앞으로 뒤집기 등 유물 회전 시 절단면이 외벽 고리(outer loop) + 내벽 고리(inner loop) 2개의 동심원 루프로 추출된 경우:
    // 중공(빈 공간)을 가로지르는 수직선이나 내부 전체 해칭이 생기지 않도록 외벽 반호와 내벽 반호를 이어 C자형 기벽 단면 다각형으로 합성한다!
    let full_loops: Vec<&Vec<[f64; 2]>> = norm_polys
        .iter()
        .filter(|np| {
            let min_x = np.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
            let max_x = np.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
            min_x < -4.0 && max_x > 4.0 && np.len() >= 6
        })
        .collect();

    if mode == PlateLayoutMode::CompleteVesselStandard && full_loops.len() >= 2 {
        let mut sorted_loops = full_loops;
        sorted_loops.sort_by(|a, b| {
            let span_a = a.iter().map(|p| p[0].abs()).fold(0.0_f64, f64::max);
            let span_b = b.iter().map(|p| p[0].abs()).fold(0.0_f64, f64::max);
            span_b
                .partial_cmp(&span_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let outer_loop = sorted_loops[0];
        let inner_loop = sorted_loops[1];
        let out_closed = {
            let f = outer_loop[0];
            let l = *outer_loop.last().unwrap_or(&f);
            (f[0] - l[0]).hypot(f[1] - l[1]) <= 6.0
        };
        let in_closed = {
            let f = inner_loop[0];
            let l = *inner_loop.last().unwrap_or(&f);
            (f[0] - l[0]).hypot(f[1] - l[1]) <= 6.0
        };

        let out_sec_chains =
            clip_polyline_half_plane_x_chains(outer_loop, out_closed, keep_positive_x);
        let in_sec_chains =
            clip_polyline_half_plane_x_chains(inner_loop, in_closed, keep_positive_x);

        if let (Some(out_arc), Some(in_arc)) = (
            out_sec_chains.iter().max_by_key(|c| c.len()),
            in_sec_chains.iter().max_by_key(|c| c.len()),
        ) {
            let mut ring_wall: Vec<[f64; 2]> = out_arc
                .iter()
                .map(|pt| [cx + pt[0], base_y + pt[1]])
                .collect();
            let out_end = *out_arc.last().unwrap_or(&out_arc[0]);
            let in_first = in_arc[0];
            let in_last = *in_arc.last().unwrap_or(&in_first);
            let d_first = (out_end[0] - in_first[0]).hypot(out_end[1] - in_first[1]);
            let d_last = (out_end[0] - in_last[0]).hypot(out_end[1] - in_last[1]);
            if d_first <= d_last {
                for pt in in_arc.iter() {
                    ring_wall.push([cx + pt[0], base_y + pt[1]]);
                }
            } else {
                for pt in in_arc.iter().rev() {
                    ring_wall.push([cx + pt[0], base_y + pt[1]]);
                }
            }
            if ring_wall.len() >= 4 {
                paths.push(PlatePathElement {
                    layer_id: String::from("단면_실선"),
                    points_mm: ring_wall,
                    closed: true,
                    stroke_hex: String::from("#141413"),
                    stroke_width_mm: 0.38,
                    dash_pattern_mm: None,
                    fill_hex: Some(String::from("#141413")),
                });
            }
        }

        // A1: 외면 쪽은 내벽(inner_loop)을 제외하고 최외곽 외벽 실루엣(outer_loop) 1줄만 추출한다.
        let raw_ext_chains =
            clip_polyline_half_plane_x_chains(outer_loop, out_closed, !keep_positive_x);
        let single_ext =
            extract_single_outer_silhouette_contour(&raw_ext_chains, height, keep_positive_x);
        if single_ext.len() >= 2 {
            let mapped_ext: Vec<[f64; 2]> = single_ext
                .iter()
                .map(|pt| [cx + pt[0], base_y + pt[1]])
                .collect();
            paths.push(PlatePathElement {
                layer_id: String::from("외곽선_실선"),
                points_mm: mapped_ext,
                closed: false,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.38,
                dash_pattern_mm: None,
                fill_hex: None,
            });
        }
    } else {
        let mut all_ext_chains_for_silhouette: Vec<Vec<[f64; 2]>> = Vec::new();
        for norm_poly in &norm_polys {
            let min_x = norm_poly.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
            let max_x = norm_poly
                .iter()
                .map(|p| p[0])
                .fold(f64::NEG_INFINITY, f64::max);
            let spans_both_halves = min_x < -4.0 && max_x > 4.0 && norm_poly.len() >= 6;

            if mode != PlateLayoutMode::CompleteVesselStandard {
                // Task A-2(2) & A8: 유물편(Potsherd 등 비완형)의 수직 단면은 절대로 x = 0에서 반토막 클리핑하거나 가로로 찌그러뜨리지 않는다!
                // 3D 정면 뷰의 적색 절단 평면이 기벽을 자른 실제 1:1 닫힌 단면 루프 전체를 온전히 보존하여 제도한다.
                let x_mid = 0.5 * (min_x + max_x);
                let mut closed_sec: Vec<[f64; 2]> = norm_poly
                    .iter()
                    .map(|p| [cx + (p[0] - x_mid), base_y + p[1]])
                    .collect();
                let first = closed_sec[0];
                let last = *closed_sec.last().unwrap_or(&first);
                if (first[0] - last[0]).hypot(first[1] - last[1]) > 6.0 {
                    let span_x = (max_x - min_x).abs();
                    let span_z = norm_poly.iter().map(|p| p[1]).fold(0.0_f64, f64::max)
                        - norm_poly.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
                    let mut inner_rev: Vec<[f64; 2]> = closed_sec
                        .iter()
                        .rev()
                        .map(|p| {
                            if span_x >= span_z {
                                [p[0], (p[1] - wall_t).max(base_y)]
                            } else {
                                [p[0] + wall_t * section_sign, p[1]]
                            }
                        })
                        .collect();
                    closed_sec.append(&mut inner_rev);
                }
                paths.push(PlatePathElement {
                    layer_id: String::from("외곽선_실선"),
                    points_mm: closed_sec.clone(),
                    closed: true,
                    stroke_hex: String::from("#141413"),
                    stroke_width_mm: 0.36,
                    dash_pattern_mm: None,
                    fill_hex: None,
                });
                paths.push(PlatePathElement {
                    layer_id: String::from("단면_실선"),
                    points_mm: closed_sec,
                    closed: true,
                    stroke_hex: String::from("#141413"),
                    stroke_width_mm: 0.36,
                    dash_pattern_mm: None,
                    fill_hex: Some(String::from("#141413")),
                });
            } else if spans_both_halves {
                let first_pt = norm_poly[0];
                let last_pt = *norm_poly.last().unwrap_or(&first_pt);
                let is_closed_loop =
                    (first_pt[0] - last_pt[0]).hypot(first_pt[1] - last_pt[1]) <= 6.0;

                let sec_chains =
                    clip_polyline_half_plane_x_chains(norm_poly, is_closed_loop, keep_positive_x);
                for chain in sec_chains {
                    if chain.len() < 2 {
                        continue;
                    }
                    let cf = chain[0];
                    let cl = *chain.last().unwrap_or(&cf);
                    let endpoints_form_base_wall = cf[0].abs() <= 1.5
                        && cl[0].abs() <= 1.5
                        && (cf[1] - cl[1]).abs() <= wall_t * 4.0;
                    let closed_local = if endpoints_form_base_wall && chain.len() >= 3 {
                        chain
                    } else {
                        offset_chain_inward_to_closed_wall(&chain, height * 0.5, wall_t)
                    };
                    if closed_local.len() >= 3 {
                        let mapped_sec: Vec<[f64; 2]> = closed_local
                            .into_iter()
                            .map(|pt| [cx + pt[0], base_y + pt[1]])
                            .collect();
                        paths.push(PlatePathElement {
                            layer_id: String::from("단면_실선"),
                            points_mm: mapped_sec,
                            closed: true,
                            stroke_hex: String::from("#141413"),
                            stroke_width_mm: 0.38,
                            dash_pattern_mm: None,
                            fill_hex: Some(String::from("#141413")),
                        });
                    }
                }

                let ext_chains =
                    clip_polyline_half_plane_x_chains(norm_poly, is_closed_loop, !keep_positive_x);
                all_ext_chains_for_silhouette.extend(ext_chains);
            } else if global_spans_both {
                let avg_x = norm_poly.iter().map(|p| p[0]).sum::<f64>() / (norm_poly.len() as f64);
                let is_on_section_side = avg_x * section_sign >= 0.0;
                if is_on_section_side {
                    let mut sec_pts: Vec<[f64; 2]> = norm_poly
                        .iter()
                        .map(|pt| [cx + pt[0], base_y + pt[1]])
                        .collect();
                    let first = sec_pts[0];
                    let last = *sec_pts.last().unwrap_or(&first);
                    if (first[0] - last[0]).hypot(first[1] - last[1]) > 2.5 {
                        let closed_local =
                            offset_chain_inward_to_closed_wall(norm_poly, height * 0.5, wall_t);
                        sec_pts = closed_local
                            .into_iter()
                            .map(|pt| [cx + pt[0], base_y + pt[1]])
                            .collect();
                    }
                    paths.push(PlatePathElement {
                        layer_id: String::from("단면_실선"),
                        points_mm: sec_pts,
                        closed: true,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: 0.38,
                        dash_pattern_mm: None,
                        fill_hex: Some(String::from("#141413")),
                    });
                } else {
                    all_ext_chains_for_silhouette.push(norm_poly.clone());
                }
            } else {
                let mut sec_pts: Vec<[f64; 2]> = norm_poly
                    .iter()
                    .map(|pt| [cx + pt[0].abs() * section_sign, base_y + pt[1]])
                    .collect();
                let first = sec_pts[0];
                let last = *sec_pts.last().unwrap_or(&first);
                if (first[0] - last[0]).hypot(first[1] - last[1]) > 2.5 {
                    let mut inner_pts: Vec<[f64; 2]> = norm_poly
                        .iter()
                        .rev()
                        .map(|pt| {
                            let r_in = (pt[0].abs() - wall_t).max(0.0);
                            [cx + r_in * section_sign, base_y + pt[1]]
                        })
                        .collect();
                    sec_pts.append(&mut inner_pts);
                }
                paths.push(PlatePathElement {
                    layer_id: String::from("단면_실선"),
                    points_mm: sec_pts,
                    closed: true,
                    stroke_hex: String::from("#141413"),
                    stroke_width_mm: 0.38,
                    dash_pattern_mm: None,
                    fill_hex: Some(String::from("#141413")),
                });

                let ext_local: Vec<[f64; 2]> = norm_poly
                    .iter()
                    .map(|pt| [-pt[0].abs() * section_sign, pt[1]])
                    .collect();
                all_ext_chains_for_silhouette.push(ext_local);
            }
        }

        // A1 & A4: 내벽 정점을 제거한 최외곽 외벽 실루엣(Outer Contour) 1줄만 추출하고,
        // 고정 비율([0.93, 0.32, 0.09]) 대신 실제 외벽 곡률 변곡점에서만 수평 입면 구조선을 제도한다.
        if !all_ext_chains_for_silhouette.is_empty() {
            let single_ext = extract_single_outer_silhouette_contour(
                &all_ext_chains_for_silhouette,
                height,
                keep_positive_x,
            );
            if single_ext.len() >= 2 {
                let mapped_ext: Vec<[f64; 2]> = single_ext
                    .iter()
                    .map(|pt| [cx + pt[0], base_y + pt[1]])
                    .collect();
                paths.push(PlatePathElement {
                    layer_id: String::from("외곽선_실선"),
                    points_mm: mapped_ext,
                    closed: false,
                    stroke_hex: String::from("#141413"),
                    stroke_width_mm: 0.38,
                    dash_pattern_mm: None,
                    fill_hex: None,
                });

                if !is_rotated_ring_section {
                    for infl_pt in detect_outer_curvature_inflection_points(&single_ext, height) {
                        paths.push(PlatePathElement {
                            layer_id: String::from("외곽선_입면구조선"),
                            points_mm: vec![
                                [cx + infl_pt[0], base_y + infl_pt[1]],
                                [cx, base_y + infl_pt[1]],
                            ],
                            closed: false,
                            stroke_hex: String::from("#141413"),
                            stroke_width_mm: 0.24,
                            dash_pattern_mm: None,
                            fill_hex: None,
                        });
                    }
                }
            }
        }
    }

    // 4. 유물군별 다중 뷰 정밀 탁본(`Rubbing`) 래스터 영역 및 특화 도판 요소 배치
    let half_d = max_d * 0.5;
    let hsec_opt = input.horizontal_sections.iter().find(|s| {
        s.orientation == strata_section::CutPlaneOrientation::HorizontalTransverse
            && s.offset_mm > -9000.0
    });
    let top_plan_opt = input.horizontal_sections.iter().find(|s| {
        s.orientation == strata_section::CutPlaneOrientation::HorizontalTransverse
            && s.offset_mm <= -9000.0
    });
    let custom_sec_opt = input
        .horizontal_sections
        .iter()
        .find(|s| s.orientation == strata_section::CutPlaneOrientation::VerticalLongitudinal);

    match mode {
        PlateLayoutMode::CompleteVesselStandard => {
            // (a) 좌하단(제1열 하단): 1:1 정면 입·단면도 기준 영역 (Image 0: 일러스트 편집용 평·단면 벡터 선화 영역)
            let ext_sign = if input.flip_exterior_right { 1i8 } else { -1i8 };
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::FrontExteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: Some(ext_sign),
                caption: String::from("1:1 정면 단면·입면도"),
            });

            // (b) 좌상단(제1열 상단, Image 0): 1:1 평면도 (실제 3D 직상방 외곽·내경 실선 + 깨진 부분 복원 점선 + 파손 실선)
            // A5 & D1: 평면도와 단면·입면도의 중심축(cx)을 수직 잠금하고, 수직 간격을 34mm로 확보해 구경 치수선과 겹치지 않게 한다.
            let plan_r = half_d;
            let cx_plan = cx;
            let cy_plan = top_y + plan_r + 34.0;
            let cx_right = cx + half_d + plan_r + 28.0;
            let base_r = (m.base_diameter_or_width_mm * 0.5).clamp(plan_r * 0.34, plan_r * 0.72);

            append_measured_top_plan_view_paths(
                &mut paths,
                cx_plan,
                cy_plan,
                plan_r,
                base_r,
                wall_t,
                is_upside_down,
                top_plan_opt,
                hsec_opt,
            );
            callouts.push(PlateTextCallout {
                layer_id: String::from("평면도_라벨"),
                pos_mm: [cx_plan - 24.0, cy_plan + plan_r + 2.8],
                text: format!("1:1 평면도 (Ø{:.1}mm)", max_d),
                font_size_pt: 8.5,
                color_hex: String::from("#141413"),
            });

            // (c) 우상단(제2열 상단, Image 1: "탁본은 여기에 1:1로 여기에 나타나게하라"): 1:1 정밀 탁본 전용 배치!
            let cy_rub = cy_plan;
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::BottomPlanRubbing,
                rect_mm: [
                    cx_right - plan_r,
                    cy_rub - plan_r,
                    cx_right + plan_r,
                    cy_rub + plan_r,
                ],
                clip_half_sign: None,
                caption: String::from("1:1 정밀 탁본"),
            });
            paths.push(PlatePathElement {
                layer_id: String::from("탁본_십자선"),
                points_mm: vec![
                    [cx_right - plan_r * 1.05, cy_rub],
                    [cx_right + plan_r * 1.05, cy_rub],
                ],
                closed: false,
                stroke_hex: String::from("#5E5D59"),
                stroke_width_mm: 0.18,
                dash_pattern_mm: None,
                fill_hex: None,
            });
            paths.push(PlatePathElement {
                layer_id: String::from("탁본_십자선"),
                points_mm: vec![
                    [cx_right, cy_rub - plan_r * 1.05],
                    [cx_right, cy_rub + plan_r * 1.05],
                ],
                closed: false,
                stroke_hex: String::from("#5E5D59"),
                stroke_width_mm: 0.18,
                dash_pattern_mm: None,
                fill_hex: None,
            });
            callouts.push(PlateTextCallout {
                layer_id: String::from("탁본_라벨"),
                pos_mm: [cx_right - 24.0, cy_rub + plan_r + 2.8],
                text: format!("1:1 정밀 탁본 (Ø{:.1}mm)", max_d),
                font_size_pt: 8.5,
                color_hex: String::from("#141413"),
            });

            // (d) 우하단(제2열 하단, Image 2): 상단 1:1 정밀 탁본 및 좌측 평·단면도와 절대 겹치지 않는 독립 영역에 배치!
            let rub_bottom = cy_rub - plan_r;
            let cx_hsec = cx_right;
            if let Some(hsec) = hsec_opt {
                let cut_z_rel = (hsec.offset_mm - z_offset).clamp(1.0, height - 1.0);
                let cut_line_y = base_y + cut_z_rel;
                paths.push(PlatePathElement {
                    layer_id: String::from("횡단면_절단위치선"),
                    points_mm: vec![
                        [cx - half_d * 1.04, cut_line_y],
                        [cx + half_d * 1.08, cut_line_y],
                    ],
                    closed: false,
                    stroke_hex: String::from("#D92B2B"),
                    stroke_width_mm: 0.32,
                    dash_pattern_mm: Some((2.2, 1.2)),
                    fill_hex: None,
                });

                let (outer_contour, inner_contour, right_half_poly, max_r) =
                    extract_measured_horizontal_section_contours(hsec, wall_t, half_d * 0.78);
                let cy_hsec_default = base_y + height * 0.38;
                let stacked_cy = rub_bottom - max_r - 18.0;
                let cy_hsec = cy_hsec_default.min(stacked_cy).max(max_r + 10.0);

                paths.push(PlatePathElement {
                    layer_id: String::from("횡단면_십자선"),
                    points_mm: vec![
                        [cx_hsec - max_r * 1.12, cy_hsec],
                        [cx_hsec + max_r * 1.12, cy_hsec],
                    ],
                    closed: false,
                    stroke_hex: String::from("#5E5D59"),
                    stroke_width_mm: 0.18,
                    dash_pattern_mm: None,
                    fill_hex: None,
                });
                paths.push(PlatePathElement {
                    layer_id: String::from("횡단면_십자선"),
                    points_mm: vec![
                        [cx_hsec, cy_hsec - max_r * 1.12],
                        [cx_hsec, cy_hsec + max_r * 1.12],
                    ],
                    closed: false,
                    stroke_hex: String::from("#5E5D59"),
                    stroke_width_mm: 0.18,
                    dash_pattern_mm: None,
                    fill_hex: None,
                });

                let is_full_closed_ring = outer_contour.len() >= 54;
                if is_full_closed_ring {
                    for (contour_pts, layer_name) in [
                        (&outer_contour, "횡단면_외벽윤곽선"),
                        (&inner_contour, "횡단면_내벽윤곽선"),
                    ] {
                        if contour_pts.len() >= 3 {
                            let ring: Vec<[f64; 2]> = contour_pts
                                .iter()
                                .map(|p| [cx_hsec + p[0], cy_hsec + p[1]])
                                .collect();
                            paths.push(PlatePathElement {
                                layer_id: String::from(layer_name),
                                points_mm: ring,
                                closed: true,
                                stroke_hex: String::from("#141413"),
                                stroke_width_mm: 0.34,
                                dash_pattern_mm: None,
                                fill_hex: None,
                            });
                        }
                    }
                    if right_half_poly.len() >= 6 {
                        let mapped_rpoly: Vec<[f64; 2]> = right_half_poly
                            .into_iter()
                            .map(|p| [cx_hsec + p[0], cy_hsec + p[1]])
                            .collect();
                        paths.push(PlatePathElement {
                            layer_id: String::from("횡단면_단면_실선"),
                            points_mm: mapped_rpoly,
                            closed: true,
                            stroke_hex: String::from("#141413"),
                            stroke_width_mm: 0.36,
                            dash_pattern_mm: None,
                            fill_hex: Some(String::from("#141413")),
                        });
                    }
                } else {
                    for (pi, raw_poly) in hsec.polylines_2d.iter().enumerate() {
                        if raw_poly.len() >= 3 {
                            let f = raw_poly[0];
                            let l = *raw_poly.last().unwrap_or(&f);
                            let is_closed = (f[0] - l[0]).hypot(f[1] - l[1]) <= 4.0;
                            let pts: Vec<[f64; 2]> = raw_poly
                                .iter()
                                .map(|p| [cx_hsec + p[0], cy_hsec + p[1]])
                                .collect();
                            paths.push(PlatePathElement {
                                layer_id: String::from(if pi == 0 {
                                    "횡단면_단면_실선"
                                } else {
                                    "횡단면_외벽윤곽선"
                                }),
                                points_mm: pts,
                                closed: is_closed,
                                stroke_hex: String::from("#141413"),
                                stroke_width_mm: 0.35,
                                dash_pattern_mm: None,
                                fill_hex: if is_closed {
                                    Some(String::from("#141413"))
                                } else {
                                    None
                                },
                            });
                        }
                    }
                }

                callouts.push(PlateTextCallout {
                    layer_id: String::from("횡단면_라벨"),
                    pos_mm: [cx_hsec - max_r * 0.85, cy_hsec + max_r + 2.5],
                    text: format!("1:1 수평 횡단면 (Z={:.1}mm · 실측단면)", cut_z_rel),
                    font_size_pt: 8.5,
                    color_hex: String::from("#B5573A"),
                });
            }
        }
        PlateLayoutMode::RimOrBaseSherdReconstructed => {
            let ext_sign = if input.flip_exterior_right { 1i8 } else { -1i8 };
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::FrontExteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: Some(ext_sign),
                caption: String::from("편조각 외면 정밀 탁본"),
            });
        }
        PlateLayoutMode::BodyOrMicroSherdThreeColumn => {
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::FrontExteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: None,
                caption: String::from("외면(정밀 탁본)"),
            });
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::BackInteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: None,
                caption: String::from("내면(정밀 탁본)"),
            });
        }
        PlateLayoutMode::HandleOrSpoutSherd => {
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::FrontExteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: Some(if input.flip_exterior_right { 1 } else { -1 }),
                caption: String::from("파수·주구 정밀 탁본"),
            });
            let attach_kind = input
                .handle_attachment
                .unwrap_or(HandleAttachmentKind::InsertedTenonJoint);
            let mid_y = base_y + height * 0.55;
            let wall_x = cx + max_d * 0.35 * section_sign;
            let tenon_depth = match attach_kind {
                HandleAttachmentKind::InsertedTenonJoint => -6.5 * section_sign,
                HandleAttachmentKind::SurfaceButtJoint => 0.0,
            };
            paths.push(PlatePathElement {
                layer_id: String::from("접합기법_파선"),
                points_mm: vec![
                    [wall_x, mid_y + 5.0],
                    [wall_x + tenon_depth, mid_y],
                    [wall_x, mid_y - 5.0],
                ],
                closed: false,
                stroke_hex: String::from("#B5573A"),
                stroke_width_mm: 0.25,
                dash_pattern_mm: Some((1.5, 1.0)),
                fill_hex: None,
            });
        }
        PlateLayoutMode::BeadJadeMicroPlate | PlateLayoutMode::CommaJadeCurvedPlate => {
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::FrontExteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: Some(-1),
                caption: String::from("옥류 외면 정밀 탁본"),
            });
            let plan_cy = top_y + max_d * 0.85 + 8.0;
            let r_outer = (max_d * 0.5).max(1.5);
            let (r_entry, r_waist) = if let Some(ref perf) = m.bead_perforation {
                (
                    perf.top_entry_diameter_mm * 0.5,
                    perf.min_bore_diameter_mm * 0.5,
                )
            } else {
                (r_outer * 0.45, r_outer * 0.32)
            };

            for (r, layer, dashed) in [
                (r_outer, "구슬상면_외경", None),
                (r_entry, "구슬상면_입구공경", None),
                (r_waist, "구슬상면_최소공경", Some((1.0, 0.8))),
            ] {
                let mut ring = Vec::with_capacity(36);
                for step in 0..36 {
                    let a = (step as f64) * std::f64::consts::TAU / 36.0;
                    ring.push([cx + r * a.cos(), plan_cy + r * a.sin()]);
                }
                paths.push(PlatePathElement {
                    layer_id: String::from(layer),
                    points_mm: ring,
                    closed: true,
                    stroke_hex: String::from("#141413"),
                    stroke_width_mm: 0.25,
                    dash_pattern_mm: dashed,
                    fill_hex: None,
                });
            }

            for span in input.occluded_spans {
                paths.push(PlatePathElement {
                    layer_id: String::from("천공내벽_파선"),
                    points_mm: vec![
                        [cx + span.start_pt[0], base_y + span.start_pt[1]],
                        [cx + span.end_pt[0], base_y + span.end_pt[1]],
                    ],
                    closed: false,
                    stroke_hex: String::from("#B5573A"),
                    stroke_width_mm: 0.25,
                    dash_pattern_mm: Some((1.5, 1.0)),
                    fill_hex: None,
                });
            }

            if let Some(dt) = input.bead_drill_type {
                callouts.push(PlateTextCallout {
                    layer_id: String::from("치수_축척막대"),
                    pos_mm: [cx + r_outer + 4.0, base_y + height * 0.5],
                    text: format!("천공: {}", dt.code_str()),
                    font_size_pt: 8.5,
                    color_hex: String::from("#B5573A"),
                });
            }
        }
        PlateLayoutMode::RoofTileAndEavesPlate => {
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::FrontExteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: Some(-1),
                caption: String::from("기와 타날문·포목흔 탁본"),
            });
            let ratio = input.wado_cut_ratio.unwrap_or(0.62).clamp(0.1, 0.95);
            let cut_y = base_y + height * ratio;
            paths.push(PlatePathElement {
                layer_id: String::from("와도_분할선"),
                points_mm: vec![[cx + 2.0, cut_y], [cx + 12.0, cut_y]],
                closed: false,
                stroke_hex: String::from("#B5573A"),
                stroke_width_mm: 0.30,
                dash_pattern_mm: None,
                fill_hex: None,
            });
            callouts.push(PlateTextCallout {
                layer_id: String::from("치수_축척막대"),
                pos_mm: [cx + 13.0, cut_y],
                text: format!("──▶ 와도 분할선 ({:.0}%)", ratio * 100.0),
                font_size_pt: 8.5,
                color_hex: String::from("#B5573A"),
            });
            if let Some(ref hemp) = input.hemp_density_label {
                callouts.push(PlateTextCallout {
                    layer_id: String::from("치수_축척막대"),
                    pos_mm: [cx - max_d * 0.45, base_y - 6.0],
                    text: hemp.clone(),
                    font_size_pt: 8.5,
                    color_hex: String::from("#141413"),
                });
            }
        }
        PlateLayoutMode::LithicAndCorrodedMetalPlate => {
            raster_regions.push(PlateRasterRegion {
                kind: PlateRasterKind::FrontExteriorRubbing,
                rect_mm: [cx - half_d, base_y, cx + half_d, top_y],
                clip_half_sign: Some(-1),
                caption: String::from("석기 박리·금속 표면 탁본"),
            });
            if m.category == ArtifactCategory::CorrodedMetal {
                for span in input.occluded_spans {
                    paths.push(PlatePathElement {
                        layer_id: String::from("부식팽창층_파선"),
                        points_mm: vec![
                            [cx + span.start_pt[0], base_y + span.start_pt[1]],
                            [cx + span.end_pt[0], base_y + span.end_pt[1]],
                        ],
                        closed: false,
                        stroke_hex: String::from("#736B5E"),
                        stroke_width_mm: 0.20,
                        dash_pattern_mm: Some((1.5, 1.2)),
                        fill_hex: None,
                    });
                }
                // 목질흔.ai 정본: 슴베·자루 하단 띠(`tang_wood_grain_z_ratio` = 하단 32%)에
                // 수평 나뭇결 세선(0.15mm) 16획. 실측 결선 길이 2~51mm를 결정적 분포로 재현한다.
                let tang_h = height * 0.32;
                for i in 0..WOOD_GRAIN_STROKE_COUNT {
                    let gy =
                        base_y + 2.0 + tang_h * (i as f64 + 0.5) / WOOD_GRAIN_STROKE_COUNT as f64;
                    let glen = 6.0 + ((i * 37 + 11) % 34) as f64;
                    let gx0 = cx - glen * 0.5 + ((i * 53 + 7) % 9) as f64 * 0.3 - 1.2;
                    paths.push(PlatePathElement {
                        layer_id: String::from("목질흔_결선"),
                        points_mm: vec![[gx0, gy], [gx0 + glen, gy]],
                        closed: false,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: WOOD_GRAIN_LINE_WIDTH_MM,
                        dash_pattern_mm: None,
                        fill_hex: None,
                    });
                }
            }
        }
    }

    // 4-B. 유물편(Potsherd) 및 비완형 유물 모드:
    // Task A-2(1)&(3) & A7 & A8: 유물편에는 절대로 `ri = ro - wall_t` 가짜 2중 고리 도넛이나 오른쪽 반원 채움을 생성하지 않고,
    // 좌상단에는 실제 외곽 실루엣 단 1줄(`평면도_외곽윤곽선`, 구연 복원편은 복원 원호+추정 파선),
    // 우하단에는 실제 수평 절단 기벽 두께(`wall_t`)의 닫힌 단면 띠(`횡단면_단면_실선`)만 제도한다!
    if mode != PlateLayoutMode::CompleteVesselStandard {
        let plan_r = half_d.max(12.0);
        let hsec_span_y = hsec_opt
            .map(|hs| {
                let min_y = hs
                    .polylines_2d
                    .iter()
                    .flatten()
                    .map(|p| p[1])
                    .fold(f64::INFINITY, f64::min);
                let max_y = hs
                    .polylines_2d
                    .iter()
                    .flatten()
                    .map(|p| p[1])
                    .fold(f64::NEG_INFINITY, f64::max);
                if min_y.is_finite() && max_y.is_finite() {
                    (max_y - min_y).max(wall_t * 2.0)
                } else {
                    wall_t * 2.0
                }
            })
            .unwrap_or(wall_t * 2.0);
        let cx_plan = cx;
        let cy_plan = (top_y + plan_r + 34.0).max(hsec_span_y + 34.0 + plan_r);
        let cx_right = cx + half_d + plan_r + 28.0;

        paths.push(PlatePathElement {
            layer_id: String::from("외저면_십자선"),
            points_mm: vec![
                [cx_plan - plan_r * 1.05, cy_plan],
                [cx_plan + plan_r * 1.05, cy_plan],
            ],
            closed: false,
            stroke_hex: String::from("#5E5D59"),
            stroke_width_mm: 0.18,
            dash_pattern_mm: None,
            fill_hex: None,
        });
        paths.push(PlatePathElement {
            layer_id: String::from("외저면_십자선"),
            points_mm: vec![
                [cx_plan, cy_plan - plan_r * 1.05],
                [cx_plan, cy_plan + plan_r * 1.05],
            ],
            closed: false,
            stroke_hex: String::from("#5E5D59"),
            stroke_width_mm: 0.18,
            dash_pattern_mm: None,
            fill_hex: None,
        });

        let valid_rim_fit = m.sherd_circle_fit.as_ref().filter(|f| {
            mode == PlateLayoutMode::RimOrBaseSherdReconstructed
                && f.plate_strategy != SherdPlateStrategy::ThreeColumnParallel
                && f.arc_span_deg >= 25.0
        });

        if let Some(fit) = valid_rim_fit {
            // Task A-3(3): 검증된 구연 복원편(Δθ >= 25~30°)은 복원 구연 원(잔존 호 실선 + 나머지 추정 파선)으로 제도
            let [a0_deg, a1_deg] = fit.arc_interval_deg;
            let span_deg = fit.arc_span_deg.clamp(25.0, 340.0);
            let mut solid_arc = Vec::with_capacity(25);
            for k in 0..=24 {
                let a = (a0_deg + (k as f64 / 24.0) * span_deg).to_radians();
                solid_arc.push([cx_plan + plan_r * a.cos(), cy_plan + plan_r * a.sin()]);
            }
            paths.push(PlatePathElement {
                layer_id: String::from("평면도_외곽윤곽선"),
                points_mm: solid_arc,
                closed: false,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.36,
                dash_pattern_mm: None,
                fill_hex: None,
            });
            let _ = a1_deg;
            let dash_span = (360.0 - span_deg).max(10.0);
            let mut dash_arc = Vec::with_capacity(37);
            for k in 0..=36 {
                let a = (a0_deg + span_deg + (k as f64 / 36.0) * dash_span).to_radians();
                dash_arc.push([cx_plan + plan_r * a.cos(), cy_plan + plan_r * a.sin()]);
            }
            paths.push(PlatePathElement {
                layer_id: String::from("평면도_결손복원_점선"),
                points_mm: dash_arc,
                closed: false,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.28,
                dash_pattern_mm: Some((2.0, 1.3)),
                fill_hex: None,
            });
        } else if let Some(tp) = top_plan_opt.and_then(|t| t.polylines_2d.first()) {
            if tp.len() >= 3 {
                let min_x = tp.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
                let max_x = tp.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
                let min_y = tp.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
                let max_y = tp.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
                let mx = 0.5 * (min_x + max_x);
                let my = 0.5 * (min_y + max_y);
                let out_pts: Vec<[f64; 2]> = tp
                    .iter()
                    .map(|p| [cx_plan + (p[0] - mx), cy_plan + (p[1] - my)])
                    .collect();
                paths.push(PlatePathElement {
                    layer_id: String::from("평면도_외곽윤곽선"),
                    points_mm: out_pts,
                    closed: true,
                    stroke_hex: String::from("#141413"),
                    stroke_width_mm: 0.36,
                    dash_pattern_mm: None,
                    fill_hex: None,
                });
            }
        } else {
            let mut out_pts = Vec::with_capacity(48);
            for k in 0..48 {
                let a = (k as f64 / 48.0) * std::f64::consts::TAU;
                out_pts.push([
                    cx_plan + plan_r * a.cos(),
                    cy_plan + (plan_r * 0.65) * a.sin(),
                ]);
            }
            paths.push(PlatePathElement {
                layer_id: String::from("평면도_외곽윤곽선"),
                points_mm: out_pts,
                closed: true,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.36,
                dash_pattern_mm: None,
                fill_hex: None,
            });
        }

        callouts.push(PlateTextCallout {
            layer_id: String::from("평면도_라벨"),
            pos_mm: [cx_plan - 20.0, cy_plan + plan_r + 2.8],
            text: format!("1:1 평면도 (폭 {:.1}mm)", max_d),
            font_size_pt: 8.5,
            color_hex: String::from("#141413"),
        });

        // 우상단: 1:1 정밀 탁본 (Image 1 위치)
        raster_regions.push(PlateRasterRegion {
            kind: PlateRasterKind::BottomPlanRubbing,
            rect_mm: [
                cx_right - plan_r,
                cy_plan - plan_r,
                cx_right + plan_r,
                cy_plan + plan_r,
            ],
            clip_half_sign: None,
            caption: String::from("1:1 정밀 탁본"),
        });
        callouts.push(PlateTextCallout {
            layer_id: String::from("탁본_라벨"),
            pos_mm: [cx_right - 22.0, cy_plan + plan_r + 2.8],
            text: format!("1:1 정밀 탁본 (폭 {:.1}mm)", max_d),
            font_size_pt: 8.5,
            color_hex: String::from("#141413"),
        });

        let rub_bottom = cy_plan - plan_r;
        let cx_hsec = cx_right;
        if let Some(hsec) = hsec_opt {
            let cut_z_rel = (hsec.offset_mm - z_offset).clamp(1.0, height - 1.0);
            let cut_line_y = base_y + cut_z_rel;
            paths.push(PlatePathElement {
                layer_id: String::from("횡단면_절단위치선"),
                points_mm: vec![
                    [cx - half_d * 1.05, cut_line_y],
                    [cx + half_d * 1.05, cut_line_y],
                ],
                closed: false,
                stroke_hex: String::from("#D92B2B"),
                stroke_width_mm: 0.30,
                dash_pattern_mm: Some((2.2, 1.2)),
                fill_hex: None,
            });

            let mut min_hx = f64::INFINITY;
            let mut max_hx = f64::NEG_INFINITY;
            let mut min_hy = f64::INFINITY;
            let mut max_hy = f64::NEG_INFINITY;
            for pt in hsec.polylines_2d.iter().flatten() {
                min_hx = min_hx.min(pt[0]);
                max_hx = max_hx.max(pt[0]);
                min_hy = min_hy.min(pt[1]);
                max_hy = max_hy.max(pt[1]);
            }
            let mid_hx = if min_hx.is_finite() && max_hx.is_finite() {
                0.5 * (min_hx + max_hx)
            } else {
                0.0
            };
            let mid_hy = if min_hy.is_finite() && max_hy.is_finite() {
                0.5 * (min_hy + max_hy)
            } else {
                0.0
            };
            let half_hy = (0.5 * (max_hy - min_hy)).clamp(wall_t, half_d * 0.85);
            let cy_hsec_default = base_y + height * 0.38;
            let stacked_cy = rub_bottom - half_hy - 18.0;
            let cy_hsec = cy_hsec_default.min(stacked_cy).max(half_hy + 12.0);

            paths.push(PlatePathElement {
                layer_id: String::from("횡단면_십자선"),
                points_mm: vec![
                    [cx_hsec - half_d * 0.85, cy_hsec],
                    [cx_hsec + half_d * 0.85, cy_hsec],
                ],
                closed: false,
                stroke_hex: String::from("#5E5D59"),
                stroke_width_mm: 0.18,
                dash_pattern_mm: None,
                fill_hex: None,
            });

            if let Some(raw_poly) = hsec.polylines_2d.iter().max_by_key(|p| p.len()) {
                if raw_poly.len() >= 3 {
                    let f = raw_poly[0];
                    let l = *raw_poly.last().unwrap_or(&f);
                    let is_closed = (f[0] - l[0]).hypot(f[1] - l[1]) <= 8.0;
                    let mut band_pts: Vec<[f64; 2]> = raw_poly
                        .iter()
                        .map(|p| [cx_hsec + (p[0] - mid_hx), cy_hsec + (p[1] - mid_hy)])
                        .collect();
                    if !is_closed {
                        let mut inner_rev: Vec<[f64; 2]> = band_pts
                            .iter()
                            .rev()
                            .map(|p| [p[0], p[1] - wall_t])
                            .collect();
                        band_pts.append(&mut inner_rev);
                    }
                    paths.push(PlatePathElement {
                        layer_id: String::from("횡단면_단면_실선"),
                        points_mm: band_pts,
                        closed: true,
                        stroke_hex: String::from("#141413"),
                        stroke_width_mm: 0.34,
                        dash_pattern_mm: None,
                        fill_hex: Some(String::from("#141413")),
                    });
                }
            }
            callouts.push(PlateTextCallout {
                layer_id: String::from("횡단면_라벨"),
                pos_mm: [cx_hsec - half_d * 0.65, cy_hsec + half_hy + 3.5],
                text: format!("1:1 수평 횡단면 (Z={:.1}mm)", cut_z_rel),
                font_size_pt: 8.5,
                color_hex: String::from("#B5573A"),
            });
        }
    }

    // 4-C. 사용자가 [+ 단면추가] 버튼으로 그은 추가 단면(A-A')이 전달된 경우:
    if let Some(custom_sec) = custom_sec_opt {
        let plan_r = half_d.max(12.0);
        let cx_right = cx + half_d + plan_r + 38.0;
        let cx_custom = cx_right + plan_r + half_d * 0.85 + 42.0;
        let cy_custom = base_y + height * 0.15;
        let z_min_c = custom_sec
            .polylines_2d
            .iter()
            .flatten()
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min);
        let z_off_c = if z_min_c.is_finite() { z_min_c } else { 0.0 };

        for poly in &custom_sec.polylines_2d {
            if poly.len() < 2 {
                continue;
            }
            let mut pts: Vec<[f64; 2]> = poly
                .iter()
                .map(|p| [cx_custom + p[0], cy_custom + (p[1] - z_off_c)])
                .collect();
            let first = pts[0];
            let last = *pts.last().unwrap_or(&first);
            if (first[0] - last[0]).hypot(first[1] - last[1]) > 2.5 {
                let mut inner_rev: Vec<[f64; 2]> = poly
                    .iter()
                    .rev()
                    .map(|p| {
                        let sign = if p[0] >= 0.0 { -1.0 } else { 1.0 };
                        [
                            cx_custom + p[0] + sign * wall_t * 0.65,
                            cy_custom + (p[1] - z_off_c),
                        ]
                    })
                    .collect();
                pts.append(&mut inner_rev);
            }
            paths.push(PlatePathElement {
                layer_id: String::from("추가단면_실선"),
                points_mm: pts,
                closed: true,
                stroke_hex: String::from("#141413"),
                stroke_width_mm: 0.36,
                dash_pattern_mm: None,
                fill_hex: Some(String::from("#141413")),
            });
        }
        callouts.push(PlateTextCallout {
            layer_id: String::from("추가단면_라벨"),
            pos_mm: [cx_custom - 24.0, cy_custom + height + 2.8],
            text: String::from("추가 단면 A-A' (1:1)"),
            font_size_pt: 8.5,
            color_hex: String::from("#D92B2B"),
        });
    }

    // 5. 실측 축척 막대 — D1 & D2: 토기 바닥(base_y=35.0) 아래 충분한 간격(bar_y=14.0)을 두고
    //    화면 Y좌표 기준 숫자 라벨(맨 위) < 눈금 상단 < 기선(baseline) < 검은 막대 바디(맨 아래) 순서를 보장한다.
    let default_scale = ScalePreset::Full1To1;
    let (bar_len_mm, bar_label) = default_scale.standard_scale_bar_mm();
    let bar_x0 = cx - bar_len_mm * 0.5;
    let bar_y = (base_y - 21.0).max(12.0);
    let bar_top = bar_y + SCALE_BAR_HEIGHT_MM;
    let baseline_y = bar_top + SCALE_BAR_BASELINE_GAP_MM;
    paths.push(PlatePathElement {
        layer_id: String::from("치수_축척바디"),
        points_mm: vec![
            [bar_x0, bar_y],
            [bar_x0 + bar_len_mm, bar_y],
            [bar_x0 + bar_len_mm, bar_top],
            [bar_x0, bar_top],
        ],
        closed: true,
        stroke_hex: String::from("#141413"),
        stroke_width_mm: 0.0,
        dash_pattern_mm: None,
        fill_hex: Some(String::from("#141413")),
    });
    paths.push(PlatePathElement {
        layer_id: String::from("치수_축척막대"),
        points_mm: vec![[bar_x0, baseline_y], [bar_x0 + bar_len_mm, baseline_y]],
        closed: false,
        stroke_hex: String::from("#141413"),
        stroke_width_mm: 0.20,
        dash_pattern_mm: None,
        fill_hex: None,
    });
    for d in 0..=10 {
        let tx = bar_x0 + bar_len_mm * d as f64 / 10.0;
        let tick_h = if d == 5 {
            SCALE_BAR_TICK_MID_MM
        } else if d == 0 || d == 10 {
            SCALE_BAR_TICK_END_MM
        } else {
            SCALE_BAR_TICK_MINOR_MM
        };
        paths.push(PlatePathElement {
            layer_id: String::from("치수_축척눈금"),
            points_mm: vec![[tx, bar_top], [tx, baseline_y + tick_h]],
            closed: false,
            stroke_hex: String::from("#141413"),
            stroke_width_mm: 0.18,
            dash_pattern_mm: None,
            fill_hex: None,
        });
    }
    let mid_label = scale_bar_mid_label(bar_len_mm);
    let label_y = baseline_y + SCALE_BAR_TICK_MID_MM + 1.8;
    callouts.push(PlateTextCallout {
        layer_id: String::from("치수_축척라벨"),
        pos_mm: [bar_x0 - 1.0, label_y],
        text: String::from("0"),
        font_size_pt: 8.0,
        color_hex: String::from("#141413"),
    });
    callouts.push(PlateTextCallout {
        layer_id: String::from("치수_축척라벨"),
        pos_mm: [cx - mid_label.len() as f64 * 0.9, label_y],
        text: mid_label,
        font_size_pt: 8.0,
        color_hex: String::from("#141413"),
    });
    callouts.push(PlateTextCallout {
        layer_id: String::from("치수_축척라벨"),
        pos_mm: [bar_x0 + bar_len_mm - 2.0, label_y],
        text: bar_label.to_string(),
        font_size_pt: 8.0,
        color_hex: String::from("#141413"),
    });

    let canvas_height_mm = if mode != PlateLayoutMode::CompleteVesselStandard {
        let actual_top_mm = paths
            .iter()
            .flat_map(|p| p.points_mm.iter().map(|pt| pt[1]))
            .chain(callouts.iter().map(|c| c.pos_mm[1] + 4.0))
            .fold(canvas_height_mm, f64::max);
        canvas_height_mm.max(actual_top_mm + 8.0)
    } else {
        canvas_height_mm
    };

    callouts.push(PlateTextCallout {
        layer_id: String::from("치수_축척막대"),
        pos_mm: [margin * 0.7, canvas_height_mm - 9.0],
        text: format!("{} · {}", input.title, m.metric_strip_summary),
        font_size_pt: 9.0,
        color_hex: String::from("#141413"),
    });

    ComposedPlate2D {
        mode,
        title: input.title.to_string(),
        scale_preset: default_scale,
        canvas_width_mm,
        canvas_height_mm,
        flip_exterior_right: input.flip_exterior_right,
        paths,
        callouts,
        raster_regions,
    }
}
