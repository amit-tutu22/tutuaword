//! U-F12-S2 — SmartArt diagram preview PNG resolved from drawing part.

use std::io::{Cursor, Write};

use tw_docx::import;
use tw_model::ShapeKind;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

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
              r:dm="rId5" r:dr="rId7"/>
          </a:graphicData>
        </a:graphic>
      </wp:inline>
    </w:drawing>
  </w:r>
</w:p>"#;

const DRAWING_PREVIEW: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<dsp:drawing xmlns:dsp="http://schemas.microsoft.com/office/drawing/2008/diagramDrawing"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <dsp:spTree>
    <a:blip r:embed="rId1"/>
  </dsp:spTree>
</dsp:drawing>"#;

fn minimal_docx(body: &str) -> Vec<u8> {
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
  <Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData" Target="diagrams/data1.xml"/>
  <Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramDrawing" Target="diagrams/drawing1.xml"/>
</Relationships>"#,
        )
        .unwrap();
        zip.start_file("word/diagrams/data1.xml", opts).unwrap();
        zip.write_all(b"<dgm:dataModel/>").unwrap();
        zip.start_file("word/diagrams/drawing1.xml", opts).unwrap();
        zip.write_all(DRAWING_PREVIEW.as_bytes()).unwrap();
        zip.start_file("word/diagrams/_rels/drawing1.xml.rels", opts).unwrap();
        zip.write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/>
</Relationships>"#,
        )
        .unwrap();
        zip.start_file("word/media/image1.png", opts).unwrap();
        zip.write_all(PNG_1X1).unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn u_f12_s2_diagram_preview_png_imported() {
    let imported = import(&minimal_docx(DIAGRAM_PARAGRAPH)).unwrap();
    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("diagram shape");
    assert_eq!(shape.shape.shape_type, ShapeKind::Diagram);
    let preview = shape
        .preview_image
        .as_ref()
        .expect("expected diagram preview image");
    assert_eq!(preview.asset_id, "word/media/image1.png");
    assert_eq!(preview.bytes.as_slice(), PNG_1X1);
    assert_eq!(imported.retention.retained_count("diagramPreview"), 1);
}
