//! F16.S2 — insert table of contents from heading outline.

use tw_edit::{Command, EditSession};
use tw_model::{RunContent, TOC_TITLE};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id
}

fn add_heading_paragraph(
    session: &mut EditSession,
    after_para_id: tw_model::NodeId,
    style_name: &str,
    text: &str,
) -> tw_model::NodeId {
    session
        .apply(Command::InsertParagraph { after_id: after_para_id })
        .unwrap();
    let para = session.document.paragraph_at(0, 1).unwrap();
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
            style_name: style_name.into(),
        })
        .unwrap();
    para_id
}

fn doc_with_headings(session: &mut EditSession) -> tw_model::NodeId {
    let first_para = session.document.paragraph_at(0, 0).unwrap().id;
    let run_id = first_run(session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Introduction".into(),
        })
        .unwrap();
    session
        .apply(Command::ApplyParagraphStyle {
            paragraph_id: first_para,
            style_name: "Heading 1".into(),
        })
        .unwrap();
    add_heading_paragraph(session, first_para, "Heading 2", "Background")
}

fn toc_blocks<'a>(session: &'a EditSession) -> impl Iterator<Item = &'a tw_model::Paragraph> + 'a {
    session.document.sections[0].blocks.iter().filter_map(|block| {
        let para = block.paragraph()?;
        if para.full_text().contains(TOC_TITLE)
            || para.runs.iter().any(|run| matches!(run.content, RunContent::Tab))
        {
            Some(para)
        } else {
            None
        }
    })
}

#[test]
fn u_f16_s2_insert_toc_from_headings() {
    let mut session = EditSession::new();
    let after = doc_with_headings(&mut session);

    session
        .apply(Command::InsertTableOfContents {
            after_block_id: after,
            page_numbers: vec![1, 1],
        })
        .unwrap();

    let toc_texts: Vec<String> = toc_blocks(&session).map(|p| p.full_text()).collect();
    assert!(
        toc_texts.iter().any(|t| t.contains(TOC_TITLE)),
        "expected TOC title paragraph"
    );
    assert!(
        toc_texts.iter().any(|t| t.contains("Introduction")),
        "expected H1 entry"
    );
    assert!(
        toc_texts.iter().any(|t| t.contains("Background")),
        "expected H2 entry"
    );

    let h2 = toc_texts
        .iter()
        .find(|t| t.contains("Background"))
        .expect("H2 toc line");
    let h2_para = toc_blocks(&session)
        .find(|p| p.full_text().contains("Background"))
        .expect("H2 para");
    assert!(
        h2_para.format.indent_left.unwrap_or(0.0) > 0.0,
        "H2 entry should be indented"
    );
    assert!(h2.contains('\t'), "TOC entry should use tab before page number");
}

#[test]
fn u_f16_s2_toc_page_numbers() {
    let mut session = EditSession::new();
    let after = doc_with_headings(&mut session);

    session
        .apply(Command::InsertTableOfContents {
            after_block_id: after,
            page_numbers: vec![1, 3],
        })
        .unwrap();

    let intro = toc_blocks(&session)
        .find(|p| p.full_text().contains("Introduction"))
        .expect("intro toc");
    assert!(
        intro.full_text().contains("1"),
        "Introduction should show page 1, got {}",
        intro.full_text()
    );

    let background = toc_blocks(&session)
        .find(|p| p.full_text().contains("Background"))
        .expect("background toc");
    assert!(
        background.full_text().ends_with('3'),
        "Background should show page 3, got {}",
        background.full_text()
    );
}

#[test]
fn u_f16_s2_toc_empty_outline_still_has_title() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertTableOfContents {
            after_block_id: after,
            page_numbers: vec![],
        })
        .unwrap();

    let title = toc_blocks(&session)
        .find(|p| p.full_text().contains(TOC_TITLE))
        .expect("title only TOC");
    assert_eq!(title.full_text(), TOC_TITLE);
}
