//! F22.S3 — export strips comments and clears core properties.

use tw_docx::{export_docx, import_docx, DocxPackage};
use tw_edit::{Command, EditSession};
use tw_model::Document;

#[test]
fn u_f22_s3_export_clears_core_properties() {
    let mut doc = Document::with_paragraph("Body");
    doc.properties.title = Some("Old Title".into());
    doc.properties.author = Some("Old Author".into());

    let bytes = export_docx(&doc, &DocxPackage::minimal()).unwrap();
    let imported = import_docx(&bytes).unwrap();
    assert_eq!(imported.document.properties.title.as_deref(), Some("Old Title"));
    assert_eq!(
        imported.document.properties.author.as_deref(),
        Some("Old Author")
    );

    // Clear via model and re-export.
    let mut cleared = imported.document;
    cleared.properties.title = None;
    cleared.properties.author = None;
    let package = imported.package;
    let bytes2 = export_docx(&cleared, &package).unwrap();
    let reimported = import_docx(&bytes2).unwrap();
    assert!(reimported.document.properties.title.is_none());
    assert!(reimported.document.properties.author.is_none());
}

#[test]
fn u_f22_s3_export_strips_comments_part() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertComment {
            run_id,
            offset: 0,
            body_text: "private".into(),
        })
        .unwrap();

    let with_comments = export_docx(&session.document, &DocxPackage::minimal()).unwrap();
    let imported = import_docx(&with_comments).unwrap();
    assert!(
        imported.package.parts.contains_key("word/comments.xml"),
        "expected comments part after insert"
    );

    session
        .apply(Command::RemoveInspectFindings {
            comments: true,
            metadata: false,
            hidden_text: false,
        })
        .unwrap();

    let stripped = export_docx(&session.document, &imported.package).unwrap();
    let reimported = import_docx(&stripped).unwrap();
    assert!(
        !reimported.package.parts.contains_key("word/comments.xml"),
        "comments part should be removed after inspect"
    );
    assert!(reimported.document.comments.is_empty());
}
