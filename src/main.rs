#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/base_token_colors.rs"));

use anyhow::{Result, anyhow};
use palette::{FromColor, IntoColor, Oklab, Oklch, Srgb};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

macro_rules! token {
    ($scope:expr, $fg:expr) => {
        TokenColor {
            name: None,
            scope: $scope.into(),
            settings: TokenSettings {
                foreground: Some(hex($fg)),
                ..Default::default()
            },
        }
    };
    ($name:expr, $scope:expr, $fg:expr) => {
        TokenColor {
            name: Some($name.into()),
            scope: $scope.into(),
            settings: TokenSettings {
                foreground: Some(hex($fg)),
                ..Default::default()
            },
        }
    };
}

macro_rules! token_full {
    ($scope:expr, $fg:expr, $style:expr) => {
        TokenColor {
            name: None,
            scope: $scope.into(),
            settings: TokenSettings {
                foreground: $fg.map(|c| hex(c)),
                font_style: $style.map(|s| s.to_string()),
                ..Default::default()
            },
        }
    };
    ($name:expr, $scope:expr, $fg:expr, $style:expr) => {
        TokenColor {
            name: Some($name.into()),
            scope: $scope.into(),
            settings: TokenSettings {
                foreground: $fg.map(|c| hex(c)),
                font_style: $style.map(|s| s.to_string()),
                ..Default::default()
            },
        }
    };
}

macro_rules! update_token {
    ($token:expr, fg: $fg:expr) => {
        TokenColor {
            settings: TokenSettings {
                foreground: Some($fg.to_string()),
                ..$token.settings.clone()
            },
            ..$token.clone()
        }
    };

    ($token:expr, style: $style:expr) => {
        TokenColor {
            settings: TokenSettings {
                font_style: Some($style.to_string()),
                ..$token.settings.clone()
            },
            ..$token.clone()
        }
    };

    ($token:expr, fg: $fg:expr, style: $style:expr) => {
        TokenColor {
            settings: TokenSettings {
                foreground: Some($fg.to_string()),
                font_style: Some($style.to_string()),
                ..$token.settings.clone()
            },
            ..$token.clone()
        }
    };
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Primitives {
    fg: StateColors,
    canvas: Canvas,
    border: Border,
    neutral: StateColors,
    accent: StateColors,
    success: StateColors,
    attention: StateColors,
    danger: StateColors,
    scale: Scale,
    severe: StateColors,
    ansi: Ansi,
    btn: Btn,
    codemirror: Codemirror,
    primer: Primer,
}

#[derive(Deserialize, Debug)]
struct StateColors {
    #[serde(default)]
    default: String,
    #[serde(default)]
    fg: String,
    #[serde(default)]
    muted: String,
    #[serde(default)]
    subtle: String,
    #[serde(default)]
    emphasis: String,
    #[serde(default, rename = "onEmphasis")]
    on_emphasis: String,
}

#[derive(Deserialize, Debug)]
struct Canvas {
    default: String,
    overlay: String,
    inset: String,
    subtle: String,
}

#[derive(Deserialize, Debug)]
struct Border {
    default: String,
    muted: String,
}

#[derive(Deserialize, Debug)]
struct Scale {
    black: String,
    white: String,
    gray: Vec<String>,
    blue: Vec<String>,
    green: Vec<String>,
    yellow: Vec<String>,
    orange: Vec<String>,
    red: Vec<String>,
    purple: Vec<String>,
    pink: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Btn {
    bg: String,
    hover_bg: String,
    primary: BtnState,
    active_bg: String,
    text: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BtnState {
    bg: String,
    text: String,
    hover_bg: String,
}

#[derive(Deserialize, Debug)]
struct Codemirror {
    #[serde(rename = "activelineBg")]
    activeline_bg: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Ansi {
    black: String,
    black_bright: String,
    white: String,
    white_bright: String,
    gray: String,
    red: String,
    red_bright: String,
    green: String,
    green_bright: String,
    yellow: String,
    yellow_bright: String,
    blue: String,
    blue_bright: String,
    magenta: String,
    magenta_bright: String,
    cyan: String,
    cyan_bright: String,
}

#[derive(Deserialize, Debug)]
struct Primer {
    border: PrimerBorder,
}

#[derive(Deserialize, Debug)]
struct PrimerBorder {
    active: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Theme {
    name: String,
    colors: BTreeMap<String, String>,
    semantic_highlighting: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_token_colors: Option<BTreeMap<String, String>>,
    token_colors: Vec<TokenColor>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TokenColor {
    #[serde(default)]
    name: Option<String>,
    scope: Scope,
    settings: TokenSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum Scope {
    Single(String),
    Multiple(Vec<String>),
}

impl Scope {
    fn contains_keyword(&self, keyword: &str) -> bool {
        match self {
            Scope::Single(s) => s.contains(keyword),
            Scope::Multiple(v) => v.iter().any(|s| s.contains(keyword)),
        }
    }
}

impl From<&str> for Scope {
    fn from(s: &str) -> Self {
        Scope::Single(s.to_string())
    }
}

impl From<Vec<&str>> for Scope {
    fn from(v: Vec<&str>) -> Self {
        Scope::Multiple(v.into_iter().map(|s| s.to_string()).collect())
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct TokenSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    foreground: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font_style: Option<String>,
}

#[derive(Deserialize)]
struct BaseThemeTokens {
    #[serde(rename = "tokenColors")]
    token_colors: Vec<TokenColor>,

    #[serde(
        rename = "semanticTokenColors",
        skip_serializing_if = "Option::is_none"
    )]
    semantic_token_colors: Option<BTreeMap<String, String>>,
}

const ACCENT: &str = "#3d5fcf";
const COMMENT: &str = "#6A9955";
const BRIGHT_ACCENT_TEXT: &str = "#85a5ff";
const DIFF_GREEN: &str = "#48e477";
const DIFF_RED: &str = "#f85149";
const SOFT_DARK_BG: &str = "#080b10";
const SOFT_WHITE: &str = "#e9e9e9";
const DIM_WHITE: &str = "#b1bac4";
const ERROR: &str = "#eb5353";
const WARNING: &str = "#e4a341";
const INFO: &str = "#3794ff";


#[rustfmt::skip]
fn main() -> Result<()> {
    let raw_colors_str = include_str!("../data/dark_colorblind.json");
    let p: Primitives = serde_json::from_str(raw_colors_str)?;
    let mut colors = BTreeMap::new();


    // Base Colors & Accents
    add(&mut colors, "focusBorder", ACCENT);
    add(&mut colors, "foreground", &p.fg.default);
    add(&mut colors, "descriptionForeground", &p.fg.muted);
    add(&mut colors, "errorForeground", &p.danger.fg);
    add(&mut colors, "widget.shadow", "rgba(1,4,9,0.85)");


    // Links & Text
    add(&mut colors, "textLink.foreground", BRIGHT_ACCENT_TEXT);
    add(&mut colors, "textLink.activeForeground", BRIGHT_ACCENT_TEXT);


    // Buttons, Inputs & Badges
    add(&mut colors, "button.background", ACCENT);
    add(&mut colors, "button.foreground", &p.btn.primary.text);
    add(&mut colors, "button.hoverBackground", &p.btn.primary.hover_bg);
    add(&mut colors, "badge.background", ACCENT);
    add(&mut colors, "progressBar.background", ACCENT);

    add(&mut colors, "input.background", &p.canvas.default);
    add(&mut colors, "input.border", &p.border.default);
    add(&mut colors, "input.foreground", &p.fg.default);
    add(&mut colors, "input.placeholderForeground", &p.fg.subtle);

    add(&mut colors, "dropdown.background", &p.canvas.overlay);
    add(&mut colors, "dropdown.border", &p.border.default);
    add(&mut colors, "dropdown.foreground", &p.fg.default);


    // Lists & Trees
    add(&mut colors, "list.highlightForeground", BRIGHT_ACCENT_TEXT);
    add(&mut colors, "list.activeSelectionForeground", BRIGHT_ACCENT_TEXT);
    add_alpha(&mut colors, "list.hoverBackground", &p.neutral.subtle, 0.5);
    add_alpha(&mut colors, "list.activeSelectionBackground", ACCENT, 0.2);
    add_alpha(&mut colors, "list.inactiveSelectionBackground", &p.neutral.muted, 0.5);
    add(&mut colors, "list.dropBackground", &p.neutral.muted);


    // Workbench: Activity Bar
    add(&mut colors, "activityBar.background", &p.canvas.default);
    add(&mut colors, "activityBar.foreground", &p.fg.default);
    add(&mut colors, "activityBar.inactiveForeground", &p.fg.muted);
    add(&mut colors, "activityBar.border", &p.border.default);
    add(&mut colors, "activityBar.activeBorder", ACCENT);
    add(&mut colors, "activityBarBadge.background", ACCENT);


    // Workbench: Side Bar
    add(&mut colors, "sideBar.background", SOFT_DARK_BG);
    add(&mut colors, "sideBar.foreground", &p.fg.default);
    add(&mut colors, "sideBar.border", &p.border.default);
    add(&mut colors, "sideBarTitle.foreground", &p.fg.muted);
    add(&mut colors, "sideBarSectionHeader.background", SOFT_DARK_BG);


    // Workbench: Editor Groups & Tabs
    add(&mut colors, "editorGroup.border", &p.border.default);
    add(&mut colors, "editorGroupHeader.tabsBorder", &p.border.default);
    add(&mut colors, "editorGroupHeader.tabsBackground", SOFT_DARK_BG);

    add(&mut colors, "tab.border", &p.border.default);
    add(&mut colors, "tab.activeBorderTop", ACCENT);
    add(&mut colors, "tab.activeBackground", &p.canvas.default);
    add(&mut colors, "tab.activeForeground", &p.fg.default);
    add(&mut colors, "tab.inactiveBackground", SOFT_DARK_BG);
    add(&mut colors, "tab.inactiveForeground", &p.fg.muted);
    add(&mut colors, "tab.hoverBackground", &p.canvas.default);


    // Editor: Basic
    add(&mut colors, "editor.background", &p.canvas.default);
    add(&mut colors, "editor.foreground", &p.fg.default);
    add(&mut colors, "editorCursor.foreground", ACCENT);
    add(&mut colors, "editorLineNumber.foreground", &p.fg.subtle);
    add(&mut colors, "editorLineNumber.activeForeground", &p.fg.default);
    add(&mut colors, "editorWidget.background", &p.canvas.overlay);
    add(&mut colors, "editor.lineHighlightBackground", &p.codemirror.activeline_bg);
    add(&mut colors, "editorWarning.foreground", WARNING);
    add(&mut colors, "editorError.foreground", ERROR);
    add(&mut colors, "editorInfo.foreground", INFO);
    add(&mut colors, "problemsWarningIcon.foreground", WARNING);
    add(&mut colors, "problemsErrorIcon.foreground", ERROR);
    add(&mut colors, "problemsInfoIcon.foreground", INFO);



    // Editor: Selection & Find
    add_alpha(&mut colors, "editor.selectionBackground", ACCENT, 0.2);
    add_alpha(&mut colors, "editor.inactiveSelectionBackground", ACCENT, 0.1);
    add_alpha(&mut colors, "editor.selectionHighlightBackground", &p.success.fg, 0.25);
    add_alpha(&mut colors, "editor.wordHighlightBackground", &p.neutral.subtle, 0.5);

    // Search highlights (Find / Replace)
    add(&mut colors, "editor.findMatchBackground", &p.attention.emphasis);
    add_alpha(&mut colors, "editor.findMatchHighlightBackground", &p.attention.fg, 0.3);


    // Editor: Scrollbar & Features
    add(&mut colors, "editorStickyScroll.background", &p.canvas.default);
    add(&mut colors, "editorStickyScrollHover.background", &p.canvas.subtle);

    // Scrollbar
    add_alpha(&mut colors, "scrollbarSlider.background", &p.neutral.emphasis, 0.2);
    add_alpha(&mut colors, "scrollbarSlider.hoverBackground", &p.neutral.emphasis, 0.3);
    add_alpha(&mut colors, "scrollbarSlider.activeBackground", &p.neutral.emphasis, 0.4);


    // Version Control & Diff
    add(&mut colors, "gitDecoration.addedResourceForeground", DIFF_GREEN);
    add(&mut colors, "gitDecoration.untrackedResourceForeground", DIFF_GREEN);

    add(&mut colors, "gitDecoration.modifiedResourceForeground", BRIGHT_ACCENT_TEXT);

    add(&mut colors, "gitDecoration.deletedResourceForeground", DIFF_RED);
    add(&mut colors, "gitDecoration.ignoredResourceForeground", &p.fg.subtle);
    add(&mut colors, "gitDecoration.conflictingResourceForeground", &p.severe.fg);

    add_alpha(&mut colors, "diffEditor.insertedTextBackground", DIFF_GREEN, 0.15);
    add_alpha(&mut colors, "diffEditor.removedTextBackground", DIFF_RED, 0.15);

    add(&mut colors, "editorGutter.addedBackground", DIFF_GREEN);
    add(&mut colors, "editorGutter.modifiedBackground", BRIGHT_ACCENT_TEXT);
    add(&mut colors, "editorGutter.deletedBackground", DIFF_RED);


    // Workbench: Misc
    add(&mut colors, "statusBar.background", &p.canvas.default);
    add(&mut colors, "statusBar.foreground", SOFT_WHITE);
    add(&mut colors, "statusBar.border", &p.border.default);

    add(&mut colors, "titleBar.activeForeground", DIM_WHITE);
    add(&mut colors, "titleBar.activeBackground", &p.canvas.default);
    add(&mut colors, "panelTitle.activeBorder", ACCENT);
    add(&mut colors, "notificationsInfoIcon.foreground", BRIGHT_ACCENT_TEXT);

    // Terminal
    add(&mut colors, "terminal.foreground", &p.fg.default);
    add(&mut colors, "terminal.ansiBlack", &p.ansi.black);
    add(&mut colors, "terminal.ansiRed", &p.ansi.red);
    add(&mut colors, "terminal.ansiGreen", &p.ansi.green);
    add(&mut colors, "terminal.ansiYellow", &p.ansi.yellow);
    add(&mut colors, "terminal.ansiBlue", &p.ansi.blue);
    add(&mut colors, "terminal.ansiMagenta", &p.ansi.magenta);
    add(&mut colors, "terminal.ansiCyan", &p.ansi.cyan);
    add(&mut colors, "terminal.ansiWhite", &p.ansi.white);

    add(&mut colors, "statusBar.debuggingBackground", ACCENT);
    add(&mut colors, "statusBar.debuggingForeground", SOFT_WHITE);
    add(&mut colors, "statusBarItem.remoteBackground", ACCENT);
    add(&mut colors, "statusBarItem.remoteForeground", SOFT_WHITE);


    let base_theme: BaseThemeTokens = serde_json::from_str(include_str!("../data/modern_dark_token_color.json"))?;
    let mut token_colors = Vec::new();

    let nk_keyword = "#7b92e0";
    let nk_variable = "#74dbf5";

    let nk_variant = "#FF9E64";
    let nk_type = "#4EC9B0";
    let nk_string = "#ce8978";
    let nk_function = "#cac68c";

    add(&mut colors, "symbolIcon.classForeground", nk_type);
    add(&mut colors, "symbolIcon.enumeratorForeground", nk_type);
    add(&mut colors, "symbolIcon.enumeratorMemberForeground", nk_variant);
    add(&mut colors, "symbolIcon.functionForeground", nk_function);
    add(&mut colors, "symbolIcon.variableForeground", nk_variable);

    let mut stc = BTreeMap::new();

    if let Some(semantic_token_colors) = base_theme.semantic_token_colors {
        stc.extend(semantic_token_colors);
    }

    // Token Colors
    for mut token in base_theme.token_colors {
        if token.scope.contains_keyword("comment") {
            token.settings.foreground = Some(COMMENT.to_string());
            token.settings.font_style = Some("italic".to_string());
            token_colors.push(token);
            continue;
        }

        use TokenName::*;

        if let Some(ref name_str) = token.name {
            let token_name = name_str.parse::<TokenName>().unwrap();

            let token = match token_name {
                KeywordGeneral => update_token!(token, fg: nk_keyword),

                IdentifierVariable | IdentifierProperty =>
                    update_token!(token, fg: nk_variable),

                IdentifierFunction =>
                    update_token!(token, fg: nk_function),

                LiteralString | LiteralStringValue | LiteralStringTag =>
                    update_token!(token, fg: nk_string),

                _ => token,
            };

            token_colors.push(token);
        } else {
            unreachable!();
        }
    }

    let theme = Theme {
        name: "Nokto Theme".to_string(),
        colors,
        semantic_highlighting: true,
        semantic_token_colors: Some(stc),
        token_colors,
    };

    let output_dir = Path::new("./themes");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let theme_json = serde_json::to_string_pretty(&theme)?;
    fs::write(output_dir.join("nokto-theme.json"), theme_json)?;

    println!("Theme generated successfully!");
    Ok(())
}

fn hex(color_str: &str) -> String {
    color_str
        .parse::<csscolorparser::Color>()
        .map(|c| c.to_css_hex())
        .unwrap_or_else(|_| color_str.to_string())
}

fn alpha(color_str: &str, a: f32) -> String {
    if let Ok(mut c) = color_str.parse::<csscolorparser::Color>() {
        c.a = a;
        c.to_css_hex()
    } else {
        color_str.to_string()
    }
}

fn add(colors: &mut BTreeMap<String, String>, key: &str, value: &str) {
    colors.insert(key.to_string(), hex(value));
}

fn add_alpha(colors: &mut BTreeMap<String, String>, key: &str, value: &str, a: f32) {
    colors.insert(key.to_string(), alpha(value, a));
}

fn modify_oklch<F>(hex_color: &str, modifier: F) -> Result<String>
where
    F: FnOnce(&mut Oklch),
{
    let parsed = hex_color
        .parse::<csscolorparser::Color>()
        .map_err(|_| anyhow!("Failed to parse color: '{}'", hex_color))?;
    let srgb = Srgb::new(parsed.r, parsed.g, parsed.b);

    let mut oklch: Oklch = srgb.into_color();

    modifier(&mut oklch);

    let srgb_out: Srgb = oklch.into_color();
    Ok(format!(
        "#{:02x}{:02x}{:02x}",
        (srgb_out.red * 255.0).round().clamp(0.0, 255.0) as u8,
        (srgb_out.green * 255.0).round().clamp(0.0, 255.0) as u8,
        (srgb_out.blue * 255.0).round().clamp(0.0, 255.0) as u8
    ))
}
