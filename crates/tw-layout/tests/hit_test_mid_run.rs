//! Mid-run clicks must resolve character offsets from X, not segment starts.

use tw_layout::LayoutEngine;
use tw_model::Document;

#[test]
fn mid_run_click_resolves_near_halfway_offset() {
    let mut engine = LayoutEngine::new();
    let mut doc = Document::new();
    let text = "Hello world";
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    doc.paragraph_at_mut(0, 0)
        .unwrap()
        .runs[0]
        .text_mut()
        .unwrap()
        .push_str(text);
    let _ = engine.layout_document(&doc);

    let map = engine.line_map(0).expect("typed text should layout");
    let line = &map.lines[0];
    let &(x_start, x_end, _, _) = line.run_map.first().expect("single run segment");
    let mid_x = x_start + (x_end - x_start) * 0.5;

    let hit = map
        .hit_test(mid_x, line.y)
        .expect("mid-run click should hit");
    assert_eq!(hit.run_id, run_id);
    assert!(
        hit.char_offset >= 4 && hit.char_offset <= 7,
        "mid-run offset should be near 5, got {}",
        hit.char_offset
    );

    let (geom_x, _, _) = map
        .caret_at(run_id, hit.char_offset)
        .expect("caret_at for hit offset");
    assert!(
        (geom_x - mid_x).abs() < (x_end - x_start) * 0.35,
        "painted caret x ({geom_x}) should track hit offset, not raw probe ({mid_x})"
    );
}

#[test]
fn caret_at_run_end_uses_x_end_and_past_end_is_none() {
    let mut engine = LayoutEngine::new();
    let mut doc = Document::new();
    let text = "Hello";
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    doc.paragraph_at_mut(0, 0)
        .unwrap()
        .runs[0]
        .text_mut()
        .unwrap()
        .push_str(text);
    let _ = engine.layout_document(&doc);

    let map = engine.line_map(0).unwrap();
    let line = &map.lines[0];
    let (x_start, x_end, _, _) = line.run_map[0];
    let len = text.chars().count();

    let (x_end_geom, _, _) = map.caret_at(run_id, len).expect("offset at run end");
    assert!(
        (x_end_geom - x_end).abs() < 1.0,
        "caret_at(len) should land at x_end ({x_end}), got {x_end_geom}"
    );
    assert!(
        map.caret_at(run_id, len + 1).is_none(),
        "offset past run end must not invent geometry"
    );
    let (x0, _, _) = map.caret_at(run_id, 0).unwrap();
    assert!((x0 - x_start).abs() < 1.0);
}
