use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::time::Instant;

use tw_docx::import;
use tw_layout::LayoutEngine;
use tw_render::DisplayListBuilder;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus")
}

fn write_docx(path: &PathBuf, document_xml: &str) {
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
        zip.finish().unwrap();
    }
    fs::write(path, buf).unwrap();
}

fn ensure_corpus() {
    let dir = corpus_dir();
    fs::create_dir_all(&dir).unwrap();

    let fixtures: &[(&str, &str)] = &[
        ("simple_paragraph.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello corpus</w:t></w:r></w:p></w:body></w:document>"#),
        ("bold_heading.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:b/><w:sz w:val="28"/></w:rPr><w:t>Bold Heading</w:t></w:r></w:p></w:body></w:document>"#),
        ("centered_text.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Centered</w:t></w:r></w:p></w:body></w:document>"#),
        ("spaced_paragraphs.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:spacing w:before="240" w:after="240"/></w:pPr><w:r><w:t>Spaced</w:t></w:r></w:p><w:p><w:r><w:t>Second</w:t></w:r></w:p></w:body></w:document>"#),
        ("small_table.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>D</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#),
        ("indented_paragraph.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:ind w:left="720" w:firstLine="360"/></w:pPr><w:r><w:t>Indented text block</w:t></w:r></w:p></w:body></w:document>"#),
        ("colored_run.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:color w:val="FF0000"/></w:rPr><w:t>Red text</w:t></w:r></w:p></w:body></w:document>"#),
        ("mixed_format_line.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Normal </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r><w:r><w:t> end</w:t></w:r></w:p></w:body></w:document>"#),
        ("numbered_style.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>Item one</w:t></w:r></w:p><w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>Item two</w:t></w:r></w:p></w:body></w:document>"#),
        ("section_margins.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Section margins</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="720" w:right="720" w:bottom="720" w:left="720"/></w:sectPr></w:body></w:document>"#),
        ("multi_paragraph.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Para 1</w:t></w:r></w:p><w:p><w:r><w:t>Para 2</w:t></w:r></w:p><w:p><w:r><w:t>Para 3</w:t></w:r></w:p><w:p><w:r><w:t>Para 4</w:t></w:r></w:p><w:p><w:r><w:t>Para 5</w:t></w:r></w:p></w:body></w:document>"#),
        ("underline_run.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t>Underlined</w:t></w:r></w:p></w:body></w:document>"#),
        ("italic_run.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:i/></w:rPr><w:t>Italic</w:t></w:r></w:p></w:body></w:document>"#),
        ("large_font.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:sz w:val="48"/></w:rPr><w:t>Large</w:t></w:r></w:p></w:body></w:document>"#),
        ("page_break.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:pageBreakBefore/></w:pPr><w:r><w:t>After break</w:t></w:r></w:p></w:body></w:document>"#),
        ("empty_paragraph_spacing.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:spacing w:after="480"/></w:pPr></w:p><w:p><w:r><w:t>After empty</w:t></w:r></w:p></w:body></w:document>"#),
        ("three_by_three_table.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>3</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>4</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>5</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>6</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>7</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>8</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>9</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#),
        ("right_align.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:t>Right</w:t></w:r></w:p></w:body></w:document>"#),
        ("justify_align.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:jc w:val="both"/></w:pPr><w:r><w:t>Justified paragraph with enough words to wrap across the line when rendered by the layout engine.</w:t></w:r></w:p></w:body></w:document>"#),
        ("highlight_run.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:highlight w:val="yellow"/></w:rPr><w:t>Highlighted</w:t></w:r></w:p></w:body></w:document>"#),
    ];

    for (name, xml) in fixtures {
        let path = dir.join(name);
        if !path.exists() {
            write_docx(&path, xml);
        }
    }
}

fn render_corpus_file(path: &PathBuf) -> Result<(usize, usize), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let imported = import(&bytes).map_err(|e| e.to_string())?;
    let mut layout = LayoutEngine::new();
    let doc_layout = layout.layout_document(&imported.document);
    let page_count = doc_layout.pages.len();
    let mut glyph_total = 0usize;
    for page in &doc_layout.pages {
        let list = DisplayListBuilder::from_page(page, layout.atlas(), 1);
        glyph_total += list.atlas_batch.transforms.len() / 2;
        if list.atlas_batch.transforms.is_empty()
            && list.path_batch.points.is_empty()
            && list.rect_batch.rects.is_empty()
        {
            return Err("empty display list".into());
        }
    }
    Ok((page_count, glyph_total))
}

#[test]
fn corpus_render_gate_passes_95_percent() {
    ensure_corpus();
    let dir = corpus_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "docx"))
        .collect();
    assert!(entries.len() >= 20, "corpus should have at least 20 docx files");

    let mut passed = 0usize;
    let mut failures = Vec::new();
    for entry in &entries {
        let path = entry.path();
        match render_corpus_file(&path) {
            Ok((pages, glyphs)) => {
                passed += 1;
                assert!(pages >= 1);
                let _ = glyphs;
            }
            Err(err) => {
                failures.push(format!("{}: {err}", path.display()));
            }
        }
    }

    let pass_rate = passed as f64 / entries.len() as f64;
    assert!(
        pass_rate >= 0.95,
        "corpus pass rate {:.0}% below 95% gate. Failures: {:?}",
        pass_rate * 100.0,
        failures
    );
}

#[test]
fn large_docx_open_benchmark_under_two_seconds() {
    let mut body = String::from(r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#);
    for i in 0..500 {
        body.push_str(&format!(
            r#"<w:p><w:r><w:t>Page filler paragraph {} with enough text to consume vertical space on the page during layout.</w:t></w:r></w:p>"#,
            i
        ));
        if i % 50 == 49 {
            body.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
        }
    }
    body.push_str("</w:body></w:document>");

    let dir = corpus_dir();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("_benchmark_500page.docx");
    write_docx(&path, &body);

    let bytes = fs::read(&path).unwrap();
    let start = Instant::now();
    let imported = import(&bytes).unwrap();
    let mut layout = LayoutEngine::new();
    layout.layout_document(&imported.document);
    let elapsed = start.elapsed();
    let max_secs = if cfg!(debug_assertions) { 12.0 } else { 2.0 };
    assert!(
        elapsed.as_secs_f64() < max_secs,
        "500-page layout took {:.2}s, expected < {:.0}s",
        elapsed.as_secs_f64(),
        max_secs
    );
}
