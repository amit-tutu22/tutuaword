//! EnsureShapeText — Add Text on shapes that lack body paragraphs.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ShapeBlock, ShapeKind, ShapeStyle};

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
fn u_f11_ensure_shape_text_seeds_empty_rectangle() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    // Legacy / imported rectangle with no body paragraph.
    let mut shape = ShapeBlock::new(
        ShapeKind::Rectangle,
        120.0,
        60.0,
        ShapeStyle::inserted_default(),
    );
    shape.paragraphs.clear();
    let shape_id = shape.id;
    session.document.sections[0]
        .blocks
        .insert(1, Block::ShapeBlock(shape));
    let _ = after;

    assert!(session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .paragraphs
        .is_empty());

    let result = session
        .apply(Command::EnsureShapeText { shape_id })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.paragraphs.len(), 1);
    assert_eq!(shape.paragraphs[0].runs.len(), 1);
    assert_eq!(
        result.seed_run_id,
        Some(shape.paragraphs[0].runs[0].id),
        "EnsureShapeText must return the body run so the UI can place a caret"
    );

    // Idempotent.
    session
        .apply(Command::EnsureShapeText { shape_id })
        .unwrap();
    assert_eq!(
        session.document.sections[0].blocks[1]
            .shape()
            .unwrap()
            .paragraphs
            .len(),
        1
    );
}

#[test]
fn u_f11_ensure_shape_text_rejects_chart() {
    let mut session = EditSession::new();
    session.document.sections[0].blocks[0] =
        Block::ShapeBlock(ShapeBlock::chart(200.0, 120.0));
    let shape_id = session.document.sections[0].blocks[0]
        .shape()
        .unwrap()
        .id;

    assert!(session
        .apply(Command::EnsureShapeText { shape_id })
        .is_err());
}

#[test]
fn u_f11_ensure_shape_text_seeds_diagram_nodes() {
    let mut session = EditSession::new();
    let mut shape = ShapeBlock::diagram(200.0, 120.0);
    shape.paragraphs.clear();
    let shape_id = shape.id;
    session.document.sections[0].blocks[0] = Block::ShapeBlock(shape);

    session
        .apply(Command::EnsureShapeText { shape_id })
        .unwrap();
    assert_eq!(
        session.document.sections[0].blocks[0]
            .shape()
            .unwrap()
            .paragraphs
            .len(),
        3
    );
}

#[test]
fn u_f11_type_into_rectangle_body() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertShape {
            after_block_id: after,
            shape_type: ShapeKind::Rectangle,
            width: 100.0,
            height: 60.0,
            style: ShapeStyle::inserted_default(),
        })
        .unwrap();

    let run_id = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello".into(),
        })
        .unwrap();

    let text: String = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs
        .iter()
        .map(|r| r.text())
        .collect();
    assert_eq!(text, "Hello");
}
