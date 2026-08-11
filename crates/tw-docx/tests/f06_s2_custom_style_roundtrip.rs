//! F06.S2 — custom paragraph styles DOCX round-trip (U-F06-S2-custom-style-roundtrip).

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, CharFormat, Document, ParaFormat};

#[test]
fn u_f06_s2_custom_style_roundtrip() {
    let mut doc = Document::with_paragraph("Custom body text");
    let normal_id = doc.styles.find_style_by_name("Normal").unwrap().id;
    let custom_id = doc
        .styles
        .create_paragraph_style(
            "Company Body".into(),
            Some(normal_id),
            CharFormat {
                font_size: Some(11.0),
                ..Default::default()
            },
            ParaFormat {
                space_after: Some(4.0),
                ..Default::default()
            },
        )
        .expect("create custom style");

    let ooxml_id = doc
        .styles
        .ooxml_id_for(custom_id)
        .expect("custom ooxml id");
    assert_eq!(ooxml_id, "CompanyBody");

    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.style_id = Some(custom_id);
    }

    let bytes = export(&doc, &DocxPackage::minimal()).expect("export");
    let imported = import(&bytes).expect("import");

    let style = imported
        .document
        .styles
        .find_style_by_name("Company Body")
        .expect("custom style by name");
    assert_eq!(
        imported
            .document
            .styles
            .ooxml_id_for(style.id)
            .as_deref(),
        Some(ooxml_id.as_str())
    );

    let resolved = imported.document.styles.resolve_char_format(
        Some(style.id),
        &CharFormat::default(),
    );
    assert_eq!(resolved.font_size, Some(11.0));

    let styles_xml = String::from_utf8(
        imported
            .package
            .parts
            .get("word/styles.xml")
            .expect("styles part")
            .clone(),
    )
    .expect("utf8");
    assert!(
        styles_xml.contains(&format!(r#"w:styleId="{ooxml_id}""#)),
        "styles.xml should contain custom style id: {styles_xml}"
    );
}

#[test]
fn u_f06_s2_renamed_custom_style_roundtrips() {
    let mut doc = Document::new();
    let normal_id = doc.styles.find_style_by_name("Normal").unwrap().id;
    let custom_id = doc
        .styles
        .create_paragraph_style(
            "Draft".into(),
            Some(normal_id),
            CharFormat::default(),
            ParaFormat::default(),
        )
        .expect("create");
    doc.styles
        .rename_paragraph_style(custom_id, "Draft v2".into())
        .expect("rename");

    let bytes = export(&doc, &DocxPackage::minimal()).expect("export");
    let imported = import(&bytes).expect("import");
    assert!(
        imported
            .document
            .styles
            .find_style_by_name("Draft v2")
            .is_some()
    );
}
