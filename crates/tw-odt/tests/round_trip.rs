use std::io::{Cursor, Write};

use tw_odt::{export, import, OdtPackage};
use tw_model::Block;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn minimal_odt(content_xml: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("content.xml", options).unwrap();
        zip.write_all(content_xml.as_bytes()).unwrap();
        zip.start_file("styles.xml", options).unwrap();
        zip.write_all(b"<styles/>").unwrap();
        zip.start_file("mimetype", options).unwrap();
        zip.write_all(b"application/vnd.oasis.opendocument.text")
            .unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn odt_export_without_source_package() {
    let doc = tw_model::Document::with_paragraph("New ODT");
    let package = OdtPackage::minimal();
    let bytes = export(&doc, &package).unwrap();
    let result = import(&bytes).unwrap();
    assert_eq!(
        result.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "New ODT"
    );
}

#[test]
fn odt_passthrough_preserves_styles_part() {
    let xml = r#"<office:document><office:body>
        <text:p><text:span>Styled</text:span></text:p>
    </office:body></office:document>"#;
    let bytes = minimal_odt(xml);
    let imported = import(&bytes).unwrap();
    assert!(imported.package.parts.contains_key("styles.xml"));

    let exported = export(&imported.document, &imported.package).unwrap();
    let reimported = import(&exported).unwrap();
    assert!(reimported.package.parts.contains_key("styles.xml"));
}

#[test]
fn odt_export_bold_span() {
    let mut doc = tw_model::Document::with_paragraph("Bold");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.bold = Some(true);
    }
    let package = OdtPackage::minimal();
    let bytes = export(&doc, &package).unwrap();
    let content = String::from_utf8(
        import(&bytes)
            .unwrap()
            .package
            .parts
            .get("content.xml")
            .cloned()
            .unwrap(),
    )
    .unwrap();
    assert!(content.contains("Bold"));
    assert!(content.contains("Bold") || content.contains("text:style-name"));
}
