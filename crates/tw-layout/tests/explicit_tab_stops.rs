use tw_layout::{layout_paragraph, ParagraphFrame};
use tw_model::{Paragraph, Run, RunContent, TabAlignment, TabStop};
use tw_shape::{GlyphAtlas, TextShaper};

#[test]
fn explicit_tab_stop_advances_beyond_default_grid() {
    let mut shaper = TextShaper::new();
    let mut atlas = GlyphAtlas::new(512, 512);
    let mut para = Paragraph::with_text("A\tB");
    para.runs = vec![
        Run {
            id: tw_model::NodeId::new(),
            format: Default::default(),
            content: RunContent::Text("A\tB".into()),
            revision: None,
        },
    ];
    para.format.tab_stops = vec![TabStop {
        position: 200.0,
        alignment: TabAlignment::Left,
    }];

    let (lines, _) = layout_paragraph(
        &mut shaper,
        &mut atlas,
        &para,
        ParagraphFrame::new(0.0, 0.0, 400.0)
            .with_tab_interval(36.0)
            .with_tab_stops(para.format.tab_stops.clone()),
        0xFF000000,
    );

    assert_eq!(lines.len(), 1);
    let b_glyph = lines[0]
        .glyphs
        .iter()
        .find(|g| g.codepoint == 'B')
        .expect("B glyph");
    assert!(b_glyph.x >= 190.0, "tab should land near 200pt, got {}", b_glyph.x);
}
