//! F25.S1 — session export_pdf_for_print integration.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;

#[test]
fn i_f25_s1_export_pdf_for_print() {
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
            text: "Print integration sample".into(),
        })
        .expect("insert");

    let request_id = session
        .export_pdf_for_print(tw_pdf::PrintLayoutOptions::default(), None)
        .expect("export_pdf_for_print enqueued");
    match session.wait_for_response(request_id, Duration::from_secs(30)) {
        WaitOutcome::Matched(BridgeEvent::DocumentSaved { request_id: rid, data }) => {
            assert_eq!(rid, request_id);
            assert!(data.starts_with(b"%PDF"), "print PDF must start with %PDF");
            assert!(data.len() > 64);
        }
        other => panic!("expected DocumentSaved with PDF, got {other:?}"),
    }
}
