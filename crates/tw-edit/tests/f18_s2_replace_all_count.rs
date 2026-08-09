//! F18.S2 — FindReplace reports replacement count.

use tw_edit::{
    document_body_range, find_matches, Command, DocRange, EditSession,
};

fn seed_text(session: &mut EditSession, text: &str) -> DocRange {
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: text.into(),
        })
        .unwrap();
    document_body_range(&session.document).unwrap()
}

#[test]
fn u_f18_s2_replace_all_count_matches_occurrences() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "cat dog cat bird cat");
    let result = session
        .apply(Command::FindReplace {
            range,
            find: "cat".into(),
            replace: "fish".into(),
            match_case: true,
            use_regex: false,
            use_wildcards: false,
        })
        .unwrap();
    assert_eq!(result.replacement_count, 3);
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "fish dog fish bird fish"
    );
}

#[test]
fn u_f18_s2_replace_all_count_zero_when_no_match() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "nothing here");
    let result = session
        .apply(Command::FindReplace {
            range,
            find: "missing".into(),
            replace: "nope".into(),
            match_case: true,
            use_regex: false,
            use_wildcards: false,
        })
        .unwrap();
    assert_eq!(result.replacement_count, 0);
}

#[test]
fn u_f18_s2_replace_all_count_respects_match_case() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "Cat cat CAT");
    let sensitive = session
        .apply(Command::FindReplace {
            range: range.clone(),
            find: "cat".into(),
            replace: "dog".into(),
            match_case: true,
            use_regex: false,
            use_wildcards: false,
        })
        .unwrap();
    assert_eq!(sensitive.replacement_count, 1);

    let mut session = EditSession::new();
    let range = seed_text(&mut session, "Cat cat CAT");
    let insensitive = session
        .apply(Command::FindReplace {
            range,
            find: "cat".into(),
            replace: "dog".into(),
            match_case: false,
            use_regex: false,
            use_wildcards: false,
        })
        .unwrap();
    assert_eq!(insensitive.replacement_count, 3);
}

#[test]
fn u_f18_s2_find_matches_count_aligns_with_replace() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "one two one three one");
    let matches = find_matches(&session.document, &range, "one", true, false, false, None).unwrap();
    let result = session
        .apply(Command::FindReplace {
            range,
            find: "one".into(),
            replace: "1".into(),
            match_case: true,
            use_regex: false,
            use_wildcards: false,
        })
        .unwrap();
    assert_eq!(matches.len(), result.replacement_count);
    assert_eq!(result.replacement_count, 3);
}
