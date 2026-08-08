//! F06.S3 — theme color slot resolution (U-F06-S3-theme-color-resolve).

use tw_model::{
    resolve_theme_color_ref, resolve_theme_colors, ThemeColorRef, ThemeColorSlot, Document,
    DocumentTheme,
};

#[test]
fn u_f06_s3_resolve_theme_color_accent1_base() {
    let theme = DocumentTheme::office();
    let reference = ThemeColorRef::new(ThemeColorSlot::Accent1, 3);
    let color = resolve_theme_color_ref(&theme, reference);
    assert_eq!(color.r, 68);
    assert_eq!(color.g, 114);
    assert_eq!(color.b, 196);
}

#[test]
fn u_f06_s3_resolve_theme_color_tint_and_shade() {
    let theme = DocumentTheme::office();
    let light = resolve_theme_color_ref(
        &theme,
        ThemeColorRef::new(ThemeColorSlot::Accent1, 0),
    );
    let dark = resolve_theme_color_ref(
        &theme,
        ThemeColorRef::new(ThemeColorSlot::Accent1, 5),
    );
    assert!(light.r > 68 && light.g > 114 && light.b > 196);
    assert!(dark.r < 68 && dark.g < 114 && dark.b < 196);
}

#[test]
fn u_f06_s3_theme_change_recolors_themed_runs() {
    let mut doc = Document::with_paragraph("Accent text");
    let reference = ThemeColorRef::new(ThemeColorSlot::Accent1, 3);
    doc.paragraph_at_mut(0, 0).unwrap().runs[0].format.theme_color = Some(reference);
    resolve_theme_colors(&mut doc);
    let office = doc.paragraph_at(0, 0).unwrap().runs[0]
        .format
        .color
        .expect("resolved color");
    assert_eq!(office.r, 68);

    doc.settings.theme = DocumentTheme::ion();
    resolve_theme_colors(&mut doc);
    let ion = doc.paragraph_at(0, 0).unwrap().runs[0]
        .format
        .color
        .expect("ion color");
    assert_eq!(ion.r, 255);
    assert_eq!(ion.g, 114);
    assert_eq!(ion.b, 0);
}

#[test]
fn u_f06_s3_picker_column_maps_to_slot() {
    assert_eq!(
        ThemeColorRef::from_picker(4, 2).map(|r| r.slot),
        Some(ThemeColorSlot::Accent1)
    );
    assert!(ThemeColorRef::from_picker(3, 2).is_none());
}
