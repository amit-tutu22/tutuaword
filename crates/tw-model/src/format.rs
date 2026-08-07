use serde::{Deserialize, Serialize};

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
    pub page_break_before: Option<bool>,
    pub keep_together: Option<bool>,
    pub keep_with_next: Option<bool>,
    pub widow_orphan_control: Option<bool>,
    #[serde(default)]
    pub tab_stops: Vec<TabStop>,
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
        if !other.tab_stops.is_empty() {
            self.tab_stops = other.tab_stops.clone();
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
            header_text: None,
            footer_text: None,
            header_blocks: Vec::new(),
            footer_blocks: Vec::new(),
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
