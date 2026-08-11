//! U-F11-S1-drawing-preserves-bytes — imported DrawingML shape XML round-trips unchanged.

use std::io::{Cursor, Write};

use tw_docx::{export, import};
use tw_model::ShapeKind;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const SHAPE_PARAGRAPH: &str = r#"<w:p>
  <w:r>
    <w:drawing>
      <wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
        distT="0" distB="0" distL="0" distR="0">
        <wp:extent cx="1828800" cy="914400"/>
        <wp:docPr id="42" name="BlueRectangle"/>
        <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
          <a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
            <wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
              <wps:cNvPr id="42" name="BlueRectangle"/>
            </wps:wsp>
          </a:graphicData>
        </a:graphic>
      </wp:inline>
    </w:drawing>
  </w:r>
</w:p>"#;

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
        zip.write_all(b"<Relationships/>").unwrap();
        zip.start_file("word/drawings/drawing1.xml", opts).unwrap();
        zip.write_all(b"<drawing/>").unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn u_f11_s1_drawing_preserves_bytes() {
    let source = minimal_docx(SHAPE_PARAGRAPH);
    let imported = import(&source).unwrap();

    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("expected a shape block");
    assert_eq!(shape.shape.shape_type, ShapeKind::TextBox);
    assert!((shape.shape.width - 144.0).abs() < 0.01);
    assert!((shape.shape.height - 72.0).abs() < 0.01);

    let exported = export(&imported.document, &imported.package).unwrap();
    let document_xml =
        String::from_utf8(exported_package_part(&exported, "word/document.xml")).unwrap();

    assert!(
        document_xml.contains(r#"docPr id="42" name="BlueRectangle""#),
        "shape drawing metadata was lost: {document_xml}"
    );
    assert!(
        document_xml.contains("wordprocessingShape"),
        "shape graphicData uri was lost: {document_xml}"
    );

    // Unrelated package parts stay untouched.
    assert_eq!(
        exported_package_part(&exported, "word/drawings/drawing1.xml"),
        b"<drawing/>"
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
