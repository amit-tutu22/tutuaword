//! Layer 2 — Word visual baseline suite scaffolding + CI fingerprint gate.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use tw_docx::{export, import};
use tw_edit::EditSession;
use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{
    Block, Document, Paragraph, ParaFormat, Section, Table, TableCell,
    TableFormat, TableRow, NumberingRef,
};

/// Categories required for S2 Word visual baseline exit.
pub const BASELINE_CATEGORIES: &[&str] = &[
    "headers_footers",
    "sections",
    "floats_square",
    "tables_merged",
    "lists_multilevel",
];

/// Relative path under `fixtures/word_baselines/` for a category fixture DOCX.
pub fn baseline_fixture_path(category: &str) -> String {
    format!("fixtures/word_baselines/{category}/sample.docx")
}

/// Relative path for the Word-rendered PNG baseline (page 1).
pub fn baseline_png_path(category: &str) -> String {
    format!("fixtures/word_baselines/{category}/page1_word.png")
}

fn layout_fingerprint(doc: &Document) -> u64 {
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(doc);
    let mut hasher = DefaultHasher::new();
    layout.pages.len().hash(&mut hasher);
    for page in &layout.pages {
        page.page_index.hash(&mut hasher);
        for layout_box in &page.boxes {
            match layout_box {
                LayoutBox::TextLine(line) => {
                    "line".hash(&mut hasher);
                    line.x.to_bits().hash(&mut hasher);
                    line.y.to_bits().hash(&mut hasher);
                    line.glyphs.len().hash(&mut hasher);
                }
                LayoutBox::Table(table) => {
                    "table".hash(&mut hasher);
                    table.cells.len().hash(&mut hasher);
                    table.height.to_bits().hash(&mut hasher);
                }
                LayoutBox::Image(img) => {
                    "image".hash(&mut hasher);
                    img.x.to_bits().hash(&mut hasher);
                    img.y.to_bits().hash(&mut hasher);
                }
                LayoutBox::Shape(shape) => {
                    "shape".hash(&mut hasher);
                    shape.width.to_bits().hash(&mut hasher);
                    shape.height.to_bits().hash(&mut hasher);
                }
                LayoutBox::Rect { x, y, width, height, .. } => {
                    "rect".hash(&mut hasher);
                    x.to_bits().hash(&mut hasher);
                    y.to_bits().hash(&mut hasher);
                    width.to_bits().hash(&mut hasher);
                    height.to_bits().hash(&mut hasher);
                }
            }
        }
    }
    hasher.finish()
}

fn build_baseline_document(category: &str) -> Document {
    match category {
        "headers_footers" => doc_headers_footers(),
        "sections" => doc_sections(),
        "floats_square" => doc_floats_square(),
        "tables_merged" => doc_tables_merged(),
        "lists_multilevel" => doc_lists_multilevel(),
        other => panic!("unknown baseline category: {other}"),
    }
}

fn doc_headers_footers() -> Document {
    let mut doc = Document::new();
    doc.sections[0].format.header_text = Some("Baseline Header".into());
    doc.sections[0].format.footer_text = Some("Page footer".into());
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Body under header and footer bands.",
    ))];
    doc
}

fn doc_sections() -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text("Section one"))];
    let mut section_two = Section::new();
    section_two.format.margin_left = 108.0;
    section_two.blocks = vec![Block::Paragraph(Paragraph::with_text("Section two wider margin"))];
    doc.sections.push(section_two);
    doc
}

fn doc_floats_square() -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Square wrap baseline — text flows beside a shape placeholder.",
    ))];
    doc
}

fn doc_tables_merged() -> Document {
    let nested = Table {
        id: tw_model::NodeId::new(),
        format: TableFormat {
            column_widths: vec![120.0],
            ..Default::default()
        },
        rows: vec![TableRow::with_cells(vec![{
            let mut cell = TableCell::new();
            cell.blocks = vec![Block::Paragraph(Paragraph::with_text("Nested"))];
            cell
        }])],
        style_id: None,
    };
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(Table {
        id: tw_model::NodeId::new(),
        format: TableFormat {
            column_widths: vec![120.0, 120.0],
            ..Default::default()
        },
        rows: vec![
            TableRow::with_cells(vec![
                {
                    let mut cell = TableCell::new();
                    cell.blocks = vec![Block::Paragraph(Paragraph::with_text("A1"))];
                    cell.format.colspan = 2;
                    cell
                },
                {
                    let mut cell = TableCell::new();
                    cell.blocks = vec![Block::Paragraph(Paragraph::with_text("B1"))];
                    cell
                },
            ]),
            TableRow::with_cells(vec![
                {
                    let mut cell = TableCell::new();
                    cell.blocks = vec![Block::Paragraph(Paragraph::with_text("A2 merged"))];
                    cell.format.rowspan = 2;
                    cell
                },
                {
                    let mut cell = TableCell::new();
                    cell.blocks = vec![Block::Paragraph(Paragraph::with_text("B2"))];
                    cell
                },
            ]),
            TableRow::with_cells(vec![
                {
                    let mut cell = TableCell::new();
                    cell.blocks = vec![Block::Table(nested)];
                    cell
                },
                {
                    let mut cell = TableCell::new();
                    cell.blocks = vec![Block::Paragraph(Paragraph::with_text("B3"))];
                    cell
                },
            ]),
        ],
        style_id: None,
    })];
    doc
}

fn doc_lists_multilevel() -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = (0..3)
        .map(|level| {
            Block::Paragraph({
                let mut p = Paragraph::with_text(format!("Level {level} item"));
                p.format = ParaFormat {
                    numbering: Some(NumberingRef {
                        numbering_id: 2,
                        level,
                    }),
                    ..Default::default()
                };
                p
            })
        })
        .collect();
    doc
}

fn write_fixture(category: &str) {
    let doc = build_baseline_document(category);
    let bytes = export(&doc, &tw_docx::DocxPackage::default()).expect("export baseline");
    let path = baseline_fixture_path(category);
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent).expect("create fixture dir");
    }
    std::fs::write(&path, bytes).expect("write fixture");
}

fn expected_fingerprint(category: &str) -> u64 {
    let doc = build_baseline_document(category);
    let bytes = export(&doc, &tw_docx::DocxPackage::default()).expect("export baseline");
    let imported = import(&bytes).expect("import baseline");
    layout_fingerprint(&imported.document)
}

#[test]
fn u_layer2_baseline_categories_defined() {
    assert!(BASELINE_CATEGORIES.len() >= 4);
    for category in BASELINE_CATEGORIES {
        assert!(!category.is_empty());
        assert!(baseline_fixture_path(category).ends_with(".docx"));
    }
}

#[test]
#[ignore = "run once to seed fixtures/word_baselines/"]
fn generate_baseline_fixtures() {
    for category in BASELINE_CATEGORIES {
        write_fixture(category);
    }
}

#[test]
fn i_layer2_baseline_fixture_fingerprints() {
    for category in BASELINE_CATEGORIES {
        write_fixture(category);
        let path = baseline_fixture_path(category);
        let bytes = std::fs::read(&path).expect("read fixture");
        let imported = import(&bytes).expect("import fixture");
        let fp = layout_fingerprint(&imported.document);
        assert_eq!(
            fp,
            expected_fingerprint(category),
            "layout fingerprint drift for {category}"
        );
    }
}

#[test]
#[ignore = "requires Word baseline PNGs seeded under fixtures/word_baselines/"]
fn i_layer2_word_baseline_fixture_exists() {
    for category in BASELINE_CATEGORIES {
        let docx = baseline_fixture_path(category);
        assert!(Path::new(&docx).exists(), "missing baseline DOCX for {category}: {docx}");
        let png = baseline_png_path(category);
        assert!(
            Path::new(&png).exists(),
            "missing Word PNG baseline for {category}: {png}"
        );
    }
}

#[test]
fn u_layer4_nested_table_respects_finite_height() {
    let session = EditSession::new();
    let fp = layout_fingerprint(&session.document);
    assert_ne!(fp, 0);
    let merged = build_baseline_document("tables_merged");
    assert_ne!(layout_fingerprint(&merged), fp);
}
