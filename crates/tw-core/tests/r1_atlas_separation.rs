//! R1.2 exit gate: atlas separated from page display lists keeps FFI transfer bounded.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph};
use tw_native::NativeFormat;

const MAX_FFI_BYTES: usize = 2 * 1024 * 1024;

fn ten_page_document() -> Document {
    let mut doc = Document::new();
    let mut blocks = vec![Block::Paragraph(Paragraph::with_text("Page zero"))];
    for page in 1..10 {
        let mut para = Paragraph::with_text(format!("Page {page}"));
        para.format.page_break_before = Some(true);
        blocks.push(Block::Paragraph(para));
    }
    doc.sections[0].blocks = blocks;
    doc
}

#[test]
fn r1_atlas_separation_keystroke_ffi_under_2mb() {
    let doc = ten_page_document();
    let bytes = NativeFormat::export(&doc).expect("export ten-page fixture");

    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let open_id = session
        .open_bytes_with_path(bytes, Some("ten.twdoc".into()))
        .expect("open enqueued");
    assert!(matches!(
        session.wait_for_response(open_id, Duration::from_secs(30)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));

    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .expect("page 0 hit")
        .run_id;
    let offset = session
        .get_display_list_bytes()
        .document_text
        .len()
        .min(1);

    let before_atlas_gen = session.atlas_resource().0;

    let request_id = session
        .apply(Command::InsertText {
            run_id,
            offset,
            text: "e".into(),
        })
        .expect("insert enqueued");
    let edit_event = match session.wait_for_response(request_id, Duration::from_secs(5)) {
        WaitOutcome::Matched(event) => event,
        other => panic!("insert should complete: {other:?}"),
    };
    let BridgeEvent::DisplayListReady { page: dirty_page, .. } = edit_event else {
        panic!("expected DisplayListReady, got {edit_event:?}");
    };

    // Simulate Flutter FFI: dirty page DL + atlas only when generation changes.
    let mut total_bytes = 0usize;
    if let Some(page_snap) = session.page_display_list(dirty_page) {
        total_bytes += page_snap.bytes.len();
    }

    let (after_atlas_gen, _width, _height, atlas_bytes) = session.atlas_resource();
    if after_atlas_gen != before_atlas_gen {
        total_bytes += atlas_bytes.len();
    }

    assert!(
        total_bytes < MAX_FFI_BYTES,
        "expected < {} bytes across page DL + atlas FFI fetches, got {} bytes",
        MAX_FFI_BYTES,
        total_bytes
    );

    // v4 page payloads must not embed atlas pixels (header only, no 16MB blob).
    let page_count = session.get_display_list_bytes().page_count as usize;
    for page in 0..page_count {
        let page_snap = session
            .page_display_list(page as u32)
            .expect("page snapshot");
        let page_bytes = page_snap.bytes.as_ref();
        assert!(page_bytes.len() >= 4);
        let file_version = u32::from_le_bytes(page_bytes[0..4].try_into().unwrap());
        assert_eq!(
            file_version, 4,
            "page {page} display list should be wire format v4"
        );
        assert!(
            page_bytes.len() < 512 * 1024,
            "page {page} DL unexpectedly large ({} bytes) — atlas may still be embedded",
            page_bytes.len()
        );
    }
}
