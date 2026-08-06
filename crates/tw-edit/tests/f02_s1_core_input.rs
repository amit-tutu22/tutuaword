//! F02.S1 — core glyph input path: insert, tab, undo/redo.

use tw_edit::{Command, EditSession};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f02_s1_insert_undo_redo() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    for _ in 0..100 {
        let offset = session.document.paragraph_at(0, 0).unwrap().full_text().chars().count();
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: "x".into(),
            })
            .unwrap();
    }

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "x".repeat(100)
    );

    for _ in 0..100 {
        assert!(session.can_undo());
        session.undo().unwrap();
    }
    assert_eq!(session.document.paragraph_at(0, 0).unwrap().full_text(), "");
    assert!(!session.can_undo());

    for i in 0..100 {
        assert!(session.can_redo());
        session.redo().unwrap();
        assert_eq!(
            session.document.paragraph_at(0, 0).unwrap().full_text(),
            "x".repeat(i + 1)
        );
    }
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "x".repeat(100)
    );
    assert!(!session.can_redo());
}

#[test]
fn tab_insert_stores_tab_character() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "\t".into(),
        })
        .unwrap();

    assert_eq!(session.document.paragraph_at(0, 0).unwrap().full_text(), "\t");
}
