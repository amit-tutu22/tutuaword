use crate::ids::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextWrap {
    #[default]
    Inline,
    Square,
    TopBottom,
    Behind,
    InFront,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub asset_id: String,
    pub mime_type: String,
    pub width_px: u32,
    pub height_px: u32,
    pub bytes: Vec<u8>,
}

impl ImageData {
    pub fn placeholder(width_px: u32, height_px: u32) -> Self {
        Self {
            asset_id: uuid::Uuid::new_v4().to_string(),
            mime_type: "image/png".into(),
            width_px,
            height_px,
            bytes: Vec::new(),
        }
    }
}

/// What an anchored image's offset is measured from (`wp:positionH`/`wp:positionV`
/// `relativeFrom`).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnchorOrigin {
    /// Text column, i.e. the content area inside the margins.
    #[default]
    Column,
    Page,
    Margin,
}

/// Placement of a floating (`wp:anchor`) image. Inline images have none.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImageAnchor {
    pub x: f32,
    pub y: f32,
    pub origin_x: AnchorOrigin,
    pub origin_y: AnchorOrigin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    pub id: NodeId,
    pub data: ImageData,
    pub display_width: f32,
    pub display_height: f32,
    pub wrap: TextWrap,
    /// Set for floating images, which are positioned absolutely and take no
    /// space in the text flow.
    #[serde(default)]
    pub anchor: Option<ImageAnchor>,
}

impl ImageBlock {
    pub fn placeholder(width: f32, height: f32) -> Self {
        Self {
            id: NodeId::new(),
            data: ImageData::placeholder(width as u32, height as u32),
            display_width: width,
            display_height: height,
            wrap: TextWrap::Square,
            anchor: None,
        }
    }
}
