//! R0.3: rapid inserts are not silently dropped; document text matches input.

use std::time::{Duration, Instant};

use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::Command;

#[test]
fn r03_thousand_rapid_inserts_no_lost_characters() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .map(|hit| hit.run_id)
        .expect("empty document is editable");

    const COUNT: usize = 1000;
    let expected: String = "x".repeat(COUNT);

    for offset in 0..COUNT {
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: "x".into(),
            })
            .expect("command must enqueue (no silent drop)");
    }

    // Coalescing may drop intermediate DisplayListReady events for the same page.
    // R0.3 exit is document correctness, not per-request paint notifications.
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        while session.poll_event().is_some() {}
        if session.get_display_list_bytes().document_text == expected {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for all {COUNT} characters"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
