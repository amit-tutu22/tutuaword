//! U-F13-S3 — chart placeholder label and selection bounds.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ShapeBlock};
use tw_render::DisplayListBuilder;

#[test]
fn u_f13_s3_chart_label_in_layout() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::chart(432.0, 216.0))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];

    assert!(page.boxes.iter().any(|b| matches!(b, LayoutBox::Shape(_))));
    let has_label = page.boxes.iter().any(|b| {
        matches!(b, LayoutBox::TextLine(line) if !line.glyphs.is_empty())
    });
    assert!(has_label, "chart placeholder should include Chart label glyphs");
}

#[test]
fn u_f13_s3_chart_shape_selection_batch() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::chart(432.0, 216.0))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);

    assert_eq!(list.shape_selection_batch.shape_ids.len(), 1);
    assert_eq!(list.shape_selection_batch.rects.len(), 4);
}
