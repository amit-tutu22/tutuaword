use parking_lot::Mutex;
use serde::Deserialize;
use std::ffi::{c_char, CStr};
use std::slice;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::Duration;
use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::{Command, DocPosition, DocRange};
use tw_model::{CharFormat, NodeId, ParaFormat};
use uuid::Uuid;

static SESSION: Mutex<Option<Session>> = Mutex::new(None);
static LAST_ERROR: StdMutex<Option<String>> = StdMutex::new(None);

type EventCallback = extern "C" fn(event_type: u32, data: *const u8, len: usize);

#[derive(Default, Deserialize)]
struct CharFormatPatch {
    #[serde(flatten)]
    format: CharFormat,
    #[serde(default)]
    clear_color: bool,
    #[serde(default)]
    clear_highlight: bool,
}

static EVENT_CALLBACK: OnceLock<EventCallback> = OnceLock::new();

/// Minimal wire format event types (LE u32 in callback payload bytes 0..4).
pub const TW_EVENT_DISPLAY_LIST_READY: u32 = 1;
pub const TW_EVENT_DOCUMENT_OPENED: u32 = 2;
pub const TW_EVENT_DOCUMENT_SAVED: u32 = 3;
pub const TW_EVENT_SPELL_CHECK_RESULT: u32 = 4;
pub const TW_EVENT_ERROR: u32 = 5;

const EVENT_WIRE_BYTES: usize = 12;

const EDIT_WAIT: Duration = Duration::from_millis(100);
const BLOCKING_WAIT: Duration = Duration::from_secs(30);
/// Returned when an exported function panics across the FFI boundary.
const FFI_PANIC: i32 = -99;

fn record_internal_panic() {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some("internal engine panic".to_string());
    }
}

fn guard_ffi<F: FnOnce() -> i32>(f: F) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(code) => code,
        Err(_) => {
            record_internal_panic();
            FFI_PANIC
        }
    }
}

fn guard_ffi_void<F: FnOnce()>(f: F) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
}

fn bridge_event_type(event: &BridgeEvent) -> u32 {
    match event {
        BridgeEvent::DisplayListReady { .. } => TW_EVENT_DISPLAY_LIST_READY,
        BridgeEvent::DocumentOpened { .. } => TW_EVENT_DOCUMENT_OPENED,
        BridgeEvent::DocumentSaved { .. } => TW_EVENT_DOCUMENT_SAVED,
        BridgeEvent::SpellCheckResult { .. } => TW_EVENT_SPELL_CHECK_RESULT,
        BridgeEvent::Error { .. } => TW_EVENT_ERROR,
    }
}

fn encode_event_wire(event: &BridgeEvent) -> [u8; EVENT_WIRE_BYTES] {
    let event_type = bridge_event_type(event);
    let request_id = event.request_id();
    let mut wire = [0u8; EVENT_WIRE_BYTES];
    wire[0..4].copy_from_slice(&event_type.to_le_bytes());
    wire[4..12].copy_from_slice(&request_id.to_le_bytes());
    wire
}

fn forward_event_to_dart(event: BridgeEvent) {
    let Some(callback) = EVENT_CALLBACK.get() else {
        return;
    };
    let event_type = bridge_event_type(&event);
    let wire = encode_event_wire(&event);
    callback(event_type, wire.as_ptr(), wire.len());
}

fn install_event_observer(session: &Session) {
    session.set_event_observer(Arc::new(forward_event_to_dart));
}

/// Transfer buffer ownership to the Dart caller. Must be released with `tw_free_buffer`.
fn transfer_bytes_to_caller(bytes: Vec<u8>, out_ptr: *mut *const u8, out_len: *mut usize) {
    let len = bytes.len();
    let boxed = bytes.into_boxed_slice();
    let ptr = Box::into_raw(boxed) as *mut u8;
    unsafe {
        *out_ptr = ptr;
        *out_len = len;
    }
}

fn with_session<F: FnOnce(&Session) -> i32>(f: F) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    f(session)
}

fn record_event_error(event: &BridgeEvent) {
    if let BridgeEvent::Error { message, .. } = event {
        if let Ok(mut guard) = LAST_ERROR.lock() {
            *guard = Some(message.clone());
        }
    }
}

fn wait_for_request(
    session: &Session,
    request_id: u64,
    timeout: Duration,
    accept: impl Fn(&BridgeEvent) -> Option<i32>,
) -> i32 {
    match session.wait_for_response(request_id, timeout) {
        WaitOutcome::Matched(event) => {
            record_event_error(&event);
            accept(&event).unwrap_or(-2)
        }
        WaitOutcome::Timeout => -3,
    }
}

fn wait_for_document_ready(session: &Session) -> i32 {
    wait_for_request(session, STARTUP_REQUEST_ID, BLOCKING_WAIT, |event| match event {
        BridgeEvent::DocumentOpened { .. } | BridgeEvent::DisplayListReady { .. } => Some(0),
        BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

fn wait_for_open(session: &Session, request_id: u64) -> i32 {
    wait_for_request(session, request_id, BLOCKING_WAIT, |event| match event {
        BridgeEvent::DocumentOpened { .. } => Some(0),
        BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

/// Brief wait on edit hot paths; return success if the worker is still catching up.
fn wait_for_document_edit(session: &Session, request_id: u64) -> i32 {
    match wait_for_request(session, request_id, EDIT_WAIT, |event| match event {
        BridgeEvent::DisplayListReady { .. } => Some(0),
        BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    }) {
        -3 => 0,
        code => code,
    }
}

fn wait_for_document_saved(
    session: &Session,
    request_id: u64,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    wait_for_request(session, request_id, BLOCKING_WAIT, |event| match event {
        BridgeEvent::DocumentSaved { data, .. } => {
            transfer_bytes_to_caller(data.clone(), out_ptr, out_len);
            Some(0)
        }
        BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

fn wait_for_spell_check(
    session: &Session,
    request_id: u64,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    wait_for_request(session, request_id, BLOCKING_WAIT, |event| match event {
        BridgeEvent::SpellCheckResult { misspellings, .. } => {
            transfer_bytes_to_caller(misspellings.join("\n").into_bytes(), out_ptr, out_len);
            Some(0)
        }
        BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

#[no_mangle]
pub extern "C" fn tw_init(callback: EventCallback) -> i32 {
    guard_ffi(|| {
        let _ = EVENT_CALLBACK.set(callback);
        {
            let mut guard = SESSION.lock();
            let session = Session::new();
            install_event_observer(&session);
            *guard = Some(session);
        }
        with_session(|session| wait_for_document_ready(session))
    })
}

#[no_mangle]
pub extern "C" fn tw_shutdown() {
    guard_ffi_void(|| {
        let mut guard = SESSION.lock();
        *guard = None;
    });
}

#[no_mangle]
pub extern "C" fn tw_new_document() -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.new_document() else {
                return -4;
            };
            wait_for_open(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_insert_text(
    run_id_ptr: *const c_char,
    offset: u32,
    text_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(run_id) = parse_node_id(run_id_ptr) else {
                return -2;
            };
            let Some(text) = parse_cstr(text_ptr) else {
                return -3;
            };
            let Some(request_id) = session.apply(Command::InsertText {
                run_id,
                offset: offset as usize,
                text,
            }) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_get_display_list(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
    out_version: *mut u64,
    out_width: *mut f32,
    out_height: *mut f32,
    out_page_count: *mut u32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };

        let snapshot = session.get_display_list_bytes();
        transfer_bytes_to_caller(snapshot.bytes.clone(), out_ptr, out_len);
        unsafe {
            *out_version = snapshot.version;
            *out_width = snapshot.page_width;
            *out_height = snapshot.page_height;
            *out_page_count = snapshot.page_count;
        }
        0
    })
}

/// Display list for a specific page, for continuous scrolling.
///
/// Unlike `tw_get_display_list` this does not depend on, or change, the
/// session's current page.
#[no_mangle]
pub extern "C" fn tw_get_page_display_list(
    page: u32,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
    out_width: *mut f32,
    out_height: *mut f32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };

        let Some(snapshot) = session.page_display_list(page) else {
            return -2;
        };

        transfer_bytes_to_caller(snapshot.bytes.as_ref().clone(), out_ptr, out_len);
        unsafe {
            *out_width = snapshot.page_width;
            *out_height = snapshot.page_height;
        }
        0
    })
}

/// Session glyph atlas resource, versioned independently from page display lists (R1.2).
#[no_mangle]
pub extern "C" fn tw_get_atlas(
    out_generation: *mut u64,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
    out_width: *mut u32,
    out_height: *mut u32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };

        let (generation, width, height, bytes) = session.atlas_resource();
        transfer_bytes_to_caller(bytes.as_ref().clone(), out_ptr, out_len);
        unsafe {
            *out_generation = generation;
            *out_width = width;
            *out_height = height;
        }
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_get_last_error(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    guard_ffi(|| {
        let guard = match LAST_ERROR.lock() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        let Some(message) = guard.as_ref() else {
            return -1;
        };
        transfer_bytes_to_caller(message.clone().into_bytes(), out_ptr, out_len);
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_get_document_properties_json(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };

        let json = session.get_display_list_bytes().document_properties_json;
        transfer_bytes_to_caller(json.into_bytes(), out_ptr, out_len);
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_is_document_read_only() -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        if session.get_display_list_bytes().read_only {
            1
        } else {
            0
        }
    })
}

#[no_mangle]
pub extern "C" fn tw_get_document_text(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };

        let text = session.get_display_list_bytes().document_text;
        transfer_bytes_to_caller(text.into_bytes(), out_ptr, out_len);
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_get_text_range(
    start_run_id_ptr: *const c_char,
    start_offset: u32,
    end_run_id_ptr: *const c_char,
    end_offset: u32,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let Some(start_run) = parse_node_id(start_run_id_ptr) else {
            return -2;
        };
        let Some(end_run) = parse_node_id(end_run_id_ptr) else {
            return -2;
        };
        let Some(text) = session.text_in_range(
            start_run,
            start_offset as usize,
            end_run,
            end_offset as usize,
        ) else {
            return -3;
        };
        transfer_bytes_to_caller(text.into_bytes(), out_ptr, out_len);
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_save_document(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.save() else {
                return -4;
            };
            wait_for_document_saved(session, request_id, out_ptr, out_len)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_open_document(data: *const u8, len: usize) -> i32 {
    guard_ffi(|| tw_open_document_with_path(data, len, std::ptr::null()))
}

#[no_mangle]
pub extern "C" fn tw_open_document_with_path(
    data: *const u8,
    len: usize,
    path_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let bytes = unsafe { slice::from_raw_parts(data, len) }.to_vec();
            let path_hint = if path_ptr.is_null() {
                None
            } else {
                Some(
                    unsafe { CStr::from_ptr(path_ptr) }
                        .to_string_lossy()
                        .into_owned(),
                )
            };
            let Some(request_id) = session.open_bytes_with_path(bytes, path_hint) else {
                return -4;
            };
            if let Ok(mut guard) = LAST_ERROR.lock() {
                *guard = None;
            }
            wait_for_open(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_free_buffer(ptr: *mut u8, len: usize) {
    guard_ffi_void(|| {
        if ptr.is_null() || len == 0 {
            return;
        }
        unsafe {
            let slice = std::ptr::slice_from_raw_parts_mut(ptr, len);
            drop(Box::from_raw(slice));
        }
    });
}

#[no_mangle]
pub extern "C" fn tw_set_current_page(page: u32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.set_current_page(page) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_heading1(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.apply_heading1_at(caret_run_id) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_normal_style(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.apply_normal_style_at(caret_run_id) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_numbered_list(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.apply_numbered_list_at(caret_run_id) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_bullet_list(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.apply_bullet_list_at(caret_run_id) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

fn parse_node_id(ptr: *const c_char) -> Option<NodeId> {
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(ptr) }.to_string_lossy();
    Uuid::parse_str(&s).ok().map(NodeId::from_uuid)
}

fn parse_cstr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
}

/// Apply a character-format JSON delta over `[start, end)`.
///
/// `format_json` is a partial `CharFormat` object, e.g. `{"bold":true}`.
/// When start and end describe the same collapsed caret, format from the caret
/// to the end of that run (not the whole run from offset 0).
#[no_mangle]
pub extern "C" fn tw_apply_char_format(
    start_run_id_ptr: *const c_char,
    start_offset: u32,
    end_run_id_ptr: *const c_char,
    end_offset: u32,
    format_json_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(start_run) = parse_node_id(start_run_id_ptr) else {
                return -2;
            };
            let Some(end_run) = parse_node_id(end_run_id_ptr) else {
                return -2;
            };
            let Some(json) = parse_cstr(format_json_ptr) else {
                return -3;
            };
            let patch: CharFormatPatch = match serde_json::from_str(&json) {
                Ok(p) => p,
                Err(_) => return -3,
            };
            let format = patch.format;

            let collapsed = start_run == end_run && start_offset == end_offset;
            let range = DocRange {
                start: DocPosition {
                    run_id: start_run,
                    char_offset: start_offset as usize,
                },
                end: DocPosition {
                    run_id: end_run,
                    char_offset: end_offset as usize,
                },
            };

            let mut last_request_id = None;

            if patch.clear_color || patch.clear_highlight {
                let Some(request_id) = session.apply(Command::ClearCharFormatFields {
                    range: range.clone(),
                    clear_color: patch.clear_color,
                    clear_highlight: patch.clear_highlight,
                }) else {
                    return -4;
                };
                last_request_id = Some(request_id);
            }

            let has_format_fields = serde_json::from_str::<serde_json::Value>(&json)
                .ok()
                .is_some_and(|value| {
                    value.as_object().is_some_and(|obj| {
                        obj.keys()
                            .any(|k| k != "clear_color" && k != "clear_highlight")
                    })
                });
            if !has_format_fields {
                return match last_request_id {
                    Some(request_id) => wait_for_document_edit(session, request_id),
                    None => 0,
                };
            }

            let command = if collapsed {
                Command::SetCharFormat {
                    run_id: start_run,
                    start: start_offset as usize,
                    end: usize::MAX,
                    format,
                    merge: true,
                }
            } else if start_run == end_run {
                Command::SetCharFormat {
                    run_id: start_run,
                    start: start_offset as usize,
                    end: end_offset as usize,
                    format,
                    merge: true,
                }
            } else {
                Command::SetCharFormatRange {
                    range: DocRange {
                        start: DocPosition {
                            run_id: start_run,
                            char_offset: start_offset as usize,
                        },
                        end: DocPosition {
                            run_id: end_run,
                            char_offset: end_offset as usize,
                        },
                    },
                    format,
                    merge: true,
                }
            };

            let Some(request_id) = session.apply(command) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

/// Apply a paragraph-format JSON delta to every paragraph touched by the range.
#[no_mangle]
pub extern "C" fn tw_apply_para_format(
    start_run_id_ptr: *const c_char,
    start_offset: u32,
    end_run_id_ptr: *const c_char,
    end_offset: u32,
    format_json_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(start_run) = parse_node_id(start_run_id_ptr) else {
                return -2;
            };
            let Some(end_run) = parse_node_id(end_run_id_ptr) else {
                return -2;
            };
            let Some(json) = parse_cstr(format_json_ptr) else {
                return -3;
            };
            let format: ParaFormat = match serde_json::from_str(&json) {
                Ok(f) => f,
                Err(_) => return -3,
            };

            let Some(request_id) = session.apply(Command::SetParaFormatRange {
                range: DocRange {
                    start: DocPosition {
                        run_id: start_run,
                        char_offset: start_offset as usize,
                    },
                    end: DocPosition {
                        run_id: end_run,
                        char_offset: end_offset as usize,
                    },
                },
                format,
                merge: true,
            }) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_get_caret_format(
    run_id_ptr: *const c_char,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let Some(run_id) = parse_node_id(run_id_ptr) else {
            return -2;
        };
        let Some(json) = session.caret_format_json(run_id) else {
            return -3;
        };
        transfer_bytes_to_caller(json.into_bytes(), out_ptr, out_len);
        0
    })
}

/// Remove direct character and paragraph formatting for the given range.
#[no_mangle]
pub extern "C" fn tw_clear_format(
    start_run_id_ptr: *const c_char,
    start_offset: u32,
    end_run_id_ptr: *const c_char,
    end_offset: u32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(start_run) = parse_node_id(start_run_id_ptr) else {
                return -2;
            };
            let Some(end_run) = parse_node_id(end_run_id_ptr) else {
                return -2;
            };

            let range = DocRange {
                start: DocPosition {
                    run_id: start_run,
                    char_offset: start_offset as usize,
                },
                end: DocPosition {
                    run_id: end_run,
                    char_offset: end_offset as usize,
                },
            };

            let collapsed = start_run == end_run && start_offset == end_offset;
            let char_command = if collapsed {
                Command::SetCharFormat {
                    run_id: start_run,
                    start: start_offset as usize,
                    end: usize::MAX,
                    format: CharFormat::default(),
                    merge: false,
                }
            } else if start_run == end_run {
                Command::SetCharFormat {
                    run_id: start_run,
                    start: start_offset as usize,
                    end: end_offset as usize,
                    format: CharFormat::default(),
                    merge: false,
                }
            } else {
                Command::SetCharFormatRange {
                    range: range.clone(),
                    format: CharFormat::default(),
                    merge: false,
                }
            };

            let Some(_) = session.apply(char_command) else {
                return -4;
            };
            let Some(request_id) = session.apply(Command::SetParaFormatRange {
                range,
                format: ParaFormat::default(),
                merge: false,
            }) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_page_break(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.insert_page_break_at(caret_run_id) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

/// Paste sanitized HTML from the system clipboard at `[run_id, offset)`.
#[no_mangle]
pub extern "C" fn tw_apply_paste_html(
    run_id_ptr: *const c_char,
    offset: u32,
    html_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(run_id) = parse_node_id(run_id_ptr) else {
                return -2;
            };
            let Some(html) = parse_cstr(html_ptr) else {
                return -3;
            };
            let Some(request_id) = session.paste_html_at(run_id, offset as usize, html.into_bytes())
            else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

/// Paste a DOCX package from the clipboard at `[run_id, offset)`.
#[no_mangle]
pub extern "C" fn tw_apply_paste_docx(
    run_id_ptr: *const c_char,
    offset: u32,
    data: *const u8,
    len: usize,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(run_id) = parse_node_id(run_id_ptr) else {
                return -2;
            };
            if data.is_null() || len == 0 {
                return -3;
            }
            let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
            let Some(request_id) = session.paste_docx_at(run_id, offset as usize, bytes) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

/// Delete characters in a single run (`[start, end)`).
#[no_mangle]
pub extern "C" fn tw_apply_delete_range(
    run_id_ptr: *const c_char,
    start: u32,
    end: u32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(run_id) = parse_node_id(run_id_ptr) else {
                return -2;
            };
            if start >= end {
                return -3;
            }
            let Some(request_id) = session.apply(Command::DeleteRange {
                run_id,
                start: start as usize,
                end: end as usize,
            }) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

/// Delete characters across an arbitrary document range (possibly spanning runs).
#[no_mangle]
pub extern "C" fn tw_apply_delete_doc_range(
    start_run_id_ptr: *const c_char,
    start_offset: u32,
    end_run_id_ptr: *const c_char,
    end_offset: u32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(start_run_id) = parse_node_id(start_run_id_ptr) else {
                return -2;
            };
            let Some(end_run_id) = parse_node_id(end_run_id_ptr) else {
                return -2;
            };
            let Some(request_id) = session.apply(Command::DeleteDocRange {
                range: DocRange {
                    start: DocPosition {
                        run_id: start_run_id,
                        char_offset: start_offset as usize,
                    },
                    end: DocPosition {
                        run_id: end_run_id,
                        char_offset: end_offset as usize,
                    },
                },
            }) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

/// Split the paragraph at `(run_id, offset)` — Word Enter / Return.
#[no_mangle]
pub extern "C" fn tw_apply_split_paragraph(run_id_ptr: *const c_char, offset: u32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(run_id) = parse_node_id(run_id_ptr) else {
                return -2;
            };
            let Some(request_id) = session.apply(Command::SplitParagraphAt {
                run_id,
                offset: offset as usize,
            }) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_table(rows: u32, cols: u32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.insert_table(rows, cols) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_image(width: f32, height: f32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.insert_image(width, height) else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_undo() -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.undo() else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_redo() -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.redo() else {
                return -4;
            };
            wait_for_document_edit(session, request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_export_pdf(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.export_pdf() else {
                return -4;
            };
            wait_for_document_saved(session, request_id, out_ptr, out_len)
        })
    })
}

fn format_from_extension_str(ext: &str) -> tw_core::DetectedFormat {
    tw_core::format_from_extension(ext).unwrap_or(tw_core::DetectedFormat::Unknown)
}

#[no_mangle]
pub extern "C" fn tw_save_document_as(
    format_ptr: *const c_char,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let format_str = unsafe { CStr::from_ptr(format_ptr) }.to_string_lossy();
            let Some(request_id) = session.save_as(format_from_extension_str(&format_str)) else {
                return -4;
            };
            wait_for_document_saved(session, request_id, out_ptr, out_len)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_spell_check_document(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.spell_check() else {
                return -4;
            };
            wait_for_spell_check(session, request_id, out_ptr, out_len)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_set_track_changes(enabled: i32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            if session.set_track_changes(enabled != 0).is_some() {
                0
            } else {
                -4
            }
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_accept_all_revisions() -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            if session.accept_all_revisions().is_some() {
                0
            } else {
                -2
            }
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_reject_all_revisions() -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            if session.reject_all_revisions().is_some() {
                0
            } else {
                -2
            }
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_hit_test(
    page: u32,
    x: f32,
    y: f32,
    out_run_id: *mut c_char,
    run_id_cap: usize,
    out_offset: *mut u32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let Some(result) = session.hit_test(page, x, y) else {
            return -2;
        };
        let run_uuid = result.run_id.as_uuid().to_string();
        let bytes = run_uuid.as_bytes();
        if run_id_cap == 0 || out_run_id.is_null() {
            return -3;
        }
        let copy_len = bytes.len().min(run_id_cap.saturating_sub(1));
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_run_id as *mut u8, copy_len);
            *out_run_id.add(copy_len) = 0;
            if !out_offset.is_null() {
                *out_offset = result.char_offset as u32;
            }
        }
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_document_tail_hit(
    page: u32,
    out_run_id: *mut c_char,
    run_id_cap: usize,
    out_offset: *mut u32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let Some(result) = session.document_tail_hit(page) else {
            return -2;
        };
        let run_uuid = result.run_id.as_uuid().to_string();
        let bytes = run_uuid.as_bytes();
        if run_id_cap == 0 || out_run_id.is_null() {
            return -3;
        }
        let copy_len = bytes.len().min(run_id_cap.saturating_sub(1));
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_run_id as *mut u8, copy_len);
            *out_run_id.add(copy_len) = 0;
            if !out_offset.is_null() {
                *out_offset = result.char_offset as u32;
            }
        }
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_caret_geometry(
    page: u32,
    x: f32,
    y: f32,
    out_x: *mut f32,
    out_y: *mut f32,
    out_height: *mut f32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let Some(result) = session.hit_test(page, x, y) else {
            return -2;
        };
        if let Some((cx, cy, height)) = session.caret_geometry(page, x, y) {
            unsafe {
                if !out_x.is_null() {
                    *out_x = cx;
                }
                if !out_y.is_null() {
                    *out_y = cy;
                }
                if !out_height.is_null() {
                    *out_height = height;
                }
            }
        } else {
            unsafe {
                if !out_x.is_null() {
                    *out_x = x;
                }
                if !out_y.is_null() {
                    *out_y = y;
                }
                if !out_height.is_null() {
                    *out_height = 16.0;
                }
            }
        }
        let _ = result;
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_caret_at_position(
    page: u32,
    run_id_ptr: *const c_char,
    char_offset: u32,
    out_x: *mut f32,
    out_y: *mut f32,
    out_height: *mut f32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        if run_id_ptr.is_null() {
            return -2;
        }
        let run_id_str = unsafe { CStr::from_ptr(run_id_ptr) }.to_string_lossy();
        let Ok(uuid) = Uuid::parse_str(&run_id_str) else {
            return -2;
        };
        let run_id = NodeId::from_uuid(uuid);
        let Some((cx, cy, height)) = session.caret_at(page, run_id, char_offset as usize) else {
            return -3;
        };
        unsafe {
            if !out_x.is_null() {
                *out_x = cx;
            }
            if !out_y.is_null() {
                *out_y = cy;
            }
            if !out_height.is_null() {
                *out_height = height;
            }
        }
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_selection_rects(
    page: u32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    out_ptr: *mut f32,
    out_cap: u32,
    out_count: *mut u32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let rects = session.selection_rects(page, start_x, start_y, end_x, end_y);
        let count = (rects.len() / 4) as u32;
        unsafe {
            if !out_count.is_null() {
                *out_count = count;
            }
            if out_ptr.is_null() || out_cap == 0 {
                return 0;
            }
            let copy_len = rects.len().min(out_cap as usize);
            std::ptr::copy_nonoverlapping(rects.as_ptr(), out_ptr, copy_len);
        }
        0
    })
}

/// Block until the worker publishes a fresh layout snapshot (for select-all, copy, etc.).
#[no_mangle]
pub extern "C" fn tw_wait_for_layout() -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            match session.wait_for_event(BLOCKING_WAIT.as_millis() as u64) {
                Some(BridgeEvent::DisplayListReady { .. })
                | Some(BridgeEvent::DocumentOpened { .. }) => 0,
                Some(BridgeEvent::Error { .. }) => -2,
                None => -3,
                _ => 0,
            }
        })
    })
}

#[cfg(test)]
mod buffer_tests {
    use super::{transfer_bytes_to_caller, tw_free_buffer};

    #[test]
    fn transfer_and_free_roundtrip() {
        let payload = b"hello-ffi-buffer".to_vec();
        let mut out_ptr: *const u8 = std::ptr::null();
        let mut out_len = 0usize;
        transfer_bytes_to_caller(payload.clone(), &mut out_ptr as *mut _, &mut out_len);
        assert_eq!(out_len, payload.len());
        let recovered = unsafe { std::slice::from_raw_parts(out_ptr, out_len) };
        assert_eq!(recovered, payload.as_slice());
        tw_free_buffer(out_ptr as *mut u8, out_len);
    }

    #[test]
    fn transfer_and_free_with_excess_capacity() {
        let mut vec = Vec::with_capacity(128);
        vec.extend_from_slice(b"capacity-greater-than-len");
        let mut out_ptr: *const u8 = std::ptr::null();
        let mut out_len = 0usize;
        transfer_bytes_to_caller(vec, &mut out_ptr as *mut _, &mut out_len);
        let recovered = unsafe { std::slice::from_raw_parts(out_ptr, out_len) };
        assert_eq!(recovered, b"capacity-greater-than-len");
        tw_free_buffer(out_ptr as *mut u8, out_len);
    }
}
