//! F17.S4 — compare documents and read-only model settings.

use tw_edit::{Command, EditSession};
use tw_model::compare_documents;

#[test]
fn u_f17_s4_compare_documents_after_edit() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Draft paragraph.".into(),
        })
        .unwrap();
    let baseline = session.document.clone();
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Revised ".into(),
        })
        .unwrap();
    let summary = compare_documents(&baseline, &session.document);
    assert!(
        summary.deletion_count > 0 || summary.insertion_count > 0,
        "expected diff after edit: {summary:?}"
    );
}


#[test]
fn u_f17_s4_read_only_setting_persists_on_document() {
    let mut session = EditSession::new();
    session.document.settings.read_only = true;
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;

    // Edit layer still applies locally; worker session blocks ApplyEdit when read_only.
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "local".into(),
        })
        .unwrap();
    assert!(session.document.settings.read_only);
    assert!(session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .full_text()
        .contains("local"));
}
