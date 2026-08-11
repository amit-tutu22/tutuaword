//! R3.1: the inline executor runs the engine with no worker thread.
//!
//! Everything here is the web execution model exercised on a native host: an
//! inline session spawns nothing, sleeps nowhere, and only makes progress while
//! the host drives it. Tests that could in principle hang are run on a helper
//! thread behind a watchdog so a regression fails the suite instead of wedging CI.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, BACKGROUND_REQUEST_ID, STARTUP_REQUEST_ID};
use tw_edit::Command;
use tw_model::{Block, Document, Paragraph};
use tw_native::NativeFormat;

/// Runs `body` on a helper thread and fails the test if it has not finished
/// within `limit`. A deadlock in the inline path then surfaces as a failure.
fn with_watchdog<T, F>(limit: Duration, body: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let value = body();
        let _ = tx.send(());
        value
    });
    match rx.recv_timeout(limit) {
        // Disconnected means the body panicked; join re-raises it with its message.
        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => match handle.join() {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        },
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("inline session did not finish within {limit:?} — it blocked")
        }
    }
}

fn first_run_id(session: &Session) -> tw_model::NodeId {
    session.document().sections[0].blocks[0]
        .paragraph()
        .expect("paragraph")
        .runs[0]
        .id
}

/// Continuous prose so an edit on page 0 shifts every later page and the reflow
/// cannot converge inside the synchronous 3-page window.
fn flowing_document(paragraphs: usize) -> Document {
    let mut doc = Document::new();
    let filler = "The quick brown fox jumps over the lazy dog while the sleepy cat \
                  watches from a sunny windowsill and the kettle boils.";
    doc.sections[0].blocks = (0..paragraphs)
        .map(|idx| Block::Paragraph(Paragraph::with_text(format!("{idx}. {filler}"))))
        .collect();
    doc
}

#[test]
fn r3_inline_session_reports_that_it_must_be_driven() {
    let session = Session::new_inline();
    assert!(session.requires_drive());
}

/// The startup layout happens in the constructor, so a driven host has a
/// document to render before it has pumped anything.
#[test]
fn r3_inline_startup_event_is_correlated() {
    with_watchdog(Duration::from_secs(30), || {
        let session = Session::new_inline();
        assert!(matches!(
            session.wait_for_startup(Duration::from_secs(5)),
            WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
                if request_id == STARTUP_REQUEST_ID
        ));
    });
}

/// The core inline contract: an edit applies and produces a completion event
/// carrying its own request id, with no thread anywhere in the session.
#[test]
fn r3_inline_edit_completes_with_correlated_event() {
    with_watchdog(Duration::from_secs(60), || {
        let session = Session::new_inline();
        assert!(matches!(
            session.wait_for_startup(Duration::from_secs(5)),
            WaitOutcome::Matched(_)
        ));

        let run_id = first_run_id(&session);
        let request_id = session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Inline".into(),
            })
            .expect("edit enqueued");

        match session.wait_for_response(request_id, Duration::from_secs(5)) {
            WaitOutcome::Matched(BridgeEvent::DisplayListReady {
                request_id: got, ..
            }) => assert_eq!(got, request_id),
            other => panic!("expected a correlated repaint, got {other:?}"),
        }
        assert!(session
            .get_display_list_bytes()
            .document_text
            .starts_with("Inline"));
    });
}

/// `submit` only queues. Without a driver the engine must not have run — this is
/// what makes the inline path safe to call from a single-threaded host.
#[test]
fn r3_inline_command_waits_for_the_host_to_drive() {
    with_watchdog(Duration::from_secs(30), || {
        let session = Session::new_inline();
        let run_id = first_run_id(&session);
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Deferred".into(),
            })
            .expect("edit enqueued");

        assert!(
            !session
                .get_display_list_bytes()
                .document_text
                .contains("Deferred"),
            "an inline command must not execute before the host drives the engine"
        );

        let driven = session.drive();
        assert!(driven > 0, "drive should have executed the queued command");
        assert!(session
            .get_display_list_bytes()
            .document_text
            .contains("Deferred"));
        assert_eq!(session.drive(), 0, "an idle inline engine reports no work");
    });
}

/// `pump_events` is the documented host driver: one call runs the queued work
/// and delivers the resulting events to the observer.
#[test]
fn r3_inline_pump_events_drives_and_delivers() {
    with_watchdog(Duration::from_secs(60), || {
        let session = Session::new_inline();
        let seen: Arc<Mutex<Vec<BridgeEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        session.set_event_observer(Arc::new(move |event| {
            sink.lock().expect("sink").push(event);
        }));

        let run_id = first_run_id(&session);
        let request_id = session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Pumped".into(),
            })
            .expect("edit enqueued");

        // A single pump is enough for a one-page edit; loop only so a slower
        // machine cannot make this flaky.
        for _ in 0..16 {
            session.pump_events();
            let delivered = seen
                .lock()
                .expect("sink")
                .iter()
                .any(|event| event.request_id() == request_id);
            if delivered {
                return;
            }
        }
        panic!("pump_events never delivered the correlated completion");
    });
}

/// A correlated wait on an inline session has no other thread to wait for, so it
/// must drive the queue. Before R3.1 this deadlocked until the timeout expired.
#[test]
fn r3_inline_correlated_wait_drives_instead_of_sleeping() {
    with_watchdog(Duration::from_secs(60), || {
        let session = Session::new_inline();
        let run_id = first_run_id(&session);
        let first = session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "A".into(),
            })
            .expect("edit enqueued");
        let second = session
            .apply(Command::InsertText {
                run_id,
                offset: 1,
                text: "B".into(),
            })
            .expect("edit enqueued");

        // Waiting on the *second* id forces the wait to drive past the first.
        assert!(matches!(
            session.wait_for_response(second, Duration::from_secs(5)),
            WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
        ));
        assert!(matches!(
            session.wait_for_response(first, Duration::from_secs(5)),
            WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
        ));
        assert!(session
            .get_display_list_bytes()
            .document_text
            .starts_with("AB"));
    });
}

/// An unmatched wait must report `Timeout` promptly once the engine is idle
/// rather than spinning or sleeping to the deadline.
#[test]
fn r3_inline_wait_for_unknown_id_returns_timeout_when_idle() {
    with_watchdog(Duration::from_secs(30), || {
        let session = Session::new_inline();
        assert_eq!(
            session.wait_for_response(9_999_999, Duration::from_secs(300)),
            WaitOutcome::Timeout
        );
    });
}

/// Coalescing may drop redundant repaints but never a correlation id — the same
/// contract the threaded path has, now on the inline queue and ack retention.
#[test]
fn r3_inline_batched_edits_all_receive_completions() {
    with_watchdog(Duration::from_secs(120), || {
        let session = Session::new_inline();
        let run_id = first_run_id(&session);
        let ids: Vec<u64> = (0..200)
            .map(|i| {
                session
                    .apply(Command::InsertText {
                        run_id,
                        offset: i,
                        text: "x".into(),
                    })
                    .expect("edit enqueued")
            })
            .collect();

        for id in ids {
            assert!(
                matches!(
                    session.wait_for_response(id, Duration::from_secs(5)),
                    WaitOutcome::Matched(_)
                ),
                "request {id} never completed"
            );
        }
        assert!(session
            .get_display_list_bytes()
            .document_text
            .starts_with(&"x".repeat(200)));
    });
}

/// Background forward relayout has no idle thread to run on inline, so it runs
/// one chunk per drive once the command queue is empty. Successive pumps must
/// therefore clear the whole pending window.
#[test]
fn r3_inline_background_reflow_advances_one_chunk_per_drive() {
    with_watchdog(Duration::from_secs(600), || {
        let bytes = NativeFormat::export(&flowing_document(240)).expect("export fixture");
        let session = Session::new_inline();
        let background_repaints = Arc::new(Mutex::new(0usize));
        let counter = Arc::clone(&background_repaints);
        session.set_event_observer(Arc::new(move |event: BridgeEvent| {
            if event.request_id() == BACKGROUND_REQUEST_ID {
                *counter.lock().expect("counter") += 1;
            }
        }));
        let open_id = session.open_bytes(bytes).expect("open enqueued");
        assert!(matches!(
            session.wait_for_response(open_id, Duration::from_secs(60)),
            WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
        ));
        let page_count = session.page_count();
        assert!(page_count > 4, "fixture should span >4 pages, got {page_count}");

        let run_id = session.hit_test(0, 72.0, 83.0).expect("page 0 hit").run_id;
        let edit_id = session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "PREFIX ".repeat(40),
            })
            .expect("edit enqueued");
        assert!(matches!(
            session.wait_for_response(edit_id, Duration::from_secs(60)),
            WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
        ));
        assert!(
            session.is_page_stale(page_count - 1),
            "the capped synchronous pass should leave the document tail pending"
        );

        let mut pumps = 0;
        while (0..session.page_count()).any(|page| session.is_page_stale(page)) {
            pumps += 1;
            assert!(pumps < 10_000, "background reflow never converged");
            session.pump_events();
        }
        assert!(
            pumps > 1,
            "the pending window should take more than one drive to clear"
        );
        assert!(
            *background_repaints.lock().expect("counter") > 0,
            "background chunks should publish repaints under BACKGROUND_REQUEST_ID"
        );
        assert!(
            session.hit_test(session.page_count() - 1, 100.0, 200.0).is_some(),
            "hit test should resolve on the last page once reflow completed"
        );
    });
}

/// Open and save round-trip through the inline queue — the shape the WASM smoke
/// test needs (bytes in, correlated bytes out, no thread).
#[test]
fn r3_inline_open_and_save_round_trip() {
    with_watchdog(Duration::from_secs(120), || {
        let bytes = NativeFormat::export(&flowing_document(3)).expect("export fixture");
        let session = Session::new_inline();
        let open_id = session.open_bytes(bytes).expect("open enqueued");
        assert!(matches!(
            session.wait_for_response(open_id, Duration::from_secs(30)),
            WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
        ));

        let save_id = session.save().expect("save enqueued");
        match session.wait_for_response(save_id, Duration::from_secs(30)) {
            WaitOutcome::Matched(BridgeEvent::DocumentSaved { data, .. }) => {
                assert!(!data.is_empty())
            }
            other => panic!("expected saved bytes, got {other:?}"),
        }
    });
}

/// Dropping an inline session with work still queued must not run or hang.
#[test]
fn r3_inline_drop_with_queued_commands_is_immediate() {
    with_watchdog(Duration::from_secs(30), || {
        let session = Session::new_inline();
        let run_id = first_run_id(&session);
        for i in 0..32 {
            let _ = session.apply(Command::InsertText {
                run_id,
                offset: i,
                text: "z".into(),
            });
        }
        drop(session);
    });
}

/// The native default keeps its worker thread: no driving, no behaviour change.
#[test]
fn r3_threaded_session_remains_the_native_default() {
    let session = Session::new();
    assert!(
        !session.requires_drive(),
        "Session::new() must stay threaded on native targets"
    );
    assert_eq!(session.drive(), 0, "a threaded session has nothing to drive");

    assert!(matches!(
        session.wait_for_startup(Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DocumentOpened { .. })
    ));
    let run_id = first_run_id(&session);
    let request_id = session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Threaded".into(),
        })
        .expect("edit enqueued");
    // No pump, no drive: the worker thread must complete this on its own.
    assert!(matches!(
        session.wait_for_response(request_id, Duration::from_secs(10)),
        WaitOutcome::Matched(BridgeEvent::DisplayListReady { .. })
    ));
    assert!(session
        .get_display_list_bytes()
        .document_text
        .starts_with("Threaded"));
}
