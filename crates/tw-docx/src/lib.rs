use std::collections::{HashMap, HashSet};
use thiserror::Error;
use tw_model::Document;

pub type PartName = String;

mod export;
mod import;
mod opc;
mod paragraph;
mod styles;
mod table;
mod xml_util;

pub use export::export_docx;
pub use import::import_docx;

/// Original OPC package retained for passthrough export (ADR-0008).
#[derive(Debug, Clone, Default)]
pub struct DocxPackage {
    pub parts: HashMap<PartName, Vec<u8>>,
    pub modified_parts: HashSet<PartName>,
    pub original_bytes: Option<Vec<u8>>,
}

impl DocxPackage {
    pub fn mark_modified(&mut self, part: PartName) {
        self.modified_parts.insert(part);
    }

    pub fn is_pristine(&self) -> bool {
        self.modified_parts.is_empty()
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
        }
    }
}

const MINIMAL_CONTENT_TYPES: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

#[derive(Debug, Error)]
pub enum DocxError {
    #[error("docx export not implemented")]
    ExportNotImplemented,
    #[error("word/document.xml missing from docx package")]
    MissingDocumentPart,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub struct ImportResult {
    pub document: Document,
    pub package: DocxPackage,
}

pub fn import(source: &[u8]) -> Result<ImportResult, DocxError> {
    import_docx(source)
}

pub fn export(doc: &Document, package: &DocxPackage) -> Result<Vec<u8>, DocxError> {
    if package.is_pristine() {
        if let Some(bytes) = &package.original_bytes {
            return Ok(bytes.clone());
        }
    }
    export_docx(doc, package)
}
