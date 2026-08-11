//! P1-6: growing the page count must not force a full display-list rebuild.

use std::sync::Arc;
use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph};
use tw_native::NativeFormat;

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
fn page_count_growth_reuses_untouched_page_snapshots() {
    let bytes = NativeFormat::export(&paged_document(10)).expect("export fixture");
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));
    let open_id = session.open_bytes(bytes).expect("open enqueued");
    assert!(matches!(
        session.wait_for_response(open_id, Duration::from_secs(60)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));
    assert_eq!(session.page_count(), 10);

    let before: Vec<*const Vec<u8>> = (0..9)
        .map(|page| {
            Arc::as_ptr(
                &session
                    .page_display_list(page)
                    .expect("page snapshot")
                    .bytes,
            )
        })
        .collect();

    // Overflowing the last paragraph appends a page, so the total page count
    // changes while every page before the edit keeps its layout.
    let tail_run = session
        .document()
        .sections[0]
        .blocks
        .last()
        .and_then(|block| block.paragraph())
        .map(|para| para.runs[0].id)
        .expect("tail run");
    let request_id = session
        .apply(Command::InsertText {
            run_id: tail_run,
            offset: 0,
            text: "overflowing the final page with a great deal of extra prose. "
                .repeat(120),
        })
        .expect("insert enqueued");
    assert!(matches!(
        session.wait_for_response(request_id, Duration::from_secs(30)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    assert!(
        session.page_count() > 10,
        "overflowing the tail paragraph should add pages, got {}",
        session.page_count()
    );
    for (page, expected) in before.iter().enumerate() {
        let current = session
            .page_display_list(page as u32)
            .expect("page still addressable");
        assert_eq!(
            Arc::as_ptr(&current.bytes),
            *expected,
            "page {page} was rebuilt even though its layout did not change"
        );
    }
}
