//! U-F14-S2 — equation preview lines participate in caret hit-testing (not decorative).

use tw_layout::LayoutEngine;
use tw_model::{Block, Paragraph, Run, RunContent};

const INLINE_OMML: &str = r#"<m:oMath><m:r><m:t>x</m:t></m:r></m:oMath>"#;

#[test]
fn u_f14_s2_equation_line_not_decorative() {
    let mut para = Paragraph::new();
    para.runs = vec![
        Run::new_text("before "),
        Run {
            id: tw_model::NodeId::new(),
            format: tw_model::CharFormat::default(),
            content: RunContent::OfficeMath {
                xml: INLINE_OMML.to_string(),
            },
            revision: None,
        },
        Run::new_text(" after"),
    ];
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let mut engine = LayoutEngine::new();
    let _ = engine.layout_document(&doc);
    let map = engine.line_map(0).expect("line map");

    assert!(
        map.lines.iter().any(|line| !line.decorative && !line.glyphs.is_empty()),
        "equation paragraph must produce non-decorative glyph lines"
    );
    assert!(
        map.lines.iter().any(|line| line.run_map.len() >= 2),
        "mixed text + math should map multiple runs for caret placement"
    );
}

#[test]
fn u_f14_s2_equation_preview_has_math_frame_decoration() {
    let mut para = Paragraph::new();
    para.runs = vec![Run {
        id: tw_model::NodeId::new(),
        format: tw_model::CharFormat::default(),
        content: RunContent::OfficeMath {
            xml: INLINE_OMML.to_string(),
        },
        revision: None,
    }];
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let has_frame = layout.pages[0].boxes.iter().any(|b| {
        matches!(
            b,
            tw_layout::LayoutBox::TextLine(line)
                if line.decorations.iter().any(|d| {
                    matches!(d.kind, tw_layout::DecorationKind::MathFrame)
                })
        )
    });
    assert!(has_frame, "equation preview should include MathFrame decoration");
}
