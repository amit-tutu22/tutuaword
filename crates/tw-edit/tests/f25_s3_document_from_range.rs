//! F25.S3 — document_from_range (unit + stress).

use tw_edit::{document_from_range, text_in_range, Command, DocPosition, DocRange, EditSession};
use tw_model::{Block, CharFormat, Paragraph};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f25_s3_document_from_range_partial_run() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello World".into(),
        })
        .unwrap();

    let range = DocRange {
        start: DocPosition {
            run_id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id,
            char_offset: 5,
        },
    };
    let subset = document_from_range(&session.document, &range).expect("subset");
    let text: String = subset
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .map(|p| p.full_text())
        .collect();
    assert_eq!(text, "Hello");
    assert_eq!(
        text_in_range(&session.document, &range).unwrap(),
        "Hello"
    );
}

#[test]
fn u_f25_s3_document_from_range_preserves_format() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "BoldMe".into(),
        })
        .unwrap();
    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 0,
            end: 6,
            format: CharFormat {
                bold: Some(true),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let range = DocRange {
        start: DocPosition {
            run_id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id,
            char_offset: 6,
        },
    };
    let subset = document_from_range(&session.document, &range).unwrap();
    let run = &subset.paragraph_at(0, 0).unwrap().runs[0];
    assert_eq!(run.format.bold, Some(true));
    assert_eq!(run.text(), "BoldMe");
}

#[test]
fn u_f25_s3_document_from_range_multi_paragraph() {
    let mut session = EditSession::new();
    let run_a = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id: run_a,
            offset: 0,
            text: "Alpha".into(),
        })
        .unwrap();
    session.document.sections[0]
        .blocks
        .push(Block::Paragraph(Paragraph::with_text("Bravo")));
    let run_b = session.document.paragraph_at(0, 1).unwrap().runs[0].id;

    let range = DocRange {
        start: DocPosition {
            run_id: run_a,
            char_offset: 2,
        },
        end: DocPosition {
            run_id: run_b,
            char_offset: 3,
        },
    };
    let subset = document_from_range(&session.document, &range).unwrap();
    assert_eq!(subset.sections[0].blocks.len(), 2);
    assert_eq!(subset.paragraph_at(0, 0).unwrap().full_text(), "pha");
    assert_eq!(subset.paragraph_at(0, 1).unwrap().full_text(), "Bra");
}

#[test]
fn u_f25_s3_document_from_range_collapsed_errors() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "x".into(),
        })
        .unwrap();
    let range = DocRange {
        start: DocPosition {
            run_id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id,
            char_offset: 0,
        },
    };
    assert!(document_from_range(&session.document, &range).is_err());
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f25_s3_document_from_range_churn() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "0123456789".repeat(20),
        })
        .unwrap();
    let len = tw_edit::run_char_len_by_id(&session.document, run_id);
    for start in (0..len).step_by(7) {
        let end = (start + 11).min(len);
        if start >= end {
            continue;
        }
        let range = DocRange {
            start: DocPosition {
                run_id,
                char_offset: start,
            },
            end: DocPosition {
                run_id,
                char_offset: end,
            },
        };
        let subset = document_from_range(&session.document, &range).unwrap();
        let expected = text_in_range(&session.document, &range).unwrap();
        let got = subset.paragraph_at(0, 0).unwrap().full_text();
        assert_eq!(got, expected, "slice {start}..{end}");
    }
}
