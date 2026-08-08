//! F09.S3 — merged cell colspan/rowspan in layout (U-F09-S3-merge-cells-span).

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, Table};

#[test]
fn u_f09_s3_merge_cells_layout_span() {
    let mut doc = Document::new();
    let mut table = Table::new(2, 2);
    table.rows[0].cells[0].format.colspan = 2;
    doc.sections[0].blocks.push(Block::Table(table));

    let layout = LayoutEngine::new().layout_document(&doc);
    let table_layout = layout
        .pages
        .iter()
        .flat_map(|page| &page.boxes)
        .find_map(|item| match item {
            LayoutBox::Table(t) => Some(t),
            _ => None,
        })
        .expect("merged table layout");

    assert_eq!(table_layout.cells.len(), 3, "one 2-col span + two row-1 cells");
    let merged = table_layout
        .cells
        .iter()
        .max_by(|a, b| a.width.partial_cmp(&b.width).unwrap())
        .expect("merged cell");
    let narrow = table_layout
        .cells
        .iter()
        .min_by(|a, b| a.width.partial_cmp(&b.width).unwrap())
        .expect("single column cell");
    assert!(
        merged.width > narrow.width * 1.5,
        "merged cell width {} should exceed single column {}",
        merged.width,
        narrow.width
    );
}
