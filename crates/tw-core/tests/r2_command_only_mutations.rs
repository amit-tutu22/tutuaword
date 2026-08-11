//! R2.3: all document edits flow through `BridgeCommand::ApplyEdit { command }`.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, NumberingRef};

const FORBIDDEN_BRIDGE_SHORTCUTS: &[&str] = &[
    "BridgeCommand::ApplyHeading",
    "BridgeCommand::InsertTable",
    "BridgeCommand::ApplyBullet",
    "BridgeCommand::ApplyNormal",
    "BridgeCommand::InsertImage",
    "BridgeCommand::InsertPageBreak",
    "BridgeCommand::AcceptAll",
    "BridgeCommand::RejectAll",
];

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
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { request_id: id, .. })
            if id == request_id
    ));
}

fn first_run_id(session: &Session) -> tw_model::NodeId {
    session
        .hit_test(0, 72.0, 83.0)
        .map(|hit| hit.run_id)
        .expect("empty document is editable")
}

fn paragraph_id(session: &Session) -> tw_model::NodeId {
    session.document().sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id
}

fn block_count(session: &Session) -> usize {
    session.document().sections[0].blocks.len()
}

fn paragraph_style_id(session: &Session) -> Option<tw_model::StyleId> {
    session
        .document()
        .paragraph_at(0, 0)
        .unwrap()
        .style_id
}

fn paragraph_numbering_id(session: &Session) -> Option<u32> {
    session
        .document()
        .paragraph_at(0, 0)
        .unwrap()
        .format
        .numbering
        .map(|n| n.numbering_id)
}

#[test]
fn bridge_command_has_no_edit_shortcuts() {
    let sources = [
        include_str!("../src/worker.rs"),
        include_str!("../src/session.rs"),
    ];
    for source in sources {
        for pattern in FORBIDDEN_BRIDGE_SHORTCUTS {
            assert!(
                !source.contains(pattern),
                "found forbidden shortcut {pattern} in tw-core source"
            );
        }
    }
}

#[test]
fn apply_heading1_via_apply_edit_and_undo() {
    let session = Session::new();
    wait_startup(&session);

    assert!(paragraph_style_id(&session).is_none());

    let request_id = session.apply_heading1().expect("heading1 enqueued");
    wait_edit(&session, request_id);
    assert!(paragraph_style_id(&session).is_some());

    let undo_id = session.undo().expect("undo enqueued");
    wait_edit(&session, undo_id);
    assert!(paragraph_style_id(&session).is_none());
}

#[test]
fn apply_bullet_list_via_apply_edit_and_undo() {
    let session = Session::new();
    wait_startup(&session);

    let request_id = session.apply_bullet_list().expect("bullet list enqueued");
    wait_edit(&session, request_id);
    assert_eq!(paragraph_numbering_id(&session), Some(1));

    let undo_id = session.undo().expect("undo enqueued");
    wait_edit(&session, undo_id);
    assert_eq!(paragraph_numbering_id(&session), None);
}

#[test]
fn insert_table_via_apply_edit_and_undo() {
    let session = Session::new();
    wait_startup(&session);

    assert_eq!(block_count(&session), 1);

    let request_id = session.insert_table(2, 2).expect("insert table enqueued");
    wait_edit(&session, request_id);
    assert_eq!(block_count(&session), 2);
    assert!(matches!(
        session.document().sections[0].blocks[1],
        Block::Table(_)
    ));

    let undo_id = session.undo().expect("undo enqueued");
    wait_edit(&session, undo_id);
    assert_eq!(block_count(&session), 1);
}

#[test]
fn accept_all_revisions_via_apply_edit() {
    let session = Session::new();
    wait_startup(&session);

    let run_id = first_run_id(&session);
    let insert_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "tracked".into(),
        })
        .expect("insert enqueued");
    wait_edit(&session, insert_id);

    let track_id = session.set_track_changes(true).expect("track changes");
    let _ = track_id;

    let insert2_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "x".into(),
        })
        .expect("tracked insert enqueued");
    wait_edit(&session, insert2_id);

    let accept_id = session.accept_all_revisions().expect("accept all enqueued");
    wait_edit(&session, accept_id);

    let has_revision = session
        .document()
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .any(|r| r.revision.is_some());
    assert!(!has_revision);
}

#[test]
fn explicit_apply_edit_heading1_matches_session_helper() {
    let session = Session::new();
    wait_startup(&session);

    let para_id = paragraph_id(&session);
    let request_id = session
        .apply(Command::ApplyParagraphStyle {
            paragraph_id: para_id,
            style_name: "Heading 1".into(),
        })
        .expect("apply edit enqueued");
    wait_edit(&session, request_id);

    let doc = session.document();
    let para = doc.paragraph_at(0, 0).unwrap();
    assert!(para.style_id.is_some());
    assert_eq!(para.runs[0].format.bold, Some(true));
}

#[test]
fn explicit_apply_edit_numbered_list() {
    let session = Session::new();
    wait_startup(&session);

    let para_id = paragraph_id(&session);
    let request_id = session
        .apply(Command::SetNumbering {
            paragraph_id: para_id,
            numbering: Some(NumberingRef {
                numbering_id: 2,
                level: 0,
            }),
        })
        .expect("numbered list enqueued");
    wait_edit(&session, request_id);
    assert_eq!(paragraph_numbering_id(&session), Some(2));
}
