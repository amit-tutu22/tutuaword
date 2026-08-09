//! F16.S1 — footnote reference layout and bottom-of-page band.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{
    Block, CharFormat, Document, Footnote, FootnoteRef, Paragraph, Run, RunContent,
};

#[test]
fn u_f16_s1_footnote_ref_layout_superscript_and_band() {
    let mut doc = Document::with_paragraph("Hello");
    doc.footnotes.push(Footnote {
        id: 1,
        blocks: vec![Block::Paragraph(Paragraph::with_text("Footnote body text."))],
    });

    let para = doc.paragraph_at_mut(0, 0).unwrap();
    para.runs.push(Run {
        id: tw_model::NodeId::new(),
        format: CharFormat {
            superscript: Some(true),
            ..Default::default()
        },
        content: RunContent::FootnoteRef(FootnoteRef {
            note_id: 1,
            display_number: Some(1),
        }),
        revision: None,
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);

    assert!(!layout.pages.is_empty());
    let page = &layout.pages[0];

    let has_ref_marker = page.boxes.iter().any(|b| {
        let LayoutBox::TextLine(line) = b else {
            return false;
        };
        line.glyphs.iter().any(|g| g.codepoint == '1')
    });
    assert!(has_ref_marker, "page should contain footnote reference marker");

    let has_separator = page
        .boxes
        .iter()
        .any(|b| matches!(b, LayoutBox::Rect { height, .. } if *height <= 2.0));
    assert!(has_separator, "page should contain footnote separator rule");

    let has_footnote_body = page.boxes.iter().any(|b| {
        let LayoutBox::TextLine(line) = b else {
            return false;
        };
        line.glyphs.iter().any(|g| g.codepoint == 'F' || g.codepoint == 'b')
    });
    assert!(has_footnote_body, "page should layout footnote body in bottom band");
}
