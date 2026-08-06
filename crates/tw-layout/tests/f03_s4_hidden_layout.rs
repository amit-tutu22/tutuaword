//! F03.S4 — hidden runs are skipped during layout.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, CharFormat, Document, Paragraph};

#[test]
fn hidden_runs_produce_no_glyphs() {
    let mut doc = Document::new();
    let mut hidden = tw_model::Run::new_text("Secret");
    hidden.format = CharFormat {
        hidden: Some(true),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![hidden],
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let glyph_count: usize = layout.pages[0]
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.glyphs.len()),
            _ => None,
        })
        .sum();

    assert_eq!(glyph_count, 0, "hidden text should not emit layout glyphs");
}

#[test]
fn visible_runs_still_layout_when_mixed_with_hidden() {
    let mut doc = Document::new();
    let visible = tw_model::Run::new_text("Hi");
    let mut hidden = tw_model::Run::new_text("X");
    hidden.format.hidden = Some(true);
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![visible, hidden],
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let line = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l),
            _ => None,
        })
        .expect("visible line");

    assert!(!line.glyphs.is_empty());
    assert!(line.width > 0.0);
}
