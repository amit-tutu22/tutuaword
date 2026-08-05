//! Display list includes decoration rects from formatted text lines.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, CharFormat, Color, Document, Paragraph, UnderlineStyle};
use tw_render::DisplayListBuilder;

#[test]
fn underlined_text_emits_rect_batch_entries() {
    let mut doc = Document::new();
    let mut run = tw_model::Run::new_text("Line");
    run.format = CharFormat {
        underline: Some(UnderlineStyle::Single),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![run],
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = layout.pages.first().unwrap();
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);

    assert!(
        !list.rect_batch.rects.is_empty(),
        "underline decorations should produce rect batch entries"
    );

    let bytes = DisplayListBuilder::to_bytes(&list);
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(!decoded.rect_batch.rects.is_empty());
}

#[test]
fn highlighted_text_emits_background_rects() {
    let mut doc = Document::new();
    let mut run = tw_model::Run::new_text("Mark");
    run.format = CharFormat {
        highlight: Some(Color {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        }),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![run],
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let has_highlight = layout.pages[0].boxes.iter().any(|b| {
        matches!(b, LayoutBox::TextLine(line) if !line.decorations.is_empty())
    });
    assert!(has_highlight);

    let page = layout.pages.first().unwrap();
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 2);
    assert!(!list.rect_batch.rects.is_empty());
}
