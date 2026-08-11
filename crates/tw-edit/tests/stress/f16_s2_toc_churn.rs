//! Stress: repeated TOC insert and DOCX round-trip with many headings.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::TOC_TITLE;

fn add_heading(session: &mut EditSession, after_id: tw_model::NodeId, text: &str) -> tw_model::NodeId {
    session
        .apply(Command::InsertParagraph { after_id: after_id })
        .unwrap();
    let para = session.document.sections[0].blocks.iter().rev().find_map(|b| b.paragraph()).unwrap();
    let para_id = para.id;
    let run_id = para.runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: text.into(),
        })
        .unwrap();
    session
        .apply(Command::ApplyParagraphStyle {
            paragraph_id: para_id,
            style_name: "Heading 1".into(),
        })
        .unwrap();
    para_id
}

fn tail_block_id(session: &EditSession) -> tw_model::NodeId {
    session.document.sections[0].blocks.last().and_then(|b| match b {
        tw_model::Block::Paragraph(p) => Some(p.id),
        tw_model::Block::Table(t) => Some(t.id),
        tw_model::Block::ImageBlock(i) => Some(i.id),
        tw_model::Block::ShapeBlock(s) => Some(s.id),
        _ => None,
    }).expect("tail block")
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f16_s2_toc_insert_churn() {
    let mut session = EditSession::new();
    let mut after = session.document.paragraph_at(0, 0).unwrap().id;

    for i in 0..25 {
        after = add_heading(&mut session, after, &format!("Section {i}"));
    }

    for round in 0..5 {
        session
            .apply(Command::InsertTableOfContents {
                after_block_id: tail_block_id(&session),
                page_numbers: (1..=25).map(|p| ((p - 1) / 5 + 1) as u32).collect(),
            })
            .unwrap_or_else(|_| panic!("toc insert round {round}"));
    }

    let toc_count = session
        .document
        .sections[0]
        .blocks
        .iter()
        .filter(|b| {
            b.paragraph()
                .is_some_and(|p| p.full_text().contains(TOC_TITLE))
        })
        .count();
    assert_eq!(toc_count, 5, "expected five TOC title blocks");
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f16_s2_toc_docx_roundtrip() {
    let mut session = EditSession::new();
    let first = session.document.paragraph_at(0, 0).unwrap().id;
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Overview".into(),
        })
        .unwrap();
    session
        .apply(Command::ApplyParagraphStyle {
            paragraph_id: first,
            style_name: "Heading 1".into(),
        })
        .unwrap();

    for i in 1..=10 {
        add_heading(&mut session, tail_block_id(&session), &format!("Topic {i}"));
    }

    session
        .apply(Command::InsertTableOfContents {
            after_block_id: tail_block_id(&session),
            page_numbers: vec![1; 11],
        })
        .unwrap();

    for _ in 0..3 {
        let package = tw_docx::DocxPackage::default();
        let bytes = export(&session.document, &package).expect("export");
        let imported = import(&bytes).expect("import");
        let titles = imported
            .document
            .sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .filter(|p| p.full_text().contains(TOC_TITLE))
            .count();
        assert_eq!(titles, 1);
        session.document = imported.document;
    }
}
