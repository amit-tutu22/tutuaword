//! F03.S4 — hidden text excluded from export plaintext.

use tw_core::document_plain_text;
use tw_model::{Block, CharFormat, Document, Paragraph};

/// U-F03-S4-hidden-not-in-plaintext: hidden runs are omitted from export plaintext.
#[test]
fn u_f03_s4_hidden_not_in_plaintext() {
    let mut doc = Document::new();
    let visible = tw_model::Run::new_text("Visible");
    let mut hidden = tw_model::Run::new_text("Secret");
    hidden.format = CharFormat {
        hidden: Some(true),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![visible, hidden],
    });

    let para = doc.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.full_text(), "VisibleSecret");
    assert_eq!(para.visible_text(), "Visible");
    assert_eq!(document_plain_text(&doc), "Visible");
}
