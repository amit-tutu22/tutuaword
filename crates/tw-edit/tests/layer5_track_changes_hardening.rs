//! Layer 5 — track-changes accept/reject all hardening.

use tw_edit::{apply, Command, EditSession};

fn session_with_two_inserts() -> EditSession {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    apply(
        &mut session.document,
        Command::InsertText {
            run_id,
            offset: 0,
            text: "A".into(),
        },
    )
    .unwrap();
    apply(
        &mut session.document,
        Command::InsertText {
            run_id,
            offset: 1,
            text: "B".into(),
        },
    )
    .unwrap();
    session
}

#[test]
fn u_layer5_accept_all_revisions_clears_markup() {
    let mut session = session_with_two_inserts();
    apply(&mut session.document, Command::AcceptAllRevisions).unwrap();
    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.full_text(), "AB");
    assert!(
        para.runs.iter().all(|r| r.revision.is_none()),
        "accept all should clear revision marks"
    );
}

#[test]
fn u_layer5_reject_all_revisions_removes_insertions() {
    let mut session = session_with_two_inserts();
    apply(&mut session.document, Command::RejectAllRevisions).unwrap();
    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.full_text(), "");
}

#[test]
fn u_layer5_accept_all_is_undoable_via_edit_session() {
    let mut session = session_with_two_inserts();
    session.apply(Command::AcceptAllRevisions).unwrap();
    assert_eq!(
        session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "AB"
    );
    session.undo().expect("undo stack should accept accept-all");
    assert!(
        session
            .document
            .sections[0]
            .blocks[0]
            .paragraph()
            .unwrap()
            .runs
            .iter()
            .any(|r| r.revision.is_some()),
        "undo should restore pre-accept state with revision marks"
    );
}
