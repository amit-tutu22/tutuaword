use std::io::{Cursor, Write};

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, CharFormat, Revision};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn minimal_docx(document_xml: &str, extra_parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options).unwrap();
        zip.write_all(b"<Relationships/>").unwrap();
        for (name, data) in extra_parts {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn export_without_source_package_builds_valid_docx() {
    let mut doc = tw_model::Document::with_paragraph("Fresh export");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format = CharFormat {
            bold: Some(true),
            ..Default::default()
        };
    }
    let package = DocxPackage::minimal();
    let bytes = export(&doc, &package).unwrap();
    let result = import(&bytes).unwrap();
    assert_eq!(
        result.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Fresh export"
    );
}

#[test]
fn passthrough_preserves_extra_parts_on_unmodified_export() {
    let xml = r#"<w:document><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;
    let theme = b"<theme name=\"Office\"/>";
    let bytes = minimal_docx(xml, &[("word/theme/theme1.xml", theme)]);
    let imported = import(&bytes).unwrap();
    assert!(imported.package.parts.contains_key("word/theme/theme1.xml"));

    let exported = export(&imported.document, &imported.package).unwrap();
    let reimported = import(&exported).unwrap();
    assert!(reimported
        .package
        .parts
        .contains_key("word/theme/theme1.xml"));
}

#[test]
fn export_serializes_track_changes_markup() {
    let mut doc = tw_model::Document::with_paragraph("Changed");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::insert("Reviewer"));
    }
    let package = DocxPackage::minimal();
    let bytes = export(&doc, &package).unwrap();
    let xml = String::from_utf8(
        import(&bytes)
            .unwrap()
            .package
            .parts
            .get("word/document.xml")
            .cloned()
            .unwrap(),
    )
    .unwrap();
    assert!(xml.contains("<w:ins"));
    assert!(xml.contains("Reviewer"));
}

#[test]
fn modified_document_xml_updates_on_edit_export() {
    let xml = r#"<w:document><w:body><w:p><w:r><w:t>Original</w:t></w:r></w:p></w:body></w:document>"#;
    let bytes = minimal_docx(xml, &[]);
    let imported = import(&bytes).unwrap();
    let mut doc = imported.document.clone();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        if let Some(text) = para.runs[0].text_mut() {
            *text = "Updated".into();
        }
    }
    let mut package = imported.package.clone();
    package.mark_modified("word/document.xml".into());
    let exported = export(&doc, &package).unwrap();
    let roundtrip = import(&exported).unwrap();
    assert_eq!(
        roundtrip.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Updated"
    );
}
