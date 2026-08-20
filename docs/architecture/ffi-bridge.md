# FFI Bridge

The FFI bridge connects the Rust engine to the Flutter UI. It defines the boundary between the UI thread (Dart) and the worker thread (Rust), including data transport, threading model, and platform-specific entry points.

> **The host must call `tw_pump_events`, or it will receive no events at all.**
>
> Registering a callback with `tw_init` is not sufficient. The callback fires only
> while the FFI layer drains the worker's event channel, and nothing drains it
> spontaneously — `tw_dispatch` and the other edit exports enqueue a command and
> return, and the worker pushes its completion into a channel that sits there until
> somebody reads it.
>
> An unpumped host sees every correlated wait run to its full 30 s timeout, and the
> symptom surfaces somewhere unrelated to the missing pump. Worse, it can look like
> it works: any blocking export on the stack drains the channel as a side effect, so
> the bug only appears once the last blocking call is removed.
>
> Call `tw_pump_events` on a timer or frame callback for the lifetime of the
> session. Dart's `NativeEventRouter` runs it adaptively: 2 ms while a correlated
> request is outstanding, 16 ms when idle so background reflow repaints still
> arrive. When a correlated wait times out having never been pumped, the engine
> prints a one-time warning to stderr naming this cause.
>
> See [Event callback contract](#event-callback-contract) for the two other rules
> that go with it: read `request_id` from the by-value argument rather than the
> payload, and treat the payload as valid only for the duration of the call.

## Design Goals

1. **Zero-copy data transfer.** Display lists and atlas data cross the boundary as flat byte buffers, not serialized object graphs.
2. **UI thread never blocks.** All expensive work (layout, file I/O, search) runs on the Rust worker thread.
3. **One API, two transports.** The same logical API is exposed via C ABI (native) and WASM bindings (web).
4. **Batch operations.** Multiple commands can be queued before the worker thread processes them.

## Threading Model

```
┌─────────────────────┐         ┌──────────────────────┐
│   Flutter UI Thread  │         │   Rust Worker Thread  │
│                      │         │                       │
│  Input capture       │         │  Command queue        │
│  Widget tree         │  cmd    │  Document model       │
│  CustomPainter       │ ──────► │  Layout engine        │
│  Animations          │         │  Display list builder │
│  Overlay widgets     │  snap   │  File I/O             │
│                      │ ◄────── │  Search               │
└─────────────────────┘         └──────────────────────┘
         │                                  │
         │         Shared Memory            │
         │    ┌─────────────────────┐       │
         └───►│  Snapshot Buffers   │◄──────┘
              │  (RwLock front/back)│
              └─────────────────────┘
```

### Command Queue (UI → Rust)

Commands are sent asynchronously. The UI thread enqueues and returns immediately.

```rust
pub enum BridgeCommand {
    // Document lifecycle
    NewDocument,
    OpenDocument { data: Vec<u8>, path_hint: Option<String> },
    SaveDocument,
    SaveDocumentAs { format: DetectedFormat },
    SpellCheckDocument,
    CloseDocument, // planned

    // Settings (not yet modeled as Command)
    ToggleTrackChanges { enabled: bool },

    // All document edits — single mutation path (ADR-0007, R2.3)
    ApplyEdit { command: Command },

    // Session undo stack (not Command enum)
    Undo,
    Redo,

    // View / navigation
    SetCurrentPage { page: u32 },

    // Import + paste (format crate returns Document; tw-edit mutates)
    PasteHtml { run_id: NodeId, offset: usize, html: Vec<u8> },
    PasteDocx { run_id: NodeId, offset: usize, bytes: Vec<u8> },

    // Export
    ExportPdf,

    Shutdown,
}
```

Ribbon helpers (`apply_heading1`, `insert_table`, …) build a `Command` in `tw-edit::command_builders` and enqueue `ApplyEdit` — they do not add new `BridgeCommand` variants.

Queue implementation:

```rust
pub struct CommandQueue {
    sender: crossbeam_channel::Sender<BridgeCommand>,
    receiver: crossbeam_channel::Receiver<BridgeCommand>,
}
```

Flutter enqueues edits via JSON dispatch (R2.5):

```dart
final bytes = CommandCodec.encode(CommandCodec.insertText(
  runId: runId,
  offset: offset,
  text: text,
));
engine.dispatchCommandBytes(bytes);
// correlate with NativeEventRouter.waitFor(engine.lastRequestId())
```

### Snapshot Notification (Rust → UI)

When the worker thread finishes processing commands and re-layouting, it publishes new snapshots and notifies the UI thread.

```rust
pub struct SnapshotNotifier {
    callback: Box<dyn Fn(SnapshotEvent) + Send>,
}

pub enum SnapshotEvent {
    DisplayListReady { doc_id: DocId, page: PageIndex, version: u64 },
    LayoutProgress { doc_id: DocId, pages_done: u32, pages_total: u32 },
    DocumentOpened { doc_id: DocId, page_count: u32 },
    DocumentSaved { doc_id: DocId, path: String },
    Error { doc_id: DocId, message: String },
    SearchResults { doc_id: DocId, results: Vec<SearchResult> },
    ImageReady { doc_id: DocId, image_id: ImageAssetId, data: Vec<u8> },
}
```

On native platforms, the callback is a C function pointer registered at init time. On WASM, it is a JS callback via `wasm-bindgen`.

Flutter receives notifications and schedules repaints:

```dart
void _onSnapshotEvent(SnapshotEvent event) {
  switch (event) {
    case DisplayListReady(:final page, :final version):
      setState(() => _pageVersions[page] = version);
    case DocumentOpened(:final pageCount):
      setState(() => _pageCount = pageCount);
    // ...
  }
}
```

## RwLock Snapshot Buffer

Display list snapshots use a front/back pair protected by `parking_lot::RwLock` (implemented in `tw-core/src/snapshot.rs`):

```rust
pub struct SnapshotBuffer {
    front: RwLock<PageSnapshot>,  // UI reads this (clone-on-read)
    back: RwLock<PageSnapshot>,  // Worker writes here first
}

impl SnapshotBuffer {
    pub fn publish(&self, snapshot: PageSnapshot) {
        let mut back = self.back.write();
        *back = snapshot;
        let mut front = self.front.write();
        std::mem::swap(&mut *back, &mut *front);
    }

    pub fn read(&self) -> PageSnapshot {
        self.front.read().clone()
    }
}
```

The worker writes to the back buffer, then swaps front and back under write locks. The UI thread clones the front snapshot on read — a short lock, not a lock-free atomic pointer swap.

### Async edit completion (R1.4)

Edit FFI exports enqueue a command and return immediately (`0` on success, negative on error). The enqueued `request_id` is available via `tw_last_request_id()`. Dart registers interest in `NativeEventRouter.waitFor(requestId)` **before or after** enqueue; early events are buffered if the worker responds before registration.

`tw_wait_for_layout` is removed from production builds (`#[cfg(test)]` only).

### Async document operations (P1-7)

Open, save, and spell check are split into an enqueue that hands back the
`request_id` immediately and a getter keyed by that id:

```rust
pub extern "C" fn tw_open_document_async(data: *const u8, len: usize, path_ptr: *const c_char, out_request_id: *mut u64) -> i32;
pub extern "C" fn tw_take_open_result(request_id: u64) -> i32;
pub extern "C" fn tw_save_document_async(out_request_id: *mut u64) -> i32;
pub extern "C" fn tw_save_document_as_async(format_ptr: *const c_char, out_request_id: *mut u64) -> i32;
pub extern "C" fn tw_take_saved_document(request_id: u64, out_ptr: *mut *const u8, out_len: *mut usize) -> i32;
pub extern "C" fn tw_spell_check_document_async(out_request_id: *mut u64) -> i32;
pub extern "C" fn tw_take_spell_check_result(request_id: u64, out_ptr: *mut *const u8, out_len: *mut usize) -> i32;
pub extern "C" fn tw_pump_events() -> i32;
```

Enqueue returns `0` on success, `-1` with no session, `-2` for a null out pointer,
`-4` when the worker channel is gone. Getters return `0` when the payload is
written, `1` while the request is still in flight, `-2` when the operation failed
(message via `tw_get_last_error`), `-3` for an id that was never enqueued, was
already collected, or aged out.

Results are held in a bounded, oldest-first-evicted table of 16 slots, so a
completion survives until the getter runs but an abandoned request cannot leak.
Payload buffers are released with `tw_free_buffer`.

Worker events reach the Dart callback only while the FFI layer drains the event
channel. A host that never blocks must call `tw_pump_events` (timer or frame
callback) to deliver events and settle result slots; the getters pump internally
as well, so a poll-only host also makes progress.

`tw_open_document_with_path`, `tw_save_document`, `tw_save_document_as`, and
`tw_spell_check_document` remain as blocking wrappers over the same worker path
(correlated wait, up to 30 s). Do not pump from another isolate while one of them
is parked: the wrapper and the pump both consume from the same correlation buffer.

### Event callback contract

```c
typedef void (*tw_event_callback)(uint32_t event_type,
                                  uint64_t request_id,
                                  const uint8_t* payload,
                                  size_t payload_len);
```

**`event_type` and `request_id` are passed by value and are the authoritative
copy.** A host must never need to read `payload` in order to correlate a request.

`payload` is valid **only for the duration of the call**. It points at a stack
buffer in the frame that invoked the callback, so a host that defers work to a
later turn of its own event loop must copy the bytes first. Today it carries the
same two scalars in the 12-byte wire encoding below; it exists so future events can
carry inline data without another ABI break.

This is not a theoretical hazard. The original signature passed the correlation id
only through `payload`, and Dart registered the callback with
`NativeCallable.listener`, which defers to a later event-loop turn — by which point
the frame was gone. `event_type` (by value) arrived correctly while the buffer's
own first four bytes decoded to garbage and later events read back as recycled
zeros with request id 0. No correlation id was ever delivered correctly, so every
`awaitEditCompletion` waited on an id that could not arrive, which is where the 30 s
select-all timeout came from. It also made `nativeFfiEventsAvailable()` always
false, silently skipping the entire real-FFI integration suite. Dart now uses
`NativeCallable.isolateLocal` and asserts that the payload's event type matches the
by-value one, and the by-value `request_id` makes correctness independent of when
the host chooses to run the callback.

**The callback never runs with an engine lock held.** Events are collected under
`SESSION` and forwarded after it is released, so a host is free to call back into
any export from its callback — including `tw_take_saved_document` for the very
result the event announces. `SESSION` is a non-reentrant `parking_lot` mutex, so
the earlier arrangement would have deadlocked such a host outright
(`crates/tw-ffi/tests/r1_callback_reentrancy.rs`).

### Event wire format and correlation guarantees

The `payload` buffer is exactly 12 bytes: LE `u32` event type followed by LE `u64` `request_id`. Any future extension appends after byte 12; the first 12 bytes keep this meaning.

The worker's `EventPublisher` coalesces redundant `DisplayListReady` events for the same page so a burst of keystrokes does not schedule a repaint per character. Coalescing drops the *event*, never the *correlation*: the `request_id` of every superseded event is retained in a compact queue and re-emitted as a `DisplayListReady` carrying the latest page and version as soon as the channel has room. A caller awaiting an enqueued edit therefore always receives a completion, even when the Dart side falls far behind the worker.

Two `request_id` values are reserved and never handed out to callers:

| Constant | Value | Meaning |
| --- | --- | --- |
| `STARTUP_REQUEST_ID` | `0` | Worker's initial `DocumentOpened` |
| `BACKGROUND_REQUEST_ID` | `u64::MAX` | Repaint produced by background forward relayout |

`BACKGROUND_REQUEST_ID` events are repaint hints only. They tell the UI that pages it may already be displaying have been re-laid-out; they never satisfy a pending edit correlation.

### Stale pages and hit testing

While a background forward relayout is owed, pages past the reflow frontier still
hold pre-edit geometry. Hit testing them would resolve a caret against content that
has since moved, so the engine refuses:

```rust
/// 1 = page is stale (pending forward reflow, hit tests unreliable)
/// 0 = page is fresh
/// -1 = no session
pub extern "C" fn tw_is_page_stale(page: u32) -> i32;
```

`tw_hit_test` returns `-4` for a stale page and keeps `-2` for a genuine miss on a
page that is current. Treating the two the same is a bug: a miss means "nothing
here, place the caret accordingly", while stale means "ask again after the next
repaint". Callers should consult `tw_is_page_stale` before falling back to any
empty-page caret placement.

Staleness gates *geometry* queries only. A pending reflow leaves the document model
fully current — the edit has already been applied — so anything that reads the model
and ignores per-page geometry must still answer. `tw_document_tail_hit` resolves the
last run of the document and only echoes `page` back for caret display, so it is
deliberately **not** gated on staleness; select-all depends on it succeeding during
catch-up. Apply the same rule to anything added later: gate it only if it reads
`line_maps`.

The stale set is `[frontier, page_count)`, where the frontier is the first page no
pass has rebuilt since the edit that opened the window. It advances as background
chunks land and is not reset by a later edit that converges, so pages already
caught up stay usable. On a 48-page document a single large front-of-document
insert marks 44 pages stale; the midpoint clears at ~215 ms and the last page at
~460 ms in release, i.e. roughly 10 ms per page of catch-up. Edits that do not
cascade past the synchronous window mark nothing stale at all.

## Zero-Copy Data Transport

### Native page DL + atlas (B1/B2)

`tw_get_page_display_list` and `tw_get_atlas` transfer **ownership** of a Rust-allocated buffer to Dart. Dart adopts the pointer with `ExternalTypedData` and a `NativeFinalizer` that calls `tw_free_buffer` on GC — no `asTypedList().sublist(0)` copy on the hot path.

At the FFI edge, `Arc::try_unwrap` moves uniquely held snapshot bytes; shared unchanged pages clone once (unavoidable while the worker retains the snapshot).

Debug/test counters: `tw_reset_transfer_stats()` / `tw_get_transfer_stats()`.

### Image-by-id (B3, wire v9)

Page display lists (`DISPLAY_LIST_VERSION = 9`) omit embedded image payloads. Hosts fetch bytes once via `tw_get_image_asset(asset_id)`; Flutter wires this through `DocumentEngine.fetchImageAssetBytes` and `DisplayListSnapshot.decodeImages(resolveAsset: …)`.

### Display List Transfer

The display list is a flat byte buffer. It crosses FFI as a pointer + length owned by the caller after `tw_get_page_display_list` returns.

Flutter reads the buffer via `adoptFfiBuffer` (native) or Transferable cache (web worker).

### Atlas Transfer

The glyph atlas is transferred once per atlas rebuild (not per frame):

```rust
#[no_mangle]
pub extern "C" fn tw_get_atlas(
    out_generation: *mut u64,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
    out_width: *mut u32,
    out_height: *mut u32,
) -> i32;

/// Generation only, no pixel copy: `0` on success, `-1` no session, `-2` null out.
#[no_mangle]
pub extern "C" fn tw_get_atlas_generation(out_generation: *mut u64) -> i32;
```

Check `tw_get_atlas_generation` first and skip `tw_get_atlas` when unchanged. When fetched, atlas pixels use the same owned-buffer contract as page DL.

Flutter creates a `ui.Image` from the RGBA data:

```dart
Future<ui.Image> _createAtlasImage(Uint8List rgba, int width, int height) {
  final completer = Completer<ui.Image>();
  ui.decodeImageFromPixels(
    rgba, width, height, ui.PixelFormat.rgba8888,
    completer.complete,
  );
  return completer.future;
}
```

### Command Serialization (R2.5)

Document edits use JSON-serialized [`Command`](../../crates/tw-edit/src/command.rs) values dispatched through a single FFI entry point:

```rust
#[no_mangle]
pub extern "C" fn tw_dispatch(command_ptr: *const u8, command_len: usize) -> i32;
```

Example wire payload:

```json
{"type":"InsertText","run_id":"00000000-0000-0000-0000-000000000004","offset":3,"text":"hi"}
```

Dart builds commands via `CommandCodec` and enqueues with `NativeEngine.dispatchCommand()`. Character and paragraph format patches use open `Map<String, dynamic>` helpers in `format_codec.dart` — new Rust `CharFormat` fields require no Dart typedef/codegen.

Legacy per-operation edit exports (`tw_apply_insert_text`, `tw_apply_char_format`, …) remain as thin ABI-compatible wrappers that call the same internal `apply_command` path.

### Stable native ABI (R2.5)

| Category | Exports |
|----------|---------|
| Lifecycle | `tw_init`, `tw_shutdown`, `tw_new_document`, `tw_open_document`, `tw_open_document_with_path`, `tw_save_document`, `tw_save_document_as`, `tw_export_pdf` |
| Lifecycle (async) | `tw_open_document_async`, `tw_save_document_async`, `tw_save_document_as_async`, `tw_spell_check_document_async`, `tw_take_open_result`, `tw_take_saved_document`, `tw_take_spell_check_result` |
| Edits | `tw_dispatch` (primary), legacy `tw_apply_*` wrappers |
| Session | `tw_undo`, `tw_redo`, `tw_set_track_changes`, `tw_set_current_page` |
| Paste | `tw_apply_paste_html`, `tw_apply_paste_docx` |
| Display | `tw_get_display_list`, `tw_get_page_display_list`, `tw_get_atlas`, `tw_get_atlas_generation` |
| Query JSON | `tw_get_caret_format`, `tw_get_document_text`, `tw_get_text_range`, `tw_hit_test`, `tw_is_page_stale`, … |
| Correlation | `tw_last_request_id`, `tw_pump_events`, `tw_free_buffer`, `tw_get_last_error` |

Query/format APIs return JSON or flat bytes; only display lists and atlas cross as opaque buffers.

### Legacy binary command sketch (superseded)

The original compact binary wire format is superseded by JSON dispatch for extensibility:

```rust
// u8 command_type
// ... command-specific fields (fixed-size where possible)
```

Example: `InsertText { run_id: [16 bytes UUID], offset: u32, text: [u32 len][utf8 bytes] }`

Total overhead for a single character insert: ~25 bytes (binary) vs ~120 bytes (JSON). JSON chosen for R2.5 exit criteria (zero Dart codegen on new fields).

## Page-Granular Invalidation

The critical performance optimization. When the user types a character:

```
1. UI thread: enqueue InsertText command (~1 μs)
2. Worker thread: apply to model (~1 ms)
3. Worker thread: find page containing affected run (~0.1 ms)
4. Worker thread: re-layout that page (~5 ms)
5. Worker thread: build display list (~1 ms)
6. Worker thread: publish snapshot, notify UI (~0.1 ms)
7. UI thread: repaint one page (~3 ms)
```

Total perceived latency: ~10 ms (within budget).

If the edit causes a page break change (paragraph grows past page boundary), chain invalidation re-layouts subsequent pages in the background. The UI shows the current page immediately; subsequent pages update as they are re-laid-out.

## Platform Entry Points

### Native (tw-ffi)

Uses C ABI with `extern "C"` functions. Dart accesses via `dart:ffi`:

```dart
import 'dart:ffi';

final lib = DynamicLibrary.open('libtw_ffi.so');  // platform-specific

typedef ApplyEditNative = Int32 Function(Uint32 docId, Pointer<Uint8> cmd, IntPtr cmdLen);
typedef ApplyEditDart = int Function(int docId, Pointer<Uint8> cmd, int cmdLen);

final applyEdit = lib.lookupFunction<ApplyEditNative, ApplyEditDart>('tw_apply_edit');
```

Technology options (decide in Phase 1 implementation):
- **JSON command dispatch (R2.5, chosen)** — extensible `Command` serde, dynamic Dart format maps, no codegen
- **flutter_rust_bridge** — deferred; may wrap query APIs later (R2.6+)
- **Manual C ABI + cbindgen** — display list / atlas hot paths

Recommendation: keep JSON dispatch for edits; add FRB only if query surface outgrows hand-written JSON parsers.

### Web (tw-wasm)

Compiled to `wasm32-unknown-unknown` with `wasm-bindgen`:

```rust
#[wasm_bindgen]
pub struct TwSession {
    inner: Session,
}

#[wasm_bindgen]
impl TwSession {
    pub fn new() -> TwSession { ... }
    pub fn apply_edit(&mut self, doc_id: u32, command_bytes: &[u8]) -> Result<(), JsValue> { ... }
    pub fn get_display_list(&self, doc_id: u32, page: u32) -> Vec<u8> { ... }
}
```

WASM-specific considerations:
- No threads in WASM (Phase 1 web) — layout runs synchronously on the main thread
- WASM threads (SharedArrayBuffer) available in Phase 2 for web worker layout
- Atlas and display list data copied (not shared memory) unless SharedArrayBuffer is available
- File I/O uses browser download/upload APIs, not filesystem

### Mobile (tw-ffi via JNI/FFI)

Same C ABI as desktop. Platform-specific considerations:

| Platform | Library | Loading |
|----------|---------|---------|
| Android | `libtw_ffi.so` | `DynamicLibrary.open('libtw_ffi.so')` |
| iOS | `libtw_ffi.a` (static) | `DynamicLibrary.process()` |

## Initialization

```rust
#[no_mangle]
pub extern "C" fn tw_init(config: *const InitConfig) -> i32 {
    // 1. Initialize font database (scan system fonts)
    // 2. Start worker thread
    // 3. Register snapshot callback
    // 4. Return 0 on success
}

#[no_mangle]
pub extern "C" fn tw_shutdown() {
    // 1. Drain command queue
    // 2. Stop worker thread
    // 3. Release resources
}
```

Flutter calls `tw_init()` during app startup and `tw_shutdown()` on app exit.

## Error Handling

Errors cross the FFI boundary as event notifications, not exceptions:

```rust
pub enum BridgeError {
    DocumentNotFound { doc_id: DocId },
    InvalidCommand { reason: String },
    FileNotFound { path: String },
    ParseError { format: String, detail: String },
    OutOfMemory,
    InternalError { message: String },
}
```

Flutter displays errors via snackbar/dialog based on the error event.

## Memory Management

| Resource | Owner | Lifetime |
|----------|-------|----------|
| Document model | Rust (worker thread) | Until document closed |
| Display list snapshots | Rust (RwLock front/back) | Until next publish for that page |
| Glyph atlas | Rust (shared Arc) | Until atlas rebuild |
| Image assets | Rust (cache) + Flutter (ui.Image) | Until document closed |
| Command bytes | Flutter (sent) | Freed by Flutter after enqueue |
| Font data | Rust (fontdb) | Application lifetime |

Flutter must not hold references to Rust-owned memory across edit operations. Display list bytes are copied/deserialized on receipt; the Rust buffer may be overwritten on the next edit.

## WASM Entry (Web Platform)

```javascript
import init, { TwSession } from './tw_wasm.js';

await init();
const session = TwSession.new();
session.open_document(docxBytes);
const displayList = session.get_display_list(0, 0);
// render displayList in Flutter web
```

WASM module size budget: <5 MB compressed (excluding font data). Font data loaded separately at runtime.

## Debugging

Development builds expose additional FFI functions:

```rust
#[cfg(debug_assertions)]
#[no_mangle]
pub extern "C" fn tw_get_layout_debug(
    doc_id: u32, page: u32, debug_type: u32,
    out_ptr: *mut *const u8, out_len: *mut usize,
) -> i32;
// Returns layout boxes, line maps, or atlas visualization as JSON
```

These are stripped in release builds.
