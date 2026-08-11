//! Which queries a pending forward reflow is allowed to refuse.
//!
//! During catch-up the document model is fully current -- the edit has been
//! applied -- and only per-page geometry lags. So a query that reads the model
//! must still answer, and only a query that reads a page's line map may refuse.

use std::ffi::{c_char, CString};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tw_ffi::{
    tw_apply_insert_text, tw_document_tail_hit, tw_free_buffer, tw_get_display_list, tw_hit_test,
    tw_init, tw_is_page_stale, tw_open_document, tw_pump_events, tw_shutdown,
};
use tw_model::{Block, Document, Paragraph};
use tw_native::NativeFormat;

/// Global session tests share one FFI singleton; run serially.
static TEST_LOCK: Mutex<()> = Mutex::new(());

extern "C" fn noop_callback(
    _event_type: u32,
    _request_id: u64,
    _payload: *const u8,
    _payload_len: usize,
) {
}

/// Continuous prose with no page breaks, so an edit on page 0 shifts every later
/// page and the reflow cannot converge inside the synchronous window.
fn flowing_document(paragraphs: usize) -> Vec<u8> {
    let mut doc = Document::new();
    let filler = "The quick brown fox jumps over the lazy dog while the sleepy cat \
                  watches from a sunny windowsill and the kettle boils.";
    doc.sections[0].blocks = (0..paragraphs)
        .map(|idx| Block::Paragraph(Paragraph::with_text(format!("{idx}. {filler}"))))
        .collect();
    NativeFormat::export(&doc).expect("export fixture")
}

fn page_count() -> u32 {
    let mut ptr: *const u8 = std::ptr::null();
    let mut len = 0usize;
    let (mut version, mut width, mut height, mut pages) = (0u64, 0f32, 0f32, 0u32);
    assert_eq!(
        tw_get_display_list(
            &mut ptr,
            &mut len,
            &mut version,
            &mut width,
            &mut height,
            &mut pages
        ),
        0
    );
    tw_free_buffer(ptr as *mut u8, len);
    pages
}

fn run_id_on_page_zero() -> CString {
    let mut buf = vec![0u8; 64];
    let mut offset = 0u32;
    assert_eq!(
        tw_hit_test(0, 72.0, 83.0, buf.as_mut_ptr() as *mut c_char, buf.len(), &mut offset),
        0,
        "page 0 should hit before any edit"
    );
    CString::new(std::str::from_utf8(buf.split(|&b| b == 0).next().unwrap()).unwrap()).unwrap()
}

/// Open a multi-page fixture and cascade an edit through it. Returns the page
/// count and a late page observed stale, or `None` if catch-up outran the poll.
fn open_and_cascade_edit() -> Option<(u32, u32)> {
    // Just long enough to cascade well past the synchronous window; a bigger
    // corpus only slows the debug build down.
    let bytes = flowing_document(400);
    assert_eq!(tw_open_document(bytes.as_ptr(), bytes.len()), 0);

    let pages = page_count();
    assert!(pages >= 15, "fixture should span many pages, got {pages}");

    let run_id = run_id_on_page_zero();
    let text = CString::new("PREFIX ".repeat(40)).unwrap();
    assert_eq!(tw_apply_insert_text(run_id.as_ptr(), 0, text.as_ptr()), 0);

    let last = pages - 1;
    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        tw_pump_events();
        if tw_is_page_stale(last) == 1 {
            return Some((pages, last));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    None
}

#[test]
fn document_tail_hit_resolves_while_pages_are_stale() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert_eq!(tw_init(noop_callback), 0);

    let staged = open_and_cascade_edit();
    let Some((_pages, last)) = staged else {
        tw_shutdown();
        panic!("background reflow finished before staleness could be observed");
    };

    let mut buf = vec![0u8; 64];
    let mut offset = 0u32;
    let code = tw_document_tail_hit(
        last,
        buf.as_mut_ptr() as *mut c_char,
        buf.len(),
        &mut offset,
    );
    tw_shutdown();

    assert_eq!(
        code, 0,
        "tail hit reads the document model, not page geometry, so a stale page \
         must not make it fail (select-all depends on this)"
    );
    assert!(
        buf[0] != 0,
        "tail hit should have written a run id even on a stale page"
    );
}

#[test]
fn hit_test_reports_stale_pages_distinctly_from_a_miss() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert_eq!(tw_init(noop_callback), 0);

    let staged = open_and_cascade_edit();
    let Some((_pages, last)) = staged else {
        tw_shutdown();
        panic!("background reflow finished before staleness could be observed");
    };

    let mut buf = vec![0u8; 64];
    let mut offset = 0u32;
    let stale_code = tw_hit_test(
        last,
        72.0,
        83.0,
        buf.as_mut_ptr() as *mut c_char,
        buf.len(),
        &mut offset,
    );
    // A page that is current but has nothing under the point is a plain miss.
    let miss_code = tw_hit_test(
        0,
        -500.0,
        -500.0,
        buf.as_mut_ptr() as *mut c_char,
        buf.len(),
        &mut offset,
    );
    tw_shutdown();

    assert_eq!(stale_code, -4, "stale page must be distinguishable");
    assert_ne!(miss_code, -4, "a genuine miss must not report as stale");
}
