//! Stress: repeated accept/reject at caret with track changes enabled.

use tw_edit::{
    accept_revision_at_caret, reject_revision_at_caret, Command, EditSession,
};

fn tail_pos(session: &EditSession) -> (tw_model::NodeId, usize) {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let run_id = para.runs.last().unwrap().id;
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    (run_id, offset)
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f17_s2_accept_reject_caret_churn() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    let mut accepted = 0usize;
    let mut rejected = 0usize;

    for i in 0..100 {
        let (run_id, offset) = tail_pos(&session);
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: format!("w{i} "),
            })
            .unwrap();
        let (run_id, _) = tail_pos(&session);
        if i % 2 == 0 {
            let cmd = accept_revision_at_caret(&session.document, Some(run_id)).unwrap();
            session.apply(cmd).unwrap();
            accepted += 1;
        } else {
            let cmd = reject_revision_at_caret(&session.document, Some(run_id)).unwrap();
            session.apply(cmd).unwrap();
            rejected += 1;
        }
    }

    assert_eq!(accepted + rejected, 100);
    assert!(session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .all(|r| r.revision.is_none()));
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f17_s2_adjacent_navigation_churn() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    for i in 0..50 {
        let (run_id, offset) = tail_pos(&session);
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: format!("t{i} "),
            })
            .unwrap();
    }

    let ids = tw_model::revision_run_ids(&session.document);
    assert_eq!(ids.len(), 50);

    let mut caret = ids[0];
    for _ in 0..200 {
        caret = tw_model::adjacent_revision_run(&session.document, caret, true).unwrap();
    }
    assert!(tw_model::revision_at_run(&session.document, caret).is_some());
}
