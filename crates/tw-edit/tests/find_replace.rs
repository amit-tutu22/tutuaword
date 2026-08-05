//! Find/replace unit tests spanning runs and undo.

use tw_edit::{Command, DocPosition, DocRange, EditSession};

#[test]
fn find_replace_across_two_runs() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "cat dog cat".into(),
        })
        .unwrap();
    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 4,
            end: 7,
            format: tw_model::CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let runs = &session.document.paragraph_at(0, 0).unwrap().runs;
    assert!(runs.len() >= 2, "formatting should split the run");

    let first_id = runs[0].id;
    let last_id = runs.last().unwrap().id;
    let last_len = runs.last().unwrap().text().len();

    session
        .apply(Command::FindReplace {
            range: DocRange {
                start: DocPosition {
                    run_id: first_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id: last_id,
                    char_offset: last_len,
                },
            },
            find: "cat".into(),
            replace: "fish".into(),
            match_case: true,
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "fish dog fish"
    );
}

#[test]
fn find_replace_case_insensitive() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Foo foo FOO".into(),
        })
        .unwrap();
    session
        .apply(Command::FindReplace {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 11,
                },
            },
            find: "foo".into(),
            replace: "bar".into(),
            match_case: false,
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "bar bar bar"
    );
}

#[test]
fn find_replace_no_match_leaves_text_unchanged() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "unchanged".into(),
        })
        .unwrap();
    session
        .apply(Command::FindReplace {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 9,
                },
            },
            find: "missing".into(),
            replace: "nope".into(),
            match_case: true,
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "unchanged"
    );
}

#[test]
fn find_replace_undo_restores_original() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "one two one".into(),
        })
        .unwrap();
    session
        .apply(Command::FindReplace {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 11,
                },
            },
            find: "one".into(),
            replace: "1".into(),
            match_case: true,
        })
        .unwrap();
    session.undo().unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "one two one"
    );
}
