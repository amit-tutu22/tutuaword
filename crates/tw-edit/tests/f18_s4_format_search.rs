//! F18.S4 — find by character/paragraph format.

use tw_edit::{
    find_matches, Command, DocPosition, DocRange, EditSession, FindFormatFilter,
};
use tw_model::CharFormat;

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

fn session_with_bold_word() -> EditSession {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "plain BOLD plain".into(),
        })
        .unwrap();
    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 6,
            end: 10,
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();
    session
}

#[test]
fn u_f18_s4_find_bold_runs_without_query() {
    let session = session_with_bold_word();
    let range = body_range(&session);
    let filter = FindFormatFilter {
        bold: Some(true),
        ..Default::default()
    };
    let matches = find_matches(&session.document, &range, "", false, false, false, Some(&filter))
        .unwrap();
    assert_eq!(matches.len(), 1);
    let text = tw_edit::run_slice_by_id(
        &session.document,
        matches[0].start.run_id,
        matches[0].start.char_offset..matches[0].end.char_offset,
    );
    assert_eq!(text, "BOLD");
}

#[test]
fn u_f18_s4_find_text_only_in_bold_runs() {
    let session = session_with_bold_word();
    let range = body_range(&session);
    let filter = FindFormatFilter {
        bold: Some(true),
        ..Default::default()
    };
    let plain =
        find_matches(&session.document, &range, "plain", false, false, false, Some(&filter))
            .unwrap();
    assert_eq!(plain.len(), 0);
    let bold =
        find_matches(&session.document, &range, "BOLD", false, false, false, Some(&filter))
            .unwrap();
    assert_eq!(bold.len(), 1);
}

#[test]
fn u_f18_s4_find_heading_style_by_name() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Title line".into(),
        })
        .unwrap();
    let heading = session.document.styles.find_style_by_name("Heading 1").unwrap();
    session.document.paragraph_at_mut(0, 0).unwrap().style_id = Some(heading.id);
    let range = body_range(&session);
    let filter = FindFormatFilter {
        style_name: Some("Heading 1".into()),
        ..Default::default()
    };
    let matches = find_matches(&session.document, &range, "", false, false, false, Some(&filter))
        .unwrap();
    assert_eq!(matches.len(), 1);
}

#[test]
fn u_f18_s4_find_heading_style_with_query() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Title line".into(),
        })
        .unwrap();
    let heading = session.document.styles.find_style_by_name("Heading 1").unwrap();
    session.document.paragraph_at_mut(0, 0).unwrap().style_id = Some(heading.id);
    let range = body_range(&session);
    let filter = FindFormatFilter {
        style_name: Some("heading 1".into()),
        ..Default::default()
    };
    let matches =
        find_matches(&session.document, &range, "Title", false, false, false, Some(&filter))
            .unwrap();
    assert_eq!(matches.len(), 1);
}
