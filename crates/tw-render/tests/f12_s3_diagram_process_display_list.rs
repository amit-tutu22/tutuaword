//! U-F12-S3 — inserted SmartArt paints a process preview.

use tw_layout::LayoutEngine;
use tw_model::{Block, ShapeBlock};
use tw_render::DisplayListBuilder;

#[test]
fn u_f12_s3_diagram_process_preview_rects() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::diagram(432.0, 216.0))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);

    // Placeholder frame + three process nodes.
    assert!(
        list.rect_batch.rects.len() / 4 >= 4,
        "expected placeholder + process nodes, got {} floats",
        list.rect_batch.rects.len()
    );
    assert!(
        list.rect_batch.colors.iter().any(|&c| c == 0xFF5B9BD5),
        "expected process node fill"
    );
    assert!(
        !list.path_batch.points.is_empty(),
        "expected connector arrow paths"
    );
}
