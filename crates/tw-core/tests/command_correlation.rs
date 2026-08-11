//! R0.2: concurrent commands receive correlated events, not cross-talk.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;

#[test]
fn concurrent_save_and_insert_get_correct_events() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let run_id = session
        .get_display_list_bytes()
        .pages
        .first()
        .and_then(|_| session.hit_test(0, 72.0, 83.0))
        .map(|hit| hit.run_id)
        .expect("empty document is editable");

    let save_id = session.save().expect("save enqueued");
    let insert_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "correlated".into(),
        })
        .expect("insert enqueued");

    assert_ne!(save_id, insert_id);

    let save_outcome = session.wait_for_response(save_id, Duration::from_secs(10));
    let insert_outcome = session.wait_for_response(insert_id, Duration::from_secs(10));

    match save_outcome {
        WaitOutcome::Matched(BridgeEvent::DocumentSaved { request_id, .. }) => {
            assert_eq!(request_id, save_id);
        }
        other => panic!("save expected DocumentSaved, got {other:?}"),
    }

    match insert_outcome {
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { request_id, .. }) => {
            assert_eq!(request_id, insert_id);
        }
        other => panic!("insert expected DisplayListReady, got {other:?}"),
    }

    assert!(session
        .get_display_list_bytes()
        .document_text
        .contains("correlated"));
}
