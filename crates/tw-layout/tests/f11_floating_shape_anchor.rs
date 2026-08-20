//! Floating shape anchors must place content at the absolute offset.

use tw_edit::Command;
use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{AnchorOrigin, ImageAnchor, ShapeKind, TextWrap};

#[test]
fn floating_shape_anchor_places_at_absolute_offset() {
    let mut session = tw_edit::EditSession::new();
    let after = match &session.document.sections[0].blocks[0] {
        tw_model::Block::Paragraph(p) => p.id,
        other => panic!("expected paragraph, got {other:?}"),
    };
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

    session
        .apply(Command::SetShapeAnchor {
            shape_id,
            anchor: ImageAnchor {
                x: 90.0,
                y: 140.0,
                origin_x: AnchorOrigin::Column,
                origin_y: AnchorOrigin::Column,
            },
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.wrap, TextWrap::Square);
    assert!(shape.anchor.is_some());

    let layout = LayoutEngine::new().layout_document(&session.document);
    let page = &layout.pages[0];
    let shape_layout = page
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::Shape(s) if s.shape_id == shape_id => Some(s),
            _ => None,
        })
        .expect("floating diagram shape box");
    assert_eq!(shape_layout.shape_type, ShapeKind::Diagram);
    // Default section margins are 72pt; Column origin adds the anchor offset.
    assert!(
        (shape_layout.x - 162.0).abs() < 1.0,
        "x={} expected ~162",
        shape_layout.x
    );
    assert!(
        (shape_layout.y - 212.0).abs() < 1.0,
        "y={} expected ~212",
        shape_layout.y
    );
}
