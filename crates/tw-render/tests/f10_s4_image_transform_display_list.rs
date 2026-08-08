//! F10.S4 — rotation/opacity/crop metadata in display list v6.

use tw_layout::LayoutEngine;
use tw_model::{Block, Document, ImageBlock, ImageData, ImageTransform, TextWrap};
use tw_render::DisplayListBuilder;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[test]
fn u_f10_s4_display_list_carries_transform_metadata() {
    let mut image = ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 72.0,
        display_height: 72.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform {
            rotation_deg: 90.0,
            crop_left: 0.05,
            opacity: 0.75,
            ..Default::default()
        },
        caption_paragraph_id: None,
    };
    image.data.bytes = PNG_1X1.to_vec();

    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::ImageBlock(image)];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);

    assert_eq!(list.image_batch.rotations.len(), 1);
    assert!((list.image_batch.rotations[0] - 90.0).abs() < 0.01);
    assert!((list.image_batch.opacities[0] - 0.75).abs() < 0.01);
    assert!((list.image_batch.crop_rects[0] - 0.05).abs() < 0.01);
}
