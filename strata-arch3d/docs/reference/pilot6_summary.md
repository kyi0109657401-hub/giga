# 발굴조사보고서 파일럿 6책 페이지 트리아지 및 유물 도면 검출 결과

- **실행 방식**: `PyMuPDF` 96 dpi 렌더링 + OpenCV 선화·채도·연결요소 휴리스틱 (`run_pilot6_triage.py`)
- **원본 경로**: `I:\report\동국참여보고서` (100% 읽기 전용)
- **파생 접촉 시트 경로**: `D:\strata-reference\pilot6\`

| 파일럿 ID | 보고서 파일명 | 비고 | 전체 쪽수 | 텍스트 쪽수 | 검출 유물도면 쪽수 | 검출 개별 크롭 수 | 접촉 시트 |
|---|---|---|---:|---:|---:|---:|---|
| `pilot-01-maeho2` | `매호동2.pdf` | 소규모 (텍스트, 15쪽) | 15 | 15 | **0** | **0** | `-` |
| `pilot-02-gimjeon2` | `청도 김전리2.pdf` | 단행본 (텍스트, 8쪽) | 8 | 8 | **1** | **1** | `pilot-02-gimjeon2_contact_sheet.png` |
| `pilot-03-daeriri` | `제45책 의성 대리리 7호분(2).pdf` | 제45책 (텍스트, 194쪽) | 194 | 183 | **19** | **59** | `pilot-03-daeriri_contact_sheet.png` |
| `pilot-04-jeju-oedo` | `36. 제주 외도일동38-1번지.pdf` | 소규모 (스캔본, 158쪽) | 158 | 0 | **88** | **292** | `pilot-04-jeju-oedo_contact_sheet.png` |
| `pilot-05-namwon1` | `남원 향교동 유적 Ⅰ권.pdf` | 제65책 (텍스트, 591쪽) | 591 | 588 | **155** | **780** | `pilot-05-namwon1_contact_sheet.png` |
| `pilot-06-yeongju-a3` | `영주 한국문화테마파크 조성부지내 유적 보고서.pdf` | 제30책 (텍스트, A3 가로, 234쪽) | 234 | 222 | **25** | **93** | `pilot-06-yeongju-a3_contact_sheet.png` |
