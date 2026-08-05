use parking_lot::Mutex;
use std::ffi::{c_char, CStr};
use std::slice;
use std::time::Duration;
use tw_core::Session;
use tw_edit::{Command, DocPosition, DocRange};
use tw_model::{CharFormat, NodeId, ParaFormat};
use uuid::Uuid;

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

type EventCallback = extern "C" fn(event_type: u32, data: *const u8, len: usize);

static mut EVENT_CALLBACK: Option<EventCallback> = None;

fn wait_for_document(session: &Session) -> i32 {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Some(event) = session.poll_event() {
            match event {
                tw_core::BridgeEvent::DocumentOpened { .. } => return 0,
                tw_core::BridgeEvent::DisplayListReady { .. } => return 0,
                tw_core::BridgeEvent::Error { .. } => return -2,
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    -3
}

#[no_mangle]
pub extern "C" fn tw_init(callback: EventCallback) -> i32 {
    unsafe { EVENT_CALLBACK = Some(callback); }
    let mut guard = SESSION.lock();
    *guard = Some(Session::new());
    0
}

#[no_mangle]
pub extern "C" fn tw_shutdown() {
    let mut guard = SESSION.lock();
    *guard = None;
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

    session.apply(Command::InsertText {
        run_id,
        offset: offset as usize,
        text,
    });
    wait_for_document(session)
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
pub extern "C" fn tw_save_document(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.save();

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Some(event) = session.poll_event() {
            if let tw_core::BridgeEvent::DocumentSaved { data } = event {
                let leaked = data.clone();
                unsafe {
                    *out_ptr = leaked.as_ptr();
                    *out_len = leaked.len();
                }
                std::mem::forget(leaked);
                return 0;
            }
            if let tw_core::BridgeEvent::Error { .. } = event {
                return -2;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    -3
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
    session.open_bytes_with_path(bytes, path_hint);
    wait_for_document(session)
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
    session.set_current_page(page);
    wait_for_document(session)
}

#[no_mangle]
pub extern "C" fn tw_apply_heading1() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.apply_heading1();
    wait_for_document(session)
}

#[no_mangle]
pub extern "C" fn tw_apply_bullet_list() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.apply_bullet_list();
    wait_for_document(session)
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
/// When start and end describe the same collapsed caret, the whole run at
/// `start_run_id` is formatted.
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
    let format: CharFormat = match serde_json::from_str(&json) {
        Ok(f) => f,
        Err(_) => return -3,
    };

    let collapsed = start_run == end_run && start_offset == end_offset;
    let command = if collapsed {
        // Format the entire run containing the caret.
        Command::SetCharFormat {
            run_id: start_run,
            start: 0,
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

    session.apply(command);
    wait_for_document(session)
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

    session.apply(Command::SetParaFormatRange {
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
    });
    wait_for_document(session)
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
    session.apply(Command::DeleteRange {
        run_id,
        start: start as usize,
        end: end as usize,
    });
    wait_for_document(session)
}

#[no_mangle]
pub extern "C" fn tw_insert_table(rows: u32, cols: u32) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.insert_table(rows, cols);
    wait_for_document(session)
}

#[no_mangle]
pub extern "C" fn tw_insert_image(width: f32, height: f32) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.insert_image(width, height);
    wait_for_document(session)
}

#[no_mangle]
pub extern "C" fn tw_undo() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.undo();
    wait_for_document(session)
}

#[no_mangle]
pub extern "C" fn tw_redo() -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.redo();
    wait_for_document(session)
}

#[no_mangle]
pub extern "C" fn tw_export_pdf(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.export_pdf();

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Some(event) = session.poll_event() {
            if let tw_core::BridgeEvent::DocumentSaved { data } = event {
                let leaked = data.clone();
                unsafe {
                    *out_ptr = leaked.as_ptr();
                    *out_len = leaked.len();
                }
                std::mem::forget(leaked);
                return 0;
            }
            if let tw_core::BridgeEvent::Error { .. } = event {
                return -2;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    -3
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
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    let format_str = unsafe { CStr::from_ptr(format_ptr) }.to_string_lossy();
    session.save_as(format_from_extension_str(&format_str));

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Some(event) = session.poll_event() {
            if let tw_core::BridgeEvent::DocumentSaved { data } = event {
                let leaked = data.clone();
                unsafe {
                    *out_ptr = leaked.as_ptr();
                    *out_len = leaked.len();
                }
                std::mem::forget(leaked);
                return 0;
            }
            if let tw_core::BridgeEvent::Error { .. } = event {
                return -2;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    -3
}

#[no_mangle]
pub extern "C" fn tw_spell_check_document(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    let guard = SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return -1;
    };
    session.spell_check();

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Some(event) = session.poll_event() {
            if let tw_core::BridgeEvent::SpellCheckResult { misspellings } = event {
                let text = misspellings.join("\n");
                let leaked = text.into_bytes();
                unsafe {
                    *out_ptr = leaked.as_ptr();
                    *out_len = leaked.len();
                }
                std::mem::forget(leaked);
                return 0;
            }
            if let tw_core::BridgeEvent::Error { .. } = event {
                return -2;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    -3
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
