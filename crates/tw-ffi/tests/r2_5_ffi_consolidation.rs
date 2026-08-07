//! R2.5 — JSON command dispatch and format field extensibility.

use std::ffi::{c_char, CString};
use std::ptr;
use std::sync::Mutex;
use std::time::Duration;
use tw_edit::{command_to_json_str, Command};
use tw_ffi::{
    tw_dispatch, tw_document_tail_hit, tw_free_buffer, tw_get_caret_format, tw_get_document_text,
    tw_init, tw_last_request_id, tw_shutdown, tw_undo,
};
use tw_model::{CharFormat, NodeId};
use uuid::Uuid;

static TEST_LOCK: Mutex<()> = Mutex::new(());

extern "C" fn noop_callback(
    _event_type: u32,
    _request_id: u64,
    _payload: *const u8,
    _payload_len: usize,
) {
}

fn init_session() {
    assert_eq!(tw_init(noop_callback), 0);
}

fn shutdown_session() {
    tw_shutdown();
}

fn tail_run_id() -> (CString, u32) {
    let mut run_buf = vec![0u8; 64];
    let mut offset = 0u32;
    assert_eq!(
        tw_document_tail_hit(0, run_buf.as_mut_ptr() as *mut c_char, run_buf.len(), &mut offset),
        0
    );
    let run_id = CString::new(
        std::str::from_utf8(run_buf.split(|&b| b == 0).next().unwrap())
            .unwrap()
            .to_string(),
    )
    .unwrap();
    (run_id, offset)
}

fn dispatch_json(command: &Command) -> i32 {
    let json = command_to_json_str(command).expect("command json");
    tw_dispatch(json.as_ptr(), json.len())
}

fn document_text() -> String {
    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(tw_get_document_text(&mut out_ptr, &mut out_len), 0);
    let text = unsafe { std::slice::from_raw_parts(out_ptr, out_len) };
    let s = String::from_utf8(text.to_vec()).unwrap();
    tw_free_buffer(out_ptr as *mut u8, out_len);
    s
}

fn wait_for_document_text<F: Fn(&str) -> bool>(predicate: F) {
    for _ in 0..500 {
        if predicate(&document_text()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for document text predicate");
}

fn caret_format_json(run_id: &CString) -> String {
    let mut out_ptr: *const u8 = ptr::null();
    let mut out_len = 0usize;
    assert_eq!(
        tw_get_caret_format(run_id.as_ptr(), &mut out_ptr, &mut out_len),
        0
    );
    let json = unsafe { std::slice::from_raw_parts(out_ptr, out_len) };
    let s = String::from_utf8(json.to_vec()).unwrap();
    tw_free_buffer(out_ptr as *mut u8, out_len);
    s
}

#[test]
fn dispatch_insert_text_and_undo() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let (run_id, offset) = tail_run_id();
    let insert = Command::InsertText {
        run_id: NodeId::from_uuid(Uuid::parse_str(run_id.to_str().unwrap()).unwrap()),
        offset: offset as usize,
        text: "R25".into(),
    };
    assert_eq!(dispatch_json(&insert), 0);
    assert!(tw_last_request_id() > 0);
    wait_for_document_text(|text| text.contains("R25"));

    let text = document_text();
    assert!(text.contains("R25"));

    assert_eq!(tw_undo(), 0);
    wait_for_document_text(|text| !text.contains("R25"));

    shutdown_session();
}

#[test]
fn dispatch_char_format_extra_field_without_dart_typedef() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let (run_id, offset) = tail_run_id();
    let run_uuid = NodeId::from_uuid(Uuid::parse_str(run_id.to_str().unwrap()).unwrap());

    assert_eq!(
        dispatch_json(&Command::InsertText {
            run_id: run_uuid,
            offset: offset as usize,
            text: "spacing".into(),
        }),
        0
    );
    wait_for_document_text(|text| text.contains("spacing"));

    let format = CharFormat {
        character_spacing: Some(3.5),
        ..Default::default()
    };
    let command = Command::SetCharFormat {
        run_id: run_uuid,
        start: offset as usize,
        end: offset as usize + "spacing".chars().count(),
        format,
        merge: true,
    };
    assert_eq!(dispatch_json(&command), 0);
    for _ in 0..500 {
        let json = caret_format_json(&run_id);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        if value
            .pointer("/char_format/character_spacing")
            .and_then(|v| v.as_f64())
            == Some(3.5)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for character_spacing in caret format");

    let json = caret_format_json(&run_id);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let spacing = value
        .pointer("/char_format/character_spacing")
        .and_then(|v| v.as_f64());
    assert_eq!(spacing, Some(3.5));

    shutdown_session();
}

#[test]
fn dispatch_rejects_invalid_json() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    let bad = b"{not json";
    assert_eq!(tw_dispatch(bad.as_ptr(), bad.len()), -3);

    shutdown_session();
}

#[test]
fn dispatch_rejects_empty_payload() {
    let _guard = TEST_LOCK.lock().unwrap();
    init_session();

    assert_eq!(tw_dispatch(ptr::null(), 0), -3);

    shutdown_session();
}
