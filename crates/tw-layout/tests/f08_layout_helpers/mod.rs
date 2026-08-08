//! Shared helpers for F08 header/footer layout tests.

#![allow(dead_code)]

use tw_layout::{LayoutBox, LayoutEngine, PageLayout, TextLine};
use tw_model::{
    Block, Document, HeaderFooter, HeaderFooterLinks, HeaderFooterType, Paragraph, Section,
    SectionFormat,
};

pub fn header_with_text(text: &str) -> HeaderFooter {
    HeaderFooter {
        blocks: vec![Block::Paragraph(Paragraph::with_text(text))],
        plain_text: None,
    }
}

pub fn paginated_document(paragraphs: usize) -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..paragraphs {
        doc.sections[0].blocks.push(Block::Paragraph(Paragraph::with_text(format!(
            "Paragraph {i} with enough text to fill vertical space on the page."
        ))));
    }
    doc
}

pub fn two_section_paginated_document(header_text: &str) -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text("Section one."))];
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text(header_text),
    );

    let mut section_two = Section::new();
    section_two.header_links = HeaderFooterLinks::linked_to_previous();
    section_two.footer_links = HeaderFooterLinks::linked_to_previous();
    section_two.blocks.clear();
    for i in 0..80 {
        section_two.blocks.push(Block::Paragraph(Paragraph::with_text(format!(
            "Section two paragraph {i} with enough text to paginate."
        ))));
    }
    doc.sections.push(section_two);
    doc
}

pub fn layout(doc: &Document) -> tw_layout::DocumentLayout {
    LayoutEngine::new().layout_document(doc)
}

pub fn margin_top(doc: &Document) -> f32 {
    doc.sections
        .first()
        .map(|s| s.format.margin_top)
        .unwrap_or_else(|| SectionFormat::default().margin_top)
}

pub fn header_line<'a>(page: &'a PageLayout, margin_top: f32) -> Option<&'a TextLine> {
    page.boxes.iter().find_map(|b| match b {
        LayoutBox::TextLine(line) if line.y < margin_top => Some(line),
        _ => None,
    })
}

pub fn header_text(page: &PageLayout, margin_top: f32) -> String {
    header_line(page, margin_top)
        .map(|line| line.glyphs.iter().map(|g| g.codepoint).collect())
        .unwrap_or_default()
}

pub fn header_contains(page: &PageLayout, margin_top: f32, needle: &str) -> bool {
    header_text(page, margin_top).contains(needle)
}
