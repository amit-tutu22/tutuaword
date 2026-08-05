use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};

use thiserror::Error;
use tw_model::Document;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub type PartName = String;

#[derive(Debug, Clone, Default)]
pub struct OdtPackage {
    pub parts: HashMap<PartName, Vec<u8>>,
    pub modified_parts: HashSet<PartName>,
    pub original_bytes: Option<Vec<u8>>,
}

impl OdtPackage {
    pub fn mark_modified(&mut self, part: PartName) {
        self.modified_parts.insert(part);
    }

    /// Empty package for exporting a document that was not opened from ODT.
    pub fn minimal() -> Self {
        let mut parts = HashMap::new();
        parts.insert("mimetype".into(), b"application/vnd.oasis.opendocument.text".to_vec());
        Self {
            parts,
            modified_parts: HashSet::new(),
            original_bytes: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum OdtError {
    #[error("odt export not implemented")]
    ExportNotImplemented,
    #[error("content.xml missing from odt package")]
    MissingContentPart,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

mod import;

pub struct ImportResult {
    pub document: Document,
    pub package: OdtPackage,
}

pub fn import(source: &[u8]) -> Result<ImportResult, OdtError> {
    import::import_odt(source)
}

pub fn export(doc: &Document, package: &OdtPackage) -> Result<Vec<u8>, OdtError> {
    if package.modified_parts.is_empty() {
        if let Some(bytes) = &package.original_bytes {
            return Ok(bytes.clone());
        }
    }
    let mut pkg = package.clone();
    let content_xml = serialize_content_xml(doc);
    pkg.parts
        .insert("content.xml".into(), content_xml.into_bytes());
    pkg.mark_modified("content.xml".into());
    repack(&pkg)
}

fn serialize_content_xml(doc: &Document) -> String {
    let mut paragraphs = String::new();
    if let Some(section) = doc.sections.first() {
        for block in &section.blocks {
            if let tw_model::Block::Paragraph(para) = block {
                paragraphs.push_str("<text:p>");
                for run in &para.runs {
                    let mut span = String::from("<text:span>");
                    if run.format.bold == Some(true) {
                        span = String::from(r#"<text:span text:style-name="Bold">"#);
                    }
                    span.push_str(&escape_xml(run.text()));
                    span.push_str("</text:span>");
                    paragraphs.push_str(&span);
                }
                paragraphs.push_str("</text:p>");
            }
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
  <office:body><office:text>{paragraphs}</office:text></office:body>
</office:document-content>"#
    )
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn repack(package: &OdtPackage) -> Result<Vec<u8>, OdtError> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let mut names: Vec<_> = package.parts.keys().cloned().collect();
        names.sort();
        for name in names {
            let data = package.parts.get(&name).expect("part");
            zip.start_file(&name, options)?;
            zip.write_all(data)?;
        }
        zip.finish()?;
    }
    Ok(buf)
}
