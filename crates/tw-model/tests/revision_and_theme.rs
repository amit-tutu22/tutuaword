use tw_model::{
    Document, DocumentSettings, DocumentTheme, Revision, RevisionType, ThemeColor,
};

#[test]
fn revision_insert_and_delete_set_type_and_author() {
    let insert = Revision::insert("Alice");
    assert_eq!(insert.revision_type, RevisionType::Insert);
    assert_eq!(insert.author, "Alice");

    let delete = Revision::delete("Bob");
    assert_eq!(delete.revision_type, RevisionType::Delete);
    assert_eq!(delete.author, "Bob");
}

#[test]
fn revision_serializes_and_deserializes() {
    let rev = Revision::insert("Reviewer");
    let json = serde_json::to_string(&rev).unwrap();
    let parsed: Revision = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.revision_type, RevisionType::Insert);
    assert_eq!(parsed.author, "Reviewer");
}

#[test]
fn document_settings_include_track_changes_and_author() {
    let settings = DocumentSettings::default_settings();
    assert!(!settings.track_changes_enabled);
    assert_eq!(settings.author_name, "Author");
    assert!(settings.theme.name.contains("Office"));
}

#[test]
fn document_theme_defaults_match_office_palette() {
    let theme = DocumentTheme::default();
    assert_eq!(theme.major_font, "Calibri Light");
    assert_eq!(theme.minor_font, "Calibri");
    assert_eq!(
        theme.accent1,
        ThemeColor {
            r: 68,
            g: 114,
            b: 196
        }
    );
}

#[test]
fn document_settings_round_trip_through_serialization() {
    let mut doc = Document::new();
    doc.settings.track_changes_enabled = true;
    doc.settings.author_name = "Editor".into();
    doc.settings.template_name = Some("Business Letter".into());

    let json = serde_json::to_string(&doc.settings).unwrap();
    let restored: DocumentSettings = serde_json::from_str(&json).unwrap();
    assert!(restored.track_changes_enabled);
    assert_eq!(restored.author_name, "Editor");
    assert_eq!(restored.template_name.as_deref(), Some("Business Letter"));
}
