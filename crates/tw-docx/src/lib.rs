use std::collections::{HashMap, HashSet};
use thiserror::Error;
use tw_model::Document;

pub type PartName = String;

mod bibliography;
mod comments;
pub mod chart;
pub mod diagram;
mod export;
mod encryption;
pub mod fonts;
pub mod fingerprint;
mod hyperlink;
mod import;
mod media;
mod numbering;
mod opc;
mod paragraph;
mod preserve;
mod properties;
pub mod retention;
mod signatures;
mod styles;
mod table;
mod vba;
mod xml_util;

pub use encryption::{
    decrypt_with_password, encrypt_with_password, is_password_protected,
};
pub use export::export_docx;
pub use fonts::EmbeddedFont;
pub use import::{import_docx, import_docx_with_password};
pub use bibliography::BIBLIOGRAPHY_PART;
pub use comments::COMMENTS_PART;
pub use retention::ImportRetentionReport;
pub use signatures::SIGNATURES_PART;
pub use vba::{is_vba_part, package_has_vba_parts};

/// Original OPC package retained for passthrough export (ADR-0008).
#[derive(Debug, Clone, Default)]
pub struct DocxPackage {
    pub parts: HashMap<PartName, Vec<u8>>,
    pub modified_parts: HashSet<PartName>,
    pub original_bytes: Option<Vec<u8>>,
    /// Fingerprint of the document as imported. Passthrough export is only
    /// safe while the document still matches it.
    pub source_fingerprint: Option<u64>,
    /// Fingerprints of Tier B catalogs at import time (R3.3).
    pub source_numbering_fingerprint: Option<u64>,
    pub source_styles_fingerprint: Option<u64>,
    /// Unedited paragraph XML preserved for within-part round-trip (R3.3).
    pub preserved_paragraphs: preserve::PreservedParagraphMap,
    /// Unedited shape paragraph XML preserved for DrawingML passthrough (F11.S1).
    pub preserved_shapes: preserve::PreservedShapeMap,
}

impl DocxPackage {
    pub fn mark_modified(&mut self, part: PartName) {
        self.modified_parts.insert(part);
    }

    pub fn is_pristine(&self) -> bool {
        self.modified_parts.is_empty()
    }

    /// Whether the original bytes can stand in for exporting `doc`. Requires
    /// both that nothing marked a part modified and that the document still
    /// hashes to what was imported, so an edit that forgot to flag itself
    /// cannot be silently dropped.
    pub fn can_pass_through(&self, doc: &Document) -> bool {
        self.is_pristine()
            && self.original_bytes.is_some()
            && self.source_fingerprint == Some(fingerprint::document_fingerprint(doc))
    }

    /// Empty package for exporting a document that was not opened from DOCX.
    pub fn minimal() -> Self {
        let mut parts = HashMap::new();
        parts.insert("[Content_Types].xml".into(), MINIMAL_CONTENT_TYPES.into());
        parts.insert(
            "word/_rels/document.xml.rels".into(),
            b"<Relationships/>".to_vec(),
        );
        Self {
            parts,
            modified_parts: HashSet::new(),
            original_bytes: None,
            source_fingerprint: None,
            source_numbering_fingerprint: None,
            source_styles_fingerprint: None,
            preserved_paragraphs: preserve::PreservedParagraphMap::new(),
            preserved_shapes: preserve::PreservedShapeMap::new(),
        }
    }
}

pub(crate) const MINIMAL_CONTENT_TYPES: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

#[derive(Debug, Error)]
pub enum DocxError {
    #[error("word/document.xml missing from docx package")]
    MissingDocumentPart,
    #[error("invalid docx package: {0}")]
    InvalidPackage(String),
    #[error("document is password-protected")]
    PasswordProtected,
    #[error("incorrect password")]
    IncorrectPassword,
    #[error("document decryption is unsupported: {0}")]
    DecryptUnsupported(String),
    #[error("document encryption failed: {0}")]
    EncryptFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub struct ImportResult {
    pub document: Document,
    pub package: DocxPackage,
    pub retention: ImportRetentionReport,
    /// Deobfuscated embedded fonts from `fontTable.xml`, ready for registration.
    pub embedded_fonts: Vec<fonts::EmbeddedFont>,
}

pub fn import(source: &[u8]) -> Result<ImportResult, DocxError> {
    import_docx(source)
}

pub fn export(doc: &Document, package: &DocxPackage) -> Result<Vec<u8>, DocxError> {
    if package.can_pass_through(doc) {
        if let Some(bytes) = &package.original_bytes {
            return Ok(bytes.clone());
        }
    }
    export_docx(doc, package)
}
