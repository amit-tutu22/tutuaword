//! F18.S2 — SyncSession replace-all returns replacement count.

use tw_core::SyncSession;
use tw_edit::{document_body_range, Command};

fn first_run(session: &SyncSession) -> tw_model::NodeId {
    session.edit.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f18_s2_sync_session_replace_all_count() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "foo bar foo".into(),
    });
    let range = document_body_range(&session.edit.document).unwrap();
    let result = session.apply(Command::FindReplace {
        range,
        find: "foo".into(),
        replace: "baz".into(),
        match_case: true,
        use_regex: false,
        use_wildcards: false,
    });
    assert_eq!(result.replacement_count, 2);
    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().full_text(),
        "baz bar baz"
    );
}
