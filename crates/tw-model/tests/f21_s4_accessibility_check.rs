//! F21.S4 — accessibility checker rules.

use tw_model::{
    check_accessibility, AccessibilityRule, AccessibilitySeverity, Block, Color, Document,
    ImageBlock, ImageData, ImageTransform, Paragraph, Table, TextWrap,
};

fn heading_para(doc: &Document, style_name: &str, text: &str) -> Paragraph {
    let style_id = doc.styles.find_style_by_name(style_name).unwrap().id;
    let mut para = Paragraph::with_text(text);
    para.style_id = Some(style_id);
    para
}

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn image_block(alt: Option<&str>) -> ImageBlock {
    ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 72.0,
        display_height: 48.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: alt.map(|s| s.to_string()),
    }
}

#[test]
fn u_f21_s4_missing_alt() {
    let mut doc = Document::new();
    let with_alt = image_block(Some("Logo"));
    let missing = image_block(None);
    doc.sections[0].blocks = vec![
        Block::ImageBlock(with_alt),
        Block::ImageBlock(missing.clone()),
    ];

    let issues = check_accessibility(&doc);
    let missing_alts: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == AccessibilityRule::MissingAlt)
        .collect();
    assert_eq!(missing_alts.len(), 1);
    assert_eq!(missing_alts[0].node_id, missing.id);
    assert_eq!(missing_alts[0].severity, AccessibilitySeverity::Error);
}

#[test]
fn u_f21_s4_missing_alt_in_table() {
    let mut doc = Document::new();
    let image = image_block(None);
    let image_id = image.id;
    let mut table = Table::new(1, 1);
    table.rows[0].cells[0].blocks = vec![Block::ImageBlock(image)];
    doc.sections[0].blocks = vec![Block::Table(table)];

    let issues = check_accessibility(&doc);
    assert!(issues.iter().any(|i| {
        i.rule == AccessibilityRule::MissingAlt && i.node_id == image_id
    }));
}

#[test]
fn u_f21_s4_empty_heading() {
    let mut doc = Document::new();
    let empty = heading_para(&doc, "Heading 1", "   ");
    let filled = heading_para(&doc, "Heading 1", "Chapter");
    doc.sections[0].blocks = vec![
        Block::Paragraph(empty.clone()),
        Block::Paragraph(filled),
    ];

    let issues = check_accessibility(&doc);
    let empties: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == AccessibilityRule::EmptyHeading)
        .collect();
    assert_eq!(empties.len(), 1);
    assert_eq!(empties[0].node_id, empty.id);
    assert_eq!(empties[0].severity, AccessibilitySeverity::Error);
}

#[test]
fn u_f21_s4_low_contrast() {
    let mut doc = Document::new();
    let mut light = Paragraph::with_text("Faint text");
    light.runs[0].format.color = Some(Color {
        r: 200,
        g: 200,
        b: 200,
        a: 255,
    });
    let ok = Paragraph::with_text("Readable text");
    doc.sections[0].blocks = vec![Block::Paragraph(light.clone()), Block::Paragraph(ok)];

    let issues = check_accessibility(&doc);
    let contrast: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == AccessibilityRule::LowContrast)
        .collect();
    assert_eq!(contrast.len(), 1);
    assert_eq!(contrast[0].node_id, light.id);
    assert_eq!(contrast[0].run_id, Some(light.runs[0].id));
    assert_eq!(contrast[0].severity, AccessibilitySeverity::Warning);

    // Black on white should not warn.
    assert!(!issues.iter().any(|i| {
        i.rule == AccessibilityRule::LowContrast && i.node_id != light.id
    }));
}
