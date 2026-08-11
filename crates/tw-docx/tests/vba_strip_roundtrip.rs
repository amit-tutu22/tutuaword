//! VBA passthrough — macro parts survive unmodified import/export.

use std::io::Write;
use tw_docx::{export_docx, import_docx};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn minimal_docx_with_vba(document_xml: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Override PartName="/word/document.xml" ContentType="application/vnd.ms-word.document.macroEnabled.main+xml"/>
<Override PartName="/word/vbaProject.bin" ContentType="application/vnd.ms-office.vbaProject"/>
</Types>"#,
        )
        .unwrap();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("word/vbaProject.bin", options).unwrap();
        zip.write_all(b"fake-vba").unwrap();
        zip.start_file("word/vbaData.xml", options).unwrap();
        zip.write_all(b"<vba/>").unwrap();
        zip.finish().unwrap();
    }
    buf
}

#[test]
fn vba_passthrough_preserves_macro_parts_on_unmodified_export() {
    let xml = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;
    let source = minimal_docx_with_vba(xml);
    let imported = import_docx(&source).expect("import");
    assert!(
        imported.package.parts.contains_key("word/vbaProject.bin"),
        "vbaProject.bin should be retained on import"
    );
    assert!(
        tw_docx::package_has_vba_parts(&imported.package),
        "package should report VBA parts"
    );

    let exported = export_docx(&imported.document, &imported.package).expect("export");
    let cursor = std::io::Cursor::new(exported);
    let mut archive = zip::ZipArchive::new(cursor).expect("zip");
    let mut found_vba = false;
    for i in 0..archive.len() {
        let name = archive.by_index(i).unwrap().name().to_string();
        if name == "word/vbaProject.bin" {
            found_vba = true;
        }
    }
    assert!(found_vba, "exported package should still contain vbaProject.bin");
}

#[test]
fn vba_passthrough_preserves_macro_parts_after_text_edit() {
    let xml = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;
    let source = minimal_docx_with_vba(xml);
    let imported = import_docx(&source).expect("import");

    let run_id = imported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    let mut session = tw_edit::EditSession::from_document(imported.document);
    tw_edit::apply(
        &mut session.document,
        tw_edit::Command::InsertText {
            run_id,
            offset: 2,
            text: " there".into(),
        },
    )
    .expect("edit");

    let exported = export_docx(&session.document, &imported.package).expect("export");
    let cursor = std::io::Cursor::new(exported);
    let mut archive = zip::ZipArchive::new(cursor).expect("zip");
    let mut found_vba = false;
    for i in 0..archive.len() {
        let name = archive.by_index(i).unwrap().name().to_string();
        if name == "word/vbaProject.bin" {
            found_vba = true;
        }
    }
    assert!(
        found_vba,
        "vbaProject.bin should survive export after trivial text edit"
    );
}
