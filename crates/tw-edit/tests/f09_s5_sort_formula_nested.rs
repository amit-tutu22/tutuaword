//! F09.S5 — sort, SUM formula, nested table (U-F09-S5-*).

use tw_edit::{Command, EditSession};
use tw_model::{Block, FieldType, RunContent};

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

fn table_with_data(session: &mut EditSession) -> (tw_model::NodeId, tw_model::NodeId) {
    let after = first_block_id(session);
    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 4,
            cols: 2,
        })
        .unwrap();
    let table_id = session.document.sections[0].blocks[1].table().unwrap().id;

    let set_cell = |session: &mut EditSession, row: u32, col: u32, text: &str| {
        let run_id = {
            let table = session.document.sections[0].blocks[1].table().unwrap();
            let cell = &table.rows[row as usize].cells[col as usize];
            cell.blocks
                .iter()
                .find_map(|b| b.paragraph())
                .unwrap()
                .runs[0]
                .id
        };
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: text.into(),
            })
            .unwrap();
    };

    set_cell(session, 0, 0, "Name");
    set_cell(session, 0, 1, "Score");
    set_cell(session, 1, 0, "Charlie");
    set_cell(session, 1, 1, "30");
    set_cell(session, 2, 0, "Alice");
    set_cell(session, 2, 1, "10");
    set_cell(session, 3, 0, "Bob");
    set_cell(session, 3, 1, "20");

    let run_id = session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[1]
        .cells[0]
        .blocks
        .iter()
        .find_map(|b| b.paragraph())
        .unwrap()
        .runs[0]
        .id;
    (table_id, run_id)
}

#[test]
fn u_f09_s5_sort_rows_ascending() {
    let mut session = EditSession::new();
    let (table_id, _) = table_with_data(&mut session);

    session
        .apply(Command::SortTableRows {
            table_id,
            column: 0,
            ascending: true,
            skip_header: true,
        })
        .unwrap();

    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows[1].cells[0].blocks[0].paragraph().unwrap().full_text(), "Alice");
    assert_eq!(table.rows[2].cells[0].blocks[0].paragraph().unwrap().full_text(), "Bob");
    assert_eq!(table.rows[3].cells[0].blocks[0].paragraph().unwrap().full_text(), "Charlie");

    session.undo().unwrap();
    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows[1].cells[0].blocks[0].paragraph().unwrap().full_text(), "Charlie");
}

#[test]
fn u_f09_s5_insert_table_sum_field() {
    let mut session = EditSession::new();
    let (_, _) = table_with_data(&mut session);

    let sum_run = {
        let table = session.document.sections[0].blocks[1].table().unwrap();
        table.rows[3].cells[1].blocks[0].paragraph().unwrap().runs[0].id
    };

    session
        .apply(Command::InsertField {
            run_id: sum_run,
            offset: 0,
            field_type: FieldType::TableSumAbove,
            merge_name: None,
        })
        .unwrap();

    let cell = &session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[3]
        .cells[1];
    match &cell.blocks[0].paragraph().unwrap().runs[0].content {
        RunContent::Field(field) => assert_eq!(field.field_type, FieldType::TableSumAbove),
        _ => panic!("expected field run"),
    }
}

#[test]
fn u_f09_s5_ensure_shape_text_accepts_table() {
    let mut session = EditSession::new();
    let (table_id, _) = table_with_data(&mut session);

    let result = session
        .apply(Command::EnsureShapeText {
            shape_id: table_id,
        })
        .expect("tables must accept EnsureShapeText so cell edit can start");
    assert_eq!(result.affected_nodes, vec![table_id]);
    assert!(result.seed_run_id.is_some());
}

#[test]
fn u_f09_s5_insert_nested_table() {
    let mut session = EditSession::new();
    let (table_id, _) = table_with_data(&mut session);

    session
        .apply(Command::InsertNestedTable {
            table_id,
            row: 1,
            col: 0,
            rows: 2,
            cols: 2,
        })
        .unwrap();

    let cell = &session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[1]
        .cells[0];
    assert_eq!(cell.blocks.len(), 2);
    assert!(matches!(cell.blocks[1], Block::Table(_)));

    session.undo().unwrap();
    let cell = &session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[1]
        .cells[0];
    assert_eq!(cell.blocks.len(), 1);
}
