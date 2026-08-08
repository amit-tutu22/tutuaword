//! U-F12-S3 — diagram placeholder label and selection bounds.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ShapeBlock, ShapeData, ShapeKind, ShapeStyle};
use tw_render::DisplayListBuilder;

#[test]
fn u_f12_s3_diagram_label_in_layout() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::diagram(432.0, 216.0))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];

    assert!(page.boxes.iter().any(|b| matches!(b, LayoutBox::Shape(_))));
    let has_label = page.boxes.iter().any(|b| {
        matches!(b, LayoutBox::TextLine(line) if !line.glyphs.is_empty())
    });
    assert!(has_label, "diagram placeholder should include SmartArt label glyphs");
}

#[test]
fn u_f12_s3_diagram_shape_selection_batch() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock {
        id: tw_model::NodeId::new(),
        shape: ShapeData {
            shape_type: ShapeKind::Diagram,
            width: 432.0,
            height: 216.0,
        },
        wrap: tw_model::TextWrap::Inline,
        style: ShapeStyle::placeholder(),
        paragraphs: Vec::new(),
        preview_image: None,
        chart_data: None,
        chart_part: None,
        diagram_data_part: None,
        diagram_layout_part: None,
    })];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);

    assert_eq!(list.shape_selection_batch.shape_ids.len(), 1);
    assert_eq!(list.shape_selection_batch.rects.len(), 4);
}
