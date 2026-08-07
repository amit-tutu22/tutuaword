use std::io::Cursor;

use thiserror::Error;
use tw_model::Document;
use zip::ZipArchive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetectedFormat {
    #[default]
    Unknown,
    Twdoc,
    Docx,
    Odt,
    Rtf,
    Html,
    Markdown,
    PlainText,
    LegacyDoc,
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("legacy .doc (Word 97-2003) format is not supported yet")]
    LegacyDocNotSupported,
    #[error("could not detect document format")]
    UnknownFormat,
    #[error("native format error: {0}")]
    Native(#[from] tw_native::NativeError),
    #[error("docx error: {0}")]
    Docx(#[from] tw_docx::DocxError),
    #[error("odt error: {0}")]
    Odt(#[from] tw_odt::OdtError),
    #[error("rtf error: {0}")]
    Rtf(#[from] tw_rtf::RtfError),
    #[error("html error: {0}")]
    Html(#[from] tw_html::HtmlError),
    #[error("markdown error: {0}")]
    Markdown(#[from] tw_markdown::MarkdownError),
    #[error("document is password-protected")]
    PasswordProtected,
}

/// File extensions supported for import (Microsoft Word-compatible set).
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "twdoc", "docx", "doc", "rtf", "txt", "odt", "html", "htm", "md", "markdown",
];

pub fn extension_from_path(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
}

pub fn detect_format(data: &[u8], path_hint: Option<&str>) -> DetectedFormat {
    if let Some(ext) = path_hint.and_then(extension_from_path) {
        if let Some(format) = format_from_extension(&ext) {
            return format;
        }
    }

    if data.starts_with(b"PK\x03\x04") {
        return detect_zip_format(data).unwrap_or(DetectedFormat::Unknown);
    }
    if data.starts_with(b"{\\rtf") || data.starts_with(b"{\\RTF") {
        return DetectedFormat::Rtf;
    }
    if data.starts_with(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1") {
        return DetectedFormat::LegacyDoc;
    }

    let trimmed = data
        .iter()
        .copied()
        .skip_while(|b| b.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if trimmed.starts_with(b"<") {
        let lower = String::from_utf8_lossy(&trimmed[..trimmed.len().min(256)]).to_ascii_lowercase();
        if lower.contains("<html") || lower.contains("<!doctype") || lower.contains("<body") {
            return DetectedFormat::Html;
        }
    }

    if std::str::from_utf8(data).is_ok() {
        return DetectedFormat::PlainText;
    }

    DetectedFormat::Unknown
}

pub fn format_from_extension(ext: &str) -> Option<DetectedFormat> {
    match ext {
        "twdoc" => Some(DetectedFormat::Twdoc),
        "docx" => Some(DetectedFormat::Docx),
        "doc" => Some(DetectedFormat::LegacyDoc),
        "rtf" => Some(DetectedFormat::Rtf),
        "txt" => Some(DetectedFormat::PlainText),
        "odt" => Some(DetectedFormat::Odt),
        "html" | "htm" => Some(DetectedFormat::Html),
        "md" | "markdown" => Some(DetectedFormat::Markdown),
        _ => None,
    }
}

fn detect_zip_format(data: &[u8]) -> Option<DetectedFormat> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).ok()?;
    let mut has_content_json = false;
    let mut has_document_xml = false;
    let mut has_odt_content = false;
    let mut has_encrypted_package = false;
    let mut has_encryption_info = false;
    let count = archive.len();
    for i in 0..count {
        let name = archive.by_index(i).ok()?.name().to_string();
        if name == "content.json" {
            has_content_json = true;
        } else if name == "word/document.xml" {
            has_document_xml = true;
        } else if name == "content.xml" {
            has_odt_content = true;
        } else if name == "EncryptedPackage" {
            has_encrypted_package = true;
        } else if name.eq_ignore_ascii_case("encryptioninfo") {
            has_encryption_info = true;
        }
    }
    if has_content_json {
        Some(DetectedFormat::Twdoc)
    } else if has_document_xml {
        Some(DetectedFormat::Docx)
    } else if has_odt_content {
        Some(DetectedFormat::Odt)
    } else if has_encrypted_package || has_encryption_info {
        Some(DetectedFormat::Docx)
    } else {
        None
    }
}

pub fn import_document(data: &[u8], path_hint: Option<&str>) -> Result<Document, ImportError> {
    Ok(crate::bundle::import_document_bundle(data, path_hint)?.document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::Document;

    #[test]
    fn detects_legacy_doc() {
        let ole = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        assert_eq!(detect_format(&ole, None), DetectedFormat::LegacyDoc);
    }

    #[test]
    fn imports_plain_text_by_extension() {
        let doc = import_document(b"Line one\nLine two", Some("notes.txt")).unwrap();
        assert_eq!(
            doc.sections[0].blocks[0].paragraph().unwrap().full_text(),
            "Line one"
        );
        assert_eq!(
            doc.sections[0].blocks[1].paragraph().unwrap().full_text(),
            "Line two"
        );
    }

    #[test]
    fn imports_markdown_as_text() {
        let doc = import_document(b"# Title\n\nBody", Some("readme.md")).unwrap();
        assert!(Document::from_plain_text("# Title\n\nBody")
            .sections[0]
            .blocks[0]
            .paragraph()
            .unwrap()
            .full_text()
            .contains('#'));
        assert!(!doc.sections.is_empty());
    }

    #[test]
    fn detects_html_by_content() {
        let html = b"<!DOCTYPE html><html><body>Hi</body></html>";
        assert_eq!(detect_format(html, None), DetectedFormat::Html);
    }

    #[test]
    fn detects_odt_and_docx_from_zip_contents() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let mut docx = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut docx));
            let options = SimpleFileOptions::default();
            zip.start_file("word/document.xml", options).unwrap();
            zip.write_all(b"<w:document/>").unwrap();
            zip.finish().unwrap();
        }
        assert_eq!(detect_format(&docx, None), DetectedFormat::Docx);

        let mut odt = Vec::new();
        {
            let mut zip = ZipWriter::new(std::io::Cursor::new(&mut odt));
            let options = SimpleFileOptions::default();
            zip.start_file("content.xml", options).unwrap();
            zip.write_all(b"<office:document/>").unwrap();
            zip.finish().unwrap();
        }
        assert_eq!(detect_format(&odt, None), DetectedFormat::Odt);
    }

    #[test]
    fn format_from_extension_covers_phase3_types() {
        assert_eq!(format_from_extension("odt"), Some(DetectedFormat::Odt));
        assert_eq!(format_from_extension("html"), Some(DetectedFormat::Html));
        assert_eq!(format_from_extension("md"), Some(DetectedFormat::Markdown));
    }
}
