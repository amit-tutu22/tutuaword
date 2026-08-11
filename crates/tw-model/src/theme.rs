use crate::format::{CharFormat, Color};
use crate::nodes::Block;
use crate::Document;
use serde::{Deserialize, Serialize};

/// Marks a `font_family` that names a theme slot rather than a real family.
pub const THEME_FONT_PREFIX: &str = "+";

/// Built-in theme color slots mirrored from Office theme columns.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThemeColorSlot {
    Background1,
    Text1,
    Accent1,
    Accent2,
}

/// Reference to a theme color swatch (column + tint/shade row in the picker).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ThemeColorRef {
    pub slot: ThemeColorSlot,
    /// Picker row 0–5: three tints, base, two shades.
    pub variant: u8,
}

impl ThemeColorRef {
    pub fn new(slot: ThemeColorSlot, variant: u8) -> Self {
        Self {
            slot,
            variant: variant.min(5),
        }
    }

    /// Map the Word theme-color grid column (0–9) to a slot, if themed.
    pub fn from_picker(column: u8, row: u8) -> Option<Self> {
        let slot = match column {
            0 => ThemeColorSlot::Background1,
            1 => ThemeColorSlot::Text1,
            4 => ThemeColorSlot::Accent1,
            5 => ThemeColorSlot::Accent2,
            _ => return None,
        };
        Some(Self::new(slot, row))
    }

    pub fn ooxml_name(&self) -> &'static str {
        match self.slot {
            ThemeColorSlot::Background1 => "background1",
            ThemeColorSlot::Text1 => "text1",
            ThemeColorSlot::Accent1 => "accent1",
            ThemeColorSlot::Accent2 => "accent2",
        }
    }

    pub fn from_ooxml(name: &str, tint: Option<u8>, shade: Option<u8>) -> Option<Self> {
        let slot = match name {
            "background1" | "lt1" => ThemeColorSlot::Background1,
            "text1" | "dk1" => ThemeColorSlot::Text1,
            "accent1" => ThemeColorSlot::Accent1,
            "accent2" => ThemeColorSlot::Accent2,
            _ => return None,
        };
        let variant = ooxml_tint_shade_to_variant(tint, shade);
        Some(Self::new(slot, variant))
    }
}

fn ooxml_tint_shade_to_variant(tint: Option<u8>, shade: Option<u8>) -> u8 {
    if let Some(shade) = shade {
        return if shade >= 128 { 5 } else { 4 };
    }
    if let Some(tint) = tint {
        return if tint >= 180 { 0 } else if tint >= 120 { 1 } else { 2 };
    }
    3
}

fn blend_channel(base: u8, target: u8, amount: f32) -> u8 {
    let amount = amount.clamp(0.0, 1.0);
    (f32::from(base) + (f32::from(target) - f32::from(base)) * amount).round() as u8
}

fn blend_theme_color(base: ThemeColor, target: ThemeColor, amount: f32) -> Color {
    Color {
        r: blend_channel(base.r, target.r, amount),
        g: blend_channel(base.g, target.g, amount),
        b: blend_channel(base.b, target.b, amount),
        a: 255,
    }
}

/// Resolve a theme slot + variant to an RGB color for the active document theme.
pub fn resolve_theme_color_ref(theme: &DocumentTheme, reference: ThemeColorRef) -> Color {
    let base = match reference.slot {
        ThemeColorSlot::Background1 => theme.background1,
        ThemeColorSlot::Text1 => theme.text1,
        ThemeColorSlot::Accent1 => theme.accent1,
        ThemeColorSlot::Accent2 => theme.accent2,
    };
    let white = ThemeColor {
        r: 255,
        g: 255,
        b: 255,
    };
    let black = ThemeColor {
        r: 0,
        g: 0,
        b: 0,
    };
    match reference.variant {
        0 => blend_theme_color(base, white, 0.80),
        1 => blend_theme_color(base, white, 0.55),
        2 => blend_theme_color(base, white, 0.30),
        3 => Color {
            r: base.r,
            g: base.g,
            b: base.b,
            a: 255,
        },
        4 => blend_theme_color(base, black, 0.25),
        _ => blend_theme_color(base, black, 0.50),
    }
}

/// Document theme colors/fonts (from DOCX theme1.xml or app defaults).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentTheme {
    pub name: String,
    pub major_font: String,
    pub minor_font: String,
    pub accent1: ThemeColor,
    pub accent2: ThemeColor,
    pub text1: ThemeColor,
    pub background1: ThemeColor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for DocumentTheme {
    fn default() -> Self {
        Self::office()
    }
}

impl DocumentTheme {
    pub fn office() -> Self {
        Self {
            name: "Office".into(),
            major_font: "Calibri Light".into(),
            minor_font: "Calibri".into(),
            accent1: ThemeColor {
                r: 68,
                g: 114,
                b: 196,
            },
            accent2: ThemeColor {
                r: 237,
                g: 125,
                b: 49,
            },
            text1: ThemeColor {
                r: 0,
                g: 0,
                b: 0,
            },
            background1: ThemeColor {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }

    pub fn facet() -> Self {
        Self {
            name: "Facet".into(),
            major_font: "Century Gothic".into(),
            minor_font: "Calibri".into(),
            accent1: ThemeColor {
                r: 75,
                g: 172,
                b: 198,
            },
            accent2: ThemeColor {
                r: 247,
                g: 150,
                b: 70,
            },
            text1: ThemeColor {
                r: 0,
                g: 0,
                b: 0,
            },
            background1: ThemeColor {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }

    pub fn ion() -> Self {
        Self {
            name: "Ion".into(),
            major_font: "Arial".into(),
            minor_font: "Arial".into(),
            accent1: ThemeColor {
                r: 255,
                g: 114,
                b: 0,
            },
            accent2: ThemeColor {
                r: 91,
                g: 155,
                b: 213,
            },
            text1: ThemeColor {
                r: 0,
                g: 0,
                b: 0,
            },
            background1: ThemeColor {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }

    /// Built-in gallery themes exposed on the Design tab.
    pub fn gallery_themes() -> &'static [&'static str] {
        &["Office", "Facet", "Ion"]
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "Office" => Some(Self::office()),
            "Facet" => Some(Self::facet()),
            "Ion" => Some(Self::ion()),
            _ => None,
        }
    }
}

/// Substitute theme font references for the families the theme names.
pub fn resolve_theme_fonts(doc: &mut Document) {
    let theme = doc.settings.theme.clone();
    let substitute = |format: &mut CharFormat| {
        let Some(family) = format.font_family.as_deref() else {
            return;
        };
        let Some(slot) = family.strip_prefix(THEME_FONT_PREFIX) else {
            return;
        };
        format.font_family = Some(if slot.starts_with("major") {
            theme.major_font.clone()
        } else {
            theme.minor_font.clone()
        });
    };

    visit_char_formats(doc, &substitute);
}

/// Re-resolve RGB for runs/styles that reference theme color slots.
pub fn resolve_theme_colors(doc: &mut Document) {
    let theme = doc.settings.theme.clone();
    let substitute = |format: &mut CharFormat| {
        let Some(reference) = format.theme_color else {
            return;
        };
        format.color = Some(resolve_theme_color_ref(&theme, reference));
    };

    visit_char_formats(doc, &substitute);
}

/// Apply all theme-linked substitutions (fonts + colors).
pub fn resolve_theme(doc: &mut Document) {
    resolve_theme_fonts(doc);
    resolve_theme_colors(doc);
}

fn visit_char_formats(doc: &mut Document, substitute: &impl Fn(&mut CharFormat)) {
    substitute(&mut doc.styles.defaults.char_format);
    for style in doc.styles.paragraph_styles.values_mut() {
        substitute(&mut style.char_format);
    }
    for style in doc.styles.character_styles.values_mut() {
        substitute(&mut style.char_format);
    }
    for section in &mut doc.sections {
        for block in &mut section.blocks {
            visit_block_runs(block, substitute);
        }
    }
}

fn visit_block_runs(block: &mut Block, substitute: &impl Fn(&mut CharFormat)) {
    match block {
        Block::Paragraph(para) => {
            for run in &mut para.runs {
                substitute(&mut run.format);
            }
        }
        Block::Table(table) => {
            for row in &mut table.rows {
                for cell in &mut row.cells {
                    for block in &mut cell.blocks {
                        visit_block_runs(block, substitute);
                    }
                }
            }
        }
        Block::ImageBlock(_) => {}
        Block::ShapeBlock(_) => {}
    }
}
