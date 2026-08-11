//! F04.S2 — `w:lineRule="exact"` maps to absolute line height in layout.

mod f04_layout_helpers;

use f04_layout_helpers::{assert_near, second_text_line_y};
use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, LineSpacing, ParaFormat, Paragraph};

fn first_line_height(doc: &Document) -> f32 {
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(doc);
    layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.line_height),
            _ => None,
        })
        .expect("text line")
}

fn paragraph_with_spacing(text: &str, spacing: LineSpacing) -> Document {
    let mut doc = Document::new();
    let id = doc.sections[0].blocks[0].paragraph().unwrap().id;
    let mut para = Paragraph::with_text(text);
    para.id = id;
    para.format = ParaFormat {
        line_spacing: Some(spacing),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(para);
    doc
}

/// U-F04-S2-line-rule-exact: 480 twips (`w:lineRule="exact"`) → Exactly(24 pt).
#[test]
fn u_f04_s2_line_rule_exact() {
    const EXACT_POINTS: f32 = 480.0 / 20.0;
    assert_near(EXACT_POINTS, 24.0, f32::EPSILON, "twips conversion");

    let exact_h = first_line_height(&paragraph_with_spacing(
        "Exact",
        LineSpacing::Exactly(EXACT_POINTS),
    ));
    let auto_h = first_line_height(&paragraph_with_spacing("Auto", LineSpacing::Single));

    assert_near(exact_h, EXACT_POINTS, 0.01, "exact lineRule height");
    assert!(
        (auto_h - EXACT_POINTS).abs() > 1.0,
        "single spacing ({auto_h}) must differ from exact ({EXACT_POINTS})"
    );
}

#[test]
fn exact_line_height_ignores_font_size() {
    let mut small = paragraph_with_spacing("Aa", LineSpacing::Exactly(36.0));
    small.paragraph_at_mut(0, 0).unwrap().runs[0].format.font_size = Some(10.0);

    let mut large = paragraph_with_spacing("Aa", LineSpacing::Exactly(36.0));
    large.paragraph_at_mut(0, 0).unwrap().runs[0].format.font_size = Some(28.0);

    assert_near(first_line_height(&small), 36.0, 0.01, "small font exact height");
    assert_near(first_line_height(&large), 36.0, 0.01, "large font exact height");
}

#[test]
fn space_before_and_after_add_exact_vertical_gap() {
    let exact = LineSpacing::Exactly(f04_layout_helpers::EXACT_LINE_PT);

    let mut spaced = Document::new();
    spaced.sections[0].blocks.clear();
    spaced.sections[0].blocks = vec![
        Block::Paragraph({
            let mut p = Paragraph::with_text("First");
            p.format = ParaFormat {
                line_spacing: Some(exact.clone()),
                space_after: Some(24.0),
                ..Default::default()
            };
            p
        }),
        Block::Paragraph({
            let mut p = Paragraph::with_text("Second");
            p.format = ParaFormat {
                line_spacing: Some(exact),
                space_before: Some(12.0),
                ..Default::default()
            };
            p
        }),
    ];

    let mut plain = Document::new();
    plain.sections[0].blocks.clear();
    plain.sections[0].blocks = vec![
        Block::Paragraph({
            let mut p = Paragraph::with_text("First");
            p.format.line_spacing = Some(LineSpacing::Exactly(f04_layout_helpers::EXACT_LINE_PT));
            p
        }),
        Block::Paragraph({
            let mut p = Paragraph::with_text("Second");
            p.format.line_spacing = Some(LineSpacing::Exactly(f04_layout_helpers::EXACT_LINE_PT));
            p
        }),
    ];

    let delta = second_text_line_y(&spaced) - second_text_line_y(&plain);
    assert_near(delta, 36.0, 0.5, "space_after(24) + space_before(12)");
}
