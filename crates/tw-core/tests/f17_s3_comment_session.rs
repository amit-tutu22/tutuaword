//! F17.S3 session API for comment insert.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};

fn wait_startup(session: &Session) {
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));
}

fn wait_edit(session: &Session, request_id: u64) {
    assert!(matches!(
        session.wait_for_response(request_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));
}

#[test]
fn session_insert_comment_at_caret() {
    let session = Session::new();
    wait_startup(&session);

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    let request_id = session
        .insert_comment_at(run_id, 0, "Session comment")
        .expect("insert comment");
    wait_edit(&session, request_id);

    let doc = session.document();
    assert_eq!(doc.comments.len(), 1);
    assert!(doc.paragraph_at(0, 0).unwrap().runs.iter().any(|run| {
        matches!(run.content, tw_model::RunContent::CommentRef(_))
    }));
}
