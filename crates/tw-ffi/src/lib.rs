use parking_lot::Mutex;
use serde::Deserialize;
use std::ffi::{c_char, CStr};
use std::slice;
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use tw_core::Session;
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

static mut EVENT_CALLBACK: Option<EventCallback> = None;

const EDIT_WAIT: Duration = Duration::from_millis(100);
const BLOCKING_WAIT: Duration = Duration::from_secs(30);

fn poll_session_event() -> Option<tw_core::BridgeEvent> {
    let guard = SESSION.lock();
    guard.as_ref().and_then(|session| session.poll_event())
}

fn wait_for_events(timeout: Duration, mut on_event: impl FnMut(tw_core::BridgeEvent) -> Option<i32>) -> i32 {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if let Some(event) = poll_session_event() {
            if let tw_core::BridgeEvent::Error { message } = &event {
                if let Ok(mut guard) = LAST_ERROR.lock() {
                    *guard = Some(message.clone());
                }
            }
            if let Some(code) = on_event(event) {
                return code;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    -3
}

fn wait_for_document_ready() -> i32 {
    wait_for_events(BLOCKING_WAIT, |event| match event {
        tw_core::BridgeEvent::DocumentOpened { .. } | tw_core::BridgeEvent::DisplayListReady { .. } => {
            Some(0)
        }
        tw_core::BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

/// Brief wait on edit hot paths; return success if the worker is still catching up.
fn wait_for_document_edit() -> i32 {
    match wait_for_events(EDIT_WAIT, |event| match event {
        tw_core::BridgeEvent::DisplayListReady { .. } => Some(0),
        tw_core::BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    }) {
        -3 => 0,
        code => code,
    }
}

fn wait_for_document_saved(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    wait_for_events(BLOCKING_WAIT, |event| match event {
        tw_core::BridgeEvent::DocumentSaved { data } => {
            let leaked = data.clone();
            unsafe {
                *out_ptr = leaked.as_ptr();
                *out_len = leaked.len();
            }
            std::mem::forget(leaked);
            Some(0)
        }
        tw_core::BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

fn wait_for_spell_check(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    wait_for_events(BLOCKING_WAIT, |event| match event {
        tw_core::BridgeEvent::SpellCheckResult { misspellings } => {
            let text = misspellings.join("\n");
            let leaked = text.into_bytes();
            unsafe {
                *out_ptr = leaked.as_ptr();
                *out_len = leaked.len();
            }
            std::mem::forget(leaked);
            Some(0)
        }
        tw_core::BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

#[no_mangle]
pub extern "C" fn tw_init(callback: EventCallback) -> i32 {
    unsafe { EVENT_CALLBACK = Some(callback); }
    {
        let mut guard = SESSION.lock();
        *guard = Some(Session::new());
    }
    // Wait for the worker's initial empty document so the first display-list
    // fetch is not racing an unpublished snapshot.
    wait_for_document_ready()
}

#[no_mangle]
pub extern "C" fn tw_shutdown() {
    let mut guard = SESSION.lock();
    *guard = None;
}

#[no_mangle]
pub extern "C" fn tw_new_document() -> i32 {
    let enqueued = {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        session.new_document()
    };
    if !enqueued {
        return -4;
    }
    wait_for_document_ready()
}

#[no_mangle]
pub extern "C" fn tw_apply_insert_text(
    run_id_ptr: *const c_char,
    offset: u32,
    text_ptr: *const c_char,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };

    let run_id_str = unsafe { CStr::from_ptr(run_id_ptr) }.to_string_lossy();
    let text = unsafe { CStr::from_ptr(text_ptr) }.to_string_lossy().into_owned();
    let run_id = NodeId::from_uuid(Uuid::parse_str(&run_id_str).unwrap_or_else(|_| Uuid::new_v4()));

    if !session.apply(Command::InsertText {
        run_id,
        offset: offset as usize,
        text,
    }) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
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
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };

    let snapshot = session.get_display_list_bytes();
    let leaked = snapshot.bytes.clone();
    unsafe {
        *out_ptr = leaked.as_ptr();
        *out_len = leaked.len();
        *out_version = snapshot.version;
        *out_width = snapshot.page_width;
        *out_height = snapshot.page_height;
        *out_page_count = snapshot.page_count;
    }
    std::mem::forget(leaked);
    0
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
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };

    let Some(snapshot) = session.page_display_list(page) else {
        return -2;
    };

    let leaked = snapshot.bytes.clone();
    unsafe {
        *out_ptr = leaked.as_ptr();
        *out_len = leaked.len();
        *out_width = snapshot.page_width;
        *out_height = snapshot.page_height;
    }
    std::mem::forget(leaked);
    0
}

#[no_mangle]
pub extern "C" fn tw_get_last_error(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    let guard = match LAST_ERROR.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(message) = guard.as_ref() else {
        return -1;
    };
    let leaked = message.clone().into_bytes();
    unsafe {
        *out_ptr = leaked.as_ptr();
        *out_len = leaked.len();
    }
    std::mem::forget(leaked);
    0
}

#[no_mangle]
pub extern "C" fn tw_get_document_properties_json(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };

    let json = session.get_display_list_bytes().document_properties_json;
    let leaked = json.into_bytes();
    unsafe {
        *out_ptr = leaked.as_ptr();
        *out_len = leaked.len();
    }
    std::mem::forget(leaked);
    0
}

#[no_mangle]
pub extern "C" fn tw_is_document_read_only() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if session.get_display_list_bytes().read_only {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn tw_get_document_text(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };

    let text = session.get_display_list_bytes().document_text;
    let leaked = text.into_bytes();
    unsafe {
        *out_ptr = leaked.as_ptr();
        *out_len = leaked.len();
    }
    std::mem::forget(leaked);
    0
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
    let leaked = text.into_bytes();
    unsafe {
        *out_ptr = leaked.as_ptr();
        *out_len = leaked.len();
    }
    std::mem::forget(leaked);
    0
}

#[no_mangle]
pub extern "C" fn tw_save_document(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    let enqueued = {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        session.save()
    };
    if !enqueued {
        return -4;
    }
    wait_for_document_saved(out_ptr, out_len)
}

#[no_mangle]
pub extern "C" fn tw_open_document(data: *const u8, len: usize) -> i32 {
    tw_open_document_with_path(data, len, std::ptr::null())
}

#[no_mangle]
pub extern "C" fn tw_open_document_with_path(
    data: *const u8,
    len: usize,
    path_ptr: *const c_char,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
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
    if !session.open_bytes_with_path(bytes, path_hint) {
        return -4;
    }
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = None;
    }
    drop(guard);
    wait_for_document_ready()
}

#[no_mangle]
pub extern "C" fn tw_free_buffer(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        let _ = Vec::from_raw_parts(ptr, len, len);
    }
}

#[no_mangle]
pub extern "C" fn tw_set_current_page(page: u32) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if !session.set_current_page(page) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_apply_heading1(caret_run_id_ptr: *const c_char) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let caret_run_id = parse_node_id(caret_run_id_ptr);
    if !session.apply_heading1_at(caret_run_id) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_apply_normal_style(caret_run_id_ptr: *const c_char) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let caret_run_id = parse_node_id(caret_run_id_ptr);
    if !session.apply_normal_style_at(caret_run_id) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_apply_numbered_list(caret_run_id_ptr: *const c_char) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let caret_run_id = parse_node_id(caret_run_id_ptr);
    if !session.apply_numbered_list_at(caret_run_id) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_apply_bullet_list(caret_run_id_ptr: *const c_char) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let caret_run_id = parse_node_id(caret_run_id_ptr);
    if !session.apply_bullet_list_at(caret_run_id) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
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

    if patch.clear_color || patch.clear_highlight {
        if !session.apply(Command::ClearCharFormatFields {
            range: range.clone(),
            clear_color: patch.clear_color,
            clear_highlight: patch.clear_highlight,
        }) {
            return -4;
        }
    }

    let has_format_fields = serde_json::from_str::<serde_json::Value>(&json)
        .ok()
        .is_some_and(|value| {
            value
                .as_object()
                .is_some_and(|obj| obj.keys().any(|k| k != "clear_color" && k != "clear_highlight"))
        });
    if !has_format_fields {
        drop(guard);
        return wait_for_document_edit();
    }

    let command = if collapsed {
        // Format from the caret to the end of the run — avoids restyling text
        // before the caret when toggling ribbon buttons with a collapsed selection.
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

    if !session.apply(command) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
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
    let Some(json) = parse_cstr(format_json_ptr) else {
        return -3;
    };
    let format: ParaFormat = match serde_json::from_str(&json) {
        Ok(f) => f,
        Err(_) => return -3,
    };

    if !session.apply(Command::SetParaFormatRange {
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
    }) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_get_caret_format(
    run_id_ptr: *const c_char,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
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
    let leaked = json.into_bytes();
    unsafe {
        *out_ptr = leaked.as_ptr();
        *out_len = leaked.len();
    }
    std::mem::forget(leaked);
    0
}

/// Remove direct character and paragraph formatting for the given range.
#[no_mangle]
pub extern "C" fn tw_clear_format(
    start_run_id_ptr: *const c_char,
    start_offset: u32,
    end_run_id_ptr: *const c_char,
    end_offset: u32,
) -> i32 {
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

    if !session.apply(char_command) {
        return -4;
    }
    if !session.apply(Command::SetParaFormatRange {
        range,
        format: ParaFormat::default(),
        merge: false,
    }) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_insert_page_break(caret_run_id_ptr: *const c_char) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let caret_run_id = parse_node_id(caret_run_id_ptr);
    if !session.insert_page_break_at(caret_run_id) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

/// Paste sanitized HTML from the system clipboard at `[run_id, offset)`.
#[no_mangle]
pub extern "C" fn tw_apply_paste_html(
    run_id_ptr: *const c_char,
    offset: u32,
    html_ptr: *const c_char,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let Some(run_id) = parse_node_id(run_id_ptr) else {
        return -2;
    };
    let Some(html) = parse_cstr(html_ptr) else {
        return -3;
    };
    if !session.paste_html_at(run_id, offset as usize, html.into_bytes()) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

/// Paste a DOCX package from the clipboard at `[run_id, offset)`.
#[no_mangle]
pub extern "C" fn tw_apply_paste_docx(
    run_id_ptr: *const c_char,
    offset: u32,
    data: *const u8,
    len: usize,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let Some(run_id) = parse_node_id(run_id_ptr) else {
        return -2;
    };
    if data.is_null() || len == 0 {
        return -3;
    }
    let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
    if !session.paste_docx_at(run_id, offset as usize, bytes) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

/// Delete characters in a single run (`[start, end)`).
#[no_mangle]
pub extern "C" fn tw_apply_delete_range(
    run_id_ptr: *const c_char,
    start: u32,
    end: u32,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let Some(run_id) = parse_node_id(run_id_ptr) else {
        return -2;
    };
    if start >= end {
        return -3;
    }
    if !session.apply(Command::DeleteRange {
        run_id,
        start: start as usize,
        end: end as usize,
    }) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

/// Delete characters across an arbitrary document range (possibly spanning runs).
#[no_mangle]
pub extern "C" fn tw_apply_delete_doc_range(
    start_run_id_ptr: *const c_char,
    start_offset: u32,
    end_run_id_ptr: *const c_char,
    end_offset: u32,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let Some(start_run_id) = parse_node_id(start_run_id_ptr) else {
        return -2;
    };
    let Some(end_run_id) = parse_node_id(end_run_id_ptr) else {
        return -2;
    };
    if !session.apply(Command::DeleteDocRange {
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
    }) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

/// Split the paragraph at `(run_id, offset)` — Word Enter / Return.
#[no_mangle]
pub extern "C" fn tw_apply_split_paragraph(run_id_ptr: *const c_char, offset: u32) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let Some(run_id) = parse_node_id(run_id_ptr) else {
        return -2;
    };
    if !session.apply(Command::SplitParagraphAt {
        run_id,
        offset: offset as usize,
    }) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_insert_table(rows: u32, cols: u32) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if !session.insert_table(rows, cols) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_insert_image(width: f32, height: f32) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if !session.insert_image(width, height) {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_undo() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if !session.undo() {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_redo() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if !session.redo() {
        return -4;
    }
    drop(guard);
    wait_for_document_edit()
}

#[no_mangle]
pub extern "C" fn tw_export_pdf(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    let enqueued = {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        session.export_pdf()
    };
    if !enqueued {
        return -4;
    }
    wait_for_document_saved(out_ptr, out_len)
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
    let enqueued = {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let format_str = unsafe { CStr::from_ptr(format_ptr) }.to_string_lossy();
        session.save_as(format_from_extension_str(&format_str))
    };
    if !enqueued {
        return -4;
    }
    wait_for_document_saved(out_ptr, out_len)
}

#[no_mangle]
pub extern "C" fn tw_spell_check_document(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    let enqueued = {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        session.spell_check()
    };
    if !enqueued {
        return -4;
    }
    wait_for_spell_check(out_ptr, out_len)
}

#[no_mangle]
pub extern "C" fn tw_set_track_changes(enabled: i32) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.set_track_changes(enabled != 0);
    0
}

#[no_mangle]
pub extern "C" fn tw_accept_all_revisions() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if session.accept_all_revisions() {
        0
    } else {
        -2
    }
}

#[no_mangle]
pub extern "C" fn tw_reject_all_revisions() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    if session.reject_all_revisions() {
        0
    } else {
        -2
    }
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
}

#[no_mangle]
pub extern "C" fn tw_document_tail_hit(
    page: u32,
    out_run_id: *mut c_char,
    run_id_cap: usize,
    out_offset: *mut u32,
) -> i32 {
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
}

/// Block until the worker publishes a fresh layout snapshot (for select-all, copy, etc.).
#[no_mangle]
pub extern "C" fn tw_wait_for_layout() -> i32 {
    wait_for_document_ready()
}
