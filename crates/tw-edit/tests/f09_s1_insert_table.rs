//! F09.S1 — insert 3×3 table (U-F09-S1-insert-3x3).

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
fn u_f09_s1_insert_3x3() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 3,
            cols: 3,
        })
        .unwrap();

    assert_eq!(session.document.sections[0].blocks.len(), 2);
    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0].cells.len(), 3);
    assert_eq!(table.rows[1].cells.len(), 3);
    assert_eq!(table.rows[2].cells.len(), 3);
}

#[test]
fn u_f09_s1_insert_3x3_undo() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 3,
            cols: 3,
        })
        .unwrap();
    session.undo().unwrap();

    assert_eq!(session.document.sections[0].blocks.len(), 1);
    assert!(session.document.sections[0].blocks[0].paragraph().is_some());
}
