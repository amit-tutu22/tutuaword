//! Shape / SmartArt / chart drag move via SetShapeAnchor.

use tw_edit::{Command, EditSession};
use tw_model::{AnchorOrigin, Block, ImageAnchor, ShapeKind, TextWrap};

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
fn set_shape_anchor_converts_inline_to_floating() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertDiagram {
            after_block_id: after,
            width: 200.0,
            height: 100.0,
            kind: Default::default(),
        })
        .unwrap();

    let shape_id = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .id;
    assert!(session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .anchor
        .is_none());
    assert_eq!(
        session.document.sections[0].blocks[1].shape().unwrap().wrap,
        TextWrap::Inline
    );

    session
        .apply(Command::SetShapeAnchor {
            shape_id,
            anchor: ImageAnchor {
                x: 40.0,
                y: 20.0,
                origin_x: AnchorOrigin::Column,
                origin_y: AnchorOrigin::Column,
            },
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::Diagram);
    assert_eq!(shape.wrap, TextWrap::Square);
    let anchor = shape.anchor.expect("anchor set");
    assert_eq!(anchor.x, 40.0);
    assert_eq!(anchor.y, 20.0);

    session.undo().unwrap();
    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.wrap, TextWrap::Inline);
    assert!(shape.anchor.is_none());
}
