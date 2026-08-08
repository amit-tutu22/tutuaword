//! F13.S3 — insert chart placeholder with default sample data.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ChartData, ShapeKind};

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
fn u_f13_s3_insert_chart_placeholder() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);

    session
        .apply(Command::InsertChart {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::Chart);
    assert_eq!(shape.chart_data.as_ref(), Some(&ChartData::sample_bar()));

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);
}
