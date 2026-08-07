//! text_in_range must not panic when layout-derived offsets exceed buffer length.

use tw_edit::{text_in_range, Command, DocPosition, DocRange, EditSession};
use tw_model::NodeId;

#[test]
fn text_in_range_clamps_oversized_end_offset() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "hello".into(),
        })
        .unwrap();

    let text = text_in_range(
        &session.document,
        &DocRange {
            start: DocPosition {
                run_id,
                char_offset: 0,
            },
            end: DocPosition {
                run_id,
                char_offset: 999,
            },
        },
    )
    .unwrap();
    assert_eq!(text, "hello");
}
