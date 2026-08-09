//! F17.S1 — session spell check returns document misspellings.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;

#[test]
fn u_f17_s1_session_spell_check_misspellings() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let run_id = session
        .document()
        .paragraph_at(0, 0)
        .unwrap()
        .runs[0]
        .id;
    let edit_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Teh document has a mispelling.".into(),
        })
        .expect("insert text");
    assert!(matches!(
        session.wait_for_response(edit_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let request = session.spell_check().expect("spell check request");
    match session.wait_for_response(request, Duration::from_secs(5)) {
        WaitOutcome::Matched(BridgeEvent::SpellCheckResult { misspellings, .. }) => {
            assert!(
                misspellings.iter().any(|w| w == "Teh"),
                "expected Teh in {misspellings:?}"
            );
            assert!(
                misspellings.iter().any(|w| w == "mispelling"),
                "expected mispelling in {misspellings:?}"
            );
        }
        other => panic!("unexpected spell check outcome: {other:?}"),
    }
}
