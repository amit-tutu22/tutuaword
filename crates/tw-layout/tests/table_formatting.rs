use tw_layout::{LayoutBox, LayoutEngine, TableLayout};
use tw_model::{Block, Document, Paragraph, Table, TableCell, TableRow};

fn cell_with(text: &str) -> TableCell {
    let mut cell = TableCell::new();
    cell.blocks = vec![Block::Paragraph(Paragraph::with_text(text.to_string()))];
    cell
}

fn table_from(rows: Vec<Vec<&str>>, column_widths: Vec<f32>) -> Table {
    let mut table = Table::new(rows.len() as u32, column_widths.len() as u32);
    table.rows = rows
        .into_iter()
        .map(|r| TableRow::with_cells(r.into_iter().map(cell_with).collect()))
        .collect();
    table.format.width = Some(column_widths.iter().sum());
    table.format.column_widths = column_widths;
    table
}

fn tables_of(doc: &Document) -> (Vec<TableLayout>, Vec<(f32, f32)>) {
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(doc);
    let mut tables = Vec::new();
    let mut page_bounds = Vec::new();
    for page in &layout.pages {
        for b in &page.boxes {
            if let LayoutBox::Table(t) = b {
                tables.push(t.clone());
                page_bounds.push((page.content_left, page.content_left + page.content_width));
            }
        }
    }
    (tables, page_bounds)
}

#[test]
fn oversized_column_widths_are_scaled_to_the_page() {
    // 6 columns of 150pt = 900pt against a 468pt text column.
    let table = table_from(
        vec![vec!["a", "b", "c", "d", "e", "f"]],
        vec![150.0; 6],
    );
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let (tables, bounds) = tables_of(&doc);
    let t = &tables[0];
    let (left, right) = bounds[0];

    assert!(
        t.width <= right - left + 0.5,
        "table width {} should fit the {}pt column",
        t.width,
        right - left
    );
    for cell in &t.cells {
        assert!(
            cell.x >= left - 0.5 && cell.x + cell.width <= right + 0.5,
            "cell [{}..{}] escapes the text column [{}..{}]",
            cell.x,
            cell.x + cell.width,
            left,
            right
        );
    }
}

#[test]
fn rows_grow_to_fit_wrapped_cell_text() {
    let long = "This cell holds a long sentence that must wrap onto several \
                lines inside a narrow column and therefore make the row taller.";
    let table = table_from(vec![vec![long, "short"]], vec![120.0, 120.0]);
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let (tables, _) = tables_of(&doc);
    let t = &tables[0];

    let wrapped = t
        .cells
        .iter()
        .max_by_key(|c| c.lines.len())
        .expect("a cell");
    assert!(
        wrapped.lines.len() > 1,
        "expected the long cell to wrap, got {} line(s)",
        wrapped.lines.len()
    );
    assert!(
        t.height >= wrapped.lines.len() as f32 * 10.0,
        "row height {} does not cover {} wrapped lines",
        t.height,
        wrapped.lines.len()
    );

    // Every glyph must stay inside its own cell band.
    for cell in &t.cells {
        for line in &cell.lines {
            assert!(
                line.y >= cell.y && line.y <= cell.y + cell.height + 1.0,
                "line at y={} escapes cell band [{}..{}]",
                line.y,
                cell.y,
                cell.y + cell.height
            );
        }
    }
}

#[test]
fn tall_table_splits_across_pages_without_overflowing() {
    let rows: Vec<Vec<&str>> = (0..60)
        .map(|_| vec!["Row content that occupies a line", "second column"])
        .collect();
    let table = table_from(rows, vec![200.0, 200.0]);
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    assert!(
        layout.pages.len() > 1,
        "a 60-row table should span multiple pages"
    );

    for page in &layout.pages {
        let bottom = page.content_top + page.content_height;
        for b in &page.boxes {
            if let LayoutBox::Table(t) = b {
                assert!(
                    t.y + t.height <= bottom + 1.0,
                    "table slice on page {} ends at {} past the {} bottom margin",
                    page.page_index,
                    t.y + t.height,
                    bottom
                );
            }
        }
    }
}

#[test]
fn spanning_cell_covers_exactly_its_columns() {
    let mut table = table_from(vec![vec!["wide"], vec!["a", "b"]], vec![100.0, 100.0]);
    table.rows[0].cells[0].format.colspan = 2;
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let (tables, _) = tables_of(&doc);
    let t = &tables[0];

    assert_eq!(t.cells.len(), 3, "one spanning cell plus two normal cells");
    let spanning = &t.cells[0];
    assert!(
        (spanning.width - t.width).abs() < 0.5,
        "spanning cell width {} should equal table width {}",
        spanning.width,
        t.width
    );
    // The second row must start back at the table's left edge.
    assert!((t.cells[1].x - t.x).abs() < 0.5);
}
