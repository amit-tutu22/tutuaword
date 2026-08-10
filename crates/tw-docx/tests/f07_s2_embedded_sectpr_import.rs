//! F07.S2 — embedded `w:pPr/w:sectPr` creates multiple sections with landscape + HF.

use tw_docx::import;
use tw_model::{Block, HeaderFooterType};

fn minimal_docx(document_xml: &str, extra: &[(&str, &str)]) -> Vec<u8> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        if !extra.iter().any(|(name, _)| *name == "word/_rels/document.xml.rels") {
            zip.start_file("word/_rels/document.xml.rels", options)
                .unwrap();
            zip.write_all(b"<Relationships/>").unwrap();
        }
        for (name, body) in extra {
            zip.start_file(*name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn u_f07_s2_embedded_sectpr_splits_sections_and_binds_hf() {
    let document = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>
  <w:p><w:r><w:t>Section one</w:t></w:r></w:p>
  <w:p><w:pPr><w:sectPr>
    <w:pgSz w:w="15840" w:h="12240" w:orient="landscape"/>
    <w:headerReference w:type="default" r:id="rId7"/>
    <w:footerReference w:type="default" r:id="rId8"/>
  </w:sectPr></w:pPr><w:r><w:t>Break</w:t></w:r></w:p>
  <w:p><w:r><w:t>Section two</w:t></w:r></w:p>
  <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
</w:body></w:document>"#;
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>
  <Relationship Id="rId8" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>
</Relationships>"#;
    let header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p><w:r><w:t>Landscape Header</w:t></w:r></w:p>
</w:hdr>"#;
    let footer = r#"<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p><w:r><w:t>Landscape Footer</w:t></w:r></w:p>
</w:ftr>"#;
    let bytes = minimal_docx(
        document,
        &[
            ("word/_rels/document.xml.rels", rels),
            ("word/header1.xml", header),
            ("word/footer1.xml", footer),
        ],
    );

    let doc = import(&bytes).unwrap().document;
    assert!(
        doc.sections.len() >= 2,
        "expected multiple sections, got {}",
        doc.sections.len()
    );
    assert!(
        doc.sections[1].format.is_landscape(),
        "second section should be landscape"
    );

    let header_text = doc
        .resolved_header(1, HeaderFooterType::Default)
        .and_then(|hf| hf.blocks.first()?.paragraph().map(|p| p.full_text()))
        .unwrap_or_default();
    assert!(
        header_text.contains("Landscape Header"),
        "embedded sectPr HF binding missing, got {header_text:?}"
    );

    let footer_text = doc
        .resolved_footer(1, HeaderFooterType::Default)
        .and_then(|hf| hf.blocks.first()?.paragraph().map(|p| p.full_text()))
        .unwrap_or_default();
    assert!(
        footer_text.contains("Landscape Footer"),
        "embedded sectPr footer binding missing, got {footer_text:?}"
    );

    let section_two = doc.sections[1]
        .blocks
        .iter()
        .find_map(|b| b.paragraph().map(|p| p.full_text()))
        .unwrap_or_default();
    assert_eq!(section_two, "Section two");

    let _ = doc.sections[0]
        .blocks
        .iter()
        .any(|b| matches!(b, Block::Paragraph(p) if p.full_text().contains("Section one")));
}
