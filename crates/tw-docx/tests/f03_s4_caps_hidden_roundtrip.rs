//! F03.S4 — caps and hidden DOCX round-trip.

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, CharFormat, Document};

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

#[test]
fn u_f03_s4_caps_roundtrip_docx() {
    let mut doc = Document::with_paragraph("Caps");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.all_caps = Some(true);
        para.runs[0].format.small_caps = Some(false);
    }

    let exported = round_trip(&doc);
    let format = first_run_format(&exported);
    assert_eq!(format.all_caps, Some(true));
}

#[test]
fn u_f03_s4_small_caps_roundtrip_docx() {
    let mut doc = Document::with_paragraph("Small");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.small_caps = Some(true);
    }

    let exported = round_trip(&doc);
    assert_eq!(first_run_format(&exported).small_caps, Some(true));
}

#[test]
fn u_f03_s4_hidden_roundtrip_docx() {
    let mut doc = Document::with_paragraph("Hide");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.hidden = Some(true);
    }

    let exported = round_trip(&doc);
    assert_eq!(first_run_format(&exported).hidden, Some(true));
}
