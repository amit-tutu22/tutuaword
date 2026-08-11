//! U-F13-S1-chart-xml-passthrough — chart OPC parts round-trip unchanged.

use std::io::{Cursor, Write};

use tw_docx::{export, import};
use tw_model::ShapeKind;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const CHART_PARAGRAPH: &str = r#"<w:p>
  <w:r>
    <w:drawing>
      <wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
        distT="0" distB="0" distL="0" distR="0">
        <wp:extent cx="5486400" cy="2743200"/>
        <wp:docPr id="3" name="Chart 1"/>
        <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
          <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart">
            <c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
              xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
              r:id="rId5"/>
          </a:graphicData>
        </a:graphic>
      </wp:inline>
    </w:drawing>
  </w:r>
</w:p>"#;

const CHART_DATA: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
  <c:chart>
    <c:plotArea>
      <c:barChart>
        <c:ser><c:idx val="0"/><c:val><c:numRef><c:f>Sheet1!$B$1</c:f></c:numRef></c:val></c:ser>
      </c:barChart>
    </c:plotArea>
  </c:chart>
</c:chartSpace>"#;

fn minimal_docx(body: &str, extra_parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();
        zip.start_file("word/document.xml", opts).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr/></w:body>
</w:document>"#
        )
        .unwrap();
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/charts/chart1.xml" ContentType="application/vnd.openxmlformats-officedocument.drawingml.chart+xml"/>
</Types>"#,
        )
        .unwrap();
        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        zip.write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart" Target="charts/chart1.xml"/>
</Relationships>"#,
        )
        .unwrap();
        for (name, data) in extra_parts {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn u_f13_s1_chart_xml_passthrough() {
    let source = minimal_docx(CHART_PARAGRAPH, &[("word/charts/chart1.xml", CHART_DATA)]);
    let imported = import(&source).unwrap();

    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("expected a chart shape block");
    assert_eq!(shape.shape.shape_type, ShapeKind::Chart);
    assert!((shape.shape.width - 432.0).abs() < 0.01);
    assert!((shape.shape.height - 216.0).abs() < 0.01);
    assert!(imported.package.parts.contains_key("word/charts/chart1.xml"));
    assert_eq!(imported.retention.retained_count("chart"), 1);
    assert_eq!(imported.retention.retained_count("chartPart"), 1);

    let exported = export(&imported.document, &imported.package).unwrap();
    let document_xml =
        String::from_utf8(exported_package_part(&exported, "word/document.xml")).unwrap();

    assert!(
        document_xml.contains("drawingml/2006/chart"),
        "chart graphicData uri was lost: {document_xml}"
    );
    assert!(
        document_xml.contains(r#"docPr id="3" name="Chart 1""#),
        "chart drawing metadata was lost: {document_xml}"
    );

    assert_eq!(
        exported_package_part(&exported, "word/charts/chart1.xml"),
        CHART_DATA,
        "chart part bytes changed on save"
    );
}

fn exported_package_part(docx: &[u8], name: &str) -> Vec<u8> {
    use std::io::Read;
    let cursor = Cursor::new(docx);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name(name).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    data
}
