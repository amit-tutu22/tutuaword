//! Stress: repeated section header link/unlink toggles.

#[path = "../f08_edit_helpers/mod.rs"]
mod helpers;

use helpers::{header_with_text, insert_section_break_after_first};
use tw_edit::{Command, EditSession};
use tw_model::{Document, HeaderFooterType};

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f08_s4_section_header_link_churn() {
    let mut doc = Document::from_plain_text("One\nTwo");
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text("Shared header"),
    );
    let mut session = EditSession::from_document(doc);
    insert_section_break_after_first(&mut session);

    for _ in 0..100 {
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
        assert_eq!(
            session
                .document
                .resolved_header(1, HeaderFooterType::Default)
                .and_then(|hf| hf.blocks.first()?.paragraph().map(|p| p.full_text()))
                .unwrap_or_default(),
            "Shared header"
        );
    }
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f08_s4_multi_section_break_link_chain() {
    let mut doc = Document::from_plain_text("Body");
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        header_with_text("H0"),
    );
    let mut session = EditSession::from_document(doc);

    for i in 0..20 {
        insert_section_break_after_first(&mut session);
        let section_index = session.document.sections.len() - 1;
        assert!(
            session.document.sections[section_index]
                .header_links
                .is_linked(HeaderFooterType::Default),
            "section {section_index} iteration {i} should stay linked"
        );
        let resolved = session
            .document
            .resolved_header(section_index, HeaderFooterType::Default)
            .and_then(|hf| hf.blocks.first()?.paragraph()?.runs.first())
            .map(|r| r.text())
            .unwrap_or_default();
        assert_eq!(resolved, "H0", "section {section_index} iteration {i}");
    }

    assert!(session.document.sections.len() >= 21);
}
