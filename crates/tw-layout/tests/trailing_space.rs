//! Trailing whitespace on the last paragraph line must layout (visible gap + caret).

use tw_layout::LayoutEngine;
use tw_model::Document;

#[test]
fn trailing_space_on_last_line_extends_run_map() {
    let mut engine = LayoutEngine::new();
    let mut doc = Document::new();
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;

    doc.paragraph_at_mut(0, 0)
        .unwrap()
        .runs[0]
        .text_mut()
        .unwrap()
        .push_str("A ");

    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).expect("layout should produce a line map");
    let (x_start, x_end, _, _) = map.lines[0].run_map[0];
    let (x_after_a, _, _) = map.caret_at(run_id, 1).expect("after A");
    let (x_after_space, _, _) = map.caret_at(run_id, 2).expect("after space");

    let mut baseline_engine = LayoutEngine::new();
    let mut baseline_doc = Document::new();
    baseline_doc.paragraph_at_mut(0, 0)
        .unwrap()
        .runs[0]
        .text_mut()
        .unwrap()
        .push_str("A");
    let _ = baseline_engine.layout_document(&baseline_doc);
    let (_, x_end_a_only, _, _) = baseline_engine.line_map(0).unwrap().lines[0].run_map[0];

    assert!(x_end > x_end_a_only, "trailing space should widen beyond 'A' alone");
    assert!(x_end > x_start, "trailing space should widen the run segment");
    assert!(
        x_after_space > x_after_a,
        "caret should advance across trailing space before the next character"
    );
}
