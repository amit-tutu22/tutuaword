//! Stress: repeated replace-all over churned document text.

use tw_edit::{document_body_range, Command, EditSession};

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
fn stress_f18_s2_replace_all_churn() {
    let mut session = EditSession::new();
    let mut total_replacements = 0usize;

    for i in 0..100 {
        let run_id = tail_run(&session);
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: format!(" token{i} token{i} "),
            })
            .unwrap();
        let range = document_body_range(&session.document).unwrap();
        let needle = format!("token{i}");
        let result = session
            .apply(Command::FindReplace {
                range,
                find: needle,
                replace: format!("t{i}"),
                match_case: true,
                use_regex: false,
                use_wildcards: false,
            })
            .unwrap();
        total_replacements += result.replacement_count;
        assert!(result.replacement_count >= 2);
    }

    assert!(total_replacements >= 200, "expected replace churn, got {total_replacements}");
}
