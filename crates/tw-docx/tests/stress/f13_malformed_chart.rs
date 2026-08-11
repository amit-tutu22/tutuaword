//! Stress: malformed chart OPC imports without panic.

use std::io::{Cursor, Write};

use tw_docx::import;
use tw_model::ShapeKind;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const CHART_PARAGRAPH: &str = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="rId5"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

fn minimal_docx(body: &str, include_chart_part: bool) -> Vec<u8> {
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
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        zip.write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart" Target="charts/chart1.xml"/></Relationships>"#,
        )
        .unwrap();
        if include_chart_part {
            zip.start_file("word/charts/chart1.xml", opts).unwrap();
            zip.write_all(b"<c:chartSpace/>").unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_malformed_chart_missing_part() {
    let imported = import(&minimal_docx(CHART_PARAGRAPH, false)).unwrap();
    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("chart shape");
    assert_eq!(shape.shape.shape_type, ShapeKind::Chart);
    assert!(shape.preview_image.is_none());
    assert!(shape.chart_data.is_none());
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_malformed_chart_empty_xml() {
    let imported = import(&minimal_docx(CHART_PARAGRAPH, true)).unwrap();
    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("chart shape");
    assert_eq!(shape.shape.shape_type, ShapeKind::Chart);
    assert!(shape.chart_data.is_none());
}
