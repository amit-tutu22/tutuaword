//! F25.S4 — session export with N-up / booklet sheet options.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_pdf::PrintLayoutOptions;

#[test]
fn i_f25_s4_export_pdf_for_print_nup() {
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
            text: "N-up print sample".into(),
        })
        .expect("insert");

    let request_id = session
        .export_pdf_for_print(PrintLayoutOptions::with_nup(2), None)
        .expect("export enqueued");
    match session.wait_for_response(request_id, Duration::from_secs(30)) {
        WaitOutcome::Matched(BridgeEvent::DocumentSaved { request_id: rid, data }) => {
            assert_eq!(rid, request_id);
            assert!(data.starts_with(b"%PDF"));
        }
        other => panic!("expected DocumentSaved with PDF, got {other:?}"),
    }
}

#[test]
fn i_f25_s4_export_pdf_for_print_booklet() {
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
            text: "Booklet print sample".into(),
        })
        .expect("insert");

    let request_id = session
        .export_pdf_for_print(PrintLayoutOptions::with_booklet(), None)
        .expect("export enqueued");
    match session.wait_for_response(request_id, Duration::from_secs(30)) {
        WaitOutcome::Matched(BridgeEvent::DocumentSaved { request_id: rid, data }) => {
            assert_eq!(rid, request_id);
            assert!(data.starts_with(b"%PDF"));
            assert!(data.len() > 64);
        }
        other => panic!("expected DocumentSaved with PDF, got {other:?}"),
    }
}
