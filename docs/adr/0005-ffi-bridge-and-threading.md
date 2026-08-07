# ADR-0005: FFI Bridge and Threading Model

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 1

## Context

The editor must achieve <10 ms typing latency and 60 FPS scrolling. Layout of a single page takes ~5 ms. File I/O for opening a 500-page document takes ~1–2 seconds. If all of this runs on the UI thread, the application will freeze during editing and document loading.

The FFI boundary between Flutter (Dart) and Rust must transfer display lists efficiently — a page display list is ~500 KB of draw commands and glyph atlas data.

## Decision

**Two-thread model with async command queue and double-buffered snapshots:**

1. **UI thread (Flutter):** Captures input, manages widgets, paints display lists. Never blocks on Rust operations.
2. **Worker thread (Rust):** Processes commands, mutates model, runs layout, builds display lists. Publishes snapshots via atomic swap.

Communication:
- UI → Rust: Command queue (`crossbeam-channel`), fire-and-forget
- Rust → UI: Snapshot notification callback, double-buffered display lists

Data transfer: flat byte buffers (zero-copy on native, copy on WASM).

Invalidation: page-granular — a keystroke re-layouts only the affected page.

## Consequences

**Positive:**
- UI thread never blocks — scrolling and cursor blink remain at 60 FPS during layout
- Typing latency bounded by single-page layout (~8 ms) not full-document layout
- Zero-copy display list transfer on native platforms
- Clean separation of concerns — UI code never touches the document model

**Negative:**
- Cursor position updates have one-frame latency (snapshot must arrive before repaint)
- Chain invalidation (page break changes) causes background re-layout of subsequent pages
- Double-buffering doubles display list memory (~1 MB per active page)
- WASM (Phase 1 web) cannot use threads — layout runs synchronously on main thread

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **Single-threaded (UI calls Rust synchronously)** | Layout blocks UI; 5 ms layout = dropped frames during typing |
| **Multiple worker threads (one per page)** | Over-engineered for Phase 1; page-level parallelism via rayon is sufficient for full-document layout |
| **Rust owns the UI thread (via native windowing)** | Loses Flutter's cross-platform UI; would need separate UI per platform |
| **Shared mutable state (Arc<Mutex<Document>>)** | Data races between UI reads and worker writes; complex locking |
| **MessagePack/Protobuf serialization for FFI** | Serialization overhead on every frame; flat bytes are faster |

## Implementation status (R1.4)

| Invariant | Status | Notes |
|-----------|--------|-------|
| UI thread never blocks on edit FFI | **Implemented** | Edit exports enqueue and return; Dart correlates via `tw_last_request_id` + `NativeEventRouter` |
| Worker → UI event callback | **Implemented** | `NativeCallable.listener` dispatches from worker thread to Dart isolate |
| Open/save blocking | Implemented | Open/save/spell check have non-blocking `tw_*_async` enqueues plus `tw_take_*` getters keyed by `request_id`; the 30 s blocking exports remain only as wrappers for callers that have not migrated |
| `tw_wait_for_layout` | Test-only | Gated behind `#[cfg(test)]` in `tw-ffi` |

## WASM Exception

Web platform (Phase 1): no `SharedArrayBuffer` requirement. Layout runs synchronously after each command. Acceptable because web is not the primary target and WASM threading is available in Phase 2.
