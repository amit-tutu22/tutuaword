//! Stress: repeated comment insert and DOCX round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};

fn tail_pos(session: &EditSession) -> (tw_model::NodeId, usize) {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let run_id = para.runs.last().unwrap().id;
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    (run_id, offset)
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f17_s3_comment_insert_churn() {
    let mut session = EditSession::new();
    for i in 0..50 {
        let (run_id, offset) = tail_pos(&session);
        session
            .apply(Command::InsertComment {
                run_id,
                offset,
                body_text: format!("Comment {i}"),
            })
            .unwrap();
    }

    assert_eq!(session.document.comments.len(), 50);
    assert_eq!(
        session
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .runs
            .iter()
            .filter(|run| matches!(run.content, tw_model::RunContent::CommentRef(_)))
            .count(),
        50
    );
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f17_s3_comment_docx_roundtrip() {
    let mut session = EditSession::new();
    for i in 0..20 {
        let (run_id, offset) = tail_pos(&session);
        session
            .apply(Command::InsertComment {
                run_id,
                offset,
                body_text: format!("Note {i}"),
            })
            .unwrap();
    }

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();
    assert_eq!(imported.document.comments.len(), 20);
}
