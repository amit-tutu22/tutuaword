//! Session helpers for chart data get / latest id (F13.S3 UI bridge).

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_model::{ChartData, ChartSeries};

#[test]
fn u_f13_s3_session_chart_data_round_trip() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let request = session
        .insert_chart_with_kind(tw_model::ChartKind::Column)
        .expect("enqueue insert");
    assert!(matches!(
        session.wait_for_response(request, Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let chart_id = session.latest_chart_id().expect("latest chart");
    let json = session.chart_data_json(chart_id).expect("chart json");
    let parsed: ChartData = serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed, ChartData::sample(tw_model::ChartKind::Column));

    let updated = ChartData {
        kind: tw_model::ChartKind::Column,
        categories: vec!["A".into(), "B".into()],
        series: vec![ChartSeries {
            name: "Revenue".into(),
            values: vec![1.0, 2.0],
        }],
    };
    let set_id = session
        .set_chart_data(chart_id, Some(updated.clone()))
        .expect("enqueue set");
    assert!(matches!(
        session.wait_for_response(set_id, Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let after = session.chart_data_json(chart_id).expect("updated json");
    let parsed_after: ChartData = serde_json::from_str(&after).expect("parse after");
    assert_eq!(parsed_after, updated);
}
