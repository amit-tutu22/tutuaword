//! Table drag move via SetShapeAnchor (shared object-anchor path).

use tw_edit::{Command, EditSession};
use tw_model::{AnchorOrigin, Block, ImageAnchor};

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
fn set_shape_anchor_moves_table() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 2,
            cols: 2,
        })
        .unwrap();
    let table_id = session.document.sections[0].blocks[1].table().unwrap().id;
    assert!(session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .anchor
        .is_none());

    session
        .apply(Command::SetShapeAnchor {
            shape_id: table_id,
            anchor: ImageAnchor {
                x: 40.0,
                y: 80.0,
                origin_x: AnchorOrigin::Column,
                origin_y: AnchorOrigin::Column,
            },
        })
        .unwrap();

    let table = session.document.sections[0].blocks[1].table().unwrap();
    let anchor = table.anchor.expect("table anchor");
    assert_eq!(anchor.x, 40.0);
    assert_eq!(anchor.y, 80.0);

    session.undo().unwrap();
    assert!(session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .anchor
        .is_none());
}
