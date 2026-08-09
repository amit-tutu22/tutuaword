//! F17.S3 — insert comment anchor and thread via InsertComment.

use tw_edit::{Command, EditSession};
use tw_model::RunContent;

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f17_s3_insert_comment_creates_ref_and_thread() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertComment {
            run_id,
            offset: 0,
            body_text: "Review this sentence.".into(),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(
        para.runs.iter().any(|run| matches!(
            &run.content,
            RunContent::CommentRef(c) if c.display_number == Some(1)
        )),
        "expected comment ref with display number 1"
    );
    assert_eq!(session.document.comments.len(), 1);
    assert_eq!(session.document.comments[0].comment_id, 0);
    assert_eq!(
        session.document.comments[0].messages[0]
            .body
            .first()
            .and_then(|b| b.paragraph())
            .map(|p| p.full_text()),
        Some("Review this sentence.".to_string())
    );
}

#[test]
fn u_f17_s3_insert_comment_ref_has_highlight() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertComment {
            run_id,
            offset: 0,
            body_text: String::new(),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    let comment_run = para
        .runs
        .iter()
        .find(|run| matches!(run.content, RunContent::CommentRef(_)))
        .expect("comment ref run");
    assert!(comment_run.format.highlight.is_some());
}

#[test]
fn u_f17_s3_multiple_comments_renumber() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertComment {
            run_id,
            offset: 0,
            body_text: "First".into(),
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
            body_text: "Second".into(),
        })
        .unwrap();

    assert_eq!(session.document.comments.len(), 2);
    let numbers: Vec<u32> = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter_map(|run| match &run.content {
            RunContent::CommentRef(c) => c.display_number,
            _ => None,
        })
        .collect();
    assert_eq!(numbers, vec![1, 2]);
}
