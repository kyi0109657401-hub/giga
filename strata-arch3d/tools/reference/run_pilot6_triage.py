#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
발굴조사보고서 파일럿 6책 페이지 트리아지(Page Triage) 및 유물 도면 검출·접촉 시트 생성 스크립트.
- 대상: I:\\report\\동국참여보고서 내 파일럿 6책 (읽기 전용)
- 방식: PyMuPDF 페이지 렌더링 + 텍스트 캡션/OpenCV 선화·채도·연결요소 휴리스틱 병행
- 산출물:
  1) strata-arch3d/docs/reference/pilot6_triage.json (sort_keys=True)
  2) strata-arch3d/docs/reference/pilot6_summary.md
  3) D:\\strata-reference\\pilot6\\ (접촉 시트 및 표본 크롭 PNG — Git 외부)
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple

import cv2
import numpy as np
import pymupdf as fitz
from PIL import Image, ImageDraw

REPORT_DIR = Path(r"I:\report\동국참여보고서")
DOCS_REF_DIR = Path(__file__).resolve().parents[2] / "docs" / "reference"
DERIVED_PILOT_DIR = Path(r"D:\strata-reference\pilot6")

PILOT_BOOKS: List[Tuple[str, str, str]] = [
    ("pilot-01-maeho2", "매호동2.pdf", "소규모 (텍스트, 15쪽)"),
    ("pilot-02-gimjeon2", "청도 김전리2.pdf", "단행본 (텍스트, 8쪽)"),
    ("pilot-03-daeriri", "제45책 의성 대리리 7호분(2).pdf", "제45책 (텍스트, 194쪽)"),
    ("pilot-04-jeju-oedo", "36. 제주 외도일동38-1번지.pdf", "소규모 (스캔본, 158쪽)"),
    ("pilot-05-namwon1", "남원 향교동 유적 Ⅰ권.pdf", "제65책 (텍스트, 591쪽)"),
    (
        "pilot-06-yeongju-a3",
        "영주 한국문화테마파크 조성부지내 유적 보고서.pdf",
        "제30책 (텍스트, A3 가로, 234쪽)",
    ),
]

RE_DRAWING_CAP = re.compile(r"(도면\s*\d+|출토\s*유물|유물\s*실측|토기|자기|기와|철기|석기)")
RE_FEATURE_ONLY = re.compile(r"(유구\s*배치도|토층도|평·단면도|평단면도|위치도|지형도)")
RE_PHOTO_PLATE = re.compile(r"^\s*도판\s*\d+")


def render_page_rgb(page: fitz.Page, dpi: int = 96) -> np.ndarray:
    pix = page.get_pixmap(dpi=dpi, alpha=False)
    arr = np.frombuffer(pix.samples, dtype=np.uint8).reshape(pix.height, pix.width, 3)
    return arr


def analyze_page_vision(rgb: np.ndarray) -> Tuple[bool, Dict[str, float], List[List[int]]]:
    """
    OpenCV 선화/레이아웃 휴리스틱으로 유물 도면 여부와 개별 유물 크롭 박스 목록을 검출한다.
    - 사진 도판 배제: 평균 채도(Saturation) 또는 중간 계조 회색 면적이 높으면 사진 도판으로 분류.
    - 유물 선도 판별: 백색 배경 비율이 높고(0.68~0.985), 세선 에지 밀도와 중·소형 연결 요소 분포가 유물 도면 범위에 해당.
    """
    h, w, _ = rgb.shape
    hsv = cv2.cvtColor(rgb, cv2.COLOR_RGB2HSV)
    gray = cv2.cvtColor(rgb, cv2.COLOR_RGB2GRAY)

    # 상하단 여백(머리말/쪽번호 7%) 제외한 본문 영역 분석
    y0, y1 = int(h * 0.07), int(h * 0.93)
    x0, x1 = int(w * 0.05), int(w * 0.95)
    roi_gray = gray[y0:y1, x0:x1]
    roi_sat = hsv[y0:y1, x0:x1, 1]

    white_ratio = float(np.mean(roi_gray > 232))
    dark_ink_ratio = float(np.mean(roi_gray < 140))
    mid_tone_ratio = float(np.mean((roi_gray >= 80) & (roi_gray <= 200)))
    mean_sat = float(np.mean(roi_sat))

    edges = cv2.Canny(roi_gray, 60, 160)
    edge_ratio = float(np.mean(edges > 0))

    # 이진화 및 모폴로지 클로징으로 개별 도면 블록 검출
    inv = (roi_gray < 210).astype(np.uint8) * 255
    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (15, 15))
    closed = cv2.morphologyEx(inv, cv2.MORPH_CLOSE, kernel)
    num_labels, _, stats, _ = cv2.connectedComponentsWithStats(closed, connectivity=8)

    roi_area = float((y1 - y0) * (x1 - x0))
    crop_boxes: List[List[int]] = []
    valid_component_areas: List[float] = []

    for lbl in range(1, num_labels):
        bx, by, bw, bh, area = stats[lbl]
        box_area = float(bw * bh)
        rel_area = box_area / max(roi_area, 1.0)
        if 0.008 <= rel_area <= 0.75 and bw >= 36 and bh >= 36:
            sub_gray = roi_gray[by : by + bh, bx : bx + bw]
            sub_mid = float(np.mean((sub_gray >= 70) & (sub_gray <= 190)))
            sub_white = float(np.mean(sub_gray > 230))
            # 실사 사진 블록(중간 회색조가 넓게 깔린 영역) 배제
            if sub_white >= 0.52 and sub_mid <= 0.35:
                pad = 8
                gx0 = max(0, x0 + int(bx) - pad)
                gy0 = max(0, y0 + int(by) - pad)
                gx1 = min(w, x0 + int(bx + bw) + pad)
                gy1 = min(h, y0 + int(by + bh) + pad)
                crop_boxes.append([gx0, gy0, gx1, gy1])
                valid_component_areas.append(rel_area)

    crop_boxes.sort(key=lambda b: (b[1] // 60, b[0]))

    is_line_drawing = (
        0.65 <= white_ratio <= 0.992
        and mean_sat <= 24.0
        and mid_tone_ratio <= 0.22
        and 0.002 <= edge_ratio <= 0.12
        and 0.001 <= dark_ink_ratio <= 0.18
        and len(crop_boxes) >= 1
    )

    metrics = {
        "crop_count": float(len(crop_boxes)),
        "dark_ink_ratio": round(dark_ink_ratio, 4),
        "edge_ratio": round(edge_ratio, 4),
        "mean_sat": round(mean_sat, 2),
        "mid_tone_ratio": round(mid_tone_ratio, 4),
        "white_ratio": round(white_ratio, 4),
    }
    return is_line_drawing, metrics, crop_boxes


def classify_book_pages(
    book_id: str, pdf_path: Path
) -> Tuple[Dict[str, Any], List[Tuple[int, np.ndarray, List[List[int]]]]]:
    doc = fitz.open(str(pdf_path))
    try:
        total_pages = len(doc)
        first_rect = doc[0].rect if total_pages > 0 else fitz.Rect(0, 0, 0, 0)
        text_pages_count = 0
        candidate_indices: List[int] = []

        # 1차: 텍스트 레이어 유무 확인 및 후보 쪽 선정
        for pno in range(total_pages):
            txt = doc[pno].get_text("text").strip()
            if len(txt) >= 10:
                text_pages_count += 1
            if txt:
                if RE_PHOTO_PLATE.search(txt):
                    continue
                if RE_DRAWING_CAP.search(txt) and not RE_FEATURE_ONLY.search(txt):
                    candidate_indices.append(pno)

        is_scan_book = text_pages_count < max(3, int(total_pages * 0.1))
        if is_scan_book or len(candidate_indices) == 0:
            # 스캔본이거나 캡션 매칭이 적은 소형 보고서는 전체 쪽(또는 후반부 도면 섹션)을 비전 스캔
            candidate_indices = list(range(total_pages))

        artifact_pages: List[Dict[str, Any]] = []
        visual_samples: List[Tuple[int, np.ndarray, List[List[int]]]] = []
        total_crops = 0

        for pno in candidate_indices:
            page = doc[pno]
            txt = page.get_text("text").strip()
            # 본문 텍스트가 지나치게 긴 줄글 페이지(예: 900자 초과)는 제외
            if len(txt) > 900:
                continue

            rgb = render_page_rgb(page, dpi=96)
            is_drawing, metrics, crop_boxes = analyze_page_vision(rgb)

            # 텍스트가 있는 보고서에서 '도판 N' 사진 쪽이나 유구 전용 도면은 추가 배제
            if txt and RE_FEATURE_ONLY.search(txt) and "출토유물" not in txt and "토기" not in txt:
                continue

            if is_drawing:
                total_crops += len(crop_boxes)
                artifact_pages.append(
                    {
                        "bboxes_px": crop_boxes,
                        "crop_count": len(crop_boxes),
                        "metrics": metrics,
                        "pdf_page": pno + 1,
                    }
                )
                if len(visual_samples) < 12:
                    visual_samples.append((pno + 1, rgb, crop_boxes))

        summary = {
            "artifact_drawing_pages_count": len(artifact_pages),
            "artifact_pages": artifact_pages,
            "book_id": book_id,
            "detected_artifact_crops": total_crops,
            "file_name": pdf_path.name,
            "is_scan_book": is_scan_book,
            "page_size_pt": [round(first_rect.width, 1), round(first_rect.height, 1)],
            "source_path": str(pdf_path),
            "text_pages_count": text_pages_count,
            "total_pages": total_pages,
        }
        return summary, visual_samples
    finally:
        doc.close()


def save_contact_sheet_and_crops(
    book_id: str,
    visual_samples: List[Tuple[int, np.ndarray, List[List[int]]]],
    out_dir: Path,
) -> Optional[str]:
    if not visual_samples:
        return None
    out_dir.mkdir(parents=True, exist_ok=True)
    crops_dir = out_dir / "crops"
    crops_dir.mkdir(parents=True, exist_ok=True)

    thumb_w, thumb_h = 260, 360
    cols = min(4, len(visual_samples))
    rows = (len(visual_samples) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * thumb_w, rows * thumb_h), (245, 244, 240))

    saved_crops = 0
    for idx, (pno, rgb, boxes) in enumerate(visual_samples):
        pil_img = Image.fromarray(rgb)
        draw = ImageDraw.Draw(pil_img)
        for bx0, by0, bx1, by1 in boxes:
            draw.rectangle([bx0, by0, bx1, by1], outline=(210, 40, 40), width=2)
            if saved_crops < 5:
                crop_im = Image.fromarray(rgb[by0:by1, bx0:bx1])
                crop_path = crops_dir / f"{book_id}_p{pno:04d}_{saved_crops + 1:02d}.png"
                crop_im.save(crop_path)
                saved_crops += 1

        thumb = pil_img.resize((thumb_w - 12, thumb_h - 28), Image.Resampling.BILINEAR)
        c = idx % cols
        r = idx // cols
        ox = c * thumb_w + 6
        oy = r * thumb_h + 22
        sheet.paste(thumb, (ox, oy))
        sdraw = ImageDraw.Draw(sheet)
        sdraw.text((ox + 4, r * thumb_h + 4), f"p.{pno} ({len(boxes)} crops)", fill=(20, 20, 19))

    sheet_path = out_dir / f"{book_id}_contact_sheet.png"
    sheet.save(sheet_path)
    return str(sheet_path)


def main() -> int:
    DOCS_REF_DIR.mkdir(parents=True, exist_ok=True)
    DERIVED_PILOT_DIR.mkdir(parents=True, exist_ok=True)

    results: List[Dict[str, Any]] = []
    md_lines = [
        "# 발굴조사보고서 파일럿 6책 페이지 트리아지 및 유물 도면 검출 결과",
        "",
        "- **실행 방식**: `PyMuPDF` 96 dpi 렌더링 + OpenCV 선화·채도·연결요소 휴리스틱 (`run_pilot6_triage.py`)",
        "- **원본 경로**: `I:\\report\\동국참여보고서` (100% 읽기 전용)",
        "- **파생 접촉 시트 경로**: `D:\\strata-reference\\pilot6\\`",
        "",
        "| 파일럿 ID | 보고서 파일명 | 비고 | 전체 쪽수 | 텍스트 쪽수 | 검출 유물도면 쪽수 | 검출 개별 크롭 수 | 접촉 시트 |",
        "|---|---|---|---:|---:|---:|---:|---|",
    ]

    for book_id, fname, note in PILOT_BOOKS:
        pdf_path = REPORT_DIR / fname
        if not pdf_path.exists():
            print(f"WARN: Missing pilot file: {pdf_path}", file=sys.stderr)
            continue
        summary, samples = classify_book_pages(book_id, pdf_path)
        sheet_path = save_contact_sheet_and_crops(book_id, samples, DERIVED_PILOT_DIR)
        summary["contact_sheet_path"] = sheet_path
        summary["note"] = note
        results.append(summary)

        sheet_name = Path(sheet_path).name if sheet_path else "-"
        md_lines.append(
            f"| `{book_id}` | `{fname}` | {note} | {summary['total_pages']} | {summary['text_pages_count']} | **{summary['artifact_drawing_pages_count']}** | **{summary['detected_artifact_crops']}** | `{sheet_name}` |"
        )

    results.sort(key=lambda x: x["book_id"])
    payload = {
        "pilot_books_count": len(results),
        "results": results,
        "schema_version": "1.0",
    }

    json_path = DOCS_REF_DIR / "pilot6_triage.json"
    json_path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    md_path = DOCS_REF_DIR / "pilot6_summary.md"
    md_path.write_text("\n".join(md_lines) + "\n", encoding="utf-8")

    print(f"OK: Generated {json_path.name} and {md_path.name} ({len(results)} books)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
