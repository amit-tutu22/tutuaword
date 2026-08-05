use serde::{Deserialize, Serialize};

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
}
