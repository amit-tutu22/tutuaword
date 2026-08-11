//! Stress: repeated find over churned document text.

use tw_edit::{find_matches, Command, DocPosition, DocRange, EditSession};

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

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f18_s1_find_matches_churn() {
    let mut session = EditSession::new();
    let mut total_hits = 0usize;

    for i in 0..200 {
        let run_id = tail_run(&session);
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        let snippet = if i % 2 == 0 {
            format!(" find{i} ")
        } else {
            format!(" FIND{i} ")
        };
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: snippet,
            })
            .unwrap();

        let range = body_range(&session);
        let case_sensitive = find_matches(&session.document, &range, "find", true, false, false, None).unwrap();
        let case_insensitive = find_matches(&session.document, &range, "find", false, false, false, None).unwrap();
        total_hits += case_sensitive.len() + case_insensitive.len();
        assert!(case_insensitive.len() >= case_sensitive.len());
    }

    assert!(total_hits > 200, "expected find churn hits, got {total_hits}");
}
