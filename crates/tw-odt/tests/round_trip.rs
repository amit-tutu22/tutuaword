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
    assert!(content.contains(r#"text:style-name="Bold""#));
    assert!(content.contains("automatic-styles"));
}

#[test]
fn u_f23_s3_odt_roundtrip() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
  <office:body><office:text>
    <text:p><text:span>Hello ODT</text:span></text:p>
  </office:text></office:body>
</office:document-content>"#;
    let bytes = minimal_odt(xml);
    let imported = import(&bytes).unwrap();
    assert_eq!(
        imported.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Hello ODT"
    );

    let mut doc = imported.document;
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs.clear();
        let mut bold = tw_model::Run::new_text("Edited");
        bold.format.bold = Some(true);
        let plain = tw_model::Run::new_text(" body");
        para.runs = vec![bold, plain];
    }

    let mut package = imported.package;
    package.mark_modified("content.xml".into());
    let exported = export(&doc, &package).unwrap();
    let reimported = import(&exported).unwrap();

    let para = reimported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    assert_eq!(para.full_text(), "Edited body");
    assert!(
        para.runs.iter().any(|r| r.format.bold == Some(true) && r.text() == "Edited"),
        "bold run must survive ODT rewrite round-trip"
    );
    assert!(
        para.runs.iter().any(|r| r.format.bold != Some(true) && r.text().contains("body")),
        "plain run must survive ODT rewrite round-trip"
    );

    let content = String::from_utf8_lossy(
        reimported
            .package
            .parts
            .get("content.xml")
            .expect("content.xml"),
    );
    assert!(content.contains(r#"text:style-name="Bold""#));
}
