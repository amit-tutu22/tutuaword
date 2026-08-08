//! F11.S3 — text boxes with embedded paragraph; WordArt insert.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ShapeKind, ShapeStyle};

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
fn u_f11_s3_insert_text_box_has_embedded_paragraph() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertTextBox {
            after_block_id: after,
            width: 180.0,
            height: 90.0,
            style: ShapeStyle::inserted_default(),
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::TextBox);
    assert_eq!(shape.paragraphs.len(), 1);
    assert_eq!(shape.paragraphs[0].runs.len(), 1);

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);
}

#[test]
fn u_f11_s3_insert_word_art_stores_styled_text() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertWordArt {
            after_block_id: after,
            text: "Hello".into(),
            width: 220.0,
            height: 72.0,
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::WordArt);
    assert_eq!(shape.paragraphs.len(), 1);
    let text: String = shape.paragraphs[0]
        .runs
        .iter()
        .map(|r| r.text())
        .collect();
    assert_eq!(text, "Hello");
    assert_eq!(shape.paragraphs[0].runs[0].format.bold, Some(true));
}
