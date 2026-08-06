//! Stage 2 unit tests: cross-run character formatting, paragraph formatting,
//! delete-range, and undo of range edits.

use tw_edit::{Command, DocPosition, DocRange, EditSession};
use tw_model::{Alignment, CharFormat, ParaFormat, UnderlineStyle};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn mid_run_bold_splits_into_three_runs() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "HelloWorld".into(),
        })
        .unwrap();

    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 5,
            end: 10,
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let runs = &session.document.paragraph_at(0, 0).unwrap().runs;
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text(), "Hello");
    assert_ne!(runs[0].format.bold, Some(true));
    assert_eq!(runs[1].text(), "World");
    assert_eq!(runs[1].format.bold, Some(true));
}

#[test]
fn prefix_format_keeps_suffix_unformatted() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "abcdef".into(),
        })
        .unwrap();

    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 0,
            end: 3,
            format: CharFormat {
                italic: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let runs = &session.document.paragraph_at(0, 0).unwrap().runs;
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text(), "abc");
    assert_eq!(runs[0].format.italic, Some(true));
    assert_eq!(runs[1].text(), "def");
    assert_ne!(runs[1].format.italic, Some(true));
}

#[test]
fn char_format_range_across_two_runs() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "abcdef".into(),
        })
        .unwrap();
    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 3,
            end: 6,
            format: CharFormat {
                underline: Some(UnderlineStyle::Single),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let runs = &session.document.paragraph_at(0, 0).unwrap().runs;
    assert_eq!(runs.len(), 2);
    let start = DocPosition {
        run_id: runs[0].id,
        char_offset: 1,
    };
    let end = DocPosition {
        run_id: runs[1].id,
        char_offset: 2,
    };

    session
        .apply(Command::SetCharFormatRange {
            range: DocRange { start, end },
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let bold: String = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter(|r| r.format.bold == Some(true))
        .map(|r| r.text().to_string())
        .collect();
    assert_eq!(bold, "bcde");
}

#[test]
fn char_format_range_undo_restores_prior_format() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "xyz".into(),
        })
        .unwrap();

    session
        .apply(Command::SetCharFormatRange {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 3,
                },
            },
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().runs[0]
            .format
            .bold,
        Some(true)
    );

    session.undo().unwrap();
    assert_ne!(
        session.document.paragraph_at(0, 0).unwrap().runs[0]
            .format
            .bold,
        Some(true)
    );
}

#[test]
fn para_format_range_sets_alignment_on_caret_paragraph() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::SetParaFormatRange {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 0,
                },
            },
            format: ParaFormat {
                alignment: Some(Alignment::Center),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().format.alignment,
        Some(Alignment::Center)
    );
}

#[test]
fn para_format_range_spans_two_paragraphs() {
    let mut session = EditSession::new();
    let first_para = session.document.paragraph_at(0, 0).unwrap().id;
    let first_run = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id: first_run,
            offset: 0,
            text: "one".into(),
        })
        .unwrap();
    session
        .apply(Command::InsertParagraph {
            after_id: first_para,
        })
        .unwrap();

    let second_run = session.document.paragraph_at(0, 1).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id: second_run,
            offset: 0,
            text: "two".into(),
        })
        .unwrap();

    session
        .apply(Command::SetParaFormatRange {
            range: DocRange {
                start: DocPosition {
                    run_id: first_run,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id: second_run,
                    char_offset: 3,
                },
            },
            format: ParaFormat {
                alignment: Some(Alignment::Right),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().format.alignment,
        Some(Alignment::Right)
    );
    assert_eq!(
        session.document.paragraph_at(0, 1).unwrap().format.alignment,
        Some(Alignment::Right)
    );
}

#[test]
fn delete_range_removes_characters() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "abcdef".into(),
        })
        .unwrap();

    session
        .apply(Command::DeleteRange {
            run_id,
            start: 2,
            end: 5,
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "abf"
    );
}

#[test]
fn delete_range_undo_restores_text() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "hello".into(),
        })
        .unwrap();
    session
        .apply(Command::DeleteRange {
            run_id,
            start: 1,
            end: 4,
        })
        .unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "ho"
    );

    session.undo().unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "hello"
    );
}

#[test]
fn underline_none_clears_underline_via_merge() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "u".into(),
        })
        .unwrap();
    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 0,
            end: 1,
            format: CharFormat {
                underline: Some(UnderlineStyle::Single),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();
    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 0,
            end: 1,
            format: CharFormat {
                underline: Some(UnderlineStyle::None),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().runs[0]
            .format
            .underline,
        Some(UnderlineStyle::None)
    );
}

#[test]
fn reversed_doc_range_is_normalized() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "abcd".into(),
        })
        .unwrap();

    // end before start — should still bold "bc"
    session
        .apply(Command::SetCharFormatRange {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 3,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 1,
                },
            },
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let bold: String = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter(|r| r.format.bold == Some(true))
        .map(|r| r.text().to_string())
        .collect();
    assert_eq!(bold, "bc");
}

#[test]
fn split_paragraph_at_end_inserts_empty_paragraph() {
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
        .apply(Command::SplitParagraphAt {
            run_id,
            offset: 5,
        })
        .unwrap();

    assert_eq!(session.document.paragraph_at(0, 0).unwrap().full_text(), "Hello");
    assert_eq!(session.document.paragraph_at(0, 1).unwrap().full_text(), "");
}

#[test]
fn split_paragraph_mid_text_moves_suffix() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "HelloWorld".into(),
        })
        .unwrap();

    session
        .apply(Command::SplitParagraphAt {
            run_id,
            offset: 5,
        })
        .unwrap();

    assert_eq!(session.document.paragraph_at(0, 0).unwrap().full_text(), "Hello");
    assert_eq!(session.document.paragraph_at(0, 1).unwrap().full_text(), "World");
}

#[test]
fn undo_split_paragraph_restores_full_text() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "HelloWorld".into(),
        })
        .unwrap();

    session
        .apply(Command::SplitParagraphAt {
            run_id,
            offset: 5,
        })
        .unwrap();

    session.undo().unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "HelloWorld"
    );
    assert!(session.document.paragraph_at(0, 1).is_none());
}

#[test]
fn font_size_at_collapsed_run_end_formats_visible_text() {
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

#[test]
fn font_family_at_collapsed_run_end_formats_visible_text() {
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
                font_family: Some("Georgia".into()),
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
            .all(|run| run.format.font_family.as_deref() == Some("Georgia")),
        "font family should apply when caret is at run end"
    );
}
