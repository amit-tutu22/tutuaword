//! WordArt body text must support insert and delete.

use tw_edit::{Command, EditSession};
use tw_model::Block;

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

#[test]
fn u_f11_wordart_delete_char() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertWordArt {
            after_block_id: after,
            text: "WordArt".into(),
            width: 220.0,
            height: 72.0,
        })
        .unwrap();

    let run_id = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "lh".into(),
        })
        .unwrap();
    let text = |s: &EditSession| -> String {
        s.document.sections[0].blocks[1]
            .shape()
            .unwrap()
            .paragraphs[0]
            .full_text()
    };
    assert_eq!(text(&session), "lhWordArt");

    // Backspace at offset 2 should remove 'h'
    session
        .apply(Command::DeleteRange {
            run_id,
            start: 1,
            end: 2,
        })
        .unwrap();
    assert_eq!(text(&session), "lWordArt");

    session
        .apply(Command::DeleteRange {
            run_id,
            start: 0,
            end: 1,
        })
        .unwrap();
    assert_eq!(text(&session), "WordArt");
}
