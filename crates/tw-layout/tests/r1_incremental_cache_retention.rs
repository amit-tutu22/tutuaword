//! P1-5: an incremental pass refreshes caches for the pages it reflowed only.

use tw_layout::LayoutEngine;
use tw_model::{Block, Document, Paragraph, RunContent};

fn paged_document(pages: usize) -> Document {
    let mut doc = Document::new();
    let mut blocks = vec![Block::Paragraph(Paragraph::with_text("Page zero"))];
    for page in 1..pages {
        let mut para = Paragraph::with_text(format!("Page {page}"));
        para.format.page_break_before = Some(true);
        blocks.push(Block::Paragraph(para));
    }
    doc.sections[0].blocks = blocks;
    doc
}

#[test]
fn incremental_pass_only_touches_reflowed_pages() {
    let doc = paged_document(50);
    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    assert_eq!(engine.page_count(), 50);

    let untouched_before = engine.line_map(30).expect("page 30 line map").clone();
    let untouched_layout_before = engine.page_layout(30).expect("page 30 layout").clone();

    let run_id = doc.sections[0].blocks[0].paragraph().unwrap().runs[0].id;
    let mut edited = doc.clone();
    if let RunContent::Text(text) = &mut edited.sections[0].blocks[0]
        .paragraph_mut()
        .unwrap()
        .runs[0]
        .content
    {
        *text = "Page zero edited".into();
    }
    engine.invalidate_nodes(&edited, &[run_id]);
    engine.layout_document(&edited);

    assert!(engine.last_pass_incremental());
    assert_eq!(
        engine.dirty_pages(),
        &[0],
        "only the edited page should be rebuilt and re-cached"
    );
    assert_eq!(engine.page_count(), 50);
    assert_eq!(engine.line_map(30), Some(&untouched_before));
    assert_eq!(engine.page_layout(30), Some(&untouched_layout_before));
    assert!(
        !engine.has_pending_reflow(),
        "reflow converged on page 1, so nothing should be left pending"
    );
}

#[test]
fn shrinking_incremental_pass_drops_caches_for_removed_pages() {
    let doc = paged_document(6);
    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    assert_eq!(engine.page_count(), 6);

    // Dropping the page break folds the last page into its predecessor.
    let mut edited = doc.clone();
    let run_id = {
        let para = edited.sections[0].blocks[5].paragraph_mut().unwrap();
        para.format.page_break_before = None;
        para.runs[0].id
    };
    engine.invalidate_nodes(&edited, &[run_id]);
    engine.layout_document(&edited);

    assert_eq!(engine.page_count(), 5);
    assert!(engine.line_map(5).is_none(), "stale line map for dropped page");
    assert!(
        engine.page_layout(5).is_none(),
        "stale page cache entry for dropped page"
    );
}
