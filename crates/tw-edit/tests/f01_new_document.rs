//! F01.S1 — empty document baseline.

use tw_edit::EditSession;

/// U-F01-S1-new-document-empty
#[test]
fn u_f01_s1_new_document_empty() {
    let session = EditSession::new();
    assert_eq!(session.document.sections.len(), 1);
    assert_eq!(session.document.sections[0].blocks.len(), 1);
    let para = session
        .document
        .sections[0]
        .blocks[0]
        .paragraph()
        .expect("default block is a paragraph");
    assert_eq!(para.full_text(), "");
    assert_eq!(para.runs.len(), 1);
}
