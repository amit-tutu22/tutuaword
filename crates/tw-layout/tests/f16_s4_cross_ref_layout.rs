//! F16.S4 — cross-reference field layout text.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{
    paragraph_layout_text, Block, Document, FieldData, FieldType, Paragraph, Run, RunContent,
};

#[test]
fn u_f16_s4_cross_ref_layout_text() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("");
    para.runs = vec![
        Run {
            id: tw_model::NodeId::new(),
            format: Default::default(),
            content: RunContent::Bookmark(tw_model::BookmarkAnchor {
                name: "SectionRef".into(),
                bookmark_id: Some(1),
            }),
            revision: None,
        },
        Run {
            id: tw_model::NodeId::new(),
            format: Default::default(),
            content: RunContent::Text("Introduction".into()),
            revision: None,
        },
        Run {
            id: tw_model::NodeId::new(),
            format: Default::default(),
            content: RunContent::Field(FieldData {
                field_type: FieldType::CrossRef,
                instruction: Some(" REF SectionRef \\h ".into()),
                display_text: Some("Introduction".into()),
                form: None,
            merge_name: None,
            }),
            revision: None,
        },
    ];
    doc.sections[0].blocks = vec![Block::Paragraph(para.clone())];

    let layout_text = paragraph_layout_text(&para, None);
    assert!(layout_text.contains("Introduction"));

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
