//! Endnote insert command (next-release word gap).

use tw_edit::{apply, Command, EditSession};
use tw_model::{RunContent, Block};

#[test]
fn u_insert_endnote_adds_endnote_ref_and_body() {
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

    assert_eq!(session.document.endnotes.len(), 1);
    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert!(
        para.runs
            .iter()
            .any(|r| matches!(r.content, RunContent::EndnoteRef(_))),
        "expected endnote reference run"
    );
}

#[test]
fn u_insert_table_of_figures_from_caption() {
    let mut session = EditSession::new();
    let caption_style = session
        .document
        .styles
        .find_style_by_name("Caption")
        .unwrap()
        .id;
    session.document.sections[0].blocks.push(Block::Paragraph({
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

    let joined: String = session
        .document
        .sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph())
        .map(|p| p.full_text())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("Table of Figures"));
    assert!(joined.contains("Figure 1 Example chart"));
}
