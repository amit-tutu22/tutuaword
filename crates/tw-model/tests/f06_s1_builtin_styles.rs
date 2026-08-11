//! F06.S1 — built-in paragraph styles (U-F06-S1-resolve-based-on).

use tw_model::{CharFormat, ParaFormat, StyleSheet};

#[test]
fn u_f06_s1_new_document_has_builtin_paragraph_styles() {
    let styles = StyleSheet::with_defaults();
    for name in [
        "Normal",
        "Heading 1",
        "Heading 2",
        "Heading 3",
        "Heading 4",
        "Heading 5",
        "Heading 6",
        "Heading 7",
        "Heading 8",
        "Heading 9",
        "Quote",
        "Caption",
    ] {
        assert!(
            styles.find_style_by_name(name).is_some(),
            "missing built-in style {name}"
        );
    }
}

#[test]
fn u_f06_s1_resolve_based_on_h3_inherits_h2_chain() {
    let styles = StyleSheet::with_defaults();
    let h2 = styles.find_style_by_name("Heading 2").unwrap();
    let h3 = styles.find_style_by_name("Heading 3").unwrap();
    let h1 = styles.find_style_by_name("Heading 1").unwrap();
    let normal = styles.find_style_by_name("Normal").unwrap();

    assert_eq!(h1.based_on, Some(normal.id));
    assert_eq!(h2.based_on, Some(h1.id));
    assert_eq!(h3.based_on, Some(h2.id));

    let resolved_char = styles.resolve_char_format(Some(h3.id), &CharFormat::default());
    assert_eq!(resolved_char.bold, Some(true), "H3 inherits bold from Heading 1 chain");
    assert_eq!(resolved_char.font_size, Some(13.0));

    let resolved_para = styles.resolve_para_format(Some(h3.id), &ParaFormat::default());
    assert_eq!(resolved_para.outline_level, Some(2));
    assert_eq!(resolved_para.space_before, Some(6.0));
}

#[test]
fn u_f06_s1_quote_and_caption_resolve() {
    let styles = StyleSheet::with_defaults();
    let quote = styles.find_style_by_name("Quote").unwrap();
    let caption = styles.find_style_by_name("Caption").unwrap();

    let quote_char = styles.resolve_char_format(Some(quote.id), &CharFormat::default());
    assert_eq!(quote_char.italic, Some(true));

    let quote_para = styles.resolve_para_format(Some(quote.id), &ParaFormat::default());
    assert_eq!(quote_para.indent_left, Some(36.0));
    assert_eq!(quote_para.indent_right, Some(36.0));

    let caption_char = styles.resolve_char_format(Some(caption.id), &CharFormat::default());
    assert_eq!(caption_char.font_size, Some(9.0));
    assert_eq!(caption_char.italic, Some(true));
}
