//! Stress: repeated SetChartData apply/undo then export round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::{Block, ChartData, ChartSeries, ShapeBlock};

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_chart_data_churn() {
    let mut session = EditSession::new();
    session.document.sections[0].blocks[0] = Block::ShapeBlock(ShapeBlock::chart(432.0, 216.0));
    let shape_id = session.document.sections[0].blocks[0]
        .shape()
        .expect("chart")
        .id;

    for i in 0..500 {
        let data = ChartData {
            categories: vec![format!("C{i}")],
            series: vec![ChartSeries {
                name: format!("S{i}"),
                values: vec![i as f64],
            }],
        };
        session
            .apply(Command::SetChartData {
                shape_id,
                chart_data: Some(data),
            })
            .unwrap();
        session.undo().unwrap();
    }

    let final_data = ChartData {
        categories: vec!["Final".into()],
        series: vec![ChartSeries {
            name: "Value".into(),
            values: vec![99.0],
        }],
    };
    session
        .apply(Command::SetChartData {
            shape_id,
            chart_data: Some(final_data.clone()),
        })
        .unwrap();

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();
    let shape = imported.document.sections[0].blocks[0]
        .shape()
        .expect("chart");
    assert_eq!(shape.chart_data.as_ref(), Some(&final_data));
}
