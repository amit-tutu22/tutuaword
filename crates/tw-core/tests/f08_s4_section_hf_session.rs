//! F08.S4 — SyncSession section header link resolution.

use tw_core::SyncSession;
use tw_edit::Command;
use tw_layout::LayoutEngine;
use tw_model::{
    Block, Document, HeaderFooter, HeaderFooterLinks, HeaderFooterType, Paragraph, Section,
};

fn header_with_text(text: &str) -> HeaderFooter {
    HeaderFooter {
        blocks: vec![Block::Paragraph(Paragraph::with_text(text))],
        plain_text: None,
    }
}

fn two_section_document() -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text("Section one."))];
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text("Header Alpha"),
    );

    let mut section_two = Section::new();
    section_two.header_links = HeaderFooterLinks::linked_to_previous();
    section_two.footer_links = HeaderFooterLinks::linked_to_previous();
    for i in 0..80 {
        section_two.blocks.push(Block::Paragraph(Paragraph::with_text(format!(
            "Section two paragraph {i} with enough text to paginate."
        ))));
    }
    doc.sections.push(section_two);
    doc
}

fn header_on_page(session: &SyncSession, page_index: usize) -> String {
    let layout = LayoutEngine::new().layout_document(&session.edit.document);
    let page = layout.pages.get(page_index).expect("page");
    let margin_top = session.edit.document.sections[0].format.margin_top;
    page.boxes.iter().find_map(|b| match b {
        tw_layout::LayoutBox::TextLine(line) if line.y < margin_top => {
            Some(line.glyphs.iter().map(|g| g.codepoint).collect::<String>())
        }
        _ => None,
    })
    .unwrap_or_default()
}

#[test]
fn u_f08_s4_sync_session_linked_header_on_section_two() {
    let mut session = SyncSession::new();
    session.edit.document = two_section_document();
    session.relayout(None);

    assert!(session.page_count() >= 2, "fixture should paginate");
    let text = header_on_page(&session, 1);
    assert!(text.contains("Alpha"), "linked section two should inherit header");
}

#[test]
fn u_f08_s4_sync_session_unlink_copies_previous_header() {
    let mut session = SyncSession::new();
    session.edit.document = two_section_document();
    session.apply(Command::SetHeaderFooterLink {
        section_index: 1,
        is_header: true,
        hf_type: HeaderFooterType::Default,
        linked: false,
    });

    assert!(!session.edit.document.sections[1]
        .header_links
        .is_linked(HeaderFooterType::Default));
    assert_eq!(
        session
            .edit
            .document
            .resolved_header(1, HeaderFooterType::Default)
            .and_then(|hf| hf.blocks.first()?.paragraph().map(|p| p.full_text()))
            .unwrap_or_default(),
        "Header Alpha"
    );
    assert_eq!(
        session
            .edit
            .document
            .resolved_header(0, HeaderFooterType::Default)
            .and_then(|hf| hf.blocks.first()?.paragraph().map(|p| p.full_text()))
            .unwrap_or_default(),
        "Header Alpha"
    );
}
