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
        }),
    ];

    let pdf = DisplayListPdfExporter
        .export(&doc, &PdfExportOptions::default())
        .expect("structural PDF with image");
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("/Subtype /Image"));
    assert!(pdf_str.contains("/XObject"));
}
