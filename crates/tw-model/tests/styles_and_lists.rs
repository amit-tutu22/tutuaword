use tw_model::{
    format_list_marker, CharFormat, Document, NumberingDefinition, ParaFormat, StyleSheet,
};

#[test]
fn new_document_has_normal_and_heading1_styles() {
    let doc = Document::new();
    assert!(doc.styles.find_style_by_name("Normal").is_some());
    assert!(doc.styles.find_style_by_name("Heading 1").is_some());
}

#[test]
fn resolve_char_format_applies_style_chain() {
    let styles = StyleSheet::with_defaults();
    let heading = styles.find_style_by_name("Heading 1").unwrap();
    let resolved = styles.resolve_char_format(Some(heading.id), &CharFormat::default());
    assert_eq!(resolved.bold, Some(true));
    assert_eq!(resolved.font_size, Some(16.0));
}

#[test]
fn resolve_char_format_direct_overrides_style() {
    let styles = StyleSheet::with_defaults();
    let heading = styles.find_style_by_name("Heading 1").unwrap();
    let direct = CharFormat {
        font_size: Some(20.0),
        ..Default::default()
    };
    let resolved = styles.resolve_char_format(Some(heading.id), &direct);
    assert_eq!(resolved.font_size, Some(20.0));
}

#[test]
fn resolve_para_format_includes_style_spacing() {
    let styles = StyleSheet::with_defaults();
    let heading = styles.find_style_by_name("Heading 1").unwrap();
    let resolved = styles.resolve_para_format(Some(heading.id), &ParaFormat::default());
    assert_eq!(resolved.space_before, Some(12.0));
    assert_eq!(resolved.space_after, Some(6.0));
}

#[test]
fn bullet_list_marker_format() {
    let bullet = NumberingDefinition::bullet();
    assert_eq!(format_list_marker(&bullet, 0, 0), "•");
}

#[test]
fn numbered_list_marker_increments() {
    let numbered = NumberingDefinition::numbered();
    assert_eq!(format_list_marker(&numbered, 0, 0), "1.");
    assert_eq!(format_list_marker(&numbered, 0, 1), "2.");
    assert_eq!(format_list_marker(&numbered, 1, 0), "a.");
}

#[test]
fn section_format_header_footer_serialization_round_trip() {
    let mut doc = Document::new();
    doc.sections[0].format.header_text = Some("Chapter 1".into());
    doc.sections[0].format.footer_text = Some("Page footer".into());

    let json = serde_json::to_string(&doc.sections[0].format).unwrap();
    let decoded: tw_model::SectionFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.header_text.as_deref(), Some("Chapter 1"));
    assert_eq!(decoded.footer_text.as_deref(), Some("Page footer"));
}

#[test]
fn from_plain_text_splits_paragraphs() {
    let doc = Document::from_plain_text("Line one\nLine two\n\nLine three");
    let texts: Vec<_> = doc
        .sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph().map(|p| p.full_text()))
        .collect();
    assert_eq!(texts, vec!["Line one", "Line two", "", "Line three"]);
}
