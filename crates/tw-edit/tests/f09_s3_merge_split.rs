//! F09.S3 — merge and split table cells (U-F09-S3-merge-cells-span).

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

fn insert_2x2(session: &mut EditSession) -> tw_model::NodeId {
    let after = first_block_id(session);
    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 2,
            cols: 2,
        })
        .unwrap();
    session.document.sections[0].blocks[1].table().unwrap().id
}

#[test]
fn u_f09_s3_merge_cells_span() {
    let mut session = EditSession::new();
    let table_id = insert_2x2(&mut session);

    session
        .apply(Command::MergeTableCells {
            table_id,
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 1,
        })
        .unwrap();

    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows[0].cells[0].format.colspan, 2);
    assert_eq!(table.rows[0].cells[0].format.rowspan, 1);
}

#[test]
fn u_f09_s3_merge_from_caret_and_undo() {
    let mut session = EditSession::new();
    insert_2x2(&mut session);
    let run_id = session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[0]
        .cells[0]
        .blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    let cmd = tw_edit::merge_table_cells_right_command_for_caret(&session.document, Some(run_id))
        .expect("merge command");
    session.apply(cmd).unwrap();
    assert_eq!(
        session.document.sections[0].blocks[1]
            .table()
            .unwrap()
            .rows[0]
            .cells[0]
            .format
            .colspan,
        2
    );

    session.undo().unwrap();
    assert_eq!(
        session.document.sections[0].blocks[1]
            .table()
            .unwrap()
            .rows[0]
            .cells[0]
            .format
            .colspan,
        1
    );
}

#[test]
fn u_f09_s3_split_cell() {
    let mut session = EditSession::new();
    let table_id = insert_2x2(&mut session);
    session
        .apply(Command::MergeTableCells {
            table_id,
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 1,
        })
        .unwrap();

    session
        .apply(Command::SplitTableCell {
            table_id,
            row: 0,
            col: 0,
        })
        .unwrap();

    let cell = &session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[0]
        .cells[0];
    assert_eq!(cell.format.colspan, 1);
    assert_eq!(cell.format.rowspan, 1);
}

#[test]
fn u_f09_s3_split_unmerged_fails() {
    let mut session = EditSession::new();
    let table_id = insert_2x2(&mut session);
    assert!(session
        .apply(Command::SplitTableCell {
            table_id,
            row: 0,
            col: 0,
        })
        .is_err());
}
