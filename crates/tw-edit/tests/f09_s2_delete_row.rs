//! F09.S2 — delete table row/column (U-F09-S2-delete-row).

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

fn table_cell_run_id(session: &EditSession) -> tw_model::NodeId {
    session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[0]
        .cells[0]
        .blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id
}

fn insert_3x3(session: &mut EditSession) -> tw_model::NodeId {
    let after = first_block_id(session);
    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 3,
            cols: 3,
        })
        .unwrap();
    session.document.sections[0].blocks[1].table().unwrap().id
}

#[test]
fn u_f09_s2_delete_row() {
    let mut session = EditSession::new();
    let table_id = insert_3x3(&mut session);

    session
        .apply(Command::DeleteTableRow { table_id, row: 1 })
        .unwrap();

    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows.len(), 2);

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks[1].table().unwrap().rows.len(), 3);
}

#[test]
fn u_f09_s2_delete_row_from_caret() {
    let mut session = EditSession::new();
    insert_3x3(&mut session);
    let run_id = table_cell_run_id(&session);
    let (table_id, row, _) = session.document.find_table_cell_for_run(run_id).unwrap();
    assert_eq!(row, 0);

    session
        .apply(Command::DeleteTableRow {
            table_id,
            row: row as u32,
        })
        .unwrap();
    assert_eq!(
        session.document.sections[0].blocks[1].table().unwrap().rows.len(),
        2
    );
}

#[test]
fn u_f09_s2_delete_column() {
    let mut session = EditSession::new();
    let table_id = insert_3x3(&mut session);

    session
        .apply(Command::DeleteTableColumn {
            table_id,
            column: 1,
        })
        .unwrap();

    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows[0].cells.len(), 2);
    assert_eq!(table.format.column_widths.len(), 2);

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks[1].table().unwrap().rows[0].cells.len(), 3);
}

#[test]
fn u_f09_s2_cannot_delete_last_row() {
    let mut session = EditSession::new();
    let after = first_block_id(&mut session);
    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 1,
            cols: 2,
        })
        .unwrap();
    let table_id = session.document.sections[0].blocks[1].table().unwrap().id;
    assert!(session
        .apply(Command::DeleteTableRow { table_id, row: 0 })
        .is_err());
}
