//! F15.S3 — re-insert recently used symbols via InsertText.

use tw_edit::{Command, EditSession};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

/// Simulates picking the same few symbols repeatedly from a recents list.
const RECENT_ROTATION: &[&str] = &["©", "€", "±", "α", "😀", "👍"];

#[test]
fn u_f15_s3_reinsert_recent_symbol_at_end() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "©".into(),
        })
        .unwrap();

    let offset = session.document.paragraph_at(0, 0).unwrap().full_text().chars().count();
    session
        .apply(Command::InsertText {
            run_id,
            offset,
            text: "©".into(),
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "©©"
    );
}

#[test]
fn u_f15_s3_recent_rotation_inserts_all() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    let mut offset = 0usize;
    for sym in RECENT_ROTATION {
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: sym.to_string(),
            })
            .unwrap();
        offset += sym.chars().count();
    }

    let text = session.document.paragraph_at(0, 0).unwrap().full_text();
    for sym in RECENT_ROTATION {
        assert!(text.contains(sym), "missing {sym} in {text:?}");
    }
}

#[test]
fn u_f15_s3_recent_symbol_burst_single_run() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    for _ in 0..6 {
        let offset = session
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .full_text()
            .chars()
            .count();
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: "©".into(),
            })
            .unwrap();
    }

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.full_text(), "©©©©©©");
    assert_eq!(para.runs.len(), 1);
}

#[test]
fn u_f15_s3_recent_emoji_reinsert() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Done ".into(),
        })
        .unwrap();

    let offset = session.document.paragraph_at(0, 0).unwrap().full_text().chars().count();
    session
        .apply(Command::InsertText {
            run_id,
            offset,
            text: "👍".into(),
        })
        .unwrap();

    let offset = session.document.paragraph_at(0, 0).unwrap().full_text().chars().count();
    session
        .apply(Command::InsertText {
            run_id,
            offset,
            text: "👍".into(),
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "Done 👍👍"
    );
}
