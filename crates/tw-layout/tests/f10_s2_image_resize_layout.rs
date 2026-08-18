//! F10.S2 — layout reflects resized image display dimensions.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, ImageBlock, ImageData, ImageTransform, TextWrap};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[test]
fn u_f10_s2_layout_reflects_display_size() {
    let mut doc = Document::new();
    doc.sections[0].blocks.push(Block::ImageBlock(ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 150.0,
        display_height: 100.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: None,
            wrap_polygon: None,
    }));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let image = layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .find_map(|b| match b {
            LayoutBox::Image(img) => Some(img),
            _ => None,
        })
        .expect("image layout");

    assert!((image.width - 150.0).abs() < 0.01);
    assert!((image.height - 100.0).abs() < 0.01);
}
