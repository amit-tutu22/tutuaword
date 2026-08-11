//! F16.S2 — session-style TOC insert with layout-derived page numbers.

use std::sync::Arc;

use tw_core::{LayoutCache, SyncSession};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph, TOC_TITLE};

fn paged_heading_document() -> Document {
    let mut doc = Document::new();

    let mut h1 = Paragraph::with_text("Chapter One");
    if let Some(style) = doc.styles.find_style_by_name("Heading 1") {
        h1.style_id = Some(style.id);
    }

    let mut filler = Paragraph::with_text("Body ".repeat(800));
    filler.format.page_break_before = Some(true);

    let mut h2 = Paragraph::with_text("Chapter Two");
    if let Some(style) = doc.styles.find_style_by_name("Heading 1") {
        h2.style_id = Some(style.id);
    }
    h2.format.page_break_before = Some(true);

    doc.sections[0].blocks = vec![
        Block::Paragraph(h1),
        Block::Paragraph(filler),
        Block::Paragraph(h2),
    ];
    doc
}

fn outline_page_numbers(session: &SyncSession) -> Vec<u32> {
    let mut cache = LayoutCache::default();
    cache.update_from_session(
        &session.layout,
        Arc::new(session.edit.document.clone()),
    );
    cache
        .document_outline_with_pages()
        .into_iter()
        .map(|(_, page)| page.max(1))
        .collect()
}

#[test]
fn u_f16_s2_session_toc_page_numbers() {
    let mut session = SyncSession::new();
    session.edit.document = paged_heading_document();
    session.relayout(None);

    assert!(
        session.page_count() >= 2,
        "fixture should span at least two pages, got {}",
        session.page_count()
    );

    let page_numbers = outline_page_numbers(&session);
    assert_eq!(page_numbers.len(), 2);

    let tail_block = session
        .edit
        .document
        .sections[0]
        .blocks
        .last()
        .and_then(|b| b.paragraph())
        .map(|p| p.id)
        .expect("tail block");

    session.apply(Command::InsertTableOfContents {
        after_block_id: tail_block,
        page_numbers,
    });

    let toc_paras: Vec<_> = session
        .edit
        .document
        .sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph())
        .filter(|p| {
            p.full_text().contains(TOC_TITLE)
                || p.runs
                    .iter()
                    .any(|r| matches!(r.content, tw_model::RunContent::Tab))
        })
        .collect();

    assert!(
        toc_paras.iter().any(|p| p.full_text().contains(TOC_TITLE)),
        "TOC title missing"
    );

    let ch1 = toc_paras
        .iter()
        .find(|p| p.full_text().contains("Chapter One"))
        .expect("Chapter One entry");
    assert!(
        ch1.full_text().contains('1'),
        "Chapter One should reference page 1, got {}",
        ch1.full_text()
    );

    let ch2 = toc_paras
        .iter()
        .find(|p| p.full_text().contains("Chapter Two"))
        .expect("Chapter Two entry");
    assert!(
        ch2.full_text().contains('2') || ch2.full_text().contains('3'),
        "Chapter Two should reference a later page, got {}",
        ch2.full_text()
    );
}
