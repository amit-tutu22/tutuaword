//! Stress: repeated format-filtered find over churned bold text.

use tw_edit::{document_body_range, find_matches, Command, EditSession, FindFormatFilter};
use tw_model::CharFormat;

fn tail_run(session: &EditSession) -> tw_model::NodeId {
    session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .last()
        .unwrap()
        .id
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f18_s4_format_find_churn() {
    let mut session = EditSession::new();
    let filter = FindFormatFilter {
        bold: Some(true),
        ..Default::default()
    };
    let mut total = 0usize;

    for i in 0..100 {
        let run_id = tail_run(&session);
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        let word = format!(" hot{i} ");
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: word.clone(),
            })
            .unwrap();
        let start = offset + 1;
        let end = start + format!("hot{i}").chars().count();
        session
            .apply(Command::SetCharFormat {
                run_id,
                start,
                end,
                format: CharFormat {
                    bold: Some(true),
                    ..Default::default()
                },
                merge: true,
            })
            .unwrap();
        let range = document_body_range(&session.document).unwrap();
        let needle = format!("hot{i}");
        let matches = find_matches(
            &session.document,
            &range,
            &needle,
            true,
            false,
            false,
            Some(&filter),
        )
        .unwrap();
        total += matches.len();
        assert!(matches.len() >= 1);
    }

    assert!(total >= 100, "expected format find churn, got {total}");
}
