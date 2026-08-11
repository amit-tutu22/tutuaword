//! Stress: repeated footnote insert and DOCX export round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::RunContent;

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

fn tail_insert_pos(session: &EditSession) -> (tw_model::NodeId, usize) {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let run_id = para.runs.last().unwrap().id;
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    (run_id, offset)
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_footnote_insert_churn() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Claims: ".into(),
        })
        .unwrap();

    for _ in 0..50 {
        let (run_id, offset) = tail_insert_pos(&session);
        session
            .apply(Command::InsertFootnote { run_id, offset })
            .unwrap();
    }

    assert_eq!(session.document.footnotes.len(), 50);
    let ref_count = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter(|run| matches!(run.content, RunContent::FootnoteRef(_)))
        .count();
    assert_eq!(ref_count, 50);
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_footnote_docx_round_trip() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    let _ = run_id;

    for i in 0..10 {
        let text = format!("Section {i} ");
        let (run_id, offset) = tail_insert_pos(&session);
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text,
            })
            .unwrap();
        let (run_id, offset) = tail_insert_pos(&session);
        session
            .apply(Command::InsertFootnote { run_id, offset })
            .unwrap();
    }

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();

    assert_eq!(imported.document.footnotes.len(), 10);
    assert!(
        imported
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .runs
            .iter()
            .filter(|run| matches!(run.content, RunContent::FootnoteRef(_)))
            .count()
            >= 10
    );
}
