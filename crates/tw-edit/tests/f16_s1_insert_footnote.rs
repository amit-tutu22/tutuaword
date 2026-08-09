//! F16.S1 — insert footnote reference and body via InsertFootnote.

use tw_edit::{Command, EditSession};
use tw_model::RunContent;

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f16_s1_insert_footnote_creates_ref_and_body() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertFootnote {
            run_id,
            offset: 0,
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(
        para.runs.iter().any(|run| matches!(
            &run.content,
            RunContent::FootnoteRef(note) if note.display_number == Some(1)
        )),
        "expected footnote ref with display number 1"
    );
    assert_eq!(session.document.footnotes.len(), 1);
    assert_eq!(session.document.footnotes[0].id, 1);
    assert!(
        session.document.footnotes[0]
            .blocks
            .iter()
            .any(|b| b.paragraph().is_some())
    );
}

#[test]
fn u_f16_s1_insert_footnote_ref_is_superscript() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertFootnote {
            run_id,
            offset: 0,
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    let footnote_run = para
        .runs
        .iter()
        .find(|run| matches!(run.content, RunContent::FootnoteRef(_)))
        .expect("footnote ref run");
    assert_eq!(footnote_run.format.superscript, Some(true));
}

#[test]
fn u_f16_s1_insert_footnote_after_text() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello".into(),
        })
        .unwrap();

    let offset = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .full_text()
        .chars()
        .count();
    session
        .apply(Command::InsertFootnote {
            run_id,
            offset,
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(para
        .runs
        .iter()
        .any(|r| matches!(r.content, RunContent::FootnoteRef(_))));
}

#[test]
fn u_f16_s1_multiple_footnotes_renumber() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertFootnote {
            run_id,
            offset: 0,
        })
        .unwrap();
    let offset = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .full_text()
        .chars()
        .count();
    session
        .apply(Command::InsertFootnote {
            run_id,
            offset,
        })
        .unwrap();

    assert_eq!(session.document.footnotes.len(), 2);
    let numbers: Vec<u32> = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter_map(|run| match &run.content {
            RunContent::FootnoteRef(note) => note.display_number,
            _ => None,
        })
        .collect();
    assert_eq!(numbers, vec![1, 2]);
}
