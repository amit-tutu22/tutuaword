//! Shared helpers for F08 DOCX export tests.

#![allow(dead_code)]

use std::io::Read;

pub fn zip_read_part(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    Some(buf)
}

pub fn document_xml(bytes: &[u8]) -> String {
    let part = zip_read_part(bytes, "word/document.xml").expect("document.xml");
    std::str::from_utf8(&part).expect("utf8").to_string()
}

pub fn settings_xml(bytes: &[u8]) -> String {
    let part = zip_read_part(bytes, "word/settings.xml").expect("settings.xml");
    std::str::from_utf8(&part).expect("utf8").to_string()
}
