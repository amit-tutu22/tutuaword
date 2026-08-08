//! F08.S1 — edit header text and roundtrip via native format (I-F08-S1-edit-header).

#[path = "f08_edit_helpers/mod.rs"]
mod helpers;

use helpers::header_plain_text;
use tw_edit::{Command, EditSession};
use tw_model::{Document, HeaderFooterType};
use tw_native::NativeFormat;

#[test]
fn u_f08_s1_ensure_header_footer_creates_seed_run() {
    let mut session = EditSession::from_document(Document::new());
    let result = session
        .apply(Command::EnsureHeaderFooter {
            section_index: 0,
            is_header: true,
            hf_type: HeaderFooterType::Default,
        })
        .unwrap();

    let seed = result.seed_run_id.expect("seed run");
    assert!(session.document.sections[0].headers.contains_key(&HeaderFooterType::Default));
    assert_eq!(
        session.document.header_footer_seed_run(0, true, HeaderFooterType::Default),
        Some(seed)
    );
}

#[test]
fn i_f08_s1_edit_header_survives_save() {
    let mut session = EditSession::from_document(Document::new());
    let seed = session
        .apply(Command::EnsureHeaderFooter {
            section_index: 0,
            is_header: true,
            hf_type: HeaderFooterType::Default,
        })
        .unwrap()
        .seed_run_id
        .unwrap();

    session
        .apply(Command::InsertText {
            run_id: seed,
            offset: 0,
            text: "Confidential".into(),
        })
        .unwrap();

    assert_eq!(
        header_plain_text(&session.document, 0, HeaderFooterType::Default),
        "Confidential"
    );

    let bytes = NativeFormat::export(&session.document).unwrap();
    let loaded = NativeFormat::import(&bytes).unwrap();
    assert_eq!(
        header_plain_text(&loaded, 0, HeaderFooterType::Default),
        "Confidential"
    );
}
