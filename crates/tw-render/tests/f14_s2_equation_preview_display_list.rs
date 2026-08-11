//! U-F14-S2 — read-only equation preview paints glyphs + frame rects.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Paragraph, Run, RunContent, MATH_FRAME_ARGB};
use tw_render::DisplayListBuilder;

const INLINE_OMML: &str = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:r><m:t>E=mc</m:t></m:r><m:r><m:t>2</m:t></m:r>
</m:oMath>"#;

const DISPLAY_OMML: &str = r#"<m:oMathPara xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:oMath><m:r><m:t>α</m:t></m:r><m:r><m:t>+β</m:t></m:r></m:oMath>
</m:oMathPara>"#;

fn paragraph_with_omml(xml: &str) -> Paragraph {
    let mut para = Paragraph::new();
    para.runs = vec![Run {
        id: tw_model::NodeId::new(),
        format: tw_model::CharFormat::default(),
        content: RunContent::OfficeMath {
            xml: xml.to_string(),
        },
        revision: None,
    }];
    para
}

fn glyph_codepoints(page: &tw_layout::PageLayout) -> Vec<char> {
    page.boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.glyphs.iter().map(|g| g.codepoint).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn u_f14_s2_inline_equation_emits_preview_glyphs() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(paragraph_with_omml(INLINE_OMML))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let codepoints = glyph_codepoints(&layout.pages[0]);

    assert!(
        codepoints.contains(&'E') && codepoints.contains(&'=') && codepoints.contains(&'2'),
        "expected OMML m:t glyphs, got {codepoints:?}"
    );
    assert!(
        !codepoints.contains(&'[') && !codepoints.contains(&']'),
        "must not paint [math] placeholder: {codepoints:?}"
    );

    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);
    assert!(
        list.rect_batch.colors.iter().any(|&c| c == MATH_FRAME_ARGB),
        "expected math frame rect color"
    );
    assert!(
        list.atlas_batch.transforms.len() >= 2,
        "expected shaped equation glyphs in display list"
    );
}

#[test]
fn u_f14_s2_display_equation_emits_frame_and_glyphs() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(paragraph_with_omml(DISPLAY_OMML))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let codepoints = glyph_codepoints(&layout.pages[0]);

    assert!(
        codepoints.contains(&'α') || codepoints.contains(&'β'),
        "display math should shape extracted Unicode: {codepoints:?}"
    );

    let list = DisplayListBuilder::from_page_without_atlas(&layout.pages[0], 1);
    assert!(
        !list.rect_batch.rects.is_empty(),
        "display equation should emit preview frame rects"
    );
    assert!(
        list.atlas_batch.transforms.len() >= 2,
        "display equation should emit multiple glyphs"
    );
}

#[test]
fn u_f14_s2_mixed_text_and_equation_shapes_both() {
    let mut para = Paragraph::new();
    para.runs = vec![
        Run::new_text("Energy "),
        Run {
            id: tw_model::NodeId::new(),
            format: tw_model::CharFormat::default(),
            content: RunContent::OfficeMath {
                xml: INLINE_OMML.to_string(),
            },
            revision: None,
        },
        Run::new_text(" law"),
    ];
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let codepoints = glyph_codepoints(&layout.pages[0]);

    assert!(codepoints.contains(&'E'), "equation glyphs present");
    assert!(codepoints.contains(&'n'), "surrounding text glyphs present");
}
