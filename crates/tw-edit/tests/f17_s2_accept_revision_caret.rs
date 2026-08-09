//! F17.S2 — accept/reject revision at caret and adjacent revision navigation.

use tw_edit::{
    accept_revision_at_caret, reject_revision_at_caret, Command, EditSession,
};
use tw_model::{revision_run_ids, Revision, RevisionType};

#[test]
fn accept_revision_at_caret_resolves_insert() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Tracked".into(),
        })
        .unwrap();

    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    let cmd = accept_revision_at_caret(&session.document, Some(run_id)).unwrap();
    session.apply(cmd).unwrap();

    let run = &session.document.paragraph_at(0, 0).unwrap().runs[0];
    assert_eq!(run.text(), "Tracked");
    assert!(run.revision.is_none());
}

#[test]
fn reject_revision_at_caret_removes_insert() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Drop".into(),
        })
        .unwrap();

    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    let cmd = reject_revision_at_caret(&session.document, Some(run_id)).unwrap();
    session.apply(cmd).unwrap();

    let text: String = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .map(|r| r.text())
        .collect();
    assert!(!text.contains("Drop"));
}

#[test]
fn accept_revision_at_caret_no_op_without_revision() {
    let session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    assert!(accept_revision_at_caret(&session.document, Some(run_id)).is_none());
}

#[test]
fn adjacent_revision_run_wraps_forward() {
    let mut session = EditSession::from_document(tw_model::Document::with_paragraph("AB"));
    if let tw_model::Block::Paragraph(para) = &mut session.document.sections[0].blocks[0] {
        let mut r0 = tw_model::Run::new_text("A");
        r0.revision = Some(Revision::insert("Alice"));
        let mut r1 = tw_model::Run::new_text("B");
        r1.revision = Some(Revision::delete("Bob"));
        para.runs = vec![r0, r1];
    }
    let ids = revision_run_ids(&session.document);
    assert_eq!(ids.len(), 2);

    let first = ids[0];
    let second = ids[1];
    assert_eq!(
        tw_model::adjacent_revision_run(&session.document, first, true),
        Some(second)
    );
    assert_eq!(
        tw_model::adjacent_revision_run(&session.document, second, true),
        Some(first)
    );
}

#[test]
fn adjacent_revision_run_from_plain_caret() {
    let mut session = EditSession::from_document(tw_model::Document::with_paragraph("XYZ"));
    if let tw_model::Block::Paragraph(para) = &mut session.document.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::insert("Author"));
        let mut mid = tw_model::Run::new_text("Y");
        let mut tail = tw_model::Run::new_text("Z");
        tail.revision = Some(Revision::delete("Author"));
        para.runs = vec![para.runs.remove(0), mid, tail];
    }
    let caret = session.document.paragraph_at(0, 0).unwrap().runs[1].id;
    let next = tw_model::adjacent_revision_run(&session.document, caret, true).unwrap();
    let prev = tw_model::adjacent_revision_run(&session.document, caret, false).unwrap();
    assert!(tw_model::revision_at_run(&session.document, next).is_some());
    assert!(tw_model::revision_at_run(&session.document, prev).is_some());
}

#[test]
fn caret_accept_is_undoable() {
    let mut session = EditSession::from_document(tw_model::Document::with_paragraph("Keep"));
    if let tw_model::Block::Paragraph(para) = &mut session.document.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::insert("Author"));
    }
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    let cmd = accept_revision_at_caret(&session.document, Some(run_id)).unwrap();
    session.apply(cmd).unwrap();
    assert!(session.document.paragraph_at(0, 0).unwrap().runs[0]
        .revision
        .is_none());

    session.undo().unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().runs[0]
            .revision
            .as_ref()
            .map(|r| r.revision_type),
        Some(RevisionType::Insert)
    );
}
