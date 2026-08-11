//! U-F12-S1-diagram-part-survives-save — SmartArt diagram OPC parts round-trip unchanged.

use std::io::{Cursor, Write};

use tw_docx::{export, import};
use tw_model::ShapeKind;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const DIAGRAM_PARAGRAPH: &str = r#"<w:p>
  <w:r>
    <w:drawing>
      <wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
        distT="0" distB="0" distL="0" distR="0">
        <wp:extent cx="5486400" cy="2743200"/>
        <wp:docPr id="2" name="SmartArt Diagram"/>
        <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
          <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/diagram">
            <dgm:relIds xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram"
              xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
              r:dm="rId5"/>
          </a:graphicData>
        </a:graphic>
      </wp:inline>
    </w:drawing>
  </w:r>
</w:p>"#;

const DIAGRAM_DATA: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<dgm:dataModel xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram">
  <dgm:ptLst><dgm:pt modelId="f12-s1-unique-token"/></dgm:ptLst>
</dgm:dataModel>"#;

const DIAGRAM_LAYOUT: &[u8] = b"<dgm:layoutDef/>";

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
  <Override PartName="/word/diagrams/data1.xml" ContentType="application/vnd.openxmlformats-officedocument.drawingml.diagramData+xml"/>
  <Override PartName="/word/diagrams/layout1.xml" ContentType="application/vnd.openxmlformats-officedocument.drawingml.diagramLayout+xml"/>
</Types>"#,
        )
        .unwrap();
        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        zip.write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData" Target="diagrams/data1.xml"/>
  <Relationship Id="rId6" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramLayout" Target="diagrams/layout1.xml"/>
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
fn u_f12_s1_diagram_part_survives_save() {
    let source = minimal_docx(
        DIAGRAM_PARAGRAPH,
        &[
            ("word/diagrams/data1.xml", DIAGRAM_DATA),
            ("word/diagrams/layout1.xml", DIAGRAM_LAYOUT),
        ],
    );
    let imported = import(&source).unwrap();

    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("expected a diagram shape block");
    assert_eq!(shape.shape.shape_type, ShapeKind::Diagram);
    assert!((shape.shape.width - 432.0).abs() < 0.01);
    assert!((shape.shape.height - 216.0).abs() < 0.01);
    assert!(imported.package.parts.contains_key("word/diagrams/data1.xml"));
    assert!(imported.package.parts.contains_key("word/diagrams/layout1.xml"));
    assert_eq!(imported.retention.retained_count("diagram"), 1);
    assert_eq!(imported.retention.retained_count("diagramPart"), 2);

    let exported = export(&imported.document, &imported.package).unwrap();
    let document_xml =
        String::from_utf8(exported_package_part(&exported, "word/document.xml")).unwrap();

    assert!(
        document_xml.contains("drawingml/2006/diagram"),
        "diagram graphicData uri was lost: {document_xml}"
    );
    assert!(
        document_xml.contains(r#"docPr id="2" name="SmartArt Diagram""#),
        "diagram drawing metadata was lost: {document_xml}"
    );

    assert_eq!(
        exported_package_part(&exported, "word/diagrams/data1.xml"),
        DIAGRAM_DATA,
        "diagram data part bytes changed on save"
    );
    assert_eq!(
        exported_package_part(&exported, "word/diagrams/layout1.xml"),
        DIAGRAM_LAYOUT,
        "diagram layout part bytes changed on save"
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
