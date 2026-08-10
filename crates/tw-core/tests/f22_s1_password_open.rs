//! F22.S1 — password-protected open through the core import bundle.

use std::path::PathBuf;
use tw_core::{
    import_document_bundle, import_document_bundle_with_password, DetectedFormat, ImportError,
};

fn encrypted_fixture() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tw-docx/tests/corpus/password_protected_standard.docx");
    std::fs::read(path).expect("password_protected_standard.docx fixture")
}

#[test]
fn u_f22_s1_bundle_requires_password() {
    let err = import_document_bundle(&encrypted_fixture(), Some("locked.docx")).unwrap_err();
    assert!(matches!(err, ImportError::PasswordProtected));
}

#[test]
fn u_f22_s1_bundle_wrong_password() {
    let err = import_document_bundle_with_password(
        &encrypted_fixture(),
        Some("locked.docx"),
        Some("nope"),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ImportError::IncorrectPassword | ImportError::DecryptUnsupported(_)
    ));
}

#[test]
fn u_f22_s1_bundle_opens_with_password() {
    let bundle = import_document_bundle_with_password(
        &encrypted_fixture(),
        Some("locked.docx"),
        Some("Password1234_"),
    )
    .expect("open encrypted docx");
    assert_eq!(bundle.source_format, DetectedFormat::Docx);
    assert!(!bundle.document.sections.is_empty());
}
