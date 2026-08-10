//! DOCX custom XML part for digital signatures (F22.S4).

use tw_model::{DigitalSignature, SignerInfo};

pub const SIGNATURES_PART: &str = "customXml/digitalSignatures.xml";

pub fn serialize_signatures_xml(signatures: &[DigitalSignature]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><tw:signatures xmlns:tw="http://tutuaword.dev/schemas/digital-signatures">"#,
    );
    for sig in signatures {
        xml.push_str("<tw:signature>");
        xml.push_str(&format!("<tw:id>{}</tw:id>", escape_xml(&sig.id)));
        xml.push_str(&format!(
            "<tw:name>{}</tw:name>",
            escape_xml(&sig.signer.name)
        ));
        xml.push_str(&format!(
            "<tw:email>{}</tw:email>",
            escape_xml(&sig.signer.email)
        ));
        if let Some(org) = &sig.signer.organization {
            xml.push_str(&format!(
                "<tw:organization>{}</tw:organization>",
                escape_xml(org)
            ));
        }
        xml.push_str(&format!(
            "<tw:timestamp>{}</tw:timestamp>",
            escape_xml(&sig.timestamp.to_rfc3339())
        ));
        xml.push_str(&format!(
            "<tw:publicKey>{}</tw:publicKey>",
            escape_xml(&sig.public_key)
        ));
        xml.push_str(&format!(
            "<tw:signatureValue>{}</tw:signatureValue>",
            escape_xml(&sig.signature_value)
        ));
        xml.push_str(&format!(
            "<tw:contentHash>{}</tw:contentHash>",
            escape_xml(&sig.signed_content_hash)
        ));
        xml.push_str("</tw:signature>");
    }
    xml.push_str("</tw:signatures>");
    xml
}

pub fn parse_signatures_xml(xml: &str) -> Vec<DigitalSignature> {
    let mut signatures = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<tw:signature>") {
        let after = &rest[start + "<tw:signature>".len()..];
        let end = after.find("</tw:signature>").unwrap_or(after.len());
        let block = &after[..end];
        let id = read_tag(block, "tw:id").unwrap_or_default();
        if id.is_empty() {
            rest = &after[end..];
            continue;
        }
        let name = read_tag(block, "tw:name").unwrap_or_else(|| "Signer".into());
        let email = read_tag(block, "tw:email").unwrap_or_default();
        let organization = read_tag(block, "tw:organization");
        let timestamp = read_tag(block, "tw:timestamp")
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);
        let public_key = read_tag(block, "tw:publicKey").unwrap_or_default();
        let signature_value = read_tag(block, "tw:signatureValue").unwrap_or_default();
        let signed_content_hash = read_tag(block, "tw:contentHash").unwrap_or_default();
        if public_key.is_empty() || signature_value.is_empty() || signed_content_hash.is_empty() {
            rest = &after[end..];
            continue;
        }
        signatures.push(DigitalSignature {
            id,
            signer: SignerInfo {
                name,
                email,
                organization,
            },
            timestamp,
            public_key,
            signature_value,
            signed_content_hash,
        });
        rest = &after[end..];
    }
    signatures
}

fn read_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    let text = unescape_xml(xml[start..end].trim());
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}
