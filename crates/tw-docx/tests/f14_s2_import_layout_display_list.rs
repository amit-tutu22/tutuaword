//! U-F14-S2 — DOCX import → layout → display list paints OMML preview geometry.

use std::io::{Cursor, Write};

use tw_docx::import;
use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::MATH_FRAME_ARGB;
use tw_render::DisplayListBuilder;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const BODY: &str = r#"<w:p>
  <w:r><w:t>See </w:t></w:r>
  <m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
    <m:r><m:t>F=ma</m:t></m:r>
  </m:oMath>
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
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
  xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <w:body>{body}<w:sectPr/></w:body>
</w:document>"#
        )
        .unwrap();
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        )
        .unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn u_f14_s2_import_to_display_list_has_omml_glyphs() {
    let imported = import(&minimal_docx(BODY)).unwrap();
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&imported.document);

    let codepoints: Vec<char> = layout.pages[0]
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => {
                Some(line.glyphs.iter().map(|g| g.codepoint).collect::<Vec<_>>())
            }
            _ => None,
        })
        .flatten()
        .collect();

    assert!(
        codepoints.contains(&'F') && codepoints.contains(&'m'),
        "imported OMML should layout as preview glyphs: {codepoints:?}"
    );
    assert!(
        codepoints.contains(&'S'),
        "surrounding paragraph text should still layout: {codepoints:?}"
    );

    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);
    assert!(
        list.rect_batch.colors.iter().any(|&c| c == MATH_FRAME_ARGB),
        "imported equation should paint math frame in display list"
    );
    assert!(
        list.atlas_batch.transforms.len() >= 4,
        "F=ma plus See should produce multiple glyphs"
    );
}
