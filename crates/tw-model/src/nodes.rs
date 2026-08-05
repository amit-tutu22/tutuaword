use crate::format::{BreakType, CharFormat, ParaFormat, SectionFormat};
use crate::ids::{NodeId, StyleId};
use crate::image::ImageBlock;
use crate::revision::Revision;
use crate::table::Table;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: NodeId,
    pub format: CharFormat,
    pub content: RunContent,
    pub revision: Option<Revision>,
}

impl Run {
    pub fn new_text(text: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            format: CharFormat::default(),
            content: RunContent::Text(text.into()),
            revision: None,
        }
    }

    pub fn text(&self) -> &str {
        match &self.content {
            RunContent::Text(t) => t,
            _ => "",
        }
    }

    pub fn text_mut(&mut self) -> Option<&mut String> {
        match &mut self.content {
            RunContent::Text(t) => Some(t),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunContent {
    Text(String),
    Tab,
    Break(BreakType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub id: NodeId,
    pub format: ParaFormat,
    pub style_id: Option<StyleId>,
    pub runs: Vec<Run>,
}

impl Paragraph {
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            format: ParaFormat::default(),
            style_id: None,
            runs: vec![Run::new_text("")],
        }
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            format: ParaFormat::default(),
            style_id: None,
            runs: vec![Run::new_text(text)],
        }
    }

    pub fn full_text(&self) -> String {
        self.runs
            .iter()
            .map(|r| r.text())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Block {
    Paragraph(Paragraph),
    Table(Table),
    ImageBlock(ImageBlock),
}

impl Block {
    pub fn paragraph(&self) -> Option<&Paragraph> {
        match self {
            Block::Paragraph(p) => Some(p),
            _ => None,
        }
    }

    pub fn paragraph_mut(&mut self) -> Option<&mut Paragraph> {
        match self {
            Block::Paragraph(p) => Some(p),
            _ => None,
        }
    }

    pub fn table(&self) -> Option<&Table> {
        match self {
            Block::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn table_mut(&mut self) -> Option<&mut Table> {
        match self {
            Block::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn image(&self) -> Option<&ImageBlock> {
        match self {
            Block::ImageBlock(i) => Some(i),
            _ => None,
        }
    }

    pub fn image_mut(&mut self) -> Option<&mut ImageBlock> {
        match self {
            Block::ImageBlock(i) => Some(i),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: NodeId,
    pub format: SectionFormat,
    pub blocks: Vec<Block>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            format: SectionFormat::default(),
            blocks: vec![Block::Paragraph(Paragraph::new())],
        }
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}
