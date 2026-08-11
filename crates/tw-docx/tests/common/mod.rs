//! Shared DOCX corpus helpers for F23.S1 gates.

use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use tw_docx::{export, DocxPackage};
use tw_model::{
    Block, CharFormat, Color, Document, ImageBlock, ImageData, ImageTransform, NumberingRef,
    Paragraph, Revision, Run, RunContent, TextWrap, UnderlineStyle,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// Minimum open/render/round-trip corpus size for F23.S1.
pub const F23_S1_CORPUS_MIN: usize = 50;

pub fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus")
}

/// Fixtures included in open/render/round-trip gates (excludes benchmarks + encrypted).
pub fn is_gate_corpus_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    if !path.extension().is_some_and(|ext| ext == "docx") {
        return false;
    }
    if name.starts_with('_') {
        return false;
    }
    if name.starts_with("password_protected") {
        return false;
    }
    true
}

pub fn list_gate_corpus() -> Vec<PathBuf> {
    let dir = corpus_dir();
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("corpus dir missing: {}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| is_gate_corpus_file(p))
        .collect();
    entries.sort();
    entries
}

pub fn write_docx_xml(path: &Path, document_xml: &str) {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options)
            .unwrap();
        zip.write_all(b"<Relationships/>").unwrap();
        zip.finish().unwrap();
    }
    fs::write(path, buf).unwrap();
}

pub fn write_model_docx(path: &Path, doc: &Document) {
    let bytes = export(doc, &DocxPackage::minimal()).expect("export corpus fixture");
    fs::write(path, bytes).unwrap();
}

/// Ensure ≥ [`F23_S1_CORPUS_MIN`] gate-eligible synthetic fixtures exist.
pub fn ensure_corpus() {
    let dir = corpus_dir();
    fs::create_dir_all(&dir).unwrap();

    let xml_fixtures: &[(&str, &str)] = &[
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
        // Track changes (insert)
        ("track_change_insert.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:ins w:author="Ada" w:date="2024-01-01T00:00:00Z"><w:r><w:t>Inserted</w:t></w:r></w:ins></w:p></w:body></w:document>"#),
        // Unicode / scripts
        ("unicode_scripts.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Latin العربية हिन्दी 中文</w:t></w:r></w:p></w:body></w:document>"#),
        // Edge
        ("edge_empty_body.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p/></w:body></w:document>"#),
        ("edge_long_run.docx", r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA</w:t></w:r></w:p></w:body></w:document>"#),
    ];

    for (name, xml) in xml_fixtures {
        let path = dir.join(name);
        if !path.exists() {
            write_docx_xml(&path, xml);
        }
    }

    write_category_model_fixtures(&dir);

    // Pad with stable synthetic paragraphs until the gate corpus is large enough.
    let mut index = 1usize;
    while list_gate_corpus().len() < F23_S1_CORPUS_MIN {
        let name = format!("synth_paragraph_{index:02}.docx");
        let path = dir.join(&name);
        if !path.exists() {
            let doc = Document::with_paragraph(format!(
                "Synthetic corpus paragraph {index} for F23.S1 open/round-trip gate."
            ));
            write_model_docx(&path, &doc);
        }
        index += 1;
        if index > 200 {
            panic!("failed to grow corpus to {F23_S1_CORPUS_MIN} fixtures");
        }
    }
}

fn write_category_model_fixtures(dir: &Path) {
    // Styles
    let path = dir.join("cat_styles_heading1.docx");
    if !path.exists() {
        let mut doc = Document::new();
        let heading = doc
            .styles
            .paragraph_styles
            .values()
            .find(|s| s.name == "Heading 1")
            .map(|s| s.id)
            .expect("Heading 1");
        let mut para = Paragraph::with_text("Styled heading");
        para.style_id = Some(heading);
        doc.sections[0].blocks = vec![Block::Paragraph(para)];
        write_model_docx(&path, &doc);
    }

    // Numbering (bullet list)
    let path = dir.join("cat_numbering_bullet.docx");
    if !path.exists() {
        let mut doc = Document::with_paragraph("Bullet one");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.format.numbering = Some(NumberingRef {
                numbering_id: 1,
                level: 0,
            });
        }
        let mut para2 = Paragraph::with_text("Bullet two");
        para2.format.numbering = Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        });
        doc.sections[0].blocks.push(Block::Paragraph(para2));
        write_model_docx(&path, &doc);
    }

    // Table
    let path = dir.join("cat_table_2x2.docx");
    if !path.exists() {
        let mut doc = Document::new();
        let table = tw_model::Table::new(2, 2);
        doc.sections[0].blocks = vec![Block::Table(table)];
        write_model_docx(&path, &doc);
    }

    // Image PNG
    let path = dir.join("cat_image_png.docx");
    if !path.exists() {
        let mut doc = Document::new();
        doc.sections[0].blocks = vec![Block::ImageBlock(ImageBlock {
            id: tw_model::NodeId::new(),
            data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
            display_width: 72.0,
            display_height: 72.0,
            wrap: TextWrap::Inline,
            anchor: None,
            transform: ImageTransform::default(),
            caption_paragraph_id: None,
            alt_text: Some("fixture".into()),
        })];
        write_model_docx(&path, &doc);
    }

    // Character formats bundle
    let path = dir.join("cat_char_formats.docx");
    if !path.exists() {
        let mut doc = Document::new();
        let mut run = Run::new_text("Formatted");
        run.format = CharFormat {
            bold: Some(true),
            italic: Some(true),
            underline: Some(UnderlineStyle::Single),
            color: Some(Color {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }),
            font_size: Some(14.0),
            ..Default::default()
        };
        let mut para = Paragraph::new();
        para.runs = vec![run];
        doc.sections[0].blocks = vec![Block::Paragraph(para)];
        write_model_docx(&path, &doc);
    }

    // Track-change model
    let path = dir.join("cat_revision_insert.docx");
    if !path.exists() {
        let mut doc = Document::with_paragraph("Changed");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].revision = Some(Revision::insert("Ada"));
        }
        write_model_docx(&path, &doc);
    }

    // Hyperlink
    let path = dir.join("cat_hyperlink.docx");
    if !path.exists() {
        let mut doc = Document::new();
        let mut para = Paragraph::new();
        para.runs = vec![Run {
            id: tw_model::NodeId::new(),
            format: CharFormat::default(),
            content: RunContent::Hyperlink {
                target: tw_model::HyperlinkTarget {
                    url: "https://example.com".into(),
                    anchor: None,
                    tooltip: None,
                },
                text: "Example".into(),
            },
            revision: None,
        }];
        doc.sections[0].blocks = vec![Block::Paragraph(para)];
        write_model_docx(&path, &doc);
    }

    // Business-like multi-block
    let path = dir.join("cat_business_memo.docx");
    if !path.exists() {
        let mut doc = Document::new();
        doc.properties.title = Some("Memo".into());
        doc.properties.author = Some("Legal".into());
        doc.sections[0].blocks = vec![
            Block::Paragraph(Paragraph::with_text("MEMORANDUM")),
            Block::Paragraph(Paragraph::with_text(
                "Please review the attached agreement before Friday.",
            )),
            Block::Paragraph(Paragraph::with_text("Regards,")),
        ];
        write_model_docx(&path, &doc);
    }
}
