//! F22.S2 — encrypt DOCX on save.

use tw_docx::{
    decrypt_with_password, encrypt_with_password, import_docx_with_password,
    is_password_protected, DocxError,
};
use tw_model::Document;

fn plain_docx_bytes() -> Vec<u8> {
    let doc = Document::with_paragraph("Secret body");
    let package = tw_docx::DocxPackage::minimal();
    tw_docx::export(&doc, &package).expect("export plaintext docx")
}

#[test]
fn u_f22_s2_encrypt_produces_protected() {
    let plain = plain_docx_bytes();
    assert!(!is_password_protected(&plain).unwrap());
    let encrypted = encrypt_with_password(&plain, "hunter2").expect("encrypt");
    assert!(is_password_protected(&encrypted).unwrap());
    assert!(encrypted.starts_with(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1"));
}

#[test]
fn u_f22_s2_encrypt_decrypt_roundtrip() {
    let plain = plain_docx_bytes();
    let encrypted = encrypt_with_password(&plain, "roundtrip-pass").unwrap();
    let decrypted = decrypt_with_password(&encrypted, "roundtrip-pass").unwrap();
    assert!(decrypted.starts_with(b"PK"));
    let imported = import_docx_with_password(&encrypted, Some("roundtrip-pass")).unwrap();
    let text: String = imported
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .flat_map(|p| p.runs.iter())
        .map(|r| r.text().to_string())
        .collect();
    assert!(text.contains("Secret body"), "got {text:?}");
}

#[test]
fn u_f22_s2_wrong_password_rejects() {
    let encrypted = encrypt_with_password(&plain_docx_bytes(), "correct").unwrap();
    let err = decrypt_with_password(&encrypted, "wrong").unwrap_err();
    assert!(matches!(
        err,
        DocxError::IncorrectPassword | DocxError::DecryptUnsupported(_)
    ));
}

#[test]
fn u_f22_s2_empty_password_rejected() {
    let err = encrypt_with_password(&plain_docx_bytes(), "").unwrap_err();
    assert!(matches!(err, DocxError::EncryptFailed(_)));
}

