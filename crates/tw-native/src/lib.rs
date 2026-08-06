use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use thiserror::Error;
use tw_model::Document;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Error)]
pub enum NativeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    format_version: String,
    created: String,
    modified: String,
    app_version: String,
    generator: String,
}

pub struct NativeFormat;

impl NativeFormat {
    pub fn export(doc: &Document) -> Result<Vec<u8>, NativeError> {
        let now = Utc::now().to_rfc3339();
        let manifest = Manifest {
            format_version: "1.0".into(),
            created: now.clone(),
            modified: now,
            app_version: "0.1.0".into(),
            generator: "tutuaword".into(),
        };

        let content = serde_json::to_string_pretty(doc)?;
        let styles = serde_json::to_string_pretty(&doc.styles)?;
        let settings = serde_json::to_string_pretty(&doc.settings)?;

        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options = SimpleFileOptions::default();

            zip.start_file("manifest.json", options)?;
            zip.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;

            zip.start_file("content.json", options)?;
            zip.write_all(content.as_bytes())?;

            zip.start_file("styles.json", options)?;
            zip.write_all(styles.as_bytes())?;

            zip.start_file("settings.json", options)?;
            zip.write_all(settings.as_bytes())?;

            zip.finish()?;
        }

        Ok(buf)
    }

    pub fn import(data: &[u8]) -> Result<Document, NativeError> {
        let cursor = std::io::Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;

        let mut content = String::new();
        archive.by_name("content.json")?.read_to_string(&mut content)?;
        let doc: Document = serde_json::from_str(&content)?;
        Ok(doc)
    }

    pub fn import_plain_text(data: &[u8]) -> Result<Document, NativeError> {
        let text = std::str::from_utf8(data).map_err(|e| {
            NativeError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;
        Ok(Document::from_plain_text(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let doc = Document::with_paragraph("Save test");
        let bytes = NativeFormat::export(&doc).unwrap();
        let loaded = NativeFormat::import(&bytes).unwrap();
        assert_eq!(
            loaded.sections[0].blocks[0].paragraph().unwrap().full_text(),
            "Save test"
        );
    }

    /// U-F01-S1-save-roundtrip-twdoc
    #[test]
    fn u_f01_s1_save_roundtrip_twdoc() {
        let doc = Document::with_paragraph("Native round-trip");
        let bytes = NativeFormat::export(&doc).unwrap();
        let loaded = NativeFormat::import(&bytes).unwrap();
        assert_eq!(
            loaded.sections[0].blocks[0].paragraph().unwrap().full_text(),
            "Native round-trip"
        );
    }
}
