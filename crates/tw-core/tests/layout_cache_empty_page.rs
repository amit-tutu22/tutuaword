//! Later pages must remain editable even when mostly blank.

use tw_core::LayoutCache;
use tw_layout::LayoutEngine;
use tw_model::{Block, Document, Paragraph};

#[test]
fn later_page_hit_test_resolves_caret() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(Paragraph::with_text("Page one content.")),
        Block::Paragraph({
            let mut p = Paragraph::with_text("");
            p.format.page_break_before = Some(true);
            p
        }),
    ];

    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    assert!(
        engine.page_count() >= 2,
        "page break should produce at least two pages"
    );

    let mut cache = LayoutCache::default();
    cache.update_from_engine(&engine);

    let hit = cache
        .hit_test(1, 100.0, 200.0)
        .expect("page 2 must resolve a caret target");
    assert_eq!(hit.page, 1);
}
