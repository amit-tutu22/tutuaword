//! Host font injection must work on an idle threaded session.
//!
//! Mobile hosts register bundled faces as their first call after `Session::new`,
//! when the worker has finished startup layout and parked itself waiting for
//! work. If that idle wait only watches the command channel, the registration
//! reply never arrives and the caller blocks forever — which on Flutter means a
//! blank screen, because `register_face` is synchronous on the UI isolate.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tw_core::Session;
use tw_layout::FontFaceSpec;

/// Runs `body` on a helper thread and fails the test if it has not finished
/// within `limit`, so a deadlock surfaces as a failure instead of wedging CI.
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
            panic!("threaded session did not answer register_face within {limit:?} — it deadlocked")
        }
    }
}

fn bundled_font() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../app/assets/fonts/NotoSans-Regular.ttf"
    );
    std::fs::read(path).expect("bundled NotoSans face")
}

#[test]
fn registers_face_on_idle_worker() {
    let data = bundled_font();
    with_watchdog(Duration::from_secs(10), move || {
        let session = Session::new();
        // No commands sent: the worker is parked on its idle wait, exactly as it
        // is when a mobile host registers fonts right after `tw_init`.
        session
            .register_face(&FontFaceSpec::new("Arial"), data)
            .expect("register Arial on an idle threaded worker");
    });
}

#[test]
fn registers_every_alias_the_mobile_host_injects() {
    let data = bundled_font();
    with_watchdog(Duration::from_secs(20), move || {
        let session = Session::new();
        for family in [
            "Arial",
            "Calibri",
            "Helvetica",
            "Times New Roman",
            "Noto Sans",
            "Liberation Sans",
        ] {
            session
                .register_face(&FontFaceSpec::new(family), data.clone())
                .unwrap_or_else(|err| panic!("register {family}: {err}"));
        }
    });
}
