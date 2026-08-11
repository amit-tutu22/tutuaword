//! F22.S2 — encrypt on save through FormatContext / export_document.

use tw_core::{
    export_document, import_document_bundle_with_password, DetectedFormat, FormatContext,
};
use tw_docx::is_password_protected;
use tw_model::Document;

#[test]
fn u_f22_s2_export_with_password() {
    let doc = Document::with_paragraph("Protected export");
    let mut ctx = FormatContext {
        save_format: DetectedFormat::Docx,
        encryption_password: Some("save-secret".into()),
        ..Default::default()
    };
    ctx.docx_package = Some(tw_docx::DocxPackage::minimal());

    let exported = export_document(&doc, &ctx).expect("export encrypted");
    assert!(is_password_protected(&exported).unwrap());

    let bundle = import_document_bundle_with_password(
        &exported,
        Some("out.docx"),
        Some("save-secret"),
    )
    .expect("re-open");
    assert_eq!(bundle.source_format, DetectedFormat::Docx);
}

#[test]
fn u_f22_s2_clear_password_plaintext() {
    let doc = Document::with_paragraph("Plain export");
    let mut ctx = FormatContext {
        save_format: DetectedFormat::Docx,
        encryption_password: Some("temp".into()),
        ..Default::default()
    };
    ctx.docx_package = Some(tw_docx::DocxPackage::minimal());
    assert!(is_password_protected(&export_document(&doc, &ctx).unwrap()).unwrap());

    ctx.encryption_password = None;
    let plain = export_document(&doc, &ctx).unwrap();
    assert!(!is_password_protected(&plain).unwrap());
    assert!(plain.starts_with(b"PK"));
}
