//! Reports how long pages stay stale after a single front-of-document edit.
//! Run with `cargo test -p tw-core --release --test r1_stale_window_probe -- --nocapture --ignored`.

use std::time::{Duration, Instant};

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph};
use tw_native::NativeFormat;

fn flowing_document(paragraphs: usize) -> Document {
    let mut doc = Document::new();
    let filler = "The quick brown fox jumps over the lazy dog while the sleepy cat \
                  watches from a sunny windowsill and the kettle boils.";
    doc.sections[0].blocks = (0..paragraphs)
        .map(|idx| Block::Paragraph(Paragraph::with_text(format!("{idx}. {filler}"))))
        .collect();
    doc
}

#[test]
#[ignore = "measurement, not a gate"]
fn report_stale_window_for_50_page_document() {
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
    let run_id = session.hit_test(0, 72.0, 83.0).expect("page 0 hit").run_id;

    let started = Instant::now();
    let request_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "PREFIX ".repeat(40),
        })
        .expect("insert enqueued");
    assert!(matches!(
        session.wait_for_response(request_id, Duration::from_secs(60)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));
    let keystroke_ms = started.elapsed().as_secs_f64() * 1000.0;

    let initial_stale = (0..page_count)
        .filter(|&page| session.is_page_stale(page))
        .count();

    // Sample when the middle and last pages become trustworthy again.
    let middle = page_count / 2;
    let last = page_count - 1;
    let mut middle_ms = None;
    let deadline = Instant::now() + Duration::from_secs(600);
    let last_ms = loop {
        if middle_ms.is_none() && !session.is_page_stale(middle) {
            middle_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
        }
        if !session.is_page_stale(last) {
            break started.elapsed().as_secs_f64() * 1000.0;
        }
        assert!(Instant::now() < deadline, "background reflow never finished");
        std::thread::sleep(Duration::from_millis(2));
    };

    println!(
        "pages={page_count} stale_after_edit={initial_stale} \
         keystroke_ack={keystroke_ms:.1}ms \
         page{middle}_fresh_at={:.1}ms page{last}_fresh_at={last_ms:.1}ms",
        middle_ms.unwrap_or_default(),
    );
}
