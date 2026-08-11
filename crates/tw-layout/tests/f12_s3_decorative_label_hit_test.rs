//! U-F12-S3 — SmartArt captions must not steal caret hit-testing.

use tw_layout::LayoutEngine;
use tw_model::{Block, ShapeBlock};

#[test]
fn u_f12_s3_diagram_label_excluded_from_line_map() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(tw_model::Paragraph::with_text("Body")),
        Block::ShapeBlock(ShapeBlock::diagram(432.0, 216.0)),
    ];

    let mut engine = LayoutEngine::new();
    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");

    assert!(
        map.lines.iter().all(|line| !line.decorative),
        "line map must not include decorative SmartArt captions"
    );
    assert!(
        map.lines.iter().any(|line| {
            line.glyphs.iter().any(|g| g.codepoint == 'B')
        }),
        "body text should remain hittable"
    );
}
