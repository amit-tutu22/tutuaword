//! Table cell hit-testing — caret must land in the clicked column, not the leftmost.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, Paragraph, Table};

fn cell_run_id(doc: &Document, row: usize, col: usize) -> tw_model::NodeId {
    let table = doc.sections[0].blocks[1].table().expect("table");
    table.rows[row].cells[col].blocks[0]
        .paragraph()
        .expect("cell para")
        .runs[0]
        .id
}

fn cell_center(engine: &LayoutEngine, cols: usize, row: usize, col: usize) -> (f32, f32) {
    let page = engine.page_layout(0).expect("page");
    let table = page
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::Table(t) => Some(t),
            _ => None,
        })
        .expect("table layout");
    let idx = row * cols + col;
    let cell = &table.cells[idx];
    (cell.x + cell.width * 0.5, cell.y + cell.height * 0.5)
}

#[test]
fn u_f09_table_hit_test_each_empty_cell() {
    let mut doc = Document::new();
    doc.sections[0]
        .blocks
        .push(Block::Table(Table::new(1, 3)));

    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");

    for col in 0..3 {
        let (x, y) = cell_center(&engine, 3, 0, col);
        let hit = map
            .hit_test(x, y)
            .unwrap_or_else(|| panic!("no hit in cell col={col} at ({x},{y})"));
        assert_eq!(
            hit.run_id,
            cell_run_id(&doc, 0, col),
            "click in column {col} must resolve that cell's run"
        );
    }
}

#[test]
fn u_f09_table_hit_test_filled_cells() {
    let mut doc = Document::new();
    let mut table = Table::new(1, 3);
    for (col, text) in ["AA", "BB", "CC"].iter().enumerate() {
        table.rows[0].cells[col].blocks = vec![Block::Paragraph(Paragraph::with_text(*text))];
    }
    doc.sections[0].blocks.push(Block::Table(table));

    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");

    for col in 0..3 {
        let (x, y) = cell_center(&engine, 3, 0, col);
        let hit = map
            .hit_test(x, y)
            .unwrap_or_else(|| panic!("no hit in filled cell col={col}"));
        assert_eq!(
            hit.run_id,
            cell_run_id(&doc, 0, col),
            "filled column {col} must resolve its own run"
        );
    }
}

#[test]
fn u_f09_table_caret_at_non_left_cell() {
    let mut doc = Document::new();
    let mut table = Table::new(1, 3);
    table.rows[0].cells[1].blocks = vec![Block::Paragraph(Paragraph::with_text("Mid"))];
    doc.sections[0].blocks.push(Block::Table(table));

    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");

    let mid_run = cell_run_id(&doc, 0, 1);
    let left_run = cell_run_id(&doc, 0, 0);
    let (cx, _, _) = map.caret_at(mid_run, 0).expect("caret in middle cell");
    let (lx, _, _) = map.caret_at(left_run, 0).expect("caret in left cell");
    assert!(
        cx > lx + 10.0,
        "middle cell caret x ({cx}) should be right of left cell ({lx})"
    );
}
