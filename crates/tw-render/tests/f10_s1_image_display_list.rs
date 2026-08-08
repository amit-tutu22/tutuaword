//! F10.S1 — real image bytes appear in the display list image batch payloads.

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
fn display_list_carries_inserted_png_payload() {
    let mut doc = Document::new();
    doc.sections[0].blocks.push(Block::ImageBlock(ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 72.0,
        display_height: 72.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
    }));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = layout.pages.first().unwrap();
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);

    assert!(!list.image_batch.payloads.is_empty());
    assert!(list
        .image_batch
        .payloads
        .iter()
        .any(|payload| payload.as_slice() == PNG_1X1));
}
