//! The pending-reflow stale floor must track the pages that genuinely still hold
//! pre-edit geometry, not every page after the current pass's short prefix.

use tw_layout::LayoutEngine;
use tw_model::{Block, Document, NodeId, Paragraph};

/// Continuous prose so an edit near the front shifts every later page and the
/// synchronous pass is guaranteed to hit the page cap.
fn flowing_document(paragraphs: usize) -> Document {
    let mut doc = Document::new();
    let filler = "The quick brown fox jumps over the lazy dog while the sleepy cat \
                  watches from a sunny windowsill and the kettle boils.";
    doc.sections[0].blocks = (0..paragraphs)
        .map(|idx| Block::Paragraph(Paragraph::with_text(format!("{idx}. {filler}"))))
        .collect();
    doc
}

fn run_id_at(doc: &Document, block: usize) -> NodeId {
    doc.sections[0].blocks[block]
        .paragraph()
        .expect("paragraph")
        .runs[0]
        .id
}

fn insert(doc: &mut Document, block: usize, text: &str) {
    let para = doc.sections[0].blocks[block]
        .paragraph_mut()
        .expect("paragraph");
    para.runs[0]
        .text_mut()
        .expect("text run")
        .insert_str(0, text);
}

#[test]
fn converged_pass_does_not_reopen_pages_already_caught_up() {
    let mut doc = flowing_document(400);
    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    assert!(engine.pending_reflow_page().is_none(), "full pass is caught up");

    // A large insert on page 0 shifts the whole document, so the pass caps.
    insert(&mut doc, 0, &"PREFIX ".repeat(40));
    engine.invalidate_nodes(&doc, &[run_id_at(&doc, 0)]);
    engine.layout_document(&doc);
    let first_floor = engine
        .pending_reflow_page()
        .expect("capped pass owes a forward reflow");

    // One background chunk moves the floor forward.
    engine.continue_layout(&doc);
    let advanced_floor = engine
        .pending_reflow_page()
        .expect("still catching up after one chunk");
    assert!(
        advanced_floor > first_floor,
        "background chunk should advance the stale floor: {first_floor} -> {advanced_floor}"
    );

    // A tiny edit that reflows without shifting anything downstream converges.
    // It must not drag the stale floor back over pages the background pass has
    // already rebuilt.
    insert(&mut doc, 0, "x");
    engine.invalidate_nodes(&doc, &[run_id_at(&doc, 0)]);
    engine.layout_document(&doc);
    assert_eq!(
        engine.pending_reflow_page(),
        Some(advanced_floor),
        "a converged pass proves the tail is unchanged, so the stale floor holds"
    );
}

#[test]
fn caught_up_layout_reports_no_stale_floor() {
    let mut doc = flowing_document(60);
    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);

    insert(&mut doc, 0, &"PREFIX ".repeat(40));
    engine.invalidate_nodes(&doc, &[run_id_at(&doc, 0)]);
    engine.layout_document(&doc);

    let mut guard = 0;
    while engine.has_pending_reflow() {
        engine.continue_layout(&doc);
        guard += 1;
        assert!(guard < 1000, "background reflow did not terminate");
    }
    assert_eq!(engine.pending_reflow_page(), None);
}
