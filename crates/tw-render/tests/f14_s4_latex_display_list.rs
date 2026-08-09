//! U-F14-S4 — LaTeX import produces layout/display-list glyphs.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Paragraph, Run, RunContent, latex_to_inline_omath};
use tw_render::DisplayListBuilder;

#[test]
fn u_f14_s4_latex_frac_display_list() {
    let xml = latex_to_inline_omath(r"\frac{\pi}{2}").expect("latex convert");
    let mut doc = tw_model::Document::new();
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
    let codepoints: Vec<char> = page
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => {
                Some(line.glyphs.iter().map(|g| g.codepoint).collect::<Vec<_>>())
            }
            _ => None,
        })
        .flatten()
        .collect();

    assert!(codepoints.contains(&'π'), "expected π from \\pi, got {codepoints:?}");

    let list = DisplayListBuilder::from_page_without_atlas(page, 1);
    assert!(
        list.atlas_batch.transforms.len() >= 2,
        "fraction should emit multiple glyphs"
    );
}
