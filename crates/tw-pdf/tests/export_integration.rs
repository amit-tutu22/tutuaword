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
        &mut session.buffer,
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
fn visual_match_fidelity_errors_until_font_embedding_exists() {
    let doc = Document::new();
    let exporter = DisplayListPdfExporter;
    let err = exporter
        .export(
            &doc,
            &PdfExportOptions {
                fidelity: PdfFidelity::VisualMatch,
                embed_fonts: false,
            },
        )
        .unwrap_err();
    assert!(
        err.to_string().contains("VisualMatch") || err.to_string().contains("embed_fonts"),
        "{err}"
    );
}
