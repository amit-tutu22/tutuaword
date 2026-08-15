use tw_edit::{Command, EditSession};

#[test]
fn paste_multiline_plain_creates_paragraphs_not_newline_glyphs() {
    let mut session = EditSession::new();
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Assess \u{F0E0} Migrate\nHyperSDK\nSecure".into(),
        })
        .unwrap();

    let paras: Vec<_> = session
        .document
        .sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph())
        .collect();
    assert!(paras.len() >= 3, "expected paragraph splits, got {}", paras.len());
    let joined: String = paras.iter().map(|p| p.full_text()).collect::<Vec<_>>().join("|");
    assert!(joined.contains("Assess"), "{joined}");
    assert!(joined.contains("Migrate"), "{joined}");
    assert!(joined.contains("HyperSDK"), "{joined}");
    assert!(joined.contains("Secure"), "{joined}");
    assert!(!joined.contains('\n'), "literal newlines must not remain: {joined}");
    assert!(!joined.contains('\u{F0E0}'), "PUA must be remapped: {joined}");
    assert!(joined.contains('→'), "arrow PUA should become →: {joined}");
}
