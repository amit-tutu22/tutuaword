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
    }

    pub fn equals(&self, other: &CharFormat) -> bool {
        self == other
    }
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
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LineSpacing {
    Single,
    Double,
    AtLeast(f32),
    Exactly(f32),
    Multiple(f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum UnderlineStyle {
    #[default]
    Single,
    Double,
    Dotted,
    Dashed,
    Wave,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
