use tw_edit::{Command, EditSession};
use tw_model::{RevisionType, Run};

fn first_run_id(session: &EditSession) -> tw_model::NodeId {
    session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id
}

#[test]
fn track_changes_off_does_not_attach_revision_on_insert() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = false;
    let run_id = first_run_id(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Plain".into(),
        })
        .unwrap();

    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert!(para.runs[0].revision.is_none());
}

#[test]
fn track_changes_on_marks_inserted_text() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    session.document.settings.author_name = "Alice".into();
    let run_id = first_run_id(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Tracked".into(),
        })
        .unwrap();

    let rev = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .revision
        .as_ref()
        .unwrap();
    assert_eq!(rev.revision_type, RevisionType::Insert);
    assert_eq!(rev.author, "Alice");
}

#[test]
fn track_changes_delete_keeps_text_and_marks_revision() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    session.document.settings.author_name = "Bob".into();
    let run_id = first_run_id(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Delete me".into(),
        })
        .unwrap();

    let run_id = first_run_id(&session);
    session
        .apply(Command::DeleteRange {
            run_id,
            start: 7,
            end: 9,
        })
        .unwrap();

    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert!(para.full_text().contains("me"));
    let deleted: Vec<&Run> = para
        .runs
        .iter()
        .filter(|r| {
            r.revision
                .as_ref()
                .is_some_and(|rev| rev.revision_type == RevisionType::Delete)
        })
        .collect();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].text(), "me");
}

#[test]
fn normalize_does_not_merge_runs_with_different_revisions() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    session.document.settings.author_name = "Carol".into();
    let run_id = first_run_id(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "ABCDE".into(),
        })
        .unwrap();

    let run_id = first_run_id(&session);
    session
        .apply(Command::DeleteRange {
            run_id,
            start: 1,
            end: 4,
        })
        .unwrap();

    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert!(para.runs.len() >= 2);
}

#[test]
fn track_changes_off_performs_real_delete() {
    let mut session = EditSession::new();
    let run_id = first_run_id(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Remove".into(),
        })
        .unwrap();

    let run_id = first_run_id(&session);
    session
        .apply(Command::DeleteRange {
            run_id,
            start: 0,
            end: 3,
        })
        .unwrap();

    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.full_text(), "ove");
}
