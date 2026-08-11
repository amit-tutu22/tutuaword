//! F08 — header/footer variant selection and link-chain resolution.

use tw_model::{
    Block, Document, HeaderFooter, HeaderFooterLinks, HeaderFooterType, Paragraph, Section,
};

fn header_with_text(text: &str) -> HeaderFooter {
    HeaderFooter {
        blocks: vec![Block::Paragraph(Paragraph::with_text(text))],
        plain_text: None,
    }
}

#[test]
fn u_f08_hf_variant_first_page_wins_over_odd_even() {
    assert_eq!(
        HeaderFooterType::for_page_layout(1, true, true, true),
        HeaderFooterType::First
    );
    assert_eq!(
        HeaderFooterType::for_page_layout(2, false, true, true),
        HeaderFooterType::Even
    );
    assert_eq!(
        HeaderFooterType::for_page_layout(3, false, false, true),
        HeaderFooterType::Odd
    );
    assert_eq!(
        HeaderFooterType::for_page_layout(2, false, false, false),
        HeaderFooterType::Default
    );
}

#[test]
fn u_f08_link_chain_resolves_previous_section_header() {
    let mut doc = Document::new();
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text("Shared"),
    );

    let mut section_two = Section::new();
    section_two.header_links = HeaderFooterLinks::linked_to_previous();
    section_two.blocks = vec![Block::Paragraph(Paragraph::with_text("Two"))];
    doc.sections.push(section_two);

    assert_eq!(doc.header_source_section(1, HeaderFooterType::Default), 0);
    assert_eq!(
        doc.resolved_header(1, HeaderFooterType::Default)
            .and_then(|hf| hf.blocks.first()?.paragraph()?.runs.first())
            .map(|r| r.text()),
        Some("Shared")
    );
}
