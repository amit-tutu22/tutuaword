use std::io::{Cursor, Write};

use tw_core::import_document_bundle;
use tw_core::ImportError;
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

fn docx_with_core_properties() -> Vec<u8> {
    let document_xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Props</w:t></w:r></w:p></w:body></w:document>"#;
    let core_xml = r#"<?xml version="1.0"?>
<cp:coreProperties xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>Quarterly Report</dc:title>
  <dc:creator>Jane Author</dc:creator>
</cp:coreProperties>"#;
    let app_xml = r#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Pages>3</Pages></Properties>"#;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("docProps/core.xml", options).unwrap();
        zip.write_all(core_xml.as_bytes()).unwrap();
        zip.start_file("docProps/app.xml", options).unwrap();
        zip.write_all(app_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options).unwrap();
        zip.write_all(b"<Relationships/>").unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn u_f01_s4_password_import_error() {
    let err = import_document_bundle(&encrypted_docx_bytes(), Some("locked.docx")).unwrap_err();
    assert!(matches!(err, ImportError::PasswordProtected));
    assert!(err.to_string().to_lowercase().contains("password"));
}

#[test]
fn docx_import_reads_core_and_app_properties() {
    let bundle = import_document_bundle(&docx_with_core_properties(), Some("props.docx")).unwrap();
    assert_eq!(
        bundle.document.properties.title.as_deref(),
        Some("Quarterly Report")
    );
    assert_eq!(
        bundle.document.properties.author.as_deref(),
        Some("Jane Author")
    );
    assert_eq!(bundle.document.properties.page_count, Some(3));
}
