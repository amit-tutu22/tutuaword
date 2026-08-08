//! F08.S1 — header blocks in margin band layout (U-F08-S1-header-blocks-layout).

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{
    Block, Document, HeaderFooter, HeaderFooterType, Paragraph, SectionFormat,
};

#[test]
fn u_f08_s1_header_blocks_layout() {
    let mut doc = Document::new();
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        HeaderFooter {
            blocks: vec![Block::Paragraph(Paragraph::with_text("Quarterly Report"))],
            plain_text: None,
        },
    );
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Body paragraph text.",
    ))];

    let format = SectionFormat::default();
    let body_y = format.margin_top;

    let layout = LayoutEngine::new().layout_document(&doc);
    let page = layout.pages.first().expect("one page");

    let line_ys: Vec<f32> = page
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.y),
            _ => None,
        })
        .collect();

    let header_line = line_ys
        .iter()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("header line");
    let body_line = line_ys
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("body line");

    assert!(
        header_line < body_y,
        "header line ({header_line}) should sit in the top margin band above content top {body_y}"
    );
    assert!(
        (body_line - body_y).abs() < 15.0,
        "body line should start near content top {body_y}, got {body_line}"
    );
    assert!(
        body_line - header_line > 20.0,
        "header and body bands should be separated, got {line_ys:?}"
    );
}
