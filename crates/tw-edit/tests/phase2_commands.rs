use tw_edit::{Command, EditSession};
use tw_model::{Block, NumberingRef};

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
fn insert_table_adds_block_after_paragraph() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 3,
            cols: 4,
        })
        .unwrap();

    assert_eq!(session.document.sections[0].blocks.len(), 2);
    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0].cells.len(), 4);
}

#[test]
fn insert_table_undo_removes_block() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 2,
            cols: 2,
        })
        .unwrap();
    session.undo().unwrap();

    assert_eq!(session.document.sections[0].blocks.len(), 1);
    assert!(session.document.sections[0].blocks[0].paragraph().is_some());
}

#[test]
fn insert_image_adds_placeholder_block() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertImage {
            after_block_id: after,
            width: 200.0,
            height: 150.0,
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.display_width, 200.0);
    assert_eq!(image.display_height, 150.0);
}

#[test]
fn merge_table_cells_sets_span() {
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
}

#[test]
fn resize_table_column_updates_widths() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 1,
            cols: 3,
        })
        .unwrap();

    let table_id = session.document.sections[0].blocks[1].table().unwrap().id;
    session
        .apply(Command::ResizeTableColumn {
            table_id,
            column: 1,
            width: 180.0,
        })
        .unwrap();

    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.format.column_widths[1], 180.0);
}

#[test]
fn apply_heading1_style_updates_paragraph() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    session
        .apply(Command::ApplyParagraphStyle {
            paragraph_id: para_id,
            style_name: "Heading 1".into(),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(para.style_id.is_some());
    assert_eq!(para.runs[0].format.bold, Some(true));
    assert_eq!(para.runs[0].format.font_size, Some(16.0));
}

#[test]
fn set_bullet_numbering_on_paragraph() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    session
        .apply(Command::SetNumbering {
            paragraph_id: para_id,
            numbering: Some(NumberingRef {
                numbering_id: 1,
                level: 0,
            }),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.format.numbering.unwrap().numbering_id, 1);
}

#[test]
fn insert_page_break_adds_paragraph_with_flag() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertPageBreak { after_block_id: after })
        .unwrap();

    let break_para = session.document.sections[0].blocks[1].paragraph().unwrap();
    assert_eq!(break_para.format.page_break_before, Some(true));
}
