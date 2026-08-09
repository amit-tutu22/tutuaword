//! F18.S3 — SyncSession regex find matches.

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
fn u_f18_s3_sync_session_regex_find_matches() {
    let session = Session::new();
    wait_startup(&session);

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    let edit_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "ticket A12 and ticket B34".into(),
        })
        .expect("insert text");
    assert!(matches!(
        session.wait_for_response(edit_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let matches = session.find_matches(r"ticket [A-Z]\d+", true, true, false, None);
    assert_eq!(matches.len(), 2);
}
