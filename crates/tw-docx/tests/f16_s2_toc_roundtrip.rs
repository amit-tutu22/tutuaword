//! F16.S2 — TOC paragraphs survive DOCX export/import.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::TOC_TITLE;

fn doc_with_toc() -> EditSession {
    let mut session = EditSession::new();
    let first_para = session.document.paragraph_at(0, 0).unwrap().id;
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Chapter One".into(),
        })
        .unwrap();
    session
        .apply(Command::ApplyParagraphStyle {
            paragraph_id: first_para,
            style_name: "Heading 1".into(),
        })
        .unwrap();

    session
        .apply(Command::InsertParagraph {
            after_id: first_para,
        })
        .unwrap();
    let second = session.document.paragraph_at(0, 1).unwrap();
    let second_id = second.id;
    let second_run = second.runs[0].id;
    session
        .apply(Command::InsertText {
            run_id: second_run,
            offset: 0,
            text: "Section A".into(),
        })
        .unwrap();
    session
        .apply(Command::ApplyParagraphStyle {
            paragraph_id: second_id,
            style_name: "Heading 2".into(),
        })
        .unwrap();

    session
        .apply(Command::InsertTableOfContents {
            after_block_id: second_id,
            page_numbers: vec![1, 1],
        })
        .unwrap();
    session
}

#[test]
fn u_f16_s2_toc_docx_roundtrip() {
    let session = doc_with_toc();
    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");

    let imported = import(&bytes).expect("import");
    let texts: Vec<String> = imported
        .document
        .sections
        .iter()
        .flat_map(|s| &s.blocks)
        .filter_map(|b| b.paragraph())
        .map(|p| p.full_text())
        .collect();

    assert!(
        texts.iter().any(|t| t.contains(TOC_TITLE)),
        "TOC title missing after round-trip"
    );
    assert!(
        texts.iter().any(|t| t.contains("Chapter One")),
        "H1 TOC entry missing"
    );
    assert!(
        texts.iter().any(|t| t.contains("Section A")),
        "H2 TOC entry missing"
    );
}
