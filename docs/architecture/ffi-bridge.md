# FFI Bridge

The FFI bridge connects the Rust engine to the Flutter UI. It defines the boundary between the UI thread (Dart) and the worker thread (Rust), including data transport, threading model, and platform-specific entry points.

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
              │  (double-buffered)  │
              └─────────────────────┘
```

### Command Queue (UI → Rust)

Commands are sent asynchronously. The UI thread enqueues and returns immediately.

```rust
pub enum BridgeCommand {
    // Document lifecycle
    NewDocument,
    OpenDocument { path: String },
    SaveDocument { doc_id: DocId, path: String, format: SaveFormat },
    CloseDocument { doc_id: DocId },

    // Editing
    ApplyEdit { doc_id: DocId, command: Command },

    // Layout queries
    RequestLayout { doc_id: DocId, page: PageIndex },

    // Search
    Search { doc_id: DocId, query: String, options: SearchOptions },

    // Cursor/selection queries
    HitTest { doc_id: DocId, page: PageIndex, x: f32, y: f32 },
    PositionForOffset { doc_id: DocId, page: PageIndex, x: f32, y: f32 },

    // Image assets
    RequestImage { doc_id: DocId, image_id: ImageAssetId },
}
```

Queue implementation:

```rust
pub struct CommandQueue {
    sender: crossbeam_channel::Sender<BridgeCommand>,
    receiver: crossbeam_channel::Receiver<BridgeCommand>,
}
```

Flutter enqueues via FFI:

```dart
void applyEdit(int docId, Uint8List commandBytes) {
  _nativeApplyEdit(docId, commandBytes, commandBytes.length);
}
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

## Double-Buffered Snapshots

Display list snapshots use double buffering to avoid tearing:

```rust
pub struct SnapshotBuffer {
    front: AtomicPtr<DisplayListSnapshot>,  // UI reads this
    back: Mutex<DisplayListSnapshot>,       // Worker writes this
}

impl SnapshotBuffer {
    pub fn publish(&self, new_snapshot: DisplayListSnapshot) {
        let mut back = self.back.lock().unwrap();
        *back = new_snapshot;
        let old_front = self.front.swap(back as *const _ as *mut _, Ordering::Release);
        // old front becomes new back
    }

    pub fn read(&self) -> &DisplayListSnapshot {
        unsafe { &*self.front.load(Ordering::Acquire) }
    }
}
```

The UI thread always reads the front buffer. The worker thread writes to the back buffer and atomically swaps. No locks on the read path.

## Zero-Copy Data Transport

### Display List Transfer

The display list is a flat byte buffer. It crosses FFI as a pointer + length:

```rust
#[no_mangle]
pub extern "C" fn tw_get_display_list(
    doc_id: u32,
    page: u32,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
    out_version: *mut u64,
) -> i32 {
    let snapshot = session.get_display_list(doc_id, page);
    unsafe {
        *out_ptr = snapshot.display_list.as_ptr();
        *out_len = snapshot.display_list.len();
        *out_version = snapshot.version;
    }
    0
}
```

Flutter reads the buffer directly:

```dart
final displayListBytes = _nativeGetDisplayList(docId, pageIndex);
final snapshot = DisplayListSnapshot.fromBytes(displayListBytes);
```

The buffer is owned by Rust and valid until the next `publish()` for that page. Flutter must deserialize and copy any data it needs to retain before the next edit.

### Atlas Transfer

The glyph atlas is transferred once per atlas rebuild (not per frame):

```rust
#[no_mangle]
pub extern "C" fn tw_get_atlas(
    doc_id: u32,
    out_ptr: *mut *const u8,
    out_width: *mut u32,
    out_height: *mut u32,
) -> i32;
```

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

### Command Serialization

Edit commands are serialized as compact binary (not JSON) for minimal overhead:

```rust
// Command wire format
// u8 command_type
// ... command-specific fields (fixed-size where possible)
```

Example: `InsertText { run_id: [16 bytes UUID], offset: u32, text: [u32 len][utf8 bytes] }`

Total overhead for a single character insert: ~25 bytes.

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
- **flutter_rust_bridge** — codegen, async support, automatic type conversion
- **Manual C ABI + cbindgen** — full control, minimal dependencies

Recommendation: start with `flutter_rust_bridge` for velocity; migrate hot paths to manual FFI if profiling shows overhead.

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
| Display list snapshots | Rust (double buffer) | Until next publish for that page |
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
