//! F18.S4 — SyncSession format find.

use tw_core::SyncSession;
use tw_edit::{Command, FindFormatFilter};

#[test]
fn u_f18_s4_sync_session_find_bold_text() {
    let mut session = SyncSession::new();
    let run_id = session.edit.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "alpha BETA gamma".into(),
    });
    session.apply(Command::SetCharFormat {
        run_id,
        start: 6,
        end: 10,
        format: tw_model::CharFormat {
            bold: Some(true),
            ..Default::default()
        },
        merge: true,
    });
    let filter = FindFormatFilter {
        bold: Some(true),
        ..Default::default()
    };
    let matches = session.find_matches("BETA", false, false, false, Some(&filter));
    assert_eq!(matches.len(), 1);
}
