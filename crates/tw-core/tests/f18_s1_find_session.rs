//! F18.S1 — session find matches API.

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
fn u_f18_s1_session_find_matches_case_sensitive() {
    let session = Session::new();
    wait_startup(&session);

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    let edit_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Find find FIND".into(),
        })
        .expect("insert text");
    assert!(matches!(
        session.wait_for_response(edit_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let sensitive = session.find_matches("find", true, false, false, None);
    assert_eq!(sensitive.len(), 1);
    let insensitive = session.find_matches("find", false, false, false, None);
    assert_eq!(insensitive.len(), 3);
}
