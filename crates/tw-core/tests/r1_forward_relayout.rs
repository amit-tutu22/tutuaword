//! P1-4: pages past the synchronous reflow cap are reflowed in the background.

use std::time::{Duration, Instant};

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph};
use tw_native::NativeFormat;

/// Continuous prose (no page breaks) so an edit on page 0 shifts every later page
/// and the reflow cannot converge inside the synchronous 3-page window.
fn flowing_document(paragraphs: usize) -> Document {
    let mut doc = Document::new();
    let filler = "The quick brown fox jumps over the lazy dog while the sleepy cat \
                  watches from a sunny windowsill and the kettle boils.";
    doc.sections[0].blocks = (0..paragraphs)
        .map(|idx| Block::Paragraph(Paragraph::with_text(format!("{idx}. {filler}"))))
        .collect();
    doc
}

fn wait_until<F: FnMut() -> bool>(timeout: Duration, mut condition: F) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    condition()
}

#[test]
fn background_relayout_catches_up_pages_beyond_the_sync_cap() {
    let bytes = NativeFormat::export(&flowing_document(1000)).expect("export fixture");
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));
    let open_id = session.open_bytes(bytes).expect("open enqueued");
    assert!(matches!(
        session.wait_for_response(open_id, Duration::from_secs(300)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));

    let page_count = session.page_count();
    assert!(
        page_count >= 40,
        "fixture should span ~50 pages, got {page_count}"
    );
    let last_page = page_count - 1;

    let run_id = session.hit_test(0, 72.0, 83.0).expect("page 0 hit").run_id;
    let request_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            // Long enough to push whole lines across every later page boundary.
            text: "PREFIX ".repeat(40),
        })
        .expect("insert enqueued");
    assert!(matches!(
        session.wait_for_response(request_id, Duration::from_secs(60)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    // The edit completes off the capped synchronous pass, so the tail of the
    // document is still reported stale at this point.
    let stale_at_completion = session.is_page_stale(last_page);

    let mut first_stale = None;
    assert!(
        wait_until(Duration::from_secs(300), || {
            first_stale = (0..session.page_count()).find(|&page| session.is_page_stale(page));
            first_stale.is_none()
        }),
        "page {first_stale:?} still stale after the background relayout window"
    );
    assert!(
        stale_at_completion,
        "page {last_page} should have been reported stale while reflow was pending"
    );

    assert!(
        session.hit_test(last_page, 100.0, 200.0).is_some(),
        "hit test should resolve on page {last_page} once reflow completed"
    );
    assert!(session
        .get_display_list_bytes()
        .document_text
        .starts_with("PREFIX "));
}
