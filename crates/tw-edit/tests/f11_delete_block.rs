//! Delete chart / shape / table blocks via DeleteBlock.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ChartKind, ShapeKind};

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block"),
    }
}

#[test]
fn u_f11_delete_inserted_chart() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertChart {
            after_block_id: after,
            width: 200.0,
            height: 120.0,
            kind: ChartKind::Column,
        })
        .unwrap();

    let chart_id = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .id;
    assert_eq!(session.document.sections[0].blocks.len(), 2);

    session
        .apply(Command::DeleteBlock { id: chart_id })
        .unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);
    assert!(session.document.sections[0].blocks[0].paragraph().is_some());

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 2);
    assert_eq!(
        session.document.sections[0].blocks[1]
            .shape()
            .unwrap()
            .shape
            .shape_type,
        ShapeKind::Chart
    );
}

#[test]
fn u_f11_delete_first_block_when_not_sole() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertChart {
            after_block_id: after,
            width: 200.0,
            height: 120.0,
            kind: ChartKind::Column,
        })
        .unwrap();

    // Delete the leading empty paragraph — chart becomes first, then delete chart.
    let para_id = first_block_id(&session);
    session
        .apply(Command::DeleteBlock { id: para_id })
        .unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 2);
}
