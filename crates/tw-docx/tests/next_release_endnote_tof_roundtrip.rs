//! Layer 1 — endnote and table-of-figures DOCX round-trip.

use tw_docx::{export, import};
use tw_edit::{apply, Command, EditSession};
use tw_model::{RunContent, TOF_TITLE};

fn doc_with_endnote() -> EditSession {
    let mut session = EditSession::new();
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    apply(
        &mut session.document,
        Command::InsertEndnote {
            run_id,
            offset: 0,
        },
    )
    .expect("insert endnote");
    session
}

#[test]
fn u_layer1_endnote_docx_roundtrip() {
    let session = doc_with_endnote();
    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let imported = import(&bytes).expect("import");

    assert_eq!(imported.document.endnotes.len(), 1);
    let para = imported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    assert!(
        para.runs
            .iter()
            .any(|r| matches!(r.content, RunContent::EndnoteRef(_))),
        "endnote reference missing after round-trip"
    );
}

fn doc_with_tof() -> EditSession {
    let mut session = EditSession::new();
    let caption_style = session
        .document
        .styles
        .find_style_by_name("Caption")
        .unwrap()
        .id;
    session.document.sections[0].blocks.push(tw_model::Block::Paragraph({
        let mut p = tw_model::Paragraph::with_text("Figure 1 Example chart");
        p.style_id = Some(caption_style);
        p
    }));
    let after = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;
    apply(
        &mut session.document,
        Command::InsertTableOfFigures {
            after_block_id: after,
            page_numbers: vec![1],
        },
    )
    .expect("insert tof");
    session
}

#[test]
fn u_layer1_tof_docx_roundtrip() {
    let session = doc_with_tof();
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
        texts.iter().any(|t| t.contains(TOF_TITLE)),
        "Table of Figures title missing after round-trip"
    );
    assert!(
        texts.iter().any(|t| t.contains("Figure 1 Example chart")),
        "ToF caption entry missing after round-trip"
    );

    let has_tof_field = imported.document.sections.iter().any(|section| {
        section.blocks.iter().any(|block| {
            block.paragraph().is_some_and(|para| {
                para.runs.iter().any(|run| {
                    matches!(
                        &run.content,
                        RunContent::Field(f)
                            if f.field_type == tw_model::FieldType::TableOfFigures
                    )
                })
            })
        })
    });
    assert!(has_tof_field, "ToF fldSimple field missing after round-trip");
}
