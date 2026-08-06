//! R0.1 — FFI buffer ownership round-trip and malformed ID rejection.

use std::ffi::{c_char, CString};
use std::ptr;
use std::sync::Mutex;
use tw_ffi::{
    tw_apply_insert_text, tw_document_tail_hit, tw_free_buffer, tw_get_document_text, tw_init,
    tw_shutdown,
};

/// Global session tests share one FFI singleton; run serially.
static TEST_LOCK: Mutex<()> = Mutex::new(());

extern "C" fn noop_callback(_event_type: u32, _data: *const u8, _len: usize) {}

fn init_session() {
    assert_eq!(tw_init(noop_callback), 0, "tw_init failed");
}

fn shutdown_session() {
    tw_shutdown();
}

#[test]
fn buffer_alloc_free_roundtrip() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(tw_get_document_text(&mut out_ptr, &mut out_len), 0);
    assert!(!out_ptr.is_null());
    tw_free_buffer(out_ptr as *mut u8, out_len);

    shutdown_session();
}

#[test]
fn invalid_run_id_is_rejected() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let bad_id = CString::new("not-a-uuid").unwrap();
    let text = CString::new("x").unwrap();
    assert_eq!(
        tw_apply_insert_text(bad_id.as_ptr(), 0, text.as_ptr()),
        -2,
        "malformed run id must return -2"
    );

    shutdown_session();
}

#[test]
fn insert_text_with_valid_run_id_succeeds() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let mut run_buf = vec![0u8; 64];
    let mut offset = 0u32;
    assert_eq!(
        tw_document_tail_hit(0, run_buf.as_mut_ptr() as *mut c_char, run_buf.len(), &mut offset),
        0
    );
    let run_id = CString::new(
        std::str::from_utf8(run_buf.split(|&b| b == 0).next().unwrap()).unwrap(),
    )
    .unwrap();
    let text = CString::new("R0").unwrap();
    assert_eq!(tw_apply_insert_text(run_id.as_ptr(), offset, text.as_ptr()), 0);

    shutdown_session();
}
