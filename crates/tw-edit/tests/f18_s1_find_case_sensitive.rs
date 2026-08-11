//! F18.S1 — find matches respect match_case.

use tw_edit::{
    find_matches, Command, DocPosition, DocRange, EditSession, FindMatch,
};

fn body_range(session: &EditSession) -> DocRange {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let last = para.runs.last().unwrap();
    DocRange {
        start: DocPosition {
            run_id: para.runs[0].id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id: last.id,
            char_offset: tw_edit::run_char_len_by_id(&session.document, last.id),
        },
    }
}

fn seed_text(session: &mut EditSession, text: &str) -> DocRange {
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: text.into(),
        })
        .unwrap();
    body_range(session)
}

#[test]
fn u_f18_s1_find_case_sensitive_matches_exact_case_only() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "Foo foo FOO");
    let matches = find_matches(&session.document, &range, "foo", true, false, false, None).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].start.char_offset, 4);
    assert_eq!(matches[0].end.char_offset, 7);
}

#[test]
fn u_f18_s1_find_case_insensitive_matches_all_variants() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "Foo foo FOO");
    let matches = find_matches(&session.document, &range, "foo", false, false, false, None).unwrap();
    assert_eq!(matches.len(), 3);
}

#[test]
fn u_f18_s1_find_replace_case_sensitive_leaves_other_cases() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Cat cat CAT".into(),
        })
        .unwrap();
    let end = tw_edit::run_char_len_by_id(&session.document, run_id);
    session
        .apply(Command::FindReplace {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: end,
                },
            },
            find: "cat".into(),
            replace: "dog".into(),
            match_case: true,
            use_regex: false,
            use_wildcards: false,
        })
        .unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "Cat dog CAT"
    );
}

#[test]
fn u_f18_s1_find_matches_across_split_runs() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "alpha beta alpha".into(),
        })
        .unwrap();
    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 6,
            end: 10,
            format: tw_model::CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();
    let range = body_range(&session);
    let matches = find_matches(&session.document, &range, "alpha", true, false, false, None).unwrap();
    assert_eq!(matches.len(), 2);
    let FindMatch { start, end } = &matches[0];
    assert_eq!(end.char_offset - start.char_offset, 5);
}
