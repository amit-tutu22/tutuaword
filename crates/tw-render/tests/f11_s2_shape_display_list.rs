//! U-F11-S2-shape-display-list — editable shapes emit path batch geometry.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ShapeBlock, ShapeKind, ShapeStyle};
use tw_render::DisplayListBuilder;

fn doc_with_shape(shape_type: ShapeKind, width: f32, height: f32) -> tw_model::Document {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks.push(Block::ShapeBlock(ShapeBlock::new(
        shape_type,
        width,
        height,
        ShapeStyle::inserted_default(),
    )));
    doc
}

#[test]
fn u_f11_s2_shape_display_list() {
    let doc = doc_with_shape(ShapeKind::Rectangle, 100.0, 60.0);
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);

    assert!(
        !list.path_batch.points.is_empty(),
        "rectangle stroke should produce path segments"
    );
    assert!(
        !list.rect_batch.rects.is_empty(),
        "rectangle fill should produce a rect batch entry"
    );
}

#[test]
fn u_f11_s2_line_shape_uses_path_batch() {
    let doc = doc_with_shape(ShapeKind::Line, 120.0, 60.0);
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];
    assert!(page.boxes.iter().any(|b| matches!(b, LayoutBox::Shape(_))));

    let list = DisplayListBuilder::from_page_without_atlas(page, 1);
    assert_eq!(list.path_batch.points.len(), 4);
    assert_eq!(list.path_batch.colors.len(), 1);
}
