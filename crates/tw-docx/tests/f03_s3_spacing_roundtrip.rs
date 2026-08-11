//! F03.S3 — character spacing and double underline DOCX round-trip.

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, CharFormat, Document, UnderlineStyle};

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
fn u_f03_s3_character_spacing_roundtrip_docx() {
    let mut doc = Document::with_paragraph("Wide");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.character_spacing = Some(2.0);
    }

    let exported = round_trip(&doc);
    assert_eq!(first_run_format(&exported).character_spacing, Some(2.0));
}

#[test]
fn u_f03_s3_double_underline_roundtrip_docx() {
    let mut doc = Document::with_paragraph("Double");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.underline = Some(UnderlineStyle::Double);
    }

    let exported = round_trip(&doc);
    assert_eq!(
        first_run_format(&exported).underline,
        Some(UnderlineStyle::Double)
    );
}
