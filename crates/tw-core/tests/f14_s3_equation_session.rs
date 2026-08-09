//! F14.S3 — session API for equation insert/edit.

use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_model::{build_display_omath_para, omml_run, RunContent};

#[test]
fn u_f14_s3_session_insert_and_fetch_office_math() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let xml = build_display_omath_para(&format!(
        "<m:oMath xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">{}</m:oMath>",
        omml_run("α+β")
    ));

    let request = session
        .insert_office_math_display(None, xml)
        .expect("insert display equation");
    assert!(matches!(
        session.wait_for_response(request, Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let latest = session.latest_office_math_run_id().expect("latest run");
    let fetched = session.office_math_xml(latest).expect("fetch xml");
    assert!(fetched.contains("α+β"));

    let updated = fetched.replace("α+β", "π");
    let set_req = session
        .set_office_math(latest, updated)
        .expect("set equation");
    assert!(matches!(
        session.wait_for_response(set_req, Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    let after = session.office_math_xml(latest).expect("after set");
    assert!(after.contains("π"));

    let doc = session.document();
    let mut found = false;
    for section in &doc.sections {
        for block in &section.blocks {
            if let Some(para) = block.paragraph() {
                for run in &para.runs {
                    if run.id == latest {
                        found = matches!(&run.content, RunContent::OfficeMath { .. });
                    }
                }
            }
        }
    }
    assert!(found);
}
