//! Clicking (or pressing End) past the last glyph must land after the last
//! character, including the spaces that never reach the glyph list.

use tw_layout::LayoutEngine;
use tw_model::Document;

fn line_end_offset_for(text: &str) -> usize {
    let mut engine = LayoutEngine::new();
    let mut doc = Document::new();
    doc.paragraph_at_mut(0, 0)
        .unwrap()
        .runs[0]
        .text_mut()
        .unwrap()
        .push_str(text);
    let _ = engine.layout_document(&doc);

    let map = engine.line_map(0).expect("typed text should layout");
    let line = &map.lines[0];
    map.hit_test(line.x + line.width + 100.0, line.y)
        .expect("blank area right of the line resolves a caret")
        .char_offset
}

#[test]
fn blank_area_click_counts_spaces() {
    // Spaces rasterize to empty bitmaps, so they are absent from `glyphs`.
    // Deriving the offset from the glyph count used to leave the caret one
    // position short per space.
    assert_eq!(line_end_offset_for("alpha beta"), 10);
    assert_eq!(line_end_offset_for("a b c"), 5);
}

#[test]
fn blank_area_click_on_a_space_free_line_is_unchanged() {
    assert_eq!(line_end_offset_for("alpha"), 5);
}

#[test]
fn trailing_space_is_still_reachable() {
    assert_eq!(line_end_offset_for("alpha "), 6);
}
