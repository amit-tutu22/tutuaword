//! F09 — `document_plain_text` includes table cell text.

use tw_core::document_plain_text;
use tw_edit::{Command, EditSession};
use tw_model::{Block, NodeId};

fn table_cell_run_ids(session: &EditSession) -> (NodeId, NodeId) {
    let table = session
        .document
        .sections[0]
        .blocks
        .iter()
        .find_map(|b| match b {
            Block::Table(t) => Some(t),
            _ => None,
        })
        .expect("table block");
    let first = table.rows[0].cells[0]
        .blocks
        .iter()
        .find_map(|b| b.paragraph()?.runs.first().map(|r| r.id))
        .expect("first cell run");
    let second = table.rows[0].cells[1]
        .blocks
        .iter()
        .find_map(|b| b.paragraph()?.runs.first().map(|r| r.id))
        .expect("second cell run");
    (first, second)
}

#[test]
fn u_f09_document_plain_text_includes_table_cells() {
    let mut session = EditSession::new();
    session
        .apply(Command::InsertText {
            run_id: session.document.paragraph_at(0, 0).unwrap().runs[0].id,
            offset: 0,
            text: "Above table".into(),
        })
        .unwrap();

    let block_id = session.document.sections[0].blocks[0]
        .paragraph()
        .expect("paragraph block")
        .id;
    session
        .apply(Command::InsertTable {
            after_block_id: block_id,
            rows: 2,
            cols: 2,
        })
        .unwrap();

    let (cell_a, cell_b) = table_cell_run_ids(&session);
    session
        .apply(Command::InsertText {
            run_id: cell_a,
            offset: 0,
            text: "Item".into(),
        })
        .unwrap();
    session
        .apply(Command::InsertText {
            run_id: cell_b,
            offset: 0,
            text: "Quarter".into(),
        })
        .unwrap();

    let text = document_plain_text(&session.document);
    assert!(text.contains("Above table"), "body text: {text}");
    assert!(text.contains("Item"), "missing table cell Item in {text}");
    assert!(text.contains("Quarter"), "missing table cell Quarter in {text}");
}
