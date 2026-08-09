//! F08.S4 — linked sections omit header references in export.

#[path = "f08_docx_helpers/mod.rs"]
mod helpers;

use helpers::document_xml;
use tw_docx::{export_docx, DocxPackage};
use tw_model::{
    Block, Document, HeaderFooter, HeaderFooterLinks, HeaderFooterType, Paragraph, Section,
};

#[test]
fn u_f08_s4_unlinked_section_exports_header_reference() {
    let mut doc = Document::from_plain_text("One\nTwo");
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        HeaderFooter {
            blocks: vec![Block::Paragraph(Paragraph::with_text("Header"))],
            plain_text: None,
        },
    );

    let mut section_two = Section::new();
    section_two.header_links.set_linked(HeaderFooterType::Default, false);
    section_two.headers.insert(
        HeaderFooterType::Default,
        HeaderFooter {
            blocks: vec![Block::Paragraph(Paragraph::with_text("Other"))],
            plain_text: None,
        },
    );
    section_two.blocks = vec![Block::Paragraph(Paragraph::with_text("Two"))];
    doc.sections.push(section_two);

    let bytes = export_docx(&doc, &DocxPackage::minimal()).unwrap();
    assert_eq!(document_xml(&bytes).matches("<w:headerReference").count(), 2);
}

#[test]
fn u_f08_s4_linked_section_omits_header_reference() {
    let mut doc = Document::from_plain_text("One\nTwo");
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        HeaderFooter {
            blocks: vec![Block::Paragraph(Paragraph::with_text("Header"))],
            plain_text: None,
        },
    );

    let mut section_two = Section::new();
    section_two.header_links = HeaderFooterLinks::linked_to_previous();
    section_two.footer_links = HeaderFooterLinks::linked_to_previous();
    section_two.blocks = vec![Block::Paragraph(Paragraph::with_text("Two"))];
    doc.sections.push(section_two);

    let bytes = export_docx(&doc, &DocxPackage::minimal()).unwrap();
    assert_eq!(document_xml(&bytes).matches("<w:headerReference").count(), 1);
}
