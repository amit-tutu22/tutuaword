//! Crash/perf Phase 2 stress: wasm/inline drive budget keeps the host responsive.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tw_core::{BridgeEvent, Session, WaitOutcome, INLINE_DRIVE_BUDGET, STARTUP_REQUEST_ID};

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
        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => match handle.join() {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        },
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("inline session did not finish within {limit:?}")
        }
    }
}

#[test]
fn stress_inline_drive_budget_caps_work_per_turn() {
    with_watchdog(Duration::from_secs(30), || {
        let session = Session::new_inline();
        assert!(matches!(
            session.wait_for_startup(Duration::from_secs(5)),
            WaitOutcome::Matched(BridgeEvent::DocumentOpened { request_id, .. })
                if request_id == STARTUP_REQUEST_ID
        ));

        // SetCurrentPage does not batch the way ApplyEdit does — each call is one
        // execute unit, which is what the wasm host budgets per frame.
        let queued = INLINE_DRIVE_BUDGET * 3;
        for page in 0..queued {
            session
                .set_current_page(page as u32 % 3)
                .expect("page set enqueued");
        }

        let first = session.drive();
        assert!(
            first > 0 && first <= INLINE_DRIVE_BUDGET + 2,
            "first drive should process a capped slice, got {first} (budget {INLINE_DRIVE_BUDGET})"
        );

        // Remaining work must still be available for later pumps — the tab stays
        // interactive instead of draining the whole queue in one turn.
        let mut remaining_drives = 0usize;
        let mut total = first;
        while remaining_drives < 64 {
            let n = session.drive();
            if n == 0 {
                break;
            }
            total += n;
            remaining_drives += 1;
            assert!(
                n <= INLINE_DRIVE_BUDGET + 2,
                "subsequent drive exceeded budget: {n}"
            );
        }
        assert!(
            total >= queued,
            "expected at least {queued} work units across pumps, got {total}"
        );
    });
}
