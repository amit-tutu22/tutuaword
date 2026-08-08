//! F12.S3 — insert read-only SmartArt diagram placeholder.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ShapeKind};

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
fn u_f12_s3_insert_diagram_placeholder() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertDiagram {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::Diagram);
    assert_eq!(shape.shape.width, 432.0);
    assert_eq!(shape.shape.height, 216.0);
    assert!(shape.preview_image.is_none());

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);
}
