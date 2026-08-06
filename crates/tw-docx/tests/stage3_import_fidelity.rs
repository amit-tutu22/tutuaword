use tw_docx::import;
use tw_model::Block;

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
fn paragraph_default_run_properties_apply_to_runs() {
    let xml = r#"<w:document><w:body>
        <w:p><w:pPr><w:rPr><w:b/><w:i/></w:rPr></w:pPr>
        <w:r><w:t>Styled</w:t></w:r></w:p>
    </w:body></w:document>"#;
    let doc = import(&minimal_docx(xml, &[])).unwrap().document;
    let para = doc.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.runs[0].format.bold, Some(true));
    assert_eq!(para.runs[0].format.italic, Some(true));
}

#[test]
fn rich_header_blocks_import_from_header_part() {
    let document = r#"<w:document><w:body>
        <w:p><w:r><w:t>Body</w:t></w:r></w:p>
        <w:sectPr><w:headerReference w:type="default" r:id="rId7"/></w:sectPr>
    </w:body></w:document>"#;
    let rels = r#"<Relationships>
        <Relationship Id="rId7" Type="header" Target="header1.xml"/>
    </Relationships>"#;
    let header = r#"<w:hdr><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Header Title</w:t></w:r></w:p></w:hdr>"#;
    let bytes = minimal_docx(
        document,
        &[
            ("word/_rels/document.xml.rels", rels),
            ("word/header1.xml", header),
        ],
    );
    let doc = import(&bytes).unwrap().document;
    assert_eq!(doc.sections[0].format.header_blocks.len(), 1);
    let header_para = doc.sections[0].format.header_blocks[0]
        .paragraph()
        .unwrap();
    assert_eq!(header_para.full_text(), "Header Title");
    assert_eq!(header_para.runs[0].format.bold, Some(true));
}

#[test]
fn table_cell_shading_and_rowspan_import() {
    let xml = r#"<w:document><w:body><w:tbl>
        <w:tr><w:tc><w:tcPr><w:vMerge w:val="restart"/><w:shd w:fill="FF0000"/></w:tcPr>
            <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>
        <w:tr><w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p/></w:tc>
        <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc></w:tr>
    </w:tbl></w:body></w:document>"#;
    let doc = import(&minimal_docx(xml, &[])).unwrap().document;
    let table = match &doc.sections[0].blocks[0] {
        Block::Table(t) => t,
        _ => panic!("expected table"),
    };
    assert_eq!(table.rows[0].cells[0].format.rowspan, 2);
    assert_eq!(table.rows[0].cells[0].format.background.map(|c| c.r), Some(255));
    assert_eq!(table.rows[1].cells.len(), 1);
}
