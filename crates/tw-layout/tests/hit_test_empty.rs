//! Empty-paragraph hit testing — an empty page must still resolve a caret run.

use tw_layout::LayoutEngine;
use tw_model::Document;

#[test]
fn empty_document_hit_test_returns_first_run() {
    let mut engine = LayoutEngine::new();
    let doc = Document::new();
    let expected = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    let _ = engine.layout_document(&doc);

    let map = engine.line_map(0).expect("empty doc still lays out one page");
    assert!(
        !map.lines.is_empty(),
        "empty paragraph should produce a caret line"
    );

    let hit = map
        .hit_test(72.0, 72.0 + 11.0)
        .expect("empty line must be hit-testable near the margin");
    assert_eq!(hit.run_id, expected);
    assert_eq!(hit.char_offset, 0);
}

#[test]
fn empty_line_run_id_is_in_the_document_model() {
    // Regression: split_paragraph_at_page_breaks used to seed segments with
    // Paragraph::new(), injecting a phantom run id that hit-testing returned
    // but InsertText could not find.
    let mut engine = LayoutEngine::new();
    let doc = Document::new();
    let model_ids: Vec<_> = doc
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .map(|r| r.id)
        .collect();
    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).unwrap();
    for line in &map.lines {
        for &(_, _, run_id, _) in &line.run_map {
            assert!(
                model_ids.contains(&run_id),
                "layout run {run_id} must exist in the document model"
            );
        }
    }
}

#[test]
fn empty_line_hit_test_accepts_x_past_zero_width_run() {
    let mut engine = LayoutEngine::new();
    let doc = Document::new();
    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).unwrap();
    let line = &map.lines[0];

    // Click well to the right of the zero-width caret target.
    let hit = map
        .hit_test(line.x + 40.0, line.y)
        .expect("blank part of an empty line should still resolve a run");
    assert_eq!(hit.run_id, doc.paragraph_at(0, 0).unwrap().runs[0].id);
}

#[test]
fn hit_test_below_all_lines_falls_back_to_last_run() {
    let mut engine = LayoutEngine::new();
    let doc = Document::new();
    let expected = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).unwrap();

    // Single-line page: last == first. Below-content must not jump to a
    // phantom "top" — it should stay on the content edge (last line).
    let hit = map
        .hit_test(100.0, 700.0)
        .expect("clicks below content should land on the last run");
    assert_eq!(hit.run_id, expected);
}
