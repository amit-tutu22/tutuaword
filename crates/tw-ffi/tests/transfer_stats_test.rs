//! B0 transfer-size instrumentation hooks.

use std::ffi::{c_char, CString};
use std::ptr;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tw_ffi::{
    tw_apply_insert_text, tw_document_tail_hit, tw_free_buffer, tw_get_page_display_list,
    tw_get_transfer_stats, tw_init, tw_pump_events, tw_reset_transfer_stats, tw_shutdown,
};

static TEST_LOCK: Mutex<()> = Mutex::new(());

extern "C" fn noop_callback(_: u32, _: u64, _: *const u8, _: usize) {}

fn init_session() {
    assert_eq!(tw_init(noop_callback), 0);
}

fn run_id_on_page_zero() -> (CString, u32) {
    let mut buf = vec![0u8; 64];
    let mut offset = 0u32;
    assert_eq!(
        tw_document_tail_hit(0, buf.as_mut_ptr() as *mut c_char, buf.len(), &mut offset),
        0,
    );
    (
        CString::new(std::str::from_utf8(buf.split(|&b| b == 0).next().unwrap()).unwrap()).unwrap(),
        offset,
    )
}

#[test]
fn transfer_stats_record_page_bytes() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();
    tw_reset_transfer_stats();

    let (run_id, offset) = run_id_on_page_zero();
    assert_eq!(tw_apply_insert_text(run_id.as_ptr(), offset, c"x".as_ptr()), 0);

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        tw_pump_events();
        std::thread::sleep(Duration::from_millis(2));
    }

    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    let mut out_version = 0u64;
    let mut out_width = 0f32;
    let mut out_height = 0f32;
    assert_eq!(
        tw_get_page_display_list(
            0,
            &mut out_ptr,
            &mut out_len,
            &mut out_version,
            &mut out_width,
            &mut out_height,
        ),
        0
    );
    assert!(out_len > 0);
    tw_free_buffer(out_ptr as *mut u8, out_len);

    let mut page_dl_bytes = 0u64;
    let mut page_transfers = 0u64;
    tw_get_transfer_stats(
        &mut page_dl_bytes,
        &mut 0,
        &mut 0,
        &mut page_transfers,
        &mut 0,
        &mut 0,
        &mut 0,
    );
    assert_eq!(page_transfers, 1);
    assert!(page_dl_bytes > 0);

    tw_shutdown();
}
