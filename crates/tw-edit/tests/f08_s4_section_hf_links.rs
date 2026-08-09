//! F08.S4 — section-specific header/footer link commands.

#[path = "f08_edit_helpers/mod.rs"]
mod helpers;

use helpers::{header_plain_text, header_with_text, insert_section_break_after_first};
use tw_edit::{Command, EditSession};
use tw_model::{Document, HeaderFooterType};

#[test]
fn u_f08_s4_footer_link_unlink_roundtrip() {
    let mut doc = Document::from_plain_text("One\nTwo");
    doc.sections[0].footers.insert(
        HeaderFooterType::Default,
        header_with_text("Footer A"),
    );
    let mut session = EditSession::from_document(doc);
    insert_section_break_after_first(&mut session);

    session
        .apply(Command::SetHeaderFooterLink {
            section_index: 1,
            is_header: false,
            hf_type: HeaderFooterType::Default,
            linked: false,
        })
        .unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "One"
    );
    assert!(!session.document.sections[1]
        .footer_links
        .is_linked(HeaderFooterType::Default));
    assert!(session.document.sections[1].footers.contains_key(&HeaderFooterType::Default));
}

#[test]
fn u_f08_s4_section_break_defaults_to_linked() {
    let mut session = EditSession::from_document(Document::from_plain_text("One\nTwo"));
    insert_section_break_after_first(&mut session);
    assert!(session.document.sections[1]
        .header_links
        .is_linked(HeaderFooterType::Default));
}

#[test]
fn u_f08_s4_unlink_copies_previous_header() {
    let mut doc = Document::from_plain_text("One\nTwo");
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text("Shared header"),
    );
    let mut session = EditSession::from_document(doc);
    insert_section_break_after_first(&mut session);

    session
        .apply(Command::SetHeaderFooterLink {
            section_index: 1,
            is_header: true,
            hf_type: HeaderFooterType::Default,
            linked: false,
        })
        .unwrap();

    assert!(!session.document.sections[1]
        .header_links
        .is_linked(HeaderFooterType::Default));
    assert_eq!(
        header_plain_text(&session.document, 1, HeaderFooterType::Default),
        "Shared header"
    );
}

#[test]
fn u_f08_s4_link_removes_local_copy() {
    let mut doc = Document::from_plain_text("One\nTwo");
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text("Shared header"),
    );
    let mut session = EditSession::from_document(doc);
    insert_section_break_after_first(&mut session);
    session
        .apply(Command::SetHeaderFooterLink {
            section_index: 1,
            is_header: true,
            hf_type: HeaderFooterType::Default,
            linked: false,
        })
        .unwrap();
    assert!(session.document.sections[1].headers.contains_key(&HeaderFooterType::Default));

    session
        .apply(Command::SetHeaderFooterLink {
            section_index: 1,
            is_header: true,
            hf_type: HeaderFooterType::Default,
            linked: true,
        })
        .unwrap();
    assert!(session.document.sections[1]
        .header_links
        .is_linked(HeaderFooterType::Default));
    assert!(!session.document.sections[1].headers.contains_key(&HeaderFooterType::Default));
}
