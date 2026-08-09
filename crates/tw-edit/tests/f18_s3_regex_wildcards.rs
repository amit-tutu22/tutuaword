//! F18.S3 — regex and wildcard find/replace.

use tw_edit::{
    find_matches, Command, DocPosition, DocRange, EditError, EditSession,
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
fn u_f18_s3_regex_finds_digit_runs() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "order 42 ships on 1001");
    let matches = find_matches(&session.document, &range, r"\d+", true, true, false, None).unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].end.char_offset - matches[0].start.char_offset, 2);
    assert_eq!(matches[1].end.char_offset - matches[1].start.char_offset, 4);
}

#[test]
fn u_f18_s3_regex_word_pattern_is_case_insensitive_when_requested() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "Cat cat CAT");
    let matches = find_matches(&session.document, &range, r"cat", false, true, false, None).unwrap();
    assert_eq!(matches.len(), 3);
}

#[test]
fn u_f18_s3_wildcard_question_mark_matches_single_character() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "cat cot car");
    let matches = find_matches(&session.document, &range, "c?t", true, false, true, None).unwrap();
    assert_eq!(matches.len(), 2);
}

#[test]
fn u_f18_s3_wildcard_star_matches_any_suffix() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "run runner running");
    let matches = find_matches(&session.document, &range, "run*", true, false, true, None).unwrap();
    assert_eq!(matches.len(), 3);
}

#[test]
fn u_f18_s3_invalid_regex_returns_error() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "broken pattern");
    let err = find_matches(&session.document, &range, r"[unclosed", true, true, false, None)
        .unwrap_err();
    assert!(matches!(err, EditError::InvalidRegex(_)));
}

#[test]
fn u_f18_s3_regex_replace_all_substitutes_matches() {
    let mut session = EditSession::new();
    let range = seed_text(&mut session, "item 1 and item 22");
    let result = session
        .apply(Command::FindReplace {
            range,
            find: r"item \d+".into(),
            replace: "entry".into(),
            match_case: true,
            use_regex: true,
            use_wildcards: false,
        })
        .unwrap();
    assert_eq!(result.replacement_count, 2);
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "entry and entry"
    );
}
