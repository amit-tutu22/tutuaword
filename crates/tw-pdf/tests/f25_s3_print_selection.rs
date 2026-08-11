//! F25.S3 — print selection PDF (unit + stress).

use tw_edit::{DocPosition, DocRange};
use tw_model::{Block, Document, Paragraph};
use tw_pdf::{prepare_print_pdf, prepare_print_pdf_selection, PrintLayoutOptions};

fn multi_para_doc() -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(Paragraph::with_text("First paragraph for print.")),
        Block::Paragraph(Paragraph::with_text("Second paragraph KEEP ME.")),
        Block::Paragraph(Paragraph::with_text("Third paragraph discard.")),
    ];
    doc
}

#[test]
fn u_f25_s3_prepare_print_pdf_selection_header() {
    let doc = multi_para_doc();
    let run_id = doc.paragraph_at(0, 1).unwrap().runs[0].id;
    let range = DocRange {
        start: DocPosition {
            run_id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id,
            char_offset: 16,
        },
    };
    let pdf = prepare_print_pdf_selection(&doc, &range, &PrintLayoutOptions::default())
        .expect("selection PDF");
    assert!(pdf.starts_with(b"%PDF"));
    assert!(pdf.len() > 64);
}

#[test]
fn u_f25_s3_selection_pdf_smaller_or_equal_full() {
    let doc = multi_para_doc();
    let full = prepare_print_pdf(&doc, &PrintLayoutOptions::default()).expect("full");
    let run_id = doc.paragraph_at(0, 1).unwrap().runs[0].id;
    let range = DocRange {
        start: DocPosition {
            run_id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id,
            char_offset: 6,
        },
    };
    let sel = prepare_print_pdf_selection(&doc, &range, &PrintLayoutOptions::default())
        .expect("selection");
    // Selection is one short word — content stream should not dwarf the full doc.
    assert!(sel.len() <= full.len() + 2048);
}

#[test]
fn i_f25_s3_selection_with_scale_layout() {
    let doc = multi_para_doc();
    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    let range = DocRange {
        start: DocPosition {
            run_id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id,
            char_offset: 5,
        },
    };
    let layout = PrintLayoutOptions::with_scale_percent(50.0);
    let pdf = prepare_print_pdf_selection(&doc, &range, &layout).expect("scaled selection");
    assert!(pdf.starts_with(b"%PDF"));
    let s = String::from_utf8_lossy(&pdf);
    assert!(
        s.contains("0.500000 0 0 0.500000") || s.contains("0.5 0 0 0.5") || s.contains(" cm\n"),
        "scaled selection PDF should include a CTM"
    );
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f25_s3_print_selection_churn() {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..40 {
        doc.sections[0]
            .blocks
            .push(Block::Paragraph(Paragraph::with_text(format!(
                "Selection churn paragraph {i} with extra words."
            ))));
    }
    for bi in 0..40 {
        let run_id = doc.paragraph_at(0, bi).unwrap().runs[0].id;
        let range = DocRange {
            start: DocPosition {
                run_id,
                char_offset: 0,
            },
            end: DocPosition {
                run_id,
                char_offset: 10,
            },
        };
        let pdf = prepare_print_pdf_selection(&doc, &range, &PrintLayoutOptions::default())
            .expect("churn");
        assert!(pdf.starts_with(b"%PDF"), "para {bi}");
    }
}
