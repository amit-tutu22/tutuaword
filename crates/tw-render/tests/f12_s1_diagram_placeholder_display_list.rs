//! U-F12-S1 — imported SmartArt diagrams emit a bounding-box placeholder.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ShapeBlock, ShapeData, ShapeKind, ShapeStyle};
use tw_render::{DisplayListBuilder, SHAPE_PLACEHOLDER_COLOR};

#[test]
fn u_f12_s1_diagram_placeholder_rect_in_display_list() {
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
    let page = &layout.pages[0];
    assert!(
        page.boxes.iter().any(|b| matches!(b, LayoutBox::Shape(_))),
        "expected diagram shape layout box"
    );

    let list = DisplayListBuilder::from_page_without_atlas(page, 1);
    let mut found_fill = false;
    for (idx, chunk) in list.rect_batch.rects.chunks(4).enumerate() {
        if chunk.len() != 4 {
            continue;
        }
        let color = list.rect_batch.colors.get(idx).copied().unwrap_or(0);
        if color == SHAPE_PLACEHOLDER_COLOR
            && (chunk[2] - 432.0).abs() < 0.01
            && (chunk[3] - 216.0).abs() < 0.01
        {
            found_fill = true;
            break;
        }
    }
    assert!(found_fill, "expected diagram placeholder fill rect");
}
