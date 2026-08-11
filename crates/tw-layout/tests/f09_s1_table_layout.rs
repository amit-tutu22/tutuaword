//! F09.S1 — 3×3 table layout (U-F09-S1-table-layout).

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, Table};

#[test]
fn u_f09_s1_table_layout_3x3() {
    let mut doc = Document::new();
    doc.sections[0].blocks.push(Block::Table(Table::new(3, 3)));

    let layout = LayoutEngine::new().layout_document(&doc);
    let table = layout
        .pages
        .iter()
        .flat_map(|page| &page.boxes)
        .find_map(|item| match item {
            LayoutBox::Table(t) => Some(t),
            _ => None,
        })
        .expect("3×3 table should produce a Table layout box");

    assert_eq!(table.cells.len(), 9);
    assert!(!table.grid_lines.is_empty());
}
