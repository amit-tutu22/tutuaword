use tw_layout::{DecorationKind, LayoutBox, LayoutEngine};
use tw_model::{Document, Paragraph, Revision, Run};

#[test]
fn deleted_revision_gets_strikethrough_and_red_glyphs() {
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
        line.glyphs.iter().all(|g| g.color == 0xFFFF_0000),
        "deleted runs should use Word-default red"
    );
}

#[test]
fn inserted_revision_gets_underline_and_red_glyphs() {
    let mut doc = Document::with_paragraph("");
    let mut run = Run::new_text("added");
    run.revision = Some(Revision::insert("Reviewer"));
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
            .any(|d| d.kind == DecorationKind::Underline),
        "inserted runs should render underline"
    );
    assert!(
        line.glyphs.iter().all(|g| g.color == 0xFFFF_0000),
        "inserted runs should use Word-default red"
    );
}

#[test]
fn tracked_change_revision_emits_margin_balloon() {
    let mut doc = Document::with_paragraph("");
    let mut run = Run::new_text("change");
    run.revision = Some(Revision::insert("Author"));
    doc.sections[0].blocks[0] = tw_model::Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![run],
    });

    let layout = LayoutEngine::new().layout_document(&doc);
    let has_balloon = layout.pages[0].boxes.iter().any(|b| {
        matches!(
            b,
            LayoutBox::Rect {
                color: 0xE6FFF8DC,
                ..
            }
        )
    });
    assert!(has_balloon, "insert revision should emit a margin balloon rect");
}
