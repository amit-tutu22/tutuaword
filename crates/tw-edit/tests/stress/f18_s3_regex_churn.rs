//! Stress: regex find/replace over churned document text.

use tw_edit::{document_body_range, find_matches, Command, EditSession};

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
fn stress_f18_s3_regex_find_replace_churn() {
    let mut session = EditSession::new();
    let mut total_matches = 0usize;
    let mut total_replacements = 0usize;

    for i in 0..100 {
        let run_id = tail_run(&session);
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: format!(" id{i} id{i} "),
            })
            .unwrap();
        let range = document_body_range(&session.document).unwrap();
        let pattern = format!(r"id{i}");
        let matches =
            find_matches(&session.document, &range, &pattern, true, true, false, None).unwrap();
        total_matches += matches.len();
        let result = session
            .apply(Command::FindReplace {
                range,
                find: pattern,
                replace: format!("#{i}"),
                match_case: true,
                use_regex: true,
                use_wildcards: false,
            })
            .unwrap();
        total_replacements += result.replacement_count;
        assert!(result.replacement_count >= 2);
    }

    assert!(total_matches >= 200, "expected regex find churn, got {total_matches}");
    assert!(
        total_replacements >= 200,
        "expected regex replace churn, got {total_replacements}"
    );
}
