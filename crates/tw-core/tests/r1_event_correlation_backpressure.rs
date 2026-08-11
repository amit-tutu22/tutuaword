//! P0-2: coalescing the event queue must never lose an edit's correlation id.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use tw_core::{BridgeEvent, Session, WaitOutcome, EVENT_CHANNEL_CAPACITY, STARTUP_REQUEST_ID};
use tw_edit::Command;

/// Several times the event channel depth, so the publisher is forced to coalesce.
const EDIT_COUNT: usize = 4 * EVENT_CHANNEL_CAPACITY;

#[test]
fn saturated_event_channel_still_completes_every_request() {
    let session = Session::new();
    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(5)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
            if request_id == STARTUP_REQUEST_ID
    ));

    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .expect("empty document is editable")
        .run_id;

    // Enqueue without draining so the bounded event channel overflows into the
    // publisher's backlog while every edit lands on the same page.
    let mut awaited: HashSet<u64> = HashSet::new();
    for offset in 0..EDIT_COUNT {
        let request_id = session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: "x".into(),
            })
            .expect("command must enqueue");
        awaited.insert(request_id);
    }

    let deadline = Instant::now() + Duration::from_secs(60);
    while !awaited.is_empty() {
        match session.poll_event() {
            Some(event) => {
                awaited.remove(&event.request_id());
            }
            None => {
                assert!(
                    Instant::now() < deadline,
                    "{} of {EDIT_COUNT} request ids never received a completion \
                     (lowest missing: {:?})",
                    awaited.len(),
                    awaited.iter().min()
                );
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }

    assert_eq!(
        session.get_display_list_bytes().document_text,
        "x".repeat(EDIT_COUNT)
    );
}

#[test]
fn coalescing_still_collapses_redundant_repaints() {
    let session = Session::new();
    session.wait_for_startup(Duration::from_secs(5));

    let run_id = session
        .hit_test(0, 72.0, 83.0)
        .expect("empty document is editable")
        .run_id;

    let mut request_ids = Vec::new();
    for offset in 0..EDIT_COUNT {
        request_ids.push(
            session
                .apply(Command::InsertText {
                    run_id,
                    offset,
                    text: "y".into(),
                })
                .expect("command must enqueue"),
        );
    }

    let last = *request_ids.last().expect("at least one edit");
    assert!(matches!(
        session.wait_for_response(last, Duration::from_secs(60)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));

    // Only one full repaint per page survives coalescing; the rest arrive as
    // correlation-only completions carrying the latest version.
    let mut versions = HashSet::new();
    while let Some(BridgeEvent::DisplayListReady { version, .. }) = session.poll_event() {
        versions.insert(version);
    }
    assert!(
        versions.len() < EDIT_COUNT,
        "expected repaint coalescing, saw {} distinct versions",
        versions.len()
    );
}
