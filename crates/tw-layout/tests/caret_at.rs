//! Caret geometry must follow the document position, not just click coordinates.

use tw_layout::LayoutEngine;
use tw_model::{Document, NodeId};

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

#[test]
fn caret_at_unknown_run_returns_none() {
    let mut engine = LayoutEngine::new();
    let doc = Document::new();
    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).unwrap();
    // Must not fall back to the first line — that pinned Enter-at-bottom carets
    // to the top of the page when the new paragraph lived on page N+1.
    assert!(
        map.caret_at(NodeId::new(), 0).is_none(),
        "unknown run must not resolve to another run's geometry"
    );
}

#[test]
fn caret_at_on_page_zero_is_none_for_wrapped_tail() {
    let mut doc = Document::new();
    let text: String = (0..80)
        .map(|i| format!("Sentence number {i} with enough words to wrap. "))
        .collect();
    doc.sections[0].blocks = vec![tw_model::Block::Paragraph(
        tw_model::Paragraph::with_text(text.clone()),
    )];
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    assert!(
        layout.pages.len() > 1,
        "fixture must wrap to a second page"
    );

    let page0 = engine.line_map(0).expect("page 0 line map");
    let page1 = engine.line_map(1).expect("page 1 line map");

    let tail_offset = text.chars().count();
    let mut page0_end = 0usize;
    for line in &page0.lines {
        for (idx, &(_, _, rid, seg_off)) in line.run_map.iter().enumerate() {
            if rid != run_id {
                continue;
            }
            let seg_chars = line.run_map_chars.get(idx).copied().unwrap_or(0);
            page0_end = page0_end.max(seg_off + seg_chars);
        }
    }
    assert!(page0_end > 0, "run should appear on page 0");

    assert!(
        page0_end < tail_offset,
        "wrapped run should continue beyond page 0"
    );
    assert!(
        page0.caret_at(run_id, page0_end + 1).is_none(),
        "page 0 must not resolve offsets laid out on page 1"
    );
    assert!(
        page1.caret_at(run_id, tail_offset).is_some(),
        "page 1 should resolve the document tail"
    );
}
