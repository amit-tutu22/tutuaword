//! F13.S3 — editable chart dataset command + undo.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ChartData, ChartSeries, ShapeBlock, ShapeKind};

#[test]
fn u_f13_s3_set_chart_data_updates_shape() {
    let mut session = EditSession::new();
    session.document.sections[0].blocks[0] = Block::ShapeBlock(ShapeBlock::chart(432.0, 216.0));
    let shape_id = session.document.sections[0].blocks[0]
        .shape()
        .expect("chart shape")
        .id;

    let updated = ChartData {
        categories: vec!["A".into(), "B".into()],
        series: vec![ChartSeries {
            name: "Revenue".into(),
            values: vec![3.0, 7.0],
        }],
    };

    session
        .apply(Command::SetChartData {
            shape_id,
            chart_data: Some(updated.clone()),
        })
        .unwrap();

    let shape = session.document.sections[0].blocks[0].shape().unwrap();
    assert_eq!(shape.chart_data.as_ref(), Some(&updated));

    session.undo().unwrap();
    assert_eq!(
        session.document.sections[0].blocks[0]
            .shape()
            .unwrap()
            .chart_data
            .as_ref(),
        Some(&ChartData::sample_bar())
    );
}
