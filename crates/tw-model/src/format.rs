use serde::{Deserialize, Serialize};

use crate::theme::ThemeColorRef;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CharFormat {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<UnderlineStyle>,
    pub strikethrough: Option<bool>,
    pub superscript: Option<bool>,
    pub subscript: Option<bool>,
    pub color: Option<Color>,
    /// Theme slot reference; resolved RGB is kept in [`color`] via [`resolve_theme_colors`].
    pub theme_color: Option<ThemeColorRef>,
    pub highlight: Option<Color>,
    pub language: Option<String>,
    /// Extra space between characters, in points (`w:spacing` in `w:rPr`).
    pub character_spacing: Option<f32>,
    /// All capitals (`w:caps`).
    pub all_caps: Option<bool>,
    /// Small capitals OpenType feature (`w:smallCaps`).
    pub small_caps: Option<bool>,
    /// Hidden / vanish text (`w:vanish`) — omitted from export plaintext and layout.
    pub hidden: Option<bool>,
    /// Standard ligatures (`liga`). `None` enables ligatures when the font supports them.
    pub ligatures: Option<bool>,
}

impl CharFormat {
    pub fn merge(&mut self, other: &CharFormat) {
        if other.font_family.is_some() {
            self.font_family = other.font_family.clone();
        }
        if other.font_size.is_some() {
            self.font_size = other.font_size;
        }
        if other.bold.is_some() {
            self.bold = other.bold;
        }
        if other.italic.is_some() {
            self.italic = other.italic;
        }
        if other.underline.is_some() {
            self.underline = other.underline.clone();
        }
        if other.strikethrough.is_some() {
            self.strikethrough = other.strikethrough;
        }
        if other.superscript.is_some() {
            self.superscript = other.superscript;
        }
        if other.subscript.is_some() {
            self.subscript = other.subscript;
        }
        if other.color.is_some() {
            self.color = other.color;
            if other.theme_color.is_none() {
                self.theme_color = None;
            }
        }
        if other.theme_color.is_some() {
            self.theme_color = other.theme_color;
        }
        if other.highlight.is_some() {
            self.highlight = other.highlight;
        }
        if other.language.is_some() {
            self.language = other.language.clone();
        }
        if other.character_spacing.is_some() {
            self.character_spacing = other.character_spacing;
        }
        if other.all_caps.is_some() {
            self.all_caps = other.all_caps;
        }
        if other.small_caps.is_some() {
            self.small_caps = other.small_caps;
        }
        if other.hidden.is_some() {
            self.hidden = other.hidden;
        }
        if other.ligatures.is_some() {
            self.ligatures = other.ligatures;
        }
    }

    pub fn equals(&self, other: &CharFormat) -> bool {
        self == other
    }

    /// Whether two formats are equivalent for merging adjacent runs.
    ///
    /// Explicit "off" values (`bold: Some(false)`, `underline: None`, etc.) match
    /// unset fields so ribbon toggles do not leave splinter runs behind.
    pub fn merge_equivalent(&self, other: &CharFormat) -> bool {
        self.normalized_for_merge() == other.normalized_for_merge()
    }

    fn normalized_for_merge(&self) -> CharFormat {
        let mut f = self.clone();
        if f.bold == Some(false) {
            f.bold = None;
        }
        if f.italic == Some(false) {
            f.italic = None;
        }
        if f.strikethrough == Some(false) {
            f.strikethrough = None;
        }
        if f.superscript == Some(false) {
            f.superscript = None;
        }
        if f.subscript == Some(false) {
            f.subscript = None;
        }
        if f.underline == Some(UnderlineStyle::None) {
            f.underline = None;
        }
        if f.character_spacing == Some(0.0) {
            f.character_spacing = None;
        }
        if f.all_caps == Some(false) {
            f.all_caps = None;
        }
        if f.small_caps == Some(false) {
            f.small_caps = None;
        }
        if f.hidden == Some(false) {
            f.hidden = None;
        }
        if f.ligatures == Some(true) {
            f.ligatures = None;
        }
        f
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TabAlignment {
    #[default]
    Left,
    Center,
    Right,
    Decimal,
    Bar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabStop {
    pub position: f32,
    pub alignment: TabAlignment,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ParaFormat {
    pub alignment: Option<Alignment>,
    pub line_spacing: Option<LineSpacing>,
    pub space_before: Option<f32>,
    pub space_after: Option<f32>,
    pub indent_left: Option<f32>,
    pub indent_right: Option<f32>,
    pub indent_first_line: Option<f32>,
    pub numbering: Option<crate::list::NumberingRef>,
    /// When true, the list counter restarts at this paragraph (`w:numRestart`).
    pub num_restart: Option<bool>,
    /// Document-map level (`w:outlineLvl`, 0–8). Level 9 = body text (hidden).
    pub outline_level: Option<u8>,
    pub page_break_before: Option<bool>,
    pub keep_together: Option<bool>,
    pub keep_with_next: Option<bool>,
    pub widow_orphan_control: Option<bool>,
    /// Explicit tab stops. `None` = leave unchanged on merge; `Some([])` clears.
    #[serde(default)]
    pub tab_stops: Option<Vec<TabStop>>,
    /// Paragraph background fill (`w:pPr/w:shd`).
    pub shading: Option<Color>,
    /// Paragraph borders (`w:pPr/w:pBdr`).
    pub borders: Option<BorderSet>,
}

impl ParaFormat {
    pub fn merge(&mut self, other: &ParaFormat) {
        if other.alignment.is_some() {
            self.alignment = other.alignment;
        }
        if other.line_spacing.is_some() {
            self.line_spacing = other.line_spacing.clone();
        }
        if other.space_before.is_some() {
            self.space_before = other.space_before;
        }
        if other.space_after.is_some() {
            self.space_after = other.space_after;
        }
        if other.indent_left.is_some() {
            self.indent_left = other.indent_left;
        }
        if other.indent_right.is_some() {
            self.indent_right = other.indent_right;
        }
        if other.indent_first_line.is_some() {
            self.indent_first_line = other.indent_first_line;
        }
        if other.numbering.is_some() {
            self.numbering = other.numbering;
        }
        if other.num_restart.is_some() {
            self.num_restart = other.num_restart;
        }
        if other.outline_level.is_some() {
            self.outline_level = other.outline_level;
        }
        if other.page_break_before.is_some() {
            self.page_break_before = other.page_break_before;
        }
        if other.keep_together.is_some() {
            self.keep_together = other.keep_together;
        }
        if other.keep_with_next.is_some() {
            self.keep_with_next = other.keep_with_next;
        }
        if other.widow_orphan_control.is_some() {
            self.widow_orphan_control = other.widow_orphan_control;
        }
        if other.tab_stops.is_some() {
            self.tab_stops = other.tab_stops.clone();
        }
        if other.shading.is_some() {
            self.shading = other.shading;
        }
        if other.borders.is_some() {
            self.borders = other.borders.clone();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnLayout {
    /// Number of newspaper-style columns (1–3).
    pub count: u32,
    /// Gap between columns in points.
    pub gap: f32,
}

impl Default for ColumnLayout {
    fn default() -> Self {
        Self { count: 1, gap: 12.0 }
    }
}

impl ColumnLayout {
    pub fn with_count(count: u32) -> Self {
        Self {
            count: count.clamp(1, 3),
            gap: 12.0,
        }
    }
}

fn default_line_number_start() -> u32 {
    1
}

/// Section-level line numbering (F07.S4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineNumberSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_line_number_start")]
    pub start: u32,
}

impl Default for LineNumberSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            start: default_line_number_start(),
        }
    }
}

fn default_watermark_color() -> Color {
    Color {
        r: 192,
        g: 192,
        b: 192,
        a: 160,
    }
}

/// Text watermark behind page content (F07.S4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatermarkSettings {
    pub text: String,
    #[serde(default = "default_watermark_color")]
    pub color: Color,
}

impl WatermarkSettings {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: default_watermark_color(),
        }
    }

    /// Semi-transparent fill for the watermark band rect.
    pub fn background_color(&self) -> Color {
        Color {
            a: 48,
            ..self.color
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionFormat {
    pub page_width: f32,
    pub page_height: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    #[serde(default)]
    pub columns: ColumnLayout,
    /// Plain-text header shown in the top margin band on every page.
    pub header_text: Option<String>,
    /// Plain-text footer shown in the bottom margin band on every page.
    pub footer_text: Option<String>,
    /// Rich header blocks imported from `word/header*.xml`.
    #[serde(default)]
    pub header_blocks: Vec<crate::nodes::Block>,
    /// Rich footer blocks imported from `word/footer*.xml`.
    #[serde(default)]
    pub footer_blocks: Vec<crate::nodes::Block>,
    /// Solid page background color.
    #[serde(default)]
    pub page_color: Option<Color>,
    /// Semi-transparent watermark behind body content.
    #[serde(default)]
    pub watermark: Option<WatermarkSettings>,
    /// Line numbers in the left gutter.
    #[serde(default)]
    pub line_numbers: LineNumberSettings,
    /// When true, the first page of this section uses the First header/footer variant (`w:titlePg`).
    #[serde(default)]
    pub different_first_page: bool,
    /// Page border (box) drawn inside the page edge (F07 page borders).
    #[serde(default)]
    pub page_borders: Option<BorderSet>,
}

impl Default for SectionFormat {
    fn default() -> Self {
        Self {
            page_width: 612.0,
            page_height: 792.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
            columns: ColumnLayout::default(),
            header_text: None,
            footer_text: None,
            header_blocks: Vec::new(),
            footer_blocks: Vec::new(),
            page_color: None,
            watermark: None,
            line_numbers: LineNumberSettings::default(),
            different_first_page: false,
            page_borders: None,
        }
    }
}

impl SectionFormat {
    pub fn is_landscape(&self) -> bool {
        self.page_width > self.page_height
    }

    pub fn with_orientation(&self, landscape: bool) -> Self {
        let mut next = self.clone();
        if landscape && !next.is_landscape() {
            std::mem::swap(&mut next.page_width, &mut next.page_height);
        } else if !landscape && next.is_landscape() {
            std::mem::swap(&mut next.page_width, &mut next.page_height);
        }
        next
    }

    /// Apply page size preset while preserving current orientation.
    pub fn with_page_size_preset(&self, preset: PageSizePreset) -> Self {
        let (mut w, mut h) = preset.dimensions();
        if self.is_landscape() {
            std::mem::swap(&mut w, &mut h);
        }
        let mut next = self.clone();
        next.page_width = w;
        next.page_height = h;
        next
    }

    pub fn margin_preset(name: &str) -> Option<Self> {
        let margins = match name {
            "Normal" => (72.0, 72.0, 72.0, 72.0),
            "Narrow" => (36.0, 36.0, 36.0, 36.0),
            "Moderate" => (54.0, 54.0, 54.0, 54.0),
            "Wide" => (108.0, 108.0, 108.0, 108.0),
            _ => return None,
        };
        Some(Self {
            margin_top: margins.0,
            margin_bottom: margins.1,
            margin_left: margins.2,
            margin_right: margins.3,
            ..Self::default()
        })
    }

    /// Replace geometry fields while keeping header/footer content.
    pub fn apply_geometry(&mut self, patch: &SectionFormat) {
        self.page_width = patch.page_width;
        self.page_height = patch.page_height;
        self.margin_top = patch.margin_top;
        self.margin_bottom = patch.margin_bottom;
        self.margin_left = patch.margin_left;
        self.margin_right = patch.margin_right;
        self.columns = patch.columns.clone();
        self.page_color = patch.page_color;
        self.watermark = patch.watermark.clone();
        self.line_numbers = patch.line_numbers;
        self.different_first_page = patch.different_first_page;
        self.page_borders = patch.page_borders.clone();
    }

    /// Inset from the page edge used when painting [Self::page_borders].
    pub const PAGE_BORDER_INSET: f32 = 24.0;

    /// Build a uniform box border on all four sides.
    pub fn box_page_borders(width: f32, color: Color) -> BorderSet {
        let spec = BorderSpec { width, color };
        BorderSet {
            top: Some(spec),
            left: Some(spec),
            bottom: Some(spec),
            right: Some(spec),
        }
    }

    /// X coordinate for line-number labels in the left gutter.
    pub fn line_number_gutter_x(&self) -> f32 {
        (self.margin_left - Self::LINE_NUMBER_GUTTER_OFFSET).max(4.0)
    }

    /// Centered watermark band `(x, y, width, height)` in page coordinates.
    pub fn watermark_band(&self) -> (f32, f32, f32, f32) {
        let width = self.page_width * Self::WATERMARK_BAND_WIDTH_FRAC;
        let height = self.page_height * Self::WATERMARK_BAND_HEIGHT_FRAC;
        let x = (self.page_width - width) / 2.0;
        let y = (self.page_height - height) / 2.0;
        (x, y, width, height)
    }

    pub const LINE_NUMBER_GUTTER_OFFSET: f32 = 18.0;
    pub const WATERMARK_BAND_WIDTH_FRAC: f32 = 0.75;
    pub const WATERMARK_BAND_HEIGHT_FRAC: f32 = 0.25;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSizePreset {
    Letter,
    A4,
    Legal,
}

impl PageSizePreset {
    pub fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Letter => (612.0, 792.0),
            Self::A4 => (595.0, 842.0),
            Self::Legal => (612.0, 1008.0),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Letter" => Some(Self::Letter),
            "A4" => Some(Self::A4),
            "Legal" => Some(Self::Legal),
            _ => None,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LineSpacing {
    Single,
    Double,
    AtLeast(f32),
    Exactly(f32),
    Multiple(f32),
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum UnderlineStyle {
    None,
    #[default]
    Single,
    Double,
    Dotted,
    Dashed,
    Wave,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    pub fn to_argb(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BreakType {
    Line,
    Page,
    Column,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BorderSet {
    pub top: Option<BorderSpec>,
    pub left: Option<BorderSpec>,
    pub bottom: Option<BorderSpec>,
    pub right: Option<BorderSpec>,
}

impl BorderSet {
    pub fn any(&self) -> bool {
        self.top.is_some()
            || self.left.is_some()
            || self.bottom.is_some()
            || self.right.is_some()
    }

    /// Uniform box with the same [BorderSpec] on every side.
    pub fn box_all(spec: BorderSpec) -> Self {
        Self {
            top: Some(spec),
            left: Some(spec),
            bottom: Some(spec),
            right: Some(spec),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BorderSpec {
    pub width: f32,
    pub color: Color,
}

impl Default for BorderSpec {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color::BLACK,
        }
    }
}
