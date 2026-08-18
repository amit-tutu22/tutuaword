//! F05.S5 — Ctrl+Up / Ctrl+Down targets and Alt+Shift+Up / Down block moves
//! as the editor sees them: through the bridge session.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::NodeId;

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

/// Three paragraphs — alpha / beta / gamma — with their first run ids.
fn three_paragraph_session() -> (Session, Vec<NodeId>) {
    let session = Session::new();
    wait_startup(&session);

    let first_run = session.document().paragraph_at(0, 0).unwrap().runs[0].id;
    let id = session
        .apply(Command::InsertText {
            run_id: first_run,
            offset: 0,
            text: "alpha".into(),
        })
        .expect("insert alpha");
    wait_edit(&session, id);

    for (index, text) in ["beta", "gamma"].iter().enumerate() {
        let after = session
            .document()
            .paragraph_at(0, index)
            .expect("paragraph")
            .id;
        let id = session
            .apply(Command::InsertParagraph { after_id: after })
            .expect("insert paragraph");
        wait_edit(&session, id);
        let run_id = session
            .document()
            .paragraph_at(0, index + 1)
            .expect("new paragraph")
            .runs[0]
            .id;
        let id = session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: (*text).into(),
            })
            .expect("insert text");
        wait_edit(&session, id);
    }

    let runs = (0..3)
        .map(|index| {
            session
                .document()
                .paragraph_at(0, index)
                .expect("paragraph")
                .runs[0]
                .id
        })
        .collect();
    (session, runs)
}

fn nav(session: &Session, run_id: NodeId) -> serde_json::Value {
    let json = session.paragraph_nav_json(run_id).expect("nav json");
    serde_json::from_str(&json).expect("valid json")
}

fn body_text(session: &Session) -> Vec<String> {
    session.document().sections[0]
        .blocks
        .iter()
        .filter_map(|block| block.paragraph())
        .map(|para| para.full_text())
        .collect()
}

#[test]
fn u_f05_s5_paragraph_nav_reports_neighbour_run_ids() {
    let (session, runs) = three_paragraph_session();

    let middle = nav(&session, runs[1]);
    assert_eq!(middle["start"]["run"], serde_json::json!(runs[1].to_string()));
    assert_eq!(middle["start"]["offset"], serde_json::json!(0));
    assert_eq!(middle["prev"]["run"], serde_json::json!(runs[0].to_string()));
    assert_eq!(middle["next"]["run"], serde_json::json!(runs[2].to_string()));
}

#[test]
fn u_f05_s5_paragraph_nav_is_null_at_the_document_edges() {
    let (session, runs) = three_paragraph_session();

    assert!(nav(&session, runs[0])["prev"].is_null());
    assert!(nav(&session, runs[2])["next"].is_null());
}

#[test]
fn u_f05_s5_session_move_block_reorders_and_undoes() {
    let (session, runs) = three_paragraph_session();
    assert_eq!(body_text(&session), ["alpha", "beta", "gamma"]);

    let id = session.move_block_at(Some(runs[2]), -1).expect("move up");
    wait_edit(&session, id);
    assert_eq!(body_text(&session), ["alpha", "gamma", "beta"]);

    let id = session.undo().expect("undo");
    wait_edit(&session, id);
    assert_eq!(body_text(&session), ["alpha", "beta", "gamma"]);
}

#[test]
fn u_f05_s5_session_move_block_stops_at_the_first_paragraph() {
    let (session, runs) = three_paragraph_session();
    assert!(session.move_block_at(Some(runs[0]), -1).is_none());
    assert_eq!(body_text(&session), ["alpha", "beta", "gamma"]);
}
