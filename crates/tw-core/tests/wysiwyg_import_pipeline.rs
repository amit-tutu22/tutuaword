use std::io::{Cursor, Write};

use tw_core::import_document_bundle;
use tw_layout::LayoutEngine;
use tw_render::DisplayListBuilder;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn minimal_docx(document_xml: &str, extra_parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options).unwrap();
        zip.write_all(b"<Relationships/>").unwrap();
        for (name, data) in extra_parts {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

fn styled_heading_with_table_docx() -> Vec<u8> {
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
  <w:p>
    <w:pPr><w:jc w:val="center"/><w:spacing w:before="240" w:after="120"/></w:pPr>
    <w:r><w:rPr><w:b/><w:sz w:val="32"/></w:rPr><w:t>Resume</w:t></w:r>
  </w:p>
  <w:p><w:r><w:t>Experience section with mixed formatting.</w:t></w:r></w:p>
  <w:tbl>
    <w:tr>
      <w:tc><w:p><w:r><w:t>A1</w:t></w:r></w:p></w:tc>
      <w:tc><w:p><w:r><w:t>B1</w:t></w:r></w:p></w:tc>
      <w:tc><w:p><w:r><w:t>C1</w:t></w:r></w:p></w:tc>
    </w:tr>
    <w:tr>
      <w:tc><w:p><w:r><w:t>A2</w:t></w:r></w:p></w:tc>
      <w:tc><w:p><w:r><w:t>B2</w:t></w:r></w:p></w:tc>
      <w:tc><w:p><w:r><w:t>C2</w:t></w:r></w:p></w:tc>
    </w:tr>
    <w:tr>
      <w:tc><w:p><w:r><w:t>A3</w:t></w:r></w:p></w:tc>
      <w:tc><w:p><w:r><w:t>B3</w:t></w:r></w:p></w:tc>
      <w:tc><w:p><w:r><w:t>C3</w:t></w:r></w:p></w:tc>
    </w:tr>
  </w:tbl>
  <w:sectPr>
    <w:pgSz w:w="12240" w:h="15840"/>
    <w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/>
  </w:sectPr>
</w:body></w:document>"#;
    minimal_docx(xml, &[])
}

fn resume_like_docx(paragraph_count: usize) -> Vec<u8> {
    let mut body = String::from(r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#);
    body.push_str(r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:sz w:val="28"/></w:rPr><w:t>Professional Resume</w:t></w:r></w:p>"#);
    for i in 0..paragraph_count {
        body.push_str(&format!(
            r#"<w:p><w:pPr><w:spacing w:after="120"/></w:pPr><w:r><w:t>Section {}.</w:t></w:r></w:p>"#,
            i + 1
        ));
    }
    body.push_str(r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#);
    minimal_docx(&body, &[])
}

#[test]
fn docx_import_layout_produces_glyph_and_table_batches() {
    let bytes = styled_heading_with_table_docx();
    let bundle = import_document_bundle(&bytes, Some("styled.docx")).unwrap();
    let mut layout = LayoutEngine::new();
    let doc_layout = layout.layout_document(&bundle.document);
    assert!(doc_layout.pages.len() >= 1, "expected at least one page");

    let page = &doc_layout.pages[0];
    let list = DisplayListBuilder::from_page(page, layout.atlas(), 1);
    let dl_bytes = DisplayListBuilder::to_bytes(&list);
    assert!(!dl_bytes.is_empty(), "display list should not be empty");
    assert!(
        !list.atlas_batch.transforms.is_empty()
            || !list.path_batch.points.is_empty()
            || !list.rect_batch.rects.is_empty(),
        "expected paintable content in display list"
    );
}

#[test]
fn resume_like_docx_paginates_reasonably() {
    let bytes = resume_like_docx(30);
    let bundle = import_document_bundle(&bytes, Some("resume.docx")).unwrap();
    let mut layout = LayoutEngine::new();
    let doc_layout = layout.layout_document(&bundle.document);
    let page_count = doc_layout.pages.len();
    assert!(
        page_count >= 1 && page_count <= 10,
        "resume-like doc should paginate reasonably, got {page_count}"
    );
}

#[test]
fn atlas_contains_antialiased_glyph_shapes() {
    let bytes = styled_heading_with_table_docx();
    let bundle = import_document_bundle(&bytes, Some("styled.docx")).unwrap();
    let mut layout = LayoutEngine::new();
    layout.layout_document(&bundle.document);

    let pixels = layout.atlas().pixels_rgba();
    let mut opaque = 0usize;
    let mut partial = 0usize;
    for px in pixels.chunks(4) {
        match px[3] {
            0 => {}
            255 => opaque += 1,
            _ => partial += 1,
        }
        // Masks are stored premultiplied white.
        assert_eq!([px[0], px[1], px[2]], [px[3], px[3], px[3]]);
    }

    assert!(opaque > 0, "expected solid glyph interiors in the atlas");
    assert!(
        partial > opaque / 4,
        "expected antialiased edges, got {partial} partial vs {opaque} opaque pixels"
    );
}

#[test]
fn glyph_quads_vary_in_size_and_sit_around_the_baseline() {
    let bytes = styled_heading_with_table_docx();
    let bundle = import_document_bundle(&bytes, Some("styled.docx")).unwrap();
    let mut layout = LayoutEngine::new();
    let doc_layout = layout.layout_document(&bundle.document);

    let line = doc_layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            tw_layout::LayoutBox::TextLine(l) if l.glyphs.len() > 3 => Some(l),
            _ => None,
        })
        .expect("a text line with glyphs");

    let widths: Vec<u32> = line.glyphs.iter().map(|g| g.width as u32).collect();
    assert!(
        widths.iter().collect::<std::collections::HashSet<_>>().len() > 1,
        "real glyph bitmaps should differ in width, got {widths:?}"
    );

    for g in &line.glyphs {
        assert!(g.width > 0.0 && g.height > 0.0);
        assert!(
            g.y < line.y && g.y + g.height > line.y - line.ascent - 1.0,
            "glyph quad should straddle the baseline"
        );
    }
}

#[test]
fn layout_hit_test_finds_run_on_first_page() {
    let bytes = styled_heading_with_table_docx();
    let bundle = import_document_bundle(&bytes, Some("styled.docx")).unwrap();
    let mut layout = LayoutEngine::new();
    layout.layout_document(&bundle.document);
    let map = layout.line_map(0).expect("line map for page 0");
    assert!(!map.lines.is_empty());
    let line = &map.lines[0];
    let hit = map.hit_test(line.x + 10.0, line.y);
    assert!(hit.is_some(), "hit test should find a run near line start");
}
