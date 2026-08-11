//! Digital signatures over document content (F22.S4).
//!
//! Signs a SHA-256 content hash with Ed25519. The verifying (public) key is
//! embedded with the signature so verification needs no external PKI. Full
//! PKCS#7 / CAdES OOXML packaging remains a compatibility follow-up.
//!
//! The content hash is ID-stable (ignores `NodeId`s) so signatures survive
//! DOCX / twdoc round-trips.

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    Block, Document, HeaderFooterType, Paragraph, Run, RunContent, Table,
};

/// Identity of the person who signed the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerInfo {
    pub name: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
}

/// One digital signature attached to a document (F22.S4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub id: String,
    pub signer: SignerInfo,
    pub timestamp: DateTime<Utc>,
    /// Ed25519 verifying key (32 bytes), base64.
    pub public_key: String,
    /// Ed25519 signature over [`signed_content_hash`], base64.
    pub signature_value: String,
    /// SHA-256 of the document content (ID-stable), hex lowercase.
    pub signed_content_hash: String,
}

/// Result of verifying a single signature against current document content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureStatus {
    Valid,
    Invalid,
    /// Document content changed since signing.
    Tampered,
}

/// Verification outcome for one stored signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub signature_id: String,
    pub status: SignatureStatus,
    pub signer_name: String,
    pub message: String,
}

/// SHA-256 of document content excluding ephemeral IDs and the signatures list.
pub fn document_content_hash(doc: &Document) -> String {
    let mut hasher = Sha256::new();
    feed_str(&mut hasher, "title", doc.properties.title.as_deref().unwrap_or(""));
    feed_str(
        &mut hasher,
        "author",
        doc.properties.author.as_deref().unwrap_or(""),
    );
    for (si, section) in doc.sections.iter().enumerate() {
        feed_str(&mut hasher, "section", &si.to_string());
        hash_blocks(&mut hasher, &section.blocks);
        let mut header_kinds: Vec<_> = section.headers.keys().copied().collect();
        header_kinds.sort_by_key(header_footer_sort_key);
        for kind in header_kinds {
            feed_str(&mut hasher, "header", &header_footer_label(kind));
            if let Some(hf) = section.headers.get(&kind) {
                hash_blocks(&mut hasher, &hf.blocks);
            }
        }
        let mut footer_kinds: Vec<_> = section.footers.keys().copied().collect();
        footer_kinds.sort_by_key(header_footer_sort_key);
        for kind in footer_kinds {
            feed_str(&mut hasher, "footer", &header_footer_label(kind));
            if let Some(hf) = section.footers.get(&kind) {
                hash_blocks(&mut hasher, &hf.blocks);
            }
        }
    }
    for source in &doc.bibliography_sources {
        feed_str(&mut hasher, "bib", &source.key);
        feed_str(&mut hasher, "bib_author", &source.author);
        feed_str(&mut hasher, "bib_title", &source.title);
        feed_str(&mut hasher, "bib_year", &source.year);
    }
    for footnote in &doc.footnotes {
        feed_str(&mut hasher, "footnote", &footnote.id.to_string());
        hash_blocks(&mut hasher, &footnote.blocks);
    }
    for thread in &doc.comments {
        feed_str(&mut hasher, "comment", &thread.comment_id.to_string());
        for message in &thread.messages {
            feed_str(&mut hasher, "comment_author", &message.author);
            hash_blocks(&mut hasher, &message.body);
        }
    }
    hex_encode(&hasher.finalize())
}

/// Sign the current document content; returns a new signature (does not attach).
pub fn sign_document(doc: &Document, signer: SignerInfo) -> Result<DigitalSignature, String> {
    if signer.name.trim().is_empty() {
        return Err("Signer name is required".into());
    }
    let hash_hex = document_content_hash(doc);
    let hash_bytes = hex_decode(&hash_hex)?;

    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    let signature = signing_key.sign(&hash_bytes);

    Ok(DigitalSignature {
        id: uuid::Uuid::new_v4().to_string(),
        signer,
        timestamp: Utc::now(),
        public_key: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            verifying_key.as_bytes(),
        ),
        signature_value: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            signature.to_bytes(),
        ),
        signed_content_hash: hash_hex,
    })
}

/// Verify one signature against the current document.
pub fn verify_signature(doc: &Document, signature: &DigitalSignature) -> SignatureVerification {
    let current_hash = document_content_hash(doc);
    if current_hash != signature.signed_content_hash {
        return SignatureVerification {
            signature_id: signature.id.clone(),
            status: SignatureStatus::Tampered,
            signer_name: signature.signer.name.clone(),
            message: "Document has changed since it was signed".into(),
        };
    }

    let status = match verify_cryptographic(signature) {
        Ok(true) => SignatureStatus::Valid,
        Ok(false) | Err(_) => SignatureStatus::Invalid,
    };
    let message = match status {
        SignatureStatus::Valid => "Signature is valid".into(),
        SignatureStatus::Invalid => "Signature cryptographic check failed".into(),
        SignatureStatus::Tampered => "Document has changed since it was signed".into(),
    };
    SignatureVerification {
        signature_id: signature.id.clone(),
        status,
        signer_name: signature.signer.name.clone(),
        message,
    }
}

/// Verify every signature on the document.
pub fn verify_all_signatures(doc: &Document) -> Vec<SignatureVerification> {
    doc.signatures
        .iter()
        .map(|sig| verify_signature(doc, sig))
        .collect()
}

fn verify_cryptographic(signature: &DigitalSignature) -> Result<bool, String> {
    let pk_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &signature.public_key,
    )
    .map_err(|e| e.to_string())?;
    let pk_arr: [u8; 32] = pk_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "invalid public key length".to_string())?;
    let verifying_key = VerifyingKey::from_bytes(&pk_arr).map_err(|e| e.to_string())?;

    let sig_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &signature.signature_value,
    )
    .map_err(|e| e.to_string())?;
    let sig_arr: [u8; 64] = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "invalid signature length".to_string())?;
    let sig = Signature::from_bytes(&sig_arr);

    let hash = hex_decode(&signature.signed_content_hash)?;
    Ok(verifying_key.verify(&hash, &sig).is_ok())
}

fn hash_blocks(hasher: &mut Sha256, blocks: &[Block]) {
    feed_str(hasher, "blocks", &blocks.len().to_string());
    for block in blocks {
        match block {
            Block::Paragraph(para) => {
                hasher.update(b"P");
                hash_paragraph(hasher, para);
            }
            Block::Table(table) => {
                hasher.update(b"T");
                hash_table(hasher, table);
            }
            Block::ImageBlock(image) => {
                hasher.update(b"I");
                feed_str(hasher, "img_len", &image.data.bytes.len().to_string());
                hasher.update(Sha256::digest(&image.data.bytes));
                feed_str(
                    hasher,
                    "img_w",
                    &image.display_width.to_bits().to_string(),
                );
                feed_str(
                    hasher,
                    "img_h",
                    &image.display_height.to_bits().to_string(),
                );
                feed_str(
                    hasher,
                    "img_alt",
                    image.alt_text.as_deref().unwrap_or(""),
                );
            }
            Block::ShapeBlock(shape) => {
                hasher.update(b"S");
                feed_str(
                    hasher,
                    "shape",
                    &format!("{:?}", shape.shape.shape_type),
                );
                for para in &shape.paragraphs {
                    hash_paragraph(hasher, para);
                }
            }
        }
    }
}

fn hash_table(hasher: &mut Sha256, table: &Table) {
    feed_str(hasher, "rows", &table.rows.len().to_string());
    for row in &table.rows {
        feed_str(hasher, "cells", &row.cells.len().to_string());
        for cell in &row.cells {
            hash_blocks(hasher, &cell.blocks);
        }
    }
}

fn hash_paragraph(hasher: &mut Sha256, para: &Paragraph) {
    feed_str(hasher, "runs", &para.runs.len().to_string());
    for run in &para.runs {
        hash_run(hasher, run);
    }
}

fn hash_run(hasher: &mut Sha256, run: &Run) {
    match &run.content {
        RunContent::Text(text) => {
            hasher.update(b"txt");
            feed_str(hasher, "t", text);
        }
        RunContent::Tab => hasher.update(b"tab"),
        RunContent::Break(kind) => {
            hasher.update(b"brk");
            feed_str(hasher, "k", &format!("{kind:?}"));
        }
        RunContent::Hyperlink { text, target } => {
            hasher.update(b"lnk");
            feed_str(hasher, "t", text);
            feed_str(hasher, "u", &target.url);
            feed_str(
                hasher,
                "a",
                target.anchor.as_deref().unwrap_or(""),
            );
        }
        RunContent::Field(field) => {
            hasher.update(b"fld");
            feed_str(
                hasher,
                "d",
                field.display_text.as_deref().unwrap_or(""),
            );
        }
        RunContent::InlineImage(img) => {
            hasher.update(b"iimg");
            feed_str(hasher, "l", &img.image.bytes.len().to_string());
            hasher.update(Sha256::digest(&img.image.bytes));
        }
        RunContent::FootnoteRef(note) => {
            hasher.update(b"fnr");
            feed_str(hasher, "n", &note.note_id.to_string());
        }
        RunContent::EndnoteRef(note) => {
            hasher.update(b"enr");
            feed_str(hasher, "n", &note.note_id.to_string());
        }
        RunContent::CitationRef(cite) => {
            hasher.update(b"cit");
            feed_str(hasher, "k", &cite.source_key);
            feed_str(
                hasher,
                "d",
                cite.display_text.as_deref().unwrap_or(""),
            );
        }
        RunContent::CommentRef(c) => {
            hasher.update(b"cmr");
            feed_str(hasher, "c", &c.comment_id.to_string());
        }
        RunContent::Bookmark(b) => {
            hasher.update(b"bm");
            feed_str(hasher, "n", &b.name);
        }
        RunContent::OfficeMath { xml } => {
            hasher.update(b"oml");
            feed_str(hasher, "x", xml);
        }
    }
    let f = &run.format;
    if f.bold == Some(true) {
        hasher.update(b"B");
    }
    if f.italic == Some(true) {
        hasher.update(b"I");
    }
    if f.underline.is_some() {
        hasher.update(b"U");
    }
    if f.hidden == Some(true) {
        hasher.update(b"H");
    }
    if let Some(size) = f.font_size {
        feed_str(hasher, "sz", &size.to_bits().to_string());
    }
    if let Some(family) = &f.font_family {
        feed_str(hasher, "ff", family);
    }
}

fn header_footer_sort_key(kind: &HeaderFooterType) -> u8 {
    match kind {
        HeaderFooterType::Default => 0,
        HeaderFooterType::First => 1,
        HeaderFooterType::Even => 2,
        HeaderFooterType::Odd => 3,
    }
}

fn header_footer_label(kind: HeaderFooterType) -> String {
    match kind {
        HeaderFooterType::Default => "default".into(),
        HeaderFooterType::First => "first".into(),
        HeaderFooterType::Even => "even".into(),
        HeaderFooterType::Odd => "odd".into(),
    }
}

fn feed_str(hasher: &mut Sha256, label: &str, value: &str) {
    hasher.update(label.as_bytes());
    hasher.update([0]);
    hasher.update(value.as_bytes());
    hasher.update([0]);
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn hex_nibble(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err("invalid hex digit".into()),
    }
}
