//! U-F13-S2 — chart preview PNG appears in the display list image batch.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ImageData, ShapeBlock, ShapeData, ShapeKind, ShapeStyle};
use tw_render::DisplayListBuilder;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[test]
fn u_f13_s2_chart_preview_in_image_batch() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock {
        id: tw_model::NodeId::new(),
        shape: ShapeData {
            shape_type: ShapeKind::Chart,
            width: 432.0,
            height: 216.0,
        },
        wrap: tw_model::TextWrap::Inline,
        anchor: None,
        style: ShapeStyle::placeholder(),
        paragraphs: Vec::new(),
        preview_image: Some(ImageData::from_bytes(
            PNG_1X1.to_vec(),
            Some("image/png".into()),
        )),
        chart_data: None,
        chart_part: None,
        diagram_kind: Default::default(),
        diagram_data_part: None,
        diagram_layout_part: None,
    })];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];

    assert!(
        page.boxes.iter().any(|b| matches!(b, LayoutBox::Image(_))),
        "chart with preview should layout as image"
    );
    assert!(
        !page.boxes.iter().any(|b| matches!(b, LayoutBox::Shape(_))),
        "placeholder shape should be replaced by preview image"
    );

    let list = DisplayListBuilder::from_page_without_atlas(page, 1);
    assert!(list.image_batch.payloads.iter().any(|payload| payload.as_slice() == PNG_1X1));
}

#[test]
fn u_f13_s2_chart_without_preview_keeps_placeholder() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock {
        id: tw_model::NodeId::new(),
        shape: ShapeData {
            shape_type: ShapeKind::Chart,
            width: 432.0,
            height: 216.0,
        },
        wrap: tw_model::TextWrap::Inline,
        anchor: None,
        style: ShapeStyle::placeholder(),
        paragraphs: Vec::new(),
        preview_image: None,
        chart_data: None,
        chart_part: None,
        diagram_kind: Default::default(),
        diagram_data_part: None,
        diagram_layout_part: None,
    })];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];

    assert!(page.boxes.iter().any(|b| matches!(b, LayoutBox::Shape(_))));
    assert!(!page.boxes.iter().any(|b| matches!(b, LayoutBox::Image(_))));
}
