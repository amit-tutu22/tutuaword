use tw_core::{document_plain_text, snapshot_from_pages, SnapshotBuffer, SinglePageSnapshot, SyncSession};
use tw_edit::Command;
use tw_layout::LayoutEngine;
use tw_model::{Block, NumberingRef};
use tw_render::DisplayListBuilder;

#[test]
fn sync_session_heading1_produces_display_list_bytes() {
    let mut session = SyncSession::new();
    let para = session.edit.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    let run_id = para.runs[0].id;
    let para_id = para.id;

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Heading text".into(),
    });
    session.apply(Command::ApplyParagraphStyle {
        paragraph_id: para_id,
        style_name: "Heading 1".into(),
    });

    let bytes = session.display_list_bytes();
    assert!(!bytes.is_empty());
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(!decoded.atlas_batch.transforms.is_empty());
}

#[test]
fn sync_session_insert_table_updates_layout() {
    let mut session = SyncSession::new();
    let block_id = session.edit.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    session.apply(Command::InsertTable {
        after_block_id: block_id,
        rows: 3,
        cols: 3,
    });

    assert_eq!(session.edit.document.sections[0].blocks.len(), 2);
    let bytes = session.display_list_bytes();
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(!decoded.path_batch.points.is_empty());
}

#[test]
fn sync_session_long_document_reports_multiple_pages() {
    let mut session = SyncSession::new();
    let block_id = session.edit.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    for i in 0..60 {
        session.apply(Command::InsertParagraph { after_id: block_id });
        let para_id = session.edit.document.sections[0].blocks[i as usize + 1]
            .paragraph()
            .unwrap()
            .id;
        let run_id = session.edit.document.sections[0].blocks[i as usize + 1]
            .paragraph()
            .unwrap()
            .runs[0]
            .id;
        session.apply(Command::InsertText {
            run_id,
            offset: 0,
            text: format!("Paragraph {i} with enough text to fill space."),
        });
        let _ = para_id;
    }

    session.relayout();
    assert!(session.page_count() > 1);
}

#[test]
fn sync_session_bullet_list_and_export_round_trip() {
    let mut session = SyncSession::new();
    let para_id = session.edit.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    session.apply(Command::SetNumbering {
        paragraph_id: para_id,
        numbering: Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        }),
    });

    let exported = session.export();
    let imported = SyncSession::import(&exported);
    let para = imported.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.format.numbering.unwrap().numbering_id, 1);
}

#[test]
fn document_plain_text_joins_paragraphs() {
    let mut session = SyncSession::new();
    let run_id = session.edit.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Hello".into(),
    });

    let text = document_plain_text(&session.edit.document);
    assert_eq!(text, "Hello");
}

#[test]
fn snapshot_buffer_stores_multiple_pages_and_switches_index() {
    let buffer = SnapshotBuffer::new();
    let pages = vec![
        SinglePageSnapshot {
            bytes: vec![1, 2, 3],
            page_width: 612.0,
            page_height: 792.0,
        },
        SinglePageSnapshot {
            bytes: vec![4, 5, 6],
            page_width: 612.0,
            page_height: 792.0,
        },
    ];

    buffer.publish(snapshot_from_pages(
        pages,
        0,
        10,
        "page one\npage two".into(),
    ));

    let snap0 = buffer.read();
    assert_eq!(snap0.page_count, 2);
    assert_eq!(snap0.page_index, 0);
    assert_eq!(snap0.bytes, vec![1, 2, 3]);

    buffer.set_current_page(1);
    let snap1 = buffer.read();
    assert_eq!(snap1.page_index, 1);
    assert_eq!(snap1.bytes, vec![4, 5, 6]);
}

#[test]
fn layout_engine_page_count_matches_document_layout() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..80 {
        doc.sections[0].blocks.push(Block::Paragraph(tw_model::Paragraph::with_text(
            format!("Line {i} with content to paginate."),
        )));
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    assert_eq!(engine.page_count(), layout.pages.len());
    assert!(engine.page_count() > 1);
}
