use crate::plate_composer::ComposedPlate2D;
use std::fmt::Write as _;
use strata_section::fit_polyline_to_cubic_bezier;

/// `1 mm = 5 px` 표준 SVG 출력 스케일 계약 (`V7`).
pub const PX_PER_MM_1TO1: f64 = 5.0;

/// 『유물제도선.ai』 정본 7단계 선 굵기 위계 (`(pt, mm)` 쌍).
pub const AI_STANDARD_LINE_WEIGHTS_PT_MM: [(f64, f64); 7] = [
    (0.80, 0.282), // 단면 외곽선
    (0.50, 0.176), // 외면 입면 외곽선 / 수평 연결선
    (0.40, 0.141), // 중심 수직선 / 석기 주 외곽선
    (0.30, 0.106), // 외면 주요 굴곡선 / 박리 능선
    (0.20, 0.071), // 보조 굴곡선 / 단차선 / 중심 연장선
    (0.15, 0.053), // 단면 45° 빗금해칭 / 깨진면 / 문양 / 연장선(|, —) / 목질흔
    (0.12, 0.042), // 석기 미세 박리결
];

/// 『유물 편집판.ai』 표준 10대 최상위 일러스트레이터 레이어 그룹 ID.
pub const AI_STANDARD_10_LAYER_GROUPS: [&str; 10] = [
    "01_방안지_판면틀",
    "02_유물실사사진",
    "03_정밀탁본_래스터",
    "04_입면_외곽선_0.5pt",
    "05_굴곡선_능선_0.3_0.2pt",
    "06_깨진면_문양_연장선_0.15pt",
    "07_단면_외곽선_0.8pt",
    "08_단면_빗금해칭_0.15pt",
    "09_목질흔_0.15pt",
    "10_축척막대_치수_캡션",
];

/// 도판에 삽입할 유물 실사 사진 슬롯(`rect_mm` = `[min_x, min_y, max_x, max_y]`).
#[derive(Debug)]
pub struct PhotoSlot<'a> {
    pub rect_mm: [f64; 4],
    pub path: &'a std::path::Path,
}

/// 도판에 임베드할 정밀 탁본 PNG 래스터 슬롯 (`rect_mm` = `[min_x, min_y, max_x, max_y]`).
#[derive(Debug, Clone)]
pub struct RubbingPngSlot {
    pub id: String,
    pub rect_mm: [f64; 4],
    pub png_bytes: Vec<u8>,
}

/// 표준 Base64 인코더(외부 크레이트 없이 SVG `<image>` 임베드용).
pub fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn photo_mime(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "webp" => "image/webp",
        _ => "image/jpeg",
    }
}

/// 합성된 2D 도판(`ComposedPlate2D`)을 Adobe Illustrator 호환 다중 레이어 SVG(`유물제도선.ai` / `.svg`)로 직렬화한다.
/// - `1 mm = 5 px` (`PX_PER_MM_1TO1 = 5.0`) 정밀 스케일 계약을 준수한다.
/// - 네 모서리에 `1×1 px` 투명 정렬 앵커 `<rect>`를 삽입하여 래스터 탁본 PNG와 Illustrator에서 0.00 px 오차로 겹쳐지게 한다 (`V7`).
pub fn render_plate_to_illustrator_svg(plate: &ComposedPlate2D) -> String {
    render_plate_to_illustrator_svg_with_meta(plate, "")
}

/// 유물 실사 사진을 SVG `<image>` 로 임베드한판(「유물 편집판」 템플릿의 사진 배치).
pub fn render_plate_to_illustrator_svg_with_photos(
    plate: &ComposedPlate2D,
    provenance: &str,
    draw_grid: bool,
    photos: &[PhotoSlot<'_>],
) -> String {
    render_plate_to_illustrator_svg_with_rubbings_and_photos(
        plate,
        provenance,
        draw_grid,
        photos,
        &[],
    )
}

/// 정밀 탁본 PNG 래스터와 실사 사진, 『유물 편집판.ai』 10대 표준 레이어, `kurbo` 3차 베지어(`M ... C ...`) 곡선을 모두 포함한 완전체 SVG 직렬화 함수.
pub fn render_plate_to_illustrator_svg_with_rubbings_and_photos(
    plate: &ComposedPlate2D,
    provenance: &str,
    draw_grid: bool,
    photos: &[PhotoSlot<'_>],
    rubbings: &[RubbingPngSlot],
) -> String {
    let mut svg = render_plate_to_illustrator_svg_with_options(plate, provenance, draw_grid);
    let px = |mm: f64| mm * PX_PER_MM_1TO1;
    let h_px = (plate.canvas_height_mm * PX_PER_MM_1TO1).round().max(50.0);

    if !photos.is_empty() {
        let mut block = String::new();
        let _ = writeln!(block, r#"    <g id="유물사진">"#);
        for (i, slot) in photos.iter().enumerate() {
            let Ok(bytes) = std::fs::read(slot.path) else {
                continue;
            };
            let b64 = base64_encode(&bytes);
            let mime = photo_mime(slot.path);
            let [x0, y0, x1, y1] = slot.rect_mm;
            let _ = writeln!(
                block,
                r#"      <image id="유물사진_{n}" x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" preserveAspectRatio="xMidYMid meet" xlink:href="data:{mime};base64,{b64}" />"#,
                n = i + 1,
                x = px(x0.min(x1)),
                y = px(y0.min(y1)),
                w = px((x1 - x0).abs()),
                h = px((y1 - y0).abs()),
                mime = mime,
                b64 = b64
            );
        }
        let _ = writeln!(block, "    </g>");
        let target = r#"  <g id="02_유물실사사진">"#;
        if let Some(pos) = svg.find(target) {
            let insert_at = pos + target.len() + 1;
            svg.insert_str(insert_at.min(svg.len()), &block);
        }
    }

    if !rubbings.is_empty() {
        let mut rblock = String::new();
        for (i, rslot) in rubbings.iter().enumerate() {
            if rslot.png_bytes.is_empty() {
                continue;
            }
            let b64 = base64_encode(&rslot.png_bytes);
            let [x0, y0, x1, y1] = rslot.rect_mm;
            let rx = px(x0.min(x1));
            let ry = (h_px - px(y0.max(y1))).max(0.0);
            let rw = px((x1 - x0).abs()).max(1.0);
            let rh = px((y1 - y0).abs()).max(1.0);
            let _ = writeln!(
                rblock,
                r#"    <image id="{id}_{n}" x="{rx:.2}" y="{ry:.2}" width="{rw:.2}" height="{rh:.2}" preserveAspectRatio="none" xlink:href="data:image/png;base64,{b64}" />"#,
                id = rslot.id,
                n = i + 1,
            );
        }
        let target = r#"  <g id="03_정밀탁본_래스터">"#;
        if let Some(pos) = svg.find(target) {
            let insert_at = pos + target.len() + 1;
            svg.insert_str(insert_at.min(svg.len()), &rblock);
        }
    }

    svg
}

fn classify_ai_canonical_layer_group(layer_id: &str) -> &'static str {
    if layer_id.contains("목질흔") {
        "09_목질흔_0.15pt"
    } else if layer_id.contains("해칭") || layer_id.contains("빗금") {
        "08_단면_빗금해칭_0.15pt"
    } else if layer_id.contains("단면") && !layer_id.contains("절단위치선") {
        "07_단면_외곽선_0.8pt"
    } else if layer_id.contains("연장선")
        || layer_id.contains("파손")
        || layer_id.contains("결손")
        || layer_id.contains("파선")
    {
        "06_깨진면_문양_연장선_0.15pt"
    } else if layer_id.contains("구조선")
        || layer_id.contains("내경선")
        || layer_id.contains("내벽")
        || layer_id.contains("십자선")
        || layer_id.contains("절단위치선")
    {
        "05_굴곡선_능선_0.3_0.2pt"
    } else if layer_id.contains("축척") || layer_id.contains("치수") || layer_id.contains("방위")
    {
        "10_축척막대_치수_캡션"
    } else {
        "04_입면_외곽선_0.5pt"
    }
}

/// 『유물제도선.ai』 7단계 표준 선 굵기(mm)로 매핑한다.
pub fn canonical_ai_stroke_width_mm(layer_id: &str, raw_mm: f64) -> f64 {
    if raw_mm <= 0.0 {
        return 0.0;
    }
    if layer_id.contains("단면_실선") || layer_id.contains("추가단면_실선") {
        0.282 // 0.8 pt
    } else if layer_id.contains("외곽선_실선")
        || layer_id.contains("평면도_외곽")
        || layer_id.contains("횡단면_외벽")
        || layer_id.contains("구경_기준선")
        || layer_id.contains("저경_기준선")
    {
        0.176 // 0.5 pt
    } else if layer_id == "중심_기준선" {
        0.141 // 0.4 pt
    } else if layer_id.contains("구조선")
        || layer_id.contains("내경선")
        || layer_id.contains("내벽윤곽선")
    {
        0.106 // 0.3 pt
    } else if layer_id.contains("십자선")
        || layer_id.contains("치수_축척막대")
        || layer_id.contains("치수_축척눈금")
    {
        0.071 // 0.2 pt
    } else if layer_id.contains("해칭")
        || layer_id.contains("연장선")
        || layer_id.contains("목질흔")
        || layer_id.contains("파손")
    {
        0.053 // 0.15 pt
    } else {
        raw_mm
    }
}

fn build_cubic_bezier_svg_path_d(points_mm: &[[f64; 2]], h_px: f64, closed: bool) -> String {
    if points_mm.len() < 2 {
        return String::new();
    }
    let flipped_mm: Vec<[f64; 2]> = points_mm
        .iter()
        .map(|pt| [pt[0], (h_px / PX_PER_MM_1TO1) - pt[1]])
        .collect();

    if flipped_mm.len() == 2 {
        let p0 = [
            flipped_mm[0][0] * PX_PER_MM_1TO1,
            flipped_mm[0][1] * PX_PER_MM_1TO1,
        ];
        let p1 = [
            flipped_mm[1][0] * PX_PER_MM_1TO1,
            flipped_mm[1][1] * PX_PER_MM_1TO1,
        ];
        let c1 = [p0[0] + (p1[0] - p0[0]) / 3.0, p0[1] + (p1[1] - p0[1]) / 3.0];
        let c2 = [
            p0[0] + 2.0 * (p1[0] - p0[0]) / 3.0,
            p0[1] + 2.0 * (p1[1] - p0[1]) / 3.0,
        ];
        return format!(
            "M {:.2} {:.2} C {:.2} {:.2}, {:.2} {:.2}, {:.2} {:.2}",
            p0[0], p0[1], c1[0], c1[1], c2[0], c2[1], p1[0], p1[1]
        );
    }

    let mut work = flipped_mm;
    if closed {
        if let (Some(&first), Some(&last)) = (work.first(), work.last()) {
            if (first[0] - last[0]).hypot(first[1] - last[1]) > 1e-4 {
                work.push(first);
            }
        }
    }
    let fitted = fit_polyline_to_cubic_bezier(&work, 0.015, PX_PER_MM_1TO1);
    let mut d = fitted.svg_path_d_1mm_5px;
    if closed && !d.ends_with(" Z") {
        d.push_str(" Z");
    }
    d
}

/// `provenance` 문자열을 SVG `<metadata>` 와 숨김 레이어로 각인해 도판의 재현 근거를 남긴다.
pub fn render_plate_to_illustrator_svg_with_meta(
    plate: &ComposedPlate2D,
    provenance: &str,
) -> String {
    render_plate_to_illustrator_svg_with_options(plate, provenance, false)
}

/// 방안지 격자 포함 여부까지 지정하는 전체 옵션판.
/// 「유물 편집판」 템플릿은 인쇄물에 방안지(1mm/5mm/10mm)를 깔므로 `draw_grid = true` 를 쓴다.
pub fn render_plate_to_illustrator_svg_with_options(
    plate: &ComposedPlate2D,
    provenance: &str,
    draw_grid: bool,
) -> String {
    let w_px = (plate.canvas_width_mm * PX_PER_MM_1TO1).round().max(50.0);
    let h_px = (plate.canvas_height_mm * PX_PER_MM_1TO1).round().max(50.0);

    let mut svg = String::with_capacity(8192);
    let _ = writeln!(
        svg,
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#
    );
    let _ = writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{:.1}mm" height="{:.1}mm" viewBox="0 0 {w_px:.0} {h_px:.0}" data-scale-px-per-mm="{PX_PER_MM_1TO1:.1}" data-plate-mode="{:?}">"#,
        plate.canvas_width_mm, plate.canvas_height_mm, plate.mode
    );
    if !provenance.is_empty() {
        let _ = writeln!(
            svg,
            "  <metadata id=\"STRATA_PROVENANCE\">{}</metadata>",
            provenance
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        );
        let _ = writeln!(
            svg,
            r#"  <g id="출처_비인쇄" display="none"><text x="0" y="0" font-size="1">{}</text></g>"#,
            provenance
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        );
    }
    let _ = writeln!(svg, r#"  <g id="01_방안지_판면틀">"#);
    let _ = writeln!(
        svg,
        r##"    <rect id="한지_배경" x="0" y="0" width="{w_px:.0}" height="{h_px:.0}" fill="#FAF9F5" />"##
    );

    // 0. 방안지 격자 (1mm / 5mm / 10mm) — 인쇄물에 깔리는 실무 템플릿 규격 (3차 베지어 C 포맷 호환)
    if draw_grid {
        let w_mm = plate.canvas_width_mm;
        let h_mm = plate.canvas_height_mm;
        let px = |mm: f64| mm * PX_PER_MM_1TO1;
        for (step, color, width) in [
            (1.0_f64, "#E6E2D6", "0.10"),
            (5.0, "#D8D2C0", "0.14"),
            (10.0, "#C7C0AA", "0.18"),
        ] {
            let mut d = String::new();
            let mut v = step;
            while v < w_mm {
                let x = px(v);
                let y = px(h_mm);
                let _ = write!(
                    d,
                    "M {x:.2} 0.00 C {x:.2} {:.2}, {x:.2} {:.2}, {x:.2} {y:.2} ",
                    y / 3.0,
                    y * 2.0 / 3.0
                );
                v += step;
            }
            let mut u = step;
            while u < h_mm {
                let x = px(w_mm);
                let y = px(u);
                let _ = write!(
                    d,
                    "M 0.00 {y:.2} C {:.2} {y:.2}, {:.2} {y:.2}, {x:.2} {y:.2} ",
                    x / 3.0,
                    x * 2.0 / 3.0
                );
                u += step;
            }
            let _ = writeln!(
                svg,
                r#"    <path id="방안지_{step:.0}mm" d="{d}" fill="none" stroke="{color}" stroke-width="{width}" />"#
            );
        }
    }

    // 1. 4-Corner 1x1px transparent alignment anchors (Illustrator 0.00px registration)
    let _ = writeln!(svg, r#"    <g id="정렬_앵커">"#);
    for (ax, ay) in [
        (0.0, 0.0),
        ((w_px - 1.0).max(0.0), 0.0),
        (0.0, (h_px - 1.0).max(0.0)),
        ((w_px - 1.0).max(0.0), (h_px - 1.0).max(0.0)),
    ] {
        let _ = writeln!(
            svg,
            r#"      <rect x="{ax:.0}" y="{ay:.0}" width="1" height="1" fill="none" stroke="none" />"#
        );
    }
    let _ = writeln!(svg, r#"    </g>"#);
    let _ = writeln!(svg, r#"  </g>"#);

    let _ = writeln!(svg, r#"  <g id="02_유물실사사진">"#);
    let _ = writeln!(svg, r#"  </g>"#);

    // 03. 정밀 탁본 래스터 레이어 (Base64 PNG <image> 슬롯)
    let _ = writeln!(svg, r#"  <g id="03_정밀탁본_래스터">"#);
    // 기본 1x1 투명 PNG Base64 (외부에서 실제 고해상도 탁본 PNG가 주입되면 상단에 추가됨)
    const TRANSPARENT_1X1_PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    for (ri, reg) in plate.raster_regions.iter().enumerate() {
        let [x0, y0, x1, y1] = reg.rect_mm;
        let rx = x0.min(x1) * PX_PER_MM_1TO1;
        let ry = (h_px - y0.max(y1) * PX_PER_MM_1TO1).max(0.0);
        let rw = ((x1 - x0).abs() * PX_PER_MM_1TO1).max(1.0);
        let rh = ((y1 - y0).abs() * PX_PER_MM_1TO1).max(1.0);
        let _ = writeln!(
            svg,
            r#"    <image id="정밀탁본_영역_{}" x="{rx:.2}" y="{ry:.2}" width="{rw:.2}" height="{rh:.2}" preserveAspectRatio="none" xlink:href="data:image/png;base64,{TRANSPARENT_1X1_PNG_B64}" />"#,
            ri + 1
        );
    }
    let _ = writeln!(svg, r#"  </g>"#);

    // 04 ~ 09. 『유물 편집판.ai & 유물제도선.ai』 표준 벡터 레이어 그룹 (kurbo 3차 베지어 C 출력)
    let mut layer_names: Vec<String> = plate.paths.iter().map(|p| p.layer_id.clone()).collect();
    layer_names.sort();
    layer_names.dedup();

    for &canon_group in &AI_STANDARD_10_LAYER_GROUPS[3..9] {
        let _ = writeln!(svg, r#"  <g id="{canon_group}">"#);
        for layer in &layer_names {
            if classify_ai_canonical_layer_group(layer) != canon_group {
                continue;
            }
            let _ = writeln!(svg, r#"    <g id="{layer}">"#);
            for elem in plate.paths.iter().filter(|p| &p.layer_id == layer) {
                if elem.points_mm.len() < 2 {
                    continue;
                }
                let d = build_cubic_bezier_svg_path_d(&elem.points_mm, h_px, elem.closed);
                if d.is_empty() {
                    continue;
                }
                let fill_attr = elem.fill_hex.as_deref().unwrap_or("none");
                let std_w_mm = canonical_ai_stroke_width_mm(layer, elem.stroke_width_mm);
                let stroke_hex = if std_w_mm <= 0.0 {
                    "none"
                } else {
                    elem.stroke_hex.as_str()
                };
                let stroke_w_px = (std_w_mm * PX_PER_MM_1TO1).max(0.2);
                let stroke_w_pt = std_w_mm * 72.0 / 25.4;
                let dash_attr = if let Some((on_mm, off_mm)) = elem.dash_pattern_mm {
                    format!(
                        r#" stroke-dasharray="{:.2},{:.2}""#,
                        on_mm * PX_PER_MM_1TO1,
                        off_mm * PX_PER_MM_1TO1
                    )
                } else {
                    String::new()
                };
                let _ = writeln!(
                    svg,
                    r#"      <path d="{d}" fill="{fill_attr}" stroke="{stroke_hex}" stroke-width="{stroke_w_px:.2}" data-stroke-pt="{stroke_w_pt:.2}" data-stroke-mm="{std_w_mm:.3}" stroke-linecap="round" stroke-linejoin="round"{dash_attr} />"#,
                );
            }
            let _ = writeln!(svg, r#"    </g>"#);
        }
        let _ = writeln!(svg, r#"  </g>"#);
    }

    // 10. 축척막대 · 치수 · 캡션 레이어 그룹
    let _ = writeln!(svg, r#"  <g id="10_축척막대_치수_캡션">"#);
    for layer in &layer_names {
        if classify_ai_canonical_layer_group(layer) != "10_축척막대_치수_캡션" {
            continue;
        }
        let _ = writeln!(svg, r#"    <g id="{layer}">"#);
        for elem in plate.paths.iter().filter(|p| &p.layer_id == layer) {
            if elem.points_mm.len() < 2 {
                continue;
            }
            let d = build_cubic_bezier_svg_path_d(&elem.points_mm, h_px, elem.closed);
            if d.is_empty() {
                continue;
            }
            let fill_attr = elem.fill_hex.as_deref().unwrap_or("none");
            let std_w_mm = canonical_ai_stroke_width_mm(layer, elem.stroke_width_mm);
            let stroke_hex = if std_w_mm <= 0.0 {
                "none"
            } else {
                elem.stroke_hex.as_str()
            };
            let stroke_w_px = (std_w_mm * PX_PER_MM_1TO1).max(0.2);
            let dash_attr = if let Some((on_mm, off_mm)) = elem.dash_pattern_mm {
                format!(
                    r#" stroke-dasharray="{:.2},{:.2}""#,
                    on_mm * PX_PER_MM_1TO1,
                    off_mm * PX_PER_MM_1TO1
                )
            } else {
                String::new()
            };
            let _ = writeln!(
                svg,
                r#"      <path d="{d}" fill="{fill_attr}" stroke="{stroke_hex}" stroke-width="{stroke_w_px:.2}" stroke-linecap="round" stroke-linejoin="round"{dash_attr} />"#,
            );
        }
        let _ = writeln!(svg, r#"    </g>"#);
    }
    let _ = writeln!(svg, r#"    <g id="치수_및_주기">"#);
    for c in &plate.callouts {
        let px = c.pos_mm[0] * PX_PER_MM_1TO1;
        let py = (h_px - c.pos_mm[1] * PX_PER_MM_1TO1).clamp(12.0, h_px - 6.0);
        let escaped = c
            .text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let _ = writeln!(
            svg,
            r#"      <text x="{px:.2}" y="{py:.2}" fill="{}" font-family="Pretendard, JetBrains Mono, sans-serif" font-size="{:.1}pt">{escaped}</text>"#,
            c.color_hex, c.font_size_pt
        );
    }
    let _ = writeln!(svg, r#"    </g>"#);
    let _ = writeln!(svg, r#"  </g>"#);
    let _ = writeln!(svg, "</svg>");

    svg
}
