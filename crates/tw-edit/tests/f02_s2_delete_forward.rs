//! F02.S2 — forward delete at caret.

use tw_edit::{Command, EditSession};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

/// U-F02-S2-delete-forward: forward delete at caret removes the next grapheme; undo restores.
#[test]
fn u_f02_s2_delete_forward() {
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
        .apply(Command::DeleteRange {
            run_id,
            start: 1,
            end: 2,
        })
        .unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "ac"
    );

    session.undo().unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "abc"
    );
}
