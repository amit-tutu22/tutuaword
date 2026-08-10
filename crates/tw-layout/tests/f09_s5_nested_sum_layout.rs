//! F09.S5 — nested table layout and SUM field evaluation (U-F09-S5-nested-layout).

use tw_layout::{LayoutBox, LayoutEngine, TableLayout};
use tw_model::{
    evaluate_field, sum_numeric_above, Block, Document, FieldData, FieldEvalContext, FieldType,
    Paragraph, Run, RunContent, Table, TableCell, TableRow,
};

fn cell_para(text: &str) -> TableCell {
    let mut cell = TableCell::new();
    cell.blocks = vec![Block::Paragraph(Paragraph::with_text(text))];
    cell
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
fn u_f09_s5_nested_table_layout() {
    let mut outer = Table::new(1, 1);
    let nested = Table::new(2, 2);
    outer.rows[0].cells[0].blocks = vec![
        Block::Paragraph(Paragraph::with_text("wrap")),
        Block::Table(nested),
    ];
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(outer)];

    let t = first_table(&doc);
    assert_eq!(t.cells.len(), 1);
    assert_eq!(t.cells[0].nested_tables.len(), 1);
    assert_eq!(t.cells[0].nested_tables[0].cells.len(), 4);
}

#[test]
fn u_f09_s5_sum_numeric_above() {
    let mut table = Table::new(4, 1);
    table.rows[0].cells[0] = cell_para("Header");
    table.rows[1].cells[0] = cell_para("10");
    table.rows[2].cells[0] = cell_para("20");
    table.rows[3].cells[0] = cell_para("total");

    assert!((sum_numeric_above(&table, 3, 0) - 30.0).abs() < f64::EPSILON);

    let ctx = FieldEvalContext::for_page(0, 1).with_table_sum_above(30.0);
    let text = evaluate_field(
        &FieldData {
            field_type: FieldType::TableSumAbove,
            instruction: Some(" =SUM(ABOVE) ".into()),
            display_text: None,
            form: None,
            merge_name: None,
        },
        &ctx,
    );
    assert_eq!(text, "30");
}

#[test]
fn u_f09_s5_sum_field_layout() {
    let mut table = Table::new(4, 1);
    table.rows[0].cells[0] = cell_para("Header");
    table.rows[1].cells[0] = cell_para("10");
    table.rows[2].cells[0] = cell_para("20");
    let mut sum_cell = TableCell::new();
    sum_cell.blocks = vec![Block::Paragraph({
        let mut para = Paragraph::new();
        para.runs = vec![Run {
            id: tw_model::NodeId::new(),
            format: Default::default(),
            content: RunContent::Field(FieldData {
                field_type: FieldType::TableSumAbove,
                instruction: Some(" =SUM(ABOVE) ".into()),
                display_text: None,
                form: None,
            merge_name: None,
            }),
            revision: None,
        }];
        para
    })];
    table.rows[3].cells[0] = sum_cell;

    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(table)];

    let t = first_table(&doc);
    let sum_cell = t.cells.last().expect("sum cell");
    let text: String = sum_cell
        .lines
        .iter()
        .flat_map(|l| l.glyphs.iter().map(|g| g.codepoint))
        .collect();
    assert!(text.contains('3'), "expected sum digits in layout, got {text}");
}
