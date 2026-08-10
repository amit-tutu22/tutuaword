//! F22.S4 — digital signature content hash, sign, verify.

use tw_model::{
    document_content_hash, sign_document, verify_all_signatures, verify_signature, Document,
    SignatureStatus, SignerInfo,
};

#[test]
fn u_f22_s4_sign_and_verify_valid() {
    let doc = Document::with_paragraph("Contract body");
    let sig = sign_document(
        &doc,
        SignerInfo {
            name: "Ada Lovelace".into(),
            email: "ada@example.com".into(),
            organization: Some("Analytical Engines".into()),
        },
    )
    .unwrap();

    let mut signed = doc.clone();
    signed.signatures.push(sig.clone());

    let result = verify_signature(&signed, &sig);
    assert_eq!(result.status, SignatureStatus::Valid);
    assert_eq!(result.signer_name, "Ada Lovelace");
}

#[test]
fn u_f22_s4_detects_tamper() {
    let doc = Document::with_paragraph("Original");
    let sig = sign_document(
        &doc,
        SignerInfo {
            name: "Bob".into(),
            email: String::new(),
            organization: None,
        },
    )
    .unwrap();

    let mut signed = Document::with_paragraph("Tampered");
    signed.signatures.push(sig.clone());

    let result = verify_signature(&signed, &sig);
    assert_eq!(result.status, SignatureStatus::Tampered);
}

#[test]
fn u_f22_s4_hash_ignores_signatures_field() {
    let mut doc = Document::with_paragraph("Stable");
    let h1 = document_content_hash(&doc);
    let sig = sign_document(
        &doc,
        SignerInfo {
            name: "C".into(),
            email: String::new(),
            organization: None,
        },
    )
    .unwrap();
    doc.signatures.push(sig);
    let h2 = document_content_hash(&doc);
    assert_eq!(h1, h2);
}

#[test]
fn u_f22_s4_verify_all() {
    let doc = Document::with_paragraph("Multi");
    let mut signed = doc.clone();
    signed.signatures.push(
        sign_document(
            &doc,
            SignerInfo {
                name: "One".into(),
                email: String::new(),
                organization: None,
            },
        )
        .unwrap(),
    );
    signed.signatures.push(
        sign_document(
            &doc,
            SignerInfo {
                name: "Two".into(),
                email: String::new(),
                organization: None,
            },
        )
        .unwrap(),
    );
    let results = verify_all_signatures(&signed);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.status == SignatureStatus::Valid));
}

#[test]
fn u_f22_s4_signer_name_required() {
    let doc = Document::with_paragraph("x");
    let err = sign_document(
        &doc,
        SignerInfo {
            name: "  ".into(),
            email: String::new(),
            organization: None,
        },
    )
    .unwrap_err();
    assert!(err.contains("name"));
}
