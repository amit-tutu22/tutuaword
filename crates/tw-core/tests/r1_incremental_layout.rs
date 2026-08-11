//! R1.1 exit gates: incremental layout reflow scope and typing latency on large docs.

use std::time::{Duration, Instant};

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_layout::LayoutEngine;
use tw_model::{Block, Document, Paragraph, RunContent};
use tw_native::NativeFormat;

const P50_GATE_MS: f64 = 10.0;
const P99_GATE_MS: f64 = 15.0;
const SAMPLE_COUNT: usize = 100;

fn fifty_page_document() -> Document {
    let mut doc = Document::new();
    let mut blocks = vec![Block::Paragraph(Paragraph::with_text("Page zero"))];
    for page in 1..50 {
        let mut para = Paragraph::with_text(format!("Page {page}"));
        para.format.page_break_before = Some(true);
        blocks.push(Block::Paragraph(para));
    }
    doc.sections[0].blocks = blocks;
    doc
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    let idx = ((sorted.len() as f64 * pct).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len().saturating_sub(1));
    sorted[idx]
}

#[test]
fn u_r1_incremental_layout_reflows_at_most_three_pages() {
    let doc = fifty_page_document();
    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    assert_eq!(engine.page_count(), 50, "fixture should produce 50 pages");

    let run_id = doc.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    let mut edited = doc.clone();
    if let RunContent::Text(text) = &mut edited.sections[0].blocks[0]
        .paragraph_mut()
        .unwrap()
        .runs[0]
        .content
    {
        *text = "Page zero — edited".into();
    }

    engine.invalidate_nodes(&edited, &[run_id]);
    engine.layout_document(&edited);

    assert!(
        engine.last_relayout_pages() <= 3,
        "expected <=3 synchronous page reflows, got {}",
        engine.last_relayout_pages()
    );
    assert_eq!(engine.page_count(), 50);
}

#[test]
fn i_r1_typing_latency_50p_p99_under_15ms() {
    if cfg!(debug_assertions) {
        return;
    }

    let doc = fifty_page_document();
    let bytes = NativeFormat::export(&doc).expect("export fifty-page fixture");

    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let open_id = session
        .open_bytes_with_path(bytes, Some("fifty.twdoc".into()))
        .expect("open enqueued");
    assert!(matches!(
        session.wait_for_response(open_id, Duration::from_secs(30)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));

    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .expect("page 0 hit")
        .run_id;

    let mut offset = session
        .get_display_list_bytes()
        .document_text
        .len()
        .min(1);

    // Warm up incremental layout + display-list cache.
    for _ in 0..50 {
        let request_id = session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: "w".into(),
            })
            .expect("warmup insert");
        assert!(matches!(
            session.wait_for_response(request_id, Duration::from_secs(5)),
            WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
        ));
        offset += 1;
    }

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT + 10 {
        let request_id = session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: "x".into(),
            })
            .expect("insert enqueued");
        let start = Instant::now();
        let outcome = session.wait_for_response(request_id, Duration::from_secs(5));
        assert!(
            matches!(outcome, WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })),
            "insert should complete: {outcome:?}"
        );
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
        offset += 1;
    }

    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let samples = &samples[10..];
    let p50 = percentile(&samples, 0.50);
    let p99 = percentile(&samples, 0.99);
    assert!(
        p50 < P50_GATE_MS,
        "p50 {:.2} ms exceeds {:.0} ms on 50-page doc",
        p50,
        P50_GATE_MS
    );
    assert!(
        p99 < P99_GATE_MS,
        "p99 {:.2} ms exceeds {:.0} ms on 50-page doc (p50 {:.2} ms)",
        p99,
        P99_GATE_MS,
        p50
    );
}
