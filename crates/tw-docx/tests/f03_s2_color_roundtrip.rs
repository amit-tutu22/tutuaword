//! F03.S2 — font color and highlight DOCX round-trip.

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, CharFormat, Color, Document};

fn round_trip(doc: &Document) -> Document {
    let package = DocxPackage::minimal();
    let bytes = export(doc, &package).unwrap();
    import(&bytes).unwrap().document
}

fn first_run_format(doc: &Document) -> CharFormat {
    doc.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .format
        .clone()
}

/// U-F03-S2-color-roundtrip-docx: import/export preserves font color ARGB.
#[test]
fn u_f03_s2_color_roundtrip_docx() {
    let mut doc = Document::with_paragraph("Red");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.color = Some(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        });
    }

    let exported = round_trip(&doc);
    assert_eq!(
        first_run_format(&exported).color,
        Some(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255
        })
    );
}

/// Yellow highlight survives DOCX export/import.
#[test]
fn u_f03_s2_highlight_roundtrip_docx() {
    let mut doc = Document::with_paragraph("Mark");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.highlight = Some(Color {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }

    let exported = round_trip(&doc);
    assert_eq!(
        first_run_format(&exported).highlight,
        Some(Color {
            r: 255,
            g: 255,
            b: 0,
            a: 255
        })
    );
}
