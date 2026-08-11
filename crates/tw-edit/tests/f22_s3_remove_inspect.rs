//! F22.S3 — remove Document Inspector findings.

use tw_edit::{Command, EditSession};
use tw_model::{CharFormat, InspectCategory, Run, RunContent, inspect_document};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f22_s3_remove_comments() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertComment {
            run_id,
            offset: 0,
            body_text: "note".into(),
        })
        .unwrap();
    assert!(!session.document.comments.is_empty());

    session
        .apply(Command::RemoveInspectFindings {
            comments: true,
            metadata: false,
            hidden_text: false,
        })
        .unwrap();

    assert!(session.document.comments.is_empty());
    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(!para
        .runs
        .iter()
        .any(|r| matches!(r.content, RunContent::CommentRef(_))));
    assert!(!inspect_document(&session.document)
        .iter()
        .any(|f| f.category == InspectCategory::Comments));
}

#[test]
fn u_f22_s3_remove_metadata() {
    let mut session = EditSession::new();
    session.document.properties.title = Some("T".into());
    session.document.properties.author = Some("A".into());

    session
        .apply(Command::RemoveInspectFindings {
            comments: false,
            metadata: true,
            hidden_text: false,
        })
        .unwrap();

    assert!(session.document.properties.title.is_none());
    assert!(session.document.properties.author.is_none());
}

#[test]
fn u_f22_s3_remove_hidden_text() {
    let mut session = EditSession::new();
    let mut hidden = Run::new_text("vanish");
    hidden.format = CharFormat {
        hidden: Some(true),
        ..Default::default()
    };
    session.document.sections[0].blocks[0]
        .paragraph_mut()
        .unwrap()
        .runs
        .push(hidden);

    session
        .apply(Command::RemoveInspectFindings {
            comments: false,
            metadata: false,
            hidden_text: true,
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(!para.runs.iter().any(|r| r.format.hidden == Some(true)));
    assert!(!inspect_document(&session.document)
        .iter()
        .any(|f| f.category == InspectCategory::HiddenText));
}
