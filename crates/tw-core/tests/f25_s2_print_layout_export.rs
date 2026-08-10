//! F25.S2 — session export with scale/margins.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_pdf::{PrintLayoutOptions, PrintScaleMode};

#[test]
fn i_f25_s2_export_pdf_for_print_scaled() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .map(|hit| hit.run_id)
        .expect("empty document is editable");
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Scaled print sample".into(),
        })
        .expect("insert");

    let layout = PrintLayoutOptions {
        scale_mode: PrintScaleMode::CustomPercent,
        scale_percent: 50.0,
        margin_left: 36.0,
        margin_right: 36.0,
        margin_top: 36.0,
        margin_bottom: 36.0,
        ..Default::default()
    };
    let request_id = session
        .export_pdf_for_print(layout, None)
        .expect("export_pdf_for_print enqueued");
    match session.wait_for_response(request_id, Duration::from_secs(30)) {
        WaitOutcome::Matched(BridgeEvent::DocumentSaved { request_id: rid, data }) => {
            assert_eq!(rid, request_id);
            assert!(data.starts_with(b"%PDF"));
            let s = String::from_utf8_lossy(&data);
            assert!(
                s.contains("0.500000 0 0 0.500000") || s.contains(" cm\n"),
                "scaled print PDF should include a CTM"
            );
        }
        other => panic!("expected DocumentSaved with PDF, got {other:?}"),
    }
}
