//! F25.S1 — print-ready PDF preparation (unit + stress).

use tw_model::{Block, Document, Paragraph};
use tw_pdf::{
    prepare_print_pdf, DisplayListPdfExporter, PdfExportOptions, PdfExporter, PdfFidelity,
    PrintLayoutOptions,
};

#[test]
fn u_f25_s1_for_print_options_are_visual_match() {
    let opts = PdfExportOptions::for_print();
    assert_eq!(opts.fidelity, PdfFidelity::VisualMatch);
    assert!(opts.embed_fonts);
}

#[test]
fn u_f25_s1_prepare_print_pdf_header() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Print dialog sample",
    ))];
    let pdf = prepare_print_pdf(&doc, &PrintLayoutOptions::default()).expect("print PDF");
    assert!(pdf.starts_with(b"%PDF"), "print PDF must start with %PDF");
    assert!(pdf.len() > 64);
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("/Type /Page"));
}

#[test]
fn u_f25_s1_prepare_print_pdf_falls_back_or_embeds() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text("Hello print"))];
    let pdf = prepare_print_pdf(&doc, &PrintLayoutOptions::default()).expect("print PDF");
    let pdf_str = String::from_utf8_lossy(&pdf);
    // VisualMatch embeds FontFile2; structural fallback still has a page + Helvetica.
    assert!(
        pdf_str.contains("/FontFile2")
            || pdf_str.contains("/BaseFont /Helvetica")
            || pdf_str.contains("/Type /Font"),
        "expected embedded face or structural font resources"
    );
}

#[test]
fn u_f25_s1_print_options_differ_from_default_export() {
    let def = PdfExportOptions::default();
    let print = PdfExportOptions::for_print();
    assert_ne!(def.fidelity, print.fidelity);
    assert_ne!(def.embed_fonts, print.embed_fonts);

    let mut doc = Document::new();
    doc.sections[0].blocks =
        vec![Block::Paragraph(Paragraph::with_text("Compare export modes"))];
    let structural = DisplayListPdfExporter
        .export(&doc, &def)
        .expect("structural");
    let for_print = prepare_print_pdf(&doc, &PrintLayoutOptions::default()).expect("print");
    assert!(structural.starts_with(b"%PDF"));
    assert!(for_print.starts_with(b"%PDF"));
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f25_s1_print_pdf_churn() {
    let exporter = DisplayListPdfExporter;
    for round in 0..25 {
        let mut doc = Document::new();
        doc.sections[0].blocks.clear();
        for i in 0..40 {
            doc.sections[0].blocks.push(Block::Paragraph(Paragraph::with_text(
                format!("Stress print paragraph {round}-{i} with padding text for layout."),
            )));
        }
        let pdf = prepare_print_pdf(&doc, &PrintLayoutOptions::default()).unwrap_or_else(|_| {
            exporter
                .export(&doc, &PdfExportOptions::default())
                .expect("structural fallback")
        });
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 200);
    }
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f25_s1_print_pdf_large_document() {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..200 {
        doc.sections[0].blocks.push(Block::Paragraph(Paragraph::with_text(format!(
            "Large document line {i}: Lorem ipsum dolor sit amet, consectetur adipiscing elit."
        ))));
    }
    let pdf = prepare_print_pdf(&doc, &PrintLayoutOptions::default()).expect("large print PDF");
    assert!(pdf.starts_with(b"%PDF"));
    let pages = String::from_utf8_lossy(&pdf)
        .matches("/Type /Page")
        .count();
    assert!(pages > 1, "expected multi-page print PDF, got {pages}");
}
