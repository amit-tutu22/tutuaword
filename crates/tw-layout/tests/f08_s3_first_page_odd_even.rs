//! F08.S3 — first-page and odd/even header/footer layout (U-F08-S3-hf-variant-layout).

#[path = "f08_layout_helpers/mod.rs"]
mod helpers;

use helpers::{header_contains, header_text, layout, margin_top, header_with_text, paginated_document};

use tw_model::HeaderFooterType;

#[test]
fn u_f08_s3_first_page_header() {
    let mut doc = paginated_document(80);
    doc.sections[0].format.different_first_page = true;
    doc.sections[0].headers.insert(
        HeaderFooterType::First,
        header_with_text("First page header"),
    );
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text("Default header"),
    );

    let document_layout = layout(&doc);
    assert!(document_layout.pages.len() >= 2);
    let margin = margin_top(&doc);
    let page0 = header_text(&document_layout.pages[0], margin);
    let page1 = header_text(&document_layout.pages[1], margin);
    assert!(page0.contains("First") && page0.contains("page"));
    assert!(page1.contains("Default"));
}

#[test]
fn u_f08_s3_odd_even_headers() {
    let mut doc = paginated_document(80);
    doc.settings.even_and_odd_headers = true;
    doc.sections[0].headers.insert(HeaderFooterType::Odd, header_with_text("Odd header"));
    doc.sections[0].headers.insert(
        HeaderFooterType::Even,
        header_with_text("Even header"),
    );

    let document_layout = layout(&doc);
    assert!(document_layout.pages.len() >= 2);
    let margin = margin_top(&doc);
    assert!(header_contains(&document_layout.pages[0], margin, "Odd"));
    assert!(header_contains(&document_layout.pages[1], margin, "Even"));
}
