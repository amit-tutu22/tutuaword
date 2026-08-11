use tw_model::{
    document_outline, numbering_contributes_to_outline, outline_level_from_style_name,
    Block, Document, NumberingRef, Paragraph,
};

#[test]
fn heading_style_maps_to_outline_level() {
    assert_eq!(outline_level_from_style_name("Heading 1"), Some(0));
    assert_eq!(outline_level_from_style_name("Heading 2"), Some(1));
    assert_eq!(outline_level_from_style_name("Heading 3"), Some(2));
    assert_eq!(outline_level_from_style_name("Heading 9"), Some(8));
    assert_eq!(outline_level_from_style_name("Normal"), None);
}

#[test]
fn numbered_list_links_outline_level() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("Section item");
    para.format.numbering = Some(NumberingRef {
        numbering_id: 2,
        level: 1,
    });
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    assert!(numbering_contributes_to_outline(&doc, 2, 1));
    let entries = document_outline(&doc);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].level, 1);
    assert_eq!(entries[0].text, "Section item");
}

#[test]
fn bullet_list_does_not_contribute_to_outline() {
    let doc = Document::new();
    assert!(!numbering_contributes_to_outline(&doc, 1, 0));
}

#[test]
fn heading1_style_appears_in_outline() {
    let doc = Document::new();
    let heading_id = doc.styles.find_style_by_name("Heading 1").unwrap().id;
    let mut para = Paragraph::with_text("Introduction");
    para.style_id = Some(heading_id);
    let mut doc = doc;
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let entries = document_outline(&doc);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].level, 0);
    assert_eq!(entries[0].text, "Introduction");
}

#[test]
fn empty_paragraphs_are_omitted_from_outline() {
    let mut doc = Document::new();
    let mut empty = Paragraph::with_text("");
    empty.format.outline_level = Some(0);
    doc.sections[0].blocks = vec![Block::Paragraph(empty)];

    assert!(document_outline(&doc).is_empty());
}

#[test]
fn direct_outline_level_wins_over_numbering() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("Custom");
    para.format.numbering = Some(NumberingRef {
        numbering_id: 2,
        level: 2,
    });
    para.format.outline_level = Some(0);
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let entries = document_outline(&doc);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].level, 0);
}

#[test]
fn outline_level_nine_is_hidden() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("Body");
    para.format.outline_level = Some(9);
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    assert!(document_outline(&doc).is_empty());
}
