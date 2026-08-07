//! P1-7 async document exports, the stale-page probe, and the generation-only
//! atlas query. Dart is written against these exact codes.

use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tw_ffi::{
    tw_free_buffer, tw_get_atlas_generation, tw_init, tw_is_page_stale, tw_open_document_async,
    tw_pump_events, tw_save_document, tw_save_document_as_async, tw_save_document_async,
    tw_shutdown, tw_spell_check_document_async, tw_take_open_result, tw_take_saved_document,
    tw_take_spell_check_result,
};

/// Global session tests share one FFI singleton; run serially.
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// `tw_init` stores the callback once per process, so every test in this binary
/// shares this one.
static EVENTS_SEEN: AtomicUsize = AtomicUsize::new(0);
static WIRE_MISMATCHES: AtomicUsize = AtomicUsize::new(0);
static LAST_REQUEST_ID: AtomicU64 = AtomicU64::new(0);

/// The by-value scalars are authoritative; the payload must agree with them for as
/// long as the call is on the stack.
extern "C" fn counting_callback(
    event_type: u32,
    request_id: u64,
    payload: *const u8,
    payload_len: usize,
) {
    EVENTS_SEEN.fetch_add(1, Ordering::Relaxed);
    LAST_REQUEST_ID.store(request_id, Ordering::Relaxed);
    if payload.is_null() || payload_len < 12 {
        WIRE_MISMATCHES.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let wire = unsafe { std::slice::from_raw_parts(payload, payload_len) };
    let wire_type = u32::from_le_bytes(wire[0..4].try_into().unwrap());
    let wire_request = u64::from_le_bytes(wire[4..12].try_into().unwrap());
    if wire_type != event_type || wire_request != request_id {
        WIRE_MISMATCHES.fetch_add(1, Ordering::Relaxed);
    }
}

fn init_session() {
    assert_eq!(tw_init(counting_callback), 0, "tw_init failed");
}

/// Poll a getter the way Dart will: pump, retry while it reports `1`.
fn poll_until_ready(mut attempt: impl FnMut() -> i32) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        tw_pump_events();
        let code = attempt();
        if code != 1 || Instant::now() >= deadline {
            return code;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn save_async_returns_request_id_and_buffers_the_result() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut request_id = 0u64;
    assert_eq!(tw_save_document_async(&mut request_id), 0);
    assert_ne!(request_id, 0, "enqueue must hand back a correlation id");

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(
        poll_until_ready(|| tw_take_saved_document(request_id, &mut out_ptr, &mut out_len)),
        0
    );
    assert!(out_len > 0, "saved document should carry bytes");
    tw_free_buffer(out_ptr as *mut u8, out_len);

    // The slot is consumed by a successful take.
    assert_eq!(
        tw_take_saved_document(request_id, &mut out_ptr, &mut out_len),
        -3
    );

    tw_shutdown();
}

#[test]
fn result_survives_until_the_getter_runs() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut request_id = 0u64;
    assert_eq!(tw_save_document_async(&mut request_id), 0);

    // Let the worker finish and keep pumping well past completion; the payload
    // must still be there when Dart eventually asks for it.
    for _ in 0..200 {
        tw_pump_events();
        std::thread::sleep(Duration::from_millis(1));
    }

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(
        tw_take_saved_document(request_id, &mut out_ptr, &mut out_len),
        0,
        "result must be buffered by request id, not consumed by the pump"
    );
    tw_free_buffer(out_ptr as *mut u8, out_len);

    tw_shutdown();
}

/// The whole reason `tw_pump_events` exists: registering a callback with `tw_init`
/// delivers nothing on its own, because no other export drains the event channel.
#[test]
fn events_reach_the_callback_only_once_pumped() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();
    // `tw_init` itself blocks on startup, which drains; start counting after it.
    EVENTS_SEEN.store(0, Ordering::Relaxed);
    WIRE_MISMATCHES.store(0, Ordering::Relaxed);

    let mut request_id = 0u64;
    assert_eq!(tw_save_document_async(&mut request_id), 0);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(
        EVENTS_SEEN.load(Ordering::Relaxed),
        0,
        "an enqueue must not deliver events by itself -- an unpumped host sees nothing"
    );

    assert!(tw_pump_events() > 0, "pump should deliver the completion");
    assert!(EVENTS_SEEN.load(Ordering::Relaxed) > 0);
    assert_eq!(
        LAST_REQUEST_ID.load(Ordering::Relaxed),
        request_id,
        "the correlation id must arrive by value, not only in the payload"
    );
    assert_eq!(
        WIRE_MISMATCHES.load(Ordering::Relaxed),
        0,
        "payload must agree with the by-value scalars while the call is on the stack"
    );

    tw_shutdown();
}

#[test]
fn unknown_request_id_is_distinct_from_pending() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(
        tw_take_saved_document(999_999, &mut out_ptr, &mut out_len),
        -3
    );
    assert_eq!(tw_take_open_result(999_999), -3);

    tw_shutdown();
}

#[test]
fn abandoned_results_are_evicted_rather_than_leaked() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut first = 0u64;
    assert_eq!(tw_save_document_async(&mut first), 0);
    // Overrun the slot bound without ever collecting.
    for _ in 0..32 {
        let mut id = 0u64;
        assert_eq!(tw_save_document_async(&mut id), 0);
    }
    tw_pump_events();

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(
        tw_take_saved_document(first, &mut out_ptr, &mut out_len),
        -3,
        "the oldest abandoned slot must be evicted"
    );

    tw_shutdown();
}

#[test]
fn save_as_and_spell_check_async_round_trip() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let format = CString::new("docx").unwrap();
    let mut save_id = 0u64;
    assert_eq!(tw_save_document_as_async(format.as_ptr(), &mut save_id), 0);
    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(
        poll_until_ready(|| tw_take_saved_document(save_id, &mut out_ptr, &mut out_len)),
        0
    );
    tw_free_buffer(out_ptr as *mut u8, out_len);

    let mut spell_id = 0u64;
    assert_eq!(tw_spell_check_document_async(&mut spell_id), 0);
    let mut spell_ptr: *const u8 = ptr::null();
    let mut spell_len = 0usize;
    assert_eq!(
        poll_until_ready(|| tw_take_spell_check_result(
            spell_id,
            &mut spell_ptr,
            &mut spell_len
        )),
        0
    );
    tw_free_buffer(spell_ptr as *mut u8, spell_len);

    tw_shutdown();
}

#[test]
fn blocking_wrappers_still_work_alongside_the_async_path() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(tw_save_document(&mut out_ptr, &mut out_len), 0);
    assert!(out_len > 0);
    tw_free_buffer(out_ptr as *mut u8, out_len);

    tw_shutdown();
}

#[test]
fn open_async_round_trips_saved_bytes() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(tw_save_document(&mut out_ptr, &mut out_len), 0);
    let bytes = unsafe { std::slice::from_raw_parts(out_ptr, out_len) }.to_vec();
    tw_free_buffer(out_ptr as *mut u8, out_len);

    let mut request_id = 0u64;
    assert_eq!(
        tw_open_document_async(bytes.as_ptr(), bytes.len(), ptr::null(), &mut request_id),
        0
    );
    assert_ne!(request_id, 0);
    assert_eq!(poll_until_ready(|| tw_take_open_result(request_id)), 0);

    tw_shutdown();
}

#[test]
fn stale_probe_and_atlas_generation_report_no_session() {
    let _guard = TEST_LOCK.lock().unwrap();
    tw_shutdown();

    assert_eq!(tw_is_page_stale(0), -1);
    let mut generation = 0u64;
    assert_eq!(tw_get_atlas_generation(&mut generation), -1);
    assert_eq!(tw_get_atlas_generation(ptr::null_mut()), -2);
}

#[test]
fn fresh_pages_are_not_stale_and_atlas_generation_is_readable() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    assert_eq!(tw_is_page_stale(0), 0);
    let mut generation = 0u64;
    assert_eq!(tw_get_atlas_generation(&mut generation), 0);

    tw_shutdown();
}
