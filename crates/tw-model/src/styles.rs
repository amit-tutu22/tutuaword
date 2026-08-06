use crate::format::{BorderSpec, CharFormat, ParaFormat};
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

impl StyleSheet {
    /// Built-in styles shipped with new documents (Normal, Heading 1, etc.).
    pub fn with_defaults() -> Self {
        let normal_id = StyleId::new();
        let heading1_id = StyleId::new();
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
        paragraph_styles.insert(
            heading1_id,
            ParagraphStyle {
                id: heading1_id,
                name: "Heading 1".into(),
                based_on: Some(normal_id),
                char_format: CharFormat {
                    bold: Some(true),
                    font_size: Some(16.0),
                    ..Default::default()
                },
                para_format: ParaFormat {
                    space_before: Some(12.0),
                    space_after: Some(6.0),
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
}
