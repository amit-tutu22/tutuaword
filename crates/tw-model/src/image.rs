use crate::ids::NodeId;
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    /// Build image payload from raw file bytes (PNG, JPEG, or SVG).
    pub fn from_bytes(bytes: Vec<u8>, mime_type: Option<String>) -> Self {
        let mime = mime_type
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| detect_mime_type(&bytes).to_string());
        let (width_px, height_px) = pixel_dimensions(&bytes, &mime);
        Self {
            asset_id: uuid::Uuid::new_v4().to_string(),
            mime_type: mime,
            width_px,
            height_px,
            bytes,
        }
    }

    /// Default layout size in points (96 DPI px → 72 pt/in), optionally capped.
    pub fn display_size(&self, max_width_pt: f32) -> (f32, f32) {
        display_size_from_pixels(self.width_px, self.height_px, max_width_pt)
    }
}

/// Guess MIME type from magic bytes / SVG markup.
pub fn detect_mime_type(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 8
        && bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    {
        "image/png"
    } else if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        "image/jpeg"
    } else if is_svg_bytes(bytes) {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}

pub fn pixel_dimensions(bytes: &[u8], mime_type: &str) -> (u32, u32) {
    if mime_type.contains("png") {
        png_dimensions(bytes).unwrap_or((1, 1))
    } else if mime_type.contains("jpeg") || mime_type.contains("jpg") {
        jpeg_dimensions(bytes).unwrap_or((1, 1))
    } else if mime_type.contains("svg") {
        svg_dimensions(bytes).unwrap_or((100, 100))
    } else {
        (1, 1)
    }
}

pub fn display_size_from_pixels(width_px: u32, height_px: u32, max_width_pt: f32) -> (f32, f32) {
    let mut w = width_px as f32 * 72.0 / 96.0;
    let mut h = height_px as f32 * 72.0 / 96.0;
    if w > max_width_pt && w > 0.0 {
        let scale = max_width_pt / w;
        w = max_width_pt;
        h *= scale;
    }
    (w.max(1.0), h.max(1.0))
}

fn is_svg_bytes(bytes: &[u8]) -> bool {
    let head = bytes.get(..256.min(bytes.len())).unwrap_or(bytes);
    let text = String::from_utf8_lossy(head);
    text.contains("<svg") || text.contains(":svg")
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((w, h))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2usize;
    while i + 9 < bytes.len() {
        if bytes[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = bytes[i + 1];
        if (0xC0..=0xC3).contains(&marker) || marker == 0xC5 || marker == 0xC6 || marker == 0xC7
        {
            let h = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
            let w = u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]) as u32;
            return Some((w, h));
        }
        if marker == 0xD8 || marker == 0xD9 {
            break;
        }
        let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        if len < 2 {
            break;
        }
        i += 2 + len;
    }
    None
}

fn svg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let text = String::from_utf8_lossy(bytes);
    if let (Some(w), Some(h)) = (parse_svg_number(&text, "width"), parse_svg_number(&text, "height"))
    {
        return Some((w.max(1), h.max(1)));
    }
    if let Some(vb) = extract_xml_attr(&text, "viewBox") {
        let parts: Vec<f32> = vb.split_whitespace().filter_map(|p| p.parse().ok()).collect();
        if parts.len() >= 4 {
            return Some((parts[2].max(1.0) as u32, parts[3].max(1.0) as u32));
        }
    }
    None
}

fn parse_svg_number(text: &str, attr: &str) -> Option<u32> {
    let raw = extract_xml_attr(text, attr)?;
    let num: f32 = raw
        .trim_end_matches("px")
        .trim_end_matches("pt")
        .trim_end_matches('%')
        .parse()
        .ok()?;
    Some(num.round().max(1.0) as u32)
}

fn extract_xml_attr(text: &str, attr: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let needle = format!("{attr}={quote}");
        let start = text.find(&needle)? + needle.len();
        let rest = &text[start..];
        let end = rest.find(quote)?;
        return Some(rest[..end].to_string());
    }
    None
}

/// Visual transform applied at layout/render time (crop, rotate, opacity).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ImageTransform {
    /// Clockwise rotation in degrees.
    pub rotation_deg: f32,
    /// Source crop as fractions of the intrinsic image (0..1).
    pub crop_left: f32,
    pub crop_top: f32,
    pub crop_right: f32,
    pub crop_bottom: f32,
    /// 1.0 = fully opaque.
    pub opacity: f32,
}

impl Default for ImageTransform {
    fn default() -> Self {
        Self {
            rotation_deg: 0.0,
            crop_left: 0.0,
            crop_top: 0.0,
            crop_right: 0.0,
            crop_bottom: 0.0,
            opacity: 1.0,
        }
    }
}

impl ImageTransform {
    pub fn normalized(mut self) -> Self {
        self.rotation_deg = self.rotation_deg.rem_euclid(360.0);
        self.crop_left = self.crop_left.clamp(0.0, 0.95);
        self.crop_top = self.crop_top.clamp(0.0, 0.95);
        self.crop_right = self.crop_right.clamp(0.0, 0.95);
        self.crop_bottom = self.crop_bottom.clamp(0.0, 0.95);
        let h_crop = self.crop_left + self.crop_right;
        let v_crop = self.crop_top + self.crop_bottom;
        if h_crop > 0.95 {
            let scale = 0.95 / h_crop;
            self.crop_left *= scale;
            self.crop_right *= scale;
        }
        if v_crop > 0.95 {
            let scale = 0.95 / v_crop;
            self.crop_top *= scale;
            self.crop_bottom *= scale;
        }
        self.opacity = self.opacity.clamp(0.0, 1.0);
        self
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
    /// Crop, rotation, and opacity applied at layout/render time (F10.S4).
    #[serde(default)]
    pub transform: ImageTransform,
    /// Caption paragraph linked to this image (F10.S4).
    #[serde(default)]
    pub caption_paragraph_id: Option<NodeId>,
    /// Accessibility alternative text (`wp:docPr/@descr`) — F21.S3.
    #[serde(default)]
    pub alt_text: Option<String>,
}

impl ImageBlock {
    /// Layout frame after crop fractions are applied.
    pub fn effective_display_size(&self) -> (f32, f32) {
        let t = &self.transform;
        let w = self.display_width * (1.0 - t.crop_left - t.crop_right).max(0.01);
        let h = self.display_height * (1.0 - t.crop_top - t.crop_bottom).max(0.01);
        (w.max(1.0), h.max(1.0))
    }

    pub fn placeholder(width: f32, height: f32) -> Self {
        Self {
            id: NodeId::new(),
            data: ImageData::placeholder(width as u32, height as u32),
            display_width: width,
            display_height: height,
            wrap: TextWrap::Square,
            anchor: None,
            transform: ImageTransform::default(),
            caption_paragraph_id: None,
            alt_text: None,
        }
    }

    pub fn from_image_data(data: ImageData, max_width_pt: f32) -> Self {
        let (display_width, display_height) = data.display_size(max_width_pt);
        Self {
            id: NodeId::new(),
            data,
            display_width,
            display_height,
            wrap: TextWrap::Inline,
            anchor: None,
            transform: ImageTransform::default(),
            caption_paragraph_id: None,
            alt_text: None,
        }
    }
}

#[cfg(test)]
mod image_transform_tests {
    use super::*;

    #[test]
    fn effective_display_size_applies_crop() {
        let mut image = ImageBlock::placeholder(100.0, 80.0);
        image.transform.crop_left = 0.1;
        image.transform.crop_right = 0.1;
        let (w, h) = image.effective_display_size();
        assert!((w - 80.0).abs() < 0.01);
        assert!((h - 80.0).abs() < 0.01);
    }
}

#[cfg(test)]
mod image_data_tests {
    use super::*;

    const PNG_1X1: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn png_bytes_get_dimensions_and_mime() {
        let data = ImageData::from_bytes(PNG_1X1.to_vec(), None);
        assert_eq!(data.mime_type, "image/png");
        assert_eq!(data.width_px, 1);
        assert_eq!(data.height_px, 1);
        assert_eq!(data.bytes, PNG_1X1);
    }
}
