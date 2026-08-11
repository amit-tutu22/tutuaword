//! F22.S4 — twdoc signatures.json round-trip.

use tw_model::{sign_document, verify_signature, Document, SignatureStatus, SignerInfo};
use tw_native::NativeFormat;

#[test]
fn u_f22_s4_twdoc_signature_roundtrip() {
    let mut doc = Document::with_paragraph("Native signed");
    doc.signatures.push(
        sign_document(
            &doc,
            SignerInfo {
                name: "Native Signer".into(),
                email: "n@example.com".into(),
                organization: Some("Org".into()),
            },
        )
        .unwrap(),
    );

    let bytes = NativeFormat::export(&doc).unwrap();
    let loaded = NativeFormat::import(&bytes).unwrap();
    assert_eq!(loaded.signatures.len(), 1);
    assert_eq!(loaded.signatures[0].signer.name, "Native Signer");
    assert_eq!(
        verify_signature(&loaded, &loaded.signatures[0]).status,
        SignatureStatus::Valid
    );
}
