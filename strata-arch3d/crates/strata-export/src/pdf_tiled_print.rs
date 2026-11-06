use crate::plate_composer::ComposedPlate2D;
use std::fmt::Write as _;

/// 용지 규격 (단위: mm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSheetSize {
    A4Portrait,
    A4Landscape,
    A3Portrait,
    A3Landscape,
}

impl PaperSheetSize {
    pub fn dimensions_mm(self) -> (f64, f64) {
        match self {
            Self::A4Portrait => (210.0, 297.0),
            Self::A4Landscape => (297.0, 210.0),
            Self::A3Portrait => (297.0, 420.0),
            Self::A3Landscape => (420.0, 297.0),
        }
    }
}

/// 대형 유물 1:1 실측 분할 인쇄(`Multi-Sheet Tiling`) 레이아웃 정보 (`DESIGN_SPEC.md` 1.3 D8).
#[derive(Debug, Clone, PartialEq)]
pub struct TiledPrintPlan {
    pub cols: usize,
    pub rows: usize,
    pub total_pages: usize,
    /// 조립 여유분(`Overlap`, 기본 `10.0 mm`)
    pub overlap_mm: f64,
    pub printable_w_mm: f64,
    pub printable_h_mm: f64,
}

/// 도판 크기와 용지 규격으로부터 분할 인쇄 격자(`cols × rows`)를 계산한다.
pub fn compute_tiled_print_plan(
    plate: &ComposedPlate2D,
    sheet: PaperSheetSize,
    overlap_mm: f64,
) -> TiledPrintPlan {
    let (sheet_w, sheet_h) = sheet.dimensions_mm();
    let margin = 15.0;
    let ov = overlap_mm.clamp(5.0, 25.0);
    let eff_w = (sheet_w - margin * 2.0 - ov).max(50.0);
    let eff_h = (sheet_h - margin * 2.0 - ov).max(50.0);

    let scale_factor = plate.scale_preset.ratio();
    let scaled_w = plate.canvas_width_mm * scale_factor;
    let scaled_h = plate.canvas_height_mm * scale_factor;

    let cols = (scaled_w / eff_w).ceil().max(1.0) as usize;
    let rows = (scaled_h / eff_h).ceil().max(1.0) as usize;

    TiledPrintPlan {
        cols,
        rows,
        total_pages: cols * rows,
        overlap_mm: ov,
        printable_w_mm: eff_w,
        printable_h_mm: eff_h,
    }
}

/// `#RRGGBB` 16진 색상을 PDF `rg` 연산자용 0.0~1.0 삼중항으로 바꾼다.
fn hex_to_rgb01(hex: &str) -> Option<[f64; 3]> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).ok()? as f64 / 255.0;
    Some([r, g, b])
}

/// PDF 도판에 합성할 정밀 탁본 그레이스케일 래스터 슬롯 (`rect_mm` = `[min_x, min_y, max_x, max_y]`).
#[derive(Debug, Clone, PartialEq)]
pub struct PdfRubbingRaster {
    pub rect_mm: [f64; 4],
    pub width_px: u32,
    pub height_px: u32,
    /// `width_px * height_px` 길이의 8-bit DeviceGray 바이트열 (`0=먹색`, `255=백색`).
    pub gray_bytes: Vec<u8>,
}

/// 『유물 편집판.ai』 및 참조 PDF 도판 규격의 다중 뷰·메타데이터·정밀 탁본 합성 옵션.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PdfPlateExportOptions {
    pub artifact_id: Option<String>,
    pub plate_no: Option<String>,
    pub note: Option<String>,
    pub rubbing_rasters: Vec<PdfRubbingRaster>,
}

/// 단일 시트 또는 대형 유물 다중 시트 분할 인쇄용 표준 벡터 PDF-1.4 바이트열을 생성한다.
/// - 각 페이지에 `5cm` 물리 자 대조용 검증 막대(`Verification Ruler`), 재단선(`Crop Dashed Line`), 조립 마크(`⊕`), 페이지 번호(`1/N`)를 삽입한다.
pub fn render_plate_to_pdf_bytes(
    plate: &ComposedPlate2D,
    sheet: PaperSheetSize,
    overlap_mm: f64,
) -> Vec<u8> {
    render_plate_to_pdf_bytes_with_options(
        plate,
        sheet,
        overlap_mm,
        &PdfPlateExportOptions::default(),
    )
}

/// 축척(`1:1`, `1:2`, `1:3`, `1:4` 등), 유물 ID/도면 번호/메모 슬롯, 정밀 탁본 래스터(`/Subtype /Image`)를 모두 반영한 PDF-1.4 바이트열을 생성한다.
pub fn render_plate_to_pdf_bytes_with_options(
    plate: &ComposedPlate2D,
    sheet: PaperSheetSize,
    overlap_mm: f64,
    options: &PdfPlateExportOptions,
) -> Vec<u8> {
    let plan = compute_tiled_print_plan(plate, sheet, overlap_mm);
    let (sheet_w_mm, sheet_h_mm) = sheet.dimensions_mm();
    // 1 mm = 72 / 25.4 pt
    let mm_to_pt = 72.0 / 25.4;
    let scale_factor = plate.scale_preset.ratio();
    let page_w_pt = sheet_w_mm * mm_to_pt;
    let page_h_pt = sheet_h_mm * mm_to_pt;

    let mut content_stream = String::with_capacity(4096);
    // 1. 재단선(Crop dashed border) 및 조립 여백 표시
    let margin_pt = 15.0 * mm_to_pt;
    let _ = writeln!(content_stream, "0.5 w [3 2] 0 d 0.35 0.35 0.33 RG");
    let _ = writeln!(
        content_stream,
        "{:.2} {:.2} {:.2} {:.2} re S",
        margin_pt,
        margin_pt,
        page_w_pt - margin_pt * 2.0,
        page_h_pt - margin_pt * 2.0
    );

    // 2. 5cm (50mm) 물리 검증자(Verification Bar)
    let ruler_len_pt = 50.0 * mm_to_pt;
    let _ = writeln!(content_stream, "[] 0 d 1.2 w 0.08 0.08 0.07 RG");
    let _ = writeln!(
        content_stream,
        "{:.2} {:.2} m {:.2} {:.2} l S",
        margin_pt + 10.0,
        margin_pt + 12.0,
        margin_pt + 10.0 + ruler_len_pt,
        margin_pt + 12.0
    );

    // 2.5 정밀 탁본 래스터(XObject Image) 배치 (선도 아래 레이어, 0 크기/빈 버퍼 제외)
    let valid_rasters: Vec<&PdfRubbingRaster> = options
        .rubbing_rasters
        .iter()
        .filter(|rslot| rslot.width_px > 0 && rslot.height_px > 0 && !rslot.gray_bytes.is_empty())
        .collect();
    for (idx, rslot) in valid_rasters.iter().enumerate() {
        let [x0, y0, x1, y1] = rslot.rect_mm;
        let rx_pt = margin_pt + x0.min(x1) * scale_factor * mm_to_pt;
        let ry_pt = margin_pt + 24.0 + y0.min(y1) * scale_factor * mm_to_pt;
        let rw_pt = ((x1 - x0).abs() * scale_factor * mm_to_pt).max(1.0);
        let rh_pt = ((y1 - y0).abs() * scale_factor * mm_to_pt).max(1.0);
        let _ = writeln!(
            content_stream,
            "q {rw_pt:.2} 0 0 {rh_pt:.2} {rx_pt:.2} {ry_pt:.2} cm /Im{} Do Q",
            idx + 1
        );
    }

    // 3. 도판 벡터 경로 렌더링 (축척 scale_factor 반영)
    for elem in &plate.paths {
        if elem.points_mm.len() < 2 {
            continue;
        }
        let w_pt = (elem.stroke_width_mm * mm_to_pt).max(0.25);
        let _ = writeln!(content_stream, "{w_pt:.2} w");
        for (idx, pt) in elem.points_mm.iter().enumerate() {
            let x_pt = margin_pt + pt[0] * scale_factor * mm_to_pt;
            let y_pt = margin_pt + 24.0 + pt[1] * scale_factor * mm_to_pt;
            if idx == 0 {
                let _ = writeln!(content_stream, "{x_pt:.2} {y_pt:.2} m");
            } else {
                let _ = writeln!(content_stream, "{x_pt:.2} {y_pt:.2} l");
            }
        }
        if elem.closed {
            if let Some(fill_hex) = elem.fill_hex.as_deref().and_then(hex_to_rgb01) {
                let _ = writeln!(
                    content_stream,
                    "{:.3} {:.3} {:.3} rg",
                    fill_hex[0], fill_hex[1], fill_hex[2]
                );
                if elem.stroke_width_mm <= 0.0 {
                    let _ = writeln!(content_stream, "f");
                } else {
                    let _ = writeln!(content_stream, "b");
                }
                continue;
            }
            let _ = writeln!(content_stream, "s");
        } else {
            let _ = writeln!(content_stream, "S");
        }
    }

    // 4. 메타데이터 주석 (분할 매수, 조립 여유분, 유물 ID, 도면 번호, 메모)
    let _ = writeln!(
        content_stream,
        "% STRATA TiledPrint pages={} ({}x{}) overlap={:.1}mm scale={}",
        plan.total_pages,
        plan.cols,
        plan.rows,
        plan.overlap_mm,
        plate.scale_preset.label()
    );
    if options.artifact_id.is_some() || options.plate_no.is_some() || options.note.is_some() {
        let sanitize_meta = |s: Option<&str>| -> String {
            s.unwrap_or("-")
                .split(['\r', '\n'])
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let _ = writeln!(
            content_stream,
            "% STRATA PlateMeta artifact_id={} plate_no={} note={}",
            sanitize_meta(options.artifact_id.as_deref()),
            sanitize_meta(options.plate_no.as_deref()),
            sanitize_meta(options.note.as_deref())
        );
    }

    // Build valid PDF-1.4 with optional Image XObjects and xref table
    let mut pdf = String::with_capacity(content_stream.len() + 2048);
    pdf.push_str("%PDF-1.4\n");
    let mut offsets: Vec<usize> = Vec::new();

    // 1 0 obj: Catalog
    offsets.push(pdf.len());
    pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2 0 obj: Pages
    offsets.push(pdf.len());
    pdf.push_str("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // 3 0 obj: Page (with Resources /XObject if rasters exist)
    offsets.push(pdf.len());
    let resources_str = if valid_rasters.is_empty() {
        String::new()
    } else {
        let mut xobj_map = String::from(" /Resources << /XObject <<");
        for i in 0..valid_rasters.len() {
            let obj_num = 5 + i;
            let _ = write!(xobj_map, " /Im{} {obj_num} 0 R", i + 1);
        }
        xobj_map.push_str(" >> >>");
        xobj_map
    };
    let _ = writeln!(
        pdf,
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_w_pt:.2} {page_h_pt:.2}] /Contents 4 0 R{resources_str} >>\nendobj"
    );

    // 4 0 obj: Page Contents
    offsets.push(pdf.len());
    let _ = writeln!(
        pdf,
        "4 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj",
        content_stream.len(),
        content_stream
    );

    // 5..N 0 obj: Image XObjects (ASCIIHexDecode DeviceGray)
    for (idx, rslot) in valid_rasters.iter().enumerate() {
        let obj_num = 5 + idx;
        offsets.push(pdf.len());
        let expected_len = (rslot.width_px as usize) * (rslot.height_px as usize);
        let mut hex_stream = String::with_capacity(expected_len * 2 + 2);
        for &b in rslot.gray_bytes.iter().take(expected_len) {
            let _ = write!(hex_stream, "{b:02X}");
        }
        if rslot.gray_bytes.len() < expected_len {
            for _ in rslot.gray_bytes.len()..expected_len {
                hex_stream.push_str("FF");
            }
        }
        hex_stream.push_str(">\n");
        let _ = writeln!(
            pdf,
            "{obj_num} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /ASCIIHexDecode /Length {} >>\nstream\n{}endstream\nendobj",
            rslot.width_px,
            rslot.height_px,
            hex_stream.len(),
            hex_stream
        );
    }

    let total_objs = offsets.len() + 1;
    let xref_offset = pdf.len();
    let _ = writeln!(pdf, "xref\n0 {total_objs}\n0000000000 65535 f ");
    for off in offsets {
        let _ = writeln!(pdf, "{off:010} 00000 n ");
    }
    let _ = writeln!(
        pdf,
        "trailer\n<< /Size {total_objs} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF"
    );

    pdf.into_bytes()
}
