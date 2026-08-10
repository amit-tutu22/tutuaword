//! F19.S4 — document bookmark navigation entries.

use tw_edit::{Command, EditSession};
use tw_model::document_bookmarks;

#[test]
fn u_f19_s4_document_bookmarks_lists_anchors() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Intro".into(),
        })
        .unwrap();
    let bookmark_run = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertBookmark {
            run_id: bookmark_run,
            offset: 0,
            name: "Top".into(),
        })
        .unwrap();

    let entries = document_bookmarks(&session.document);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "Top");
}
