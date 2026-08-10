//! Office Open XML password protection (F01.S4 detect, F22.S1 decrypt, F22.S2 encrypt).

use std::io::{Cursor, Write};

use ms_offcrypto_writer::Ecma376AgileWriter;
use office_crypto::DecryptError;
use rand::rngs::StdRng;
use rand::SeedableRng;
use zip::ZipArchive;

use crate::DocxError;

const OLE_MAGIC: &[u8] = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1";

/// Returns true when the package uses Office encryption (password required).
pub fn is_password_protected(source: &[u8]) -> Result<bool, DocxError> {
    if source.starts_with(OLE_MAGIC) {
        // Encrypted OOXML is a CFB/OLE container with EncryptionInfo / EncryptedPackage.
        // Stream names are stored as UTF-16LE in the CFB directory.
        return Ok(contains_ole_stream_name(source, "EncryptionInfo")
            || contains_ole_stream_name(source, "EncryptedPackage"));
    }

    if !source.starts_with(b"PK\x03\x04") {
        return Ok(false);
    }

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

    Ok(has_encrypted_package
        || has_encryption_info
        || (!has_document && looks_like_encrypted_docx(&mut archive)))
}

/// Decrypt a password-protected Office package to raw OOXML ZIP bytes (F22.S1).
pub fn decrypt_with_password(source: &[u8], password: &str) -> Result<Vec<u8>, DocxError> {
    if !is_password_protected(source)? {
        return Ok(source.to_vec());
    }
    // office-crypto can panic on malformed/wrong-password Agile payloads
    // (unsigned underflow in segment sizing). Isolate that from our callers.
    let decrypted = match std::panic::catch_unwind(|| {
        office_crypto::decrypt_from_bytes(source.to_vec(), password)
    }) {
        Ok(result) => result,
        Err(_) => return Err(DocxError::IncorrectPassword),
    };
    match decrypted {
        Ok(bytes) => {
            // Standard encryption may "succeed" with a wrong password and yield
            // garbage; require a ZIP/OOXML signature before accepting.
            if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
                Ok(bytes)
            } else {
                Err(DocxError::IncorrectPassword)
            }
        }
        Err(DecryptError::NotEncrypted) => Ok(source.to_vec()),
        Err(DecryptError::Unimplemented(msg)) => Err(DocxError::DecryptUnsupported(msg)),
        Err(DecryptError::IoError(err)) => Err(DocxError::Io(err)),
        // office-crypto does not expose a dedicated wrong-password variant.
        Err(DecryptError::InvalidStructure)
        | Err(DecryptError::InvalidHeader)
        | Err(DecryptError::Unknown) => Err(DocxError::IncorrectPassword),
    }
}

/// Minimum plaintext OOXML size before Agile encrypt.
///
/// `office-crypto` 0.3 panics (unsigned underflow) when decrypting Agile packages
/// whose decrypted payload is smaller than one 4096-byte segment. Padding keeps
/// our encrypt→decrypt round-trip reliable for short documents.
const MIN_OOXML_ENCRYPT_SIZE: usize = 8192;

/// Encrypt plaintext OOXML ZIP bytes into a Word-compatible Agile package (F22.S2).
pub fn encrypt_with_password(ooxml: &[u8], password: &str) -> Result<Vec<u8>, DocxError> {
    if password.is_empty() {
        return Err(DocxError::EncryptFailed("password must not be empty".into()));
    }
    if !ooxml.starts_with(b"PK\x03\x04") && !ooxml.starts_with(b"PK\x05\x06") {
        return Err(DocxError::EncryptFailed(
            "plaintext package must be a ZIP/OOXML document".into(),
        ));
    }

    let plaintext = ensure_min_ooxml_size(ooxml, MIN_OOXML_ENCRYPT_SIZE)?;
    let mut rng = StdRng::from_os_rng();
    let mut writer = Ecma376AgileWriter::create(&mut rng, password, Cursor::new(Vec::new()))
        .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
    writer
        .write_all(&plaintext)
        .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
    let cursor = writer
        .into_inner()
        .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
    let encrypted = cursor.into_inner();
    if !is_password_protected(&encrypted)? {
        return Err(DocxError::EncryptFailed(
            "encrypted output missing Office encryption markers".into(),
        ));
    }
    Ok(encrypted)
}

fn ensure_min_ooxml_size(ooxml: &[u8], min: usize) -> Result<Vec<u8>, DocxError> {
    if ooxml.len() >= min {
        return Ok(ooxml.to_vec());
    }

    use std::io::Read;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let mut archive = ZipArchive::new(Cursor::new(ooxml))?;
    let mut entries = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        entries.push((name, data));
    }

    // Store padding uncompressed — Deflate collapses zeros and would leave the
    // ZIP under office-crypto's 4KiB Agile decrypt segment threshold.
    let pad_len = min.saturating_sub(ooxml.len()).max(1024);
    let mut out = Vec::with_capacity(min + 256);
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut out));
        let options = SimpleFileOptions::default();
        for (name, data) in entries {
            writer
                .start_file(name, options)
                .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
            writer
                .write_all(&data)
                .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
        }
        let pad_options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer
            .start_file("tutuaword/encryption-padding.bin", pad_options)
            .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
        writer
            .write_all(&vec![0u8; pad_len])
            .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
        writer
            .finish()
            .map_err(|err| DocxError::EncryptFailed(err.to_string()))?;
    }
    if out.len() < min {
        return Err(DocxError::EncryptFailed(format!(
            "failed to pad OOXML package to {min} bytes (got {})",
            out.len()
        )));
    }
    Ok(out)
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

fn contains_ascii_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|window| {
        window
            .iter()
            .zip(needle.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
    })
}

fn contains_ole_stream_name(haystack: &[u8], name: &str) -> bool {
    if contains_ascii_ci(haystack, name.as_bytes()) {
        return true;
    }
    let mut utf16 = Vec::with_capacity(name.len() * 2);
    for ch in name.encode_utf16() {
        utf16.extend_from_slice(&ch.to_le_bytes());
    }
    haystack
        .windows(utf16.len())
        .any(|window| window.eq_ignore_ascii_case(&utf16))
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

    #[test]
    fn u_f22_s1_zip_fixture_decrypt_yields_incorrect_password() {
        let err = decrypt_with_password(&encrypted_docx_bytes(), "guess").unwrap_err();
        assert!(matches!(
            err,
            DocxError::IncorrectPassword | DocxError::DecryptUnsupported(_)
        ));
    }

    #[test]
    fn detects_ole_encryption_markers() {
        let mut bytes = OLE_MAGIC.to_vec();
        bytes.extend_from_slice(b"....EncryptionInfo....");
        assert!(is_password_protected(&bytes).unwrap());
    }

    #[test]
    fn detects_ole_utf16_stream_names() {
        let mut bytes = OLE_MAGIC.to_vec();
        for ch in "EncryptionInfo".encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        assert!(is_password_protected(&bytes).unwrap());
    }
}
