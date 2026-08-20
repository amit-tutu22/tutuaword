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

#[test]
fn u_f09_table_arrow_probe_crosses_empty_cells() {
    let mut doc = Document::new();
    doc.sections[0]
        .blocks
        .push(Block::Table(Table::new(2, 3)));

    let mut engine = LayoutEngine::new();
    engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");

    let left = cell_run_id(&doc, 0, 0);
    let mid = cell_run_id(&doc, 0, 1);
    let (x0, y0, height) = map.caret_at(left, 0).expect("caret cell 0");

    // Mid-cell probe (well inside the neighbor column) must leave cell 0.
    let (cx, _) = cell_center(&engine, 3, 0, 1);
    let hit = map
        .hit_test(cx, y0)
        .expect("center of cell 1 must hit");
    assert_eq!(hit.run_id, mid, "cell center hit must be the mid column");

    // Small Right-arrow-style probes must also cross once past cell 0's width.
    let mut found_mid = false;
    for d in [2.0f32, 24.0, 60.0, 110.0, 180.0] {
        let hx = x0 + d;
        if let Some(h) = map.hit_test(hx, y0) {
            if h.run_id == mid {
                found_mid = true;
                break;
            }
        }
    }
    assert!(found_mid, "Right-arrow probes from cell 0 must reach cell 1");

    let below = cell_run_id(&doc, 1, 0);
    let step = height.max(15.4) * 1.5;
    let hit_down = map.hit_test(x0, y0 + step).expect("down hit");
    assert_eq!(
        hit_down.run_id, below,
        "Down from row0 col0 should land row1 col0"
    );
}
