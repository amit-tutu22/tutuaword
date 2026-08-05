use std::io::{Cursor, Read, Write};

use tw_core::{export_document, import_document_bundle, DetectedFormat, FormatContext};
use tw_edit::{Command, EditSession};
use tw_model::{RevisionType, Run};
use tw_spell::SpellChecker;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;
use zip::ZipWriter;

fn minimal_docx(document_xml: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options).unwrap();
        zip.write_all(b"<Relationships/>").unwrap();
        zip.finish().unwrap();
    }
    buf
}

fn minimal_odt(content_xml: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("content.xml", options).unwrap();
        zip.write_all(content_xml.as_bytes()).unwrap();
        zip.start_file("mimetype", options).unwrap();
        zip.write_all(b"application/vnd.oasis.opendocument.text")
            .unwrap();
        zip.finish().unwrap();
    }
    buf
}

fn docx_text(bytes: &[u8]) -> String {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    xml
}

#[test]
fn docx_round_trip_preserves_text_and_unknown_parts() {
    let xml = r#"<w:document><w:body>
        <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Bold text</w:t></w:r></w:p>
    </w:body></w:document>"#;
    let source = minimal_docx(xml);
    let bundle = import_document_bundle(&source, Some("sample.docx")).unwrap();
    assert_eq!(bundle.source_format, DetectedFormat::Docx);
    assert!(bundle.docx_package.is_some());

    let ctx = FormatContext::from_bundle(bundle.clone(), Some("sample.docx".into()));
    let exported = export_document(&bundle.document, &ctx).unwrap();

    let roundtrip = import_document_bundle(&exported, Some("sample.docx")).unwrap();
    assert_eq!(
        roundtrip
            .document
            .sections[0]
            .blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Bold text"
    );
    assert!(roundtrip.docx_package.unwrap().parts.contains_key("[Content_Types].xml"));
}

#[test]
fn odt_round_trip_preserves_text() {
    let xml = r#"<office:document><office:body>
        <text:p><text:span>Hello ODT</text:span></text:p>
    </office:body></office:document>"#;
    let source = minimal_odt(xml);
    let bundle = import_document_bundle(&source, Some("sample.odt")).unwrap();
    assert_eq!(bundle.source_format, DetectedFormat::Odt);

    let ctx = FormatContext::from_bundle(bundle.clone(), Some("sample.odt".into()));
    let exported = export_document(&bundle.document, &ctx).unwrap();

    let roundtrip = import_document_bundle(&exported, Some("sample.odt")).unwrap();
    assert_eq!(
        roundtrip
            .document
            .sections[0]
            .blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Hello ODT"
    );
}

#[test]
fn markdown_import_export_docx_preserves_structure() {
    let md = b"# Heading\n\nHello **world**";
    let bundle = import_document_bundle(md, Some("readme.md")).unwrap();
    assert_eq!(bundle.source_format, DetectedFormat::Markdown);

    let mut ctx = FormatContext::from_bundle(bundle.clone(), Some("readme.md".into()));
    ctx.save_format = DetectedFormat::Docx;
    let docx_bytes = export_document(&bundle.document, &ctx).unwrap();

    let xml = docx_text(&docx_bytes);
    assert!(xml.contains("Heading") || xml.contains("Heading1"));
    assert!(xml.contains("world"));
    assert!(xml.contains("<w:b") || xml.contains("Bold"));
}

#[test]
fn spell_check_flags_ten_known_misspellings() {
    let checker = SpellChecker::english();
    let text = "Teh quikc brown fox recieved an invitaion to teh party with mispelled wrds and erors galore.";
    let issues = checker.check_text(text);
    assert!(
        issues.len() >= 10,
        "expected at least 10 misspellings, got {}",
        issues.len()
    );
}

#[test]
fn track_changes_insert_and_delete_attach_revision_metadata() {
    let mut session = EditSession::new();
    session.document.settings.track_changes_enabled = true;
    session.document.settings.author_name = "Test Author".into();

    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello".into(),
        })
        .unwrap();

    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    let insert_rev = para.runs[0].revision.as_ref().unwrap();
    assert_eq!(insert_rev.revision_type, RevisionType::Insert);
    assert_eq!(insert_rev.author, "Test Author");

    session
        .apply(Command::DeleteRange {
            run_id: para.runs[0].id,
            start: 1,
            end: 4,
        })
        .unwrap();

    let para = session.document.sections[0].blocks[0].paragraph().unwrap();
    let marked: Vec<&Run> = para
        .runs
        .iter()
        .filter(|r| r.revision.as_ref().is_some_and(|rev| rev.revision_type == RevisionType::Delete))
        .collect();
    assert!(!marked.is_empty());
    assert_eq!(marked[0].text(), "ell");
}

#[test]
fn html_round_trip_via_core_bundle() {
    let html = br#"<!DOCTYPE html><html><body><h1>Title</h1><p>Body</p></body></html>"#;
    let bundle = import_document_bundle(html, Some("page.html")).unwrap();
    assert_eq!(bundle.source_format, DetectedFormat::Html);

    let ctx = FormatContext::from_bundle(bundle.clone(), Some("page.html".into()));
    let exported = export_document(&bundle.document, &ctx).unwrap();
    let roundtrip = import_document_bundle(&exported, Some("page.html")).unwrap();
    assert!(roundtrip
        .document
        .sections[0]
        .blocks[0]
        .paragraph()
        .unwrap()
        .full_text()
        .contains("Title"));
}

#[test]
fn save_as_docx_without_source_package() {
    let bundle = import_document_bundle(b"# Title\n\nBody", Some("readme.md")).unwrap();
    let mut ctx = FormatContext::from_bundle(bundle.clone(), None);
    ctx.save_format = DetectedFormat::Docx;
    let docx = export_document(&bundle.document, &ctx).unwrap();
    assert!(docx.starts_with(b"PK"));
    let reimported = import_document_bundle(&docx, Some("out.docx")).unwrap();
    assert_eq!(reimported.source_format, DetectedFormat::Docx);
}

#[test]
fn format_context_save_format_follows_path_hint() {
    let bundle = import_document_bundle(b"Hello", Some("notes.txt")).unwrap();
    let ctx = FormatContext::from_bundle(bundle, Some("export.odt".into()));
    assert_eq!(ctx.save_format, DetectedFormat::Odt);
}

#[test]
fn markdown_to_html_export_chain() {
    let bundle = import_document_bundle(b"# Heading\n\n**Bold** body", Some("doc.md")).unwrap();
    let mut ctx = FormatContext::from_bundle(bundle.clone(), Some("doc.md".into()));
    ctx.save_format = DetectedFormat::Html;
    let html_bytes = export_document(&bundle.document, &ctx).unwrap();
    let html = String::from_utf8(html_bytes).unwrap();
    assert!(html.contains("Heading"));
    assert!(html.contains("Bold") || html.contains("<strong>"));
}
