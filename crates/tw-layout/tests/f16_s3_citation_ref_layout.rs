//! F16.S3 — citation ref layout text.

use tw_layout::{LayoutEngine, LayoutBox};
use tw_model::{
    paragraph_layout_text, Block, CitationRef, Document, Paragraph, Run, RunContent,
};

#[test]
fn u_f16_s3_citation_ref_layout_text() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("See ");
    para.runs.push(Run {
        id: tw_model::NodeId::new(),
        format: Default::default(),
        content: RunContent::CitationRef(CitationRef {
            source_key: "Smith2020".into(),
            display_text: Some("(Smith, 2020)".into()),
        }),
        revision: None,
    });
    doc.sections[0].blocks = vec![Block::Paragraph(para.clone())];

    let layout_text = paragraph_layout_text(&para, None);
    assert!(layout_text.contains("See"));
    assert!(layout_text.contains("(Smith, 2020)"));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let line = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.clone()),
            _ => None,
        })
        .expect("text line");
    assert!(!line.glyphs.is_empty());
}
