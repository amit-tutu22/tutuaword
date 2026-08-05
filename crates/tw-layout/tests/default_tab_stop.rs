use tw_layout::{LayoutBox, LayoutEngine, ParagraphFrame, layout_paragraph};
use tw_model::{Document, Paragraph, Run, RunContent};
use tw_shape::{GlyphAtlas, TextShaper};

#[test]
fn custom_default_tab_stop_advances_further_than_default() {
    let mut shaper = TextShaper::new();
    let mut atlas = GlyphAtlas::default();
    let mut para = Paragraph::new();
    para.runs = vec![
        Run {
            id: para.runs[0].id,
            format: Default::default(),
            content: RunContent::Text("A".into()),
            revision: None,
        },
        Run {
            id: tw_model::NodeId::new(),
            format: Default::default(),
            content: RunContent::Text("\tB".into()),
            revision: None,
        },
    ];

    let narrow = layout_paragraph(
        &mut shaper,
        &mut atlas,
        &para,
        ParagraphFrame::new(0.0, 0.0, 400.0).with_tab_interval(36.0),
        0xFF000000,
    )
    .0;
    let wide = layout_paragraph(
        &mut shaper,
        &mut atlas,
        &para,
        ParagraphFrame::new(0.0, 0.0, 400.0).with_tab_interval(72.0),
        0xFF000000,
    )
    .0;

    let b_narrow = narrow[0]
        .glyphs
        .iter()
        .find(|g| g.codepoint == 'B')
        .map(|g| g.x)
        .unwrap();
    let b_wide = wide[0]
        .glyphs
        .iter()
        .find(|g| g.codepoint == 'B')
        .map(|g| g.x)
        .unwrap();
    assert!(b_wide > b_narrow, "wider tab stops should push B further right");
}

#[test]
fn layout_engine_reads_document_default_tab_stop() {
    let mut doc = Document::with_paragraph("X\tY");
    doc.settings.default_tab_stop = 96.0;
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let line = match &layout.pages[0].boxes[0] {
        LayoutBox::TextLine(l) => l,
        _ => panic!("expected text line"),
    };
    let y = line
        .glyphs
        .iter()
        .find(|g| g.codepoint == 'Y')
        .map(|g| g.x)
        .unwrap();
    assert!(y >= 90.0, "Y should honor the document tab stop setting");
}
