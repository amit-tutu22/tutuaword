use parking_lot::Mutex;
use serde::Deserialize;
use std::collections::VecDeque;
use std::ffi::{c_char, CStr};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::Duration;
use tw_core::{BridgeEvent, Session, WaitOutcome, STARTUP_REQUEST_ID};
use tw_edit::{command_from_json, Command, DocPosition, DocRange};
use tw_layout::FontFaceSpec;
use tw_model::{CharFormat, FieldType, NodeId, ParaFormat, SectionFormat};
use uuid::Uuid;

static SESSION: Mutex<Option<Session>> = Mutex::new(None);
static LAST_ERROR: StdMutex<Option<String>> = StdMutex::new(None);

/// Host event callback.
///
/// `event_type` and `request_id` are passed **by value** and are the authoritative
/// copy: a host must never have to read the payload to correlate a request. This
/// is deliberate. A host that defers the callback to a later turn of its own event
/// loop (Dart's `NativeCallable.listener`, for instance) will find `payload`
/// dangling, because it points at a stack buffer belonging to the frame that made
/// the call.
///
/// `payload` is valid only for the duration of the call and must be copied before
/// returning if it is needed afterwards. It currently holds the same two scalars
/// in the 12-byte wire encoding (LE `u32` type, LE `u64` request id) and exists so
/// future events can carry inline data without another ABI break.
type EventCallback = extern "C" fn(
    event_type: u32,
    request_id: u64,
    payload: *const u8,
    payload_len: usize,
);

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
static LAST_REQUEST_ID: AtomicU64 = AtomicU64::new(0);

/// Minimal wire format event types (LE u32 in callback payload bytes 0..4).
pub const TW_EVENT_DISPLAY_LIST_READY: u32 = 1;
pub const TW_EVENT_DOCUMENT_OPENED: u32 = 2;
pub const TW_EVENT_DOCUMENT_SAVED: u32 = 3;
pub const TW_EVENT_SPELL_CHECK_RESULT: u32 = 4;
pub const TW_EVENT_ERROR: u32 = 5;

const EVENT_WIRE_BYTES: usize = 12;

const BLOCKING_WAIT: Duration = Duration::from_secs(30);
/// Returned when an exported function panics across the FFI boundary.
const FFI_PANIC: i32 = -99;

/// `tw_hit_test` refused because the page still carries pre-edit geometry.
/// Distinct from `-2`, which means the page is current and nothing was under the
/// point. Only applies to queries that read a page's line map.
const HIT_PAGE_STALE: i32 = -4;

fn record_last_error(message: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(message);
    }
}

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

fn forward_event_to_host(event: &BridgeEvent) {
    let Some(callback) = EVENT_CALLBACK.get() else {
        return;
    };
    let event_type = bridge_event_type(event);
    let wire = encode_event_wire(event);
    callback(event_type, event.request_id(), wire.as_ptr(), wire.len());
}

/// Events collected under the engine locks, waiting to be handed to the host.
///
/// The host callback must never run while `SESSION` is held. A host calling back
/// into any export from its event callback is entirely reasonable, and `SESSION`
/// is a non-reentrant `parking_lot` mutex, so doing that under the lock would
/// deadlock the caller. Everything the observer sees therefore lands here and is
/// drained by [`flush_event_outbox`] once the lock is released.
static EVENT_OUTBOX: Mutex<VecDeque<BridgeEvent>> = Mutex::new(VecDeque::new());

/// Cap on undelivered events, matching the session's own correlation buffer. Only
/// reachable if the host stops flushing entirely, in which case it is not
/// listening anyway; async results are already recorded and survive the drop.
const MAX_OUTBOX_EVENTS: usize = 1024;

fn install_event_observer(session: &Session) {
    // Runs with `SESSION` held: bookkeeping only, no host code, no re-entry.
    session.set_event_observer(Arc::new(|event| {
        ASYNC_RESULTS.lock().record(&event);
        let mut outbox = EVENT_OUTBOX.lock();
        outbox.push_back(event);
        while outbox.len() > MAX_OUTBOX_EVENTS {
            outbox.pop_front();
        }
    }));
}

/// Hand collected events to the host. Callers **must** have released `SESSION`
/// first. Returns the number delivered.
fn flush_event_outbox() -> usize {
    let events = {
        let mut outbox = EVENT_OUTBOX.lock();
        std::mem::take(&mut *outbox)
    };
    for event in &events {
        forward_event_to_host(event);
    }
    events.len()
}

/// Run `f` under the session lock, then deliver whatever it produced. The two
/// steps are separate so the host callback never sees a held lock.
fn with_session_then_flush<F: FnOnce(&Session) -> i32>(f: F) -> i32 {
    let code = with_session(f);
    flush_event_outbox();
    code
}

/// Async result slots kept alive between an enqueue and its getter (P1-7).
///
/// Dart enqueues on the UI isolate, learns the outcome from the event callback,
/// and only then fetches the payload, so the worker's completion has to survive
/// until the getter runs. Slots are bounded and evicted oldest-first: an
/// abandoned request costs at most one saved document until it ages out.
const MAX_ASYNC_RESULTS: usize = 16;

enum AsyncResult {
    Pending,
    Done(Vec<u8>),
    Failed,
}

struct AsyncResultStore {
    slots: VecDeque<(u64, AsyncResult)>,
}

impl AsyncResultStore {
    fn slot(&mut self, request_id: u64) -> Option<&mut AsyncResult> {
        self.slots
            .iter_mut()
            .find(|(id, _)| *id == request_id)
            .map(|(_, result)| result)
    }

    fn clear(&mut self) {
        self.slots.clear();
    }

    fn track(&mut self, request_id: u64) {
        self.slots.retain(|(id, _)| *id != request_id);
        while self.slots.len() >= MAX_ASYNC_RESULTS {
            self.slots.pop_front();
        }
        self.slots.push_back((request_id, AsyncResult::Pending));
    }

    /// Fill the slot for a tracked request; untracked ids (edits, background
    /// reflow) are ignored so they cannot evict a result Dart is waiting on.
    fn record(&mut self, event: &BridgeEvent) {
        let result = match event {
            BridgeEvent::DocumentOpened { .. } => AsyncResult::Done(Vec::new()),
            BridgeEvent::DocumentSaved { data, .. } => AsyncResult::Done(data.clone()),
            BridgeEvent::SpellCheckResult { misspellings, .. } => {
                AsyncResult::Done(misspellings.join("\n").into_bytes())
            }
            BridgeEvent::Error { .. } => AsyncResult::Failed,
            BridgeEvent::DisplayListReady { .. } => return,
        };
        if let Some(slot) = self.slot(event.request_id()) {
            if matches!(slot, AsyncResult::Pending) {
                *slot = result;
            }
        }
    }

    /// `None` = never enqueued or already taken; a settled slot is removed.
    fn take(&mut self, request_id: u64) -> Option<AsyncResult> {
        let index = self.slots.iter().position(|(id, _)| *id == request_id)?;
        if matches!(self.slots[index].1, AsyncResult::Pending) {
            return Some(AsyncResult::Pending);
        }
        self.slots.remove(index).map(|(_, result)| result)
    }
}

static ASYNC_RESULTS: Mutex<AsyncResultStore> = Mutex::new(AsyncResultStore {
    slots: VecDeque::new(),
});

/// Enqueue `command` and hand the correlation id straight back to Dart.
fn enqueue_async(out_request_id: *mut u64, enqueue: impl FnOnce(&Session) -> Option<u64>) -> i32 {
    if out_request_id.is_null() {
        return -2;
    }
    with_session(|session| {
        let Some(request_id) = enqueue(session) else {
            return -4;
        };
        ASYNC_RESULTS.lock().track(request_id);
        unsafe {
            *out_request_id = request_id;
        }
        0
    })
}

/// Shared getter body: 0 = ready, 1 = still pending, -2 = failed, -3 = unknown id.
fn take_async_result(
    request_id: u64,
    deliver: impl FnOnce(Vec<u8>),
) -> i32 {
    with_session_then_flush(|session| {
        session.pump_events();
        match ASYNC_RESULTS.lock().take(request_id) {
            None => -3,
            Some(AsyncResult::Pending) => 1,
            Some(AsyncResult::Failed) => -2,
            Some(AsyncResult::Done(bytes)) => {
                deliver(bytes);
                0
            }
        }
    })
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

fn wait_for_startup(session: &Session, timeout: Duration) -> i32 {
    let code = wait_for_request(session, STARTUP_REQUEST_ID, timeout, |event| match event {
        BridgeEvent::DocumentOpened { .. } | BridgeEvent::DisplayListReady { .. } => Some(0),
        BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    });
  // The worker may still be publishing startup while the host attaches its pump.
    if code == -3 {
        0
    } else {
        code
    }
}

fn wait_for_open(session: &Session, request_id: u64) -> i32 {
    wait_for_request(session, request_id, BLOCKING_WAIT, |event| match event {
        BridgeEvent::DocumentOpened { .. } => Some(0),
        BridgeEvent::Error { .. } => Some(-2),
        _ => None,
    })
}

/// Fire-and-forget edit enqueue (R1.4): record [request_id] for Dart correlation.
fn finish_edit_enqueue(request_id: u64) -> i32 {
    LAST_REQUEST_ID.store(request_id, Ordering::Relaxed);
    0
}

/// Apply a deserialized [`Command`] on the worker thread (R2.5 single edit path).
fn apply_command(command: Command) -> i32 {
    with_session(|session| {
        let Some(request_id) = session.apply(command) else {
            return -4;
        };
        finish_edit_enqueue(request_id)
    })
}

/// Deserialize JSON command bytes and enqueue (R2.5 consolidated edit dispatch).
fn dispatch_command_bytes(bytes: &[u8]) -> i32 {
    match command_from_json(bytes) {
        Ok(command) => apply_command(command),
        Err(_) => -3,
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

/// Create the session and register the event callback.
///
/// # The host must pump
///
/// **Registering `callback` here is not enough to receive events.** `callback`
/// fires only while the FFI layer drains the worker's event channel, and nothing
/// drains it spontaneously: `tw_dispatch` and the other edit exports enqueue a
/// command and return, and the worker pushes its completion into a channel that
/// sits there until somebody reads it.
///
/// A host that never calls [`tw_pump_events`] therefore sees no events at all,
/// and every correlated wait runs to its full 30 s timeout. The failure looks
/// like a hang in an unrelated feature -- select-all timing out, say -- because
/// the only thing that used to drain the channel was whichever blocking export
/// happened to be on the stack.
///
/// Call `tw_pump_events` on a timer or frame callback for the lifetime of the
/// session. Something in the low milliseconds while a correlated request is
/// outstanding and ~16 ms when idle keeps background reflow repaints flowing
/// without spinning.
#[no_mangle]
pub extern "C" fn tw_init(callback: EventCallback) -> i32 {
    guard_ffi(|| {
        let _ = EVENT_CALLBACK.set(callback);
        {
            let mut guard = SESSION.lock();
            let session = Session::new();
            install_event_observer(&session);
            *guard = Some(session);
            ASYNC_RESULTS.lock().clear();
            EVENT_OUTBOX.lock().clear();
        }
        // Startup layout runs on the worker thread; call [`tw_await_startup`] after
        // registering fonts on mobile hosts.
        0
    })
}

/// Wait until the worker's startup document is laid out. Call after a
/// non-blocking [`tw_init`]. `timeout_ms` is capped at 30s. Returns `0` when
/// ready, `-1` without a session, `-2` on worker error. A timeout is treated as
/// success so a slow first layout does not fail initialization.
#[no_mangle]
pub extern "C" fn tw_await_startup(timeout_ms: u32) -> i32 {
    guard_ffi(|| {
        let ms = timeout_ms.min(30000) as u64;
        with_session_then_flush(|session| wait_for_startup(session, Duration::from_millis(ms)))
    })
}

#[no_mangle]
pub extern "C" fn tw_shutdown() {
    guard_ffi_void(|| {
        let mut guard = SESSION.lock();
        *guard = None;
        // Request ids restart with the next session, so results left over from
        // this one would otherwise collide with fresh ids.
        ASYNC_RESULTS.lock().clear();
        EVENT_OUTBOX.lock().clear();
    });
}

/// Register a font face from raw bytes before opening documents.
///
/// `family` is matched case-insensitively against document font requests.
/// Returns `0` on success.
#[no_mangle]
pub extern "C" fn tw_register_font(
    family_ptr: *const c_char,
    bold: bool,
    italic: bool,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    guard_ffi(|| {
        let Some(family) = parse_cstr(family_ptr) else {
            return -2;
        };
        if data_ptr.is_null() || data_len == 0 {
            return -3;
        }
        let data = unsafe { slice::from_raw_parts(data_ptr, data_len) }.to_vec();
        let mut spec = FontFaceSpec::new(family);
        if bold {
            spec = spec.bold();
        }
        if italic {
            spec = spec.italic();
        }
        with_session(|session| match session.register_face(&spec, data) {
            Ok(_) => 0,
            Err(e) => {
                record_last_error(e.to_string());
                -4
            }
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_new_document() -> i32 {
    guard_ffi(|| {
        with_session_then_flush(|session| {
            let Some(request_id) = session.new_document() else {
                return -4;
            };
            wait_for_open(session, request_id)
        })
    })
}

/// JSON command dispatch for all document edits (R2.5).
///
/// `command_ptr`/`command_len` must contain a serialized [`Command`] JSON object.
/// Returns `0` on enqueue success; use `tw_last_request_id()` for async correlation.
#[no_mangle]
pub extern "C" fn tw_dispatch(command_ptr: *const u8, command_len: usize) -> i32 {
    guard_ffi(|| {
        if command_ptr.is_null() || command_len == 0 {
            return -3;
        }
        let bytes = unsafe { slice::from_raw_parts(command_ptr, command_len) };
        dispatch_command_bytes(bytes)
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_insert_text(
    run_id_ptr: *const c_char,
    offset: u32,
    text_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        let Some(run_id) = parse_node_id(run_id_ptr) else {
            return -2;
        };
        let Some(text) = parse_cstr(text_ptr) else {
            return -3;
        };
        apply_command(Command::InsertText {
            run_id,
            offset: offset as usize,
            text,
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
        transfer_bytes_to_caller(snapshot.bytes.as_ref().clone(), out_ptr, out_len);
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
    out_version: *mut u64,
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
            if !out_version.is_null() {
                *out_version = snapshot.version;
            }
            *out_width = snapshot.page_width;
            *out_height = snapshot.page_height;
        }
        0
    })
}

/// Current atlas generation without cloning the pixel buffer, so a caller can
/// decide whether `tw_get_atlas` is worth paying for.
///
/// 0 = generation written, -1 = no session, -2 = null out pointer.
#[no_mangle]
pub extern "C" fn tw_get_atlas_generation(out_generation: *mut u64) -> i32 {
    guard_ffi(|| {
        if out_generation.is_null() {
            return -2;
        }
        with_session(|session| {
            unsafe {
                *out_generation = session.atlas_generation();
            }
            0
        })
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

        let json = session.get_display_list_bytes().document_properties_json.clone();
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

        let text = session.get_display_list_bytes().document_text.clone();
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

/// Blocking wrapper over [`tw_save_document_async`] + [`tw_take_saved_document`].
#[no_mangle]
pub extern "C" fn tw_save_document(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    guard_ffi(|| {
        with_session_then_flush(|session| {
            let Some(request_id) = session.save() else {
                return -4;
            };
            wait_for_document_saved(session, request_id, out_ptr, out_len)
        })
    })
}

/// Enqueue a save and return immediately with the correlation id. The bytes are
/// held until [`tw_take_saved_document`] collects them.
///
/// 0 = enqueued, -1 = no session, -2 = null out pointer, -4 = worker unavailable.
#[no_mangle]
pub extern "C" fn tw_save_document_async(out_request_id: *mut u64) -> i32 {
    guard_ffi(|| enqueue_async(out_request_id, |session| session.save()))
}

/// Enqueue a save in `format` and return the correlation id. Collected with
/// [`tw_take_saved_document`].
#[no_mangle]
pub extern "C" fn tw_save_document_as_async(
    format_ptr: *const c_char,
    out_request_id: *mut u64,
) -> i32 {
    guard_ffi(|| {
        if format_ptr.is_null() {
            return -2;
        }
        let format_str = unsafe { CStr::from_ptr(format_ptr) }.to_string_lossy().into_owned();
        enqueue_async(out_request_id, |session| {
            session.save_as(format_from_extension_str(&format_str))
        })
    })
}

/// Collect the bytes produced by a save enqueued with `tw_save_document_async`
/// or `tw_save_document_as_async`. The buffer must be released with
/// `tw_free_buffer`.
///
/// 0 = ready (out params written), 1 = not ready yet (call again after the
/// `DocumentSaved` event), -1 = no session, -2 = the save failed (details via
/// `tw_get_last_error`), -3 = unknown request id: never enqueued, already
/// collected, or evicted after too many outstanding requests.
#[no_mangle]
pub extern "C" fn tw_take_saved_document(
    request_id: u64,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        if out_ptr.is_null() || out_len.is_null() {
            return -2;
        }
        take_async_result(request_id, |bytes| {
            transfer_bytes_to_caller(bytes, out_ptr, out_len)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_open_document(data: *const u8, len: usize) -> i32 {
    guard_ffi(|| tw_open_document_with_path(data, len, std::ptr::null()))
}

/// Blocking wrapper over [`tw_open_document_async`] + [`tw_take_open_result`].
#[no_mangle]
pub extern "C" fn tw_open_document_with_path(
    data: *const u8,
    len: usize,
    path_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session_then_flush(|session| {
            let Some(request_id) = enqueue_open(session, data, len, path_ptr) else {
                return -4;
            };
            wait_for_open(session, request_id)
        })
    })
}

/// Enqueue an open and return the correlation id. Completion is reported by the
/// `DocumentOpened` event and confirmed with [`tw_take_open_result`].
///
/// 0 = enqueued, -1 = no session, -2 = null out pointer, -4 = worker unavailable.
#[no_mangle]
pub extern "C" fn tw_open_document_async(
    data: *const u8,
    len: usize,
    path_ptr: *const c_char,
    out_request_id: *mut u64,
) -> i32 {
    guard_ffi(|| {
        enqueue_async(out_request_id, |session| {
            enqueue_open(session, data, len, path_ptr)
        })
    })
}

/// Outcome of an open enqueued with `tw_open_document_async`. There is no
/// payload; the document is read through the usual snapshot exports.
///
/// 0 = opened, 1 = not ready yet, -1 = no session, -2 = the open failed
/// (details via `tw_get_last_error`), -3 = unknown request id.
#[no_mangle]
pub extern "C" fn tw_take_open_result(request_id: u64) -> i32 {
    guard_ffi(|| take_async_result(request_id, |_| {}))
}

fn enqueue_open(
    session: &Session,
    data: *const u8,
    len: usize,
    path_ptr: *const c_char,
) -> Option<u64> {
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
    let request_id = session.open_bytes_with_path(bytes, path_hint)?;
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = None;
    }
    Some(request_id)
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_paragraph_style(
    caret_run_id_ptr: *const c_char,
    style_name_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(style_name) = parse_cstr(style_name_ptr) else {
                return -2;
            };
            if style_name.is_empty() {
                return -2;
            }
            let Some(request_id) = session.apply_paragraph_style_at(caret_run_id, &style_name)
            else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_document_theme(theme_name_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(theme_name) = parse_cstr(theme_name_ptr) else {
                return -2;
            };
            if theme_name.is_empty() {
                return -2;
            }
            let Some(request_id) = session.apply_document_theme(&theme_name) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_apply_section_format_json(
    format_json_ptr: *const c_char,
    caret_run_id_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(json) = parse_cstr(format_json_ptr) else {
                return -2;
            };
            let format: SectionFormat = match serde_json::from_str(&json) {
                Ok(f) => f,
                Err(_) => return -3,
            };
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.apply_section_format_at(caret_run_id, format) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_get_section_format_json(
    caret_run_id_ptr: *const c_char,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let caret_run_id = parse_node_id(caret_run_id_ptr);
        let Some(json) = session.section_format_json_at(caret_run_id) else {
            return -3;
        };
        transfer_bytes_to_caller(json.into_bytes(), out_ptr, out_len);
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_section_break(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.insert_section_break_at(caret_run_id) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_ensure_header_footer(
    caret_run_id_ptr: *const c_char,
    is_header: i32,
    page_index: i32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let page = if page_index < 0 {
                None
            } else {
                Some(page_index as u32)
            };
            let Some(request_id) =
                session.ensure_header_footer_at(caret_run_id, is_header != 0, page)
            else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_set_even_and_odd_headers(enabled: i32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.set_even_and_odd_headers(enabled != 0) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_even_and_odd_headers_enabled() -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        i32::from(session.document().settings.even_and_odd_headers)
    })
}

#[no_mangle]
pub extern "C" fn tw_header_footer_seed_run(
    caret_run_id_ptr: *const c_char,
    is_header: i32,
    page_index: i32,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let caret_run_id = parse_node_id(caret_run_id_ptr);
        let page = if page_index < 0 {
            None
        } else {
            Some(page_index as u32)
        };
        let Some(run_id) =
            session.header_footer_seed_run_at(caret_run_id, is_header != 0, page)
        else {
            return -3;
        };
        transfer_bytes_to_caller(run_id.to_string().into_bytes(), out_ptr, out_len);
        0
    })
}

#[no_mangle]
pub extern "C" fn tw_header_footer_linked(
    caret_run_id_ptr: *const c_char,
    is_header: i32,
    page_index: i32,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let caret_run_id = parse_node_id(caret_run_id_ptr);
        let page = if page_index < 0 {
            None
        } else {
            Some(page_index as u32)
        };
        i32::from(session.header_footer_linked_at(
            caret_run_id,
            is_header != 0,
            page,
        ))
    })
}

#[no_mangle]
pub extern "C" fn tw_set_header_footer_link(
    caret_run_id_ptr: *const c_char,
    is_header: i32,
    page_index: i32,
    linked: i32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let page = if page_index < 0 {
                None
            } else {
                Some(page_index as u32)
            };
            let Some(request_id) = session.set_header_footer_link_at(
                caret_run_id,
                is_header != 0,
                page,
                linked != 0,
            ) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_field(
    run_id_ptr: *const c_char,
    offset: i32,
    field_type_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(run_id) = parse_node_id(run_id_ptr) else {
                return -3;
            };
            let Some(field_type) = parse_field_type(field_type_ptr) else {
                return -3;
            };
            let Some(request_id) = session.insert_field_at(
                run_id,
                offset.max(0) as usize,
                field_type,
            ) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
        })
    })
}

/// Promote (+1) or demote (−1) the list level at the caret paragraph (F05.S2).
///
/// Returns `0` when an edit was enqueued, `1` when the level is unchanged (at
/// min/max), `-4` when the paragraph is not in a list.
#[no_mangle]
pub extern "C" fn tw_adjust_list_level(caret_run_id_ptr: *const c_char, delta: i32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(outcome) = session.adjust_list_level_at(caret_run_id, delta) else {
                return -4;
            };
            let Some(request_id) = outcome else {
                return 1;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

/// Restart list numbering at the caret paragraph (F05.S3).
#[no_mangle]
pub extern "C" fn tw_restart_numbering(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.restart_numbering_at(caret_run_id) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

/// Continue list numbering from the running counter (F05.S3).
#[no_mangle]
pub extern "C" fn tw_continue_numbering(caret_run_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret_run_id = parse_node_id(caret_run_id_ptr);
            let Some(request_id) = session.continue_numbering_at(caret_run_id) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
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

fn parse_field_type(ptr: *const c_char) -> Option<FieldType> {
    let name = parse_cstr(ptr)?;
    match name.to_ascii_lowercase().as_str() {
        "page" => Some(FieldType::Page),
        "numpages" => Some(FieldType::NumPages),
        "date" => Some(FieldType::Date),
        "time" => Some(FieldType::Time),
        "tablesum" | "sum" => Some(FieldType::TableSumAbove),
        _ => None,
    }
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
                    Some(request_id) => finish_edit_enqueue(request_id),
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
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

/// JSON array of outline entries: `{ paragraph_id, level, text, run_id, page }`.
#[no_mangle]
pub extern "C" fn tw_get_document_outline(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        let guard = SESSION.lock();
        let Some(session) = guard.as_ref() else {
            return -1;
        };
        let Some(json) = session.document_outline_json() else {
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
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
        let Some(run_id) = parse_node_id(run_id_ptr) else {
            return -2;
        };
        if start >= end {
            return -3;
        }
        apply_command(Command::DeleteRange {
            run_id,
            start: start as usize,
            end: end as usize,
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
        let Some(start_run_id) = parse_node_id(start_run_id_ptr) else {
            return -2;
        };
        let Some(end_run_id) = parse_node_id(end_run_id_ptr) else {
            return -2;
        };
        apply_command(Command::DeleteDocRange {
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
        })
    })
}

/// Split the paragraph at `(run_id, offset)` — Word Enter / Return.
#[no_mangle]
pub extern "C" fn tw_apply_split_paragraph(run_id_ptr: *const c_char, offset: u32) -> i32 {
    guard_ffi(|| {
        let Some(run_id) = parse_node_id(run_id_ptr) else {
            return -2;
        };
        apply_command(Command::SplitParagraphAt {
            run_id,
            offset: offset as usize,
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_table(
    rows: u32,
    cols: u32,
    caret_run_id: *const std::os::raw::c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.insert_table_at(caret, rows, cols) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_delete_table_row(caret_run_id: *const std::os::raw::c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.delete_table_row_at(caret) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_delete_table_column(caret_run_id: *const std::os::raw::c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.delete_table_column_at(caret) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_merge_table_cells(caret_run_id: *const std::os::raw::c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.merge_table_cells_at(caret) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_split_table_cell(caret_run_id: *const std::os::raw::c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.split_table_cell_at(caret) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

/// Set table border at caret. `width` <= 0 clears the border (F09.S4).
#[no_mangle]
pub extern "C" fn tw_set_table_border(
    caret_run_id: *const std::os::raw::c_char,
    width: f32,
    color_r: u8,
    color_g: u8,
    color_b: u8,
    color_a: u8,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let border = if width > 0.0 {
                Some(tw_model::BorderSpec {
                    width,
                    color: tw_model::Color {
                        r: color_r,
                        g: color_g,
                        b: color_b,
                        a: color_a,
                    },
                })
            } else {
                None
            };
            let Some(request_id) = session.set_table_border_at(caret, border) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

/// Set cell shading at caret. Pass `color_r` < 0 to clear (F09.S4).
#[no_mangle]
pub extern "C" fn tw_set_table_cell_shading(
    caret_run_id: *const std::os::raw::c_char,
    color_r: i32,
    color_g: u8,
    color_b: u8,
    color_a: u8,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let background = if color_r < 0 {
                None
            } else {
                Some(tw_model::Color {
                    r: color_r as u8,
                    g: color_g,
                    b: color_b,
                    a: color_a,
                })
            };
            let Some(request_id) = session.set_table_cell_shading_at(caret, background) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_resize_table_column(
    caret_run_id: *const std::os::raw::c_char,
    width: f32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.resize_table_column_at(caret, width) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_autofit_table(caret_run_id: *const std::os::raw::c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.autofit_table_at(caret) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_sort_table_rows(
    caret_run_id: *const std::os::raw::c_char,
    ascending: bool,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.sort_table_rows_at(caret, ascending) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_nested_table(
    caret_run_id: *const std::os::raw::c_char,
    rows: u32,
    cols: u32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.insert_nested_table_at(caret, rows, cols) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_table_sum_field(caret_run_id: *const std::os::raw::c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let caret = parse_node_id(caret_run_id);
            let Some(request_id) = session.insert_table_sum_field_at(caret) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_shape(shape_type: i32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let kind = match shape_type {
                0 => tw_model::ShapeKind::Rectangle,
                1 => tw_model::ShapeKind::Line,
                2 => tw_model::ShapeKind::Ellipse,
                3 => tw_model::ShapeKind::TextBox,
                4 => tw_model::ShapeKind::WordArt,
                _ => tw_model::ShapeKind::Other,
            };
            let Some(request_id) = session.insert_shape(kind) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_text_box() -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(request_id) = session.insert_text_box() else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_word_art(text_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            if text_ptr.is_null() {
                return -3;
            }
            let text = unsafe { CStr::from_ptr(text_ptr) }
                .to_string_lossy()
                .into_owned();
            let Some(request_id) = session.insert_word_art(text) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_diagram(diagram_type: i32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let kind = tw_model::DiagramKind::from_i32(diagram_type);
            let Some(request_id) = session.insert_diagram_with_kind(kind) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_chart(chart_type: i32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let kind = tw_model::ChartKind::from_i32(chart_type);
            let Some(request_id) = session.insert_chart_with_kind(kind) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_image_bytes(
    data: *const u8,
    len: usize,
    mime_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            if data.is_null() || len == 0 {
                return -3;
            }
            let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
            let mime = if mime_ptr.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(mime_ptr) }
                    .to_string_lossy()
                    .into_owned()
            };
            let Some(request_id) = session.insert_image_bytes(bytes, mime) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_set_image_size(
    image_id_ptr: *const c_char,
    width: f32,
    height: f32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(image_id) = parse_node_id(image_id_ptr) else {
                return -2;
            };
            if width <= 0.0 || height <= 0.0 {
                return -3;
            }
            let Some(request_id) = session.set_image_size(image_id, width, height) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_replace_image_bytes(
    image_id_ptr: *const c_char,
    data: *const u8,
    len: usize,
    mime_ptr: *const c_char,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(image_id) = parse_node_id(image_id_ptr) else {
                return -2;
            };
            if data.is_null() || len == 0 {
                return -3;
            }
            let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
            let mime = if mime_ptr.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(mime_ptr) }
                    .to_string_lossy()
                    .into_owned()
            };
            let Some(request_id) = session.replace_image_bytes(image_id, bytes, mime) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_set_image_wrap(image_id_ptr: *const c_char, wrap: u8) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(image_id) = parse_node_id(image_id_ptr) else {
                return -2;
            };
            let Some(wrap) = text_wrap_from_u8(wrap) else {
                return -3;
            };
            let Some(request_id) = session.set_image_wrap(image_id, wrap) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_set_image_anchor(
    image_id_ptr: *const c_char,
    x: f32,
    y: f32,
    origin_x: u8,
    origin_y: u8,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(image_id) = parse_node_id(image_id_ptr) else {
                return -2;
            };
            let Some(origin_x) = anchor_origin_from_u8(origin_x) else {
                return -3;
            };
            let Some(origin_y) = anchor_origin_from_u8(origin_y) else {
                return -3;
            };
            let Some(request_id) = session.set_image_anchor(
                image_id,
                tw_model::ImageAnchor {
                    x,
                    y,
                    origin_x,
                    origin_y,
                },
            ) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

fn text_wrap_from_u8(value: u8) -> Option<tw_model::TextWrap> {
    match value {
        0 => Some(tw_model::TextWrap::Inline),
        1 => Some(tw_model::TextWrap::Square),
        2 => Some(tw_model::TextWrap::TopBottom),
        3 => Some(tw_model::TextWrap::Behind),
        4 => Some(tw_model::TextWrap::InFront),
        _ => None,
    }
}

fn anchor_origin_from_u8(value: u8) -> Option<tw_model::AnchorOrigin> {
    match value {
        0 => Some(tw_model::AnchorOrigin::Column),
        1 => Some(tw_model::AnchorOrigin::Page),
        2 => Some(tw_model::AnchorOrigin::Margin),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn tw_set_image_transform(
    image_id_ptr: *const c_char,
    rotation_deg: f32,
    crop_left: f32,
    crop_top: f32,
    crop_right: f32,
    crop_bottom: f32,
    opacity: f32,
) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(image_id) = parse_node_id(image_id_ptr) else {
                return -2;
            };
            let transform = tw_model::ImageTransform {
                rotation_deg,
                crop_left,
                crop_top,
                crop_right,
                crop_bottom,
                opacity,
            };
            let Some(request_id) = session.set_image_transform(image_id, transform) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_insert_image_caption(image_id_ptr: *const c_char) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(image_id) = parse_node_id(image_id_ptr) else {
                return -2;
            };
            let Some(request_id) = session.insert_image_caption(image_id) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_compress_image(image_id_ptr: *const c_char, quality: u8) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            let Some(image_id) = parse_node_id(image_id_ptr) else {
                return -2;
            };
            if quality == 0 {
                return -3;
            }
            let Some(request_id) = session.compress_image(image_id, quality) else {
                return -4;
            };
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
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
            finish_edit_enqueue(request_id)
        })
    })
}

#[no_mangle]
pub extern "C" fn tw_export_pdf(out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    guard_ffi(|| {
        with_session_then_flush(|session| {
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
        with_session_then_flush(|session| {
            let format_str = unsafe { CStr::from_ptr(format_ptr) }.to_string_lossy();
            let Some(request_id) = session.save_as(format_from_extension_str(&format_str)) else {
                return -4;
            };
            wait_for_document_saved(session, request_id, out_ptr, out_len)
        })
    })
}

/// Blocking wrapper over [`tw_spell_check_document_async`] +
/// [`tw_take_spell_check_result`].
#[no_mangle]
pub extern "C" fn tw_spell_check_document(
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        with_session_then_flush(|session| {
            let Some(request_id) = session.spell_check() else {
                return -4;
            };
            wait_for_spell_check(session, request_id, out_ptr, out_len)
        })
    })
}

/// Enqueue a spell check and return the correlation id.
///
/// 0 = enqueued, -1 = no session, -2 = null out pointer, -4 = worker unavailable.
#[no_mangle]
pub extern "C" fn tw_spell_check_document_async(out_request_id: *mut u64) -> i32 {
    guard_ffi(|| enqueue_async(out_request_id, |session| session.spell_check()))
}

/// Collect misspellings for a request enqueued with
/// `tw_spell_check_document_async`, newline separated, released with
/// `tw_free_buffer`. An empty buffer means no misspellings.
///
/// 0 = ready, 1 = not ready yet, -1 = no session, -2 = the check failed,
/// -3 = unknown request id.
#[no_mangle]
pub extern "C" fn tw_take_spell_check_result(
    request_id: u64,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    guard_ffi(|| {
        if out_ptr.is_null() || out_len.is_null() {
            return -2;
        }
        take_async_result(request_id, |bytes| {
            transfer_bytes_to_caller(bytes, out_ptr, out_len)
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

/// Whether `page` still carries pre-edit geometry awaiting background reflow.
/// A stale page must not be hit tested: the line map it holds describes content
/// that has since moved, so a caret placed from it lands somewhere unrelated.
///
/// 1 = page is stale (pending forward reflow, hit tests unreliable)
/// 0 = page is fresh
/// -1 = no session
#[no_mangle]
pub extern "C" fn tw_is_page_stale(page: u32) -> i32 {
    guard_ffi(|| {
        with_session(|session| {
            if session.is_page_stale(page) {
                1
            } else {
                0
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
        if session.is_page_stale(page) {
            return HIT_PAGE_STALE;
        }
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
        // Deliberately not gated on page staleness: this resolves the last run of
        // the document model, which a pending reflow leaves fully current, and
        // never reads the page's geometry. `page` is echoed back for caret display.
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

/// Last enqueued edit `request_id` (R1.4 async correlation). Zero before first edit.
#[no_mangle]
pub extern "C" fn tw_last_request_id() -> u64 {
    LAST_REQUEST_ID.load(Ordering::Relaxed)
}

/// Deliver any worker events that have arrived since the last call to the
/// callback registered with `tw_init`, and settle the matching async result
/// slots. A host that never blocks in one of the wrapper exports must call this
/// (on a timer or frame callback) for events to be observed at all.
///
/// Returns the number of events delivered, or -1 when there is no session.
#[no_mangle]
pub extern "C" fn tw_pump_events() -> i32 {
    guard_ffi(|| {
        // Collect under the session lock, deliver after releasing it, so the host
        // is free to call back into the engine from its callback.
        let collected = with_session(|session| {
            session.pump_events();
            0
        });
        if collected != 0 {
            return collected;
        }
        flush_event_outbox() as i32
    })
}

#[cfg(test)]
pub(crate) fn wait_for_document_edit(session: &Session, request_id: u64) -> i32 {
    match wait_for_request(session, request_id, Duration::from_millis(100), |event| {
        match event {
            BridgeEvent::DisplayListReady { .. } => Some(0),
            BridgeEvent::Error { .. } => Some(-2),
            _ => None,
        }
    }) {
        -3 => 0,
        code => code,
    }
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn tw_wait_for_layout() -> i32 {
    guard_ffi(|| {
        with_session_then_flush(|session| {
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
