//! F17.S4 — restrict editing blocks edits at session worker.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;

fn wait_startup(session: &Session) {
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));
}

#[test]
fn u_f17_s4_read_only_blocks_apply_edit() {
    let session = Session::new();
    wait_startup(&session);

    let lock_id = session.set_read_only(true).expect("set read-only");
    assert!(matches!(
        session.wait_for_response(lock_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));
    assert!(session.document().settings.read_only);

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    let edit_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "blocked".into(),
        })
        .expect("enqueue edit");
    match session.wait_for_response(edit_id, Duration::from_secs(5)) {
        WaitOutcome::Matched(BridgeEvent::Error { message, .. }) => {
            assert!(message.contains("read-only"), "unexpected error: {message}");
        }
        other => panic!("expected read-only error, got {other:?}"),
    }

    let unlock_id = session.set_read_only(false).expect("clear read-only");
    assert!(matches!(
        session.wait_for_response(unlock_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let edit_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "allowed".into(),
        })
        .expect("enqueue edit after unlock");
    assert!(matches!(
        session.wait_for_response(edit_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));
}
