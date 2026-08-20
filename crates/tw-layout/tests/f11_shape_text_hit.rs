//! Text-box / shape body text hit-testing.
use tw_layout::LayoutEngine;
use tw_model::{Block, ShapeBlock, ShapeKind, ShapeStyle};

#[test]
fn text_box_empty_paragraph_is_hittable() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::text_box(
        180.0,
        90.0,
        ShapeStyle::inserted_default(),
    ))];
    let mut engine = LayoutEngine::new();
    let _layout = engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");
    assert!(!map.lines.is_empty(), "text box must expose caret lines");
    let line = &map.lines[0];
    let hit = map
        .hit_test(line.x + 8.0, line.y)
        .expect("empty text box interior must hit caret");
    let shape = doc.sections[0].blocks[0].shape().unwrap();
    let run_id = shape.paragraphs[0].runs[0].id;
    assert_eq!(hit.run_id, run_id);
}

#[test]
fn rectangle_body_paragraph_is_hittable() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::new(
        ShapeKind::Rectangle,
        180.0,
        90.0,
        ShapeStyle::inserted_default(),
    ))];
    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");
    assert!(
        !map.lines.is_empty(),
        "rectangle body text must expose caret lines"
    );
    let line = &map.lines[0];
    let hit = map
        .hit_test(line.x + 8.0, line.y)
        .expect("rectangle interior must hit caret");
    let run_id = doc.sections[0].blocks[0]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;
    assert_eq!(hit.run_id, run_id);
}

#[test]
fn empty_ellipse_center_is_hittable() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::new(
        ShapeKind::Ellipse,
        200.0,
        120.0,
        ShapeStyle::inserted_default(),
    ))];
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let shape = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            tw_layout::LayoutBox::Shape(s) => Some(s),
            _ => None,
        })
        .expect("ellipse shape");
    let map = engine.line_map(0).expect("line map");
    let hit = map
        .hit_test(shape.x + shape.width * 0.5, shape.y + shape.height * 0.5)
        .expect("center of empty ellipse must accept caret");
    let run_id = doc.sections[0].blocks[0]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;
    assert_eq!(hit.run_id, run_id);
}

#[test]
fn line_shape_has_no_text_hit() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::new(
        ShapeKind::Line,
        180.0,
        90.0,
        ShapeStyle::inserted_default(),
    ))];
    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");
    assert!(
        map.lines.is_empty(),
        "line shapes should not expose text lines, got {}",
        map.lines.len()
    );
}

#[test]
fn word_art_body_paragraph_is_hittable() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::word_art(
        "WordArt",
        240.0,
        72.0,
    ))];
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let shape = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            tw_layout::LayoutBox::Shape(s) => Some(s),
            _ => None,
        })
        .expect("word art shape");
    let map = engine.line_map(0).expect("line map");
    let hit = map
        .hit_test(shape.x + 12.0, shape.y + 20.0)
        .expect("WordArt body text must accept caret");
    let run_id = doc.sections[0].blocks[0]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;
    assert_eq!(hit.run_id, run_id);
}
