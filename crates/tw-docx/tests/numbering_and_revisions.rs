//! Unit tests for numbering.xml export and wrapper-level track-change import.

use std::io::{Cursor, Write};

use tw_docx::{export, import, DocxPackage};
use tw_model::{Document, RevisionType};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn minimal_docx(document_xml: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options)
            .unwrap();
        zip.write_all(b"<Relationships/>").unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn numbering_xml_round_trips_through_export_and_import() {
    let doc = Document::with_paragraph("List item");
    let bytes = export(&doc, &DocxPackage::minimal()).unwrap();
    let imported = import(&bytes).unwrap();
    assert!(
        imported.package.parts.contains_key("word/numbering.xml"),
        "export must write numbering.xml"
    );
    assert!(imported.document.settings.numbering.get(1).is_some());
    assert!(imported.document.settings.numbering.get(2).is_some());
    assert_eq!(
        imported
            .document
            .settings
            .numbering
            .get(2)
            .unwrap()
            .levels[0]
            .format,
        tw_model::ListMarkerFormat::Decimal
    );
}

#[test]
fn fresh_export_includes_bullet_and_numbered_definitions() {
    let doc = Document::with_paragraph("x");
    let imported = import(&export(&doc, &DocxPackage::minimal()).unwrap()).unwrap();
    assert!(imported.document.settings.numbering.get(1).is_some());
    assert!(imported.document.settings.numbering.get(2).is_some());
}

#[test]
fn wrapper_ins_and_del_import_as_revision_runs() {
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:ins w:id="1" w:author="Alice"><w:r><w:t>Added</w:t></w:r></w:ins><w:del w:id="2" w:author="Bob"><w:r><w:t>Removed</w:t></w:r></w:del></w:p></w:body></w:document>"#;
    let imported = import(&minimal_docx(xml)).unwrap();
    let para = imported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    assert_eq!(para.runs.len(), 2);
    assert_eq!(para.runs[0].text(), "Added");
    assert_eq!(para.runs[1].text(), "Removed");
    assert!(matches!(
        para.runs[0].revision.as_ref().map(|r| r.revision_type),
        Some(RevisionType::Insert)
    ));
    assert!(matches!(
        para.runs[1].revision.as_ref().map(|r| r.revision_type),
        Some(RevisionType::Delete)
    ));
}

#[test]
fn exported_revision_ids_are_numeric_not_uuid() {
    let mut doc = Document::with_paragraph("Rev");
    if let tw_model::Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].revision = Some(tw_model::Revision::insert("Author"));
    }
    let bytes = export(&doc, &DocxPackage::minimal()).unwrap();
    let document_xml = String::from_utf8(
        import(&bytes)
            .unwrap()
            .package
            .parts["word/document.xml"]
            .clone(),
    )
    .unwrap();
    assert!(document_xml.contains("<w:ins "));
    let id_fragment = document_xml
        .split("<w:ins ")
        .nth(1)
        .and_then(|s| s.split('"').nth(1))
        .expect("w:id attribute");
    assert!(
        id_fragment.chars().all(|c| c.is_ascii_digit()),
        "revision w:id should be numeric, got {id_fragment}"
    );
}
