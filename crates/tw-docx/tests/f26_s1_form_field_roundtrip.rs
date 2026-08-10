//! F26.S1 — form field DOCX export/import round-trip (integration).

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::{FieldType, FormFieldKind, RunContent};

fn package_part(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    use std::io::{Cursor, Read};
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;
    Some(data)
}

fn insert_form_text(session: &mut EditSession) {
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertFormField {
            run_id,
            offset: 0,
            kind: FormFieldKind::PlainText,
            name: Some("City".into()),
            initial_value: Some("Paris".into()),
        })
        .unwrap();
}

fn insert_checkbox(session: &mut EditSession, checked: bool) {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let run_id = para.runs.last().unwrap().id;
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    session
        .apply(Command::InsertFormField {
            run_id,
            offset,
            kind: FormFieldKind::Checkbox,
            name: Some("OptIn".into()),
            initial_value: Some(checked.to_string()),
        })
        .unwrap();
}

#[test]
fn i_f26_s1_form_text_docx_roundtrip() {
    let mut session = EditSession::new();
    insert_form_text(&mut session);

    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let document_xml = package_part(&bytes, "word/document.xml").expect("document.xml");
    let xml = String::from_utf8_lossy(&document_xml);
    assert!(
        xml.contains("FORMTEXT") && xml.contains("Paris"),
        "export must emit FORMTEXT with display text: {xml}"
    );

    let imported = import(&bytes).expect("import");
    let field = imported
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .find_map(|block| {
            block.paragraph().and_then(|para| {
                para.runs.iter().find_map(|run| match &run.content {
                    RunContent::Field(f) if f.field_type == FieldType::FormText => Some(f),
                    _ => None,
                })
            })
        })
        .expect("form text missing after round-trip");
    assert_eq!(field.display_text.as_deref(), Some("Paris"));
    assert_eq!(
        field.form.as_ref().and_then(|f| f.default_text.as_deref()),
        Some("Paris")
    );
}

#[test]
fn i_f26_s1_form_checkbox_docx_roundtrip() {
    let mut session = EditSession::new();
    insert_checkbox(&mut session, true);

    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let document_xml = package_part(&bytes, "word/document.xml").expect("document.xml");
    let xml = String::from_utf8_lossy(&document_xml);
    assert!(
        xml.contains("FORMCHECKBOX"),
        "export must emit FORMCHECKBOX: {xml}"
    );

    let imported = import(&bytes).expect("import");
    let field = imported
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .find_map(|block| {
            block.paragraph().and_then(|para| {
                para.runs.iter().find_map(|run| match &run.content {
                    RunContent::Field(f) if f.field_type == FieldType::FormCheckbox => Some(f),
                    _ => None,
                })
            })
        })
        .expect("checkbox missing after round-trip");
    assert_eq!(field.form.as_ref().and_then(|f| f.checked), Some(true));
    assert_eq!(field.display_text.as_deref(), Some("☑"));
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f26_s1_form_field_docx_roundtrip_churn() {
    let mut session = EditSession::new();
    for i in 0..40 {
        if i % 2 == 0 {
            let para = session.document.paragraph_at(0, 0).unwrap();
            let run_id = para.runs.last().unwrap().id;
            let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
            session
                .apply(Command::InsertFormField {
                    run_id,
                    offset,
                    kind: FormFieldKind::PlainText,
                    name: Some(format!("t{i}")),
                    initial_value: Some(format!("val{i}")),
                })
                .unwrap();
        } else {
            insert_checkbox(&mut session, i % 4 == 1);
        }
    }

    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let imported = import(&bytes).expect("import");
    let count = imported
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .flat_map(|b| b.paragraph().into_iter())
        .flat_map(|p| p.runs.iter())
        .filter(|r| {
            matches!(
                &r.content,
                RunContent::Field(f)
                    if matches!(f.field_type, FieldType::FormText | FieldType::FormCheckbox)
            )
        })
        .count();
    assert_eq!(count, 40);
}
