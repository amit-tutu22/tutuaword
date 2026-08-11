//! F06.S3 — theme font resolution (U-F06-S3-theme-font-resolve).

use tw_model::{
    resolve_theme_fonts, Block, Document, DocumentTheme, THEME_FONT_PREFIX,
};

#[test]
fn u_f06_s3_theme_font_resolve_minor_and_major() {
    let mut doc = Document::with_paragraph("Hi");
    doc.settings.theme = DocumentTheme::facet();
    doc.styles.defaults.char_format.font_family = Some(format!("{THEME_FONT_PREFIX}minorHAnsi"));
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].format.font_family = Some(format!("{THEME_FONT_PREFIX}majorHAnsi"));
    }

    resolve_theme_fonts(&mut doc);

    assert_eq!(
        doc.styles.defaults.char_format.font_family.as_deref(),
        Some("Calibri")
    );
    let Block::Paragraph(para) = &doc.sections[0].blocks[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(
        para.runs[0].format.font_family.as_deref(),
        Some("Century Gothic")
    );
}

#[test]
fn u_f06_s3_theme_font_resolve_updates_when_theme_changes() {
    let mut doc = Document::with_paragraph("Hi");
    doc.styles.defaults.char_format.font_family = Some(format!("{THEME_FONT_PREFIX}minorHAnsi"));

    doc.settings.theme = DocumentTheme::office();
    resolve_theme_fonts(&mut doc);
    assert_eq!(
        doc.styles.defaults.char_format.font_family.as_deref(),
        Some("Calibri")
    );

    doc.settings.theme = DocumentTheme::ion();
    doc.styles.defaults.char_format.font_family = Some(format!("{THEME_FONT_PREFIX}minorHAnsi"));
    resolve_theme_fonts(&mut doc);
    assert_eq!(
        doc.styles.defaults.char_format.font_family.as_deref(),
        Some("Arial")
    );
}

#[test]
fn u_f06_s3_explicit_font_is_not_overwritten() {
    let mut doc = Document::with_paragraph("Hi");
    doc.settings.theme = DocumentTheme::ion();
    doc.styles.defaults.char_format.font_family = Some("Times New Roman".into());

    resolve_theme_fonts(&mut doc);

    assert_eq!(
        doc.styles.defaults.char_format.font_family.as_deref(),
        Some("Times New Roman")
    );
}

#[test]
fn u_f06_s3_builtin_theme_gallery_names() {
    assert!(DocumentTheme::by_name("Office").is_some());
    assert!(DocumentTheme::by_name("Facet").is_some());
    assert!(DocumentTheme::by_name("Ion").is_some());
    assert_eq!(DocumentTheme::gallery_themes(), &["Office", "Facet", "Ion"]);
}
