//! Stress: grammar check + compare over churned document text.

use tw_edit::{Command, EditSession};
use tw_model::compare_text;
use tw_spell::GrammarChecker;

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

fn visible_text(session: &EditSession) -> String {
    tw_model::document_plain_text(&session.document)
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f17_s4_grammar_compare_churn() {
    let checker = GrammarChecker::english();
    let mut session = EditSession::new();
    let mut previous = visible_text(&session);
    let mut grammar_hits = 0usize;
    let mut compare_ops = 0usize;

    for i in 0..100 {
        let run_id = tail_run(&session);
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        let snippet = if i % 3 == 0 {
            format!(" could of block{i} ")
        } else if i % 3 == 1 {
            format!(" clean line{i} ")
        } else {
            format!(" alot  gap{i} ")
        };
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: snippet,
            })
            .unwrap();

        let text = visible_text(&session);
        let issues = checker.check_text(&text);
        grammar_hits += issues.len();

        let summary = compare_text(&previous, &text);
        compare_ops += summary.insertion_count + summary.deletion_count;
        previous = text;
    }

    assert!(grammar_hits > 50, "expected grammar hits, got {grammar_hits}");
    assert!(compare_ops > 50, "expected compare churn, got {compare_ops}");
}
