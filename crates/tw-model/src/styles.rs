use crate::format::{Alignment, BorderSpec, CharFormat, ParaFormat};
use crate::ids::StyleId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentDefaults {
    pub char_format: CharFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub id: StyleId,
    pub name: String,
    pub based_on: Option<StyleId>,
    pub char_format: CharFormat,
    pub para_format: ParaFormat,
    pub next_style: Option<StyleId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStyle {
    pub id: StyleId,
    pub name: String,
    pub based_on: Option<StyleId>,
    pub char_format: CharFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStyle {
    pub id: StyleId,
    pub name: String,
    pub border: Option<BorderSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleSheet {
    pub defaults: DocumentDefaults,
    pub paragraph_styles: HashMap<StyleId, ParagraphStyle>,
    pub character_styles: HashMap<StyleId, CharacterStyle>,
    #[serde(default)]
    pub table_styles: HashMap<StyleId, TableStyle>,
    /// OOXML `w:styleId` string → internal style id (from import).
    #[serde(default)]
    pub ooxml_style_ids: HashMap<String, StyleId>,
    /// OOXML table style ids → internal style id.
    #[serde(default)]
    pub ooxml_table_style_ids: HashMap<String, StyleId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleSheetError {
    BuiltinProtected(String),
    DuplicateName(String),
    NotFound(String),
}

/// Built-in paragraph style names that cannot be renamed or deleted.
pub const BUILTIN_PARAGRAPH_STYLE_NAMES: &[&str] = &[
    "Normal",
    "Heading 1",
    "Heading 2",
    "Heading 3",
    "Heading 4",
    "Heading 5",
    "Heading 6",
    "Heading 7",
    "Heading 8",
    "Heading 9",
    "Quote",
    "Caption",
];

impl StyleSheet {
    /// Built-in styles shipped with new documents (Normal, Heading 1–9, Quote, Caption).
    pub fn with_defaults() -> Self {
        let normal_id = StyleId::new();
        let mut paragraph_styles = HashMap::new();

        paragraph_styles.insert(
            normal_id,
            ParagraphStyle {
                id: normal_id,
                name: "Normal".into(),
                based_on: None,
                char_format: CharFormat {
                    font_size: Some(12.0),
                    ..Default::default()
                },
                para_format: ParaFormat::default(),
                next_style: Some(normal_id),
            },
        );

        let mut prev_heading_id = normal_id;
        for level in 1..=9 {
            let id = StyleId::new();
            let char_format = heading_level_char_format(level);
            paragraph_styles.insert(
                id,
                ParagraphStyle {
                    id,
                    name: format!("Heading {level}"),
                    based_on: Some(if level == 1 { normal_id } else { prev_heading_id }),
                    char_format,
                    para_format: ParaFormat {
                        space_before: Some(if level <= 2 { 12.0 } else { 6.0 }),
                        space_after: Some(if level <= 2 { 6.0 } else { 3.0 }),
                        outline_level: Some((level - 1) as u8),
                        ..Default::default()
                    },
                    next_style: Some(normal_id),
                },
            );
            prev_heading_id = id;
        }

        let quote_id = StyleId::new();
        paragraph_styles.insert(
            quote_id,
            ParagraphStyle {
                id: quote_id,
                name: "Quote".into(),
                based_on: Some(normal_id),
                char_format: CharFormat {
                    italic: Some(true),
                    ..Default::default()
                },
                para_format: ParaFormat {
                    indent_left: Some(36.0),
                    indent_right: Some(36.0),
                    space_before: Some(6.0),
                    space_after: Some(6.0),
                    ..Default::default()
                },
                next_style: Some(normal_id),
            },
        );

        let caption_id = StyleId::new();
        paragraph_styles.insert(
            caption_id,
            ParagraphStyle {
                id: caption_id,
                name: "Caption".into(),
                based_on: Some(normal_id),
                char_format: CharFormat {
                    font_size: Some(9.0),
                    italic: Some(true),
                    ..Default::default()
                },
                para_format: ParaFormat {
                    alignment: Some(Alignment::Center),
                    space_before: Some(3.0),
                    space_after: Some(3.0),
                    ..Default::default()
                },
                next_style: Some(normal_id),
            },
        );

        Self {
            defaults: DocumentDefaults::default(),
            paragraph_styles,
            character_styles: HashMap::new(),
            table_styles: HashMap::new(),
            ooxml_style_ids: HashMap::new(),
            ooxml_table_style_ids: HashMap::new(),
        }
    }

    pub fn with_heading1() -> Self {
        let mut styles = HashMap::new();
        let id = StyleId::new();
        styles.insert(
            id,
            ParagraphStyle {
                id,
                name: "Heading 1".into(),
                based_on: None,
                char_format: CharFormat {
                    bold: Some(true),
                    font_size: Some(16.0),
                    ..Default::default()
                },
                para_format: ParaFormat {
                    space_before: Some(12.0),
                    space_after: Some(6.0),
                    outline_level: Some(0),
                    ..Default::default()
                },
                next_style: None,
            },
        );
        Self {
            defaults: DocumentDefaults::default(),
            paragraph_styles: styles,
            character_styles: HashMap::new(),
            table_styles: HashMap::new(),
            ooxml_style_ids: HashMap::new(),
            ooxml_table_style_ids: HashMap::new(),
        }
    }

    pub fn resolve_char_format(
        &self,
        style_id: Option<StyleId>,
        direct: &CharFormat,
    ) -> CharFormat {
        let mut result = self.defaults.char_format.clone();
        if let Some(id) = style_id {
            self.apply_paragraph_style_chain(&mut result, id);
        }
        result.merge(direct);
        result
    }

    pub fn resolve_para_format(
        &self,
        style_id: Option<StyleId>,
        direct: &ParaFormat,
    ) -> ParaFormat {
        let mut result = ParaFormat::default();
        if let Some(id) = style_id {
            self.apply_para_style_chain(&mut result, id);
        }
        result.merge(direct);
        result
    }

    fn apply_paragraph_style_chain(&self, result: &mut CharFormat, style_id: StyleId) {
        if let Some(style) = self.paragraph_styles.get(&style_id) {
            if let Some(base) = style.based_on {
                self.apply_paragraph_style_chain(result, base);
            }
            result.merge(&style.char_format);
        }
    }

    fn apply_para_style_chain(&self, result: &mut ParaFormat, style_id: StyleId) {
        if let Some(style) = self.paragraph_styles.get(&style_id) {
            if let Some(base) = style.based_on {
                self.apply_para_style_chain(result, base);
            }
            result.merge(&style.para_format);
        }
    }

    pub fn find_style_by_name(&self, name: &str) -> Option<&ParagraphStyle> {
        self.paragraph_styles.values().find(|s| s.name == name)
    }

    pub fn find_style_by_ooxml_id(&self, ooxml_id: &str) -> Option<&ParagraphStyle> {
        self.ooxml_style_ids
            .get(ooxml_id)
            .and_then(|id| self.paragraph_styles.get(id))
    }

    pub fn is_builtin_paragraph_style(name: &str) -> bool {
        BUILTIN_PARAGRAPH_STYLE_NAMES.contains(&name)
    }

    pub fn ooxml_id_for(&self, style_id: StyleId) -> Option<String> {
        if let Some((ooxml_id, _)) = self
            .ooxml_style_ids
            .iter()
            .find(|(_, id)| **id == style_id)
        {
            return Some(ooxml_id.clone());
        }
        self.paragraph_styles
            .get(&style_id)
            .map(|style| style.name.replace(' ', ""))
    }

    pub fn create_paragraph_style(
        &mut self,
        name: String,
        based_on: Option<StyleId>,
        char_format: CharFormat,
        para_format: ParaFormat,
    ) -> Result<StyleId, StyleSheetError> {
        if self.find_style_by_name(&name).is_some() {
            return Err(StyleSheetError::DuplicateName(name));
        }
        let normal_id = self
            .find_style_by_name("Normal")
            .map(|s| s.id)
            .unwrap_or_else(StyleId::new);
        let id = StyleId::new();
        let ooxml_id = allocate_ooxml_style_id(&name, self);
        self.ooxml_style_ids.insert(ooxml_id, id);
        self.paragraph_styles.insert(
            id,
            ParagraphStyle {
                id,
                name,
                based_on,
                char_format,
                para_format,
                next_style: Some(normal_id),
            },
        );
        Ok(id)
    }

    pub fn rename_paragraph_style(
        &mut self,
        style_id: StyleId,
        new_name: String,
    ) -> Result<String, StyleSheetError> {
        let Some(style) = self.paragraph_styles.get(&style_id) else {
            return Err(StyleSheetError::NotFound(format!("{style_id:?}")));
        };
        if Self::is_builtin_paragraph_style(&style.name) {
            return Err(StyleSheetError::BuiltinProtected(style.name.clone()));
        }
        if self
            .paragraph_styles
            .values()
            .any(|s| s.id != style_id && s.name == new_name)
        {
            return Err(StyleSheetError::DuplicateName(new_name));
        }
        let old_name = style.name.clone();
        self.paragraph_styles.get_mut(&style_id).unwrap().name = new_name;
        Ok(old_name)
    }

    pub fn delete_paragraph_style(
        &mut self,
        style_id: StyleId,
    ) -> Result<ParagraphStyle, StyleSheetError> {
        let Some(style) = self.paragraph_styles.get(&style_id) else {
            return Err(StyleSheetError::NotFound(format!("{style_id:?}")));
        };
        if Self::is_builtin_paragraph_style(&style.name) {
            return Err(StyleSheetError::BuiltinProtected(style.name.clone()));
        }
        let style = self.paragraph_styles.remove(&style_id).unwrap();
        self.ooxml_style_ids.retain(|_, id| *id != style_id);
        Ok(style)
    }
}

/// Character format for built-in heading levels (Heading 1 bold; deeper levels taper size).
fn heading_level_char_format(level: u8) -> CharFormat {
    let font_size = match level {
        1 => 16.0,
        2 => 14.0,
        3 => 13.0,
        4 => 12.0,
        5 => 11.0,
        6 => 10.0,
        7 => 10.0,
        8 => 9.0,
        _ => 9.0,
    };
    let mut format = CharFormat {
        font_size: Some(font_size),
        ..Default::default()
    };
    if level == 1 || level <= 3 {
        format.bold = Some(true);
    }
    if level == 4 || level == 6 || level == 8 {
        format.italic = Some(true);
        format.bold = Some(false);
    }
    format
}

fn allocate_ooxml_style_id(name: &str, sheet: &StyleSheet) -> String {
    let base: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let base = if base.is_empty() {
        "CustomStyle".into()
    } else {
        base
    };
    if !sheet.ooxml_style_ids.contains_key(&base) {
        return base;
    }
    for suffix in 2..=999u32 {
        let candidate = format!("{base}{suffix}");
        if !sheet.ooxml_style_ids.contains_key(&candidate) {
            return candidate;
        }
    }
    base
}
