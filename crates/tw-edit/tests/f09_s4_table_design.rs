//! F09.S4 — table borders, shading, column resize, AutoFit (U-F09-S4-*).

use tw_edit::{autofit_table_command_for_caret, Command, EditSession};
use tw_model::{Block, Color};

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

fn insert_3x3(session: &mut EditSession) -> (tw_model::NodeId, tw_model::NodeId) {
    let after = first_block_id(session);
    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 3,
            cols: 3,
        })
        .unwrap();
    let table = session.document.sections[0].blocks[1].table().unwrap();
    let run_id = table.rows[0].cells[0]
        .blocks
        .iter()
        .find_map(|b| b.paragraph())
        .unwrap()
        .runs[0]
        .id;
    (table.id, run_id)
}

#[test]
fn u_f09_s4_set_table_border() {
    let mut session = EditSession::new();
    let (table_id, _) = insert_3x3(&mut session);

    session
        .apply(Command::SetTableBorder {
            table_id,
            border: Some(tw_model::BorderSpec {
                width: 2.5,
                color: Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            }),
        })
        .unwrap();

    let border = session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .format
        .border
        .unwrap();
    assert!((border.width - 2.5).abs() < 0.01);
    assert_eq!(border.color.r, 255);

    session.undo().unwrap();
    let restored = session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .format
        .border
        .unwrap();
    assert!((restored.width - 1.0).abs() < 0.01);
}

#[test]
fn u_f09_s4_cell_shading() {
    let mut session = EditSession::new();
    let (table_id, _) = insert_3x3(&mut session);
    let shading = Color {
        r: 200,
        g: 220,
        b: 255,
        a: 255,
    };

    session
        .apply(Command::SetTableCellShading {
            table_id,
            row: 0,
            col: 0,
            background: Some(shading),
        })
        .unwrap();

    let cell = &session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[0]
        .cells[0];
    assert_eq!(cell.format.background, Some(shading));

    session.undo().unwrap();
    assert!(session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .rows[0]
        .cells[0]
        .format
        .background
        .is_none());
}

#[test]
fn u_f09_s4_resize_column_undo() {
    let mut session = EditSession::new();
    let (table_id, _) = insert_3x3(&mut session);

    session
        .apply(Command::ResizeTableColumn {
            table_id,
            column: 1,
            width: 150.0,
        })
        .unwrap();

    let widths = &session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .format
        .column_widths;
    assert!((widths[1] - 150.0).abs() < 0.01);

    session.undo().unwrap();
    assert!((session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .format
        .column_widths[1]
        - 100.0)
        .abs()
        < 0.01);
}

#[test]
fn u_f09_s4_autofit_to_window() {
    let mut session = EditSession::new();
    let (table_id, run_id) = insert_3x3(&mut session);

    let cmd = autofit_table_command_for_caret(&session.document, Some(run_id)).unwrap();
    let Command::AutoFitTable { target_width, .. } = cmd else {
        panic!("expected AutoFitTable");
    };
    assert!((target_width - 468.0).abs() < 0.01);

    session.apply(cmd).unwrap();
    let table = session.document.sections[0].blocks[1].table().unwrap();
    let sum: f32 = table.format.column_widths.iter().sum();
    assert!((sum - target_width).abs() < 0.5);
    assert!((table.format.width.unwrap() - target_width).abs() < 0.5);

    session.undo().unwrap();
    let restored_sum: f32 = session.document.sections[0].blocks[1]
        .table()
        .unwrap()
        .format
        .column_widths
        .iter()
        .sum();
    assert!((restored_sum - 300.0).abs() < 0.5);
}

#[test]
fn u_f09_s4_caret_commands_require_table() {
    let session = EditSession::new();
    assert!(autofit_table_command_for_caret(&session.document, None).is_none());
}
