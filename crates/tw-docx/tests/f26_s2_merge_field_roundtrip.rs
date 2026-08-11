//! F26.S2 — MERGEFIELD DOCX round-trip (integration).

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::{FieldType, RunContent, apply_mail_merge_row};
use std::collections::BTreeMap;

fn package_part(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    use std::io::{Cursor, Read};
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;
    Some(data)
}

#[test]
fn i_f26_s2_merge_field_docx_roundtrip() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertMergeField {
            run_id,
            offset: 0,
            name: "Name".into(),
        })
        .unwrap();

    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let document_xml = package_part(&bytes, "word/document.xml").expect("document.xml");
    let xml = String::from_utf8_lossy(&document_xml);
    assert!(
        xml.contains("MERGEFIELD") && xml.contains("Name"),
        "export must emit MERGEFIELD Name: {xml}"
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
                    RunContent::Field(f) if f.field_type == FieldType::MergeField => Some(f),
                    _ => None,
                })
            })
        })
        .expect("merge field missing after round-trip");
    assert_eq!(field.merge_name.as_deref(), Some("Name"));
    assert_eq!(field.display_text.as_deref(), Some("«Name»"));
}

#[test]
fn i_f26_s2_merge_then_export_plaintext() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertMergeField {
            run_id,
            offset: 0,
            name: "City".into(),
        })
        .unwrap();
    let mut values = BTreeMap::new();
    values.insert("City".into(), "Paris".into());
    apply_mail_merge_row(&mut session.document, &values);

    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let document_xml = package_part(&bytes, "word/document.xml").expect("document.xml");
    let xml = String::from_utf8_lossy(&document_xml);
    assert!(xml.contains("Paris"), "merged value missing: {xml}");
    assert!(
        !xml.contains("MERGEFIELD"),
        "merge field should be replaced with text before export in this path"
    );
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f26_s2_merge_field_docx_churn() {
    let mut session = EditSession::new();
    for name in ["Name", "City", "Email", "Title", "Company"] {
        let para = session.document.paragraph_at(0, 0).unwrap();
        let run_id = para.runs.last().unwrap().id;
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        session
            .apply(Command::InsertMergeField {
                run_id,
                offset,
                name: name.into(),
            })
            .unwrap();
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
        .filter(|r| matches!(&r.content, RunContent::Field(f) if f.field_type == FieldType::MergeField))
        .count();
    assert_eq!(count, 5);
}
