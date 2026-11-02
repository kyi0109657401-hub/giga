//! 화면 UI 전용 다크 모드 테마 (`UI_RULES.md` R2 준수).
//!
//! - 기존 라이트 토큰 체계(`strata_core::strata_tokens::tokens`)는 그대로 유지한다.
//! - 다크 팔레트는 이 모듈의 `DARK_*` 상수(화면 UI 전용, 한국어 이름 병기)에만 둔다.
//! - **철칙**: 실측 도판 출력물(용지·방안지·제도선·탁본·치수·라벨) 색상은 항상 라이트 기준.
//!   `UiChrome::paper`·출력 계열 색상은 모드와 무관하게 라이트 토큰 값을 돌려준다.
//!   화면 UI(상단 바·중앙 여백·3D 뷰포트 배경·대화상자·칩이 아닌 바탕 텍스트)만 다크로 바뀐다.

use eframe::egui;
use strata_core::strata_tokens::tokens::{
    ACTION, CLAY_WASH, DESK, EDGE, GROUND, INK, LINE, MUTED, SURFACE, WASH,
};

/// 화면 UI 테마 모드.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// 라이트 (기본값).
    Light,
    /// 다크 모드 (화면 UI만, 출력물은 라이트 유지).
    Dark,
}

impl ThemeMode {
    /// 불리언 플래그를 테마 모드로 바꾼다.
    pub fn from_bool(dark_mode: bool) -> Self {
        if dark_mode {
            Self::Dark
        } else {
            Self::Light
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    /// 상단 바 체크박스용 한국어 라벨.
    ///
    /// 체크박스는 모드와 무관하게 같은 문구를 쓴다(옵션 이름이지 상태 표시가 아니다).
    pub fn toggle_label_kr(self) -> &'static str {
        "다크 모드"
    }
}

/// 다크 팔레트 (화면 UI 전용).
///
/// - 밤종이 바탕 `#23211D` · 작업대 `#161513` · 달빛 한지 글자 `#EAE4D6`.
/// - 흙(테라코타) 강조는 라이트와 같은 `#B5573A`를 써서 강조색 일관성을 유지한다.
pub mod dark {
    /// 밤종이 (패널·창 바탕).
    pub const PANEL: [u8; 3] = [0x23, 0x21, 0x1D];
    /// 밤 작업대 (중앙 여백·뷰포트 바탕).
    pub const DESK: [u8; 3] = [0x16, 0x15, 0x13];
    /// 달빛 한지 (기본 글자).
    pub const INK: [u8; 3] = [0xEA, 0xE4, 0xD6];
    /// 달빛 보조 글자.
    pub const MUTED: [u8; 3] = [0xA3, 0x9C, 0x8D];
    /// 흐린 글자.
    pub const FAINT: [u8; 3] = [0x6E, 0x68, 0x5C];
    /// 밤 구분선.
    pub const LINE: [u8; 3] = [0x3B, 0x37, 0x2F];
    /// 밤 테두리.
    pub const EDGE: [u8; 3] = [0x5A, 0x53, 0x44];
    /// 선택 바탕 (짙은 흙).
    pub const SELECT_BG: [u8; 3] = [0x6B, 0x42, 0x2C];
    /// 선택 글자.
    pub const SELECT_TEXT: [u8; 3] = [0xF5, 0xEF, 0xE2];
    /// 대화상자 오류 글자 (다크 바탕용 밝힘).
    pub const DIALOG_CRIMSON: [u8; 3] = [0xD9, 0x7B, 0x7B];
    /// 대화상자 통과 글자 (다크 바탕용 밝힘).
    pub const DIALOG_JADE: [u8; 3] = [0x8F, 0xB8, 0x7B];
}

/// 모드별 화면 크롬 색상 묶음.
///
/// `paper`는 출력물(흰 용지)이라 항상 라이트 `SURFACE`를 돌려준다 — 다크에서도 흰 종이를 유지한다.
#[derive(Debug, Clone, Copy)]
pub struct UiChrome {
    pub panel: egui::Color32,
    pub window: egui::Color32,
    pub desk: egui::Color32,
    pub ink: egui::Color32,
    pub muted: egui::Color32,
    pub faint: egui::Color32,
    pub line: egui::Color32,
    pub edge: egui::Color32,
    pub select_bg: egui::Color32,
    pub select_text: egui::Color32,
    pub accent: egui::Color32,
    /// 실측 도판 용지 — 항상 흰색 (출력물 불변 철칙).
    pub paper: egui::Color32,
}

fn rgb(c: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

/// 모드에 맞는 화면 크롬 색상을 돌려준다.
pub fn chrome_for(mode: ThemeMode) -> UiChrome {
    if mode.is_dark() {
        UiChrome {
            panel: rgb(dark::PANEL),
            window: rgb(dark::PANEL),
            desk: rgb(dark::DESK),
            ink: rgb(dark::INK),
            muted: rgb(dark::MUTED),
            faint: rgb(dark::FAINT),
            line: rgb(dark::LINE),
            edge: rgb(dark::EDGE),
            select_bg: rgb(dark::SELECT_BG),
            select_text: rgb(dark::SELECT_TEXT),
            accent: rgb(ACTION.rgb),
            paper: rgb(SURFACE.rgb),
        }
    } else {
        UiChrome {
            panel: rgb(GROUND.rgb),
            window: rgb(GROUND.rgb),
            desk: rgb(DESK.rgb),
            ink: rgb(INK.rgb),
            muted: rgb(MUTED.rgb),
            faint: rgb(MUTED.rgb),
            line: rgb(LINE.rgb),
            edge: rgb(EDGE.rgb),
            select_bg: rgb(CLAY_WASH.rgb),
            select_text: rgb(INK.rgb),
            accent: rgb(ACTION.rgb),
            paper: rgb(SURFACE.rgb),
        }
    }
}

/// `egui` 전역 스타일을 모드에 맞게 적용한다.
///
/// 스톡 위젯(버튼·입력·스크롤·기본 글자)은 `Visuals`가 맡고,
/// 명시적 바탕 칠(상단 바·중앙 여백·뷰포트)은 각 렌더러가 `chrome_for` 값을 쓴다.
pub fn apply_app_visuals(ctx: &egui::Context, mode: ThemeMode) {
    let chrome = chrome_for(mode);
    ctx.all_styles_mut(|style| {
        style.visuals = if mode.is_dark() {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        style.visuals.panel_fill = chrome.panel;
        style.visuals.window_fill = chrome.window;
        style.visuals.selection.bg_fill = chrome.select_bg;
        style.visuals.selection.stroke = egui::Stroke::new(1.0, chrome.select_text);
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, chrome.edge);
    });
}

/// 도판 출력물 색상 묶음 — 모드와 무관하게 항상 라이트 토큰이어야 한다.
///
/// 출력 경로(`strata-export`, 방안지·제도선·탁본)가 테마에 끌려가지 않는지 고정한다.
pub fn plate_output_colors() -> [([u8; 3], &'static str); 4] {
    [
        (SURFACE.rgb, "용지"),
        (INK.rgb, "먹"),
        (WASH.rgb, "옅은 종이"),
        (GROUND.rgb, "종이"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_label_is_korean() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let label = ThemeMode::toggle_label_kr(mode);
            assert_eq!(label, "다크 모드");
            assert!(label.chars().any(|c| ('가'..='힣').contains(&c)));
        }
    }

    #[test]
    fn test_theme_mode_from_bool_roundtrip() {
        assert_eq!(ThemeMode::from_bool(false), ThemeMode::Light);
        assert_eq!(ThemeMode::from_bool(true), ThemeMode::Dark);
        assert!(!ThemeMode::Light.is_dark());
        assert!(ThemeMode::Dark.is_dark());
    }

    #[test]
    fn test_light_chrome_matches_light_tokens() {
        let light = chrome_for(ThemeMode::Light);
        assert_eq!(light.panel, rgb(GROUND.rgb));
        assert_eq!(light.desk, rgb(DESK.rgb));
        assert_eq!(light.ink, rgb(INK.rgb));
        assert_eq!(light.accent, rgb(ACTION.rgb));
    }

    #[test]
    fn test_paper_stays_white_in_both_modes() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let chrome = chrome_for(mode);
            assert_eq!(chrome.paper, egui::Color32::WHITE, "용지는 다크에서도 흰색");
        }
    }

    #[test]
    fn test_dark_chrome_differs_for_workspace_but_not_output() {
        let light = chrome_for(ThemeMode::Light);
        let dark = chrome_for(ThemeMode::Dark);
        assert_ne!(dark.panel, light.panel, "패널은 다크에서 바뀌어야 함");
        assert_ne!(dark.desk, light.desk, "작업대는 다크에서 바뀌어야 함");
        assert_ne!(dark.ink, light.ink, "글자는 다크에서 바뀌어야 함");
        assert_eq!(dark.paper, light.paper, "용지는 바뀌면 안 됨");
        assert_eq!(dark.accent, light.accent, "강조 흙색은 양쪽 동일");
    }

    #[test]
    fn test_plate_output_colors_are_light_tokens() {
        for (rgb_val, name) in plate_output_colors() {
            let token_hit =
                [SURFACE.rgb, INK.rgb, WASH.rgb, GROUND.rgb, DESK.rgb].contains(&rgb_val);
            assert!(token_hit, "출력 색상 {name} 가 라이트 토큰이 아님");
        }
    }

    #[test]
    fn test_apply_app_visuals_does_not_panic() {
        let ctx = egui::Context::default();
        apply_app_visuals(&ctx, ThemeMode::Light);
        apply_app_visuals(&ctx, ThemeMode::Dark);
    }
}
