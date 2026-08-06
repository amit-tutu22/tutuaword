//! Track-change accept/reject (risk-mitigation TC ladder step c).

use tw_edit::{Command, EditSession};
use tw_model::{Revision, RevisionType};

#[test]
fn accept_insert_keeps_text_and_clears_revision() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello".into(),
        })
        .unwrap();

    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    session
        .apply(Command::AcceptRevision { run_id })
        .unwrap();

    let run = &session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0];
    assert_eq!(run.text(), "Hello");
    assert!(run.revision.is_none());
}

#[test]
fn reject_insert_removes_text() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Gone".into(),
        })
        .unwrap();

    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    session
        .apply(Command::RejectRevision { run_id })
        .unwrap();

    let text: String = session
        .document
        .sections[0]
        .blocks[0]
        .paragraph()
        .unwrap()
        .runs
        .iter()
        .map(|r| r.text())
        .collect();
    assert!(!text.contains("Gone"));
}

#[test]
fn accept_delete_removes_marked_text() {
    let mut session = EditSession::from_document(tw_model::Document::with_paragraph("Keep DeleteMe"));
    if let tw_model::Block::Paragraph(para) = &mut session.document.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::delete("Alice"));
    }
    session.resync_buffer();

    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    session
        .apply(Command::AcceptRevision { run_id })
        .unwrap();

    let text: String = session
        .document
        .sections[0]
        .blocks[0]
        .paragraph()
        .unwrap()
        .runs
        .iter()
        .map(|r| r.text())
        .collect();
    assert!(text.is_empty() || !text.contains("DeleteMe"));
}

#[test]
fn reject_delete_keeps_text_without_revision() {
    let mut session = EditSession::from_document(tw_model::Document::with_paragraph("Survives"));
    if let tw_model::Block::Paragraph(para) = &mut session.document.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::delete("Bob"));
    }
    session.resync_buffer();

    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    session
        .apply(Command::RejectRevision { run_id })
        .unwrap();

    let run = &session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0];
    assert_eq!(run.text(), "Survives");
    assert!(run.revision.is_none());
}

#[test]
fn accept_all_clears_mixed_revisions() {
    let mut session = EditSession::from_document(tw_model::Document::with_paragraph("A"));
    if let tw_model::Block::Paragraph(para) = &mut session.document.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::insert("A"));
        let mut del = tw_model::Run::new_text("B");
        del.revision = Some(Revision::delete("A"));
        para.runs.push(del);
    }
    session.resync_buffer();

    session.apply(Command::AcceptAllRevisions).unwrap();

    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert!(para.runs.iter().all(|r| r.revision.is_none()));
    assert!(para.runs.iter().any(|r| r.text() == "A"));
    assert!(!para.runs.iter().any(|r| r.text() == "B"));
}

#[test]
fn accept_revision_is_undoable() {
    let mut session = EditSession::from_document(tw_model::Document::with_paragraph("X"));
    if let tw_model::Block::Paragraph(para) = &mut session.document.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::insert("Author"));
    }
    session.resync_buffer();
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    session
        .apply(Command::AcceptRevision { run_id })
        .unwrap();
    assert!(session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .revision
        .is_none());

    session.undo().unwrap();
    assert_eq!(
        session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .revision
            .as_ref()
            .map(|r| r.revision_type),
        Some(RevisionType::Insert)
    );
}
