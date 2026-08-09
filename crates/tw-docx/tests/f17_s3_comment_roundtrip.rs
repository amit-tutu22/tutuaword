//! F17.S3 — comment DOCX export/import round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::RunContent;

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f17_s3_comment_docx_round_trip() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Draft".into(),
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
        .apply(Command::InsertComment {
            run_id,
            offset,
            body_text: "Please review.".into(),
        })
        .unwrap();

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();

    assert_eq!(imported.document.comments.len(), 1);
    assert_eq!(
        imported.document.comments[0].messages[0]
            .body
            .first()
            .and_then(|b| b.paragraph())
            .map(|p| p.full_text()),
        Some("Please review.".to_string())
    );
    assert!(
        imported
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .runs
            .iter()
            .any(|run| matches!(run.content, RunContent::CommentRef(_)))
    );
    assert!(imported
        .package
        .parts
        .contains_key("word/comments.xml"));
}
