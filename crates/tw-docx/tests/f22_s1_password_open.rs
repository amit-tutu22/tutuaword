//! F22.S1 — password-protected DOCX open / decrypt.

use std::path::PathBuf;
use tw_docx::{
    decrypt_with_password, import_docx_with_password, is_password_protected, DocxError,
};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/password_protected_standard.docx")
}

fn encrypted_fixture() -> Vec<u8> {
    std::fs::read(fixture_path()).expect("password_protected_standard.docx fixture")
}

#[test]
fn u_f22_s1_detects_ole_encrypted_fixture() {
    let bytes = encrypted_fixture();
    assert!(is_password_protected(&bytes).unwrap());
}

#[test]
fn u_f22_s1_import_without_password_errors() {
    match import_docx_with_password(&encrypted_fixture(), None) {
        Err(DocxError::PasswordProtected) => {}
        Err(err) => panic!("expected PasswordProtected, got {err}"),
        Ok(_) => panic!("expected PasswordProtected, got Ok"),
    }
}

#[test]
fn u_f22_s1_wrong_password_errors() {
    let err = decrypt_with_password(&encrypted_fixture(), "wrong-password").unwrap_err();
    assert!(matches!(
        err,
        DocxError::IncorrectPassword | DocxError::DecryptUnsupported(_)
    ));
}

#[test]
fn u_f22_s1_correct_password_decrypts_and_imports() {
    let bytes = encrypted_fixture();
    let decrypted = decrypt_with_password(&bytes, "Password1234_").expect("decrypt");
    assert!(
        decrypted.starts_with(b"PK"),
        "decrypted package should be a ZIP"
    );
    let imported = import_docx_with_password(&bytes, Some("Password1234_"))
        .expect("import with password");
    assert!(
        !imported.document.sections.is_empty(),
        "imported document should have sections"
    );
}
