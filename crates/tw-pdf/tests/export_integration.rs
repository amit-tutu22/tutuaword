use tw_edit::{apply, Command, EditSession};
use tw_model::{Block, Document, Paragraph};
use tw_pdf::{DisplayListPdfExporter, PdfExportOptions, PdfExporter, PdfFidelity};

#[test]
fn u_f01_s3_pdf_starts_with_header() {
    let doc = Document::new();
    let exporter = DisplayListPdfExporter;
    let pdf = exporter.export(&doc, &PdfExportOptions::default()).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

#[test]
fn export_empty_document_produces_valid_pdf() {
    let doc = Document::new();
    let exporter = DisplayListPdfExporter;
    let pdf = exporter.export(&doc, &PdfExportOptions::default()).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert!(pdf.ends_with(b"%%EOF\n") || pdf.ends_with(b"%%EOF"));
}

    #[test]
    fn export_multi_paragraph_document_contains_real_characters() {
        let mut doc = Document::new();
        doc.sections[0].blocks = vec![
            Block::Paragraph(Paragraph::with_text("Title")),
            Block::Paragraph(Paragraph::with_text("Body paragraph one.")),
            Block::Paragraph(Paragraph::with_text("Body paragraph two.")),
        ];

        let exporter = DisplayListPdfExporter;
        let pdf = exporter.export(&doc, &PdfExportOptions::default()).unwrap();
        assert!(pdf.len() > 200);
        let pdf_str = String::from_utf8_lossy(&pdf);
        assert!(pdf_str.contains("/Type /Page"));
        assert!(pdf_str.contains("(Title) Tj") || pdf_str.contains("(Body paragraph one.) Tj"));
    }

#[test]
fn export_document_with_header_footer_and_table() {
    let mut session = EditSession::new();
    session.document.sections[0].format.header_text = Some("Report".into());
    session.document.sections[0].format.footer_text = Some("Confidential".into());

    let block_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;
    apply(
        &mut session.document,
        Command::InsertTable {
            after_block_id: block_id,
            rows: 2,
            cols: 2,
        },
    )
    .unwrap();

    let exporter = DisplayListPdfExporter;
    let pdf = exporter
        .export(&session.document, &PdfExportOptions::default())
        .unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("/Type /Page"));
    assert!(pdf_str.contains(" l S") || pdf_str.contains(" m "));
}

#[test]
fn export_long_document_spans_multiple_pdf_pages() {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..100 {
        doc.sections[0].blocks.push(Block::Paragraph(Paragraph::with_text(format!(
            "Paragraph {i} with enough text to force pagination in the layout engine."
        ))));
    }

    let exporter = DisplayListPdfExporter;
    let pdf = exporter.export(&doc, &PdfExportOptions::default()).unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf);
    let page_count = pdf_str.matches("/Type /Page").count();
    assert!(page_count > 1, "expected multiple PDF pages, got {page_count}");
}

#[test]
fn u_f23_s2_visual_match_exports_with_fontfile2() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "VisualMatch sample text",
    ))];
    let exporter = DisplayListPdfExporter;
    let pdf = exporter
        .export(
            &doc,
            &PdfExportOptions {
                fidelity: PdfFidelity::VisualMatch,
                embed_fonts: false,
                ..Default::default()
            },
        )
        .expect("VisualMatch should export once font embedding is ready");
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("/FontFile2"));
    assert!(pdf_str.contains("/Identity-H"));
}

#[test]
fn u_f23_s2_embed_fonts_writes_fontfile2() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text("Embed fonts"))];
    let pdf = DisplayListPdfExporter
        .export(
            &doc,
            &PdfExportOptions {
                fidelity: PdfFidelity::Structural,
                embed_fonts: true,
                ..Default::default()
            },
        )
        .expect("embed_fonts should write FontFile2");
    assert!(String::from_utf8_lossy(&pdf).contains("/FontFile2"));
}

#[test]
fn u_f23_s2_images_written_as_xobjects() {
    const PNG_1X1: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    use tw_model::{ImageBlock, ImageData, ImageTransform, TextWrap};

    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(Paragraph::with_text("Caption")),
        Block::ImageBlock(ImageBlock {
            id: tw_model::NodeId::new(),
            data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
            display_width: 72.0,
            display_height: 72.0,
            wrap: TextWrap::Inline,
            anchor: None,
            transform: ImageTransform::default(),
            caption_paragraph_id: None,
            alt_text: None,
            wrap_polygon: None,
        }),
    ];

    let pdf = DisplayListPdfExporter
        .export(&doc, &PdfExportOptions::default())
        .expect("structural PDF with image");
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("/Subtype /Image"));
    assert!(pdf_str.contains("/XObject"));
}

#[test]
fn gif_image_exports_as_pdf_xobject_without_aborting() {
    const GIF: &[u8] = include_bytes!("sample.gif");

    use tw_model::{ImageBlock, ImageData, ImageTransform, TextWrap};

    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::ImageBlock(ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(GIF.to_vec(), Some("image/gif".into())),
        display_width: 120.0,
        display_height: 80.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: None,
        wrap_polygon: None,
    })];

    let pdf = DisplayListPdfExporter
        .export(&doc, &PdfExportOptions::default())
        .expect("GIF must not abort PDF export");
    assert!(pdf.starts_with(b"%PDF"));
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(
        pdf_str.contains("/Subtype /Image"),
        "GIF should decode into a PDF image XObject"
    );
}

#[test]
fn export_headings_emit_linked_outlines() {
    let mut doc = Document::new();
    let h1 = doc.styles.find_style_by_name("Heading 1").unwrap().id;
    let h2 = doc.styles.find_style_by_name("Heading 2").unwrap().id;
    let mut first = Paragraph::with_text("Alpha");
    first.style_id = Some(h1);
    let mut second = Paragraph::with_text("Beta");
    second.style_id = Some(h2);
    doc.sections[0].blocks = vec![Block::Paragraph(first), Block::Paragraph(second)];

    let pdf = DisplayListPdfExporter
        .export(&doc, &PdfExportOptions::default())
        .unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("/Type /Outlines"), "{pdf_str}");
    assert!(pdf_str.contains("/Next "), "outline items must form a linked list");
    assert!(pdf_str.contains("/Prev "), "outline items must form a linked list");
    // Destinations must not all be the page top — heading Y should appear.
    let dests: Vec<_> = pdf_str
        .match_indices("/XYZ 0 ")
        .map(|(i, _)| {
            let rest = &pdf_str[i + "/XYZ 0 ".len()..];
            rest.split_whitespace()
                .next()
                .and_then(|s| s.parse::<f32>().ok())
        })
        .collect();
    assert!(dests.len() >= 2, "expected outline Dest entries, got {dests:?}");
    let tops: Vec<f32> = dests.into_iter().flatten().collect();
    assert!(
        tops.windows(2).any(|w| (w[0] - w[1]).abs() > 1.0),
        "outline Dest Y values should differ per heading, got {tops:?}"
    );
}

#[test]
fn export_internal_hyperlink_uses_goto() {
    use tw_model::{bookmark_run, hyperlink_run};

    let mut doc = Document::new();
    let mut target = Paragraph::with_text("Target heading");
    target.runs.insert(0, bookmark_run("sec1", 1));
    let mut source = Paragraph::new();
    source.runs = vec![hyperlink_run("#sec1", "jump", None)];
    let pad: Vec<_> = (0..20)
        .map(|i| Block::Paragraph(Paragraph::with_text(format!("Pad {i}"))))
        .collect();
    let mut blocks = vec![Block::Paragraph(source)];
    blocks.extend(pad);
    blocks.push(Block::Paragraph(target));
    doc.sections[0].blocks = blocks;

    let pdf = DisplayListPdfExporter
        .export(&doc, &PdfExportOptions::default())
        .unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(
        pdf_str.contains("/S /GoTo"),
        "internal #anchor links must use GoTo, not URI: {pdf_str}"
    );
    assert!(
        !pdf_str.contains("/URI (#sec1)"),
        "internal anchors must not be URI actions"
    );
}
