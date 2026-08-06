//! document_tail_hit must reflect full buffer length after typing.

use tw_core::LayoutCache;
use tw_edit::{Command, EditSession};
use tw_layout::LayoutEngine;

#[test]
fn document_tail_hit_covers_full_typed_text() {
    let mut session = EditSession::new();
    let run_id = session
        .document
        .sections[0]
        .blocks[0]
        .paragraph()
        .expect("paragraph")
        .runs[0]
        .id;

    let mut offset = 0usize;
    for ch in "alpha beta".chars() {
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: ch.to_string(),
            })
            .expect("insert");
        offset += 1;
    }

    let mut layout = LayoutEngine::new();
    layout.invalidate_all();
    layout.layout_document(&session.document);

    let mut cache = LayoutCache::default();
    cache.update_from_session(&layout, &session.document, &session.buffer);

    let hit = cache.document_tail_hit(0).expect("tail hit");
    assert_eq!(hit.run_id, run_id);
    assert_eq!(hit.char_offset, 10);
    assert_eq!(session.buffer.len(run_id), 10);

    let text = cache
        .text_in_range(run_id, 0, hit.run_id, hit.char_offset)
        .expect("text");
    assert_eq!(text, "alpha beta");
}
