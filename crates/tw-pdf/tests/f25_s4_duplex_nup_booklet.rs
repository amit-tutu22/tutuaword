//! F25.S4 — duplex / N-up / booklet (unit + stress).

use tw_model::{Block, Document, Paragraph};
use tw_pdf::{
    booklet_page_order, build_print_sheet_plan, normalize_pages_per_sheet, nup_cell_transform,
    nup_grid, prepare_print_pdf, PrintDuplexMode, PrintLayoutOptions,
};

fn multi_page_doc(pages: usize) -> Document {
    let mut doc = Document::new();
    // Force multiple layout pages with page breaks via long paragraphs + page breaks.
    doc.sections[0].blocks.clear();
    for i in 0..pages {
        doc.sections[0]
            .blocks
            .push(Block::Paragraph(Paragraph::with_text(format!(
                "Sheet page {i}. {}",
                "word ".repeat(80)
            ))));
        if i + 1 < pages {
            doc.sections[0]
                .blocks
                .push(Block::Paragraph(Paragraph::with_text("\u{000C}")));
        }
    }
    doc
}

#[test]
fn u_f25_s4_normalize_pages_per_sheet() {
    assert_eq!(normalize_pages_per_sheet(0), 1);
    assert_eq!(normalize_pages_per_sheet(1), 1);
    assert_eq!(normalize_pages_per_sheet(2), 2);
    assert_eq!(normalize_pages_per_sheet(3), 4);
    assert_eq!(normalize_pages_per_sheet(4), 4);
    assert_eq!(normalize_pages_per_sheet(6), 6);
    assert_eq!(normalize_pages_per_sheet(9), 9);
    assert_eq!(normalize_pages_per_sheet(12), 16);
}

#[test]
fn u_f25_s4_nup_grid() {
    assert_eq!(nup_grid(1), (1, 1));
    assert_eq!(nup_grid(2), (2, 1));
    assert_eq!(nup_grid(4), (2, 2));
    assert_eq!(nup_grid(6), (3, 2));
    assert_eq!(nup_grid(9), (3, 3));
    assert_eq!(nup_grid(16), (4, 4));
}

#[test]
fn u_f25_s4_booklet_page_order_eight() {
    let order = booklet_page_order(8);
    assert_eq!(
        order,
        vec![
            Some(7),
            Some(0),
            Some(1),
            Some(6),
            Some(5),
            Some(2),
            Some(3),
            Some(4),
        ]
    );
}

#[test]
fn u_f25_s4_booklet_pads_to_multiple_of_four() {
    let order = booklet_page_order(5);
    assert_eq!(order.len(), 8);
    assert!(order.iter().filter(|s| s.is_none()).count() >= 1);
    assert_eq!(order.iter().filter_map(|s| *s).count(), 5);
}

#[test]
fn u_f25_s4_sheet_plan_nup_four() {
    let layout = PrintLayoutOptions::with_nup(4);
    let plan = build_print_sheet_plan(5, &layout);
    assert_eq!(plan.pages_per_sheet, 4);
    assert_eq!(plan.cols, 2);
    assert_eq!(plan.rows, 2);
    assert_eq!(plan.sheets.len(), 2);
    assert_eq!(
        plan.sheets[0].slots,
        vec![Some(0), Some(1), Some(2), Some(3)]
    );
    assert_eq!(plan.sheets[1].slots, vec![Some(4), None, None, None]);
}

#[test]
fn u_f25_s4_sheet_plan_booklet_forces_duplex_long_edge() {
    let layout = PrintLayoutOptions::with_booklet();
    let plan = build_print_sheet_plan(3, &layout);
    assert!(plan.booklet);
    assert_eq!(plan.pages_per_sheet, 2);
    assert_eq!(plan.duplex, PrintDuplexMode::LongEdge);
    assert_eq!(plan.sheets.len(), 2); // 4 padded slots → 2 sheets of 2
}

#[test]
fn u_f25_s4_nup_cell_transform_two_up() {
    let left = nup_cell_transform(612.0, 792.0, 2, 1, 0, 612.0, 792.0);
    let right = nup_cell_transform(612.0, 792.0, 2, 1, 1, 612.0, 792.0);
    assert!((left.scale - 0.5).abs() < 1e-4);
    assert!(left.tx < right.tx);
}

#[test]
fn u_f25_s4_prepare_print_pdf_nup_header() {
    let doc = Document::with_paragraph("N-up sample page content for printing.");
    let layout = PrintLayoutOptions::with_nup(2);
    let pdf = prepare_print_pdf(&doc, &layout).expect("n-up PDF");
    assert!(pdf.starts_with(b"%PDF"));
}

#[test]
fn u_f25_s4_prepare_print_pdf_booklet_header() {
    let doc = multi_page_doc(3);
    let layout = PrintLayoutOptions::with_booklet();
    let pdf = prepare_print_pdf(&doc, &layout).expect("booklet PDF");
    assert!(pdf.starts_with(b"%PDF"));
    // Booklet pads to 4 logical pages → 2 physical sheets.
    let s = String::from_utf8_lossy(&pdf);
    let count = s.matches("/Type /Page ").count();
    assert!(count >= 1, "expected imposed sheet pages, got {count}");
}

#[test]
fn u_f25_s4_effective_duplex_booklet() {
    let layout = PrintLayoutOptions {
        duplex: PrintDuplexMode::Simplex,
        booklet: true,
        ..Default::default()
    };
    assert_eq!(layout.effective_duplex(), PrintDuplexMode::LongEdge);
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f25_s4_sheet_plan_churn() {
    for pages in 1..=64 {
        for nup in [1u8, 2, 4, 6, 9, 16] {
            let layout = PrintLayoutOptions::with_nup(nup);
            let plan = build_print_sheet_plan(pages, &layout);
            let filled: usize = plan
                .sheets
                .iter()
                .flat_map(|s| s.slots.iter())
                .filter(|s| s.is_some())
                .count();
            assert_eq!(filled, pages, "pages={pages} nup={nup}");
        }
        let booklet = build_print_sheet_plan(pages, &PrintLayoutOptions::with_booklet());
        let filled: usize = booklet
            .sheets
            .iter()
            .flat_map(|s| s.slots.iter())
            .filter(|s| s.is_some())
            .count();
        assert_eq!(filled, pages, "booklet pages={pages}");
    }
}
