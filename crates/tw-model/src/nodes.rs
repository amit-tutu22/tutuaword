use std::collections::HashMap;

use crate::format::{BreakType, CharFormat, ParaFormat, SectionFormat};
use crate::ids::{NodeId, StyleId};
use crate::image::ImageBlock;
use crate::revision::Revision;
use crate::table::Table;
use crate::vocabulary::{
    BookmarkAnchor, CommentRef, FieldData, FootnoteRef, HeaderFooter, HeaderFooterLinks,
    HeaderFooterType, HyperlinkTarget, InlineImageRef, ShapeBlock,
};
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

    /// Display text for layout, export plaintext, and search.
    pub fn text(&self) -> &str {
        self.content.display_text()
    }

    pub fn text_mut(&mut self) -> Option<&mut String> {
        match &mut self.content {
            RunContent::Text(t) => Some(t),
            _ => None,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunContent {
    Text(String),
    Tab,
    Break(BreakType),
    Hyperlink {
        target: HyperlinkTarget,
        text: String,
    },
    Field(FieldData),
    InlineImage(InlineImageRef),
    FootnoteRef(FootnoteRef),
    CommentRef(CommentRef),
    Bookmark(BookmarkAnchor),
}

impl RunContent {
    /// Placeholder-friendly display string for layout and plaintext export.
    pub fn display_text(&self) -> &str {
        match self {
            RunContent::Text(t) => t,
            RunContent::Tab => "\t",
            RunContent::Break(BreakType::Line) => "\n",
            RunContent::Break(BreakType::Page) | RunContent::Break(BreakType::Column) => "",
            RunContent::Hyperlink { text, .. } => text,
            RunContent::Field(field) => field
                .display_text
                .as_deref()
                .unwrap_or("[field]"),
            RunContent::InlineImage(_) => "[image]",
            RunContent::FootnoteRef(note) => {
                // Stable placeholder; layout may substitute superscript number later.
                if note.display_number.is_some() {
                    // Leak is not acceptable; use a thread-local or return owned.
                    // For &str return we use static placeholders per number bucket — keep simple:
                    "[fn]"
                } else {
                    "[fn]"
                }
            }
            RunContent::CommentRef(_) => "[comment]",
            RunContent::Bookmark(b) => {
                // Bookmark names vary; use generic placeholder for &str API.
                let _ = b;
                "[bookmark]"
            }
        }
    }

    /// Whether adjacent runs with the same format may merge.
    pub fn is_mergeable_text(&self) -> bool {
        matches!(self, RunContent::Text(_))
    }
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

    /// Plaintext for export/search — excludes hidden runs.
    pub fn visible_text(&self) -> String {
        self.runs
            .iter()
            .filter(|r| r.format.hidden != Some(true))
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

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Block {
    Paragraph(Paragraph),
    Table(Table),
    ImageBlock(ImageBlock),
    ShapeBlock(ShapeBlock),
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

    pub fn shape(&self) -> Option<&ShapeBlock> {
        match self {
            Block::ShapeBlock(s) => Some(s),
            _ => None,
        }
    }

    pub fn shape_mut(&mut self) -> Option<&mut ShapeBlock> {
        match self {
            Block::ShapeBlock(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: NodeId,
    pub format: SectionFormat,
    #[serde(default)]
    pub headers: HashMap<HeaderFooterType, HeaderFooter>,
    #[serde(default)]
    pub footers: HashMap<HeaderFooterType, HeaderFooter>,
    /// When true for a variant, this section uses the previous section's header band.
    #[serde(default)]
    pub header_links: HeaderFooterLinks,
    /// When true for a variant, this section uses the previous section's footer band.
    #[serde(default)]
    pub footer_links: HeaderFooterLinks,
    pub blocks: Vec<Block>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            format: SectionFormat::default(),
            headers: HashMap::new(),
            footers: HashMap::new(),
            header_links: HeaderFooterLinks::default(),
            footer_links: HeaderFooterLinks::default(),
            blocks: vec![Block::Paragraph(Paragraph::new())],
        }
    }

    /// Migrate legacy `SectionFormat` header/footer fields into typed maps.
    pub fn migrate_legacy_headers_footers(&mut self) {
        if self.headers.is_empty() {
            if !self.format.header_blocks.is_empty() || self.format.header_text.is_some() {
                self.headers.insert(
                    HeaderFooterType::Default,
                    HeaderFooter {
                        blocks: std::mem::take(&mut self.format.header_blocks),
                        plain_text: self.format.header_text.take(),
                    },
                );
            }
        }
        if self.footers.is_empty() {
            if !self.format.footer_blocks.is_empty() || self.format.footer_text.is_some() {
                self.footers.insert(
                    HeaderFooterType::Default,
                    HeaderFooter {
                        blocks: std::mem::take(&mut self.format.footer_blocks),
                        plain_text: self.format.footer_text.take(),
                    },
                );
            }
        }
    }

    /// Header blocks for layout: typed map first, then legacy format fields.
    pub fn header_for_layout(&self, kind: HeaderFooterType) -> Option<&HeaderFooter> {
        self.resolve_header(kind)
    }

    /// Footer blocks for layout: typed map first, then legacy format fields.
    pub fn footer_for_layout(&self, kind: HeaderFooterType) -> Option<&HeaderFooter> {
        self.resolve_footer(kind)
    }

    /// Resolve a header variant, falling back to Default when the specific band is empty.
    pub fn resolve_header(&self, kind: HeaderFooterType) -> Option<&HeaderFooter> {
        if let Some(hf) = self.headers.get(&kind) {
            if !hf.blocks.is_empty() || hf.plain_text.is_some() {
                return Some(hf);
            }
        }
        if kind != HeaderFooterType::Default {
            return self.resolve_header(HeaderFooterType::Default);
        }
        None
    }

    /// Resolve a footer variant, falling back to Default when the specific band is empty.
    pub fn resolve_footer(&self, kind: HeaderFooterType) -> Option<&HeaderFooter> {
        if let Some(hf) = self.footers.get(&kind) {
            if !hf.blocks.is_empty() || hf.plain_text.is_some() {
                return Some(hf);
            }
        }
        if kind != HeaderFooterType::Default {
            return self.resolve_footer(HeaderFooterType::Default);
        }
        None
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}
