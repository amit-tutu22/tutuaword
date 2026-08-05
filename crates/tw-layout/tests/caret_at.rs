//! Caret geometry must follow the document position, not just click coordinates.

use tw_layout::LayoutEngine;
use tw_model::Document;

#[test]
fn caret_x_advances_with_character_offset() {
    let mut engine = LayoutEngine::new();
    let mut doc = Document::new();
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    doc.paragraph_at_mut(0, 0)
        .unwrap()
        .runs[0]
        .text_mut()
        .unwrap()
        .push_str("Hello");
    let _ = engine.layout_document(&doc);

    let map = engine.line_map(0).expect("typed text should layout");
    let (x0, _, _) = map.caret_at(run_id, 0).expect("offset 0");
    let (x3, _, _) = map.caret_at(run_id, 3).expect("offset 3");
    let (x5, _, _) = map.caret_at(run_id, 5).expect("offset 5");

    assert!(x3 > x0, "caret should move right as offset increases");
    assert!(x5 > x3, "caret should reach the end of the run");
}

#[test]
fn empty_run_caret_stays_at_line_start() {
    let mut engine = LayoutEngine::new();
    let doc = Document::new();
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).unwrap();
    let (x, y, height) = map.caret_at(run_id, 0).expect("empty run caret");
    assert!(x > 0.0);
    assert!(y > 0.0);
    assert!(height > 0.0);
}

#[test]
fn caret_advances_across_space_character() {
    let mut engine = LayoutEngine::new();
    let mut doc = Document::new();
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;

    doc.paragraph_at_mut(0, 0)
        .unwrap()
        .runs[0]
        .text_mut()
        .unwrap()
        .push_str("A B");

    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).expect("typed text should layout");

    // Offsets are character offsets within the run: "A B"
    // 0: before 'A'
    // 1: after 'A' (before space)
    // 2: after space (before 'B')
    // 3: after 'B'
    let (x_after_a, _, _) = map.caret_at(run_id, 1).expect("offset 1");
    let (x_after_space, _, _) = map.caret_at(run_id, 2).expect("offset 2");
    let (x_after_b, _, _) = map.caret_at(run_id, 3).expect("offset 3");

    assert!(x_after_space > x_after_a, "caret should move across space");
    assert!(x_after_b >= x_after_space, "caret should not move left across offsets");
}
