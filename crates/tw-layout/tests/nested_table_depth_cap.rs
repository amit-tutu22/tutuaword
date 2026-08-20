//! Nested table depth cap (crash/perf Phase 5).

use tw_layout::{LayoutBox, LayoutEngine, MAX_TABLE_NEST_DEPTH};
use tw_model::{Block, Document, Paragraph, Table};

fn nest_tables(depth: u32) -> Table {
    let mut table = Table::new(1, 1);
    table.format.column_widths = vec![200.0];
    if depth == 0 {
        table.rows[0].cells[0].blocks =
            vec![Block::Paragraph(Paragraph::with_text("leaf"))];
    } else {
        table.rows[0].cells[0].blocks = vec![Block::Table(nest_tables(depth - 1))];
    }
    table
}

fn nested_table_count(t: &tw_layout::TableLayout) -> usize {
    let mut n = 1;
    for cell in &t.cells {
        for nested in &cell.nested_tables {
            n += nested_table_count(nested);
        }
    }
    n
}

#[test]
fn nested_table_depth_is_capped() {
    let deep = nest_tables(MAX_TABLE_NEST_DEPTH + 4);
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(deep)];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = layout.pages.first().expect("page");
    let root = page
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::Table(t) => Some(t),
            _ => None,
        })
        .expect("root table");
    let tables = nested_table_count(root);
    assert!(
        tables <= MAX_TABLE_NEST_DEPTH as usize,
        "expected at most {MAX_TABLE_NEST_DEPTH} nested tables, got {tables}"
    );
}
