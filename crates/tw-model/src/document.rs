use crate::ids::NodeId;
use crate::list::NumberingCatalog;
use crate::nodes::{Block, Paragraph, Section};
use crate::properties::DocumentProperties;
use crate::styles::StyleSheet;
use crate::theme::DocumentTheme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentSettings {
    pub track_changes_enabled: bool,
    pub author_name: String,
    pub default_tab_stop: f32,
    pub numbering: NumberingCatalog,
    pub theme: DocumentTheme,
    pub template_name: Option<String>,
    /// When true, the document cannot be edited (from DOCX protection or app policy).
    #[serde(default)]
    pub read_only: bool,
}

impl DocumentSettings {
    pub fn default_settings() -> Self {
        Self {
            track_changes_enabled: false,
            author_name: "Author".into(),
            default_tab_stop: 36.0,
            numbering: NumberingCatalog::with_defaults(),
            theme: DocumentTheme::default(),
            template_name: None,
            read_only: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: NodeId,
    pub styles: StyleSheet,
    pub settings: DocumentSettings,
    #[serde(default)]
    pub properties: DocumentProperties,
    pub sections: Vec<Section>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            styles: StyleSheet::with_defaults(),
            settings: DocumentSettings::default_settings(),
            properties: DocumentProperties::default(),
            sections: vec![Section::new()],
        }
    }

    pub fn with_paragraph(text: impl Into<String>) -> Self {
        let mut doc = Self::new();
        if let Some(section) = doc.sections.first_mut() {
            section.blocks = vec![Block::Paragraph(Paragraph::with_text(text))];
        }
        doc
    }

    /// Build a document from plain text, splitting on blank lines or single newlines.
    pub fn from_plain_text(text: &str) -> Self {
        let mut doc = Self::new();
        if let Some(section) = doc.sections.first_mut() {
            let paragraphs: Vec<&str> = text
                .split('\n')
                .map(str::trim_end)
                .collect();
            section.blocks = if paragraphs.is_empty() || (paragraphs.len() == 1 && paragraphs[0].is_empty()) {
                vec![Block::Paragraph(Paragraph::new())]
            } else {
                paragraphs
                    .into_iter()
                    .map(|p| Block::Paragraph(Paragraph::with_text(p)))
                    .collect()
            };
        }
        doc
    }

    pub fn first_section_mut(&mut self) -> Option<&mut Section> {
        self.sections.first_mut()
    }

    pub fn paragraphs_mut(&mut self) -> impl Iterator<Item = &mut Paragraph> {
        self.sections.iter_mut().flat_map(|s| {
            s.blocks.iter_mut().filter_map(|b| match b {
                Block::Paragraph(p) => Some(p),
                Block::Table(_) | Block::ImageBlock(_) => None,
            })
        })
    }

    pub fn find_run_location(&self, run_id: NodeId) -> Option<(usize, usize, usize)> {
        for (si, section) in self.sections.iter().enumerate() {
            for (bi, block) in section.blocks.iter().enumerate() {
                if let Block::Paragraph(p) = block {
                    for (ri, run) in p.runs.iter().enumerate() {
                        if run.id == run_id {
                            return Some((si, bi, ri));
                        }
                    }
                }
            }
        }
        None
    }

    pub fn find_paragraph_location(&self, para_id: NodeId) -> Option<(usize, usize)> {
        for (si, section) in self.sections.iter().enumerate() {
            for (bi, block) in section.blocks.iter().enumerate() {
                if let Block::Paragraph(p) = block {
                    if p.id == para_id {
                        return Some((si, bi));
                    }
                }
            }
        }
        None
    }

    pub fn paragraph_at_mut(&mut self, si: usize, bi: usize) -> Option<&mut Paragraph> {
        self.sections
            .get_mut(si)?
            .blocks
            .get_mut(bi)?
            .paragraph_mut()
    }

    pub fn find_block_location(&self, block_id: NodeId) -> Option<(usize, usize)> {
        for (si, section) in self.sections.iter().enumerate() {
            for (bi, block) in section.blocks.iter().enumerate() {
                let id = match block {
                    Block::Paragraph(p) => p.id,
                    Block::Table(t) => t.id,
                    Block::ImageBlock(i) => i.id,
                };
                if id == block_id {
                    return Some((si, bi));
                }
            }
        }
        None
    }

    pub fn block_at_mut(&mut self, si: usize, bi: usize) -> Option<&mut Block> {
        self.sections.get_mut(si)?.blocks.get_mut(bi)
    }

    pub fn paragraph_at(&self, si: usize, bi: usize) -> Option<&Paragraph> {
        self.sections.get(si)?.blocks.get(bi)?.paragraph()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
