//! R2.2: layout cache stores Arc<Document>; incremental updates share the same Arc.

use std::sync::Arc;
use tw_core::LayoutCache;
use tw_edit::{Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::Document;

#[test]
fn layout_cache_uses_arc_without_reclone_on_incremental() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello world".into(),
        })
        .unwrap();

    let doc_arc = Arc::new(session.document.clone());
    let mut layout = LayoutEngine::new();
    layout.invalidate_all();
    layout.layout_document(&session.document);

    let mut cache = LayoutCache::default();
    cache.update_from_session(&layout, Arc::clone(&doc_arc));
    let ptr_after_full = Arc::as_ptr(&cache.document_snapshot());

    layout.invalidate_all();
    layout.layout_document(&session.document);
    cache.update_from_session_incremental(&layout, doc_arc, 2, 1, false);
    let ptr_after_incremental = Arc::as_ptr(&cache.document_snapshot());

    assert_eq!(
        ptr_after_full, ptr_after_incremental,
        "incremental update should reuse the same Arc pointer"
    );
    assert_eq!(cache.document_version(), 2);
    assert_eq!(
        cache.document().paragraph_at(0, 0).unwrap().full_text(),
        "Hello world"
    );
}

#[test]
fn layout_cache_document_version_bumps_on_full_update() {
    let doc = Document::with_paragraph("Test");
    let doc_arc = Arc::new(doc);
    let mut layout = LayoutEngine::new();
    layout.layout_document(&doc_arc);

    let mut cache = LayoutCache::default();
    cache.update_from_session(&layout, Arc::clone(&doc_arc));
    let v1 = cache.document_version();

    cache.update_from_session(&layout, Arc::clone(&doc_arc));
    let v2 = cache.document_version();

    assert!(v2 > v1);
}
