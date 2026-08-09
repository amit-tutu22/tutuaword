//! End-to-end integration tests for the stub-and-gap remediation plan:
//! numbered lists, find/replace, split paragraphs, DOCX round-trip, and PDF export.

use tw_core::SyncSession;
use tw_edit::numbered_list_command;
use tw_docx::{export, import, DocxPackage};
use tw_edit::{Command, DocPosition, DocRange, EditSession};
use tw_model::{Alignment, Block, CharFormat, Document, NumberingRef, ParaFormat, Revision};
use tw_pdf::{DisplayListPdfExporter, PdfExportOptions, PdfExporter};
use tw_render::DisplayListBuilder;

fn first_run(session: &SyncSession) -> tw_model::NodeId {
    session.edit.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn numbered_list_command_applies_decimal_numbering() {
    let doc = Document::with_paragraph("Item");
    let cmd = numbered_list_command(&doc).unwrap();
    let mut session = EditSession::from_document(doc);
    session.apply(cmd).unwrap();
    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(
        para.format.numbering,
        Some(NumberingRef {
            numbering_id: 2,
            level: 0,
        })
    );
}

#[test]
fn sync_session_find_replace_updates_text_and_relayouts() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "alpha beta alpha".into(),
    });
    session.apply(Command::FindReplace {
        range: DocRange {
            start: DocPosition {
                run_id,
                char_offset: 0,
            },
            end: DocPosition {
                run_id,
                char_offset: 16,
            },
        },
        find: "alpha".into(),
        replace: "gamma".into(),
        match_case: true,
        use_regex: false,
        use_wildcards: false,
    });

    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().full_text(),
        "gamma beta gamma"
    );
    let bytes = session.display_list_bytes();
    assert!(!bytes.is_empty());
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(!decoded.atlas_batch.transforms.is_empty());
}

#[test]
fn sync_session_split_paragraph_creates_two_blocks() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Line one".into(),
    });
    session.apply(Command::SplitParagraphAt {
        run_id,
        offset: 4,
    });

    assert_eq!(session.edit.document.sections[0].blocks.len(), 2);
    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().full_text(),
        "Line"
    );
    assert_eq!(
        session.edit.document.paragraph_at(0, 1).unwrap().full_text(),
        " one"
    );
}

#[test]
fn docx_export_import_preserves_track_changes_and_numbering() {
    let mut doc = Document::with_paragraph("Changed");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].revision = Some(Revision::insert("Tester"));
        para.format.numbering = Some(NumberingRef {
            numbering_id: 2,
            level: 0,
        });
    }

    let bytes = export(&doc, &DocxPackage::minimal()).unwrap();
    let imported = import(&bytes).unwrap();

    assert!(
        imported
            .package
            .parts
            .contains_key("word/numbering.xml")
    );
    let para = imported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    assert!(para.runs[0].revision.is_some());
    assert_eq!(para.format.numbering.unwrap().numbering_id, 2);
}

#[test]
fn pdf_export_after_formatting_contains_body_characters() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "ExportMe".into(),
    });
    session.apply(Command::SetCharFormat {
        run_id,
        start: 0,
        end: 8,
        format: CharFormat {
            bold: Some(true),
            ..Default::default()
        },
        merge: true,
    });

    let exporter = DisplayListPdfExporter;
    let pdf = exporter
        .export(&session.edit.document, &PdfExportOptions::default())
        .unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("(ExportMe) Tj"));
}

#[test]
fn justified_paragraph_relayouts_without_error() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "One two three four five six seven eight nine ten eleven twelve".into(),
    });
    session.apply(Command::SetParaFormatRange {
        range: DocRange {
            start: DocPosition {
                run_id,
                char_offset: 0,
            },
            end: DocPosition {
                run_id,
                char_offset: 60,
            },
        },
        format: ParaFormat {
            alignment: Some(Alignment::Justify),
            ..Default::default()
        },
        merge: true,
    });

    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().format.alignment,
        Some(Alignment::Justify)
    );
    assert!(session.page_count() >= 1);
}
