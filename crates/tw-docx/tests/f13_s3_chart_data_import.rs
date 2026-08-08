//! U-F13-S3 — chart dataset parsed from chart part on import.

use std::io::{Cursor, Write};

use tw_docx::import;
use tw_model::{ChartData, ShapeKind};
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

fn minimal_docx(body: &str, chart_xml: &str) -> Vec<u8> {
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
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        zip.write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart" Target="charts/chart1.xml"/>
</Relationships>"#,
        )
        .unwrap();
        zip.start_file("word/charts/chart1.xml", opts).unwrap();
        zip.write_all(chart_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn u_f13_s3_chart_data_imported() {
    let chart_xml = tw_docx::chart::serialize_chart_xml(&ChartData::sample_bar());
    let imported = import(&minimal_docx(CHART_PARAGRAPH, &chart_xml)).unwrap();
    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("chart shape");

    assert_eq!(shape.shape.shape_type, ShapeKind::Chart);
    assert_eq!(shape.chart_part.as_deref(), Some("word/charts/chart1.xml"));
    assert_eq!(shape.chart_data.as_ref(), Some(&ChartData::sample_bar()));
    assert_eq!(imported.retention.retained_count("chartData"), 1);
}
