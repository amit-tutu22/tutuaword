//! F21.S3 — DOCX `descr` alt-text round-trip.

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, Document, ImageBlock, ImageData, ImageTransform, TextWrap};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn package_part(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    use std::io::{Cursor, Read};
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;
    Some(data)
}

#[test]
fn u_f21_s3_docx_descr_roundtrip() {
    let mut doc = Document::new();
    let image = ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 72.0,
        display_height: 72.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: Some("A red apple".into()),
            wrap_polygon: None,
    };
    doc.sections[0].blocks = vec![Block::ImageBlock(image)];

    let bytes = export(&doc, &DocxPackage::minimal()).expect("export");
    let document_xml = package_part(&bytes, "word/document.xml").expect("document.xml");
    let xml = String::from_utf8_lossy(&document_xml);
    assert!(
        xml.contains(r#"descr="A red apple""#),
        "export must emit wp:docPr descr: {xml}"
    );

    let imported = import(&bytes).expect("import");
    let image = imported.document.sections[0].blocks[0]
        .image()
        .expect("image block");
    assert_eq!(image.alt_text.as_deref(), Some("A red apple"));
}
