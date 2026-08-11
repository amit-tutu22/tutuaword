//! F17.S4 — session grammar check returns document issues.

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
fn u_f17_s4_session_grammar_check_flags_issues() {
    let session = Session::new();
    wait_startup(&session);

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    let edit_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "I could of done alot  better.".into(),
        })
        .expect("insert text");
    assert!(matches!(
        session.wait_for_response(edit_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let request = session.grammar_check().expect("grammar check request");
    match session.wait_for_response(request, Duration::from_secs(5)) {
        WaitOutcome::Matched(BridgeEvent::GrammarCheckResult { issues, .. }) => {
            assert!(
                issues.iter().any(|m| m.contains("could have")),
                "expected could-have issue in {issues:?}"
            );
            assert!(
                issues.iter().any(|m| m.contains("a lot") || m.contains("extra space")),
                "expected alot or double-space issue in {issues:?}"
            );
        }
        other => panic!("unexpected grammar check outcome: {other:?}"),
    }
}

#[test]
fn u_f17_s4_session_compare_with_text() {
    let session = Session::new();
    wait_startup(&session);

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    let edit_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Version one.".into(),
        })
        .expect("insert text");
    assert!(matches!(
        session.wait_for_response(edit_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let summary = session.compare_with_text("Version two.");
    assert_eq!(summary.insertion_count, 1);
    assert_eq!(summary.deletion_count, 1);
}
