//! F11.S2 — insert native shapes with stroke/fill.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ShapeKind, ShapeStyle};

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
fn u_f11_s2_insert_rectangle_shape() {
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

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::Rectangle);
    assert_eq!(shape.shape.width, 100.0);
    assert_eq!(shape.shape.height, 60.0);
    assert_eq!(shape.style.fill, Some(0xFFD0E8FF));
    assert_eq!(shape.style.stroke, Some(0xFF000000));
    assert_eq!(
        shape.paragraphs.len(),
        1,
        "rectangle must host a body paragraph for typing"
    );

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);
}

#[test]
fn u_f11_s2_insert_line_and_ellipse() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertShape {
            after_block_id: after,
            shape_type: ShapeKind::Line,
            width: 80.0,
            height: 40.0,
            style: ShapeStyle::inserted_default(),
        })
        .unwrap();
    let line_id = session.document.sections[0].blocks[1].shape().unwrap().id;
    session
        .apply(Command::InsertShape {
            after_block_id: line_id,
            shape_type: ShapeKind::Ellipse,
            width: 90.0,
            height: 50.0,
            style: ShapeStyle::inserted_default(),
        })
        .unwrap();

    let blocks = &session.document.sections[0].blocks;
    assert!(matches!(blocks[1], Block::ShapeBlock(_)));
    assert!(matches!(blocks[2], Block::ShapeBlock(_)));
    assert_eq!(blocks[1].shape().unwrap().shape.shape_type, ShapeKind::Line);
    assert_eq!(blocks[2].shape().unwrap().shape.shape_type, ShapeKind::Ellipse);
    assert!(
        blocks[1].shape().unwrap().paragraphs.is_empty(),
        "lines are not text hosts"
    );
    assert_eq!(
        blocks[2].shape().unwrap().paragraphs.len(),
        1,
        "ellipse must host a body paragraph for typing"
    );
}
