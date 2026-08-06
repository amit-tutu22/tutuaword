//! F01.S1 document lifecycle integration tests.

use std::io::{Cursor, Write};

use tw_core::{export_document, import_document_bundle, DetectedFormat, FormatContext, Session, SyncSession};
use tw_edit::{Command, EditSession};
use tw_render::DisplayListBuilder;
use zip::write::SimpleFileOptions;
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
        zip.finish().unwrap();
    }
    buf
}

fn first_run(session: &SyncSession) -> tw_model::NodeId {
    session.edit.document.paragraph_at(0, 0).unwrap().runs[0].id
}

/// I-F01-S1-open-docx: open sample DOCX → non-empty display list.
#[test]
fn i_f01_s1_open_docx_produces_display_list() {
    let corpus = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tw-docx/tests/corpus/simple_paragraph.docx"
    );
    let data = std::fs::read(corpus).expect("corpus docx fixture");
    let bundle = import_document_bundle(&data, Some("simple_paragraph.docx")).unwrap();

    let mut session = SyncSession::new();
    session.edit = EditSession::from_document(bundle.document);
    session.relayout();

    let bytes = session.display_list_bytes();
    assert!(!bytes.is_empty());
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(decoded.page_width > 0.0);
    assert!(decoded.page_height > 0.0);
}

/// I-F01-S1-save-as-docx: edit → save as DOCX → reopen text matches.
#[test]
fn i_f01_s1_save_as_docx_roundtrip() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Saved via DOCX".into(),
    });

    let mut ctx = FormatContext::new_document();
    ctx.save_format = DetectedFormat::Docx;
    let exported = export_document(&session.edit.document, &ctx).unwrap();

    let roundtrip = import_document_bundle(&exported, Some("saved.docx")).unwrap();
    assert_eq!(
        roundtrip
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .full_text(),
        "Saved via DOCX"
    );
}

#[test]
fn i_f01_s1_new_document_resets_async_session() {
    let session = Session::new();
    assert!(session.wait_for_event(5_000).is_some());

    let run_id = session
        .get_display_list_bytes()
        .pages
        .first()
        .and_then(|_| {
            session.hit_test(0, 72.0, 83.0)
                .map(|hit| hit.run_id)
        })
        .expect("empty document is editable");

    assert!(session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Draft".into(),
    }));
    assert!(session.wait_for_event(5_000).is_some());
    assert!(session.get_display_list_bytes().document_text.contains("Draft"));

    assert!(session.new_document());
    assert!(session.wait_for_event(5_000).is_some());
    assert_eq!(session.get_display_list_bytes().document_text.trim(), "");
}

#[test]
fn i_f01_s1_open_minimal_docx_via_bundle() {
    let xml = r#"<w:document><w:body>
        <w:p><w:r><w:t>Open me</w:t></w:r></w:p>
    </w:body></w:document>"#;
    let data = minimal_docx(xml);
    let bundle = import_document_bundle(&data, Some("sample.docx")).unwrap();
    assert_eq!(
        bundle.document.paragraph_at(0, 0).unwrap().full_text(),
        "Open me"
    );
}
