//! F19.S3 — InsertHyperlink command.

use tw_edit::{Command, EditSession};
use tw_model::RunContent;

#[test]
fn u_f19_s3_insert_hyperlink_creates_run() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertHyperlink {
            run_id,
            offset: 0,
            url: "https://example.com".into(),
            text: "Example".into(),
            tooltip: Some("tip".into()),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    let found = para.runs.iter().any(|run| {
        matches!(
            &run.content,
            RunContent::Hyperlink { target, text }
                if text == "Example"
                    && target.url == "https://example.com"
                    && target.tooltip.as_deref() == Some("tip")
        )
    });
    assert!(found, "hyperlink run not inserted");
}

#[test]
fn u_f19_s3_edit_hyperlink_updates_target() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertHyperlink {
            run_id,
            offset: 0,
            url: "https://old.example".into(),
            text: "Old".into(),
            tooltip: None,
        })
        .unwrap();
    let link_id = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .find(|r| matches!(r.content, RunContent::Hyperlink { .. }))
        .unwrap()
        .id;

    session
        .apply(Command::InsertHyperlink {
            run_id: link_id,
            offset: 0,
            url: "https://new.example".into(),
            text: "New".into(),
            tooltip: Some("updated".into()),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    let links: Vec<_> = para
        .runs
        .iter()
        .filter(|r| matches!(r.content, RunContent::Hyperlink { .. }))
        .collect();
    assert_eq!(links.len(), 1, "edit must update in place, not duplicate");
    match &links[0].content {
        RunContent::Hyperlink { target, text } => {
            assert_eq!(text, "New");
            assert_eq!(target.url, "https://new.example");
            assert_eq!(target.tooltip.as_deref(), Some("updated"));
        }
        _ => unreachable!(),
    }
}
