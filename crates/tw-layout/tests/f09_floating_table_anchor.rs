//! Floating table anchors must place content at the absolute offset.

use tw_edit::Command;
use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{AnchorOrigin, ImageAnchor};

#[test]
fn floating_table_anchor_places_at_absolute_offset() {
    let mut session = tw_edit::EditSession::new();
    let after = match &session.document.sections[0].blocks[0] {
        tw_model::Block::Paragraph(p) => p.id,
        other => panic!("expected paragraph, got {other:?}"),
    };
    session
        .apply(Command::InsertTable {
            after_block_id: after,
            rows: 2,
            cols: 2,
        })
        .unwrap();
    let table_id = session.document.sections[0].blocks[1].table().unwrap().id;

    session
        .apply(Command::SetShapeAnchor {
            shape_id: table_id,
            anchor: ImageAnchor {
                x: 40.0,
                y: 80.0,
                origin_x: AnchorOrigin::Column,
                origin_y: AnchorOrigin::Column,
            },
        })
        .unwrap();

    let layout = LayoutEngine::new().layout_document(&session.document);
    let placed = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::Table(t) if t.table_id == table_id => Some(t),
            _ => None,
        })
        .expect("floating table layout");
    // Default margins 72pt + Column anchor offsets.
    assert!(
        (placed.x - 112.0).abs() < 1.0,
        "x={} expected ~112",
        placed.x
    );
    assert!(
        (placed.y - 152.0).abs() < 1.0,
        "y={} expected ~152",
        placed.y
    );
}
