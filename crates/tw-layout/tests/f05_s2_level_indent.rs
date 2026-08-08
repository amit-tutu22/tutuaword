#[path = "f05_layout_helpers/mod.rs"]
mod f05_layout_helpers;

use f05_layout_helpers::{assert_hanging, assert_text_x, list_lines};
use tw_model::{
    Block, Document, ListLevel, ListMarkerFormat, ListSuffix, NumberingDefinition, NumberingRef,
    Paragraph,
};

fn multilevel_list_doc(level: u32, text: &str) -> Document {
    let mut doc = Document::new();
    doc.settings.numbering.definitions.insert(
        1,
        NumberingDefinition {
            id: 1,
            name: "Bullet".into(),
            levels: vec![
                ListLevel {
                    level: 0,
                    format: ListMarkerFormat::Bullet,
                    indent: 36.0,
                    hanging: 18.0,
                    suffix: ListSuffix::Tab,
                    marker_text: None,
                    start: 1,
                    char_format: Default::default(),
                },
                ListLevel {
                    level: 1,
                    format: ListMarkerFormat::Bullet,
                    indent: 72.0,
                    hanging: 18.0,
                    suffix: ListSuffix::Tab,
                    marker_text: None,
                    start: 1,
                    char_format: Default::default(),
                },
            ],
        },
    );

    let mut para = Paragraph::with_text(text);
    para.format.numbering = Some(NumberingRef {
        numbering_id: 1,
        level,
    });
    doc.sections[0].blocks = vec![Block::Paragraph(para)];
    doc
}

/// U-F05-S2-level-indent: level 1 hanging indent layout.
#[test]
fn level_one_hanging_indent_layout() {
    let doc = multilevel_list_doc(1, "Nested item");
    let margin = doc.sections[0].format.margin_left;
    let line = &list_lines(&doc)[0];

    assert_text_x(line, margin, 72.0);
    assert_hanging(line, f05_layout_helpers::HANGING_PT);
}

#[test]
fn level_zero_and_one_indents_differ_by_36pt() {
    let margin = Document::new().sections[0].format.margin_left;
    let level0_line = &list_lines(&multilevel_list_doc(0, "Top"))[0];
    let level1_line = &list_lines(&multilevel_list_doc(1, "Nested"))[0];

    assert_text_x(level0_line, margin, 36.0);
    assert_text_x(level1_line, margin, 72.0);
    assert!(
        (level1_line.x - level0_line.x - 36.0).abs() < f05_layout_helpers::POS_TOL,
        "level 1 should be 36 pt right of level 0"
    );
}
