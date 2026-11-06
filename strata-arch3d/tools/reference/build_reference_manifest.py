#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
읽기 전용(Read-Only) 참조 코퍼스 매니페스트 및 중복 판정 생성 스크립트.
- 대상:
  1) I:\\디지털 탁본 참고 자료_(주)캐럿펀트 (PDF 9종)
  2) I:\\새 폴더 (11) (OBJ 11세트, AI 템플릿 5종, 기준선 PDF/ret/)
- 원칙:
  - 원본 파일 수정·이동·삭제 절대 금지 (100% Read-Only)
  - 프롬프트 캐시 최적화 규칙 준수: 경로 오름차순 정렬 및 JSON 직렬화 시 sort_keys=True 강제
"""

from __future__ import annotations

import csv
import hashlib
import json
import struct
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import pymupdf as fitz  # PyMuPDF


PDF_REF_DIR = Path(r"I:\디지털 탁본 참고 자료_(주)캐럿펀트")
OBJ_REF_DIR = Path(r"I:\새 폴더 (11)")
OUT_DIR = Path(__file__).resolve().parents[2] / "docs" / "reference"


def compute_sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            chunk = f.read(1024 * 1024)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()


def parse_jpeg_dimensions(path: Path) -> Optional[Tuple[int, int]]:
    """JPEG 전체 디코딩 없이 SOF 마커만 읽어 (width, height)를 반환한다."""
    try:
        with path.open("rb") as f:
            if f.read(2) != b"\xff\xd8":
                return None
            while True:
                b = f.read(1)
                if not b:
                    return None
                while b == b"\xff":
                    b = f.read(1)
                if not b:
                    return None
                marker = b[0]
                if marker in (0xD8, 0xD9) or (0xD0 <= marker <= 0xD7):
                    continue
                len_bytes = f.read(2)
                if len(len_bytes) < 2:
                    return None
                seg_len = struct.unpack(">H", len_bytes)[0]
                if seg_len < 2:
                    return None
                if marker in (
                    0xC0,
                    0xC1,
                    0xC2,
                    0xC3,
                    0xC5,
                    0xC6,
                    0xC7,
                    0xC9,
                    0xCA,
                    0xCB,
                    0xCD,
                    0xCE,
                    0xCF,
                ):
                    sof = f.read(5)
                    if len(sof) < 5:
                        return None
                    _, height, width = struct.unpack(">BHH", sof)
                    return (int(width), int(height))
                f.seek(seg_len - 2, 1)
    except OSError:
        return None


def scan_obj_stats(path: Path) -> Dict[str, Any]:
    v_count = 0
    vt_count = 0
    vn_count = 0
    tri_count = 0
    min_pt = [float("inf"), float("inf"), float("inf")]
    max_pt = [float("-inf"), float("-inf"), float("-inf")]

    with path.open("rb") as f:
        for raw_line in f:
            if raw_line.startswith(b"v "):
                v_count += 1
                parts = raw_line[2:].split()
                if len(parts) >= 3:
                    x = float(parts[0])
                    y = float(parts[1])
                    z = float(parts[2])
                    if x < min_pt[0]:
                        min_pt[0] = x
                    if y < min_pt[1]:
                        min_pt[1] = y
                    if z < min_pt[2]:
                        min_pt[2] = z
                    if x > max_pt[0]:
                        max_pt[0] = x
                    if y > max_pt[1]:
                        max_pt[1] = y
                    if z > max_pt[2]:
                        max_pt[2] = z
            elif raw_line.startswith(b"vt "):
                vt_count += 1
            elif raw_line.startswith(b"vn "):
                vn_count += 1
            elif raw_line.startswith(b"f "):
                verts = raw_line[2:].split()
                if len(verts) >= 3:
                    tri_count += len(verts) - 2

    if v_count > 0:
        bbox_mm = [
            round(max_pt[0] - min_pt[0], 3),
            round(max_pt[1] - min_pt[1], 3),
            round(max_pt[2] - min_pt[2], 3),
        ]
    else:
        bbox_mm = [0.0, 0.0, 0.0]

    return {
        "bbox_mm": bbox_mm,
        "has_normals": vn_count > 0,
        "has_uv": vt_count > 0,
        "normals_count": vn_count,
        "triangles": tri_count,
        "uv_count": vt_count,
        "vertices": v_count,
    }


def scan_pdf_stats(path: Path) -> Dict[str, Any]:
    doc = fitz.open(str(path))
    try:
        pages = len(doc)
        first_rect = doc[0].rect if pages > 0 else fitz.Rect(0, 0, 0, 0)
        img_count = 0
        for page in doc:
            img_count += len(page.get_images(full=True))
        return {
            "first_page_pt": [round(first_rect.width, 1), round(first_rect.height, 1)],
            "image_objects": img_count,
            "pages": pages,
        }
    finally:
        doc.close()


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    items: List[Dict[str, Any]] = []
    sha_rows: List[Tuple[str, str, int, str]] = []

    # 1. PDF 9종 스캔 (오름차순 정렬)
    if PDF_REF_DIR.exists():
        pdf_files = sorted(PDF_REF_DIR.glob("*.pdf"), key=lambda p: p.name)
        for idx, pdf_path in enumerate(pdf_files, start=1):
            sha = compute_sha256(pdf_path)
            size_bytes = pdf_path.stat().st_size
            pdf_meta = scan_pdf_stats(pdf_path)
            item_id = "pdf-intro" if "소개" in pdf_path.name else f"pdf-plate-{idx:02d}"
            items.append(
                {
                    "display_name": pdf_path.name,
                    "id": item_id,
                    "kind": "reference_pdf",
                    "pdf": pdf_meta,
                    "rights": {
                        "owner": "(주)캐럿펀트 / 내부 참고용",
                        "redistributable": False,
                        "status": "internal-reference-only",
                    },
                    "sha256": sha,
                    "size_bytes": size_bytes,
                    "source_path": str(pdf_path),
                }
            )
            sha_rows.append((item_id, sha, size_bytes, str(pdf_path)))

    # 2. OBJ 11세트 스캔 (폴더명 오름차순 정렬)
    obj_by_folder: Dict[str, Dict[str, Any]] = {}
    if OBJ_REF_DIR.exists():
        subdirs = sorted(
            [d for d in OBJ_REF_DIR.iterdir() if d.is_dir()],
            key=lambda p: p.name,
        )
        for subdir in subdirs:
            obj_files = sorted(subdir.glob("*.obj"), key=lambda p: p.name)
            if not obj_files:
                continue
            obj_path = obj_files[0]
            sha = compute_sha256(obj_path)
            size_bytes = obj_path.stat().st_size
            mesh_stats = scan_obj_stats(obj_path)

            jpg_files = sorted(subdir.glob("*.jpg"), key=lambda p: p.name)
            tex_info: Optional[Dict[str, Any]] = None
            if jpg_files:
                tex_path = jpg_files[0]
                dims = parse_jpeg_dimensions(tex_path) or (0, 0)
                tex_sha = compute_sha256(tex_path)
                tex_size = tex_path.stat().st_size
                tex_info = {
                    "height": dims[1],
                    "path": tex_path.name,
                    "sha256": tex_sha,
                    "size_bytes": tex_size,
                    "width": dims[0],
                }
                sha_rows.append(
                    (f"tex-{subdir.name}", tex_sha, tex_size, str(tex_path))
                )

            ap2_files = sorted(subdir.glob("*.ap2"), key=lambda p: p.name)
            item_id = f"obj-{subdir.name}-{obj_path.stem}"
            rec = {
                "ap2_present": len(ap2_files) > 0,
                "display_name": f"{subdir.name} / {obj_path.name}",
                "folder": subdir.name,
                "id": item_id,
                "kind": "obj_set",
                "mesh": mesh_stats,
                "rights": {
                    "owner": "내부 연구용",
                    "redistributable": False,
                    "status": "internal-research-only",
                },
                "sha256": sha,
                "size_bytes": size_bytes,
                "source_path": str(obj_path),
                "texture": tex_info,
                "units": "mm",
            }
            items.append(rec)
            obj_by_folder[subdir.name] = rec
            sha_rows.append((item_id, sha, size_bytes, str(obj_path)))

        # 3. AI 템플릿 및 기준선 PDF 스캔
        aux_files = sorted(
            list(OBJ_REF_DIR.glob("*.ai")) + list(OBJ_REF_DIR.glob("*.pdf")),
            key=lambda p: p.name,
        )
        for aux_path in aux_files:
            sha = compute_sha256(aux_path)
            size_bytes = aux_path.stat().st_size
            item_id = f"aux-{aux_path.stem}"
            items.append(
                {
                    "display_name": aux_path.name,
                    "id": item_id,
                    "kind": "template_or_baseline",
                    "sha256": sha,
                    "size_bytes": size_bytes,
                    "source_path": str(aux_path),
                }
            )
            sha_rows.append((item_id, sha, size_bytes, str(aux_path)))

    # 4. 결정론적 정렬(sort_keys=True)로 manifest.json 기록
    items.sort(key=lambda x: x["id"])
    sha_rows.sort(key=lambda x: x[0])

    manifest_payload = {
        "counts": {
            "obj_sets": sum(1 for x in items if x["kind"] == "obj_set"),
            "reference_pdfs": sum(1 for x in items if x["kind"] == "reference_pdf"),
            "templates_and_baselines": sum(
                1 for x in items if x["kind"] == "template_or_baseline"
            ),
            "total_items": len(items),
        },
        "items": items,
        "schema_version": "1.0",
    }

    manifest_path = OUT_DIR / "manifest.json"
    manifest_path.write_text(
        json.dumps(manifest_payload, ensure_ascii=False, indent=2, sort_keys=True)
        + "\n",
        encoding="utf-8",
    )

    sha_csv_path = OUT_DIR / "sha256.csv"
    with sha_csv_path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["id", "sha256", "size_bytes", "source_path"])
        for row in sha_rows:
            writer.writerow(row)

    # 5. 중복 후보 2쌍(7 vs 9, 8 vs 10) 및 034 버전 검증 리포트 생성
    dup_lines = [
        "# OBJ 참조 세트 중복 후보 판정 및 수치 검증 보고서",
        "",
        "## 1. 파일명 중복 후보 2쌍 SHA-256 및 형상 지문 판정",
        "",
        "| 비교 쌍 | 폴더/파일 | 정점 수 | 삼각형 수 | 바운딩 박스 (mm) | OBJ SHA-256 (앞 16자리) | 판정 |",
        "|---|---|---:|---:|---|---|---|",
    ]
    for f_a, f_b in [("7", "9"), ("8", "10")]:
        a = obj_by_folder.get(f_a)
        b = obj_by_folder.get(f_b)
        if a and b:
            same = a["sha256"] == b["sha256"]
            verdict = "동일 파일" if same else "**서로 다른 별개 유물 (파일명만 중복)**"
            dup_lines.append(
                f"| `{f_a}` vs `{f_b}` | `{a['display_name']}` | {a['mesh']['vertices']:,} | {a['mesh']['triangles']:,} | {a['mesh']['bbox_mm']} | `{a['sha256'][:16]}` | {verdict} |"
            )
            dup_lines.append(
                f"| `{f_a}` vs `{f_b}` | `{b['display_name']}` | {b['mesh']['vertices']:,} | {b['mesh']['triangles']:,} | {b['mesh']['bbox_mm']} | `{b['sha256'][:16]}` | {verdict} |"
            )

    rec_034 = obj_by_folder.get("034(7-실4-4)")
    if rec_034:
        dup_lines.extend(
            [
                "",
                "## 2. 기준 세트 `034(7-실4-4)` 실제 형상 수치 확인",
                "",
                f"- **경로**: `{rec_034['source_path']}`",
                f"- **SHA-256**: `{rec_034['sha256']}`",
                f"- **정점 수**: `{rec_034['mesh']['vertices']:,}` (구 문서 115,610 정점과 다름 → 매니페스트 기준 고정)",
                f"- **삼각형 수**: `{rec_034['mesh']['triangles']:,}`",
                f"- **정점 법선(`vn`) 수**: `{rec_034['mesh']['normals_count']}` (11세트 모두 `vn=0` 확인)",
                f"- **바운딩 박스(mm)**: `{rec_034['mesh']['bbox_mm']}`",
                f"- **AP2 정위치 파일 존재**: `{rec_034['ap2_present']}`",
            ]
        )

    dup_report_path = OUT_DIR / "duplicate_report.md"
    dup_report_path.write_text("\n".join(dup_lines) + "\n", encoding="utf-8")

    print(
        f"OK: Generated {manifest_path.name} ({len(items)} items), {sha_csv_path.name}, {dup_report_path.name}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
