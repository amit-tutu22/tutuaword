//! F17.S3 — comment margin markers and inline anchor layout.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{
    CharFormat, Color, CommentRef, CommentThread, Document, Run, RunContent,
};

#[test]
fn u_f17_s3_comment_margin_marker_layout() {
    let mut doc = Document::with_paragraph("Hello");
    doc.comments.push(CommentThread::new(
        0,
        doc.paragraph_at(0, 0).unwrap().runs[0].id,
        0,
        "Reviewer",
        "Needs clarification.",
    ));

    let para = doc.paragraph_at_mut(0, 0).unwrap();
    para.runs.push(Run {
        id: tw_model::NodeId::new(),
        format: CharFormat {
            highlight: Some(Color {
                r: 255,
                g: 255,
                b: 0,
                a: 64,
            }),
            ..Default::default()
        },
        content: RunContent::CommentRef(CommentRef {
            comment_id: 0,
            display_number: Some(1),
        }),
        revision: None,
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);

    assert!(!layout.pages.is_empty());
    let page = &layout.pages[0];

    let has_inline_marker = page.boxes.iter().any(|b| {
        let LayoutBox::TextLine(line) = b else {
            return false;
        };
        line.glyphs.iter().any(|g| g.codepoint == 'C')
    });
    assert!(has_inline_marker, "page should contain inline comment marker");

    let has_margin_marker = page.boxes.iter().any(|b| {
        matches!(
            b,
            LayoutBox::Rect {
                color: 0xFFFFA500,
                ..
            }
        )
    });
    assert!(has_margin_marker, "page should contain right-margin comment marker");
}
