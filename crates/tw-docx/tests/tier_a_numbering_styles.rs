//! Tier A gates for styles + numbering edit round-trip (risk-mitigation.md).

use tw_docx::{export, import, DocxPackage};
use tw_model::{
    Block, CharFormat, Document, ListLevel, ListMarkerFormat, ListSuffix, NumberingDefinition,
    NumberingRef, Paragraph,
};

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
