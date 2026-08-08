//! F06.S2 — custom style edit commands.

use tw_edit::{Command, EditError, EditSession};
use tw_model::{CharFormat, ParaFormat};

#[test]
fn u_f06_s2_create_rename_delete_commands() {
    let mut session = EditSession::new();
    session
        .apply(Command::CreateParagraphStyle {
            name: "Draft".into(),
            based_on_name: Some("Normal".into()),
            char_format: CharFormat {
                italic: Some(true),
                ..Default::default()
            },
            para_format: ParaFormat::default(),
        })
        .expect("create");

    let draft = session
        .document
        .styles
        .find_style_by_name("Draft")
        .expect("draft style");
    assert_eq!(
        session
            .document
            .styles
            .resolve_char_format(Some(draft.id), &CharFormat::default())
            .italic,
        Some(true)
    );

    session
        .apply(Command::RenameParagraphStyle {
            style_name: "Draft".into(),
            new_name: "Draft v2".into(),
        })
        .expect("rename");
    assert!(
        session
            .document
            .styles
            .find_style_by_name("Draft v2")
            .is_some()
    );

    let draft_id = session
        .document
        .styles
        .find_style_by_name("Draft v2")
        .unwrap()
        .id;
    session
        .apply(Command::DeleteParagraphStyle { style_id: draft_id })
        .expect("delete");
    assert!(
        session
            .document
            .styles
            .find_style_by_name("Draft v2")
            .is_none()
    );
}

#[test]
fn u_f06_s2_builtin_styles_are_protected() {
    let mut session = EditSession::new();
    let heading = session
        .document
        .styles
        .find_style_by_name("Heading 1")
        .unwrap()
        .id;

    let rename_err = session
        .apply(Command::RenameParagraphStyle {
            style_name: "Heading 1".into(),
            new_name: "Title".into(),
        })
        .unwrap_err();
    assert!(matches!(rename_err, EditError::BuiltinStyleProtected(_)));

    let delete_err = session
        .apply(Command::DeleteParagraphStyle { style_id: heading })
        .unwrap_err();
    assert!(matches!(delete_err, EditError::BuiltinStyleProtected(_)));
}
