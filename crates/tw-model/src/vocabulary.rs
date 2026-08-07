use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::NodeId;
use crate::image::{ImageData, TextWrap};
use crate::nodes::Block;

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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ShapeKind {
    #[default]
    Rectangle,
    Line,
    TextBox,
    Other,
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
}

impl ShapeBlock {
    pub fn placeholder(width: f32, height: f32) -> Self {
        Self {
            id: NodeId::new(),
            shape: ShapeData {
                shape_type: ShapeKind::Rectangle,
                width,
                height,
            },
            wrap: TextWrap::Square,
        }
    }
}

/// Typed header/footer maps keyed by variant.
pub type HeaderFooterMap = HashMap<HeaderFooterType, HeaderFooter>;
