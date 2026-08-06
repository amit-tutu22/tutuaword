//! F03.S3 — apply character spacing via edit commands.

use tw_edit::{Command, EditSession};
use tw_model::CharFormat;

#[test]
fn u_f03_s3_character_spacing_applies_to_run() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Track".into(),
        })
        .unwrap();

    session
        .apply(Command::SetCharFormat {
            run_id,
            start: 0,
            end: usize::MAX,
            format: CharFormat {
                character_spacing: Some(3.0),
                ..Default::default()
            },
            merge: true,
        })
        .unwrap();

    let format = &session.document.paragraph_at(0, 0).unwrap().runs[0].format;
    assert_eq!(format.character_spacing, Some(3.0));
}
