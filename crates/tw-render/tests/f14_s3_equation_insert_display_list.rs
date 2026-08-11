//! U-F14-S3 — inserted equation produces shaped glyphs in display list.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Paragraph, Run, RunContent, build_inline_omath, omml_run};
use tw_render::DisplayListBuilder;

#[test]
fn u_f14_s3_insert_equation_display_list_has_glyphs() {
    let mut doc = tw_model::Document::new();
    let xml = build_inline_omath(&omml_run("π"));
    doc.sections[0].blocks = vec![Block::Paragraph({
        let mut para = Paragraph::new();
        para.runs = vec![Run {
            id: tw_model::NodeId::new(),
            format: tw_model::CharFormat::default(),
            content: RunContent::OfficeMath { xml },
            revision: None,
        }];
        para
    })];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];
    let has_pi = page.boxes.iter().any(|b| match b {
        LayoutBox::TextLine(line) => line.glyphs.iter().any(|g| g.codepoint == 'π'),
        _ => false,
    });
    assert!(has_pi, "expected π glyph after equation insert");

    let list = DisplayListBuilder::from_page_without_atlas(page, 1);
    assert!(
        list.atlas_batch.transforms.len() >= 1,
        "expected shaped equation glyphs in display list"
    );
}
