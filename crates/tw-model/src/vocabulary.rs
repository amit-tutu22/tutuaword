use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::NodeId;
use crate::image::{ImageData, TextWrap};
use crate::nodes::{Block, Paragraph};

/// Which header/footer variant applies (default, first page, even/odd).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum HeaderFooterType {
    #[default]
    Default,
    First,
    Even,
    Odd,
}

impl HeaderFooterType {
    pub fn from_ooxml(value: &str) -> Self {
        match value {
            "first" => Self::First,
            "even" => Self::Even,
            "odd" => Self::Odd,
            _ => Self::Default,
        }
    }

    /// Pick the header/footer variant for a page during layout or editing.
    pub fn for_page_layout(
        page_number: u32,
        is_section_first_page: bool,
        different_first_page: bool,
        even_and_odd_headers: bool,
    ) -> Self {
        if different_first_page && is_section_first_page {
            return Self::First;
        }
        if even_and_odd_headers {
            if page_number % 2 == 0 {
                Self::Even
            } else {
                Self::Odd
            }
        } else {
            Self::Default
        }
    }
}

/// Which document band a block belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockZone {
    Body,
    Header(HeaderFooterType),
    Footer(HeaderFooterType),
}

impl BlockZone {
    pub fn sort_key(self) -> u8 {
        match self {
            Self::Header(_) => 0,
            Self::Body => 1,
            Self::Footer(_) => 2,
        }
    }
}

/// Location of a run in the section tree (body or header/footer band).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunLocation {
    pub section_index: usize,
    pub zone: BlockZone,
    pub block_index: usize,
    pub run_index: usize,
    /// When the run lives inside a table cell: `(row, cell_index, block_index_in_cell)`.
    pub table_cell: Option<(usize, usize, usize)>,
    /// When the run lives inside a shape text box: index in `ShapeBlock.paragraphs`.
    pub shape_paragraph: Option<usize>,
}

impl RunLocation {
    pub fn block_key(self) -> (usize, u8, usize) {
        (self.section_index, self.zone.sort_key(), self.block_index)
    }
}

impl PartialOrd for RunLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RunLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.section_index,
            self.zone.sort_key(),
            self.block_index,
            self.table_cell,
            self.shape_paragraph,
            self.run_index,
        )
            .cmp(&(
                other.section_index,
                other.zone.sort_key(),
                other.block_index,
                other.table_cell,
                other.shape_paragraph,
                other.run_index,
            ))
    }
}

/// Rich header or footer content for a section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeaderFooter {
    #[serde(default)]
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub plain_text: Option<String>,
}

/// Hyperlink target extracted from `w:hyperlink`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HyperlinkTarget {
    pub url: String,
    #[serde(default)]
    pub anchor: Option<String>,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// Word field types (`w:fldSimple`, `w:instrText`).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FieldType {
    Page,
    NumPages,
    Date,
    Time,
    Filename,
    Author,
    Title,
    CrossRef,
    /// Sum numeric cells above the current table cell (F09.S5).
    TableSumAbove,
    Other(String),
}

/// Field run content with optional instruction and cached display text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldData {
    pub field_type: FieldType,
    #[serde(default)]
    pub instruction: Option<String>,
    #[serde(default)]
    pub display_text: Option<String>,
}

/// Inline image reference inside a run (`w:drawing` / `w:pict` in `w:r`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineImageRef {
    pub image: ImageData,
    pub display_width: f32,
    pub display_height: f32,
}

/// Footnote/endnote reference marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteRef {
    pub note_id: i32,
    #[serde(default)]
    pub display_number: Option<u32>,
}

/// Comment range reference marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentRef {
    pub comment_id: i32,
}

/// Bookmark start anchor (`w:bookmarkStart`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkAnchor {
    pub name: String,
    #[serde(default)]
    pub bookmark_id: Option<i32>,
}

/// Shape kind for floating/inline shapes.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub enum ShapeKind {
    #[default]
    Rectangle,
    Line,
    Ellipse,
    TextBox,
    WordArt,
    /// SmartArt / diagram graphics (F12.S1 — preserve-only).
    Diagram,
    /// Chart graphics (F13.S1 — preserve-only).
    Chart,
    Other,
}

/// Fill/stroke styling for editable shapes (F11.S2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapeStyle {
    pub fill: Option<u32>,
    pub stroke: Option<u32>,
    pub stroke_width: f32,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self::placeholder()
    }
}

impl ShapeStyle {
    /// Imported DrawingML shapes with unknown styling (F11.S1 placeholder).
    pub fn placeholder() -> Self {
        Self {
            fill: None,
            stroke: None,
            stroke_width: 1.0,
        }
    }

    /// Default styling for newly inserted shapes.
    pub fn inserted_default() -> Self {
        Self {
            fill: Some(0xFFD0E8FF),
            stroke: Some(0xFF000000),
            stroke_width: 1.0,
        }
    }

    pub fn is_placeholder(&self) -> bool {
        self.fill.is_none() && self.stroke.is_none()
    }
}

/// Shape geometry imported from DrawingML / VML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeData {
    pub shape_type: ShapeKind,
    pub width: f32,
    pub height: f32,
}

/// Block-level shape (`w:drawing` without image blip, VML shape, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeBlock {
    pub id: NodeId,
    pub shape: ShapeData,
    pub wrap: TextWrap,
    #[serde(default)]
    pub style: ShapeStyle,
    /// Embedded paragraphs for text boxes and WordArt (F11.S3).
    #[serde(default)]
    pub paragraphs: Vec<Paragraph>,
    /// Raster preview for preserved SmartArt diagrams (F12.S2).
    #[serde(default)]
    pub preview_image: Option<crate::ImageData>,
    /// Editable chart dataset (F13.S3).
    #[serde(default)]
    pub chart_data: Option<crate::ChartData>,
    /// OPC path to the linked chart part (e.g. `word/charts/chart1.xml`).
    #[serde(default)]
    pub chart_part: Option<String>,
    /// OPC path to linked diagram data part (F12.S3 hardening).
    #[serde(default)]
    pub diagram_data_part: Option<String>,
    /// OPC path to linked diagram layout part (F12.S3 hardening).
    #[serde(default)]
    pub diagram_layout_part: Option<String>,
}

impl ShapeBlock {
    pub fn new(shape_type: ShapeKind, width: f32, height: f32, style: ShapeStyle) -> Self {
        Self {
            id: NodeId::new(),
            shape: ShapeData {
                shape_type,
                width,
                height,
            },
            wrap: TextWrap::Inline,
            style,
            paragraphs: Vec::new(),
            preview_image: None,
            chart_data: None,
            chart_part: None,
            diagram_data_part: None,
            diagram_layout_part: None,
        }
    }

    pub fn text_box(width: f32, height: f32, style: ShapeStyle) -> Self {
        let mut shape = Self::new(ShapeKind::TextBox, width, height, style);
        shape.paragraphs.push(Paragraph::new());
        shape
    }

    pub fn word_art(text: impl Into<String>, width: f32, height: f32) -> Self {
        use crate::format::CharFormat;
        use crate::nodes::Run;

        let mut para = Paragraph::with_text(text.into());
        if let Some(run) = para.runs.first_mut() {
            run.format = CharFormat {
                bold: Some(true),
                italic: Some(true),
                font_size: Some(28.0),
                color: Some(crate::format::Color {
                    r: 0x1F,
                    g: 0x4E,
                    b: 0x79,
                    a: 255,
                }),
                ..Default::default()
            };
        } else {
            let mut run = Run::new_text("");
            run.format.font_size = Some(28.0);
            run.format.bold = Some(true);
            run.format.italic = Some(true);
            run.format.color = Some(crate::format::Color {
                r: 0x1F,
                g: 0x4E,
                b: 0x79,
                a: 255,
            });
            para.runs.push(run);
        }
        let mut shape = Self::new(
            ShapeKind::WordArt,
            width,
            height,
            ShapeStyle {
                fill: None,
                stroke: None,
                stroke_width: 1.0,
            },
        );
        shape.paragraphs.push(para);
        shape
    }

    pub fn diagram(width: f32, height: f32) -> Self {
        Self::new(ShapeKind::Diagram, width, height, ShapeStyle::placeholder())
    }

    pub fn chart(width: f32, height: f32) -> Self {
        let mut shape = Self::new(ShapeKind::Chart, width, height, ShapeStyle::placeholder());
        shape.chart_data = Some(crate::ChartData::sample_bar());
        shape
    }

    pub fn placeholder(width: f32, height: f32) -> Self {
        Self::new(
            ShapeKind::Rectangle,
            width,
            height,
            ShapeStyle::inserted_default(),
        )
    }
}

/// Typed header/footer maps keyed by variant.
pub type HeaderFooterMap = HashMap<HeaderFooterType, HeaderFooter>;

/// Per-variant link-to-previous flags for a section's header or footer band.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderFooterLinks {
    #[serde(default)]
    linked: HashMap<HeaderFooterType, bool>,
}

impl HeaderFooterLinks {
    /// Default link state for a newly inserted section (all variants linked).
    pub fn linked_to_previous() -> Self {
        let mut linked = HashMap::new();
        for kind in [
            HeaderFooterType::Default,
            HeaderFooterType::First,
            HeaderFooterType::Even,
            HeaderFooterType::Odd,
        ] {
            linked.insert(kind, true);
        }
        Self { linked }
    }

    pub fn is_linked(&self, kind: HeaderFooterType) -> bool {
        self.linked.get(&kind).copied().unwrap_or(true)
    }

    pub fn set_linked(&mut self, kind: HeaderFooterType, linked: bool) {
        self.linked.insert(kind, linked);
    }
}
