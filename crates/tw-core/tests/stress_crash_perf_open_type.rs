//! Crash/perf Phase stress: multipage open+type, atlas reuse under load.

use std::time::{Duration, Instant};

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph};
use tw_native::NativeFormat;
use tw_render::DisplayListBuilder;

fn multipage_document(pages: usize) -> Document {
    let mut doc = Document::new();
    let mut blocks = Vec::with_capacity(pages);
    for page in 0..pages {
        let mut para = Paragraph::with_text(format!(
            "Page {page} — stress typing with spaces and\ttabs for ¶ marks."
        ));
        if page > 0 {
            para.format.page_break_before = Some(true);
        }
        blocks.push(Block::Paragraph(para));
    }
    doc.sections[0].blocks = blocks;
    doc
}

fn open_multipage(session: &Session, pages: usize) {
    let bytes = NativeFormat::export(&multipage_document(pages)).expect("export fixture");
    let open_id = session
        .open_bytes_with_path(bytes, Some(format!("{pages}-page.twdoc")))
        .expect("open enqueued");
    assert!(matches!(
        session.wait_for_response(open_id, Duration::from_secs(120)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));
    let snap = session.get_display_list_bytes();
    assert!(
        snap.page_count as usize >= pages.min(2),
        "expected multipage layout, got page_count={}",
        snap.page_count
    );
}

fn type_burst(session: &Session, chars: usize) {
    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .expect("editable page 0")
        .run_id;
    let mut offset = 0usize;
    for i in 0..chars {
        let text = if i % 7 == 0 { " " } else { "x" };
        let request_id = session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: text.into(),
            })
            .expect("insert enqueued");
        match session.wait_for_response(request_id, Duration::from_secs(30)) {
            WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. }) => {}
            other => panic!("keystroke {i} failed: {other:?}"),
        }
        offset += text.chars().count();

        // Formatting-mark path: page bytes must carry marks without a full atlas copy
        // when the atlas generation is unchanged.
        if let Some(page) = session.page_display_list(0) {
            let list = DisplayListBuilder::from_bytes(page.bytes.as_ref())
                .expect("page display list decodes");
            assert!(
                list.atlas_pixels.is_empty(),
                "page wire format must omit atlas pixels"
            );
            assert!(
                !list.formatting_marks_batch.kinds.is_empty()
                    || !list.atlas_batch.transforms.is_empty(),
                "page should expose glyphs or formatting marks"
            );
        }
    }
}

#[test]
fn stress_ten_page_open_while_typing() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));
    let started = Instant::now();
    open_multipage(&session, 10);
    type_burst(&session, 40);
    assert!(
        started.elapsed() < Duration::from_secs(90),
        "ten-page open+type took {:?}",
        started.elapsed()
    );
}

/// Nightly / manual: 50-page DOCX-equivalent open while typing with ¶-mark payload.
#[test]
#[ignore = "slow multipage stress; run with --ignored"]
fn stress_fifty_page_open_while_typing() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));
    let started = Instant::now();
    open_multipage(&session, 50);
    type_burst(&session, 80);
    assert!(
        started.elapsed() < Duration::from_secs(180),
        "fifty-page open+type took {:?}",
        started.elapsed()
    );
}
