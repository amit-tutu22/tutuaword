//! A host calling back into the engine from its event callback must not deadlock.
//!
//! The callback used to run inside `with_session`, i.e. with the non-reentrant
//! `SESSION` mutex held, so any re-entrant export would block forever. Events are
//! now collected under the lock and forwarded after releasing it.

use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;
use tw_ffi::{
    tw_free_buffer, tw_get_atlas_generation, tw_get_document_text, tw_init, tw_is_page_stale,
    tw_pump_events, tw_save_document_async, tw_shutdown, tw_take_saved_document,
};

/// Global session tests share one FFI singleton; run serially.
static TEST_LOCK: Mutex<()> = Mutex::new(());

static REENTRANT_CALLS: AtomicUsize = AtomicUsize::new(0);
static REENTRANT_FAILURES: AtomicUsize = AtomicUsize::new(0);

/// Re-enters a spread of exports that all take the session lock, including one
/// that touches the async result table the observer also writes.
extern "C" fn reentrant_callback(
    _event_type: u32,
    request_id: u64,
    _payload: *const u8,
    _payload_len: usize,
) {
    REENTRANT_CALLS.fetch_add(1, Ordering::Relaxed);

    if tw_is_page_stale(0) < 0 {
        REENTRANT_FAILURES.fetch_add(1, Ordering::Relaxed);
    }

    let mut generation = 0u64;
    if tw_get_atlas_generation(&mut generation) != 0 {
        REENTRANT_FAILURES.fetch_add(1, Ordering::Relaxed);
    }

    let mut ptr_out: *const u8 = ptr::null();
    let mut len_out = 0usize;
    if tw_get_document_text(&mut ptr_out, &mut len_out) == 0 {
        tw_free_buffer(ptr_out as *mut u8, len_out);
    } else {
        REENTRANT_FAILURES.fetch_add(1, Ordering::Relaxed);
    }

    // Collecting the very result this event announces, from inside the callback,
    // is the most natural thing a host would try.
    let mut saved: *const u8 = ptr::null();
    let mut saved_len = 0usize;
    if tw_take_saved_document(request_id, &mut saved, &mut saved_len) == 0 {
        tw_free_buffer(saved as *mut u8, saved_len);
    }
}

#[test]
fn reentering_an_export_from_the_event_callback_completes() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert_eq!(tw_init(reentrant_callback), 0);
    REENTRANT_CALLS.store(0, Ordering::Relaxed);
    REENTRANT_FAILURES.store(0, Ordering::Relaxed);

    let mut request_id = 0u64;
    assert_eq!(tw_save_document_async(&mut request_id), 0);
    std::thread::sleep(Duration::from_millis(200));

    // A deadlock here would hang the whole test binary, so drive the pump on a
    // worker and fail the test rather than wedging CI.
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let delivered = tw_pump_events();
        let _ = tx.send(delivered);
    });
    let delivered = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("pump deadlocked: the host callback ran while an engine lock was held");

    assert!(delivered > 0, "pump should have delivered the completion");
    assert!(
        REENTRANT_CALLS.load(Ordering::Relaxed) > 0,
        "callback should have run"
    );
    assert_eq!(
        REENTRANT_FAILURES.load(Ordering::Relaxed),
        0,
        "re-entrant exports should see a live session, not a poisoned or absent one"
    );

    tw_shutdown();
}
