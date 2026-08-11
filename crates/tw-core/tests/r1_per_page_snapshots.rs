//! R1.3 exit gate: per-page snapshot store retains Arc-shared pages across edits.

use std::sync::Arc;
use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph, RunContent};
use tw_native::NativeFormat;

/// Proxy for the 500-page / <400 MB budget in docs/performance-budgets.md.
/// One resident copy per page (~50 KB each) stays well under budget; a full
/// duplicate set after a keystroke would exceed this threshold.
const MAX_STORED_PAGE_BYTES: usize = 200 * 1024 * 1024;

fn five_hundred_page_document() -> Document {
    let mut doc = Document::new();
    let mut blocks = vec![Block::Paragraph(Paragraph::with_text("Page zero"))];
    for page in 1..500 {
        let mut para = Paragraph::with_text(format!("Page {page}"));
        para.format.page_break_before = Some(true);
        blocks.push(Block::Paragraph(para));
    }
    doc.sections[0].blocks = blocks;
    doc
}

#[test]
fn r1_per_page_snapshots_share_unchanged_pages_after_keystroke() {
    let doc = five_hundred_page_document();
    let bytes = NativeFormat::export(&doc).expect("export 500-page fixture");

    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let open_id = session
        .open_bytes_with_path(bytes, Some("five-hundred.twdoc".into()))
        .expect("open enqueued");
    assert!(matches!(
        session.wait_for_response(open_id, Duration::from_secs(60)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));

    let before = session.get_display_list_bytes();
    assert_eq!(before.page_count, 500);
    let unchanged_ptrs: Vec<*const Vec<u8>> = (1..500)
        .filter_map(|page| {
            session
                .page_display_list(page)
                .map(|snap| Arc::as_ptr(&snap.bytes))
        })
        .collect();
    assert_eq!(unchanged_ptrs.len(), 499);
    let stored_before = before.total_stored_page_bytes();

    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .expect("page 0 hit")
        .run_id;
    let offset = session.get_display_list_bytes().document_text.len().min(1);

    let page0_before = session
        .page_display_list(0)
        .expect("page 0 before edit");
    let page0_before_ptr = Arc::as_ptr(&page0_before.bytes);

    let request_id = session
        .apply(Command::InsertText {
            run_id,
            offset,
            text: "x".into(),
        })
        .expect("insert enqueued");
    let edit_event = match session.wait_for_response(request_id, Duration::from_secs(10)) {
        WaitOutcome::Matched(event) => event,
        other => panic!("insert should complete: {other:?}"),
    };
    let BridgeEvent::DisplayListReady { page: dirty_page, version, .. } = edit_event else {
        panic!("expected DisplayListReady, got {edit_event:?}");
    };
    assert_eq!(dirty_page, 0);
    assert!(version > before.version);

    let after = session.get_display_list_bytes();
    assert_eq!(after.page_count, 500);
    assert!(
        after.total_stored_page_bytes() <= MAX_STORED_PAGE_BYTES,
        "stored page bytes {} exceed budget proxy {}",
        after.total_stored_page_bytes(),
        MAX_STORED_PAGE_BYTES
    );
    assert!(
        after.total_stored_page_bytes() <= stored_before.saturating_mul(11) / 10,
        "keystroke should not duplicate entire page store (before={stored_before}, after={})",
        after.total_stored_page_bytes()
    );

    for page in 1..500 {
        let current = session
            .page_display_list(page)
            .expect("unchanged page should remain addressable");
        let ptr = Arc::as_ptr(&current.bytes);
        let before_idx = (page - 1) as usize;
        assert_eq!(
            ptr,
            unchanged_ptrs[before_idx],
            "page {page} bytes Arc should be reused after incremental edit"
        );
    }

    let page0 = session.page_display_list(0).expect("page 0");
    assert_eq!(page0.version, version);
    assert_ne!(Arc::as_ptr(&page0.bytes), page0_before_ptr);
}

#[test]
fn r1_snapshot_read_returns_arc_without_deep_clone() {
    let doc = five_hundred_page_document();
    let bytes = NativeFormat::export(&doc).expect("export");

    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));

    let open_id = session.open_bytes(bytes).expect("open");
    assert!(matches!(
        session.wait_for_response(open_id, Duration::from_secs(60)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));

    let snap_a = session.get_display_list_bytes();
    let snap_b = session.get_display_list_bytes();
    assert!(Arc::ptr_eq(&snap_a.pages, &snap_b.pages));

    let page1_a = session.page_display_list(1).expect("page 1");
    let page1_b = session.page_display_list(1).expect("page 1");
    assert!(Arc::ptr_eq(&page1_a, &page1_b));
    assert!(Arc::ptr_eq(&page1_a.bytes, &page1_b.bytes));
}

#[test]
fn r1_per_page_version_tracks_layout_epoch() {
    let mut doc = Document::new();
    if let RunContent::Text(text) = &mut doc.sections[0].blocks[0]
        .paragraph_mut()
        .unwrap()
        .runs[0]
        .content
    {
        *text = "Hello".into();
    }
    let bytes = NativeFormat::export(&doc).unwrap();

    let session = Session::new();
    session.wait_for_startup(Duration::from_secs(5));
    let open_id = session.open_bytes(bytes).unwrap();
    session.wait_for_response(open_id, Duration::from_secs(10));

    let v0 = session.page_display_version(0).unwrap();
    let run_id = session.hit_test(0, 72.0, 83.0).unwrap().run_id;
    let req = session
        .apply(Command::InsertText {
            run_id,
            offset: 5,
            text: "!".into(),
        })
        .unwrap();
    session.wait_for_response(req, Duration::from_secs(5));
    let v1 = session.page_display_version(0).unwrap();
    assert!(v1 > v0);
}
