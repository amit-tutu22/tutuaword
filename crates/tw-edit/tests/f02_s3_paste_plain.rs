//! F02.S3 — plain paste at caret merges runs.

use tw_edit::{paste, Command, EditSession};
use tw_model::CharFormat;

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

/// U-F02-S3-paste-plain-merges-runs: plain paste inserts at caret and merges adjacent runs.
#[test]
fn u_f02_s3_paste_plain_merges_runs() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "hello".into(),
        })
        .unwrap();

    paste::paste_plain_at(&mut session, run_id, 5, " world").unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.runs.len(), 1);
    assert_eq!(para.full_text(), "hello world");
}

#[test]
fn u_f02_s3_paste_formatted_applies_bold() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    paste::paste_inline_segments_at(
        &mut session,
        run_id,
        0,
        &[paste::PasteSegment {
            text: "bold".into(),
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
        }],
    )
    .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(para.runs.iter().any(|r| r.format.bold == Some(true)));
}
