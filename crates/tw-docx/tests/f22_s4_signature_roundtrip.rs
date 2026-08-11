//! F22.S4 — DOCX embed / import digital signatures.

use tw_docx::{export_docx, import_docx, DocxPackage, SIGNATURES_PART};
use tw_model::{sign_document, verify_signature, Document, SignatureStatus, SignerInfo};

#[test]
fn u_f22_s4_docx_signature_roundtrip() {
    let mut doc = Document::with_paragraph("Signed contract");
    let sig = sign_document(
        &doc,
        SignerInfo {
            name: "Ada".into(),
            email: "ada@example.com".into(),
            organization: None,
        },
    )
    .unwrap();
    doc.signatures.push(sig);

    let bytes = export_docx(&doc, &DocxPackage::minimal()).unwrap();
    assert!(
        String::from_utf8_lossy(&bytes).contains("digitalSignatures")
            || {
                let imported = import_docx(&bytes).unwrap();
                imported.package.parts.contains_key(SIGNATURES_PART)
            }
    );

    let imported = import_docx(&bytes).unwrap();
    assert!(imported.package.parts.contains_key(SIGNATURES_PART));
    assert_eq!(imported.document.signatures.len(), 1);
    assert_eq!(imported.document.signatures[0].signer.name, "Ada");
    assert_eq!(
        verify_signature(&imported.document, &imported.document.signatures[0]).status,
        SignatureStatus::Valid
    );
}

#[test]
fn u_f22_s4_docx_clears_signatures_part() {
    let mut doc = Document::with_paragraph("Temp");
    doc.signatures.push(
        sign_document(
            &doc,
            SignerInfo {
                name: "X".into(),
                email: String::new(),
                organization: None,
            },
        )
        .unwrap(),
    );
    let with_sig = export_docx(&doc, &DocxPackage::minimal()).unwrap();
    let imported = import_docx(&with_sig).unwrap();
    assert!(imported.package.parts.contains_key(SIGNATURES_PART));

    let mut cleared = imported.document;
    cleared.signatures.clear();
    let stripped = export_docx(&cleared, &imported.package).unwrap();
    let reimported = import_docx(&stripped).unwrap();
    assert!(!reimported.package.parts.contains_key(SIGNATURES_PART));
}
