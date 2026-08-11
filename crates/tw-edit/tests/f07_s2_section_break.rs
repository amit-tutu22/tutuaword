//! F07.S2 — section break edit command.

use tw_edit::{Command, EditSession};
use tw_model::Document;

#[test]
fn u_f07_s2_insert_section_break_splits_document() {
    let doc = Document::from_plain_text("Alpha\nBeta\nGamma");
    let after = doc.sections[0].blocks[0].paragraph().unwrap().id;

    let mut session = EditSession::from_document(doc);
    session
        .apply(Command::InsertSectionBreak { after_block_id: after })
        .unwrap();

    assert_eq!(session.document.sections.len(), 2);
    assert_eq!(session.document.sections[0].blocks.len(), 1);
    assert_eq!(session.document.sections[1].blocks.len(), 2);
}

#[test]
fn u_f07_s2_insert_section_break_undo_merges_sections() {
    let doc = Document::from_plain_text("One\nTwo");
    let after = doc.sections[0].blocks[0].paragraph().unwrap().id;

    let mut session = EditSession::from_document(doc);
    session
        .apply(Command::InsertSectionBreak { after_block_id: after })
        .unwrap();
    assert_eq!(session.document.sections.len(), 2);

    session.undo().unwrap();
    assert_eq!(session.document.sections.len(), 1);
    assert_eq!(session.document.sections[0].blocks.len(), 2);
}

#[test]
fn u_f07_s2_section_format_per_section() {
    let doc = Document::from_plain_text("Section one\nSection two");
    let after = doc.sections[0].blocks[0].paragraph().unwrap().id;

    let mut session = EditSession::from_document(doc);
    session
        .apply(Command::InsertSectionBreak { after_block_id: after })
        .unwrap();

    session
        .apply(Command::SetSectionFormat {
            section_index: 1,
            format: tw_model::SectionFormat {
                margin_left: 36.0,
                margin_right: 36.0,
                ..Default::default()
            },
        })
        .unwrap();

    assert_eq!(session.document.sections[0].format.margin_left, 72.0);
    assert_eq!(session.document.sections[1].format.margin_left, 36.0);
}
