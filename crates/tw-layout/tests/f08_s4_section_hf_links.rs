//! F08.S4 — per-section header/footer link-to-previous resolution.

#[path = "f08_layout_helpers/mod.rs"]
mod helpers;

use helpers::{
    header_text, layout, margin_top, header_with_text, two_section_paginated_document,
};

use tw_model::HeaderFooterType;

fn section_two_first_page<'a>(
    document_layout: &'a tw_layout::DocumentLayout,
) -> Option<&'a tw_layout::PageLayout> {
    document_layout.pages.get(1)
}

#[test]
fn u_f08_s4_linked_inherits_previous_header() {
    let doc = two_section_paginated_document("Header Alpha");
    let document_layout = layout(&doc);
    let page = section_two_first_page(&document_layout).expect("section two should paginate");
    let margin = margin_top(&doc);
    assert!(header_text(page, margin).contains("Alpha"));
}

#[test]
fn u_f08_s4_unlinked_distinct_header() {
    let mut doc = two_section_paginated_document("Header Alpha");
    doc.sections[1].header_links.set_linked(HeaderFooterType::Default, false);
    doc.sections[1].headers.insert(
        HeaderFooterType::Default,
        header_with_text("Header Beta"),
    );

    let document_layout = layout(&doc);
    let page = section_two_first_page(&document_layout).expect("section two should paginate");
    let margin = margin_top(&doc);
    let text = header_text(page, margin);
    assert!(text.contains("Beta"));
    assert!(!text.contains("Alpha"));
}
