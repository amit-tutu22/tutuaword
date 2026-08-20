//! U-F13-S3 — chart placeholder label and selection bounds.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ChartKind, ShapeBlock};
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
fn u_f13_s3_chart_legend_series_names_in_layout() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::chart(432.0, 252.0))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];

    let decorative_lines: Vec<_> = page
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) if line.decorative && !line.glyphs.is_empty() => Some(line),
            _ => None,
        })
        .collect();
    // Title + Series 1 + Series 2
    assert!(
        decorative_lines.len() >= 3,
        "expected chart title and legend series labels, got {}",
        decorative_lines.len()
    );

    let list = DisplayListBuilder::from_page(&layout.pages[0], engine.atlas(), 1);
    assert!(
        list.atlas_batch.transforms.len() >= 20,
        "legend labels must emit glyph draws, got {}",
        list.atlas_batch.transforms.len() / 2
    );
}

#[test]
fn u_f13_s3_pie_legend_uses_category_names() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::chart_with_kind(
        432.0,
        252.0,
        ChartKind::Pie,
    ))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let decorative = layout.pages[0]
        .boxes
        .iter()
        .filter(|b| matches!(b, LayoutBox::TextLine(line) if line.decorative))
        .count();
    // Title + up to 4 category labels
    assert!(
        decorative >= 3,
        "pie chart should layout category legend labels, got {decorative}"
    );
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
