use tw_layout::{DecorationKind, LayoutBox, LayoutEngine};
use tw_model::{Document, Paragraph, Revision, Run};

#[test]
fn deleted_revision_gets_strikethrough_and_faded_glyphs() {
    let mut doc = Document::with_paragraph("");
    let mut run = Run::new_text("deleted");
    run.revision = Some(Revision::delete("Author"));
    doc.sections[0].blocks[0] = tw_model::Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![run],
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
        .expect("text line");

    assert!(
        line.decorations
            .iter()
            .any(|d| d.kind == DecorationKind::Strikethrough),
        "deleted runs should render strikethrough"
    );
    assert!(
        line.glyphs.iter().all(|g| (g.color >> 24) < 200),
        "deleted runs should use reduced opacity"
    );
}
