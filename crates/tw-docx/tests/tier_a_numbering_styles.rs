//! U-F23-S1-tier-a-numbering — Tier A round-trip gates per category
//! (styles, numbering, tables, char formats). See risk-mitigation.md.

use tw_docx::{export, import, DocxPackage};
use tw_model::{
    Alignment, Block, CharFormat, Color, Document, ImageBlock, ImageData, ImageTransform,
    ListLevel, ListMarkerFormat, ListSuffix, NumberingDefinition, NumberingRef, Paragraph,
    TextWrap, UnderlineStyle,
};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn round_trip(doc: &Document) -> Document {
    import(&export(doc, &DocxPackage::minimal()).unwrap())
        .unwrap()
        .document
}

#[test]
fn custom_lvl_text_and_start_round_trip() {
    let mut doc = Document::with_paragraph("Item");
    doc.settings.numbering.definitions.insert(
        40,
        NumberingDefinition {
            id: 40,
            name: "Custom".into(),
            levels: vec![ListLevel {
                level: 0,
                format: ListMarkerFormat::Decimal,
                indent: 36.0,
                hanging: 18.0,
                suffix: ListSuffix::Tab,
                marker_text: Some("(%1)".into()),
                start: 3,
                char_format: CharFormat {
                    font_family: Some("Arial".into()),
                    bold: Some(true),
                    ..Default::default()
                },
            }],
        },
    );
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.format.numbering = Some(NumberingRef {
            numbering_id: 40,
            level: 0,
        });
    }

    let imported = import(&export(&doc, &DocxPackage::minimal()).unwrap()).unwrap();
    let lvl = imported
        .document
        .settings
        .numbering
        .get(40)
        .expect("custom numbering id")
        .levels
        .first()
        .expect("level 0");
    assert_eq!(lvl.marker_text.as_deref(), Some("(%1)"));
    assert_eq!(lvl.start, 3);
    assert_eq!(lvl.char_format.font_family.as_deref(), Some("Arial"));
    assert_eq!(lvl.char_format.bold, Some(true));

    let para = imported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    assert_eq!(
        para.format.numbering,
        Some(NumberingRef {
            numbering_id: 40,
            level: 0
        })
    );
}

#[test]
fn heading_style_emits_pstyle_and_reimports() {
    let mut doc = Document::new();
    let heading = doc
        .styles
        .paragraph_styles
        .values()
        .find(|s| s.name == "Heading 1")
        .map(|s| s.id)
        .expect("Heading 1");
    let mut para = Paragraph::with_text("Title");
    para.style_id = Some(heading);
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let bytes = export(&doc, &DocxPackage::minimal()).unwrap();
    let package = import(&bytes).unwrap();
    let document_xml = String::from_utf8(
        package.package.parts["word/document.xml"].clone(),
    )
    .unwrap();
    assert!(
        document_xml.contains("w:pStyle")
            && (document_xml.contains("Heading") || document_xml.contains("heading")),
        "exported paragraph should reference a heading style: {document_xml}"
    );
    assert!(
        package.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .style_id
            .is_some(),
        "re-import should attach a paragraph style"
    );
}

#[test]
fn u_f23_s1_tier_a_table_round_trip() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Table(tw_model::Table::new(3, 2))];
    let imported = round_trip(&doc);
    let table = imported.sections[0].blocks[0].table().unwrap();
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0].cells.len(), 2);
}

#[test]
fn u_f23_s1_tier_a_char_format_round_trip() {
    let mut doc = Document::new();
    let mut run = tw_model::Run::new_text("Styled");
    run.format = CharFormat {
        bold: Some(true),
        italic: Some(true),
        underline: Some(UnderlineStyle::Single),
        color: Some(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }),
        font_size: Some(16.0),
        ..Default::default()
    };
    let mut para = Paragraph::new();
    para.format.alignment = Some(Alignment::Center);
    para.runs = vec![run];
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let imported = round_trip(&doc);
    let para = imported.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.format.alignment, Some(Alignment::Center));
    let fmt = &para.runs[0].format;
    assert_eq!(fmt.bold, Some(true));
    assert_eq!(fmt.italic, Some(true));
    assert_eq!(fmt.underline, Some(UnderlineStyle::Single));
    assert_eq!(fmt.color.map(|c| (c.r, c.g, c.b)), Some((255, 0, 0)));
}

#[test]
fn u_f23_s1_tier_a_image_round_trip() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::ImageBlock(ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 96.0,
        display_height: 48.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: Some("logo".into()),
    })];
    let imported = round_trip(&doc);
    let image = imported.sections[0].blocks[0].image().unwrap();
    assert_eq!(image.data.bytes, PNG_1X1);
    assert_eq!(image.alt_text.as_deref(), Some("logo"));
}
