//! F22.S4 — attach / clear digital signatures via edit commands.

use tw_edit::{Command, EditSession};
use tw_model::{sign_document, verify_signature, SignatureStatus, SignerInfo};

#[test]
fn u_f22_s4_add_and_clear_signature() {
    let mut session = EditSession::new();
    let signature = sign_document(
        &session.document,
        SignerInfo {
            name: "Signer".into(),
            email: "s@example.com".into(),
            organization: None,
        },
    )
    .unwrap();

    session
        .apply(Command::AddDigitalSignature {
            signature: signature.clone(),
        })
        .unwrap();
    assert_eq!(session.document.signatures.len(), 1);
    assert_eq!(
        verify_signature(&session.document, &session.document.signatures[0]).status,
        SignatureStatus::Valid
    );

    session.apply(Command::ClearDigitalSignatures).unwrap();
    assert!(session.document.signatures.is_empty());
}
