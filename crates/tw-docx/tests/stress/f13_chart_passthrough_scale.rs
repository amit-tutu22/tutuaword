//! Stress: fifty chart parts survive import/export passthrough.

use std::io::{Cursor, Write};

use tw_docx::{export, import};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const CHART_BODY: &str = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="{rel}"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

const CHART_XML: &str = r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart/></c:chartSpace>"#;

fn build_docx(count: usize) -> Vec<u8> {
    let mut body = String::new();
    let mut rels = String::from(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    let mut types = String::from(
        r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
    );

    for i in 1..=count {
        let rel = format!("rId{i}");
        body.push_str(&CHART_BODY.replace("{rel}", &rel));
        rels.push_str(&format!(
            r#"<Relationship Id="{rel}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart" Target="charts/chart{i}.xml"/>"#
        ));
        types.push_str(&format!(
            r#"<Override PartName="/word/charts/chart{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.drawingml.chart+xml"/>"#
        ));
    }
    rels.push_str("</Relationships>");
    types.push_str("</Types>");

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();
        zip.start_file("word/document.xml", opts).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#
        )
        .unwrap();
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(types.as_bytes()).unwrap();
        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        zip.write_all(rels.as_bytes()).unwrap();
        for i in 1..=count {
            zip.start_file(format!("word/charts/chart{i}.xml"), opts)
                .unwrap();
            zip.write_all(CHART_XML.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_chart_passthrough_fifty_parts() {
    let count = 50;
    let source = build_docx(count);
    let imported = import(&source).unwrap();
    assert_eq!(imported.retention.retained_count("chart"), count);
    assert_eq!(imported.retention.retained_count("chartPart"), count);

    let exported = export(&imported.document, &imported.package).unwrap();
    let reimported = import(&exported).unwrap();
    assert_eq!(reimported.retention.retained_count("chartPart"), count);
    for i in 1..=count {
        let part = format!("word/charts/chart{i}.xml");
        assert_eq!(
            reimported.package.parts.get(&part).map(|b| b.as_slice()),
            imported.package.parts.get(&part).map(|b| b.as_slice()),
        );
    }
}
