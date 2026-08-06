use std::io::Cursor;

use zip::ZipArchive;

use crate::DocxError;

/// Returns true when the OPC package uses Office encryption (password required).
pub fn is_password_protected(source: &[u8]) -> Result<bool, DocxError> {
    let cursor = Cursor::new(source);
    let mut archive = ZipArchive::new(cursor)?;
    let mut has_document = false;
    let mut has_encrypted_package = false;
    let mut has_encryption_info = false;

    for i in 0..archive.len() {
        let name = archive.by_index(i)?.name().to_string();
        if name == "word/document.xml" {
            has_document = true;
        } else if name == "EncryptedPackage" {
            has_encrypted_package = true;
        } else if name.eq_ignore_ascii_case("encryptioninfo") {
            has_encryption_info = true;
        }
    }

    Ok(has_encrypted_package || has_encryption_info || (!has_document && looks_like_encrypted_docx(&mut archive)))
}

fn looks_like_encrypted_docx(archive: &mut ZipArchive<Cursor<&[u8]>>) -> bool {
    let mut has_content_types = false;
    let mut has_word_rels = false;
    for i in 0..archive.len() {
        let Ok(file) = archive.by_index(i) else {
            continue;
        };
        let name = file.name();
        if name == "[Content_Types].xml" {
            has_content_types = true;
        }
        if name.starts_with("word/") {
            has_word_rels = true;
        }
    }
    has_content_types && !has_word_rels
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn encrypted_docx_bytes() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buf));
            let options = SimpleFileOptions::default();
            zip.start_file("[Content_Types].xml", options).unwrap();
            zip.write_all(b"<?xml version=\"1.0\"?><Types/>").unwrap();
            zip.start_file("EncryptionInfo", options).unwrap();
            zip.write_all(b"encrypted").unwrap();
            zip.start_file("EncryptedPackage", options).unwrap();
            zip.write_all(b"encrypted").unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn detects_encrypted_package() {
        assert!(is_password_protected(&encrypted_docx_bytes()).unwrap());
    }
}
