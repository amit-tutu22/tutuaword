//! F07.S1 — page setup layout (U-F07-S1-landscape-swaps-dimensions, I-F07-S1-margin-preset).

#[path = "f04_layout_helpers/mod.rs"]
mod f04_layout_helpers;

use f04_layout_helpers::{count_para_lines, layout_doc};
use tw_layout::LayoutEngine;
use tw_model::Document;

#[test]
fn u_f07_s1_landscape_swaps_dimensions() {
    let mut doc = Document::with_paragraph("Landscape page");
    doc.sections[0].format.page_width = 842.0;
    doc.sections[0].format.page_height = 595.0;

    let layout = LayoutEngine::new().layout_document(&doc);
    assert_eq!(layout.pages[0].width, 842.0);
    assert_eq!(layout.pages[0].height, 595.0);
}

#[test]
fn i_f07_s1_margin_preset_reflows_text() {
    let words: String = std::iter::repeat("word ").take(400).collect();
    let doc = Document::from_plain_text(words.trim());
    let para_id = doc.sections[0].blocks[0].paragraph().unwrap().id;

    let normal_lines = count_para_lines(&layout_doc(&doc), para_id);

    let mut narrow = doc.clone();
    narrow.sections[0].format.margin_left = 36.0;
    narrow.sections[0].format.margin_right = 36.0;
    let narrow_lines = count_para_lines(&layout_doc(&narrow), para_id);

    assert!(
        narrow_lines < normal_lines,
        "narrow margins should fit fewer lines (normal={normal_lines}, narrow={narrow_lines})"
    );
}
