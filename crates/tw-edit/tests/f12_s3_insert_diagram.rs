//! F12.S3 — insert read-only SmartArt diagram placeholder.

use tw_edit::{
    insert_diagram_command_with_kind_for_caret, Command, EditSession,
};
use tw_model::{Block, ShapeKind};

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

#[test]
fn u_f12_s3_insert_diagram_placeholder() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertDiagram {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
            kind: Default::default(),
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::Diagram);
    assert_eq!(shape.shape.width, 432.0);
    assert_eq!(shape.shape.height, 216.0);
    assert!(shape.preview_image.is_none());
    assert_eq!(
        shape.paragraphs.len(),
        3,
        "process SmartArt must seed editable node paragraphs"
    );

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);
}

#[test]
fn u_f12_s3_insert_diagram_at_caret_not_document_end() {
    let mut session = EditSession::new();
    let first_para = first_block_id(&session);
    session
        .apply(Command::InsertParagraph { after_id: first_para })
        .unwrap();
    session
        .apply(Command::InsertParagraph {
            after_id: session.document.paragraph_at(0, 1).unwrap().id,
        })
        .unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 3);

    let caret_run = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    let cmd = insert_diagram_command_with_kind_for_caret(
        &session.document,
        Some(caret_run),
        Default::default(),
    )
    .unwrap();
    session.apply(cmd).unwrap();

    let blocks = &session.document.sections[0].blocks;
    assert_eq!(blocks.len(), 4);
    assert!(
        blocks[1].shape().is_some(),
        "diagram must insert after the caret block, not after the last paragraph"
    );
    assert_eq!(
        blocks[1].shape().unwrap().shape.shape_type,
        ShapeKind::Diagram
    );
    assert!(blocks[2].paragraph().is_some());
    assert!(blocks[3].paragraph().is_some());
}
