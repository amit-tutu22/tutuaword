//! F06.S3 — apply document theme from Design tab.

use tw_edit::{Command, EditSession};
use tw_model::{resolve_theme_color_ref, ThemeColorRef, ThemeColorSlot, THEME_FONT_PREFIX};

#[test]
fn u_f06_s3_set_document_theme_command() {
    let mut session = EditSession::new();
    session.document.styles.defaults.char_format.font_family =
        Some(format!("{THEME_FONT_PREFIX}minorHAnsi"));

    session
        .apply(Command::SetDocumentTheme {
            theme_name: "Ion".into(),
        })
        .expect("apply ion");

    assert_eq!(session.document.settings.theme.name, "Ion");
    assert_eq!(
        session
            .document
            .styles
            .defaults
            .char_format
            .font_family
            .as_deref(),
        Some("Arial")
    );
}

#[test]
fn u_f06_s3_set_document_theme_undo_restores_previous() {
    let mut session = EditSession::new();
    assert_eq!(session.document.settings.theme.name, "Office");

    session
        .apply(Command::SetDocumentTheme {
            theme_name: "Ion".into(),
        })
        .expect("apply ion");
    assert_eq!(session.document.settings.theme.name, "Ion");

    session.undo().expect("undo");
    assert_eq!(session.document.settings.theme.name, "Office");
}

#[test]
fn u_f06_s3_set_document_theme_recolors_themed_runs() {
    let mut session = EditSession::new();
    let reference = ThemeColorRef::new(ThemeColorSlot::Accent1, 3);
    session.document.sections[0].blocks[0]
        .paragraph_mut()
        .unwrap()
        .runs[0]
        .format
        .theme_color = Some(reference);

    session
        .apply(Command::SetDocumentTheme {
            theme_name: "Ion".into(),
        })
        .expect("apply ion");

    let color = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .format
        .color
        .expect("resolved");
    let expected = resolve_theme_color_ref(&session.document.settings.theme, reference);
    assert_eq!(color, expected);
}
