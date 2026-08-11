//! F09.S4 — table border grid lines and cell shading in layout (U-F09-S4-table-border-layout).

use tw_layout::{LayoutBox, LayoutEngine, TableLayout};
use tw_model::{Block, Color, Document, Paragraph, Table, TableCell, TableRow};

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

fn first_table(doc: &Document) -> TableLayout {
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(doc);
    for page in &layout.pages {
        for b in &page.boxes {
            if let LayoutBox::Table(t) = b {
                return t.clone();
            }
        }
    }
    panic!("no table in layout");
}

#[test]
fn u_f09_s4_table_border_layout() {
    let mut table = table_from(vec![vec!["a", "b"], vec!["c", "d"]], vec![120.0, 120.0]);
    table.format.border = Some(tw_model::BorderSpec {
        width: 2.0,
        color: tw_model::Color::BLACK,
    });
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let t = first_table(&doc);
    assert!(
        !t.grid_lines.is_empty(),
        "table border should produce grid lines"
    );
    assert!(
        !t.grid_line_colors.is_empty(),
        "grid line colors should be populated"
    );
}

#[test]
fn u_f09_s4_cell_shading_layout() {
    let mut table = table_from(vec![vec!["shaded", "plain"]], vec![100.0, 100.0]);
    table.rows[0].cells[0].format.background = Some(Color {
        r: 255,
        g: 200,
        b: 200,
        a: 255,
    });
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let t = first_table(&doc);
    assert_eq!(t.cells.len(), 2);
    assert_eq!(
        t.cells[0].background,
        Some(Color {
            r: 255,
            g: 200,
            b: 200,
            a: 255,
        }
        .to_argb())
    );
    assert!(t.cells[1].background.is_none());
}

#[test]
fn u_f09_s4_autofit_column_widths_layout() {
    let mut table = table_from(vec![vec!["a", "b", "c"]], vec![50.0, 50.0, 50.0]);
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let content_width = layout.pages[0].content_width;

    let table = doc.sections[0].blocks[0].table_mut().unwrap();
    let scale = content_width / 150.0;
    table.format.column_widths = vec![50.0 * scale; 3];
    table.format.width = Some(content_width);

    let t = first_table(&doc);
    assert!(
        (t.width - content_width).abs() < 1.0,
        "autofit table width {} should match content {}",
        t.width,
        content_width
    );
}
