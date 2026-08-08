//! F07.S2 — per-section page geometry in layout (U-F07-S2-section-format-per-section).

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, Paragraph};

#[test]
fn u_f07_s2_section_format_per_section() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Portrait section with normal margins.",
    ))];
    doc.sections[0].format.margin_left = 72.0;

    let mut section_two = tw_model::Section::new();
    section_two.blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Landscape section with narrow margins.",
    ))];
    section_two.format.page_width = 842.0;
    section_two.format.page_height = 595.0;
    section_two.format.margin_left = 36.0;
    section_two.format.margin_right = 36.0;
    doc.sections.push(section_two);

    let layout = LayoutEngine::new().layout_document(&doc);
    assert!(layout.pages.len() >= 2, "section break should start a new page");

    let line_x: Vec<f32> = layout
        .pages
        .iter()
        .flat_map(|page| page.boxes.iter())
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.x),
            _ => None,
        })
        .collect();

    assert!(
        line_x.iter().any(|x| (*x - 72.0).abs() < 1.0),
        "first section text should use 72pt left margin, got {line_x:?}"
    );
    assert!(
        line_x.iter().any(|x| (*x - 36.0).abs() < 1.0),
        "second section text should use 36pt left margin, got {line_x:?}"
    );
}
