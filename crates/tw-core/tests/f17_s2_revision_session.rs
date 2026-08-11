//! F17.S2 session API for caret revision resolve and navigation.

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

fn wait_edit(session: &Session, request_id: u64) {
    assert!(matches!(
        session.wait_for_response(request_id, Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));
}

fn tail_run(session: &Session) -> tw_model::NodeId {
    let doc = session.document();
    let para = doc.paragraph_at(0, 0).unwrap();
    para.runs.last().unwrap().id
}

#[test]
fn session_accept_revision_at_caret() {
    let session = Session::new();
    wait_startup(&session);
    session.set_track_changes(true).expect("track changes");

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    wait_edit(
        &session,
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Tracked".into(),
            })
            .expect("insert"),
    );

    let run_id = tail_run(&session);
    let request_id = session.accept_revision_at(Some(run_id)).expect("enqueued");
    wait_edit(&session, request_id);

    let doc = session.document();
    let run = &doc.paragraph_at(0, 0).unwrap().runs[0];
    assert_eq!(run.text(), "Tracked");
    assert!(run.revision.is_none());
}

#[test]
fn session_reject_revision_at_caret() {
    let session = Session::new();
    wait_startup(&session);
    session.set_track_changes(true).expect("track changes");

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    wait_edit(
        &session,
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Drop".into(),
            })
            .expect("insert"),
    );

    let run_id = tail_run(&session);
    let request_id = session.reject_revision_at(Some(run_id)).expect("enqueued");
    wait_edit(&session, request_id);

    let text: String = session
        .document()
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .map(|r| r.text())
        .collect();
    assert!(!text.contains("Drop"));
}

#[test]
fn session_adjacent_revision_run_navigation() {
    let session = Session::new();
    wait_startup(&session);
    session.set_track_changes(true).expect("track changes");

    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    wait_edit(
        &session,
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "First ".into(),
            })
            .expect("insert first"),
    );
    let run_id = tail_run(&session);
    wait_edit(
        &session,
        session
            .apply(Command::InsertText {
                run_id,
                offset: tw_edit::run_char_len_by_id(&session.document(), run_id),
                text: "Second".into(),
            })
            .expect("insert second"),
    );

    let ids = tw_model::revision_run_ids(&session.document());
    assert!(!ids.is_empty(), "expected tracked revisions");
    let first = ids[0];
    let next = session
        .adjacent_revision_run(Some(first), true)
        .expect("next revision");
    assert!(tw_model::revision_at_run(&session.document(), next).is_some());
    if ids.len() >= 2 {
        assert_eq!(next, ids[1]);
        assert_eq!(
            session.adjacent_revision_run(Some(ids[1]), false),
            Some(ids[0])
        );
    } else {
        assert_eq!(next, first);
    }
}

#[test]
fn session_accept_revision_at_returns_none_without_marker() {
    let session = Session::new();
    wait_startup(&session);
    let run_id = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    assert!(session.accept_revision_at(Some(run_id)).is_none());
}
