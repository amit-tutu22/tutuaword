//! U-F13-S3 — inserted charts with sample data paint Word-like columns.

use tw_layout::LayoutEngine;
use tw_model::{Block, ChartKind, ShapeBlock};
use tw_render::DisplayListBuilder;

#[test]
fn u_f13_s3_chart_sample_data_emits_bar_rects() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::chart(432.0, 252.0))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);

    // White surface + clustered columns for 4 categories × 2 series.
    assert!(
        list.rect_batch.rects.len() / 4 >= 8,
        "expected chart surface + column rects, got {} floats",
        list.rect_batch.rects.len()
    );
    assert!(
        list.rect_batch.colors.iter().any(|&c| c == 0xFF4472C4),
        "expected Word-blue series fill"
    );
    assert!(
        list.rect_batch.colors.iter().any(|&c| c == 0xFFED7D31),
        "expected Word-orange series fill"
    );
}

#[test]
fn u_f13_s3_pie_chart_emits_slice_paths() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::chart_with_kind(
        432.0,
        252.0,
        ChartKind::Pie,
    ))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);

    assert!(
        list.path_batch.points.len() >= 16,
        "pie preview should emit radial slice paths"
    );
}
