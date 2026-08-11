//! F03.S1 — core character formatting ribbon.

use tw_edit::{Command, EditSession};
use tw_model::CharFormat;

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

/// U-F03-S1-bold-merge-runs: adjacent runs with equivalent formatting merge.
#[test]
fn u_f03_s1_bold_merge_runs() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "abc".into(),
        })
        .unwrap();

    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 1,
            end: 2,
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let mid_run = session.document.paragraph_at(0, 0).unwrap().runs[1].id;
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().runs.len(),
        3,
        "bold middle char should split into three runs"
    );

    session
        .apply(Command::SetCharFormat {
            run_id: mid_run,
            start: 0,
            end: 1,
            format: CharFormat {
                bold: Some(false),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.runs.len(), 1, "same-format adjacent runs should merge");
    assert_eq!(para.full_text(), "abc");
    assert_ne!(para.runs[0].format.bold, Some(true));
}

/// Collapsed caret at run end applies font size to visible text (ribbon path).
#[test]
fn u_f03_s1_font_size_at_run_end_via_collapsed_caret() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello".into(),
        })
        .unwrap();

    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 5,
            end: usize::MAX,
            format: CharFormat {
                font_size: Some(24.0),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.full_text(), "Hello");
    assert!(
        para.runs
            .iter()
            .all(|run| run.format.font_size == Some(24.0)),
        "font size should apply when caret is at run end"
    );
}
